use serde::{Serialize, Deserialize};

/// 编辑器设置模型
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EditorSettings {
    pub id: i32, // 固定为 1，单例模式
    pub content_font_family: String,
    pub content_font_size: i32,
    pub content_font_weight: i32,
    pub content_line_height: f64,
    pub heading_font_family: String,
    pub heading_font_weight: i32,
    pub code_font_family: String,
    pub code_font_size: i32,
    pub updated_at: i64,
}

/// 更新编辑器设置请求
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEditorSettingsRequest {
    pub content_font_family: Option<String>,
    pub content_font_size: Option<i32>,
    pub content_font_weight: Option<i32>,
    pub content_line_height: Option<f64>,
    pub heading_font_family: Option<String>,
    pub heading_font_weight: Option<i32>,
    pub code_font_family: Option<String>,
    pub code_font_size: Option<i32>,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            id: 1,
            content_font_family: "Inter, Avenir, Helvetica, Arial, sans-serif".to_string(),
            content_font_size: 16,
            content_font_weight: 400,
            content_line_height: 1.7,
            heading_font_family: "Inter, Avenir, Helvetica, Arial, sans-serif".to_string(),
            heading_font_weight: 600,
            code_font_family: "JetBrains Mono, Fira Code, Consolas, Courier New, monospace".to_string(),
            code_font_size: 14,
            updated_at: chrono::Utc::now().timestamp(),
        }
    }
}
