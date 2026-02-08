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

    /// 获取当前工作空间 ID（基于当前用户的 is_current 标记）
    fn get_current_workspace_id(&self) -> Result<Option<String>> {
        let conn = self.pool.get()?;

        // 获取当前用户 ID
        let user_id: Option<String> = conn
            .query_row(
                "SELECT user_id FROM user_auth WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();

        let user_id = match user_id {
            Some(uid) => uid,
            None => return Ok(None),  // 未登录
        };

        // 查询该用户的当前工作空间（is_current = 1）
        let workspace_id: Option<String> = conn
            .query_row(
                "SELECT id FROM workspaces WHERE user_id = ? AND is_current = 1 AND is_deleted = 0 LIMIT 1",
                params![&user_id],
                |row| row.get(0),
            )
            .ok();

        Ok(workspace_id)
    }

    /// 根据 ID 获取快照
    pub fn find_by_id(&self, id: &str) -> Result<Option<NoteSnapshot>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, note_id, title, content, snapshot_name, created_at, workspace_id, server_ver, is_dirty, last_synced_at
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
                workspace_id: row.get(6)?,
                server_ver: row.get(7)?,
                is_dirty: row.get(8)?,
                last_synced_at: row.get(9)?,
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
        let workspace_id = self.get_current_workspace_id()?;
        let mut stmt = conn.prepare(
            "SELECT id, note_id, title, content, snapshot_name, created_at, workspace_id, server_ver, is_dirty, last_synced_at
             FROM note_snapshots
             WHERE note_id = ?1 AND (workspace_id = ?2 OR workspace_id IS NULL)
             ORDER BY created_at DESC"
        )?;

        let snapshots = stmt.query_map(params![note_id, workspace_id], |row| {
            Ok(NoteSnapshot {
                id: row.get(0)?,
                note_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                snapshot_name: row.get(4)?,
                created_at: row.get(5)?,
                workspace_id: row.get(6)?,
                server_ver: row.get(7)?,
                is_dirty: row.get(8)?,
                last_synced_at: row.get(9)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(snapshots)
    }

    /// 获取所有快照
    pub fn find_all(&self) -> Result<Vec<NoteSnapshot>> {
        let conn = self.pool.get()?;
        let workspace_id = self.get_current_workspace_id()?;
        let mut stmt = conn.prepare(
            "SELECT id, note_id, title, content, snapshot_name, created_at, workspace_id, server_ver, is_dirty, last_synced_at
             FROM note_snapshots
             WHERE (workspace_id = ?1 OR workspace_id IS NULL)
             ORDER BY created_at DESC"
        )?;

        let snapshots = stmt.query_map(params![workspace_id], |row| {
            Ok(NoteSnapshot {
                id: row.get(0)?,
                note_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                snapshot_name: row.get(4)?,
                created_at: row.get(5)?,
                workspace_id: row.get(6)?,
                server_ver: row.get(7)?,
                is_dirty: row.get(8)?,
                last_synced_at: row.get(9)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(snapshots)
    }
}
