use crate::models::{UserProfile, UpdateProfileRequest, error::{Result, AppError}};
use crate::database::repositories::UserProfileRepository;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde_json::json;
use reqwest::Client;
use std::time::Duration;

pub struct UserProfileService {
    repo: UserProfileRepository,
    client: Client,
    pool: Pool<SqliteConnectionManager>,
}

impl UserProfileService {
    pub fn new(repo: UserProfileRepository, pool: Pool<SqliteConnectionManager>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { repo, client, pool }
    }

    /// 获取当前用户的资料
    pub fn get_profile(&self, user_id: &str) -> Result<UserProfile> {
        self.repo
            .find_by_user_id(user_id)?
            .ok_or_else(|| AppError::NotFound(format!("用户 {} 的资料未找到", user_id)))
    }

    /// 更新当前用户的资料
    pub fn update_profile(&self, user_id: &str, req: UpdateProfileRequest) -> Result<UserProfile> {
        log::info!("[user_profile_service.rs::update_profile] 开始更新用户资料: user_id={}", user_id);
        log::info!("[user_profile_service.rs::update_profile] 请求数据: avatar_data={}, avatar_mime_type={}",
            req.avatar_data.as_ref().map(|d| format!("{} bytes", d.len())).unwrap_or_else(|| "None".to_string()),
            req.avatar_mime_type.as_deref().unwrap_or("None"));

        // 获取当前资料
        let mut profile = self.get_profile(user_id)?;

        // 更新字段
        profile.update(req);

        log::info!("[user_profile_service.rs::update_profile] 更新后的头像: avatar_data={}, avatar_mime_type={}",
            profile.avatar_data.as_ref().map(|d| format!("{} bytes", d.len())).unwrap_or_else(|| "None".to_string()),
            profile.avatar_mime_type.as_deref().unwrap_or("None"));

        // 保存到数据库
        let result = self.repo.update(&profile)?;

        log::info!("[user_profile_service.rs::update_profile] 保存成功，返回头像: avatar_data={}, avatar_mime_type={}",
            result.avatar_data.as_ref().map(|d| format!("{} bytes", d.len())).unwrap_or_else(|| "None".to_string()),
            result.avatar_mime_type.as_deref().unwrap_or("None"));

        Ok(result)
    }

    /// 初始化用户资料（首次登录时）
    pub fn init_profile(&self, user_id: &str) -> Result<UserProfile> {
        self.repo.get_or_create(user_id)
    }

    /// 同步用户资料到云端
    pub async fn sync_profile(&self, user_id: &str) -> Result<UserProfile> {
        use crate::services::AuthService;

        // 1. 获取本地资料
        let profile = self.get_profile(user_id)?;

        // 2. 获取服务器 URL 和 token
        let auth_service = AuthService::new(self.pool.clone());
        let (server_url, token) = auth_service.get_auth_info()?;

        // 3. 调用服务器同步接口
        let url = format!("{}/profile/sync", server_url.trim_end_matches('/'));

        let request_body = json!({
            "user_id": profile.user_id,
            "username": profile.username,
            "phone": profile.phone,
            "qq": profile.qq,
            "wechat": profile.wechat,
            "avatar_data": profile.avatar_data,
            "avatar_mime_type": profile.avatar_mime_type,
            "bio": profile.bio,
        });

        // 准备日志用的请求数据（截断 avatar_data 以便阅读）
        let log_body = {
            let mut body = serde_json::json!({
                "user_id": profile.user_id,
                "username": profile.username,
                "phone": profile.phone,
                "qq": profile.qq,
                "wechat": profile.wechat,
                "avatar_data": profile.avatar_data,
                "avatar_mime_type": profile.avatar_mime_type,
                "bio": profile.bio,
            });

            // 截断 avatar_data 用于日志显示
            if let Some(avatar) = body.get_mut("avatar_data") {
                if let Some(data_str) = avatar.as_str() {
                    if data_str.len() > 30 {
                        *avatar = serde_json::json!(format!("{}...", &data_str[..30]));
                    }
                }
            }

            body
        };

        log::info!("Syncing profile to {}: {}", url, log_body);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                log::error!("同步资料失败: {}", e);
                AppError::NetworkError(format!("同步用户资料失败: {}", e))
            })?;

        let status = response.status();

        // 先尝试解析响应为 JSON
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            log::error!("解析响应失败: {}", e);
            AppError::NetworkError(format!("响应无效: {}", e))
        })?;

        if !status.is_success() {
            // 解析错误消息
            let error_msg = response_json["error"]
                .as_str()
                .unwrap_or("未知错误");
            log::error!("服务器返回错误 {}: {}", status, error_msg);
            return Err(AppError::NetworkError(error_msg.to_string()));
        }

        // 手动解析服务器响应（snake_case）为 UserProfile（camelCase）
        // 服务器返回：user_id, username, phone, qq, wechat, avatar_data, avatar_mime_type, bio, created_at, updated_at
        let user_id = response_json["user_id"].as_str()
            .ok_or_else(|| AppError::NetworkError("响应中缺少 user_id".to_string()))?;
        let username = response_json["username"].as_str().map(|s| s.to_string());
        let phone = response_json["phone"].as_str().map(|s| s.to_string());
        let qq = response_json["qq"].as_str().map(|s| s.to_string());
        let wechat = response_json["wechat"].as_str().map(|s| s.to_string());
        let avatar_data = response_json["avatar_data"].as_str().map(|s| s.to_string());
        let avatar_mime_type = response_json["avatar_mime_type"].as_str().map(|s| s.to_string());
        let bio = response_json["bio"].as_str().map(|s| s.to_string());
        let created_at = response_json["created_at"].as_i64()
            .ok_or_else(|| AppError::NetworkError("响应中 created_at 无效".to_string()))?;
        let updated_at = response_json["updated_at"].as_i64()
            .ok_or_else(|| AppError::NetworkError("响应中 updated_at 无效".to_string()))?;

        // 获取当前本地 profile 的 id
        let local_profile = self.repo.find_by_user_id(user_id)?;
        let id = local_profile.and_then(|p| p.id);

        let server_profile = UserProfile {
            id,
            user_id: user_id.to_string(),
            username,
            phone,
            qq,
            wechat,
            avatar_data,
            avatar_mime_type,
            bio,
            created_at,
            updated_at,
        };

        log::info!("Profile synced successfully: user_id={}, username={:?}",
                 server_profile.user_id, server_profile.username);

        // 更新本地数据库
        let result = self.repo.update(&server_profile)?;

        Ok(result)
    }
}
