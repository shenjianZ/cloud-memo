use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
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
    pub revoked: bool,
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
    pub title: String,
    pub content: String,
    pub folder_id: Option<String>,
    pub is_deleted: bool,
    pub deleted_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub server_ver: i32,
    // ===== 客户端同步字段 =====
    #[serde(default)]
    pub is_dirty: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Folder {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub is_deleted: bool,
    pub deleted_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub server_ver: i32,
    // ===== 客户端同步字段 =====
    #[serde(default)]
    pub is_dirty: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<i64>,
}
