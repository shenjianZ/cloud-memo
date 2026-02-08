mod config;
mod db;
mod handlers;
mod middleware;
mod models;
mod services;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use clap::Parser;
use services::token_blacklist::TokenBlacklist;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// 命令行参数
#[derive(clap::Parser)]
#[command(name = "note-sync-server")]
#[command(about = "Note sync server", long_about = None)]
struct CliArgs {
    /// 指定配置文件路径
    #[arg(short, long)]
    config: Option<String>,

    /// 指定运行环境 (development/production)
    #[arg(short = 'e', long)]
    env: Option<String>,
}

/// 应用状态，包含数据库连接池和 Token 黑名单
#[derive(Clone)]
pub struct AppState {
    pub pool: db::DbPool,
    pub token_blacklist: Arc<TokenBlacklist>,
    pub config: config::AppConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 解析命令行参数
    let args = CliArgs::parse();

    // 如果通过命令行指定了环境，设置环境变量
    if let Some(env) = args.env {
        std::env::set_var("CLOUDMEMO_ENV", env);
    }

    // 加载配置
    let config = config::AppConfig::load(args.config).expect("Failed to load configuration");

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
    let redis_url = {
        let url = &config.redis.url;

        // 如果 URL 中已经包含密码（:password@），直接使用
        if url.contains("@") {
            url.clone()
        }
        // 如果有 password 字段，添加到 URL
        else if let Some(password) = &config.redis.password {
            if password.is_empty() {
                url.clone()
            } else if url.starts_with("redis://") {
                format!("redis://:{}{}", password, &url[8..])
            } else {
                url.clone()
            }
        }
        // 否则直接使用 URL
        else {
            url.clone()
        }
    };

    // 初始化 Redis Token 黑名单
    let token_blacklist = Arc::new(
        TokenBlacklist::new(&redis_url)
            .await
            .expect("Failed to connect to Redis"),
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
        .route(
            "/auth/delete",
            axum::routing::delete(handlers::auth::delete_account),
        )
        // 同步端点
        .route("/sync", post(handlers::sync::sync))
        // 同步历史端点
        .route("/sync/history", get(handlers::history::get_history))
        .route(
            "/sync/history",
            axum::routing::delete(handlers::history::clear_history),
        )
        // 用户资料端点
        .route("/profile/:user_id", get(handlers::profile::get_profile))
        .route(
            "/profile/:user_id",
            axum::routing::patch(handlers::profile::update_profile),
        )
        .route("/profile/sync", post(handlers::profile::sync_profile))
        // 笔记端点
        .route(
            "/notes/:id/snapshots",
            post(handlers::notes::create_snapshot),
        )
        .route("/notes/:id/snapshots", get(handlers::notes::list_snapshots))
        // 文件夹端点
        .route("/folders", get(handlers::folders::list_folders))
        .route("/folders", post(handlers::folders::create_folder))
        // 工作空间端点
        .route("/workspaces", get(handlers::workspaces::list_workspaces))
        .route("/workspaces", post(handlers::workspaces::create_workspace))
        .route(
            "/workspaces/:id",
            axum::routing::patch(handlers::workspaces::update_workspace),
        )
        .route(
            "/workspaces/:id",
            axum::routing::delete(handlers::workspaces::delete_workspace),
        )
        .route(
            "/workspaces/:id/default",
            post(handlers::workspaces::set_default_workspace),
        )
        // 设备管理端点
        .route("/devices", get(handlers::devices::list_devices))
        .route(
            "/devices/:id",
            axum::routing::delete(handlers::devices::revoke_device),
        )
        .route(
            "/devices/:id/heartbeat",
            post(handlers::devices::device_heartbeat),
        )
        // JWT 认证中间件（仅应用于受保护路由）
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth_middleware,
        ));

    // ========== 合并路由 ==========
    let app = public_routes
        .merge(protected_routes)
        // CORS（应用于所有路由）
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        // 自定义日志中间件（应用于所有路由）
        .layer(axum::middleware::from_fn(
            middleware::logging::request_logging_middleware,
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
