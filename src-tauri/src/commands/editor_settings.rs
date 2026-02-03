use crate::services::EditorSettingsService;
use crate::models::{EditorSettings, UpdateEditorSettingsRequest};
use tauri::State;

type EditorSettingsSvc<'a> = State<'a, EditorSettingsService>;

/// 获取编辑器设置
#[tauri::command]
pub async fn get_editor_settings(
    service: EditorSettingsSvc<'_>,
) -> std::result::Result<EditorSettings, String> {
    service.get_settings().map_err(|e| e.to_string())
}

/// 更新编辑器设置
#[tauri::command]
pub async fn update_editor_settings(
    req: UpdateEditorSettingsRequest,
    service: EditorSettingsSvc<'_>,
) -> std::result::Result<EditorSettings, String> {
    service.update_settings(req).map_err(|e| e.to_string())
}
