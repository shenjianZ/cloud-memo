use crate::services::FolderService;
use crate::models::{Folder, CreateFolderRequest, UpdateFolderRequest, MoveFolderRequest};
use tauri::State;

/// Folder service 类型别名
type FolderSvc<'a> = State<'a, FolderService>;

/// 创建文件夹
#[tauri::command]
pub async fn create_folder(
    req: CreateFolderRequest,
    service: FolderSvc<'_>,
) -> std::result::Result<Folder, String> {
    log::info!("[commands/folders.rs::create_folder] 创建文件夹: name={}", req.name);

    service.create_folder(req)
        .map_err(|e| {
            log::error!("[commands/folders.rs::create_folder] 创建失败: {}", e);
            e.to_string()
        })
        .map(|folder| {
            log::info!("[commands/folders.rs::create_folder] 创建成功: id={}, name={}", folder.id, folder.name);
            folder
        })
}

/// 获取单个文件夹
#[tauri::command]
pub async fn get_folder(
    id: String,
    service: FolderSvc<'_>,
) -> std::result::Result<Folder, String> {
    log::debug!("[commands/folders.rs::get_folder] 获取文件夹: id={}", id);

    service.get_folder(&id)
        .map_err(|e| {
            log::error!("[commands/folders.rs::get_folder] 获取失败: id={}, error={}", id, e);
            e.to_string()
        })
}

/// 更新文件夹
#[tauri::command]
pub async fn update_folder(
    req: UpdateFolderRequest,
    service: FolderSvc<'_>,
) -> std::result::Result<Folder, String> {
    let folder_id = req.id.clone();
    log::debug!("[commands/folders.rs::update_folder] 更新文件夹: id={}", folder_id);

    service.update_folder(req)
        .map_err(|e| {
            log::error!("[commands/folders.rs::update_folder] 更新失败: id={}, error={}", folder_id, e);
            e.to_string()
        })
        .map(|folder| {
            log::debug!("[commands/folders.rs::update_folder] 更新成功: id={}", folder_id);
            folder
        })
}

/// 删除文件夹（软删除）
#[tauri::command]
pub async fn delete_folder(
    id: String,
    service: FolderSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/folders.rs::delete_folder] 删除文件夹: id={}", id);

    service.delete_folder(&id)
        .map_err(|e| {
            log::error!("[commands/folders.rs::delete_folder] 删除失败: id={}, error={}", id, e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/folders.rs::delete_folder] 删除成功: id={}", id);
        })
}

/// 获取所有文件夹
#[tauri::command]
pub async fn list_folders(
    service: FolderSvc<'_>,
) -> std::result::Result<Vec<Folder>, String> {
    log::debug!("[commands/folders.rs::list_folders] 获取文件夹列表");

    service.list_folders()
        .map_err(|e| {
            log::error!("[commands/folders.rs::list_folders] 获取失败: {}", e);
            e.to_string()
        })
        .map(|folders| {
            log::debug!("[commands/folders.rs::list_folders] 获取成功: count={}", folders.len());
            folders
        })
}

/// 移动文件夹
#[tauri::command]
pub async fn move_folder(
    req: MoveFolderRequest,
    service: FolderSvc<'_>,
) -> std::result::Result<Folder, String> {
    let folder_id = req.id.clone();
    let new_parent_id_display = req.new_parent_id.as_deref().unwrap_or("root");
    log::info!("[commands/folders.rs::move_folder] 移动文件夹: id={}, new_parent_id={}", folder_id, new_parent_id_display);

    service.move_folder(req)
        .map_err(|e| {
            log::error!("[commands/folders.rs::move_folder] 移动失败: {}", e);
            e.to_string()
        })
        .map(|folder| {
            log::info!("[commands/folders.rs::move_folder] 移动成功: id={}", folder_id);
            folder
        })
}

/// 获取文件夹路径
#[tauri::command]
pub async fn get_folder_path(
    id: String,
    service: FolderSvc<'_>,
) -> std::result::Result<Vec<Folder>, String> {
    log::debug!("[commands/folders.rs::get_folder_path] 获取文件夹路径: id={}", id);

    service.get_folder_path(&id)
        .map_err(|e| {
            log::error!("[commands/folders.rs::get_folder_path] 获取失败: id={}, error={}", id, e);
            e.to_string()
        })
        .map(|path| {
            log::debug!("[commands/folders.rs::get_folder_path] 获取成功: path_count={}", path.len());
            path
        })
}
