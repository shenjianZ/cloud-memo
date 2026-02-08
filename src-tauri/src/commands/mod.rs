pub mod keybindings;
pub mod notes;
pub mod folders;
pub mod editor_settings;
pub mod tag;
// ===== 云端同步相关命令 =====
pub mod sync;
pub mod auth;
pub mod snapshot;
pub mod profile;
pub mod app_settings;
pub mod workspaces;

pub use keybindings::*;
pub use notes::*;
pub use folders::*;
pub use editor_settings::*;
pub use tag::*;
// ===== 云端同步命令导出 =====
pub use sync::*;
pub use auth::*;
pub use snapshot::*;
pub use profile::*;
pub use app_settings::*;
pub use workspaces::*;

// 兼容性命令（已废弃，保留兼容性）
#[tauri::command]
pub async fn note_generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[tauri::command]
pub async fn folder_generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
