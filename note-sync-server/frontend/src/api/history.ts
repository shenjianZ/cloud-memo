import apiClient from './client'

export interface SyncHistoryEntry {
  id: string
  user_id: string
  sync_type: 'push' | 'pull' | 'full'
  pushed_count: number
  pulled_count: number
  conflict_count: number
  error?: string
  duration_ms: number
  created_at: number
}

/**
 * 获取同步历史记录
 */
export async function getSyncHistory(limit = 50): Promise<SyncHistoryEntry[]> {
  const response = await apiClient.get<SyncHistoryEntry[]>('/sync/history', {
    params: { limit },
  })
  return response.data
}

/**
 * 清空同步历史
 */
export async function clearSyncHistory(): Promise<void> {
  await apiClient.delete('/sync/history')
}
