use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// 笔记模型
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    // ===== 基础字段 =====
    pub id: String,  // 笔记唯一标识（UUID）
    pub title: String,  // 笔记标题
    pub content: String,  // 笔记内容（Tiptap JSON 格式）

    // ===== 摘要与缓存 =====
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,  // 内容摘要（前 200 字符）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown_cache: Option<String>,  // Markdown 缓存（用于导出/兼容）

    // ===== 分类与标记 =====
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,  // 所属工作空间 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,  // 所属文件夹 ID
    #[serde(default)]
    pub is_favorite: bool,  // 是否收藏
    #[serde(default)]
    pub is_pinned: bool,  // 是否置顶
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,  // 作者

    // ===== 状态字段 =====
    #[serde(default)]
    pub is_deleted: bool,  // 是否已删除（软删除）

    // ===== 时间戳 =====
    pub created_at: i64,  // 创建时间（Unix 时间戳，秒）
    pub updated_at: i64,  // 更新时间（Unix 时间戳，秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,  // 删除时间（Unix 时间戳，秒）

    // ===== 统计字段 =====
    #[serde(default)]
    pub word_count: u32,  // 字数统计
    #[serde(default)]
    pub read_time_minutes: u32,  // 预计阅读时间（分钟）

    // ===== 云端同步字段 =====
    #[serde(default)]
    pub server_ver: i32,  // 服务器版本号（用于冲突检测）
    #[serde(default)]
    pub is_dirty: bool,  // 是否需要同步到服务器
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<i64>,  // 最后同步时间（Unix 时间戳，秒）
}

impl Note {
    /// 创建新笔记（构造函数）
    ///
    /// # 参数
    /// - `title`: 笔记标题
    /// - `content`: 笔记内容（Tiptap JSON 或 Markdown）
    /// - `folder_id`: 所属文件夹 ID（可选）
    pub fn new(title: String, content: String, folder_id: Option<String>) -> Self {
        let now = chrono::Utc::now().timestamp();
        let word_count = Self::count_words(&content);

        Self {
            id: Uuid::new_v4().to_string(),
            title,
            excerpt: Self::generate_excerpt(&content),
            markdown_cache: None,
            content,
            workspace_id: None,  // 将由 Service 层设置
            folder_id,
            is_favorite: false,
            is_deleted: false,
            is_pinned: false,
            author: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            word_count,
            read_time_minutes: Self::calculate_read_time(word_count),
            server_ver: 0,
            is_dirty: true,
            last_synced_at: None,
        }
    }

    /// 更新笔记内容并重新计算衍生值
    ///
    /// 应该在更新 content 时调用此方法，确保 excerpt、word_count、read_time_minutes 保持同步
    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.excerpt = Self::generate_excerpt(&self.content);
        self.word_count = Self::count_words(&self.content);
        self.read_time_minutes = Self::calculate_read_time(self.word_count);
    }

    /// 创建冲突副本（用于同步冲突解决）
    pub fn conflict_copy(&self, suffix: &str) -> Self {
        let now = chrono::Utc::now().timestamp();

        Self {
            id: Uuid::new_v4().to_string(),
            title: format!("{} ({})", self.title, suffix),
            excerpt: self.excerpt.clone(),
            markdown_cache: self.markdown_cache.clone(),
            content: self.content.clone(),
            workspace_id: self.workspace_id.clone(),
            folder_id: self.folder_id.clone(),
            is_favorite: self.is_favorite,
            is_deleted: false,
            is_pinned: self.is_pinned,
            author: self.author.clone(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            word_count: self.word_count,
            read_time_minutes: self.read_time_minutes,
            server_ver: 0,
            is_dirty: true,
            last_synced_at: None,
        }
    }

    /// 生成摘要（前 200 个字符）
    fn generate_excerpt(content: &str) -> Option<String> {
        let chars: Vec<char> = content.chars().collect();
        if chars.len() <= 200 {
            None
        } else {
            Some(chars[..200].iter().collect())
        }
    }

    /// 计算字数（按空白字符分割）
    fn count_words(content: &str) -> u32 {
        content.split_whitespace().count() as u32
    }

    /// 计算阅读时间（假设每分钟 200 字）
    fn calculate_read_time(word_count: u32) -> u32 {
        (word_count / 200).max(1)
    }
}

/// 创建笔记请求
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteRequest {
    pub title: String,  // 笔记标题
    pub content: String,  // 笔记内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,  // 所属文件夹 ID
}

/// 更新笔记请求
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteRequest {
    pub id: String,  // 笔记 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,  // 新标题
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,  // 新内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,  // 新文件夹 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,  // 是否收藏
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pinned: Option<bool>,  // 是否置顶
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,  // 作者
}
