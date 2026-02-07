use crate::models::Folder;
use crate::database::DbPool;
use crate::models::error::{Result, AppError};
use r2d2_sqlite::rusqlite::params;

/// 文件夹树节点（用于构建树形结构）
#[derive(Debug, Clone)]
pub struct FolderNode {
    pub folder: Folder,
    pub children: Vec<FolderNode>,
}

/// 文件夹数据访问层
///
/// 负责所有与文件夹相关的数据库操作
#[derive(Clone)]
pub struct FolderRepository {
    pool: DbPool,
}

impl FolderRepository {
    /// 统一的 SQL 查询字段列表
    /// 字段顺序必须与 Folder 结构体初始化顺序一致
    const SELECT_FIELDS: &'static str =
        "id, name, parent_id, icon, color, sort_order, created_at, updated_at,
         is_deleted, deleted_at, server_ver, is_dirty, last_synced_at";

    /// 创建新的 FolderRepository 实例
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 根据 ID 查找文件夹
    pub fn find_by_id(&self, id: &str) -> Result<Option<Folder>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, icon, color, sort_order, created_at, updated_at,
                    is_deleted, deleted_at, server_ver, is_dirty, last_synced_at
             FROM folders
             WHERE id = ? AND is_deleted = 0"
        )?;

        let folder = stmt.query_row(params![id], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                sort_order: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                is_deleted: row.get(8)?,
                deleted_at: row.get(9)?,
                server_ver: row.get(10)?,
                is_dirty: row.get(11)?,
                last_synced_at: row.get(12)?,
            })
        });

        match folder {
            Ok(f) => Ok(Some(f)),
            Err(r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    /// 查找所有文件夹
    pub fn find_all(&self) -> Result<Vec<Folder>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, icon, color, sort_order, created_at, updated_at,
                    is_deleted, deleted_at, server_ver, is_dirty, last_synced_at
             FROM folders
             WHERE is_deleted = 0
             ORDER BY sort_order ASC, created_at ASC"
        )?;

        let folders = stmt.query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                sort_order: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                is_deleted: row.get(8)?,
                deleted_at: row.get(9)?,
                server_ver: row.get(10)?,
                is_dirty: row.get(11)?,
                last_synced_at: row.get(12)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()
          .map_err(AppError::Database)?;

        Ok(folders)
    }

    /// 根据名称查找文件夹（包括已删除的）
    pub fn find_by_name_include_deleted(&self, name: &str) -> Result<Option<Folder>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, icon, color, sort_order, created_at, updated_at,
                    is_deleted, deleted_at, server_ver, is_dirty, last_synced_at
             FROM folders
             WHERE name = ?
             LIMIT 1"
        )?;

        let folder = stmt.query_row(params![name], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                sort_order: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                is_deleted: row.get(8)?,
                deleted_at: row.get(9)?,
                server_ver: row.get(10)?,
                is_dirty: row.get(11)?,
                last_synced_at: row.get(12)?,
            })
        });

        match folder {
            Ok(f) => Ok(Some(f)),
            Err(r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    /// 恢复已删除的文件夹
    pub fn restore(&self, id: &str) -> Result<Folder> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp();

        // 先获取文件夹信息
        let folder = self.find_by_id_include_deleted(id)?;

        // 更新为未删除状态
        conn.execute(
            "UPDATE folders SET is_deleted = 0, deleted_at = NULL, updated_at = ?, is_dirty = 1 WHERE id = ?",
            params![now, id],
        )?;

        let mut restored_folder = folder.unwrap();
        restored_folder.is_deleted = false;
        restored_folder.deleted_at = None;
        restored_folder.updated_at = now;
        restored_folder.is_dirty = true;

        log::info!("Folder restored: {}", id);
        Ok(restored_folder)
    }

    /// 根据 ID 查找文件夹（包括已删除的）
    fn find_by_id_include_deleted(&self, id: &str) -> Result<Option<Folder>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, icon, color, sort_order, created_at, updated_at,
                    is_deleted, deleted_at, server_ver, is_dirty, last_synced_at
             FROM folders
             WHERE id = ?"
        )?;

        let folder = stmt.query_row(params![id], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                sort_order: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                is_deleted: row.get(8)?,
                deleted_at: row.get(9)?,
                server_ver: row.get(10)?,
                is_dirty: row.get(11)?,
                last_synced_at: row.get(12)?,
            })
        });

        match folder {
            Ok(f) => Ok(Some(f)),
            Err(r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    /// 创建新文件夹
    pub fn create(&self, folder: &Folder) -> Result<Folder> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO folders (id, name, parent_id, icon, color, sort_order, created_at, updated_at,
                                is_deleted, deleted_at, server_ver, is_dirty, last_synced_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                folder.id, folder.name, folder.parent_id, folder.icon, folder.color,
                folder.sort_order, folder.created_at, folder.updated_at,
                folder.is_deleted as i32, folder.deleted_at,
                folder.server_ver, folder.is_dirty as i32, folder.last_synced_at
            ],
        )?;

        log::debug!("Folder created: {}", folder.id);
        Ok(folder.clone())
    }

    /// 更新文件夹
    pub fn update(&self, folder: &Folder) -> Result<Folder> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE folders
             SET name = ?, parent_id = ?, icon = ?, color = ?, sort_order = ?, updated_at = ?,
                 is_dirty = ?
             WHERE id = ? AND is_deleted = 0",
            params![
                folder.name, folder.parent_id, folder.icon, folder.color,
                folder.sort_order, folder.updated_at, folder.is_dirty as i32, folder.id
            ],
        )?;

        log::debug!("Folder updated: {}", folder.id);
        Ok(folder.clone())
    }

    /// 删除文件夹（软删除）
    ///
    /// ## 删除行为
    ///
    /// - **文件夹**：标记为已删除（软删除）
    /// - **子文件夹**：递归标记所有子文件夹为已删除
    /// - **笔记**：笔记不会被删除，folder_id 保持不变
    ///
    /// ## 示例
    ///
    /// ```text
    /// 删除前：                 删除后：
    /// 📁 工作文件夹             📁 工作文件夹（is_deleted=1）
    ///   ├─ 📄 工作笔记1         📄 工作笔记1（folder_id 不变）
    ///   ├─ 📄 工作笔记2         📄 工作笔记2（folder_id 不变）
    ///   └─ 📁 2024项目         📁 2024项目（is_deleted=1）
    ///       └─ 📄 项目笔记     📄 项目笔记（folder_id 不变）
    /// ```
    ///
    /// ## 注意事项
    ///
    /// - ✅ **可恢复**：文件夹和子文件夹可以恢复
    /// - ✅ **笔记级联删除**：文件夹及其子文件夹下的所有笔记也会被软删除
    /// - ⚠️ **同步标记**：删除操作会被标记为需要同步
    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp();

        // 1. 软删除文件夹及所有子文件夹（使用递归CTE）
        let affected_folders = conn.execute(
            "WITH RECURSIVE folder_tree AS (
                -- 起始文件夹
                SELECT id FROM folders WHERE id = ?1
                UNION ALL
                -- 子文件夹
                SELECT f.id FROM folders f
                INNER JOIN folder_tree ft ON f.parent_id = ft.id
                WHERE f.is_deleted = 0
            )
            UPDATE folders SET is_deleted = 1, deleted_at = ?2, is_dirty = 1
            WHERE id IN folder_tree",
            params![id, now],
        )?;

        // 2. 软删除这些文件夹下的所有笔记（级联删除）
        // 注意：不能使用 is_deleted = 0 过滤，因为第 1 步已经将文件夹标记为删除
        let affected_notes = conn.execute(
            "WITH RECURSIVE folder_tree AS (
                -- 起始文件夹
                SELECT id FROM folders WHERE id = ?1
                UNION ALL
                -- 递归查找所有子文件夹（不管是否已标记删除）
                SELECT f.id FROM folders f
                INNER JOIN folder_tree ft ON f.parent_id = ft.id
            )
            UPDATE notes SET is_deleted = 1, deleted_at = ?2, is_dirty = 1
            WHERE folder_id IN folder_tree AND is_deleted = 0",
            params![id, now],
        )?;

        log::info!(
            "Folder soft deleted: id={}, folders_affected={}, notes_affected={}",
            id,
            affected_folders,
            affected_notes
        );
        Ok(())
    }

    /// 查找子文件夹
    pub fn find_children(&self, parent_id: Option<&str>) -> Result<Vec<Folder>> {
        let conn = self.pool.get()?;

        if let Some(pid) = parent_id {
            let mut stmt = conn.prepare(&format!(
                "SELECT {} FROM folders
                 WHERE parent_id = ? AND is_deleted = 0
                 ORDER BY sort_order ASC, created_at ASC",
                Self::SELECT_FIELDS
            ))?;
            let folders = stmt.query_map(params![pid], |row| self.row_to_folder(row))?;
            folders.collect::<std::result::Result<Vec<_>, _>>().map_err(AppError::Database)
        } else {
            let mut stmt = conn.prepare(&format!(
                "SELECT {} FROM folders
                 WHERE parent_id IS NULL AND is_deleted = 0
                 ORDER BY sort_order ASC, created_at ASC",
                Self::SELECT_FIELDS
            ))?;
            let folders = stmt.query_map([], |row| self.row_to_folder(row))?;
            folders.collect::<std::result::Result<Vec<_>, _>>().map_err(AppError::Database)
        }
    }

    /// 构建文件夹树
    pub fn find_tree(&self) -> Result<Vec<FolderNode>> {
        let all_folders = self.find_all()?;
        let mut folder_map: std::collections::HashMap<String, FolderNode> = std::collections::HashMap::new();

        // 初始化所有节点
        for folder in all_folders {
            folder_map.insert(folder.id.clone(), FolderNode {
                folder,
                children: Vec::new(),
            });
        }

        // 收集所有父节点关系
        let mut child_parent_relations: Vec<(String, Option<String>)> = Vec::new();
        for (id, node) in folder_map.iter() {
            child_parent_relations.push((id.clone(), node.folder.parent_id.clone()));
        }

        // 构建树形结构
        let mut roots = Vec::new();
        for (child_id, parent_id) in child_parent_relations {
            let child_node = folder_map.get(&child_id).cloned().unwrap();
            if let Some(pid) = parent_id {
                if let Some(parent_node) = folder_map.get_mut(&pid) {
                    parent_node.children.push(child_node);
                }
            } else {
                roots.push(child_node);
            }
        }

        Ok(roots)
    }

    /// 获取文件夹路径（从根到当前文件夹）
    pub fn get_path(&self, id: &str) -> Result<Vec<Folder>> {
        let mut path = Vec::new();
        let mut current_id = Some(id.to_string());

        while let Some(folder_id) = current_id {
            if let Some(folder) = self.find_by_id(&folder_id)? {
                current_id = folder.parent_id.clone();
                path.insert(0, folder);
            } else {
                break;
            }
        }

        Ok(path)
    }

    /// 获取指定父文件夹下的最大排序值
    pub fn get_max_sort_order(&self, parent_id: Option<&str>) -> Result<i32> {
        let conn = self.pool.get()?;

        let max_order: Option<i32> = if let Some(pid) = parent_id {
            let mut stmt = conn.prepare(
                "SELECT MAX(sort_order) FROM folders WHERE parent_id = ?"
            )?;
            stmt.query_row(params![pid], |row| row.get(0))?
        } else {
            let mut stmt = conn.prepare(
                "SELECT MAX(sort_order) FROM folders WHERE parent_id IS NULL"
            )?;
            stmt.query_row([], |row| row.get(0))?
        };

        Ok(max_order.unwrap_or(0))
    }

    /// 检查循环引用
    pub fn check_circular_reference(&self, folder_id: &str, new_parent_id: &str) -> Result<bool> {
        let mut current_id = Some(new_parent_id.to_string());
        let mut visited = std::collections::HashSet::new();

        while let Some(check_id) = current_id {
            // 如果回到自己，存在循环引用
            if check_id == folder_id {
                return Ok(true);
            }

            // 防止无限循环
            if visited.contains(&check_id) {
                return Ok(true);
            }
            visited.insert(check_id.clone());

            if let Some(folder) = self.find_by_id(&check_id)? {
                current_id = folder.parent_id.clone();
            } else {
                break;
            }
        }

        Ok(false)
    }

    /// 辅助方法：从行数据转换为 Folder
    /// 字段顺序必须与 SELECT_FIELDS 一致
    fn row_to_folder(&self, row: &r2d2_sqlite::rusqlite::Row) -> std::result::Result<Folder, r2d2_sqlite::rusqlite::Error> {
        Ok(Folder {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            icon: row.get(3)?,
            color: row.get(4)?,
            sort_order: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
            is_deleted: row.get(8)?,
            deleted_at: row.get(9)?,
            server_ver: row.get(10)?,
            is_dirty: row.get(11)?,
            last_synced_at: row.get(12)?,
        })
    }

    /// 硬删除文件夹（永久删除，包括子文件夹和所有笔记）
    ///
    /// ## 删除行为
    ///
    /// - 递归删除文件夹及其所有子文件夹
    /// - 删除这些文件夹下的所有笔记（包括软删除的笔记）
    /// - 外键约束会自动处理 `note_tags` 等关联数据
    ///
    /// ## 安全性
    ///
    /// - ⚠️ 此操作不可逆，会删除整个文件夹树
    /// - ⚠️ 包括软删除的笔记也会被永久删除
    pub fn hard_delete(&self, id: &str) -> Result<()> {
        let conn = self.pool.get()?;

        // 使用递归 CTE 查找所有子文件夹，然后删除
        let affected = conn.execute(
            "WITH RECURSIVE folder_tree AS (
                -- 起始文件夹
                SELECT id FROM folders WHERE id = ?1
                UNION ALL
                -- 子文件夹
                SELECT f.id FROM folders f
                INNER JOIN folder_tree ft ON f.parent_id = ft.id
            )
            -- 1. 删除文件夹树下的所有笔记（包括软删除的）
            DELETE FROM notes WHERE folder_id IN folder_tree;

            -- 2. 删除文件夹树
            WITH RECURSIVE folder_tree AS (
                SELECT id FROM folders WHERE id = ?1
                UNION ALL
                SELECT f.id FROM folders f
                INNER JOIN folder_tree ft ON f.parent_id = ft.id
            )
            DELETE FROM folders WHERE id IN folder_tree",
            params![id, id],
        )?;

        log::info!("[FolderRepository] 硬删除文件夹: id={}, affected={}", id, affected);
        Ok(())
    }

    /// 清理超过指定天数的软删除文件夹
    ///
    /// ## 参数
    ///
    /// - `days`: 软删除后的保留天数（如 30 天）
    ///
    /// ## 返回
    ///
    /// 返回清理的文件夹数量
    pub fn purge_old_deleted_folders(&self, days: i64) -> Result<i64> {
        let conn = self.pool.get()?;
        let cutoff_time = chrono::Utc::now().timestamp() - (days * 86400);

        // 先删除这些文件夹下的所有笔记
        let notes_affected = conn.execute(
            "WITH RECURSIVE folder_tree AS (
                SELECT id FROM folders WHERE is_deleted = 1 AND deleted_at < ?
                UNION ALL
                SELECT f.id FROM folders f
                INNER JOIN folder_tree ft ON f.parent_id = ft.id
            )
            DELETE FROM notes WHERE folder_id IN folder_tree",
            params![cutoff_time],
        ).map_err(AppError::Database)?;

        // 再删除文件夹
        let folders_affected = conn.execute(
            "WITH RECURSIVE folder_tree AS (
                SELECT id FROM folders WHERE is_deleted = 1 AND deleted_at < ?
                UNION ALL
                SELECT f.id FROM folders f
                INNER JOIN folder_tree ft ON f.parent_id = ft.id
            )
            DELETE FROM folders WHERE id IN folder_tree",
            params![cutoff_time],
        ).map_err(AppError::Database)?;

        log::info!("[FolderRepository] 清理旧文件夹: days={}, folders={}, notes={}", days, folders_affected, notes_affected);
        Ok(folders_affected as i64)
    }
}
