use crate::database::repositories::WorkspaceRepository;
use crate::models::{Workspace, CreateWorkspaceRequest, UpdateWorkspaceRequest};
use crate::models::error::{Result, AppError};
use serde::Serialize;
use r2d2_sqlite::rusqlite::params;

/// 迁移结果统计
#[derive(Debug, Clone, Serialize)]
pub struct MigrateResult {
    pub notes: usize,
    pub folders: usize,
    pub tags: usize,
    pub snapshots: usize,
}



/// 工作空间业务逻辑层
///
/// 处理工作空间相关的业务逻辑，调用 Repository 进行数据操作
#[derive(Clone)]
pub struct WorkspaceService {
    repo: WorkspaceRepository,
}

impl WorkspaceService {
    /// 创建新的 WorkspaceService 实例
    pub fn new(repo: WorkspaceRepository) -> Self {
        Self { repo }
    }

    /// 获取当前用户 ID
    fn get_current_user_id(&self) -> Result<String> {
        self.repo.get_current_user_id()
    }

    /// 创建工作空间
    pub fn create_workspace(&self, req: CreateWorkspaceRequest) -> Result<Workspace> {
        // 获取当前用户 ID
        let user_id = self.get_current_user_id()?;

        // 获取最大排序值
        let sort_order = self.repo.get_max_sort_order(&user_id)? + 1;

        // 创建工作空间
        let mut workspace = Workspace::new(user_id, req.name, req.description, req.icon, req.color);
        workspace.sort_order = sort_order;

        self.repo.create(&workspace)
    }

    /// 根据 ID 获取工作空间
    pub fn get_workspace(&self, id: &str) -> Result<Workspace> {
        self.repo.find_by_id(id)?
            .ok_or(AppError::NotFound(format!("工作空间 {} 未找到", id)))
    }

    /// 获取所有工作空间
    pub fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        let user_id = self.get_current_user_id()?;
        self.repo.find_by_user_id(&user_id)
    }

    /// 获取当前工作空间
    pub fn get_current_workspace(&self) -> Result<Workspace> {
        let user_id = self.get_current_user_id()?;

        // 先尝试查找当前激活的工作空间
        if let Some(workspace) = self.repo.find_current_by_user_id(&user_id)? {
            return Ok(workspace);
        }

        // 如果没有设置，则使用默认空间
        if let Some(workspace) = self.repo.find_default_by_user_id(&user_id)? {
            // 自动设置为当前工作空间
            let _ = self.repo.set_current(&user_id, &workspace.id);
            Ok(workspace)
        } else {
            Err(AppError::NotFound("未找到工作空间".to_string()))
        }
    }

    /// 获取默认工作空间
    pub fn get_default_workspace(&self, user_id: &str) -> Result<Workspace> {
        self.repo.find_default_by_user_id(user_id)?
            .ok_or_else(|| AppError::NotFound("未找到默认工作空间".to_string()))
    }

    /// 更新工作空间
    pub fn update_workspace(&self, req: UpdateWorkspaceRequest) -> Result<Workspace> {
        let mut workspace = self.get_workspace(&req.id)?;

        // 只允许更新以下字段
        if let Some(name) = req.name {
            workspace.name = name;
        }
        if let Some(description) = req.description {
            workspace.description = Some(description);
        }
        if let Some(icon) = req.icon {
            workspace.icon = Some(icon);
        }
        if let Some(color) = req.color {
            workspace.color = Some(color);
        }

        workspace.updated_at = chrono::Utc::now().timestamp();
        // 云端同步：修改工作空间时标记为需要同步
        workspace.is_dirty = true;

        self.repo.update(&workspace)
    }

    /// 删除工作空间（软删除）
    pub fn delete_workspace(&self, id: &str) -> Result<()> {
        // 检查是否存在
        let workspace = self.get_workspace(id)?;

        // 不允许删除默认空间
        if workspace.is_default {
            return Err(AppError::InvalidOperation("不允许删除默认工作空间".to_string()));
        }

        self.repo.delete(id)
    }

    /// 设置默认工作空间（已禁用）
    ///
    /// 默认工作空间只在用户注册时创建，不允许修改
    pub fn set_default_workspace(&self, id: &str) -> Result<()> {
        Err(AppError::InvalidOperation("默认工作空间不能修改，只能在注册时创建".to_string()))
    }

    /// 切换工作空间
    pub fn switch_workspace(&self, id: &str) -> Result<()> {
        // 验证工作空间是否存在
        self.get_workspace(id)?;

        // 设置为当前工作空间
        let user_id = self.get_current_user_id()?;
        self.repo.set_current(&user_id, id)
    }

    /// 迁移孤立数据到当前工作空间
    ///
    /// 将所有 workspace_id = NULL 的数据（未登录时创建的数据）分配到指定的工作空间
    pub fn migrate_orphan_data_to_workspace(&self, workspace_id: &str) -> Result<MigrateResult> {
        log::info!("[WorkspaceService] 开始迁移孤立数据到工作空间: workspace_id={}", workspace_id);

        // 调用 Repository 层的迁移方法，直接返回详细的统计信息
        self.repo.migrate_orphan_data_to_workspace(workspace_id)
    }
}
