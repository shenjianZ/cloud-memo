use crate::models::{AppSettings, UpdateAppSettings};
use crate::services::AppSettingsService;
use tauri::State;

/// AppSettings service 类型别名
type AppSettingsSvc<'a> = State<'a, AppSettingsService>;

/// 获取应用设置
#[tauri::command]
pub async fn get_app_settings(
    service: AppSettingsSvc<'_>,
) -> Result<AppSettings, String> {
    log::debug!("[commands/app_settings.rs::get_app_settings] 获取应用设置");

    service.get_settings()
        .map_err(|e| {
            log::error!("[commands/app_settings.rs::get_app_settings] 获取失败: {}", e);
            e.to_string()
        })
        .map(|settings| {
            log::debug!("[commands/app_settings.rs::get_app_settings] 获取成功");
            settings
        })
}

/// 更新应用设置
#[tauri::command]
pub async fn update_app_settings(
    service: AppSettingsSvc<'_>,
    updates: UpdateAppSettings,
) -> Result<AppSettings, String> {
    log::info!("[commands/app_settings.rs::update_app_settings] 更新应用设置");

    service.update_settings(updates)
        .map_err(|e| {
            log::error!("[commands/app_settings.rs::update_app_settings] 更新失败: {}", e);
            e.to_string()
        })
        .map(|settings| {
            log::info!("[commands/app_settings.rs::update_app_settings] 更新成功");
            settings
        })
}

/// 重置应用设置为默认值
#[tauri::command]
pub async fn reset_app_settings(
    service: AppSettingsSvc<'_>,
) -> Result<AppSettings, String> {
    log::info!("[commands/app_settings.rs::reset_app_settings] 重置应用设置为默认值");

    service.reset_to_default()
        .map_err(|e| {
            log::error!("[commands/app_settings.rs::reset_app_settings] 重置失败: {}", e);
            e.to_string()
        })
        .map(|settings| {
            log::info!("[commands/app_settings.rs::reset_app_settings] 重置成功");
            settings
        })
}

/// 获取默认服务器 URL
#[tauri::command]
pub async fn get_default_server_url(
    service: AppSettingsSvc<'_>,
) -> Result<String, String> {
    log::debug!("[commands/app_settings.rs::get_default_server_url] 获取默认服务器 URL");

    service.get_default_server_url()
        .map_err(|e| {
            log::error!("[commands/app_settings.rs::get_default_server_url] 获取失败: {}", e);
            e.to_string()
        })
        .map(|url| {
            log::debug!("[commands/app_settings.rs::get_default_server_url] 获取成功: url={}", url);
            url
        })
}
