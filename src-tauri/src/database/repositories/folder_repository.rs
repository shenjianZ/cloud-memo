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
    /// 创建新的 FolderRepository 实例
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 根据 ID 查找文件夹
    pub fn find_by_id(&self, id: &str) -> Result<Option<Folder>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, icon, color, sort_order, is_deleted, created_at, updated_at, deleted_at,
                    server_ver, is_dirty, last_synced_at
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
            "SELECT id, name, parent_id, icon, color, sort_order, is_deleted, created_at, updated_at, deleted_at,
                    server_ver, is_dirty, last_synced_at
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
                server_ver: row.get(10)?,
                is_dirty: row.get(11)?,
                last_synced_at: row.get(12)?,
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

    /// 软删除文件夹（已废弃，请使用 hard_delete）
    ///
    /// ⚠️ 警告：软删除文件夹会导致树结构逻辑断裂
    /// - 子文件夹的 parent_id 指向已删除的父节点
    /// - 查询时需要额外过滤 is_deleted
    /// - 恢复时子文件夹状态不一致
    #[deprecated(note = "请使用 hard_delete 代替")]
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

    /// 物理删除文件夹（推荐使用）
    ///
    /// ## 删除行为
    ///
    /// ### 对文件夹的影响
    /// - 删除目标文件夹本身
    /// - **级联删除所有子文件夹**（通过 ON DELETE CASCADE）
    /// - 子文件夹的子文件夹……整棵子树全部删除
    ///
    /// ### 对笔记的影响
    /// - **软删除该文件夹及所有子文件夹下的笔记**（is_deleted = 1）
    /// - 这些笔记会进入回收站，可以被恢复
    ///
    /// ## 示例
    ///
    /// ```text
    /// 删除前：                 删除后：
    /// 📁 工作文件夹             📄 工作笔记1（回收站）
    ///   ├─ 📄 工作笔记1         📄 工作笔记2（回收站）
    ///   ├─ 📄 工作笔记2
    ///   └─ 📁 2024项目         ❌ 整个子树被删除
    ///       └─ 📄 项目笔记     📄 项目笔记（回收站）
    /// ```
    ///
    /// ## 为什么推荐物理删除？
    ///
    /// 1. **树结构完整**：外键约束自动维护，不会出现断裂
    /// 2. **代码简单**：不需要递归逻辑，数据库自动处理
    /// 3. **同步友好**：删除事件清晰，不会产生状态冲突
    /// 4. **性能更好**：一次 DELETE，数据库自动级联
    ///
    /// ## 注意事项
    ///
    /// - ⚠️ **不可恢复文件夹**：物理删除无法恢复，删除前应提示用户
    /// - ✅ **笔记可恢复**：笔记进入回收站，可以恢复
    /// - ✅ **递归软删除**：自动软删除所有子文件夹下的笔记
    pub fn hard_delete(&self, id: &str) -> Result<()> {
        let conn = self.pool.get()?;

        // 第一步：获取所有子孙文件夹的 ID（包括自己）
        let folder_ids = self.get_all_descendant_ids(id)?;

        // 第二步：软删除这些文件夹下的所有笔记
        let now = chrono::Utc::now().timestamp();
        if !folder_ids.is_empty() {
            let placeholders = folder_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "UPDATE notes SET is_deleted = 1, deleted_at = ? WHERE folder_id IN ({})",
                placeholders
            );

            let mut params_list: Vec<&dyn r2d2_sqlite::rusqlite::ToSql> = vec![&now];
            for folder_id in &folder_ids {
                params_list.push(folder_id);
            }

            conn.execute(&sql, params_list.as_slice())?;
            log::debug!("Soft deleted notes in folders: {:?}", folder_ids);
        }

        // 第三步：物理删除文件夹（级联删除子文件夹）
        conn.execute("DELETE FROM folders WHERE id = ?", params![id])?;

        log::debug!("Folder hard deleted: {} (cascade to children, notes moved to trash)", id);
        Ok(())
    }

    /// 获取文件夹的所有子孙文件夹 ID（包括自己）
    ///
    /// ## 实现原理
    /// 1. 递归查询数据库（利用 parent_id 外键）
    /// 2. 收集所有子孙节点的 ID
    fn get_all_descendant_ids(&self, id: &str) -> Result<Vec<String>> {
        let mut ids = Vec::new();
        self.collect_descendant_ids_recursive(id, &mut ids)?;
        Ok(ids)
    }

    /// 递归收集子孙文件夹 ID
    fn collect_descendant_ids_recursive(&self, parent_id: &str, ids: &mut Vec<String>) -> Result<()> {
        // 添加自己
        ids.push(parent_id.to_string());

        // 查找直接子文件夹
        let children = self.find_children(Some(parent_id))?;

        // 递归处理每个子文件夹
        for child in children {
            self.collect_descendant_ids_recursive(&child.id, ids)?;
        }

        Ok(())
    }

    /// 查找子文件夹
    pub fn find_children(&self, parent_id: Option<&str>) -> Result<Vec<Folder>> {
        let conn = self.pool.get()?;

        if let Some(pid) = parent_id {
            let mut stmt = conn.prepare(
                "SELECT id, name, parent_id, icon, color, sort_order, is_deleted, created_at, updated_at, deleted_at,
                        server_ver, is_dirty, last_synced_at
                 FROM folders
                 WHERE parent_id = ? AND is_deleted = 0
                 ORDER BY sort_order ASC, created_at ASC"
            )?;
            let folders = stmt.query_map(params![pid], |row| self.row_to_folder(row))?;
            folders.collect::<std::result::Result<Vec<_>, _>>().map_err(AppError::Database)
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, name, parent_id, icon, color, sort_order, is_deleted, created_at, updated_at, deleted_at,
                        server_ver, is_dirty, last_synced_at
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
            server_ver: row.get(10)?,
            is_dirty: row.get(11)?,
            last_synced_at: row.get(12)?,
        })
    }
}
