use serde::{Serialize, Deserialize};
use crate::models::user_profile::UserProfile;

/// 登录请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,  // 用户邮箱
    pub password: String,  // 用户密码
    pub server_url: String,  // 服务器 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,  // 设备唯一标识
}

/// 注册请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub email: String,  // 用户邮箱
    pub password: String,  // 用户密码
    pub server_url: String,  // 服务器 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,  // 设备唯一标识
}

/// 认证响应（从服务器返回）
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,  // JWT 访问令牌
    pub refresh_token: String,  // 刷新令牌
    pub user_id: String,  // 用户 ID
    pub email: String,  // 用户邮箱
    pub device_id: String,  // 设备 ID
}

/// 用户信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,  // 用户 ID
    pub email: String,  // 用户邮箱
    pub server_url: String,  // 服务器 URL
    pub device_id: String,  // 设备 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync_at: Option<i64>,  // 最后同步时间（Unix 时间戳，秒）
}

/// 账号信息（包含用户资料）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountWithProfile {
    pub id: String,  // 用户 ID
    pub email: String,  // 用户邮箱
    pub server_url: String,  // 服务器 URL
    pub device_id: String,  // 设备 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync_at: Option<i64>,  // 最后同步时间（Unix 时间戳，秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfile>,  // 用户资料
}

/// Token 刷新请求
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,  // 刷新令牌
}

/// Token 刷新响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,  // 新的访问令牌
    pub expires_at: i64,  // 令牌过期时间（Unix 时间戳，秒）
}

/// 登出请求
#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub device_id: String,  // 设备 ID
}
