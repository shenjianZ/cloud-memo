use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Utc, Duration};
use anyhow::Result;
use sqlx::MySqlPool;
use rand::Rng;

use crate::models::User;
use super::token_service::TokenService;

pub struct AuthService {
    pool: MySqlPool,
}

impl AuthService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 检查邮箱是否已存在
    pub async fn check_email_exists(&self, email: &str) -> Result<bool> {
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(existing > 0)
    }

    /// 哈希密码（同步操作）
    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("密码哈希失败: {}", e))?
            .to_string();
        Ok(password_hash)
    }

    /// 生成用户 ID（同步操作，不保证唯一性）
    pub fn generate_user_id(&self) -> String {
        let mut rng = rand::thread_rng();
        rng.gen_range(1_000_000_000i64..10_000_000_000i64).to_string()
    }

    /// 生成唯一的用户 ID（确保数据库中不存在）
    pub async fn generate_unique_user_id(&self) -> Result<String> {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 10;

        loop {
            // 生成候选 ID
            let candidate_id = self.generate_user_id();

            // 检查是否已存在
            let existing = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE id = ?"
            )
            .bind(&candidate_id)
            .fetch_one(&self.pool)
            .await?;

            if existing == 0 {
                // ID 不存在，可以使用
                return Ok(candidate_id);
            }

            // ID 已存在，重试
            attempts += 1;
            if attempts >= MAX_ATTEMPTS {
                return Err(anyhow::anyhow!("生成唯一用户 ID 失败：已达到最大重试次数"));
            }
        }
    }

    /// 创建用户（返回用户 ID）
    pub async fn create_user(&self, email: &str, password_hash: &str, user_id: &str) -> Result<i64> {
        let now = Utc::now().timestamp();

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at)
             VALUES (?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(email)
        .bind(password_hash)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(now)
    }

    /// 完成注册后的 token 生成
    pub async fn complete_registration(&self, user_id: &str, email: &str, created_at: i64, device_id: Option<String>) -> Result<(User, String, String)> {
        // 生成 token
        let config = crate::config::AppConfig::load(None)?;
        let (access_token, refresh_token) = TokenService::generate_token_pair(user_id, config.auth.jwt_expiration_days, &config.auth.jwt_secret)?;

        // 保存 refresh_token 到数据库
        self.save_refresh_token(user_id, &refresh_token, device_id.unwrap_or_else(|| "default".to_string())).await?;

        let user = User {
            id: user_id.to_string(),
            email: email.to_string(),
            created_at,
        };

        Ok((user, access_token, refresh_token))
    }

    /// 注册用户并返回用户信息和 token
    pub async fn register(&self, email: &str, password: &str, device_id: Option<String>) -> Result<(User, String, String)> {
        // 1. 检查邮箱是否已存在
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        if existing > 0 {
            return Err(anyhow::anyhow!("邮箱已注册"));
        }

        // 2. 哈希密码（同步操作）
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("密码哈希失败: {}", e))?
            .to_string();

        // 3. 生成用户 ID（同步操作）
        let mut rng = rand::thread_rng();
        let user_id = rng.gen_range(1_000_000_000i64..10_000_000_000i64).to_string();
        let now = Utc::now().timestamp();

        // 4. 插入数据库
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at)
             VALUES (?, ?, ?, ?)"
        )
        .bind(&user_id)
        .bind(email)
        .bind(&password_hash)
        .bind(now)
        .execute(&self.pool)
        .await?;

        // 5. 创建 User 对象
        let user = User {
            id: user_id.clone(),
            email: email.to_string(),
            created_at: now,
        };

        // 6. 生成并保存 token
        let config = crate::config::AppConfig::load(None)?;
        let (access_token, refresh_token) = TokenService::generate_token_pair(&user_id, config.auth.jwt_expiration_days, &config.auth.jwt_secret)?;
        self.save_refresh_token(&user_id, &refresh_token, device_id.unwrap_or_else(|| "default".to_string())).await?;

        Ok((user, access_token, refresh_token))
    }

    pub async fn login(&self, email: &str, password: &str, device_id: Option<String>) -> Result<(User, String, String)> {
        // 1. 查询用户
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, created_at FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("邮箱或密码错误"))?;

        // 2. 验证密码
        let password_hash: String = sqlx::query_scalar(
            "SELECT password_hash FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        let parsed_hash = PasswordHash::new(&password_hash)
            .map_err(|e| anyhow::anyhow!("解析密码哈希失败: {}", e))?;
        let argon2 = Argon2::default();

        argon2.verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow::anyhow!("邮箱或密码错误"))?;

        // 3. 生成 token
        let config = crate::config::AppConfig::load(None)?;
        let (access_token, refresh_token) = TokenService::generate_token_pair(&user.id, config.auth.jwt_expiration_days, &config.auth.jwt_secret)?;

        // 4. 保存 refresh_token 到数据库
        self.save_refresh_token(&user.id, &refresh_token, device_id.unwrap_or_else(|| "default".to_string())).await?;

        Ok((user, access_token, refresh_token))
    }

    /// 保存 refresh_token 到数据库
    async fn save_refresh_token(&self, user_id: &str, refresh_token: &str, device_id: String) -> Result<()> {
        // 计算 refresh_token 的哈希
        let token_hash = TokenService::hash_token(refresh_token);

        // 设置过期时间（30天）
        let expires_at = Utc::now()
            .checked_add_signed(Duration::days(30))
            .expect("valid timestamp")
            .timestamp();

        let now = Utc::now().timestamp();
        let id = uuid::Uuid::new_v4().to_string();

        // 删除该设备的旧 refresh_token
        sqlx::query(
            "DELETE FROM refresh_tokens WHERE user_id = ? AND device_id = ?"
        )
        .bind(user_id)
        .bind(&device_id)
        .execute(&self.pool)
        .await?;

        // 插入新的 refresh_token
        sqlx::query(
            "INSERT INTO refresh_tokens (id, user_id, token_hash, device_id, expires_at, created_at)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(user_id)
        .bind(&token_hash)
        .bind(&device_id)
        .bind(expires_at)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 使用 refresh_token 刷新 access_token
    pub async fn refresh_access_token(&self, refresh_token: &str, device_id: String) -> Result<(String, String)> {
        // 1. 计算 refresh_token 的哈希
        let token_hash = TokenService::hash_token(refresh_token);

        // 2. 查询 refresh_token 记录
        let (user_id, expires_at): (String, i64) = sqlx::query_as(
            "SELECT user_id, expires_at FROM refresh_tokens WHERE token_hash = ? AND device_id = ?"
        )
        .bind(&token_hash)
        .bind(&device_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("无效的刷新令牌"))?;

        // 3. 检查是否过期
        let now = Utc::now().timestamp();
        if expires_at < now {
            // 删除过期的 refresh_token
            sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = ?")
                .bind(&token_hash)
                .execute(&self.pool)
                .await?;
            return Err(anyhow::anyhow!("刷新令牌已过期"));
        }

        // 4. 获取用户信息
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, created_at FROM users WHERE id = ?"
        )
        .bind(&user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;

        // 5. 生成新的 token
        let config = crate::config::AppConfig::load(None)?;
        let (access_token, new_refresh_token) = TokenService::generate_token_pair(&user.id, config.auth.jwt_expiration_days, &config.auth.jwt_secret)?;

        // 6. 保存新的 refresh_token（轮换策略）
        self.save_refresh_token(&user_id, &new_refresh_token, device_id).await?;

        Ok((access_token, new_refresh_token))
    }

    /// 删除用户账号（级联删除所有相关数据）
    pub async fn delete_user(&self, user_id: &str, password: &str) -> Result<()> {
        // 1. 验证密码
        let password_hash: String = sqlx::query_scalar(
            "SELECT password_hash FROM users WHERE id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;

        let parsed_hash = PasswordHash::new(&password_hash)
            .map_err(|e| anyhow::anyhow!("解析密码哈希失败: {}", e))?;
        let argon2 = Argon2::default();

        argon2.verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow::anyhow!("密码错误"))?;

        // 2. 删除用户（外键会级联删除所有相关数据）
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
