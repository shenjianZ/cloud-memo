mod commands;
mod config;
mod database;
mod models;
mod services;

use database::init_db_pool;
use database::repositories::{
    EditorSettingsRepository, FolderRepository, KeybindingRepository, NoteRepository,
    TagRepository, UserProfileRepository,
};
use services::{AppSettingsService, AuthService, SnapshotService, SyncService, UserProfileService};
use services::{EditorSettingsService, FolderService, KeybindingService, NoteService, TagService};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 准备日志目录
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let app_data_dir = home_dir.join(".notes-data");
    let log_dir = app_data_dir.join("log");

    // 创建日志目录
    std::fs::create_dir_all(&log_dir).expect("Failed to create log directory");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        // 配置日志插件（终端输出 + 文件输出）
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug) // 开发环境使用 debug 级别
                .clear_targets() // 清除默认目标，避免重复
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                )) // 输出到终端
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Folder {
                        path: log_dir,
                        file_name: Some("app".to_string()),
                    },
                )) // 输出到文件
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal) // 使用本地时区
                .build(),
        )
        .setup(|app| {
            // 初始化数据库
            // 使用用户家目录下的 .notes-data 文件夹
            let home_dir = dirs::home_dir().expect("Failed to get home directory");

            let app_data_dir = home_dir.join(".notes-data");

            // 创建目录（如果不存在）
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create .notes-data directory");

            let db_path = app_data_dir.join("notes.db");

            log::info!("Initializing database at: {:?}", db_path);

            let pool =
                init_db_pool(db_path.to_str().unwrap()).expect("Failed to initialize database");

            // 初始化仓库（先创建所有仓库）
            let note_repo = NoteRepository::new(pool.clone());
            let folder_repo = FolderRepository::new(pool.clone());

            // 初始化服务（NoteService 需要 FolderRepository）
            let note_service = NoteService::new(note_repo, folder_repo.clone());
            let folder_service = FolderService::new(folder_repo);

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

            // ===== 初始化云端同步相关服务 =====
            // 同步服务需要直接使用连接池
            let sync_service = SyncService::new(pool.clone());

            // 认证服务
            let auth_service = AuthService::new(pool.clone());

            // 快照服务
            let snapshot_service = SnapshotService::new(pool.clone());

            // 用户资料服务
            let user_profile_repo = UserProfileRepository::new(pool.clone());
            let user_profile_service = UserProfileService::new(user_profile_repo, pool.clone());

            // 应用设置服务
            let app_settings_service = AppSettingsService::new(pool.clone());

            // 注册服务到 Tauri 状态
            app.manage(note_service);
            app.manage(folder_service);
            app.manage(keybinding_service);
            app.manage(editor_settings_service);
            app.manage(tag_service);
            // ===== 云端同步服务 =====
            app.manage(sync_service);
            app.manage(auth_service);
            app.manage(snapshot_service);
            app.manage(user_profile_service);
            app.manage(app_settings_service);

            log::info!("Application services initialized");

            // 开发模式下自动打开开发者工具
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
                // 也为 Auth 窗口打开开发者工具
                if let Some(auth_window) = app.get_webview_window("auth") {
                    auth_window.open_devtools();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 笔记命令
            commands::create_note,
            commands::get_note,
            commands::update_note,
            commands::delete_note,
            commands::restore_note,
            commands::restore_notes,
            commands::list_notes,
            commands::list_deleted_notes,
            commands::search_notes,
            commands::move_notes_to_folder,
            // 文件夹命令
            commands::create_folder,
            commands::get_folder,
            commands::update_folder,
            commands::delete_folder,
            commands::list_folders,
            commands::move_folder,
            commands::get_folder_path,
            // 快捷键命令
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
            // ===== 云端同步命令 =====
            commands::sync_now,
            commands::get_sync_status,
            commands::login,
            commands::register,
            commands::logout,
            commands::get_current_user,
            commands::is_authenticated,
            commands::list_accounts,
            commands::switch_account,
            commands::remove_account,
            commands::refresh_access_token,
            commands::create_snapshot,
            commands::list_snapshots,
            commands::get_snapshot,
            commands::delete_snapshot,
            commands::restore_from_snapshot,
            // 用户资料命令
            commands::get_user_profile,
            commands::update_user_profile,
            commands::sync_profile,
            // 应用设置命令
            commands::get_app_settings,
            commands::update_app_settings,
            commands::reset_app_settings,
            commands::get_default_server_url,
            // 兼容性命令（已废弃，保留兼容性）
            commands::note_generate_id,
            commands::folder_generate_id,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
