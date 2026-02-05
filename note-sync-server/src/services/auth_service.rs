use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::{Utc, Duration};
use anyhow::Result;
use sqlx::{MySqlPool};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

use crate::models::User;

pub struct AuthService {
    pool: MySqlPool,
}

impl AuthService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn register(&self, email: &str, password: &str) -> Result<User> {
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

        // 2. 使用 Argon2 哈希密码（推荐，比 bcrypt 更安全）
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("密码哈希失败: {}", e))?
            .to_string();

        // 3. 创建用户
        let user_id = uuid::Uuid::new_v4().to_string();
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

        Ok(User {
            id: user_id,
            email: email.to_string(),
            created_at: now,
        })
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

        // 2. 验证密码（使用 Argon2）
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

        // 3. 生成 JWT access_token（短期，7天）
        let config = crate::config::AppConfig::load()?;
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(config.auth.jwt_expiration_days))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = serde_json::json!({
            "sub": user.id.clone(),
            "exp": expiration,
        });

        let jwt_secret = config.auth.jwt_secret;
        let access_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_ref()),
        )?;

        // 4. 生成 refresh_token（长期，30天）
        let refresh_token = self.generate_refresh_token(&user.id, device_id.unwrap_or_else(|| "default".to_string())).await?;

        Ok((user, access_token, refresh_token))
    }

    /// 生成 refresh_token 并存储到数据库
    async fn generate_refresh_token(&self, user_id: &str, device_id: String) -> Result<String> {
        // 生成随机 refresh_token
        let refresh_token = uuid::Uuid::new_v4().to_string();

        // 计算哈希存储（SHA256 + base64）
        let mut hasher = Sha256::new();
        hasher.update(refresh_token.as_bytes());
        let token_hash = general_purpose::STANDARD.encode(hasher.finalize());

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

        Ok(refresh_token)
    }

    /// 使用 refresh_token 刷新 access_token
    pub async fn refresh_access_token(&self, refresh_token: &str, device_id: String) -> Result<(String, String)> {
        // 1. 计算 refresh_token 的哈希
        let mut hasher = Sha256::new();
        hasher.update(refresh_token.as_bytes());
        let token_hash = general_purpose::STANDARD.encode(hasher.finalize());

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

        // 5. 生成新的 access_token
        let config = crate::config::AppConfig::load()?;
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(config.auth.jwt_expiration_days))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = serde_json::json!({
            "sub": user.id,
            "exp": expiration,
        });

        let jwt_secret = config.auth.jwt_secret;
        let access_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_ref()),
        )?;

        // 6. 生成新的 refresh_token（轮换策略）
        let new_refresh_token = self.generate_refresh_token(&user_id, device_id).await?;

        Ok((access_token, new_refresh_token))
    }
}
