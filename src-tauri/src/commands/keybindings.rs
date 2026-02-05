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
    log::debug!("[commands/keybindings.rs::load_keybindings] 加载快捷键配置");

    service.load_keybindings()
        .map_err(|e| {
            log::error!("[commands/keybindings.rs::load_keybindings] 加载失败: {}", e);
            e.to_string()
        })
        .map(|data| {
            log::debug!("[commands/keybindings.rs::load_keybindings] 加载成功: keybindings_count={}", data.keybindings.len());
            data
        })
}

/// 保存快捷键配置
#[tauri::command]
pub async fn save_keybindings(
    keybindings: HashMap<String, KeyCombination>,
    presets: Vec<KeybindingPreset>,
    service: KeybindingSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/keybindings.rs::save_keybindings] 保存快捷键配置: count={}", keybindings.len());

    service.save_keybindings(keybindings, presets)
        .map_err(|e| {
            log::error!("[commands/keybindings.rs::save_keybindings] 保存失败: {}", e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/keybindings.rs::save_keybindings] 保存成功");
        })
}

/// 导入快捷键配置
#[tauri::command]
pub async fn import_keybindings(
    json_string: String,
    service: KeybindingSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/keybindings.rs::import_keybindings] 导入快捷键配置: json_length={}", json_string.len());

    service.import_keybindings(&json_string)
        .map_err(|e| {
            log::error!("[commands/keybindings.rs::import_keybindings] 导入失败: {}", e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/keybindings.rs::import_keybindings] 导入成功");
        })
}

/// 重置为默认配置
#[tauri::command]
pub async fn reset_keybindings(
    service: KeybindingSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/keybindings.rs::reset_keybindings] 重置为默认配置");

    service.reset_keybindings()
        .map_err(|e| {
            log::error!("[commands/keybindings.rs::reset_keybindings] 重置失败: {}", e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/keybindings.rs::reset_keybindings] 重置成功");
        })
}
