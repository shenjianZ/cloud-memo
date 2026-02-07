use anyhow::Result;
use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::Utc;
use crate::models::SyncHistoryEntry;

/// 同步历史服务
pub struct SyncHistoryService {
    pool: MySqlPool,
}

impl SyncHistoryService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 创建同步历史记录
    ///
    /// 自动清理策略：
    /// - 保留最近 1000 条记录
    /// - 或保留最近 90 天的记录
    pub async fn create(
        &self,
        user_id: &str,
        sync_type: &str,
        pushed_count: i32,
        pulled_count: i32,
        conflict_count: i32,
        error: Option<String>,
        duration_ms: i64,
    ) -> Result<SyncHistoryEntry> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        sqlx::query(
            "INSERT INTO sync_history (id, user_id, sync_type, pushed_count, pulled_count, conflict_count, error, duration_ms, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(user_id)
        .bind(sync_type)
        .bind(pushed_count)
        .bind(pulled_count)
        .bind(conflict_count)
        .bind(&error)
        .bind(duration_ms)
        .bind(now)
        .execute(&self.pool)
        .await?;

        // 自动清理旧记录（在后台执行，不影响主流程）
        let pool = self.pool.clone();
        let user_id_clone = user_id.to_string();
        tokio::spawn(async move {
            if let Err(e) = Self::cleanup_old_records(&pool, &user_id_clone).await {
                tracing::warn!("清理同步历史失败: user_id={}, error={}", user_id_clone, e);
            }
        });

        Ok(SyncHistoryEntry {
            id,
            user_id: user_id.to_string(),
            sync_type: sync_type.to_string(),
            pushed_count,
            pulled_count,
            conflict_count,
            error,
            duration_ms,
            created_at: now,
        })
    }

    /// 清理旧的同步历史记录
    ///
    /// 清理策略：
    /// 1. 删除超过 90 天的记录
    /// 2. 如果记录数超过 1000 条，删除最旧的记录
    async fn cleanup_old_records(pool: &MySqlPool, user_id: &str) -> Result<()> {
        let now = Utc::now().timestamp();
        const MAX_RECORDS: i64 = 1000;
        const RETENTION_DAYS: i64 = 90;

        // 1. 删除超过 90 天的记录
        let cutoff_timestamp = now - (RETENTION_DAYS * 24 * 60 * 60);
        let deleted_old = sqlx::query(
            "DELETE FROM sync_history WHERE user_id = ? AND created_at < ?"
        )
        .bind(user_id)
        .bind(cutoff_timestamp)
        .execute(pool)
        .await?;

        if deleted_old.rows_affected() > 0 {
            tracing::info!(
                "清理超过 {} 天的同步历史: user_id={}, deleted={}",
                RETENTION_DAYS,
                user_id,
                deleted_old.rows_affected()
            );
        }

        // 2. 检查记录总数，如果超过 1000 条，删除最旧的
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sync_history WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        if count > MAX_RECORDS {
            let to_delete = count - MAX_RECORDS;

            // 删除最旧的记录（保留最新的 1000 条）
            let deleted_excess = sqlx::query(
                "DELETE FROM sync_history
                 WHERE user_id = ? AND id IN (
                     SELECT id FROM sync_history
                     WHERE user_id = ?
                     ORDER BY created_at ASC
                     LIMIT ?
                 )"
            )
            .bind(user_id)
            .bind(user_id)
            .bind(to_delete)
            .execute(pool)
            .await?;

            tracing::info!(
                "清理多余的同步历史: user_id={}, count={}, deleted={}",
                user_id,
                count,
                deleted_excess.rows_affected()
            );
        }

        Ok(())
    }

    /// 获取用户的同步历史记录
    pub async fn list(&self, user_id: &str, limit: usize) -> Result<Vec<SyncHistoryEntry>> {
        let limit = limit.min(100); // 最多 100 条

        let history = sqlx::query_as::<_, SyncHistoryEntry>(
            "SELECT * FROM sync_history
             WHERE user_id = ?
             ORDER BY created_at DESC
             LIMIT ?"
        )
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    /// 清空用户的同步历史
    pub async fn clear(&self, user_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM sync_history WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 删除指定时间之前的历史记录
    pub async fn delete_before(&self, user_id: &str, before_timestamp: i64) -> Result<u64> {
        let result = sqlx::query("DELETE FROM sync_history WHERE user_id = ? AND created_at < ?")
            .bind(user_id)
            .bind(before_timestamp)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
