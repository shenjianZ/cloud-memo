use anyhow::Result;
use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use crate::models::SyncLock;

/// 同步锁服务
/// 用于获取和释放同步操作锁，防止并发冲突
#[derive(Clone)]
pub struct SyncLockService {
    pool: MySqlPool,
}

/// 同步锁守卫（RAII 模式）
///
/// 当守卫被 drop 时，会自动释放锁。
/// 这确保了即使发生 panic 或早期返回，锁也能被正确释放。
pub struct SyncLockGuard {
    lock_id: Option<String>,
    user_id: String,
    service: SyncLockService,
}

impl SyncLockGuard {
    /// 创建一个新的守卫
    pub fn new(lock_id: String, user_id: String, service: SyncLockService) -> Self {
        Self {
            lock_id: Some(lock_id),
            user_id,
            service,
        }
    }

    /// 手动释放锁（可选）
    ///
    /// 如果已经释放，再次调用不会产生效果
    pub async fn release(mut self) {
        if let Some(lock_id) = self.lock_id.take() {
            let _ = self.service.release_lock(&lock_id, &self.user_id).await;
        }
    }
}

impl Drop for SyncLockGuard {
    fn drop(&mut self) {
        if let Some(lock_id) = self.lock_id.take() {
            let user_id = self.user_id.clone();
            let service = self.service.clone();

            // 在后台任务中异步释放锁
            // Drop trait 不能是 async，所以我们需要 spawn 一个任务
            tokio::spawn(async move {
                if let Err(e) = service.release_lock(&lock_id, &user_id).await {
                    tracing::warn!("释放同步锁失败: lock_id={}, error={}", lock_id, e);
                } else {
                    tracing::info!("自动释放同步锁: lock_id={}", lock_id);
                }
            });
        }
    }
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

    /// 获取同步操作锁，返回守卫（RAII 模式）
    ///
    /// 守卫会在 drop 时自动释放锁，推荐使用此方法而不是 acquire_lock
    pub async fn acquire_guard(
        &self,
        user_id: &str,
        device_id: &str,
        lock_duration_seconds: i64,
    ) -> Result<SyncLockGuard> {
        let lock_id = self.acquire_lock(user_id, device_id, lock_duration_seconds).await?;
        Ok(SyncLockGuard::new(lock_id, user_id.to_string(), self.clone()))
    }
}