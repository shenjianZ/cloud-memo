mod config;
mod commands;
mod models;
mod database;
mod services;

use tauri::Manager;
use database::init_db_pool;
use database::repositories::{NoteRepository, KeybindingRepository, EditorSettingsRepository, TagRepository};
use services::{NoteService, KeybindingService, EditorSettingsService, TagService};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 初始化数据库
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data dir");

            // 创建目录（如果不存在）
            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data dir");

            let db_path = app_data_dir.join("notes.db");

            log::info!("Initializing database at: {:?}", db_path);

            let pool = init_db_pool(db_path.to_str().unwrap())
                .expect("Failed to initialize database");

            // 初始化服务和仓库
            let note_repo = NoteRepository::new(pool.clone());
            let note_service = NoteService::new(note_repo);

            // 初始化快捷键服务（使用文件存储）
            let keybinding_storage_path = app_data_dir.join("keybindings.json");
            let keybinding_repo = KeybindingRepository::new(keybinding_storage_path);
            let keybinding_service = KeybindingService::new(keybinding_repo);

            // 初始化编辑器设置服务
            let editor_settings_repo = EditorSettingsRepository::new(pool.clone());
            let editor_settings_service = EditorSettingsService::new(editor_settings_repo);

            // 初始化标签服务
            let tag_repo = TagRepository::new(pool.clone());
            let tag_service = TagService::new(tag_repo);

            // 注册服务到 Tauri 状态
            app.manage(note_service);
            app.manage(keybinding_service);
            app.manage(editor_settings_service);
            app.manage(tag_service);

            log::info!("Application services initialized");

            // 开发模式下自动打开开发者工具
            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("main") {
                window.open_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 笔记命令
            commands::create_note,
            commands::get_note,
            commands::update_note,
            commands::delete_note,
            commands::list_notes,
            commands::search_notes,
            // 快捷键命令（优化后的命名）
            commands::load_keybindings,
            commands::save_keybindings,
            commands::import_keybindings,
            commands::reset_keybindings,
            // 编辑器设置命令
            commands::get_editor_settings,
            commands::update_editor_settings,
            // 标签命令
            commands::get_all_tags,
            commands::get_tag,
            commands::get_note_tags,
            commands::create_tag,
            commands::update_tag,
            commands::delete_tag,
            commands::add_tag_to_note,
            commands::remove_tag_from_note,
            commands::set_note_tags,
            // 兼容性命令（已废弃，保留兼容性）
            commands::note_generate_id,
            commands::folder_generate_id,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
