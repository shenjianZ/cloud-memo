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
    service.create_folder(req)
        .map_err(|e| e.to_string())
}

/// 获取单个文件夹
#[tauri::command]
pub async fn get_folder(
    id: String,
    service: FolderSvc<'_>,
) -> std::result::Result<Folder, String> {
    service.get_folder(&id)
        .map_err(|e| e.to_string())
}

/// 更新文件夹
#[tauri::command]
pub async fn update_folder(
    req: UpdateFolderRequest,
    service: FolderSvc<'_>,
) -> std::result::Result<Folder, String> {
    service.update_folder(req)
        .map_err(|e| e.to_string())
}

/// 删除文件夹（软删除）
#[tauri::command]
pub async fn delete_folder(
    id: String,
    service: FolderSvc<'_>,
) -> std::result::Result<(), String> {
    service.delete_folder(&id)
        .map_err(|e| e.to_string())
}

/// 获取所有文件夹
#[tauri::command]
pub async fn list_folders(
    service: FolderSvc<'_>,
) -> std::result::Result<Vec<Folder>, String> {
    service.list_folders()
        .map_err(|e| e.to_string())
}

/// 移动文件夹
#[tauri::command]
pub async fn move_folder(
    req: MoveFolderRequest,
    service: FolderSvc<'_>,
) -> std::result::Result<Folder, String> {
    service.move_folder(req)
        .map_err(|e| e.to_string())
}

/// 获取文件夹路径
#[tauri::command]
pub async fn get_folder_path(
    id: String,
    service: FolderSvc<'_>,
) -> std::result::Result<Vec<Folder>, String> {
    service.get_folder_path(&id)
        .map_err(|e| e.to_string())
}
