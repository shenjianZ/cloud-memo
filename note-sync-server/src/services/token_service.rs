use anyhow::Result;
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, EncodingKey, Header};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use serde::{Serialize, Deserialize};

/// Token 服务，负责生成和验证 JWT token
pub struct TokenService;

impl TokenService {
    /// 生成 JWT access token
    pub fn generate_access_token(user_id: &str, expiration_days: i64, jwt_secret: &str) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(expiration_days))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration,
            token_type: TokenType::Access,
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref()))?;

        Ok(token)
    }

    /// 生成 refresh token
    pub fn generate_refresh_token(user_id: &str, jwt_secret: &str) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(30)) // refresh token 有效期 30 天
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration,
            token_type: TokenType::Refresh,
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref()))?;

        Ok(token)
    }

    /// 生成 access token 和 refresh token
    pub fn generate_token_pair(user_id: &str, expiration_days: i64, jwt_secret: &str) -> Result<(String, String)> {
        let access_token = Self::generate_access_token(user_id, expiration_days, jwt_secret)?;
        let refresh_token = Self::generate_refresh_token(user_id, jwt_secret)?;

        Ok((access_token, refresh_token))
    }

    /// 从 token 中提取 user_id
    pub fn extract_user_id(token: &str) -> Result<String> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("无效的 token 格式"));
        }

        // 解码 payload 部分（中间部分）
        let decoded = general_purpose::STANDARD.decode(parts[1])?;
        let payload_str = String::from_utf8(decoded)?;
        let claims: Claims = serde_json::from_str(&payload_str)?;

        Ok(claims.sub)
    }

    /// 生成 token 哈希（用于存储到数据库或黑名单）
    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let result = hasher.finalize();
        general_purpose::STANDARD.encode(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: String,    // user_id
    exp: usize,     // 过期时间
    token_type: TokenType,
}
