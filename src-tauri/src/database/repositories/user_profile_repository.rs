use crate::models::{UserProfile, error::{Result, AppError}};
use crate::database::DbPool;
use r2d2_sqlite::rusqlite::{self as rusqlite, Row, params};

pub struct UserProfileRepository {
    pool: DbPool,
}

impl UserProfileRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 根据 user_id 获取用户资料
    pub fn find_by_user_id(&self, user_id: &str) -> Result<Option<UserProfile>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, user_id, username, phone, qq, wechat, avatar_data, avatar_mime_type, bio, created_at, updated_at
             FROM user_profiles
             WHERE user_id = ?1"
        )?;

        let result = stmt.query_row(params![user_id], |row: &Row| {
            Ok(UserProfile {
                id: Some(row.get(0)?),
                user_id: row.get(1)?,
                username: row.get(2)?,
                phone: row.get(3)?,
                qq: row.get(4)?,
                wechat: row.get(5)?,
                avatar_data: row.get(6)?,
                avatar_mime_type: row.get(7)?,
                bio: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        });

        match result {
            Ok(profile) => Ok(Some(profile)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    /// 创建用户资料
    pub fn create(&self, profile: &UserProfile) -> Result<UserProfile> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO user_profiles (user_id, username, phone, qq, wechat, avatar_data, avatar_mime_type, bio, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                &profile.user_id,
                &profile.username,
                &profile.phone,
                &profile.qq,
                &profile.wechat,
                &profile.avatar_data,
                &profile.avatar_mime_type,
                &profile.bio,
                profile.created_at,
                profile.updated_at,
            ],
        )?;

        // 获取插入后的 ID
        let id = conn.last_insert_rowid();

        Ok(UserProfile {
            id: Some(id),
            user_id: profile.user_id.clone(),
            username: profile.username.clone(),
            phone: profile.phone.clone(),
            qq: profile.qq.clone(),
            wechat: profile.wechat.clone(),
            avatar_data: profile.avatar_data.clone(),
            avatar_mime_type: profile.avatar_mime_type.clone(),
            bio: profile.bio.clone(),
            created_at: profile.created_at,
            updated_at: profile.updated_at,
        })
    }

    /// 更新用户资料
    pub fn update(&self, profile: &UserProfile) -> Result<UserProfile> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE user_profiles
             SET username = ?1, phone = ?2, qq = ?3, wechat = ?4, avatar_data = ?5, avatar_mime_type = ?6, bio = ?7, updated_at = ?8
             WHERE user_id = ?9",
            params![
                &profile.username,
                &profile.phone,
                &profile.qq,
                &profile.wechat,
                &profile.avatar_data,
                &profile.avatar_mime_type,
                &profile.bio,
                profile.updated_at,
                &profile.user_id,
            ],
        )?;

        Ok(profile.clone())
    }

    /// 获取或创建用户资料（便捷方法）
    pub fn get_or_create(&self, user_id: &str) -> Result<UserProfile> {
        // 先尝试获取
        if let Some(profile) = self.find_by_user_id(user_id)? {
            return Ok(profile);
        }

        // 不存在则创建
        let new_profile = UserProfile::new(user_id.to_string());
        self.create(&new_profile)
    }
}
