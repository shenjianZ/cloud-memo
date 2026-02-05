use axum::{Json, extract::{State, Extension}, http::StatusCode, response::{IntoResponse, Response}};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::AppState;
use crate::services::auth_service::AuthService;
use crate::middleware::logging::{RequestId, log_info};
use std::fmt;

/// 错误响应结构
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// 自定义 Debug 实现，隐藏密码
impl fmt::Debug for LoginRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoginRequest")
            .field("email", &self.email)
            .field("password", &"***")
            .finish()
    }
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

// 自定义 Debug 实现，隐藏密码
impl fmt::Debug for RegisterRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegisterRequest")
            .field("email", &self.email)
            .field("password", &"***")
            .finish()
    }
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

// 自定义 Debug 实现，隐藏 token
impl fmt::Debug for AuthResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let token_preview = format!("{}...", &self.token[..std::cmp::min(20, self.token.len())]);
        f.debug_struct("AuthResponse")
            .field("user_id", &self.user_id)
            .field("email", &self.email)
            .field("token", &token_preview)
            .finish()
    }
}

pub async fn register(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // 第2条日志：请求参数
    log_info(&request_id, "注册请求参数", &payload);

    let service = AuthService::new(state.pool);

    match service.register(&payload.email, &payload.password).await {
        Ok(_user) => {
            // 注册成功，直接生成 token（使用默认 device_id）
            match service.login(&payload.email, &payload.password, Some("default".to_string())).await {
                Ok((user, token, refresh_token)) => {
                    let response = AuthResponse {
                        token: token.clone(),
                        refresh_token,
                        user_id: user.id.clone(),
                        email: user.email.clone(),
                    };

                    // 第2条日志：响应内容
                    log_info(&request_id, "注册成功，返回用户信息", &response);

                    Json(response).into_response()
                }
                Err(e) => {
                    log_info(&request_id, "生成 token 失败", &e.to_string());
                    let error_response = ErrorResponse {
                        error: e.to_string(),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
                }
            }
        }
        Err(e) => {
            log_info(&request_id, "注册失败", &e.to_string());
            // 返回错误消息
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::CONFLICT, Json(error_response)).into_response()
        }
    }
}

pub async fn login(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // 第2条日志：请求参数
    log_info(&request_id, "登录请求参数", &payload);

    let service = AuthService::new(state.pool);

    match service.login(&payload.email, &payload.password, Some("default".to_string())).await {
        Ok((user, token, refresh_token)) => {
            let response = AuthResponse {
                token,
                refresh_token,
                user_id: user.id,
                email: user.email,
            };

            // 第2条日志：响应内容
            log_info(&request_id, "登录成功，返回用户信息", &response);

            Json(response).into_response()
        }
        Err(e) => {
            log_info(&request_id, "登录失败", &e.to_string());
            // 返回错误消息
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::UNAUTHORIZED, Json(error_response)).into_response()
        }
    }
}

pub async fn refresh(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> impl IntoResponse {
    log_info(&request_id, "刷新 token 请求", &json!({"device_id": "default"}));

    let service = AuthService::new(state.pool.clone());

    match service.refresh_access_token(&payload.refresh_token, "default".to_string()).await {
        Ok((access_token, refresh_token)) => {
            // 获取用户信息
            // 从 access_token 中解码 user_id
            use jsonwebtoken::{decode, Validation, DecodingKey};
            use serde::Deserialize;

            #[derive(Deserialize)]
            struct Claims {
                sub: String,
            }

            let token_data = decode::<Claims>(
                &access_token,
                &DecodingKey::from_secret(state.config.auth.jwt_secret.as_ref()),
                &Validation::default(),
            );

            let (user_id, email) = match token_data {
                Ok(data) => {
                    // 获取用户 email
                    match sqlx::query_scalar::<_, String>("SELECT email FROM users WHERE id = ?")
                        .bind(&data.claims.sub)
                        .fetch_one(&state.pool)
                        .await
                    {
                        Ok(email) => (data.claims.sub.clone(), email),
                        Err(_) => {
                            log_info(&request_id, "查询用户邮箱失败", "用户不存在");
                            let error_response = ErrorResponse {
                                error: "用户不存在".to_string(),
                            };
                            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
                        }
                    }
                }
                Err(_) => {
                    log_info(&request_id, "解码 token 失败", "Invalid token");
                    let error_response = ErrorResponse {
                        error: "无效的令牌".to_string(),
                    };
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
                }
            };

            let response = AuthResponse {
                token: access_token,
                refresh_token,
                user_id: user_id.clone(),
                email,
            };

            log_info(&request_id, "刷新 token 成功", &json!({"user_id": user_id}));

            Json(response).into_response()
        }
        Err(e) => {
            log_info(&request_id, "刷新 token 失败", &e.to_string());
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::UNAUTHORIZED, Json(error_response)).into_response()
        }
    }
}

pub async fn logout(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // 提取 Authorization header
    let auth_header = match headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        Some(h) => h,
        None => {
            return StatusCode::BAD_REQUEST;
        }
    };

    if !auth_header.starts_with("Bearer ") {
        return StatusCode::BAD_REQUEST;
    }

    let token = &auth_header[7..];

    // 计算 token 的剩余 TTL（秒）
    let ttl_seconds = match extract_token_ttl(token) {
        Some(ttl) => ttl,
        None => {
            tracing::error!("Failed to extract token TTL");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    // 将 token 加入 Redis 黑名单
    match state.token_blacklist.add(token, ttl_seconds).await {
        Ok(_) => {
            tracing::info!("Token added to blacklist for user");
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("Failed to add token to blacklist: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// 从 JWT token 中提取剩余有效时间（秒）
fn extract_token_ttl(token: &str) -> Option<u64> {
    use jsonwebtoken::{decode, Validation, DecodingKey};
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Claims {
        exp: usize,
    }

    // 解码 token（不验证签名，只提取 exp）
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(b"dummy"), // 签名验证不重要，我们只读 exp
        &Validation::default(),
    ).ok()?;

    let now = chrono::Utc::now().timestamp() as usize;
    let exp = token_data.claims.exp;

    if exp > now {
        Some((exp - now) as u64)
    } else {
        // Token 已过期，设置一个较短的 TTL（60秒）
        Some(60)
    }
}
