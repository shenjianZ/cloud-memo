use crate::models::{Workspace, CreateWorkspaceRequest, UpdateWorkspaceRequest};
use crate::services::{WorkspaceService, AutoSyncService, workspace_service::MigrateResult};
use tauri::State;

/// Workspace service 类型别名
type WorkspaceSvc<'a> = State<'a, WorkspaceService>;
type AutoSyncSvc<'a> = State<'a, AutoSyncService>;

/// 列出所有工作空间
#[tauri::command]
pub async fn list_workspaces(
    service: WorkspaceSvc<'_>,
) -> std::result::Result<Vec<Workspace>, String> {
    log::info!("[commands/workspaces.rs::list_workspaces] 列出工作空间");

    service
        .list_workspaces()
        .map_err(|e| {
            log::error!("[commands/workspaces.rs::list_workspaces] 列出失败: {}", e);
            e.to_string()
        })
        .map(|workspaces| {
            log::info!("[commands/workspaces.rs::list_workspaces] 列出成功，找到 {} 个工作空间", workspaces.len());
            workspaces
        })
}

/// 创建工作空间
#[tauri::command]
pub async fn create_workspace(
    req: CreateWorkspaceRequest,
    service: WorkspaceSvc<'_>,
) -> std::result::Result<Workspace, String> {
    log::info!("[commands/workspaces.rs::create_workspace] 创建工作空间: name={}", req.name);

    service
        .create_workspace(req)
        .map_err(|e| {
            log::error!("[commands/workspaces.rs::create_workspace] 创建失败: {}", e);
            e.to_string()
        })
        .map(|workspace| {
            log::info!("[commands/workspaces.rs::create_workspace] 创建成功: id={}, name={}", workspace.id, workspace.name);
            workspace
        })
}

/// 更新工作空间
#[tauri::command]
pub async fn update_workspace(
    req: UpdateWorkspaceRequest,
    service: WorkspaceSvc<'_>,
) -> std::result::Result<Workspace, String> {
    let id = req.id.clone();
    log::info!("[commands/workspaces.rs::update_workspace] 更新工作空间: id={}", id);

    service
        .update_workspace(req)
        .map_err(|e| {
            log::error!("[commands/workspaces.rs::update_workspace] 更新失败: id={}, error={}", id, e);
            e.to_string()
        })
        .map(|workspace| {
            log::info!("[commands/workspaces.rs::update_workspace] 更新成功: id={}", id);
            workspace
        })
}

/// 删除工作空间（软删除）
#[tauri::command]
pub async fn delete_workspace(
    id: String,
    service: WorkspaceSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/workspaces.rs::delete_workspace] 删除工作空间: id={}", id);

    service
        .delete_workspace(&id)
        .map_err(|e| {
            log::error!("[commands/workspaces.rs::delete_workspace] 删除失败: id={}, error={}", id, e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/workspaces.rs::delete_workspace] 删除成功: id={}", id);
        })
}

/// 设置默认工作空间
#[tauri::command]
pub async fn set_default_workspace(
    id: String,
    service: WorkspaceSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/workspaces.rs::set_default_workspace] 设置默认工作空间: id={}", id);

    service
        .set_default_workspace(&id)
        .map_err(|e| {
            log::error!("[commands/workspaces.rs::set_default_workspace] 设置失败: id={}, error={}", id, e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/workspaces.rs::set_default_workspace] 设置成功: id={}", id);
        })
}

/// 获取当前工作空间
#[tauri::command]
pub async fn get_current_workspace(
    service: WorkspaceSvc<'_>,
) -> std::result::Result<Workspace, String> {
    log::info!("[commands/workspaces.rs::get_current_workspace] 获取当前工作空间");

    service
        .get_current_workspace()
        .map_err(|e| {
            log::error!("[commands/workspaces.rs::get_current_workspace] 获取失败: {}", e);
            e.to_string()
        })
        .map(|workspace| {
            log::info!("[commands/workspaces.rs::get_current_workspace] 获取成功: id={}, name={}", workspace.id, workspace.name);
            workspace
        })
}

/// 切换工作空间
#[tauri::command]
pub async fn switch_workspace(
    id: String,
    service: WorkspaceSvc<'_>,
    auto_sync: AutoSyncSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/workspaces.rs::switch_workspace] 切换工作空间: id={}", id);

    // 注意：工作空间切换是同一用户内的操作，不需要停止自动同步服务
    // 但正在进行的同步会通过会话验证机制自动检测并取消

    service
        .switch_workspace(&id)
        .map_err(|e| {
            log::error!("[commands/workspaces.rs::switch_workspace] 切换失败: id={}, error={}", id, e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/workspaces.rs::switch_workspace] 切换成功: id={}", id);
        })
}

/// 迁移孤立数据到当前工作空间
///
/// 将所有 workspace_id = NULL 的数据（未登录时创建的数据）分配到指定的工作空间
#[tauri::command]
pub async fn migrate_orphan_data_to_workspace(
    workspace_id: String,
    service: WorkspaceSvc<'_>,
) -> std::result::Result<MigrateResult, String> {
    log::info!("[commands/workspaces.rs::migrate_orphan_data_to_workspace] 开始迁移孤立数据到工作空间: workspace_id={}", workspace_id);

    service
        .migrate_orphan_data_to_workspace(&workspace_id)
        .map_err(|e| {
            log::error!("[commands/workspaces.rs::migrate_orphan_data_to_workspace] 迁移失败: {}", e);
            e.to_string()
        })
        .map(|result| {
            log::info!(
                "[commands/workspaces.rs::migrate_orphan_data_to_workspace] 迁移成功: notes={}, folders={}, tags={}, snapshots={}",
                result.notes,
                result.folders,
                result.tags,
                result.snapshots
            );
            result
        })
}


