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
