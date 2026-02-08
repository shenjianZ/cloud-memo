use axum::{Json, extract::{State, Extension}, http::StatusCode};
use axum::http::{HeaderMap, header};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::AppState;
use crate::services::auth_service::AuthService;
use crate::services::device_service::DeviceService;
use crate::services::device_identifier_service::DeviceIdentifierService;
use crate::middleware::logging::{RequestId, log_info};
use super::ErrorResponse;
use std::fmt;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub device_id: Option<String>,  // 设备唯一标识（可选）
}

// 自定义 Debug 实现，隐藏密码
impl fmt::Debug for LoginRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoginRequest")
            .field("email", &self.email)
            .field("password", &"***")
            .field("device_id", &self.device_id)
            .finish()
    }
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub device_id: Option<String>,  // 设备唯一标识（可选）
}

// 自定义 Debug 实现，隐藏密码
impl fmt::Debug for RegisterRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegisterRequest")
            .field("email", &self.email)
            .field("password", &"***")
            .field("device_id", &self.device_id)
            .finish()
    }
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub email: String,
    pub device_id: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
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
    headers: HeaderMap,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, ErrorResponse> {
    log_info(&request_id, "注册请求参数", &payload);

    let service = AuthService::new(state.pool.clone());
    let device_service = DeviceService::new(state.pool.clone());

    // 1. 检查邮箱是否已存在
    let exists = service.check_email_exists(&payload.email).await
        .map_err(|e| ErrorResponse::new(format!("检查邮箱失败: {}", e)))?;

    if exists {
        log_info(&request_id, "邮箱已注册", &payload.email);
        return Err(ErrorResponse::new("邮箱已注册".to_string()));
    }

    // 2. 哈希密码（同步操作）
    let password_hash = service.hash_password(&payload.password)
        .map_err(|e| ErrorResponse::new(format!("密码哈希失败: {}", e)))?;

    // 3. 生成唯一的用户 ID
    let user_id = service.generate_unique_user_id().await
        .map_err(|e| ErrorResponse::new(format!("生成用户 ID 失败: {}", e)))?;

    // 4. 创建用户
    let created_at = service.create_user(&payload.email, &password_hash, &user_id).await
        .map_err(|e| ErrorResponse::new(format!("创建用户失败: {}", e)))?;

    // 5. 注册设备（使用客户端提供的 device_id 或生成默认值）
    let client_device_id = payload.device_id.clone().unwrap_or_else(|| {
        format!("default-{:x}", md5::compute(&payload.email))
    });

    // 从请求头提取 User-Agent
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok());

    // 从 device_id 和 User-Agent 识别设备信息
    let device_info = DeviceIdentifierService::identify_device(
        &client_device_id,
        user_agent,
    ).unwrap_or_else(|_| {
        // 解析失败，使用默认值
        DeviceIdentifierService::parse_device_id(&client_device_id).unwrap()
    });

    let device_name = DeviceIdentifierService::get_device_name(&device_info);
    let device_type = device_info.device_type.as_str();

    // 如果识别后的类型与原始 device_id 不一致，重新生成 device_id
    // 例如：mobile-ios-xxx 被识别为 tablet，则更新为 tablet-ios-xxx
    let final_device_id = if client_device_id.starts_with("mobile-")
        && device_info.device_type == crate::services::device_identifier_service::DeviceType::Tablet
    {
        // 提取 UUID 部分（最后一段）
        let uuid_part = client_device_id.rsplit('-').next().unwrap_or(&client_device_id);
        format!("{}-{}-{}",
            device_type,
            device_info.platform.as_str(),
            uuid_part
        )
    } else {
        client_device_id.clone()
    };

    log_info(&request_id, "设备识别信息", &format!(
        "original_id={}, final_id={}, type={}, platform={}, name={}",
        client_device_id,
        final_device_id,
        device_type,
        device_info.platform.as_str(),
        device_name
    ));

    let device = device_service.register_or_update(&user_id, &final_device_id, &device_name, device_type).await
        .map_err(|e| ErrorResponse::new(format!("注册设备失败: {}", e)))?;

    log_info(&request_id, "设备注册成功", &format!("device_id={}, name={}", device.id, device_name));

    // 6. 生成 token 并完成注册
    let (user, token, refresh_token) = service.complete_registration(&user_id, &payload.email, created_at, Some(device.id.clone())).await
        .map_err(|e| ErrorResponse::new(format!("生成 token 失败: {}", e)))?;

    let response = AuthResponse {
        token,
        refresh_token,
        user_id: user.id,
        email: user.email,
        device_id: device.id,
    };

    log_info(&request_id, "注册成功，返回用户信息", &response);

    Ok(Json(response))
}

pub async fn login(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ErrorResponse> {
    // 第2条日志：请求参数
    log_info(&request_id, "登录请求参数", &payload);

    let service = AuthService::new(state.pool.clone());
    let device_service = DeviceService::new(state.pool);

    match service.login(&payload.email, &payload.password, Some("default".to_string())).await {
        Ok((user, token, refresh_token)) => {
            // 注册或更新设备（使用客户端提供的 device_id 或生成默认值）
            let client_device_id = payload.device_id.clone().unwrap_or_else(|| {
                format!("default-{:x}", md5::compute(&payload.email))
            });

            // 从请求头提取 User-Agent
            let user_agent = headers
                .get(header::USER_AGENT)
                .and_then(|v| v.to_str().ok());

            // 从 device_id 和 User-Agent 识别设备信息
            let device_info = DeviceIdentifierService::identify_device(
                &client_device_id,
                user_agent,
            ).unwrap();
            let device_name = DeviceIdentifierService::get_device_name(&device_info);
            let device_type = device_info.device_type.as_str();

            // 如果识别后的类型与原始 device_id 不一致，重新生成 device_id
            // 例如：mobile-ios-xxx 被识别为 tablet，则更新为 tablet-ios-xxx
            let final_device_id = if client_device_id.starts_with("mobile-")
                && device_info.device_type == crate::services::device_identifier_service::DeviceType::Tablet
            {
                // 提取 UUID 部分（最后一段）
                let uuid_part = client_device_id.rsplit('-').next().unwrap_or(&client_device_id);
                format!("{}-{}-{}",
                    device_type,
                    device_info.platform.as_str(),
                    uuid_part
                )
            } else {
                client_device_id.clone()
            };

            log_info(&request_id, "设备识别信息", &format!(
                "original_id={}, final_id={}, type={}, platform={}, name={}",
                client_device_id,
                final_device_id,
                device_type,
                device_info.platform.as_str(),
                device_name
            ));

            let device = device_service.register_or_update(&user.id, &final_device_id, &device_name, device_type).await
                .map_err(|e| ErrorResponse::new(format!("设备注册失败: {}", e)))?;

            let response = AuthResponse {
                token,
                refresh_token,
                user_id: user.id,
                email: user.email,
                device_id: device.id,
            };

            // 第2条日志：响应内容
            log_info(&request_id, "登录成功，返回用户信息", &response);

            Ok(Json(response))
        }
        Err(e) => {
            log_info(&request_id, "登录失败", &e.to_string());
            // 返回错误消息
            Err(ErrorResponse::new(e.to_string()))
        }
    }
}

pub async fn refresh(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, ErrorResponse> {
    log_info(&request_id, "刷新 token 请求", &json!({"device_id": "default"}));

    let service = AuthService::new(state.pool.clone());
    let device_service = DeviceService::new(state.pool.clone());

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
                            return Err(ErrorResponse::new("用户不存在".to_string()));
                        }
                    }
                }
                Err(_) => {
                    log_info(&request_id, "解码 access_token 失败", "无效的 token");
                    return Err(ErrorResponse::new("无效的 access_token".to_string()));
                }
            };

            // 获取或注册设备（使用默认 device_id）
            let device = device_service.register_or_update(&user_id, "default-refresh", "default", "desktop").await
                .map_err(|e| ErrorResponse::new(format!("设备注册失败: {}", e)))?;

            let response = AuthResponse {
                token: access_token,
                refresh_token,
                user_id,
                email,
                device_id: device.id,
            };

            log_info(&request_id, "刷新成功", &json!({"user_id": response.user_id}));

            Ok(Json(response))
        }
        Err(e) => {
            log_info(&request_id, "刷新失败", &e.to_string());
            Err(ErrorResponse::new(e.to_string()))
        }
    }
}

pub async fn logout(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<StatusCode, ErrorResponse> {
    // 提取 Authorization header
    let auth_header = match headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        Some(h) => h,
        None => {
            return Err(ErrorResponse::new("缺少 Authorization header".to_string()));
        }
    };

    if !auth_header.starts_with("Bearer ") {
        return Err(ErrorResponse::new("无效的 Authorization 格式".to_string()));
    }

    let token = &auth_header[7..];

    // 计算 token 的剩余 TTL（秒）
    let ttl_seconds = match extract_token_ttl(token) {
        Some(ttl) => ttl,
        None => {
            return Err(ErrorResponse::new("无效的 token".to_string()));
        }
    };

    let mut blacklist = state.token_blacklist.clone();
    match blacklist.add(token, ttl_seconds).await {
        Ok(_) => {
            tracing::info!("Token 已加入黑名单");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            tracing::error!("将 Token 加入黑名单失败: {:?}", e);
            Err(ErrorResponse::new(format!("登出失败: {}", e)))
        }
    }
}

/// 删除用户账号
pub async fn delete_account(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<DeleteAccountRequest>,
) -> Result<StatusCode, ErrorResponse> {
    log_info(&request_id, "删除账号请求", &format!("user_id={}", user_id));

    let service = AuthService::new(state.pool);

    match service.delete_user(&user_id, &payload.password).await {
        Ok(_) => {
            log_info(&request_id, "账号删除成功", &format!("user_id={}", user_id));
            Ok(StatusCode::OK)
        }
        Err(e) => {
            log_info(&request_id, "账号删除失败", &e.to_string());
            Err(ErrorResponse::new(e.to_string()))
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
