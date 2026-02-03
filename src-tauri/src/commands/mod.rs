pub mod keybindings;
pub mod notes;
pub mod editor_settings;
pub mod tag;

pub use keybindings::*;
pub use notes::*;
pub use editor_settings::*;
pub use tag::*;

// 兼容性命令（已废弃，保留兼容性）
#[tauri::command]
pub async fn note_generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[tauri::command]
pub async fn folder_generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
