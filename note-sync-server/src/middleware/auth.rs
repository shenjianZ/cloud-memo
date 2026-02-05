// 认证中间件
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};
use jsonwebtoken::{decode, Validation, DecodingKey};
use serde::Deserialize;
use crate::AppState;

#[derive(Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id
    pub exp: usize,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. 提取 Authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];

    // 2. 检查 Token 是否在黑名单中
    if state.token_blacklist.contains(token).await
        .map_err(|e| {
            tracing::error!("Failed to check token blacklist: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    {
        tracing::warn!("Token in blacklist, rejecting request");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 3. 验证 JWT
    let jwt_secret = &state.config.auth.jwt_secret;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 4. 将 user_id 添加到请求扩展
    req.extensions_mut().insert(token_data.claims.sub);

    Ok(next.run(req).await)
}
