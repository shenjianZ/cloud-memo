use serde::{Deserialize, Serialize};

/// 标签模型
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,  // 标签唯一标识（UUID）
    pub name: String,  // 标签名称
    pub color: Option<String>,  // 标签颜色（十六进制或颜色名）
    pub created_at: i64,  // 创建时间（Unix 时间戳，秒）
    pub updated_at: i64,  // 更新时间（Unix 时间戳，秒）
    #[serde(default)]
    pub is_deleted: bool,  // 是否已删除（软删除）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,  // 删除时间（Unix 时间戳，秒）
    #[serde(default)]
    pub server_ver: i32,  // 服务器版本号
    #[serde(default)]
    pub is_dirty: bool,  // 是否需要同步
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<i64>,  // 最后同步时间（Unix 时间戳，秒）
}

/// 创建标签请求
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagRequest {
    pub name: String,  // 标签名称
    pub color: Option<String>,  // 标签颜色
}

/// 更新标签请求
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTagRequest {
    pub name: Option<String>,  // 新标签名称
    pub color: Option<String>,  // 新标签颜色
}

/// 笔记-标签关联请求
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteTagRequest {
    pub note_id: String,  // 笔记 ID
    pub tag_id: String,  // 标签 ID
}

/// 笔记-标签关联模型
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NoteTagRelation {
    pub note_id: String,
    pub tag_id: String,
    pub user_id: String,
    pub created_at: i64,
    #[serde(default)]
    pub is_deleted: bool,  // 是否已删除（软删除）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,  // 删除时间（Unix 时间戳，秒）
}
