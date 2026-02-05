use crate::db::DbPool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserProfile {
    pub id: String,
    pub user_id: String,
    pub username: Option<String>,
    pub phone: Option<String>,
    pub qq: Option<String>,
    pub wechat: Option<String>,
    pub avatar_data: Option<String>,  // 头像图片数据（Base64 编码）
    pub avatar_mime_type: Option<String>,  // 头像图片类型
    pub bio: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateProfileRequest {
    pub user_id: String,
    pub username: Option<String>,
    pub phone: Option<String>,
    pub qq: Option<String>,
    pub wechat: Option<String>,
    pub avatar_data: Option<String>,  // 头像图片数据（Base64 编码）
    pub avatar_mime_type: Option<String>,  // 头像图片类型
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub phone: Option<String>,
    pub qq: Option<String>,
    pub wechat: Option<String>,
    pub avatar_data: Option<String>,  // 头像图片数据（Base64 编码）
    pub avatar_mime_type: Option<String>,  // 头像图片类型
    pub bio: Option<String>,
}

pub struct ProfileService {
    pool: DbPool,
}

impl ProfileService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 获取用户资料
    pub async fn get_profile(&self, user_id: &str) -> Result<Option<UserProfile>> {
        let mut conn = self.pool.acquire().await?;

        let profile = sqlx::query_as::<_, UserProfile>(
            "SELECT id, user_id, username, phone, qq, wechat, avatar_data, avatar_mime_type, bio, created_at, updated_at
             FROM user_profiles
             WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&mut *conn)
        .await?;

        Ok(profile)
    }

    /// 创建用户资料
    pub async fn create_profile(&self, req: CreateProfileRequest) -> Result<UserProfile> {
        let mut conn = self.pool.acquire().await?;

        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            "INSERT INTO user_profiles (id, user_id, username, phone, qq, wechat, avatar_data, avatar_mime_type, bio, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&req.user_id)
        .bind(&req.username)
        .bind(&req.phone)
        .bind(&req.qq)
        .bind(&req.wechat)
        .bind(&req.avatar_data)
        .bind(&req.avatar_mime_type)
        .bind(&req.bio)
        .bind(now)
        .bind(now)
        .execute(&mut *conn)
        .await?;

        Ok(UserProfile {
            id,
            user_id: req.user_id,
            username: req.username,
            phone: req.phone,
            qq: req.qq,
            wechat: req.wechat,
            avatar_data: req.avatar_data,
            avatar_mime_type: req.avatar_mime_type,
            bio: req.bio,
            created_at: now,
            updated_at: now,
        })
    }

    /// 更新用户资料
    pub async fn update_profile(&self, user_id: &str, req: UpdateProfileRequest) -> Result<Option<UserProfile>> {
        let mut conn = self.pool.acquire().await?;

        // 检查资料是否存在
        let existing = self.get_profile(user_id).await?;

        if existing.is_none() {
            return Ok(None);
        }

        let now = chrono::Utc::now().timestamp();

        // 构建动态 UPDATE 语句
        let mut query = String::from("UPDATE user_profiles SET updated_at = ?");
        let mut param_count = 1;

        if req.username.is_some() {
            query.push_str(&format!(", username = ?"));
            param_count += 1;
        }
        if req.phone.is_some() {
            query.push_str(&format!(", phone = ?"));
            param_count += 1;
        }
        if req.qq.is_some() {
            query.push_str(&format!(", qq = ?"));
            param_count += 1;
        }
        if req.wechat.is_some() {
            query.push_str(&format!(", wechat = ?"));
            param_count += 1;
        }
        if req.avatar_data.is_some() {
            query.push_str(&format!(", avatar_data = ?"));
            param_count += 1;
        }
        if req.avatar_mime_type.is_some() {
            query.push_str(&format!(", avatar_mime_type = ?"));
            param_count += 1;
        }
        if req.bio.is_some() {
            query.push_str(&format!(", bio = ?"));
            param_count += 1;
        }

        query.push_str(" WHERE user_id = ?");

        // 使用宏构建动态查询（简化处理）
        let mut query_builder = sqlx::query(&query).bind(now);

        if let Some(v) = req.username {
            query_builder = query_builder.bind(v);
        }
        if let Some(v) = req.phone {
            query_builder = query_builder.bind(v);
        }
        if let Some(v) = req.qq {
            query_builder = query_builder.bind(v);
        }
        if let Some(v) = req.wechat {
            query_builder = query_builder.bind(v);
        }
        if let Some(v) = req.avatar_data {
            query_builder = query_builder.bind(v);
        }
        if let Some(v) = req.avatar_mime_type {
            query_builder = query_builder.bind(v);
        }
        if let Some(v) = req.bio {
            query_builder = query_builder.bind(v);
        }

        query_builder = query_builder.bind(user_id);

        query_builder.execute(&mut *conn).await?;

        // 返回更新后的资料
        self.get_profile(user_id).await
    }

    /// 上传或更新用户资料（用于同步）
    pub async fn upsert_profile(&self, user_id: &str, req: CreateProfileRequest) -> Result<UserProfile> {
        let mut conn = self.pool.acquire().await?;

        let now = chrono::Utc::now().timestamp();

        // 尝试更新
        let existing = self.get_profile(user_id).await?;

        if existing.is_some() {
            // 存在则更新
            let update_req = UpdateProfileRequest {
                username: req.username.clone(),
                phone: req.phone.clone(),
                qq: req.qq.clone(),
                wechat: req.wechat.clone(),
                avatar_data: req.avatar_data.clone(),
                avatar_mime_type: req.avatar_mime_type.clone(),
                bio: req.bio.clone(),
            };

            match self.update_profile(user_id, update_req).await? {
                Some(profile) => Ok(profile),
                None => {
                    // 更新失败，尝试创建
                    self.create_profile(req).await
                }
            }
        } else {
            // 不存在则创建
            self.create_profile(req).await
        }
    }
}
