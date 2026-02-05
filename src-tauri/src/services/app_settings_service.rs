use crate::models::{AppSettings, UpdateAppSettings};
use crate::models::error::{Result, AppError};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

/// 应用设置服务
///
/// 管理全局应用配置（默认服务器、同步设置等）
pub struct AppSettingsService {
    pool: Pool<SqliteConnectionManager>,
}

impl AppSettingsService {
    /// 创建新的 AppSettingsService 实例
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    /// 获取应用设置
    pub fn get_settings(&self) -> Result<AppSettings> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, default_server_url, auto_sync_enabled, sync_interval_minutes,
                    theme, language, updated_at
             FROM app_settings
             WHERE id = 1"
        ).map_err(|e| AppError::DatabaseError(format!("Failed to query settings: {}", e)))?;

        let settings = stmt.query_row([], |row| {
            Ok(AppSettings {
                id: row.get(0)?,
                default_server_url: row.get(1)?,
                auto_sync_enabled: row.get(2)?,
                sync_interval_minutes: row.get(3)?,
                theme: row.get(4)?,
                language: row.get(5)?,
                updated_at: row.get(6)?,
            })
        }).map_err(|e| AppError::DatabaseError(format!("Settings not found: {}", e)))?;

        Ok(settings)
    }

    /// 更新应用设置
    pub fn update_settings(&self, updates: UpdateAppSettings) -> Result<AppSettings> {
        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 获取当前设置
        let current = self.get_settings()?;

        // 构建更新后的设置
        let updated = AppSettings {
            default_server_url: updates.default_server_url.unwrap_or(current.default_server_url),
            auto_sync_enabled: updates.auto_sync_enabled.unwrap_or(current.auto_sync_enabled),
            sync_interval_minutes: updates.sync_interval_minutes.unwrap_or(current.sync_interval_minutes),
            theme: updates.theme.unwrap_or(current.theme),
            language: updates.language.unwrap_or(current.language),
            updated_at: chrono::Utc::now().timestamp(),
            id: 1,
        };

        // 更新数据库
        conn.execute(
            "UPDATE app_settings
             SET default_server_url = ?1, auto_sync_enabled = ?2, sync_interval_minutes = ?3,
                 theme = ?4, language = ?5, updated_at = ?6
             WHERE id = 1",
            (
                &updated.default_server_url,
                updated.auto_sync_enabled,
                updated.sync_interval_minutes,
                &updated.theme,
                &updated.language,
                updated.updated_at,
            ),
        ).map_err(|e| AppError::DatabaseError(format!("Failed to update settings: {}", e)))?;

        log::info!("App settings updated");
        Ok(updated)
    }

    /// 获取默认服务器 URL
    pub fn get_default_server_url(&self) -> Result<String> {
        let settings = self.get_settings()?;
        Ok(settings.default_server_url)
    }

    /// 重置为默认设置
    pub fn reset_to_default(&self) -> Result<AppSettings> {
        let default = AppSettings::default();
        let now = chrono::Utc::now().timestamp();

        let conn = self.pool.get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        conn.execute(
            "UPDATE app_settings
             SET default_server_url = ?1, auto_sync_enabled = ?2, sync_interval_minutes = ?3,
                 theme = ?4, language = ?5, updated_at = ?6
             WHERE id = 1",
            (
                &default.default_server_url,
                default.auto_sync_enabled,
                default.sync_interval_minutes,
                &default.theme,
                &default.language,
                now,
            ),
        ).map_err(|e| AppError::DatabaseError(format!("Failed to reset settings: {}", e)))?;

        log::info!("App settings reset to default");
        Ok(default)
    }
}
