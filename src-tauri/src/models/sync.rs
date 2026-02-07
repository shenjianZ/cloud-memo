use serde::{Serialize, Deserialize};
use crate::models::Note;
use crate::models::Folder;
use crate::models::Tag;
use crate::models::NoteSnapshot;

/// 同步类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncType {
    Notes,
    Folders,
    Tags,
    Snapshots,
    All,
}

/// ===== 服务器通信专用结构体（snake_case） =====

/// 服务器笔记（用于与服务器通信，snake_case）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerNote {
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
    // 客户端 UI 特有字段
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

impl From<Note> for ServerNote {
    fn from(note: Note) -> Self {
        ServerNote {
            id: note.id,
            user_id: String::new(), // 客户端 Note 没有 user_id，由服务器端填充
            title: note.title,
            content: note.content,
            folder_id: note.folder_id,
            is_deleted: note.is_deleted,
            deleted_at: note.deleted_at,
            created_at: note.created_at,
            updated_at: note.updated_at,
            server_ver: note.server_ver,
            // 保留客户端特有字段（类型转换 u32 -> i32）
            excerpt: note.excerpt,
            markdown_cache: note.markdown_cache,
            is_favorite: note.is_favorite,
            is_pinned: note.is_pinned,
            author: note.author,
            word_count: note.word_count as i32,
            read_time_minutes: note.read_time_minutes as i32,
        }
    }
}

impl From<ServerNote> for Note {
    fn from(note: ServerNote) -> Self {
        Note {
            id: note.id,
            title: note.title,
            content: note.content,
            folder_id: note.folder_id,
            is_deleted: note.is_deleted,
            deleted_at: note.deleted_at,
            created_at: note.created_at,
            updated_at: note.updated_at,
            server_ver: note.server_ver,
            // 使用服务器返回的客户端特有字段（类型转换 i32 -> u32）
            excerpt: note.excerpt,
            markdown_cache: note.markdown_cache,
            is_favorite: note.is_favorite,
            is_pinned: note.is_pinned,
            author: note.author,
            word_count: note.word_count as u32,
            read_time_minutes: note.read_time_minutes as u32,
            // ✅ 客户端本地管理这些字段
            is_dirty: false,
            last_synced_at: Some(chrono::Utc::now().timestamp()),
        }
    }
}

/// 服务器文件夹（用于与服务器通信，snake_case）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerFolder {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub is_deleted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
    pub server_ver: i32,
}

impl From<Folder> for ServerFolder {
    fn from(folder: Folder) -> Self {
        ServerFolder {
            id: folder.id,
            user_id: String::new(), // 客户端 Folder 没有 user_id，由服务器端填充
            name: folder.name,
            parent_id: folder.parent_id,
            created_at: folder.created_at,
            updated_at: folder.updated_at,
            is_deleted: folder.is_deleted,
            deleted_at: folder.deleted_at,
            server_ver: folder.server_ver,
        }
    }
}

impl From<ServerFolder> for Folder {
    fn from(folder: ServerFolder) -> Self {
        Folder {
            id: folder.id,
            name: folder.name,
            parent_id: folder.parent_id,
            created_at: folder.created_at,
            updated_at: folder.updated_at,
            is_deleted: folder.is_deleted,
            deleted_at: folder.deleted_at,
            server_ver: folder.server_ver,
            // ✅ 客户端本地管理这些字段
            is_dirty: false,
            last_synced_at: Some(chrono::Utc::now().timestamp()),
            // Folder 特有字段，服务器不返回
            icon: None,
            color: None,
            sort_order: 0,
        }
    }
}

/// 服务器标签（用于与服务器通信，snake_case）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerTag {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub is_deleted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
    pub server_ver: i32,
}

impl From<Tag> for ServerTag {
    fn from(tag: Tag) -> Self {
        ServerTag {
            id: tag.id,
            user_id: String::new(), // 客户端 Tag 没有 user_id，由服务器端填充
            name: tag.name,
            color: tag.color,
            created_at: tag.created_at,
            updated_at: tag.updated_at,
            is_deleted: tag.is_deleted,
            deleted_at: tag.deleted_at,
            server_ver: tag.server_ver,
        }
    }
}

impl From<ServerTag> for Tag {
    fn from(tag: ServerTag) -> Self {
        Tag {
            id: tag.id,
            name: tag.name,
            color: tag.color,
            created_at: tag.created_at,
            updated_at: tag.updated_at,
            is_deleted: tag.is_deleted,
            deleted_at: tag.deleted_at,
            server_ver: tag.server_ver,
            // ✅ 客户端本地管理这些字段
            is_dirty: false,
            last_synced_at: Some(chrono::Utc::now().timestamp()),
        }
    }
}

/// 服务器快照（用于与服务器通信，snake_case）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerNoteSnapshot {
    pub id: String,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub snapshot_name: Option<String>,
    pub created_at: i64,
    #[serde(default)]
    pub server_ver: i32,
}

impl From<NoteSnapshot> for ServerNoteSnapshot {
    fn from(snapshot: NoteSnapshot) -> Self {
        ServerNoteSnapshot {
            id: snapshot.id,
            note_id: snapshot.note_id,
            title: snapshot.title,
            content: snapshot.content,
            snapshot_name: snapshot.snapshot_name,
            created_at: snapshot.created_at,
            server_ver: snapshot.server_ver,
        }
    }
}

impl From<ServerNoteSnapshot> for NoteSnapshot {
    fn from(snapshot: ServerNoteSnapshot) -> Self {
        NoteSnapshot {
            id: snapshot.id,
            note_id: snapshot.note_id,
            title: snapshot.title,
            content: snapshot.content,
            snapshot_name: snapshot.snapshot_name,
            created_at: snapshot.created_at,
            server_ver: snapshot.server_ver,
            // ✅ 客户端本地管理这些字段
            is_dirty: false,
            last_synced_at: Some(chrono::Utc::now().timestamp()),
        }
    }
}

/// 笔记标签关联（用于与服务器通信，snake_case）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerNoteTagRelation {
    pub note_id: String,
    pub tag_id: String,
    pub user_id: String,
    pub created_at: i64,
}

impl From<NoteTagRelation> for ServerNoteTagRelation {
    fn from(rel: NoteTagRelation) -> Self {
        ServerNoteTagRelation {
            note_id: rel.note_id,
            tag_id: rel.tag_id,
            user_id: rel.user_id,
            created_at: rel.created_at,
            // is_deleted 和 deleted_at 不发送到服务器（服务器有自己的删除管理）
        }
    }
}

impl From<ServerNoteTagRelation> for NoteTagRelation {
    fn from(rel: ServerNoteTagRelation) -> Self {
        NoteTagRelation {
            note_id: rel.note_id,
            tag_id: rel.tag_id,
            user_id: rel.user_id,
            created_at: rel.created_at,
            is_deleted: false,
            deleted_at: None,
        }
    }
}

/// 同步请求（使用 snake_case 版本的结构体）
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SyncRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<ServerNote>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folders: Option<Vec<ServerFolder>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<ServerTag>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshots: Option<Vec<ServerNoteSnapshot>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_tags: Option<Vec<ServerNoteTagRelation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync_at: Option<i64>,
    /// 冲突解决策略（默认：创建冲突副本）
    #[serde(default)]
    pub conflict_resolution: ConflictStrategy,
    /// 设备ID（用于操作锁）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}

/// 同步响应（使用 snake_case 版本的结构体）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncResponse {
    pub status: String,
    pub server_time: i64,
    pub last_sync_at: i64,

    pub upserted_notes: Vec<ServerNote>,
    pub upserted_folders: Vec<ServerFolder>,
    pub upserted_tags: Vec<ServerTag>,
    pub upserted_snapshots: Vec<ServerNoteSnapshot>,
    pub upserted_note_tags: Vec<ServerNoteTagRelation>,

    #[serde(default)]
    pub deleted_note_ids: Vec<String>,
    #[serde(default)]
    pub deleted_folder_ids: Vec<String>,
    #[serde(default)]
    pub deleted_tag_ids: Vec<String>,

    // 推送统计（服务器确认实际更新的数量）
    pub pushed_notes: usize,
    pub pushed_folders: usize,
    pub pushed_tags: usize,
    pub pushed_snapshots: usize,
    pub pushed_note_tags: usize,
    pub pushed_total: usize,  // 推送总数

    // 拉取统计（服务器端真正的新数据）
    pub pulled_notes: usize,
    pub pulled_folders: usize,
    pub pulled_tags: usize,
    pub pulled_snapshots: usize,
    pub pulled_note_tags: usize,
    pub pulled_total: usize,  // 拉取总数

    #[serde(default)]
    pub conflicts: Vec<ConflictInfo>,
}

/// 笔记标签关联（前端使用，camelCase）
#[derive(Debug, Serialize, Deserialize, Clone)]
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

/// 冲突信息（用于服务器通信，snake_case）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConflictInfo {
    pub id: String,
    pub entity_type: String,
    pub local_version: i32,
    pub server_version: i32,
    pub title: String,
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
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SyncReport {
    pub success: bool,  // 同步是否成功

    // 推送到服务器的详细统计
    pub pushed_notes: usize,
    pub pushed_folders: usize,
    pub pushed_tags: usize,
    pub pushed_snapshots: usize,
    pub pushed_note_tags: usize,

    // 从服务器拉取的详细统计
    pub pulled_notes: usize,
    pub pulled_folders: usize,
    pub pulled_tags: usize,
    pub pulled_snapshots: usize,
    pub pulled_note_tags: usize,

    // 删除的数据统计
    pub deleted_notes: usize,
    pub deleted_folders: usize,
    pub deleted_tags: usize,

    pub conflict_count: usize,  // 冲突数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,  // 错误信息（如果有）

    // 兼容旧版本的汇总字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pushed_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pulled_count: Option<usize>,
}

impl SyncReport {
    /// 获取总推送数量（兼容旧版本）
    pub fn total_pushed(&self) -> usize {
        self.pushed_notes + self.pushed_folders + self.pushed_tags + self.pushed_snapshots + self.pushed_note_tags
    }

    /// 获取总拉取数量（兼容旧版本）
    pub fn total_pulled(&self) -> usize {
        self.pulled_notes + self.pulled_folders + self.pulled_tags + self.pulled_snapshots + self.pulled_note_tags
    }
}

/// 冲突解决策略
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub enum ConflictStrategy {
    #[default]
    KeepBoth,  // 保留两个版本（创建冲突副本，默认）
    KeepServer,  // 保留服务器版本
    KeepLocal,  // 保留本地版本
}
