use crate::services::SyncService;
use crate::models::{SyncReport, SyncStatus};
use tauri::State;

/// Sync service 类型别名
type SyncSvc<'a> = State<'a, SyncService>;

/// 手动触发同步
#[tauri::command]
pub async fn sync_now(
    service: SyncSvc<'_>,
) -> std::result::Result<SyncReport, String> {
    log::info!("[commands/sync.rs::sync_now] 开始手动同步");

    service.full_sync()
        .await
        .map_err(|e| {
            log::error!("[commands/sync.rs::sync_now] 同步失败: {}", e);
            e.to_string()
        })
        .map(|report| {
            log::info!(
                "[commands/sync.rs::sync_now] 同步成功: pushed={}, pulled={}, conflicts={}",
                report.pushed_count,
                report.pulled_count,
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
