use serde::{Serialize, Deserialize};

/// 笔记快照（手动版本）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NoteSnapshot {
    pub id: String,  // 快照唯一标识（UUID）
    pub note_id: String,  // 关联的笔记 ID
    pub title: String,  // 快照标题（继承自笔记）
    pub content: String,  // 快照内容（Tiptap JSON 格式）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_name: Option<String>,  // 快照名称（用户自定义）
    pub created_at: i64,  // 创建时间（Unix 时间戳，秒）
    #[serde(default)]
    pub server_ver: i32,  // 服务器版本号
    #[serde(default)]
    pub is_dirty: bool,  // 是否需要同步
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<i64>,  // 最后同步时间（Unix 时间戳，秒）
}

/// 创建快照请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSnapshotRequest {
    pub note_id: String,  // 笔记 ID
    pub title: String,  // 笔记标题
    pub content: String,  // 笔记内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_name: Option<String>,  // 快照名称（可选）
}

/// 快照列表项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotListItem {
    pub id: String,  // 快照 ID
    pub note_id: String,  // 关联的笔记 ID
    pub title: String,  // 快照标题
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_name: Option<String>,  // 快照名称
    pub created_at: i64,  // 创建时间（Unix 时间戳，秒）
    pub created_at_display: String,  // 格式化的时间显示（用于 UI 显示）
}
