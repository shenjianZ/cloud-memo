use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Device {
    pub id: String,
    pub user_id: String,
    pub device_name: String,
    pub device_type: String,  // "desktop", "laptop", "mobile", "tablet"
    pub revoked: bool,  // ✅ 已存在
    pub last_seen_at: i64,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SyncHistoryEntry {
    pub id: String,
    pub user_id: String,
    pub sync_type: String,  // "push", "pull", "full"
    pub pushed_count: i32,
    pub pulled_count: i32,
    pub conflict_count: i32,
    pub error: Option<String>,
    pub duration_ms: i64,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Note {
    pub id: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub title: String,
    pub content: String,
    pub folder_id: Option<String>,
    pub is_deleted: bool,
    pub deleted_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub server_ver: i32,
    // ===== 设备追踪字段 =====
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by_device: Option<String>,
    // ===== 客户端 UI 特有字段 =====
    #[serde(default)]
    pub excerpt: Option<String>,
    #[serde(default)]
    pub markdown_cache: Option<String>,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub is_pinned: bool,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub word_count: i32,
    #[serde(default)]
    pub read_time_minutes: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Folder {
    pub id: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub name: String,
    pub parent_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub is_deleted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
    pub server_ver: i32,
    // ===== 设备追踪字段 =====
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by_device: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub name: String,
    pub color: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub is_deleted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
    pub server_ver: i32,
    // ===== 设备追踪字段 =====
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by_device: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NoteVersion {
    pub id: String,
    pub note_id: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub title: String,
    pub content: String,
    pub snapshot_name: Option<String>,
    pub created_at: i64,
    pub server_ver: i32,
    // ===== 设备追踪字段 =====
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NoteTagRelation {
    pub note_id: String,
    pub tag_id: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub created_at: i64,
    #[serde(default)]
    pub is_deleted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Workspace {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    pub is_deleted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub server_ver: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by_device: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SyncLock {
    pub id: String,
    pub user_id: String,
    pub device_id: String,
    pub workspace_id: Option<String>,
    pub acquired_at: i64,
    pub expires_at: i64,
}

/// 冲突解决策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ConflictResolutionStrategy {
    /// 保留两个版本（创建冲突副本）
    #[default]
    KeepBoth,
    /// 服务器版本优先
    KeepServer,
    /// 本地版本优先
    KeepLocal,
    /// 手动合并（等待用户处理）
    ManualMerge,
}
