pub mod error;
pub mod note;
pub mod folder;
pub mod keybinding;
pub mod editor_settings;
pub mod tag;
// ===== 云端同步相关模型 =====
pub mod sync;
pub mod snapshot;
pub mod auth;
pub mod user_profile;
pub mod app_settings;

#[allow(unused_imports)]
pub use error::{AppError, Result};
pub use note::{Note, CreateNoteRequest, UpdateNoteRequest};
pub use folder::{Folder, CreateFolderRequest, UpdateFolderRequest, MoveFolderRequest, MoveNotesRequest};
pub use keybinding::{KeyCombination, KeybindingPreset, KeybindingsData, get_default_keybindings};
pub use editor_settings::{EditorSettings, UpdateEditorSettingsRequest};
pub use tag::{Tag, CreateTagRequest, UpdateTagRequest, NoteTagRequest};
// ===== 云端同步相关导出 =====
pub use sync::{SyncRequest, SyncResponse, ConflictInfo, SyncStatus, SyncReport, SyncType, NoteTagRelation, ConflictStrategy};
pub use snapshot::{NoteSnapshot, CreateSnapshotRequest, SnapshotListItem};
pub use auth::{LoginRequest, RegisterRequest, AuthResponse, User, AccountWithProfile};
// CreateProfileRequest 是预留功能（用户注册时创建资料）
#[allow(unused_imports)]
pub use user_profile::{UserProfile, CreateProfileRequest, UpdateProfileRequest};
pub use app_settings::{AppSettings, UpdateAppSettings};
