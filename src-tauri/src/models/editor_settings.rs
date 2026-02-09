use serde::{Serialize, Deserialize};

/// 编辑器设置模型
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EditorSettings {
    pub id: i32,  // 设置 ID（固定为 1，单例模式）
    // ===== 内容字体设置 =====
    pub content_font_family: String,  // 正文字体族
    pub content_font_size: i32,  // 正文字体大小（px）
    pub content_font_weight: i32,  // 正文字体粗细（100-900）
    pub content_line_height: f64,  // 正文行高（倍数，如 1.7）
    // ===== 标题字体设置 =====
    pub heading_font_family: String,  // 标题字体族
    pub heading_font_weight: i32,  // 标题字体粗细（100-900）
    // ===== 代码字体设置 =====
    pub code_font_family: String,  // 代码字体族
    pub code_font_size: i32,  // 代码字体大小（px）
    // ===== Markdown 预览样式设置 =====
    pub markdown_preview_style: String,  // Markdown 预览样式：minimal（朴素）、default（默认）、rich（丰富）
    // ===== 时间戳 =====
    pub updated_at: i64,  // 更新时间（Unix 时间戳，秒）
}

/// 更新编辑器设置请求
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEditorSettingsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_font_family: Option<String>,  // 新正文字体族
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_font_size: Option<i32>,  // 新正文字体大小
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_font_weight: Option<i32>,  // 新正文字体粗细
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_line_height: Option<f64>,  // 新正文行高
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_font_family: Option<String>,  // 新标题字体族
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_font_weight: Option<i32>,  // 新标题字体粗细
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_font_family: Option<String>,  // 新代码字体族
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_font_size: Option<i32>,  // 新代码字体大小
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown_preview_style: Option<String>,  // 新 Markdown 预览样式
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
            markdown_preview_style: "default".to_string(),
            updated_at: chrono::Utc::now().timestamp(),
        }
    }
}
