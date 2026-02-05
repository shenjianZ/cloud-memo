mod config;
mod models;
mod db;
mod handlers;
mod middleware;
mod services;

use axum::{
    Router,
    routing::{get, post},
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;
use services::token_blacklist::TokenBlacklist;

/// 应用状态，包含数据库连接池和 Token 黑名单
#[derive(Clone)]
pub struct AppState {
    pub pool: db::DbPool,
    pub token_blacklist: Arc<TokenBlacklist>,
    pub config: config::AppConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载配置
    let config = config::AppConfig::load()
        .expect("Failed to load configuration");

    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "note_sync_server=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Configuration loaded successfully");

    // 初始化数据库连接池
    let pool = db::create_pool(&config.database).await?;

    // 构建 Redis 连接字符串（支持可选密码）
    let redis_url = if let Some(password) = &config.redis.password {
        // 有密码：redis://:password@host:port
        let url = &config.redis.url;
        if url.starts_with("redis://") {
            format!("redis://:{}{}", password, &url[8..])
        } else {
            config.redis.url.clone()
        }
    } else {
        config.redis.url.clone()
    };

    // 初始化 Redis Token 黑名单
    let token_blacklist = Arc::new(
        TokenBlacklist::new(&redis_url)
            .await
            .expect("Failed to connect to Redis")
    );
    tracing::info!("Connected to Redis at {}", config.redis.url);

    // 创建应用状态
    let app_state = AppState {
        pool: pool.clone(),
        token_blacklist,
        config: config.clone(),
    };

    // ========== 公开路由（无需认证） ==========
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh)); // refresh token（公开，需要 refresh_token）

    // ========== 受保护路由（需要认证） ==========
    let protected_routes = Router::new()
        .route("/auth/logout", post(handlers::auth::logout))
        // 同步端点
        .route("/sync/push", post(handlers::sync::push))
        .route("/sync/pull", post(handlers::sync::pull))
        // 同步历史端点
        .route("/sync/history", get(handlers::history::get_history))
        .route("/sync/history", axum::routing::delete(handlers::history::clear_history))
        // 用户资料端点
        .route("/profile/:user_id", get(handlers::profile::get_profile))
        .route("/profile/:user_id", axum::routing::patch(handlers::profile::update_profile))
        .route("/profile/sync", post(handlers::profile::sync_profile))
        // 笔记端点
        .route("/notes/:id/snapshots", post(handlers::notes::create_snapshot))
        .route("/notes/:id/snapshots", get(handlers::notes::list_snapshots))
        // 文件夹端点
        .route("/folders", get(handlers::folders::list_folders))
        .route("/folders", post(handlers::folders::create_folder))
        // 设备管理端点
        .route("/devices", get(handlers::devices::list_devices))
        .route("/devices/register", post(handlers::devices::register_device))
        .route("/devices/:id", axum::routing::delete(handlers::devices::revoke_device))
        .route("/devices/:id/heartbeat", post(handlers::devices::device_heartbeat))
        // JWT 认证中间件（仅应用于受保护路由）
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth_middleware,
        ));

    // ========== 合并路由 ==========
    let app = public_routes
        .merge(protected_routes)
        // CORS（应用于所有路由）
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        // 自定义日志中间件（应用于所有路由）
        .layer(axum::middleware::from_fn(
            middleware::logging::request_logging_middleware
        ))
        // 应用状态
        .with_state(app_state);

    // 启动服务器
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

// 健康检查端点
async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    match state.pool.acquire().await {
        Ok(_) => (StatusCode::OK, "OK"),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "Database unavailable"),
    }
}

