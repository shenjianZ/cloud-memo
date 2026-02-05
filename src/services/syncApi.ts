import { invoke } from '@tauri-apps/api/core'

export interface SyncReport {
  success: boolean
  pushed_count: number
  pulled_count: number
  conflict_count: number
  error?: string
}

export interface SyncStatus {
  last_sync_at: number | null
  pending_count: number
  conflict_count: number
  last_error: string | null
}

/**
 * 手动触发同步
 */
export async function syncNow(): Promise<SyncReport> {
  return await invoke<SyncReport>('sync_now')
}

/**
 * 获取同步状态
 */
export async function getSyncStatus(): Promise<SyncStatus> {
  return await invoke<SyncStatus>('get_sync_status')
}
