use crate::database::repositories::FolderRepository;
use crate::models::{Folder, CreateFolderRequest, UpdateFolderRequest, MoveFolderRequest};
use crate::models::error::{Result, AppError};
use uuid::Uuid;

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
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        // 获取最大排序值
        let sort_order = self.repo.get_max_sort_order(req.parent_id.as_deref())? + 1;

        let folder = Folder {
            id,
            name: req.name,
            parent_id: req.parent_id,
            icon: req.icon,
            color: req.color,
            sort_order,
            is_deleted: false,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        self.repo.create(&folder)
    }

    /// 根据 ID 获取文件夹
    pub fn get_folder(&self, id: &str) -> Result<Folder> {
        self.repo.find_by_id(id)?
            .ok_or(AppError::NotFound(format!("Folder {} not found", id)))
    }

    /// 更新文件夹
    pub fn update_folder(&self, req: UpdateFolderRequest) -> Result<Folder> {
        let mut folder = self.get_folder(&req.id)?;

        // 如果要修改父文件夹，需要检查循环引用
        if let Some(new_parent_id) = &req.parent_id {
            // 如果新父文件夹是自己，不允许
            if new_parent_id == &folder.id {
                return Err(AppError::InvalidOperation("Cannot set folder as its own parent".to_string()));
            }

            // 检查循环引用
            if self.repo.check_circular_reference(&folder.id, new_parent_id)? {
                return Err(AppError::InvalidOperation("Moving would create a circular reference".to_string()));
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

        self.repo.update(&folder)
    }

    /// 删除文件夹（软删除）
    pub fn delete_folder(&self, id: &str) -> Result<()> {
        // 验证文件夹存在
        self.get_folder(id)?;

        // TODO: 这里可以选择：
        // 1. 级联删除子文件夹和笔记
        // 2. 仅删除文件夹，子文件夹和笔记保留
        // 当前实现：仅软删除文件夹本身
        self.repo.soft_delete(id)
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
