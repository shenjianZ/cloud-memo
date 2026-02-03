use crate::models::Note;
use crate::database::DbPool;
use crate::models::error::{Result, AppError};
use r2d2_sqlite::rusqlite::params;

/// 笔记数据访问层
///
/// 负责所有与笔记相关的数据库操作
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
                    word_count, read_time_minutes
             FROM notes
             WHERE id = ? AND is_deleted = 0"
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
                    word_count, read_time_minutes
             FROM notes
             WHERE is_deleted = 0
             ORDER BY updated_at DESC"
        )?;

        let notes = stmt.query_map([], |row| {
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
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()
          .map_err(AppError::Database)?;

        Ok(notes)
    }

    /// 创建新笔记
    pub fn create(&self, note: &Note) -> Result<Note> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO notes (id, title, content, excerpt, folder_id,
                              is_favorite, is_deleted, is_pinned, author,
                              created_at, updated_at, word_count, read_time_minutes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                note.id, note.title, note.content, note.excerpt, note.folder_id,
                note.is_favorite as i32, note.is_deleted as i32, note.is_pinned as i32,
                note.author, note.created_at, note.updated_at, note.word_count, note.read_time_minutes
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
                 updated_at = ?, word_count = ?, read_time_minutes = ?
             WHERE id = ?",
            params![
                note.title, note.content, note.excerpt, note.folder_id,
                note.is_favorite as i32, note.is_pinned as i32, note.author,
                note.updated_at, note.word_count, note.read_time_minutes,
                note.id
            ],
        )?;

        log::debug!("Note updated: {}", note.id);
        Ok(note.clone())
    }

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

    /// 全文搜索笔记
    pub fn search(&self, query: &str) -> Result<Vec<Note>> {
        let conn = self.pool.get()?;
        let search_query = format!("{}*", query); // FTS5 前缀搜索

        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, n.content, n.excerpt, n.markdown_cache, n.folder_id, n.is_favorite,
                    n.is_deleted, n.is_pinned, n.author, n.created_at, n.updated_at, n.deleted_at,
                    n.word_count, n.read_time_minutes
             FROM notes n
             JOIN notes_fts f ON n.id = f.note_id
             WHERE notes_fts MATCH ? AND n.is_deleted = 0
             ORDER BY n.updated_at DESC
             LIMIT 50"
        )?;

        let notes = stmt.query_map(params![search_query], |row| {
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
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()
          .map_err(AppError::Database)?;

        log::debug!("Search completed: {} results", notes.len());
        Ok(notes)
    }
}
