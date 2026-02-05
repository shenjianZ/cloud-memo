use serde::{Serialize, Deserialize};
use crate::models::Note;
use crate::models::Folder;

/// 同步请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRequest {
    pub notes: Vec<Note>,  // 要同步的笔记列表
    pub folders: Vec<Folder>,  // 要同步的文件夹列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync_at: Option<i64>,  // 上次同步时间（增量同步使用，Unix 时间戳，秒）
}

/// 同步响应
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResponse {
    pub notes: Vec<Note>,  // 服务器返回的笔记列表
    pub folders: Vec<Folder>,  // 服务器返回的文件夹列表
    #[serde(default)]
    pub conflicts: Vec<ConflictInfo>,  // 冲突列表
    pub server_time: i64,  // 服务器当前时间（Unix 时间戳，秒）
}

/// 冲突信息
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConflictInfo {
    pub id: String,  // 冲突实体的 ID
    pub entity_type: String,  // 实体类型："note" 或 "folder"
    pub local_version: i32,  // 本地版本号
    pub server_version: i32,  // 服务器版本号
    pub title: String,  // 实体标题（用于显示）
}

/// 同步状态
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub last_sync_at: Option<i64>,  // 最后同步时间（Unix 时间戳，秒）
    pub pending_count: i32,  // 待同步数量
    pub conflict_count: i32,  // 冲突数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,  // 最后一次错误信息
}

/// 同步结果报告
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncReport {
    pub success: bool,  // 同步是否成功
    pub pushed_count: usize,  // 推送到服务器的数量
    pub pulled_count: usize,  // 从服务器拉取的数量
    pub conflict_count: usize,  // 冲突数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,  // 错误信息（如果有）
}

/// 冲突解决策略
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ConflictStrategy {
    KeepServer,  // 保留服务器版本
    KeepLocal,  // 保留本地版本
    KeepBoth,  // 保留两个版本（创建冲突副本）
}
