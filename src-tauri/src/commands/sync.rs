use crate::services::{SyncService, SingleSyncService, AutoSyncService};
use crate::models::{SyncReport, SyncStatus};
use tauri::State;

/// Sync service 类型别名
type SyncSvc<'a> = State<'a, SyncService>;
type SingleSyncSvc<'a> = State<'a, SingleSyncService>;
type AutoSyncSvc<'a> = State<'a, AutoSyncService>;

/// 手动触发同步（带互斥机制）
#[tauri::command]
pub async fn sync_now(
    sync_service: SyncSvc<'_>,
    auto_sync: AutoSyncSvc<'_>,
) -> std::result::Result<SyncReport, String> {
    log::info!("[commands/sync.rs::sync_now] 开始手动同步");

    // 标记手动同步开始（自动同步将跳过本次）
    auto_sync.begin_manual_sync().await;

    // 执行同步
    let result = sync_service.full_sync()
        .await
        .map_err(|e| {
            log::error!("[commands/sync.rs::sync_now] 同步失败: {}", e);
            e.to_string()
        });

    // 标记手动同步结束
    auto_sync.end_manual_sync().await;

    result.map(|report| {
        log::info!(
            "[commands/sync.rs::sync_now] 同步成功: pushed_notes={}, pulled_notes={}, pulled_folders={}, pulled_tags={}, conflicts={}",
            report.pushed_notes,
            report.pulled_notes,
            report.pulled_folders,
            report.pulled_tags,
            report.conflict_count
        );
        report
    })
}

/// 获取同步状态
#[tauri::command]
pub async fn get_sync_status(
    service: SyncSvc<'_>,
) -> std::result::Result<SyncStatus, String> {
    log::debug!("[commands/sync.rs::get_sync_status] 获取同步状态");

    service.get_sync_status()
        .map_err(|e| {
            log::error!("[commands/sync.rs::get_sync_status] 获取失败: {}", e);
            e.to_string()
        })
        .map(|status| {
            log::debug!(
                "[commands/sync.rs::get_sync_status] 同步状态: last_sync_at={}",
                status.last_sync_at.map(|t| t.to_string()).unwrap_or_else(|| "None".to_string())
            );
            status
        })
}

/// 同步单个笔记（包含其标签和快照）
#[tauri::command]
pub async fn sync_single_note(
    service: SingleSyncSvc<'_>,
    note_id: String,
) -> std::result::Result<SyncReport, String> {
    log::info!("[commands/sync.rs::sync_single_note] 同步单个笔记: {}", note_id);

    service.sync_single_note(&note_id)
        .await
        .map_err(|e| {
            log::error!("[commands/sync.rs::sync_single_note] 同步失败: {}", e);
            e.to_string()
        })
        .map(|report| {
            log::info!(
                "[commands/sync.rs::sync_single_note] 同步成功: pushed_notes={}, pulled_notes={}, conflicts={}",
                report.pushed_notes,
                report.pulled_notes,
                report.conflict_count
            );
            report
        })
}

/// 同步单个标签
#[tauri::command]
pub async fn sync_single_tag(
    service: SingleSyncSvc<'_>,
    tag_id: String,
) -> std::result::Result<SyncReport, String> {
    log::info!("[commands/sync.rs::sync_single_tag] 同步单个标签: {}", tag_id);

    service.sync_single_tag(&tag_id)
        .await
        .map_err(|e| {
            log::error!("[commands/sync.rs::sync_single_tag] 同步失败: {}", e);
            e.to_string()
        })
        .map(|report| {
            log::info!(
                "[commands/sync.rs::sync_single_tag] 同步成功: pushed_tags={}, pulled_tags={}, conflicts={}",
                report.pushed_tags,
                report.pulled_tags,
                report.conflict_count
            );
            report
        })
}

/// 同步单个快照
#[tauri::command]
pub async fn sync_single_snapshot(
    service: SingleSyncSvc<'_>,
    snapshot_id: String,
) -> std::result::Result<SyncReport, String> {
    log::info!("[commands/sync.rs::sync_single_snapshot] 同步单个快照: {}", snapshot_id);

    service.sync_single_snapshot(&snapshot_id)
        .await
        .map_err(|e| {
            log::error!("[commands/sync.rs::sync_single_snapshot] 同步失败: {}", e);
            e.to_string()
        })
        .map(|report| {
            log::info!(
                "[commands/sync.rs::sync_single_snapshot] 同步成功: pushed_snapshots={}, pulled_snapshots={}, conflicts={}",
                report.pushed_snapshots,
                report.pulled_snapshots,
                report.conflict_count
            );
            report
        })
}

/// 同步单个文件夹及其包含的所有笔记（含标签和快照）
#[tauri::command]
pub async fn sync_single_folder(
    service: SingleSyncSvc<'_>,
    folder_id: String,
) -> std::result::Result<SyncReport, String> {
    log::info!("[commands/sync.rs::sync_single_folder] 同步单个文件夹: {}", folder_id);

    service.sync_single_folder(&folder_id)
        .await
        .map_err(|e| {
            log::error!("[commands/sync.rs::sync_single_folder] 同步失败: {}", e);
            e.to_string()
        })
        .map(|report| {
            log::info!(
                "[commands/sync.rs::sync_single_folder] 同步成功: pushed_notes={}, pushed_folders={}, pulled_notes={}, pulled_folders={}, conflicts={}",
                report.pushed_notes,
                report.pushed_folders,
                report.pulled_notes,
                report.pulled_folders,
                report.conflict_count
            );
            report
        })
}
