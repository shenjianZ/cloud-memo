use crate::models::{Tag, CreateTagRequest, UpdateTagRequest, NoteTagRequest, error::{Result, AppError}};
use crate::database::DbPool;
use r2d2_sqlite::rusqlite::{self as rusqlite, Row, params};

pub struct TagRepository {
    pool: DbPool,
}

impl TagRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 获取所有标签
    pub fn find_all(&self) -> Result<Vec<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at, updated_at
             FROM tags
             ORDER BY name"
        )?;

        let tags = stmt.query_map([], |row: &Row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tags)
    }

    /// 根据 ID 获取标签
    pub fn find_by_id(&self, id: &str) -> Result<Option<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at, updated_at
             FROM tags WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id], |row: &Row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        });

        match result {
            Ok(tag) => Ok(Some(tag)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    /// 获取笔记的所有标签
    pub fn find_by_note_id(&self, note_id: &str) -> Result<Vec<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at, t.updated_at
             FROM tags t
             INNER JOIN note_tags nt ON t.id = nt.tag_id
             WHERE nt.note_id = ?1
             ORDER BY t.name"
        )?;

        let tags = stmt.query_map(params![note_id], |row: &Row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tags)
    }

    /// 创建标签
    pub fn create(&self, req: &CreateTagRequest) -> Result<Tag> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO tags (id, name, color, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&id, &req.name, &req.color, now, now],
        )?;

        Ok(Tag {
            id,
            name: req.name.clone(),
            color: req.color.clone(),
            created_at: now,
            updated_at: now,
        })
    }

    /// 更新标签
    pub fn update(&self, id: &str, req: &UpdateTagRequest) -> Result<Tag> {
        let current = self.find_by_id(id)?
            .ok_or(AppError::Internal(format!("Tag {} not found", id)))?;

        let updated = Tag {
            id: id.to_string(),
            name: req.name.clone().unwrap_or(current.name),
            color: req.color.clone().or(current.color),
            created_at: current.created_at,
            updated_at: chrono::Utc::now().timestamp(),
        };

        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE tags SET name = ?1, color = ?2, updated_at = ?3 WHERE id = ?4",
            params![&updated.name, &updated.color, updated.updated_at, id],
        )?;

        Ok(updated)
    }

    /// 删除标签
    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM tags WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// 为笔记添加标签
    pub fn add_tag_to_note(&self, req: &NoteTagRequest) -> Result<()> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR IGNORE INTO note_tags (note_id, tag_id, created_at)
             VALUES (?1, ?2, ?3)",
            params![&req.note_id, &req.tag_id, now],
        )?;
        Ok(())
    }

    /// 从笔记移除标签
    pub fn remove_tag_from_note(&self, note_id: &str, tag_id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM note_tags WHERE note_id = ?1 AND tag_id = ?2",
            params![note_id, tag_id],
        )?;
        Ok(())
    }

    /// 设置笔记的标签（替换所有标签）
    pub fn set_note_tags(&self, note_id: &str, tag_ids: &[String]) -> Result<()> {
        let conn = self.pool.get()?;

        // 先删除现有标签
        conn.execute(
            "DELETE FROM note_tags WHERE note_id = ?1",
            params![note_id],
        )?;

        // 添加新标签
        let now = chrono::Utc::now().timestamp();
        for tag_id in tag_ids {
            conn.execute(
                "INSERT INTO note_tags (note_id, tag_id, created_at)
                 VALUES (?1, ?2, ?3)",
                params![note_id, tag_id, now],
            )?;
        }

        Ok(())
    }
}
