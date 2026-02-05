use crate::services::SnapshotService;
use crate::models::{NoteSnapshot, CreateSnapshotRequest, SnapshotListItem};
use tauri::State;

/// Snapshot service 类型别名
type SnapshotSvc<'a> = State<'a, SnapshotService>;

/// 创建快照
#[tauri::command]
pub async fn create_snapshot(
    req: CreateSnapshotRequest,
    service: SnapshotSvc<'_>,
) -> std::result::Result<NoteSnapshot, String> {
    let note_id = req.note_id.clone();
    log::info!("[commands/snapshot.rs::create_snapshot] 创建快照: note_id={}", note_id);

    service.create_snapshot(req)
        .map_err(|e| {
            log::error!("[commands/snapshot.rs::create_snapshot] 创建失败: {}", e);
            e.to_string()
        })
        .map(|snapshot| {
            log::info!("[commands/snapshot.rs::create_snapshot] 创建成功: id={}, note_id={}", snapshot.id, snapshot.note_id);
            snapshot
        })
}

/// 列出笔记的所有快照
#[tauri::command]
pub async fn list_snapshots(
    note_id: String,
    service: SnapshotSvc<'_>,
) -> std::result::Result<Vec<SnapshotListItem>, String> {
    log::debug!("[commands/snapshot.rs::list_snapshots] 列出快照: note_id={}", note_id);

    service.list_snapshots(&note_id)
        .map_err(|e| {
            log::error!("[commands/snapshot.rs::list_snapshots] 列出失败: note_id={}, error={}", note_id, e);
            e.to_string()
        })
        .map(|snapshots| {
            log::debug!("[commands/snapshot.rs::list_snapshots] 列出成功: note_id={}, count={}", note_id, snapshots.len());
            snapshots
        })
}

/// 获取单个快照详情
#[tauri::command]
pub async fn get_snapshot(
    snapshot_id: String,
    service: SnapshotSvc<'_>,
) -> std::result::Result<NoteSnapshot, String> {
    log::debug!("[commands/snapshot.rs::get_snapshot] 获取快照: snapshot_id={}", snapshot_id);

    service.get_snapshot(&snapshot_id)
        .map_err(|e| {
            log::error!("[commands/snapshot.rs::get_snapshot] 获取失败: snapshot_id={}, error={}", snapshot_id, e);
            e.to_string()
        })
}

/// 删除快照
#[tauri::command]
pub async fn delete_snapshot(
    snapshot_id: String,
    service: SnapshotSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/snapshot.rs::delete_snapshot] 删除快照: snapshot_id={}", snapshot_id);

    service.delete_snapshot(&snapshot_id)
        .map_err(|e| {
            log::error!("[commands/snapshot.rs::delete_snapshot] 删除失败: snapshot_id={}, error={}", snapshot_id, e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/snapshot.rs::delete_snapshot] 删除成功: snapshot_id={}", snapshot_id);
        })
}

/// 从快照恢复（返回快照内容，由前端调用 update_note）
#[tauri::command]
pub async fn restore_from_snapshot(
    snapshot_id: String,
    service: SnapshotSvc<'_>,
) -> std::result::Result<NoteSnapshot, String> {
    log::info!("[commands/snapshot.rs::restore_from_snapshot] 从快照恢复: snapshot_id={}", snapshot_id);

    service.restore_from_snapshot(&snapshot_id)
        .map_err(|e| {
            log::error!("[commands/snapshot.rs::restore_from_snapshot] 恢复失败: snapshot_id={}, error={}", snapshot_id, e);
            e.to_string()
        })
        .map(|snapshot| {
            log::info!("[commands/snapshot.rs::restore_from_snapshot] 恢复成功: snapshot_id={}, note_id={}", snapshot_id, snapshot.note_id);
            snapshot
        })
}
