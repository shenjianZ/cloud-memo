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
pub struct FolderRepository {
    pool: DbPool,
}

impl FolderRepository {
    /// 创建新的 FolderRepository 实例
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 根据 ID 查找文件夹
    pub fn find_by_id(&self, id: &str) -> Result<Option<Folder>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, icon, color, sort_order, is_deleted, created_at, updated_at, deleted_at
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
                is_deleted: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                deleted_at: row.get(9)?,
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
            "SELECT id, name, parent_id, icon, color, sort_order, is_deleted, created_at, updated_at, deleted_at
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
                is_deleted: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                deleted_at: row.get(9)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()
          .map_err(AppError::Database)?;

        Ok(folders)
    }

    /// 创建新文件夹
    pub fn create(&self, folder: &Folder) -> Result<Folder> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO folders (id, name, parent_id, icon, color, sort_order, is_deleted, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                folder.id, folder.name, folder.parent_id, folder.icon, folder.color,
                folder.sort_order, folder.is_deleted as i32, folder.created_at, folder.updated_at
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
             SET name = ?, parent_id = ?, icon = ?, color = ?, sort_order = ?, updated_at = ?
             WHERE id = ?",
            params![
                folder.name, folder.parent_id, folder.icon, folder.color,
                folder.sort_order, folder.updated_at, folder.id
            ],
        )?;

        log::debug!("Folder updated: {}", folder.id);
        Ok(folder.clone())
    }

    /// 软删除文件夹
    pub fn soft_delete(&self, id: &str) -> Result<()> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "UPDATE folders SET is_deleted = 1, deleted_at = ? WHERE id = ?",
            params![now, id],
        )?;

        log::debug!("Folder soft deleted: {}", id);
        Ok(())
    }

    /// 查找子文件夹
    pub fn find_children(&self, parent_id: Option<&str>) -> Result<Vec<Folder>> {
        let conn = self.pool.get()?;

        if let Some(pid) = parent_id {
            let mut stmt = conn.prepare(
                "SELECT id, name, parent_id, icon, color, sort_order, is_deleted, created_at, updated_at, deleted_at
                 FROM folders
                 WHERE parent_id = ? AND is_deleted = 0
                 ORDER BY sort_order ASC, created_at ASC"
            )?;
            let folders = stmt.query_map(params![pid], |row| self.row_to_folder(row))?;
            folders.collect::<std::result::Result<Vec<_>, _>>().map_err(AppError::Database)
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, name, parent_id, icon, color, sort_order, is_deleted, created_at, updated_at, deleted_at
                 FROM folders
                 WHERE parent_id IS NULL AND is_deleted = 0
                 ORDER BY sort_order ASC, created_at ASC"
            )?;
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
                "SELECT MAX(sort_order) FROM folders WHERE parent_id = ? AND is_deleted = 0"
            )?;
            stmt.query_row(params![pid], |row| row.get(0))?
        } else {
            let mut stmt = conn.prepare(
                "SELECT MAX(sort_order) FROM folders WHERE parent_id IS NULL AND is_deleted = 0"
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
    fn row_to_folder(&self, row: &r2d2_sqlite::rusqlite::Row) -> std::result::Result<Folder, r2d2_sqlite::rusqlite::Error> {
        Ok(Folder {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            icon: row.get(3)?,
            color: row.get(4)?,
            sort_order: row.get(5)?,
            is_deleted: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            deleted_at: row.get(9)?,
        })
    }
}
