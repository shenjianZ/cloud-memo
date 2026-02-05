use crate::models::{LoginRequest, RegisterRequest, AuthResponse, User};
use crate::models::error::{Result, AppError};
use crate::services::{AppSettingsService, UserProfileService};
use crate::database::repositories::UserProfileRepository;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use uuid::Uuid;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{Engine as _, engine::general_purpose};
use reqwest::Client;
use std::time::Duration;
use serde_json::json;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use r2d2_sqlite::rusqlite;

/// 认证服务
///
/// 管理用户登录、注册、token 加密存储
pub struct AuthService {
    pool: Pool<SqliteConnectionManager>,
    client: Client,
}

impl AuthService {
    /// 创建新的 AuthService 实例
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { pool, client }
    }

    /// 用户登录
    pub async fn login(&self, mut req: LoginRequest) -> Result<AuthResponse> {
        let device_id = self.get_or_create_device_id()?;

        // 如果 server_url 为空，从 app_settings 获取默认值
        if req.server_url.trim().is_empty() {
            let app_settings_service = AppSettingsService::new(self.pool.clone());
            req.server_url = app_settings_service.get_default_server_url()?;
        }

        let server_url = req.server_url.trim_end_matches('/');
        let url = format!("{}/auth/login", server_url);

        log::info!("Logging in to {}", url);

        let request_body = json!({
            "email": req.email,
            "password": req.password
        });

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to send login request: {}", e);
                AppError::NetworkError(format!("Login request failed: {}", e))
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
            return Err(AppError::AuthenticationError(error_msg.to_string()));
        }

        // 解析成功响应

        let token = response_json["token"]
            .as_str()
            .ok_or_else(|| AppError::AuthenticationError("Missing token in response".to_string()))?;

        let refresh_token = response_json["refresh_token"]
            .as_str()
            .ok_or_else(|| AppError::AuthenticationError("Missing refresh_token in response".to_string()))?;

        let user_id = response_json["user_id"]
            .as_str()
            .ok_or_else(|| AppError::AuthenticationError("Missing user_id in response".to_string()))?;

        // 创建 AuthResponse（expires_at 从 JWT token 解析，这里简化处理）
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + 7 * 24 * 3600; // 7 天后过期

        let response = AuthResponse {
            token: token.to_string(),
            refresh_token: refresh_token.to_string(),
            user_id: user_id.to_string(),
            email: req.email.clone(),
            expires_at: Some(expires_at),
        };

        // 加密并存储 token
        self.save_user_auth(
            &response.user_id,
            &req.server_url,
            &req.email,
            &response.token,
            &response.refresh_token,
            expires_at,
            &device_id,
        )?;

        log::info!("User logged in: {}", req.email);

        // 初始化用户资料（如果不存在）
        log::debug!("[AuthService::login] 初始化用户资料: user_id={}", response.user_id);
        let profile_service = UserProfileService::new(
            UserProfileRepository::new(self.pool.clone()),
            self.pool.clone()
        );
        if let Err(e) = profile_service.init_profile(&response.user_id) {
            log::warn!("[AuthService::login] 初始化用户资料失败（非致命错误）: {}", e);
        }

        Ok(response)
    }

    /// 用户注册
    pub async fn register(&self, mut req: RegisterRequest) -> Result<AuthResponse> {
        log::info!("[AuthService::register] 开始注册流程: email={}, server_url='{}'", req.email, req.server_url);

        let device_id = self.get_or_create_device_id()?;
        log::info!("[AuthService::register] 获取到 device_id: {}", device_id);

        // 如果 server_url 为空，从 app_settings 获取默认值
        if req.server_url.trim().is_empty() {
            log::info!("[AuthService::register] server_url 为空，尝试从 app_settings 获取默认值");
            let app_settings_service = AppSettingsService::new(self.pool.clone());
            req.server_url = app_settings_service.get_default_server_url()?;
            log::info!("[AuthService::register] 从 app_settings 获取到默认服务器: {}", req.server_url);
        }

        let server_url = req.server_url.trim_end_matches('/');
        let url = format!("{}/auth/register", server_url);

        log::info!("[AuthService::register] 准备发送注册请求到: {}", url);

        let request_body = json!({
            "email": req.email,
            "password": req.password
        });

        log::debug!("[AuthService::register] 请求体: {}", request_body);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                log::error!("[AuthService::register] 发送注册请求失败: {}", e);
                AppError::NetworkError(format!("Register request failed: {}", e))
            })?;

        log::info!("[AuthService::register] 收到服务器响应，状态码: {}", response.status());

        let status = response.status();

        // 先尝试解析响应为 JSON
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            log::error!("[AuthService::register] 解析响应失败: {}", e);
            AppError::NetworkError(format!("Invalid response: {}", e))
        })?;

        if !status.is_success() {
            // 解析错误消息
            let error_msg = response_json["error"]
                .as_str()
                .unwrap_or("Unknown error");
            log::error!("[AuthService::register] 服务器返回错误 {}: {}", status, error_msg);
            return Err(AppError::AuthenticationError(error_msg.to_string()));
        }

        log::info!("[AuthService::register] 服务器响应内容: {}", response_json);

        let token = response_json["token"]
            .as_str()
            .ok_or_else(|| AppError::AuthenticationError("Missing token in response".to_string()))?;

        let refresh_token = response_json["refresh_token"]
            .as_str()
            .ok_or_else(|| AppError::AuthenticationError("Missing refresh_token in response".to_string()))?;

        let user_id = response_json["user_id"]
            .as_str()
            .ok_or_else(|| AppError::AuthenticationError("Missing user_id in response".to_string()))?;

        log::info!("[AuthService::register] 成功提取 token、refresh_token 和 user_id: user_id={}", user_id);

        // 创建 AuthResponse（expires_at 从 JWT token 解析，这里简化处理）
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + 7 * 24 * 3600; // 7 天后过期

        let response = AuthResponse {
            token: token.to_string(),
            refresh_token: refresh_token.to_string(),
            user_id: user_id.to_string(),
            email: req.email.clone(),
            expires_at: Some(expires_at),
        };

        log::info!("[AuthService::register] 准备加密并存储 token");

        // 加密并存储 token
        self.save_user_auth(
            &response.user_id,
            &req.server_url,
            &req.email,
            &response.token,
            &response.refresh_token,
            expires_at,
            &device_id,
        )?;

        log::info!("[AuthService::register] 注册成功，user={}, user_id={}", req.email, response.user_id);

        // 初始化用户资料
        log::debug!("[AuthService::register] 初始化用户资料: user_id={}", response.user_id);
        let profile_service = UserProfileService::new(
            UserProfileRepository::new(self.pool.clone()),
            self.pool.clone()
        );
        if let Err(e) = profile_service.init_profile(&response.user_id) {
            log::warn!("[AuthService::register] 初始化用户资料失败（非致命错误）: {}", e);
        }

        Ok(response)
    }

    /// 用户登出
    ///
    /// 删除当前登录账号（is_current = 1）
    pub fn logout(&self) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        conn.execute(
            "DELETE FROM user_auth WHERE is_current = 1",
            [],
        ).map_err(|e| AppError::DatabaseError(format!("Failed to logout: {}", e)))?;

        log::info!("User logged out");
        Ok(())
    }

    /// 获取当前登录用户
    ///
    /// 本地验证，不向服务器发起请求
    /// 如果 token 已过期，返回 NotAuthenticated 错误
    pub fn get_current_user(&self) -> Result<User> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT user_id, server_url, email, device_id, last_sync_at, token_expires_at
             FROM user_auth
             WHERE is_current = 1"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to get current user: {}", e)))?;

        stmt.query_row([], |row| {
            let expires_at: i64 = row.get(5)?;
            let now = chrono::Utc::now().timestamp();

            // 检查 token 是否过期（提前 5 分钟判定过期，避免临界点问题）
            if expires_at < now - 300 {
                return Err(rusqlite::Error::InvalidQuery);
            }

            Ok(User {
                id: row.get(0)?,
                email: row.get(2)?,
                server_url: row.get(1)?,
                device_id: row.get(3)?,
                last_sync_at: row.get(4)?,
            })
        }).map_err(|_| AppError::NotAuthenticated("No user logged in or token expired".to_string()))
    }

    /// 检查是否已登录
    ///
    /// 本地验证，不向服务器发起请求
    /// 检查是否有当前用户，且 token 未过期
    pub fn is_authenticated(&self) -> Result<bool> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT token_expires_at FROM user_auth WHERE is_current = 1"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to query auth: {}", e)))?;

        let result = stmt.query_row([], |row| {
            let expires_at: i64 = row.get(0)?;
            let now = chrono::Utc::now().timestamp();

            // 检查 token 是否过期（提前 5 分钟判定过期）
            Ok(expires_at >= now - 300)
        });

        match result {
            Ok(is_valid) => Ok(is_valid),
            Err(_) => Ok(false), // 没有当前用户或查询失败
        }
    }

    /// 获取所有已登录的账号列表（包含用户资料）
    pub fn list_accounts(&self) -> Result<Vec<crate::models::AccountWithProfile>> {
        use crate::models::{AccountWithProfile, User, UserProfile};

        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 使用 LEFT JOIN 连接 user_auth 和 user_profiles 表
        let mut stmt = conn.prepare(
            "SELECT ua.user_id, ua.server_url, ua.email, ua.device_id, ua.last_sync_at,
                    up.id, up.user_id, up.username, up.phone, up.qq, up.wechat,
                    up.avatar_data, up.avatar_mime_type, up.bio, up.created_at, up.updated_at
             FROM user_auth ua
             LEFT JOIN user_profiles up ON ua.user_id = up.user_id
             ORDER BY ua.updated_at DESC"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to list accounts: {}", e)))?;

        let accounts_iter = stmt.query_map([], |row| {
            // 构建 User 对象
            let user = User {
                id: row.get(0)?,
                email: row.get(2)?,
                server_url: row.get(1)?,
                device_id: row.get(3)?,
                last_sync_at: row.get(4)?,
            };

            // 构建 UserProfile 对象（可能为 None）
            let profile_id: Option<i64> = row.get(5)?;
            let profile = match profile_id {
                Some(id) => {
                    Some(UserProfile {
                        id: Some(id),
                        user_id: row.get(6)?,
                        username: row.get(7)?,
                        phone: row.get(8)?,
                        qq: row.get(9)?,
                        wechat: row.get(10)?,
                        avatar_data: row.get(11)?,
                        avatar_mime_type: row.get(12)?,
                        bio: row.get(13)?,
                        created_at: row.get(14)?,
                        updated_at: row.get(15)?,
                    })
                }
                None => None,
            };

            Ok(AccountWithProfile {
                id: user.id,
                email: user.email,
                server_url: user.server_url,
                device_id: user.device_id,
                last_sync_at: user.last_sync_at,
                profile,
            })
        }).map_err(|e| AppError::DatabaseError(format!("Failed to parse accounts: {}", e)))?;

        let mut accounts = Vec::new();
        for account in accounts_iter {
            accounts.push(account.map_err(|e| AppError::DatabaseError(format!("Failed to read account: {}", e)))?);
        }

        Ok(accounts)
    }

    /// 切换到指定账号
    pub fn switch_account(&self, user_id: &str) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 开启事务
        let tx = conn.unchecked_transaction()
            .map_err(|e| AppError::DatabaseError(format!("Failed to begin transaction: {}", e)))?;

        // 将所有账号设为非当前
        tx.execute(
            "UPDATE user_auth SET is_current = 0",
            [],
        ).map_err(|e| AppError::DatabaseError(format!("Failed to reset current flag: {}", e)))?;

        // 将指定账号设为当前
        let params: &[&dyn rusqlite::ToSql] = &[
            &chrono::Utc::now().timestamp(),
            &user_id,
        ];
        let updated = tx.execute(
            "UPDATE user_auth SET is_current = 1, updated_at = ?1 WHERE user_id = ?2",
            params,
        ).map_err(|e| AppError::DatabaseError(format!("Failed to set current account: {}", e)))?;

        if updated == 0 {
            return Err(AppError::NotFound(format!("Account not found: {}", user_id)));
        }

        tx.commit()
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        log::info!("Switched to account: {}", user_id);
        Ok(())
    }

    /// 删除指定账号（不能删除当前账号）
    pub fn remove_account(&self, user_id: &str) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 检查是否为当前账号
        let is_current: bool = conn.query_row(
            "SELECT is_current FROM user_auth WHERE user_id = ?1",
            [user_id],
            |row| row.get(0)
        ).map_err(|_| AppError::NotFound(format!("Account not found: {}", user_id)))?;

        if is_current {
            return Err(AppError::InvalidOperation("Cannot remove current account. Please switch to another account first.".to_string()));
        }

        conn.execute(
            "DELETE FROM user_auth WHERE user_id = ?1",
            [user_id],
        ).map_err(|e| AppError::DatabaseError(format!("Failed to remove account: {}", e)))?;

        log::info!("Removed account: {}", user_id);
        Ok(())
    }

    /// 刷新 access_token（使用 refresh_token 向服务器请求）
    ///
    /// 这是一个异步方法，因为需要向服务器发起 HTTP 请求
    pub async fn refresh_access_token(&self) -> Result<AuthResponse> {
        // 1. 从数据库获取当前用户的 refresh_token
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let (server_url, encrypted_refresh_token, user_id, email, _device_id): (String, String, String, String, String) =
            conn.query_row(
                "SELECT server_url, refresh_token_encrypted, user_id, email, device_id
                 FROM user_auth
                 WHERE is_current = 1",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            ).map_err(|_| AppError::NotAuthenticated("No user logged in".to_string()))?;

        // 2. 解密 refresh_token
        let key = self.get_encryption_key();
        let refresh_token = self.decrypt_token(&encrypted_refresh_token)?;

        // 3. 向服务器发起刷新请求
        let url = format!("{}/auth/refresh", server_url.trim_end_matches('/'));

        log::info!("Refreshing access token from {}", url);

        let request_body = json!({
            "refresh_token": refresh_token,
        });

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to send refresh request: {}", e);
                AppError::NetworkError(format!("Refresh request failed: {}", e))
            })?;

        let status = response.status();

        // 先尝试解析响应为 JSON
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            log::error!("Failed to parse refresh response: {}", e);
            AppError::NetworkError(format!("Invalid refresh response: {}", e))
        })?;

        if !status.is_success() {
            // 解析错误消息
            let error_msg = response_json["error"]
                .as_str()
                .unwrap_or("Unknown error");
            log::error!("Server returned error {}: {}", status, error_msg);
            return Err(AppError::AuthenticationError(error_msg.to_string()));
        }

        // 4. 解析服务器响应
        let new_access_token = response_json["token"]
            .as_str()
            .ok_or_else(|| AppError::AuthenticationError("Missing token in refresh response".to_string()))?;

        let new_refresh_token = response_json["refresh_token"]
            .as_str()
            .ok_or_else(|| AppError::AuthenticationError("Missing refresh_token in refresh response".to_string()))?;

        // 5. 计算新的过期时间（7天后）
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + 7 * 24 * 3600;

        // 6. 更新数据库（加密存储新的 token）
        let encrypted_access = self.encrypt_token(new_access_token, &key)?;
        let encrypted_refresh = self.encrypt_token(new_refresh_token, &key)?;

        conn.execute(
            "UPDATE user_auth
             SET access_token_encrypted = ?1,
                 refresh_token_encrypted = ?2,
                 token_expires_at = ?3,
                 updated_at = ?4
             WHERE user_id = ?5",
            [&encrypted_access as &dyn r2d2_sqlite::rusqlite::ToSql,
             &encrypted_refresh as &dyn r2d2_sqlite::rusqlite::ToSql,
             &expires_at as &dyn r2d2_sqlite::rusqlite::ToSql,
             &now as &dyn r2d2_sqlite::rusqlite::ToSql,
             &user_id as &dyn r2d2_sqlite::rusqlite::ToSql],
        ).map_err(|e| AppError::DatabaseError(format!("Failed to update tokens: {}", e)))?;

        log::info!("Access token refreshed successfully for user: {}", email);

        Ok(AuthResponse {
            token: new_access_token.to_string(),
            refresh_token: new_refresh_token.to_string(),
            user_id,
            email,
            expires_at: Some(expires_at),
        })
    }

    /// 获取认证信息（服务器 URL 和 token）
    ///
    /// 注意：此方法不检查 token 是否过期，只返回当前存储的 token
    /// 如需自动刷新，请使用 get_auth_info_with_refresh() 或手动检查 token_expires_at
    pub fn get_auth_info(&self) -> Result<(String, String)> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT server_url, access_token_encrypted
             FROM user_auth
             WHERE is_current = 1"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to get auth info: {}", e)))?;

        let (server_url, encrypted_token): (String, String) = stmt.query_row([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).map_err(|_| AppError::NotAuthenticated("No user logged in".to_string()))?;

        // 解密 token
        let token = self.decrypt_token(&encrypted_token)?;

        Ok((server_url, token))
    }

    /// 获取认证信息（包含 token 过期时间）
    pub fn get_auth_info_with_expires(&self) -> Result<(String, String, i64)> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT server_url, access_token_encrypted, token_expires_at
             FROM user_auth
             WHERE is_current = 1"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to get auth info: {}", e)))?;

        let (server_url, encrypted_token, expires_at): (String, String, i64) = stmt.query_row([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }).map_err(|_| AppError::NotAuthenticated("No user logged in".to_string()))?;

        // 解密 token
        let token = self.decrypt_token(&encrypted_token)?;

        Ok((server_url, token, expires_at))
    }

    // ===== 私有方法 =====

    /// 保存用户认证信息（加密，支持多账号）
    fn save_user_auth(
        &self,
        user_id: &str,  // 使用服务器返回的 user_id
        server_url: &str,
        email: &str,
        access_token: &str,
        refresh_token: &str,
        expires_at: i64,
        device_id: &str,
    ) -> Result<()> {
        log::debug!("[AuthService::save_user_auth] 开始保存用户认证信息: email={}, server_url={}, user_id={}", email, server_url, user_id);

        // 加密 token
        log::debug!("[AuthService::save_user_auth] 开始加密 token");
        let key = self.get_encryption_key();
        let encrypted_access = self.encrypt_token(access_token, &key)?;
        let encrypted_refresh = self.encrypt_token(refresh_token, &key)?;
        log::debug!("[AuthService::save_user_auth] token 加密完成");

        let now = chrono::Utc::now().timestamp();

        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 开启事务
        let tx = conn.unchecked_transaction()
            .map_err(|e| AppError::DatabaseError(format!("Failed to begin transaction: {}", e)))?;

        log::debug!("[AuthService::save_user_auth] 准备插入数据库: user_id={}, device_id={}", user_id, device_id);

        // 将所有账号设为非当前
        tx.execute(
            "UPDATE user_auth SET is_current = 0",
            [],
        ).map_err(|e| AppError::DatabaseError(format!("Failed to reset current flag: {}", e)))?;

        // 插入或更新当前账号（is_current = 1）
        // 注意：这里使用 INSERT OR REPLACE，会根据 user_id 唯一约束判断是插入还是更新
        tx.execute(
            "INSERT INTO user_auth
             (user_id, server_url, email, access_token_encrypted, refresh_token_encrypted,
              token_expires_at, device_id, last_sync_at, is_current, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?10)
             ON CONFLICT(user_id) DO UPDATE SET
                server_url = excluded.server_url,
                email = excluded.email,
                access_token_encrypted = excluded.access_token_encrypted,
                refresh_token_encrypted = excluded.refresh_token_encrypted,
                token_expires_at = excluded.token_expires_at,
                device_id = excluded.device_id,
                is_current = 1,
                updated_at = excluded.updated_at",
            (
                user_id,
                server_url,
                email,
                &encrypted_access,
                &encrypted_refresh,
                expires_at,
                device_id,
                None::<i64>,  // last_sync_at
                now,   // created_at
                now,   // updated_at
            ),
        ).map_err(|e| AppError::DatabaseError(format!("Failed to save auth: {}", e)))?;

        tx.commit()
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        log::info!("[AuthService::save_user_auth] 用户认证信息保存成功: user_id={}", user_id);
        Ok(())
    }

    /// 获取或创建设备 ID（设备级，所有用户共享）
    fn get_or_create_device_id(&self) -> Result<String> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 从 app_config 表获取设备 ID（设备级配置，不与特定用户绑定）
        let mut stmt = conn.prepare("SELECT device_id FROM app_config WHERE id = 1")?;

        match stmt.query_row([], |row| row.get::<_, String>(0)) {
            Ok(device_id) => {
                log::debug!("[AuthService::get_or_create_device_id] 使用已存在的设备 ID: {}", device_id);
                Ok(device_id)
            }
            Err(_) => {
                // 首次运行，生成并保存设备 ID
                let device_id = Uuid::new_v4().to_string();
                let now = chrono::Utc::now().timestamp();

                log::info!("[AuthService::get_or_create_device_id] 创建新的设备 ID: {}", device_id);

                conn.execute(
                    "INSERT INTO app_config (id, device_id, created_at, updated_at)
                     VALUES (1, ?1, ?2, ?3)",
                    [&device_id, &now as &dyn r2d2_sqlite::rusqlite::ToSql, &now as &dyn r2d2_sqlite::rusqlite::ToSql],
                ).map_err(|e| AppError::DatabaseError(format!("Failed to save device_id: {}", e)))?;

                Ok(device_id)
            }
        }
    }

    /// 获取加密密钥（基于设备 ID 派生）
    fn get_encryption_key(&self) -> [u8; 32] {
        // 应用特定盐值（硬编码，防止跨应用密钥重用）
        const APP_SALT: &[u8] = b"markdown-notes-app-salt-2024";

        // 尝试从数据库获取设备 ID
        let device_id = self.get_device_id_for_key()
            .unwrap_or_else(|_| "fallback-device-id".to_string());

        // PBKDF2 参数
        const ITERATIONS: u32 = 100_000; // OWASP 推荐的最小迭代次数

        let mut key = [0u8; 32];

        // 使用 PBKDF2-HMAC-SHA256 派生密钥
        pbkdf2_hmac::<Sha256>(
            device_id.as_bytes(),     // 密码（使用 device_id）
            APP_SALT,                  // 盐值（应用特定）
            ITERATIONS,                // 迭代次数
            &mut key,                  // 输出密钥
        );

        log::debug!("Encryption key derived from device_id");
        key
    }

    /// 获取设备 ID（用于密钥派生）
    ///
    /// 这个方法专门用于密钥派生，避免循环依赖
    fn get_device_id_for_key(&self) -> Result<String> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare("SELECT device_id FROM user_auth WHERE is_current = 1")?;

        match stmt.query_row([], |row| row.get::<_, String>(0)) {
            Ok(device_id) => Ok(device_id),
            Err(_) => {
                // 如果数据库中没有 device_id，生成一个新的
                let device_id = Uuid::new_v4().to_string();
                log::warn!("No device_id found, generated new one for key derivation");
                Ok(device_id)
            }
        }
    }

    /// 加密 token
    fn encrypt_token(&self, token: &str, key: &[u8; 32]) -> Result<String> {
        let cipher = Aes256Gcm::new(key.into());
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher.encrypt(&nonce, token.as_bytes())
            .map_err(|e| AppError::EncryptionError(format!("Failed to encrypt: {}", e)))?;

        // 组合 nonce 和 ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(&result))
    }

    /// 解密 token
    pub fn decrypt_token(&self, encrypted: &str) -> Result<String> {
        let key = self.get_encryption_key();
        let data = general_purpose::STANDARD.decode(encrypted)
            .map_err(|e| AppError::EncryptionError(format!("Failed to decode: {}", e)))?;

        if data.len() < 12 {
            return Err(AppError::EncryptionError("Invalid encrypted data".to_string()));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher = Aes256Gcm::new((&key).into());
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| AppError::EncryptionError(format!("Failed to decrypt: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| AppError::EncryptionError(format!("Invalid UTF-8: {}", e)))
    }
}
