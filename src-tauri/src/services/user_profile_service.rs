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
            .ok_or_else(|| AppError::NotFound(format!("UserProfile for user {} not found", user_id)))
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

        log::info!("Syncing profile to {}: {}", url, request_body);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to sync profile: {}", e);
                AppError::NetworkError(format!("Sync profile failed: {}", e))
            })?;

        let status = response.status();

        // 先尝试解析响应为 JSON
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            log::error!("Failed to parse response: {}", e);
            AppError::NetworkError(format!("Invalid response: {}", e))
        })?;

        if !status.is_success() {
            // 解析错误消息
            let error_msg = response_json["error"]
                .as_str()
                .unwrap_or("Unknown error");
            log::error!("Server returned error {}: {}", status, error_msg);
            return Err(AppError::NetworkError(error_msg.to_string()));
        }

        // 4. 记录响应（服务器返回更新后的资料）
        log::info!("Profile synced successfully: {:?}", response_json);

        Ok(profile)
    }
}
