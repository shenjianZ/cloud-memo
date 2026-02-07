use crate::models::NoteSnapshot;
use crate::models::error::{Result, AppError};
use crate::database::DbPool;
use r2d2_sqlite::rusqlite::{self as rusqlite, params};

#[derive(Clone)]
pub struct SnapshotRepository {
    pool: DbPool,
}

impl SnapshotRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 根据 ID 获取快照
    pub fn find_by_id(&self, id: &str) -> Result<Option<NoteSnapshot>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, note_id, title, content, snapshot_name, created_at, server_ver, is_dirty, last_synced_at
             FROM note_snapshots WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id], |row| {
            Ok(NoteSnapshot {
                id: row.get(0)?,
                note_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                snapshot_name: row.get(4)?,
                created_at: row.get(5)?,
                server_ver: row.get(6)?,
                is_dirty: row.get(7)?,
                last_synced_at: row.get(8)?,
            })
        });

        match result {
            Ok(snapshot) => Ok(Some(snapshot)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    /// 获取笔记的所有快照
    pub fn find_by_note_id(&self, note_id: &str) -> Result<Vec<NoteSnapshot>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, note_id, title, content, snapshot_name, created_at, server_ver, is_dirty, last_synced_at
             FROM note_snapshots
             WHERE note_id = ?1
             ORDER BY created_at DESC"
        )?;

        let snapshots = stmt.query_map(params![note_id], |row| {
            Ok(NoteSnapshot {
                id: row.get(0)?,
                note_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                snapshot_name: row.get(4)?,
                created_at: row.get(5)?,
                server_ver: row.get(6)?,
                is_dirty: row.get(7)?,
                last_synced_at: row.get(8)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(snapshots)
    }

    /// 获取所有快照
    pub fn find_all(&self) -> Result<Vec<NoteSnapshot>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, note_id, title, content, snapshot_name, created_at, server_ver, is_dirty, last_synced_at
             FROM note_snapshots
             ORDER BY created_at DESC"
        )?;

        let snapshots = stmt.query_map([], |row| {
            Ok(NoteSnapshot {
                id: row.get(0)?,
                note_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                snapshot_name: row.get(4)?,
                created_at: row.get(5)?,
                server_ver: row.get(6)?,
                is_dirty: row.get(7)?,
                last_synced_at: row.get(8)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(snapshots)
    }
}
