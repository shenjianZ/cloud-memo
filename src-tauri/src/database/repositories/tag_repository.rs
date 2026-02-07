use crate::models::{Tag, CreateTagRequest, UpdateTagRequest, NoteTagRequest, error::{Result, AppError}};
use crate::database::DbPool;
use r2d2_sqlite::rusqlite::{self as rusqlite, params};

#[derive(Clone)]
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
            "SELECT id, name, color, created_at, updated_at, is_deleted, deleted_at, server_ver, is_dirty, last_synced_at
             FROM tags
             WHERE is_deleted = 0
             ORDER BY name"
        )?;

        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                is_deleted: row.get(5)?,
                deleted_at: row.get(6)?,
                server_ver: row.get(7)?,
                is_dirty: row.get(8)?,
                last_synced_at: row.get(9)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tags)
    }

    /// 根据 ID 获取标签
    pub fn find_by_id(&self, id: &str) -> Result<Option<Tag>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at, updated_at, is_deleted, deleted_at, server_ver, is_dirty, last_synced_at
             FROM tags WHERE id = ?1 AND is_deleted = 0"
        )?;

        let result = stmt.query_row(params![id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                is_deleted: row.get(5)?,
                deleted_at: row.get(6)?,
                server_ver: row.get(7)?,
                is_dirty: row.get(8)?,
                last_synced_at: row.get(9)?,
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
            "SELECT t.id, t.name, t.color, t.created_at, t.updated_at, t.is_deleted, t.deleted_at, t.server_ver, t.is_dirty, t.last_synced_at
             FROM tags t
             INNER JOIN note_tags nt ON t.id = nt.tag_id AND nt.is_deleted = 0
             WHERE nt.note_id = ?1 AND t.is_deleted = 0
             ORDER BY t.name"
        )?;

        let tags = stmt.query_map(params![note_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                is_deleted: row.get(5)?,
                deleted_at: row.get(6)?,
                server_ver: row.get(7)?,
                is_dirty: row.get(8)?,
                last_synced_at: row.get(9)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tags)
    }

    /// 获取笔记的标签关联（包含真实的 created_at）
    pub fn find_note_tag_relations(&self, note_id: &str) -> Result<Vec<crate::models::sync::NoteTagRelation>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT note_id, tag_id, created_at
             FROM note_tags
             WHERE note_id = ?1 AND is_deleted = 0"
        )?;

        let relations = stmt.query_map(params![note_id], |row| {
            Ok(crate::models::sync::NoteTagRelation {
                note_id: row.get(0)?,
                tag_id: row.get(1)?,
                user_id: String::new(),
                created_at: row.get(2)?,
                is_deleted: false,
                deleted_at: None,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(relations)
    }

    /// 创建标签
    pub fn create(&self, req: &CreateTagRequest) -> Result<Tag> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO tags (id, name, color, created_at, updated_at, is_deleted, deleted_at, server_ver, is_dirty, last_synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, NULL, 1, 1, NULL)",
            params![&id, &req.name, &req.color, now, now],
        )?;

        Ok(Tag {
            id,
            name: req.name.clone(),
            color: req.color.clone(),
            created_at: now,
            updated_at: now,
            is_deleted: false,
            deleted_at: None,
            server_ver: 1,
            is_dirty: true,
            last_synced_at: None,
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
            is_deleted: current.is_deleted,
            deleted_at: current.deleted_at,
            server_ver: current.server_ver,
            is_dirty: true,  // 更新后标记为脏
            last_synced_at: current.last_synced_at,
        };

        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE tags SET name = ?1, color = ?2, updated_at = ?3, is_dirty = 1 WHERE id = ?4",
            params![&updated.name, &updated.color, updated.updated_at, id],
        )?;

        Ok(updated)
    }

    /// 删除标签（软删除）
    ///
    /// 利用外键约束自动处理：
    /// - note_tags 表中的关联记录会被标记为已删除
    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp();

        // 软删除标签
        conn.execute(
            "UPDATE tags SET is_deleted = 1, deleted_at = ?, is_dirty = 1 WHERE id = ?",
            params![now, id],
        )?;

        // 同时软删除所有关联的 note_tags
        conn.execute(
            "UPDATE note_tags SET is_deleted = 1, deleted_at = ? WHERE tag_id = ?",
            params![now, id],
        )?;

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

    /// 硬删除标签（永久删除）
    ///
    /// ## 删除行为
    ///
    /// - 从 `tags` 表中物理删除记录
    /// - 外键约束会自动删除 `note_tags` 中的关联记录
    pub fn hard_delete(&self, id: &str) -> Result<()> {
        let conn = self.pool.get()?;

        let rows_affected = conn.execute(
            "DELETE FROM tags WHERE id = ?",
            params![id],
        )?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!("Tag {} not found", id)));
        }

        log::info!("[TagRepository] 硬删除标签: id={}", id);
        Ok(())
    }

    /// 批量硬删除标签
    ///
    /// ## 返回
    ///
    /// 返回成功删除的标签数量
    pub fn hard_delete_batch(&self, ids: &[String]) -> Result<i64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let conn = self.pool.get()?;

        let sql = format!(
            "DELETE FROM tags WHERE id IN ({})",
            ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
        );

        let params: Vec<&dyn r2d2_sqlite::rusqlite::ToSql> = ids.iter().map(|s| s as &dyn r2d2_sqlite::rusqlite::ToSql).collect();

        let rows_affected = conn.execute(&sql, params.as_slice())
            .map_err(AppError::Database)?;

        log::info!("[TagRepository] 批量硬删除标签: count={}", rows_affected);
        Ok(rows_affected as i64)
    }

    /// 清理超过指定天数的软删除标签
    ///
    /// ## 参数
    ///
    /// - `days`: 软删除后的保留天数（如 30 天）
    ///
    /// ## 返回
    ///
    /// 返回清理的标签数量
    pub fn purge_old_deleted_tags(&self, days: i64) -> Result<i64> {
        let conn = self.pool.get()?;
        let cutoff_time = chrono::Utc::now().timestamp() - (days * 86400);

        let rows_affected = conn.execute(
            "DELETE FROM tags WHERE is_deleted = 1 AND deleted_at < ?",
            params![cutoff_time],
        ).map_err(AppError::Database)?;

        log::info!("[TagRepository] 清理旧标签: days={}, count={}", days, rows_affected);
        Ok(rows_affected as i64)
    }
}
