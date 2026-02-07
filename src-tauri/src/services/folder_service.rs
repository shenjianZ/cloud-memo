use crate::database::repositories::FolderRepository;
use crate::models::{Folder, CreateFolderRequest, UpdateFolderRequest, MoveFolderRequest};
use crate::models::error::{Result, AppError};

/// 文件夹业务逻辑层
///
/// 处理文件夹相关的业务逻辑，调用 Repository 进行数据操作
pub struct FolderService {
    repo: FolderRepository,
}

impl FolderService {
    /// 创建新的 FolderService 实例
    pub fn new(repo: FolderRepository) -> Self {
        Self { repo }
    }

    /// 创建文件夹
    pub fn create_folder(&self, req: CreateFolderRequest) -> Result<Folder> {
        // 获取最大排序值
        let sort_order = self.repo.get_max_sort_order(req.parent_id.as_deref())? + 1;

        // 使用构造函数创建文件夹
        let mut folder = Folder::new(req.name, req.parent_id, req.color, req.icon);

        // 设置计算得到的 sort_order
        folder.sort_order = sort_order;

        self.repo.create(&folder)
    }

    /// 根据 ID 获取文件夹
    pub fn get_folder(&self, id: &str) -> Result<Folder> {
        self.repo.find_by_id(id)?
            .ok_or(AppError::NotFound(format!("文件夹 {} 未找到", id)))
    }

    /// 更新文件夹
    pub fn update_folder(&self, req: UpdateFolderRequest) -> Result<Folder> {
        let mut folder = self.get_folder(&req.id)?;

        // 如果要修改父文件夹，需要检查循环引用
        if let Some(new_parent_id) = &req.parent_id {
            // 如果新父文件夹是自己，不允许
            if new_parent_id == &folder.id {
                return Err(AppError::InvalidOperation("无法将文件夹设置为自己的父文件夹".to_string()));
            }

            // 检查循环引用
            if self.repo.check_circular_reference(&folder.id, new_parent_id)? {
                return Err(AppError::InvalidOperation("移动会创建循环引用".to_string()));
            }

            folder.parent_id = Some(new_parent_id.clone());
        }

        if let Some(name) = req.name {
            folder.name = name;
        }
        if let Some(color) = req.color {
            folder.color = Some(color);
        }
        if let Some(icon) = req.icon {
            folder.icon = Some(icon);
        }
        if let Some(sort_order) = req.sort_order {
            folder.sort_order = sort_order;
        }

        folder.updated_at = chrono::Utc::now().timestamp();
        // 云端同步：修改文件夹时标记为需要同步
        folder.is_dirty = true;

        self.repo.update(&folder)
    }

    /// 删除文件夹（物理删除，级联删除子文件夹）
    ///
    /// ## 删除行为
    ///
    /// ### 对文件夹的影响
    /// - 删除目标文件夹本身
    /// - **级联删除所有子文件夹**（数据库外键自动处理）
    /// - 整棵子树全部删除
    ///
    /// ### 对笔记的影响
    /// - **笔记不会被删除**（用户内容保护）
    /// - 所有笔记的 `folder_id` 被设为 `NULL`
    /// - 笔记变成"未分类"状态，仍在应用中
    ///
    /// ## 示例场景
    ///
    /// ```text
    /// 删除"工作"文件夹：
    ///
    /// 删除前：                           删除后：
    /// 📁 工作                            （文件夹树中消失）
    ///   ├─ 📄 项目A笔记                  📄 项目A笔记（未分类）
    ///   ├─ 📄 项目B笔记                  📄 项目B笔记（未分类）
    ///   └─ 📁 2024                       （被级联删除）
    ///       └─ 📄 年度计划                📄 年度计划（未分类）
    /// ```
    ///
    /// ## 设计理念
    ///
    /// - ✅ **文件夹 = 组织结构**：可删除，支持动态调整
    /// - ✅ **笔记 = 用户内容**：永不因文件夹删除而丢失
    /// - ✅ **回收站**：笔记有独立的软删除机制
    ///
    /// ## 注意事项
    ///
    /// - ⚠️ **不可恢复**：文件夹和子文件夹删除后无法恢复
    /// - ✅ **笔记安全**：笔记仍然存在，只是不再属于任何文件夹
    /// - 💡 **建议**：删除前应提示用户"此操作将删除文件夹及子文件夹，笔记会保留在未分类"
    pub fn delete_folder(&self, id: &str) -> Result<()> {
        // 验证文件夹存在
        self.get_folder(id)?;

        // 物理删除：数据库外键自动级联删除子文件夹，笔记 folder_id 设为 NULL
        self.repo.delete(id)
    }

    /// 获取所有文件夹
    pub fn list_folders(&self) -> Result<Vec<Folder>> {
        self.repo.find_all()
    }

    /// 获取文件夹树
    pub fn get_folder_tree(&self) -> Result<Vec<Folder>> {
        self.repo.find_all()
        // 前端负责构建树形结构
    }

    /// 移动文件夹
    pub fn move_folder(&self, req: MoveFolderRequest) -> Result<Folder> {
        let update_req = UpdateFolderRequest {
            id: req.id,
            parent_id: req.new_parent_id,
            name: None,
            color: None,
            icon: None,
            sort_order: req.new_sort_order,
        };

        self.update_folder(update_req)
    }

    /// 获取文件夹路径
    pub fn get_folder_path(&self, id: &str) -> Result<Vec<Folder>> {
        self.repo.get_path(id)
    }

    /// 获取指定文件夹下的所有笔记（通过 NoteRepository）
    /// 注意：这个方法实际在 NoteService 中实现，这里仅作为接口定义
    pub fn get_folder_notes(&self, _folder_id: &str) -> Result<Vec<crate::models::Note>> {
        // 这个方法需要 NoteRepository，实际实现在组合服务中
        Err(AppError::NotFound("Use NoteService to get folder notes".to_string()))
    }
}
