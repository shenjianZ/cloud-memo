use serde::{Deserialize, Serialize};

/// 应用设置模型（全局配置）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub id: i32,
    pub default_server_url: String,
    pub auto_sync_enabled: bool,
    pub sync_interval_minutes: i32,
    pub theme: String,
    pub language: String,
    pub updated_at: i64,
}

/// 更新应用设置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAppSettings {
    pub default_server_url: Option<String>,
    pub auto_sync_enabled: Option<bool>,
    pub sync_interval_minutes: Option<i32>,
    pub theme: Option<String>,
    pub language: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: 1,
            default_server_url: "https://api.noteapp.com".to_string(),
            auto_sync_enabled: true,
            sync_interval_minutes: 5,
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
            updated_at: now,
        }
    }
}
