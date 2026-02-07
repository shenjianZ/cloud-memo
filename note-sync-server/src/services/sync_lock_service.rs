use anyhow::Result;
use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::Utc;
use crate::models::SyncLock;

/// 同步锁服务
/// 用于获取和释放同步操作锁，防止并发冲突
pub struct SyncLockService {
    pool: MySqlPool,
}

impl SyncLockService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 获取同步操作锁
    /// 如果锁已被其他设备持有且未过期，返回 Err
    /// 成功获取锁后，返回锁 ID
    pub async fn acquire_lock(
        &self,
        user_id: &str,
        device_id: &str,
        lock_duration_seconds: i64,
    ) -> Result<String> {
        let now = Utc::now().timestamp();
        let expires_at = now + lock_duration_seconds;
        let lock_id = Uuid::new_v4().to_string();

        // 首先清理过期的锁
        sqlx::query(
            "DELETE FROM sync_locks WHERE expires_at < ?"
        )
        .bind(now)
        .execute(&self.pool)
        .await?;

        // 检查是否已有该用户的锁
        let existing_lock: Option<SyncLock> = sqlx::query_as::<_, SyncLock>(
            "SELECT * FROM sync_locks WHERE user_id = ? AND device_id = ? AND expires_at > ?"
        )
        .bind(user_id)
        .bind(device_id)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(lock) = existing_lock {
            // 该设备已持有锁，更新过期时间
            sqlx::query(
                "UPDATE sync_locks SET expires_at = ? WHERE id = ?"
            )
            .bind(expires_at)
            .bind(&lock.id)
            .execute(&self.pool)
            .await?;
            Ok(lock.id)
        } else {
            // 检查是否有其他设备持有锁
            let other_device_lock: Option<SyncLock> = sqlx::query_as::<_, SyncLock>(
                "SELECT * FROM sync_locks WHERE user_id = ? AND device_id != ? AND expires_at > ?"
            )
            .bind(user_id)
            .bind(device_id)
            .bind(now)
            .fetch_optional(&self.pool)
            .await?;

            if other_device_lock.is_some() {
                return Err(anyhow::anyhow!("同步锁已被其他设备持有"));
            }

            // 创建新锁
            sqlx::query(
                "INSERT INTO sync_locks (id, user_id, device_id, acquired_at, expires_at)
                 VALUES (?, ?, ?, ?, ?)"
            )
            .bind(&lock_id)
            .bind(user_id)
            .bind(device_id)
            .bind(now)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;

            Ok(lock_id)
        }
    }

    /// 释放同步操作锁
    pub async fn release_lock(&self, lock_id: &str, user_id: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM sync_locks WHERE id = ? AND user_id = ?"
        )
        .bind(lock_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 清理所有过期的锁
    pub async fn cleanup_expired_locks(&self) -> Result<()> {
        let now = Utc::now().timestamp();
        sqlx::query(
            "DELETE FROM sync_locks WHERE expires_at < ?"
        )
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}