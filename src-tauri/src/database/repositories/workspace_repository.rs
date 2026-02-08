use crate::models::Workspace;
use crate::database::DbPool;
use crate::models::error::{Result, AppError};
use r2d2_sqlite::rusqlite::params;

/// 工作空间数据访问层
///
/// 负责所有与工作空间相关的数据库操作
#[derive(Clone)]
pub struct WorkspaceRepository {
    pool: DbPool,
}

impl WorkspaceRepository {
    /// 统一的 SQL 查询字段列表
    /// 字段顺序必须与 Workspace 结构体初始化顺序一致
    const SELECT_FIELDS: &'static str =
        "id, user_id, name, description, icon, color, is_default, is_current, sort_order,
         created_at, updated_at, is_deleted, deleted_at, server_ver, is_dirty, last_synced_at";

    /// 创建新的 WorkspaceRepository 实例
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 创建工作空间
    pub fn create(&self, workspace: &Workspace) -> Result<Workspace> {
        let conn = self.pool.get()?;

        conn.execute(
            "INSERT INTO workspaces (id, user_id, name, description, icon, color, is_default, is_current, sort_order,
                                   is_deleted, deleted_at, created_at, updated_at, server_ver, is_dirty, last_synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                &workspace.id,
                &workspace.user_id,
                &workspace.name,
                &workspace.description,
                &workspace.icon,
                &workspace.color,
                &workspace.is_default,
                &workspace.is_current,
                &workspace.sort_order,
                &workspace.is_deleted,
                &workspace.deleted_at,
                &workspace.created_at,
                &workspace.updated_at,
                &workspace.server_ver,
                &workspace.is_dirty,
                &workspace.last_synced_at,
            ],
        ).map_err(AppError::Database)?;

        Ok(workspace.clone())
    }

    /// 根据 ID 查找工作空间
    pub fn find_by_id(&self, id: &str) -> Result<Option<Workspace>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM workspaces WHERE id = ?",
            Self::SELECT_FIELDS
        ))?;

        let workspace = stmt.query_row(params![id], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                icon: row.get(4)?,
                color: row.get(5)?,
                is_default: row.get(6)?,
                is_current: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                is_deleted: row.get(11)?,
                deleted_at: row.get(12)?,
                server_ver: row.get(13)?,
                is_dirty: row.get(14)?,
                last_synced_at: row.get(15)?,
            })
        });

        match workspace {
            Ok(w) => Ok(Some(w)),
            Err(r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    /// 查找所有工作空间（未删除）
    pub fn find_all(&self) -> Result<Vec<Workspace>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM workspaces WHERE is_deleted = 0 ORDER BY sort_order ASC, created_at ASC",
            Self::SELECT_FIELDS
        ))?;

        let workspaces = stmt.query_map([], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                icon: row.get(4)?,
                color: row.get(5)?,
                is_default: row.get(6)?,
                is_current: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                is_deleted: row.get(11)?,
                deleted_at: row.get(12)?,
                server_ver: row.get(13)?,
                is_dirty: row.get(14)?,
                last_synced_at: row.get(15)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()
          .map_err(AppError::Database)?;

        Ok(workspaces)
    }

    /// 根据用户 ID 查找工作空间
    pub fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Workspace>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM workspaces WHERE user_id = ? AND is_deleted = 0 ORDER BY sort_order ASC, created_at ASC",
            Self::SELECT_FIELDS
        ))?;

        let workspaces = stmt.query_map(params![user_id], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                icon: row.get(4)?,
                color: row.get(5)?,
                is_default: row.get(6)?,
                is_current: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                is_deleted: row.get(11)?,
                deleted_at: row.get(12)?,
                server_ver: row.get(13)?,
                is_dirty: row.get(14)?,
                last_synced_at: row.get(15)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()
          .map_err(AppError::Database)?;

        Ok(workspaces)
    }

    /// 查找用户的默认工作空间
    pub fn find_default_by_user_id(&self, user_id: &str) -> Result<Option<Workspace>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM workspaces WHERE user_id = ? AND is_default = 1 AND is_deleted = 0 LIMIT 1",
            Self::SELECT_FIELDS
        ))?;

        let workspace = stmt.query_row(params![user_id], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                icon: row.get(4)?,
                color: row.get(5)?,
                is_default: row.get(6)?,
                is_current: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                is_deleted: row.get(11)?,
                deleted_at: row.get(12)?,
                server_ver: row.get(13)?,
                is_dirty: row.get(14)?,
                last_synced_at: row.get(15)?,
            })
        });

        match workspace {
            Ok(w) => Ok(Some(w)),
            Err(r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    /// 更新工作空间
    pub fn update(&self, workspace: &Workspace) -> Result<Workspace> {
        let conn = self.pool.get()?;

        conn.execute(
            "UPDATE workspaces
             SET name = ?1, description = ?2, icon = ?3, color = ?4,
                 is_default = ?5, is_current = ?6, sort_order = ?7, updated_at = ?8,
                 is_deleted = ?9, deleted_at = ?10, server_ver = ?11,
                 is_dirty = ?12, last_synced_at = ?13
             WHERE id = ?14",
            params![
                &workspace.name,
                &workspace.description,
                &workspace.icon,
                &workspace.color,
                &workspace.is_default,
                &workspace.is_current,
                &workspace.sort_order,
                &workspace.updated_at,
                &workspace.is_deleted,
                &workspace.deleted_at,
                &workspace.server_ver,
                &workspace.is_dirty,
                &workspace.last_synced_at,
                &workspace.id,
            ],
        ).map_err(AppError::Database)?;

        Ok(workspace.clone())
    }

    /// 删除工作空间（软删除）
    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp();

        // 获取用户 ID（用于查找默认工作空间）
        let user_id: Option<String> = conn
            .query_row(
                "SELECT user_id FROM workspaces WHERE id = ?",
                params![id],
                |row| row.get(0),
            )
            .ok();

        // 检查是否是当前工作空间（通过 is_current 字段）
        let is_current: bool = conn
            .query_row(
                "SELECT is_current FROM workspaces WHERE id = ?",
                params![id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        // 执行软删除（同时清除 is_current 标记）
        conn.execute(
            "UPDATE workspaces SET is_deleted = 1, deleted_at = ?, is_dirty = 1, is_current = 0 WHERE id = ?",
            params![now, id],
        ).map_err(AppError::Database)?;

        // 如果删除的是当前工作空间，切换到默认工作空间
        if is_current {
            if let Some(uid) = user_id {
                // 查找用户的默认工作空间
                let default_workspace: Option<String> = conn
                    .query_row(
                        "SELECT id FROM workspaces WHERE user_id = ? AND is_default = 1 AND is_deleted = 0 LIMIT 1",
                        params![uid],
                        |row| row.get(0),
                    )
                    .ok();

                if let Some(default_id) = default_workspace {
                    // 切换到默认工作空间
                    conn.execute(
                        "UPDATE workspaces SET is_current = 1 WHERE id = ?",
                        params![&default_id],
                    ).map_err(AppError::Database)?;
                }
                // 如果没有默认工作空间，则不做任何操作（用户下次需要手动选择）
            }
        }

        Ok(())
    }

    /// 设置默认工作空间
    pub fn set_default(&self, user_id: &str, workspace_id: &str) -> Result<()> {
        let conn = self.pool.get()?;

        // 开始事务
        let tx = conn.unchecked_transaction().map_err(AppError::Database)?;

        // 取消所有默认空间
        tx.execute(
            "UPDATE workspaces SET is_default = 0 WHERE user_id = ?",
            params![user_id],
        ).map_err(AppError::Database)?;

        // 设置新的默认空间
        tx.execute(
            "UPDATE workspaces SET is_default = 1 WHERE id = ? AND user_id = ?",
            params![workspace_id, user_id],
        ).map_err(AppError::Database)?;

        tx.commit().map_err(AppError::Database)?;

        Ok(())
    }

    /// 获取当前用户 ID
    pub fn get_current_user_id(&self) -> Result<String> {
        let conn = self.pool.get()?;

        conn.query_row(
            "SELECT user_id FROM user_auth WHERE is_current = 1 LIMIT 1",
            [],
            |row| row.get(0),
        ).map_err(|e| {
            if let r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows = e {
                AppError::NotAuthenticated("未登录".to_string())
            } else {
                AppError::Database(e)
            }
        })
    }

    /// 获取最大排序值
    pub fn get_max_sort_order(&self, user_id: &str) -> Result<i32> {
        let conn = self.pool.get()?;

        let max_order: Option<i32> = conn.query_row(
            "SELECT MAX(sort_order) FROM workspaces WHERE user_id = ? AND is_deleted = 0",
            params![user_id],
            |row| row.get(0),
        ).ok();

        Ok(max_order.unwrap_or(0))
    }

    /// 获取当前工作空间 ID（基于当前用户的 is_current 标记）
    pub fn get_current_workspace_id(&self) -> Result<Option<String>> {
        let conn = self.pool.get()?;

        // 获取当前用户 ID
        let user_id = match self.get_current_user_id() {
            Ok(uid) => uid,
            Err(_) => return Ok(None),  // 未登录
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

    /// 查找用户的当前激活工作空间
    pub fn find_current_by_user_id(&self, user_id: &str) -> Result<Option<Workspace>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM workspaces WHERE user_id = ? AND is_current = 1 AND is_deleted = 0 LIMIT 1",
            Self::SELECT_FIELDS
        ))?;

        let workspace = stmt.query_row(params![user_id], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                icon: row.get(4)?,
                color: row.get(5)?,
                is_default: row.get(6)?,
                is_current: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                is_deleted: row.get(11)?,
                deleted_at: row.get(12)?,
                server_ver: row.get(13)?,
                is_dirty: row.get(14)?,
                last_synced_at: row.get(15)?,
            })
        });

        match workspace {
            Ok(w) => Ok(Some(w)),
            Err(r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    /// 设置当前工作空间
    pub fn set_current(&self, user_id: &str, workspace_id: &str) -> Result<()> {
        let conn = self.pool.get()?;

        // 开始事务
        let tx = conn.unchecked_transaction().map_err(AppError::Database)?;

        // 验证工作空间存在且未删除
        let workspace_exists: bool = tx.query_row(
            "SELECT COUNT(*) > 0 FROM workspaces WHERE id = ? AND user_id = ? AND is_deleted = 0",
            params![workspace_id, user_id],
            |row| row.get(0),
        ).map_err(AppError::Database)?;

        if !workspace_exists {
            return Err(AppError::NotFound(format!("工作空间不存在或已删除: {}", workspace_id)));
        }

        // 取消所有当前激活标记
        tx.execute(
            "UPDATE workspaces SET is_current = 0 WHERE user_id = ?",
            params![user_id],
        ).map_err(AppError::Database)?;

        // 设置新的当前工作空间
        tx.execute(
            "UPDATE workspaces SET is_current = 1 WHERE id = ? AND user_id = ?",
            params![workspace_id, user_id],
        ).map_err(AppError::Database)?;

        tx.commit().map_err(AppError::Database)?;

        Ok(())
    }

    /// 迁移孤立数据到指定工作空间
    ///
    /// 将所有 workspace_id = NULL 的数据（未登录时创建的数据）分配到指定的工作空间
    /// 返回详细的迁移统计信息
    pub fn migrate_orphan_data_to_workspace(&self, workspace_id: &str) -> Result<crate::services::workspace_service::MigrateResult> {
        let conn = self.pool.get()?;

        // 开始事务确保所有更新要么全部成功，要么全部失败
        let tx = conn.unchecked_transaction().map_err(AppError::Database)?;

        // 1. 迁移 notes
        let notes = tx.execute(
            "UPDATE notes SET workspace_id = ?1 WHERE workspace_id IS NULL",
            params![workspace_id],
        ).map_err(AppError::Database)? as usize;

        // 2. 迁移 folders
        let folders = tx.execute(
            "UPDATE folders SET workspace_id = ?1 WHERE workspace_id IS NULL",
            params![workspace_id],
        ).map_err(AppError::Database)? as usize;

        // 3. 迁移 tags
        let tags = tx.execute(
            "UPDATE tags SET workspace_id = ?1 WHERE workspace_id IS NULL",
            params![workspace_id],
        ).map_err(AppError::Database)? as usize;

        // 4. 迁移 note_snapshots
        let snapshots = tx.execute(
            "UPDATE note_snapshots SET workspace_id = ?1 WHERE workspace_id IS NULL",
            params![workspace_id],
        ).map_err(AppError::Database)? as usize;

        // 5. 迁移 note_tags 关联表
        let _note_tags = tx.execute(
            "UPDATE note_tags SET workspace_id = ?1 WHERE workspace_id IS NULL",
            params![workspace_id],
        ).map_err(AppError::Database)? as usize;

        tx.commit().map_err(AppError::Database)?;

        log::info!(
            "[WorkspaceRepository] 迁移完成: notes={}, folders={}, tags={}, snapshots={}",
            notes, folders, tags, snapshots
        );

        Ok(crate::services::workspace_service::MigrateResult {
            notes,
            folders,
            tags,
            snapshots,
        })
    }
}

