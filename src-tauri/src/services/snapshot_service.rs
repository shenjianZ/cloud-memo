use crate::models::{NoteSnapshot, CreateSnapshotRequest, SnapshotListItem};
use crate::models::error::{Result, AppError};
use uuid::Uuid;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use chrono::Utc;

/// 快照服务
///
/// 管理笔记的手动版本快照（不同步到云端）
pub struct SnapshotService {
    pool: Pool<SqliteConnectionManager>,
}

impl SnapshotService {
    /// 创建新的 SnapshotService 实例
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    /// 创建快照
    pub fn create_snapshot(&self, req: CreateSnapshotRequest) -> Result<NoteSnapshot> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        let snapshot = NoteSnapshot {
            id: id.clone(),
            note_id: req.note_id,
            title: req.title,
            content: req.content,
            snapshot_name: req.snapshot_name,
            created_at: now,
        };

        conn.execute(
            "INSERT INTO note_snapshots (id, note_id, title, content, snapshot_name, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&snapshot.id, &snapshot.note_id, &snapshot.title, &snapshot.content, &snapshot.snapshot_name, snapshot.created_at),
        ).map_err(|e| AppError::DatabaseError(format!("Failed to create snapshot: {}", e)))?;

        log::info!("Created snapshot {} for note {}", id, snapshot.note_id);
        Ok(snapshot)
    }

    /// 列出笔记的所有快照
    pub fn list_snapshots(&self, note_id: &str) -> Result<Vec<SnapshotListItem>> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, note_id, title, snapshot_name, created_at
             FROM note_snapshots
             WHERE note_id = ?1
             ORDER BY created_at DESC"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to list snapshots: {}", e)))?;

        let snapshots = stmt.query_map([note_id], |row| {
            Ok(SnapshotListItem {
                id: row.get(0)?,
                note_id: row.get(1)?,
                title: row.get(2)?,
                snapshot_name: row.get(3)?,
                created_at: row.get(4)?,
                created_at_display: format_datetime(row.get(4)?),
            })
        })
        .map_err(|e| AppError::DatabaseError(format!("Failed to parse snapshots: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect snapshots: {}", e)))?;

        Ok(snapshots)
    }

    /// 获取单个快照详情
    pub fn get_snapshot(&self, snapshot_id: &str) -> Result<NoteSnapshot> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, note_id, title, content, snapshot_name, created_at
             FROM note_snapshots
             WHERE id = ?1"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to get snapshot: {}", e)))?;

        let snapshot = stmt.query_row([snapshot_id], |row| {
            Ok(NoteSnapshot {
                id: row.get(0)?,
                note_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                snapshot_name: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).map_err(|e| AppError::NotFound(format!("Snapshot not found: {}", e)))?;

        Ok(snapshot)
    }

    /// 删除快照
    pub fn delete_snapshot(&self, snapshot_id: &str) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        conn.execute(
            "DELETE FROM note_snapshots WHERE id = ?1",
            [snapshot_id]
        ).map_err(|e| AppError::DatabaseError(format!("Failed to delete snapshot: {}", e)))?;

        log::info!("Deleted snapshot {}", snapshot_id);
        Ok(())
    }

    /// 从快照恢复笔记（返回快照内容，由调用者更新笔记）
    pub fn restore_from_snapshot(&self, snapshot_id: &str) -> Result<NoteSnapshot> {
        let snapshot = self.get_snapshot(snapshot_id)?;
        log::info!("Restored note {} from snapshot {}", snapshot.note_id, snapshot_id);
        Ok(snapshot)
    }

    /// 删除笔记的所有快照
    pub fn delete_note_snapshots(&self, note_id: &str) -> Result<usize> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        conn.execute(
            "DELETE FROM note_snapshots WHERE note_id = ?1",
            [note_id]
        ).map_err(|e| AppError::DatabaseError(format!("Failed to delete note snapshots: {}", e)))
    }
}

/// 格式化时间戳为可读字符串
fn format_datetime(timestamp: i64) -> String {
    let datetime = chrono::DateTime::<Utc>::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
