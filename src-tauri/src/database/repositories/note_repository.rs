use crate::database::DbPool;
use crate::models::error::{AppError, Result};
use crate::models::Note;
use r2d2_sqlite::rusqlite::params;

/// 笔记数据访问层
///
/// 负责所有与笔记相关的数据库操作
#[derive(Clone)]
pub struct NoteRepository {
    pool: DbPool,
}

impl NoteRepository {
    /// 创建新的 NoteRepository 实例
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 根据 ID 查找笔记
    pub fn find_by_id(&self, id: &str) -> Result<Option<Note>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, content, excerpt, markdown_cache, folder_id, is_favorite,
                    is_deleted, is_pinned, author, created_at, updated_at, deleted_at,
                    word_count, read_time_minutes,
                    server_ver, is_dirty, last_synced_at
             FROM notes
             WHERE id = ? AND is_deleted = 0",
        )?;

        let note = stmt.query_row(params![id], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                excerpt: row.get(3)?,
                markdown_cache: row.get(4)?,
                folder_id: row.get(5)?,
                is_favorite: row.get(6)?,
                is_deleted: row.get(7)?,
                is_pinned: row.get(8)?,
                author: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                deleted_at: row.get(12)?,
                word_count: row.get(13)?,
                read_time_minutes: row.get(14)?,
                server_ver: row.get(15)?,
                is_dirty: row.get(16)?,
                last_synced_at: row.get(17)?,
            })
        });

        match note {
            Ok(n) => Ok(Some(n)),
            Err(r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    /// 查找所有笔记
    pub fn find_all(&self) -> Result<Vec<Note>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, content, excerpt, markdown_cache, folder_id, is_favorite,
                    is_deleted, is_pinned, author, created_at, updated_at, deleted_at,
                    word_count, read_time_minutes,
                    server_ver, is_dirty, last_synced_at
             FROM notes
             WHERE is_deleted = 0
             ORDER BY updated_at DESC",
        )?;

        let notes = stmt
            .query_map([], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    excerpt: row.get(3)?,
                    markdown_cache: row.get(4)?,
                    folder_id: row.get(5)?,
                    is_favorite: row.get(6)?,
                    is_deleted: row.get(7)?,
                    is_pinned: row.get(8)?,
                    author: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                    deleted_at: row.get(12)?,
                    word_count: row.get(13)?,
                    read_time_minutes: row.get(14)?,
                    server_ver: row.get(15)?,
                    is_dirty: row.get(16)?,
                    last_synced_at: row.get(17)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(AppError::Database)?;

        Ok(notes)
    }

    /// 查找所有已删除的笔记（回收站）
    ///
    /// ## 查询条件
    ///
    /// - 只返回 `is_deleted = 1` 的笔记
    /// - 按删除时间倒序排列（最新删除的在前）
    ///
    /// ## 返回
    ///
    /// 返回所有已删除的笔记列表，包含完整的笔记信息
    pub fn find_deleted(&self) -> Result<Vec<Note>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, content, excerpt, markdown_cache, folder_id, is_favorite,
                    is_deleted, is_pinned, author, created_at, updated_at, deleted_at,
                    word_count, read_time_minutes,
                    server_ver, is_dirty, last_synced_at
             FROM notes
             WHERE is_deleted = 1
             ORDER BY deleted_at DESC",
        )?;

        let notes = stmt
            .query_map([], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    excerpt: row.get(3)?,
                    markdown_cache: row.get(4)?,
                    folder_id: row.get(5)?,
                    is_favorite: row.get(6)?,
                    is_deleted: row.get(7)?,
                    is_pinned: row.get(8)?,
                    author: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                    deleted_at: row.get(12)?,
                    word_count: row.get(13)?,
                    read_time_minutes: row.get(14)?,
                    server_ver: row.get(15)?,
                    is_dirty: row.get(16)?,
                    last_synced_at: row.get(17)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(AppError::Database)?;

        log::debug!("Found {} deleted notes in trash", notes.len());
        Ok(notes)
    }

    /// 创建新笔记
    pub fn create(&self, note: &Note) -> Result<Note> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO notes (id, title, content, excerpt, folder_id,
                              is_favorite, is_deleted, is_pinned, author,
                              created_at, updated_at, word_count, read_time_minutes,
                              server_ver, is_dirty, last_synced_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                note.id,
                note.title,
                note.content,
                note.excerpt,
                note.folder_id,
                note.is_favorite as i32,
                note.is_deleted as i32,
                note.is_pinned as i32,
                note.author,
                note.created_at,
                note.updated_at,
                note.word_count,
                note.read_time_minutes,
                note.server_ver,
                note.is_dirty as i32,
                note.last_synced_at
            ],
        )?;

        log::debug!("Note created: {}", note.id);
        Ok(note.clone())
    }

    /// 更新笔记
    pub fn update(&self, note: &Note) -> Result<Note> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE notes
             SET title = ?, content = ?, excerpt = ?, folder_id = ?,
                 is_favorite = ?, is_pinned = ?, author = ?,
                 updated_at = ?, word_count = ?, read_time_minutes = ?,
                 is_dirty = ?
             WHERE id = ?",
            params![
                note.title,
                note.content,
                note.excerpt,
                note.folder_id,
                note.is_favorite as i32,
                note.is_pinned as i32,
                note.author,
                note.updated_at,
                note.word_count,
                note.read_time_minutes,
                note.is_dirty as i32,
                note.id
            ],
        )?;

        log::debug!("Note updated: {}", note.id);
        Ok(note.clone())
    }

    /// 软删除笔记
    /// 软删除笔记
    pub fn soft_delete(&self, id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "UPDATE notes SET is_deleted = 1, deleted_at = ? WHERE id = ?",
            params![now, id],
        )?;

        log::debug!("Note soft deleted: {}", id);
        Ok(())
    }

    /// 恢复已删除的笔记到"已恢复笔记"文件夹
    ///
    /// ## 恢复行为
    ///
    /// - 将 `is_deleted` 设为 `false`
    /// - 将 `deleted_at` 设为 `NULL`
    /// - 将 `folder_id` 设为"已恢复笔记"文件夹的 ID
    /// - 更新 `updated_at` 时间戳
    ///
    /// ## 参数
    ///
    /// - `id`: 笔记 ID
    /// - `recovered_folder_id`: "已恢复笔记"文件夹的 ID
    pub fn restore(&self, id: &str, recovered_folder_id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "UPDATE notes
             SET is_deleted = 0,
                 deleted_at = NULL,
                 folder_id = ?,
                 updated_at = ?
             WHERE id = ?",
            params![recovered_folder_id, now, id],
        )?;

        log::debug!("Note restored: {} -> folder: {}", id, recovered_folder_id);
        Ok(())
    }

    /// 全文搜索笔记
    pub fn search(&self, query: &str) -> Result<Vec<Note>> {
        let conn = self.pool.get()?;
        let search_query = format!("{}*", query); // FTS5 前缀搜索

        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, n.content, n.excerpt, n.markdown_cache, n.folder_id, n.is_favorite,
                    n.is_deleted, n.is_pinned, n.author, n.created_at, n.updated_at, n.deleted_at,
                    n.word_count, n.read_time_minutes,
                    n.server_ver, n.is_dirty, n.last_synced_at
             FROM notes n
             JOIN notes_fts f ON n.id = f.note_id
             WHERE notes_fts MATCH ? AND n.is_deleted = 0
             ORDER BY n.updated_at DESC
             LIMIT 50"
        )?;

        let notes = stmt
            .query_map(params![search_query], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    excerpt: row.get(3)?,
                    markdown_cache: row.get(4)?,
                    folder_id: row.get(5)?,
                    is_favorite: row.get(6)?,
                    is_deleted: row.get(7)?,
                    is_pinned: row.get(8)?,
                    author: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                    deleted_at: row.get(12)?,
                    word_count: row.get(13)?,
                    read_time_minutes: row.get(14)?,
                    server_ver: row.get(15)?,
                    is_dirty: row.get(16)?,
                    last_synced_at: row.get(17)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(AppError::Database)?;

        log::debug!("Search completed: {} results", notes.len());
        Ok(notes)
    }

    /// 统计笔记数量（不包括软删除的笔记）
    ///
    /// ## 返回
    ///
    /// 返回 `is_deleted = 0` 的笔记总数
    pub fn count(&self) -> Result<i64> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM notes WHERE is_deleted = 0",
            [],
            |row| row.get(0),
        )?;

        log::debug!("Note count: {}", count);
        Ok(count)
    }

    /// 硬删除笔记（永久删除，不可恢复）
    ///
    /// ## 删除行为
    ///
    /// - 从 `notes` 表中物理删除记录
    /// - FTS 触发器会自动删除 `notes_fts` 中的索引
    /// - 外键约束会自动删除 `note_tags` 中的关联记录
    /// - **不会触发同步**（硬删除的数据不再同步）
    ///
    /// ## 安全性
    ///
    /// - ⚠️ 此操作不可逆，请谨慎使用
    /// - ✅ FTS 索引会自动同步删除（触发器: `notes_ad`）
    /// - ✅ 笔记标签关联会自动级联删除（外键: `ON DELETE CASCADE`）
    pub fn hard_delete(&self, id: &str) -> Result<()> {
        let conn = self.pool.get()?;

        let rows_affected = conn.execute(
            "DELETE FROM notes WHERE id = ?",
            params![id],
        )?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!("Note {} not found", id)));
        }

        log::info!("[NoteRepository] 硬删除笔记: id={}", id);
        Ok(())
    }

    /// 批量硬删除笔记
    ///
    /// ## 返回
    ///
    /// 返回成功删除的笔记数量
    pub fn hard_delete_batch(&self, ids: &[String]) -> Result<i64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let conn = self.pool.get()?;

        // 使用 IN 批量删除
        let sql = format!(
            "DELETE FROM notes WHERE id IN ({})",
            ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
        );

        let params: Vec<&dyn r2d2_sqlite::rusqlite::ToSql> = ids.iter().map(|s| s as &dyn r2d2_sqlite::rusqlite::ToSql).collect();

        let rows_affected = conn.execute(&sql, params.as_slice())
            .map_err(AppError::Database)?;

        log::info!("[NoteRepository] 批量硬删除笔记: count={}", rows_affected);
        Ok(rows_affected as i64)
    }

    /// 清理超过指定天数的软删除笔记
    ///
    /// ## 参数
    ///
    /// - `days`: 软删除后的保留天数（如 30 天）
    ///
    /// ## 返回
    ///
    /// 返回清理的笔记数量
    pub fn purge_old_deleted_notes(&self, days: i64) -> Result<i64> {
        let conn = self.pool.get()?;
        let cutoff_time = chrono::Utc::now().timestamp() - (days * 86400);

        let rows_affected = conn.execute(
            "DELETE FROM notes WHERE is_deleted = 1 AND deleted_at < ?",
            params![cutoff_time],
        ).map_err(AppError::Database)?;

        log::info!("[NoteRepository] 清理旧笔记: days={}, count={}", days, rows_affected);
        Ok(rows_affected as i64)
    }
}
