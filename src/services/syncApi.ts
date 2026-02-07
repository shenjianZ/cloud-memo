import { invoke } from '@tauri-apps/api/core'
import type { SyncReport, SyncStatus, SyncOptions } from '@/types/sync'

/**
 * 手动触发同步
 * @param options 同步选项（可选）
 */
export async function syncNow(options?: SyncOptions): Promise<SyncReport> {
  return await invoke<SyncReport>('sync_now', options || {})
}

/**
 * 获取同步状态
 */
export async function getSyncStatus(): Promise<SyncStatus> {
  return await invoke<SyncStatus>('get_sync_status')
}

/**
 * 同步单个笔记（包含其标签和快照）
 */
export async function syncSingleNote(noteId: string): Promise<SyncReport> {
  return await invoke<SyncReport>('sync_single_note', { noteId })
}

/**
 * 同步单个标签
 */
export async function syncSingleTag(tagId: string): Promise<SyncReport> {
  return await invoke<SyncReport>('sync_single_tag', { tagId })
}

/**
 * 同步单个快照
 */
export async function syncSingleSnapshot(snapshotId: string): Promise<SyncReport> {
  return await invoke<SyncReport>('sync_single_snapshot', { snapshotId })
}

/**
 * 同步单个文件夹及其包含的所有笔记（含标签和快照）
 */
export async function syncSingleFolder(folderId: string): Promise<SyncReport> {
  return await invoke<SyncReport>('sync_single_folder', { folderId })
}
