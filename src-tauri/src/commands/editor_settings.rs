use crate::services::EditorSettingsService;
use crate::models::{EditorSettings, UpdateEditorSettingsRequest};
use tauri::State;

type EditorSettingsSvc<'a> = State<'a, EditorSettingsService>;

/// 获取编辑器设置
#[tauri::command]
pub async fn get_editor_settings(
    service: EditorSettingsSvc<'_>,
) -> std::result::Result<EditorSettings, String> {
    log::debug!("[commands/editor_settings.rs::get_editor_settings] 获取编辑器设置");

    service.get_settings()
        .map_err(|e| {
            log::error!("[commands/editor_settings.rs::get_editor_settings] 获取失败: {}", e);
            e.to_string()
        })
        .map(|settings| {
            log::debug!("[commands/editor_settings.rs::get_editor_settings] 获取成功");
            settings
        })
}

/// 更新编辑器设置
#[tauri::command]
pub async fn update_editor_settings(
    req: UpdateEditorSettingsRequest,
    service: EditorSettingsSvc<'_>,
) -> std::result::Result<EditorSettings, String> {
    log::info!("[commands/editor_settings.rs::update_editor_settings] 更新编辑器设置");

    service.update_settings(req)
        .map_err(|e| {
            log::error!("[commands/editor_settings.rs::update_editor_settings] 更新失败: {}", e);
            e.to_string()
        })
        .map(|settings| {
            log::info!("[commands/editor_settings.rs::update_editor_settings] 更新成功");
            settings
        })
}
