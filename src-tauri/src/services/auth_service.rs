use crate::models::{LoginRequest, RegisterRequest, AuthResponse, User};
use crate::models::error::{Result, AppError};
use crate::services::{AppSettingsService, UserProfileService, CryptoService};
use crate::database::repositories::UserProfileRepository;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use uuid::Uuid;
use reqwest::Client;
use std::time::Duration;
use serde_json::json;
use r2d2_sqlite::rusqlite;

/// 认证服务
///
/// 管理用户登录、注册、token 加密存储
#[derive(Clone)]
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
        // 如果 server_url 为空，从 app_settings 获取默认值
        if req.server_url.trim().is_empty() {
            let app_settings_service = AppSettingsService::new(self.pool.clone());
            req.server_url = app_settings_service.get_default_server_url()?;
        }

        // 自动添加 device_id（如果客户端未提供）
        if req.device_id.is_none() {
            use crate::services::DeviceIdentifierService;
            let device_service = DeviceIdentifierService::new(self.pool.clone());
            req.device_id = Some(device_service.get_or_create_device_id()?);
            log::info!("[AuthService::login] 自动添加 device_id: {:?}", req.device_id);
        }

        let server_url = req.server_url.trim_end_matches('/');
        let url = format!("{}/auth/login", server_url);

        log::info!("Logging in to {}", url);

        let request_body = json!({
            "email": req.email,
            "password": req.password,
            "device_id": req.device_id  // snake_case 发送
        });

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to send login request: {}", e);
                AppError::NetworkError(format!("登录请求失败: {}", e))
            })?;

        let status = response.status();

        // 先解析响应为 JSON Value（用于错误处理）
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            log::error!("Failed to parse response: {}", e);
            AppError::NetworkError(format!("响应无效: {}", e))
        })?;

        if !status.is_success() {
            // 解析错误消息
            let error_msg = response_json["error"]
                .as_str()
                .unwrap_or("未知错误");
            log::error!("Server returned error {}: {}", status, error_msg);
            return Err(AppError::AuthenticationError(error_msg.to_string()));
        }

        // 直接反序列化为 AuthResponse（服务器和客户端都使用 snake_case）
        let auth_response: AuthResponse = serde_json::from_value(response_json)
            .map_err(|e| {
                log::error!("Failed to parse auth response: {}", e);
                AppError::AuthenticationError(format!("认证响应无效: {}", e))
            })?;

        // 客户端计算 token 过期时间（7天后）
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + 7 * 24 * 3600;

        // 加密并存储 token
        self.save_user_auth(
            &auth_response.user_id,
            &req.server_url,
            &req.email,
            &auth_response.token,
            &auth_response.refresh_token,
            expires_at,
            &auth_response.device_id,
        )?;

        log::info!("User logged in: {}", req.email);

        // 初始化用户资料（如果不存在）
        log::debug!("[AuthService::login] 初始化用户资料: user_id={}", auth_response.user_id);
        let profile_service = UserProfileService::new(
            UserProfileRepository::new(self.pool.clone()),
            self.pool.clone()
        );
        if let Err(e) = profile_service.init_profile(&auth_response.user_id) {
            log::warn!("[AuthService::login] 初始化用户资料失败（非致命错误）: {}", e);
        }

        // 确保默认工作空间存在
        log::debug!("[AuthService::login] 检查默认工作空间: user_id={}", auth_response.user_id);
        use crate::services::WorkspaceService;
        use crate::database::repositories::WorkspaceRepository;

        let workspace_service = WorkspaceService::new(WorkspaceRepository::new(self.pool.clone()));
        let default_workspace = workspace_service.get_default_workspace(&auth_response.user_id);

        if default_workspace.is_err() {
            // 默认工作空间不存在，创建一个
            log::info!("[AuthService::login] 默认工作空间不存在，创建中: user_id={}", auth_response.user_id);

            // 直接使用 repository 创建默认工作空间（绕过 create_workspace 的用户 ID 验证）
            use crate::models::Workspace;
            let new_default_workspace = Workspace::new_with_default(
                auth_response.user_id.clone(),
                "我的空间".to_string(),
                None,
                None,
                None,
                true,  // is_default = true
            );

            let repo = WorkspaceRepository::new(self.pool.clone());
            match repo.create(&new_default_workspace) {
                Ok(workspace) => {
                    log::info!("[AuthService::login] 默认工作空间创建成功: user_id={}, workspace_id={}", auth_response.user_id, workspace.id);

                    // 设置为当前工作空间
                    if let Err(e) = repo.set_current(&auth_response.user_id, &workspace.id) {
                        log::warn!("[AuthService::login] 设置默认工作空间为当前空间失败（非致命错误）: {}", e);
                    } else {
                        log::info!("[AuthService::login] 默认工作空间已设置为当前空间: workspace_id={}", workspace.id);
                    }
                }
                Err(e) => {
                    log::warn!("[AuthService::login] 创建默认工作空间失败（非致命错误）: {}", e);
                }
            }
        } else {
            log::debug!("[AuthService::login] 默认工作空间已存在: user_id={}", auth_response.user_id);
        }

        Ok(auth_response)
    }

    /// 用户注册
    pub async fn register(&self, mut req: RegisterRequest) -> Result<AuthResponse> {
        log::info!("[AuthService::register] 开始注册流程: email={}, server_url='{}'", req.email, req.server_url);

        // 如果 server_url 为空，从 app_settings 获取默认值
        if req.server_url.trim().is_empty() {
            log::info!("[AuthService::register] server_url 为空，尝试从 app_settings 获取默认值");
            let app_settings_service = AppSettingsService::new(self.pool.clone());
            req.server_url = app_settings_service.get_default_server_url()?;
            log::info!("[AuthService::register] 从 app_settings 获取到默认服务器: {}", req.server_url);
        }

        // 自动添加 device_id（如果客户端未提供）
        if req.device_id.is_none() {
            use crate::services::DeviceIdentifierService;
            let device_service = DeviceIdentifierService::new(self.pool.clone());
            req.device_id = Some(device_service.get_or_create_device_id()?);
            log::info!("[AuthService::register] 自动添加 device_id: {:?}", req.device_id);
        }

        let server_url = req.server_url.trim_end_matches('/');
        let url = format!("{}/auth/register", server_url);

        log::info!("[AuthService::register] 准备发送注册请求到: {}", url);

        let request_body = json!({
            "email": req.email,
            "password": req.password,
            "device_id": req.device_id  // snake_case 发送
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
                AppError::NetworkError(format!("注册请求失败: {}", e))
            })?;

        log::info!("[AuthService::register] 收到服务器响应，状态码: {}", response.status());

        let status = response.status();

        // 先解析响应为 JSON Value（用于错误处理）
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            log::error!("[AuthService::register] 解析响应失败: {}", e);
            AppError::NetworkError(format!("响应无效: {}", e))
        })?;

        if !status.is_success() {
            // 解析错误消息
            let error_msg = response_json["error"]
                .as_str()
                .unwrap_or("未知错误");
            log::error!("[AuthService::register] 服务器返回错误 {}: {}", status, error_msg);
            return Err(AppError::AuthenticationError(error_msg.to_string()));
        }

        log::info!("[AuthService::register] 服务器响应内容: {}", response_json);

        // 直接反序列化为 AuthResponse（服务器和客户端都使用 snake_case）
        let auth_response: AuthResponse = serde_json::from_value(response_json)
            .map_err(|e| {
                log::error!("Failed to parse auth response: {}", e);
                AppError::AuthenticationError(format!("认证响应无效: {}", e))
            })?;

        log::info!("[AuthService::register] 成功提取 auth response: user_id={}, device_id={}",
                 auth_response.user_id, auth_response.device_id);

        // 客户端计算 token 过期时间（7天后）
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + 7 * 24 * 3600;

        log::info!("[AuthService::register] 准备加密并存储 token");

        // 加密并存储 token
        self.save_user_auth(
            &auth_response.user_id,
            &req.server_url,
            &req.email,
            &auth_response.token,
            &auth_response.refresh_token,
            expires_at,
            &auth_response.device_id,
        )?;

        log::info!("[AuthService::register] 注册成功，user={}, user_id={}", req.email, auth_response.user_id);

        // 初始化用户资料
        log::debug!("[AuthService::register] 初始化用户资料: user_id={}", auth_response.user_id);
        let profile_service = UserProfileService::new(
            UserProfileRepository::new(self.pool.clone()),
            self.pool.clone()
        );
        if let Err(e) = profile_service.init_profile(&auth_response.user_id) {
            log::warn!("[AuthService::register] 初始化用户资料失败（非致命错误）: {}", e);
        }

        // 创建默认工作空间
        log::debug!("[AuthService::register] 创建默认工作空间: user_id={}", auth_response.user_id);
        use crate::database::repositories::WorkspaceRepository;
        use crate::models::Workspace;

        let default_workspace = Workspace::new_with_default(
            auth_response.user_id.clone(),
            "我的空间".to_string(),
            None,
            None,
            None,
            true,  // is_default = true
        );

        let repo = WorkspaceRepository::new(self.pool.clone());
        match repo.create(&default_workspace) {
            Ok(workspace) => {
                log::info!("[AuthService::register] 默认工作空间创建成功: user_id={}, workspace_id={}", auth_response.user_id, workspace.id);

                // 设置为当前工作空间
                if let Err(e) = repo.set_current(&auth_response.user_id, &workspace.id) {
                    log::warn!("[AuthService::register] 设置默认工作空间为当前空间失败（非致命错误）: {}", e);
                } else {
                    log::info!("[AuthService::register] 默认工作空间已设置为当前空间: workspace_id={}", workspace.id);
                }
            }
            Err(e) => {
                log::warn!("[AuthService::register] 创建默认工作空间失败（非致命错误）: {}", e);
            }
        }

        Ok(auth_response)
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
        ).map_err(|e| AppError::DatabaseError(format!("登出失败: {}", e)))?;

        log::info!("User logged out");
        Ok(())
    }

    /// 获取当前用户的访问 token
    fn get_access_token(&self) -> Result<String> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT access_token_encrypted, device_id FROM user_auth WHERE is_current = 1"
        ).map_err(|e| AppError::DatabaseError(format!("获取访问令牌失败: {}", e)))?;

        let (encrypted_token, device_id): (String, String) = stmt.query_row([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).map_err(|_| AppError::NotAuthenticated("用户未登录".to_string()))?;

        // 使用 device_id 生成加密密钥
        let key = CryptoService::derive_key_from_device_id(&device_id);

        // 解密 token
        let token = CryptoService::decrypt_token(&encrypted_token, &key)?;
        Ok(token)
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
        ).map_err(|e| AppError::DatabaseError(format!("获取当前用户失败: {}", e)))?;

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
        }).map_err(|_| AppError::NotAuthenticated("用户未登录或令牌已过期".to_string()))
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
        ).map_err(|e| AppError::DatabaseError(format!("查询认证信息失败: {}", e)))?;

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
        ).map_err(|e| AppError::DatabaseError(format!("列出账号失败: {}", e)))?;

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
            accounts.push(account.map_err(|e| AppError::DatabaseError(format!("读取账号失败: {}", e)))?);
        }

        Ok(accounts)
    }

    /// 切换到指定账号
    pub fn switch_account(&self, user_id: &str) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 开启事务
        let tx = conn.unchecked_transaction()
            .map_err(|e| AppError::DatabaseError(format!("开始事务失败: {}", e)))?;

        // 将所有账号设为非当前
        tx.execute(
            "UPDATE user_auth SET is_current = 0",
            [],
        ).map_err(|e| AppError::DatabaseError(format!("重置当前标志失败: {}", e)))?;

        // 将指定账号设为当前
        let params: &[&dyn rusqlite::ToSql] = &[
            &chrono::Utc::now().timestamp(),
            &user_id,
        ];
        let updated = tx.execute(
            "UPDATE user_auth SET is_current = 1, updated_at = ?1 WHERE user_id = ?2",
            params,
        ).map_err(|e| AppError::DatabaseError(format!("设置当前账号失败: {}", e)))?;

        if updated == 0 {
            return Err(AppError::NotFound(format!("账号不存在: {}", user_id)));
        }

        tx.commit()
            .map_err(|e| AppError::DatabaseError(format!("提交事务失败: {}", e)))?;

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
        ).map_err(|_| AppError::NotFound(format!("账号不存在: {}", user_id)))?;

        if is_current {
            return Err(AppError::InvalidOperation("无法移除当前账号。请先切换到另一个账号。".to_string()));
        }

        conn.execute(
            "DELETE FROM user_auth WHERE user_id = ?1",
            [user_id],
        ).map_err(|e| AppError::DatabaseError(format!("移除账号失败: {}", e)))?;

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

        let (server_url, encrypted_refresh_token, user_id, email, device_id): (String, String, String, String, String) =
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

        // 2. 使用 device_id 解密 refresh_token
        let key = CryptoService::derive_key_from_device_id(&device_id);
        let refresh_token = CryptoService::decrypt_token(&encrypted_refresh_token, &key)?;

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
                AppError::NetworkError(format!("刷新请求失败: {}", e))
            })?;

        let status = response.status();

        // 先解析响应为 JSON Value（用于错误处理）
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            log::error!("Failed to parse refresh response: {}", e);
            AppError::NetworkError(format!("刷新响应无效: {}", e))
        })?;

        if !status.is_success() {
            // 解析错误消息
            let error_msg = response_json["error"]
                .as_str()
                .unwrap_or("未知错误");
            log::error!("Server returned error {}: {}", status, error_msg);
            return Err(AppError::AuthenticationError(error_msg.to_string()));
        }

        // 从数据库获取 device_id（用于构建 AuthResponse）
        let device_id: String = conn.query_row(
            "SELECT device_id FROM user_auth WHERE user_id = ?1",
            [&user_id],
            |row| row.get(0)
        ).map_err(|e| AppError::DatabaseError(format!("获取 device_id 失败: {}", e)))?;

        // 4. 解析服务器响应（直接读取 token 和 refresh_token）
        let new_access_token = response_json["token"]
            .as_str()
            .ok_or_else(|| AppError::AuthenticationError("刷新响应中缺少令牌".to_string()))?;

        let new_refresh_token = response_json["refresh_token"]
            .as_str()
            .ok_or_else(|| AppError::AuthenticationError("刷新响应中缺少 refresh_token".to_string()))?;

        // 5. 计算新的过期时间（7天后）
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + 7 * 24 * 3600;

        // 6. 更新数据库（加密存储新的 token）
        let encrypted_access = CryptoService::encrypt_token(new_access_token, &key)?;
        let encrypted_refresh = CryptoService::encrypt_token(new_refresh_token, &key)?;

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
        ).map_err(|e| AppError::DatabaseError(format!("更新令牌失败: {}", e)))?;

        log::info!("Access token refreshed successfully for user: {}", email);

        // 构建完整的 AuthResponse
        Ok(AuthResponse {
            token: new_access_token.to_string(),
            refresh_token: new_refresh_token.to_string(),
            user_id,
            email,
            device_id,
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
            "SELECT server_url, access_token_encrypted, device_id
             FROM user_auth
             WHERE is_current = 1"
        ).map_err(|e| AppError::DatabaseError(format!("获取认证信息失败: {}", e)))?;

        let (server_url, encrypted_token, device_id): (String, String, String) = stmt.query_row([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }).map_err(|_| AppError::NotAuthenticated("用户未登录".to_string()))?;

        // 使用 device_id 解密 token
        let key = CryptoService::derive_key_from_device_id(&device_id);
        let token = CryptoService::decrypt_token(&encrypted_token, &key)?;

        Ok((server_url, token))
    }

    /// 获取认证信息（包含 token 过期时间）
    pub fn get_auth_info_with_expires(&self) -> Result<(String, String, i64)> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT server_url, access_token_encrypted, token_expires_at, device_id
             FROM user_auth
             WHERE is_current = 1"
        ).map_err(|e| AppError::DatabaseError(format!("获取认证信息失败: {}", e)))?;

        let (server_url, encrypted_token, expires_at, device_id): (String, String, i64, String) = stmt.query_row([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        }).map_err(|_| AppError::NotAuthenticated("用户未登录".to_string()))?;

        // 使用 device_id 解密 token
        let key = CryptoService::derive_key_from_device_id(&device_id);
        let token = CryptoService::decrypt_token(&encrypted_token, &key)?;

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

        // 使用 device_id 加密 token
        log::debug!("[AuthService::save_user_auth] 开始加密 token");
        let key = CryptoService::derive_key_from_device_id(device_id);
        let encrypted_access = CryptoService::encrypt_token(access_token, &key)?;
        let encrypted_refresh = CryptoService::encrypt_token(refresh_token, &key)?;
        log::debug!("[AuthService::save_user_auth] token 加密完成");

        let now = chrono::Utc::now().timestamp();

        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 开启事务
        let tx = conn.unchecked_transaction()
            .map_err(|e| AppError::DatabaseError(format!("开始事务失败: {}", e)))?;

        log::debug!("[AuthService::save_user_auth] 准备插入数据库: user_id={}, device_id={}", user_id, device_id);

        // 将所有账号设为非当前
        tx.execute(
            "UPDATE user_auth SET is_current = 0",
            [],
        ).map_err(|e| AppError::DatabaseError(format!("重置当前标志失败: {}", e)))?;

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
        ).map_err(|e| AppError::DatabaseError(format!("保存认证信息失败: {}", e)))?;

        tx.commit()
            .map_err(|e| AppError::DatabaseError(format!("提交事务失败: {}", e)))?;

        log::info!("[AuthService::save_user_auth] 用户认证信息保存成功: user_id={}", user_id);
        Ok(())
    }

    /// 获取用户的 device_id（从 user_auth 表）
    /// 如果不存在则生成新的 device_id
    fn get_device_id_for_user(&self, user_id: &str) -> Result<String> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 尝试从 user_auth 表获取已有的 device_id
        let mut stmt = conn.prepare("SELECT device_id FROM user_auth WHERE user_id = ?1 LIMIT 1")?;

        match stmt.query_row([user_id], |row| row.get::<_, String>(0)) {
            Ok(device_id) => {
                log::debug!("[AuthService::get_device_id_for_user] 使用已存在的 device_id: {}", device_id);
                Ok(device_id)
            }
            Err(_) => {
                // 首次登录该用户，生成新的 device_id（由服务器注册时生成，这里先用临时 UUID）
                let device_id = Uuid::new_v4().to_string();
                log::info!("[AuthService::get_device_id_for_user] 生成新的临时 device_id: {}", device_id);
                Ok(device_id)
            }
        }
    }

    /// 删除账号（需要密码验证）
    pub async fn delete_account(&self, password: String) -> Result<()> {
        // 获取当前用户信息
        let user = self.get_current_user()?;
        let server_url = user.server_url;
        let access_token = self.get_access_token()?;

        let url = format!("{}/auth/delete", server_url.trim_end_matches('/'));

        log::info!("Deleting account at {}", url);

        let request_body = json!({
            "password": password
        });

        let response = self.client
            .delete(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to send delete account request: {}", e);
                AppError::NetworkError(format!("删除账号请求失败: {}", e))
            })?;

        let status = response.status();

        if !status.is_success() {
            let error_msg = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            log::error!("Server returned error {}: {}", status, error_msg);
            return Err(AppError::AuthenticationError(error_msg));
        }

        // 删除成功后，清除本地数据
        self.logout()?;

        log::info!("Account deleted successfully: user_id={}", user.id);
        Ok(())
    }
}
