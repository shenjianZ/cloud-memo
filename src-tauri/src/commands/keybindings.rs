use crate::services::KeybindingService;
use crate::models::{KeyCombination, KeybindingPreset};
use tauri::State;
use std::collections::HashMap;

/// Keybinding service 类型别名
type KeybindingSvc<'a> = State<'a, KeybindingService>;

/// 加载快捷键配置
#[tauri::command]
pub async fn load_keybindings(
    service: KeybindingSvc<'_>,
) -> std::result::Result<crate::models::KeybindingsData, String> {
    service.load_keybindings()
        .map_err(|e| e.to_string())
}

/// 保存快捷键配置
#[tauri::command]
pub async fn save_keybindings(
    keybindings: HashMap<String, KeyCombination>,
    presets: Vec<KeybindingPreset>,
    service: KeybindingSvc<'_>,
) -> std::result::Result<(), String> {
    service.save_keybindings(keybindings, presets)
        .map_err(|e| e.to_string())
}

/// 导入快捷键配置
#[tauri::command]
pub async fn import_keybindings(
    json_string: String,
    service: KeybindingSvc<'_>,
) -> std::result::Result<(), String> {
    service.import_keybindings(&json_string)
        .map_err(|e| e.to_string())
}

/// 重置为默认配置
#[tauri::command]
pub async fn reset_keybindings(
    service: KeybindingSvc<'_>,
) -> std::result::Result<(), String> {
    service.reset_keybindings()
        .map_err(|e| e.to_string())
}
