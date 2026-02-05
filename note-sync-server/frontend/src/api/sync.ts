import apiClient from './client'

// 笔记和文件夹的类型定义（从后端模型映射）
export interface Note {
  id: string
  user_id: string
  title: string
  content: string
  folder_id?: string
  is_deleted: boolean
  deleted_at?: number
  created_at: number
  updated_at: number
  server_ver: number
}

export interface Folder {
  id: string
  user_id: string
  name: string
  parent_id?: string
  is_deleted: boolean
  deleted_at?: number
  created_at: number
  updated_at: number
  server_ver: number
}

export interface SyncRequest {
  notes: Note[]
  folders: Folder[]
  last_sync_at?: number
}

export interface ConflictInfo {
  id: string
  entity_type: 'note' | 'folder'
  local_version: number
  server_version: number
  title: string
}

export interface SyncResponse {
  notes: Note[]
  folders: Folder[]
  conflicts: ConflictInfo[]
  server_time: number
}

export interface SyncReport {
  success: boolean
  pushed_count: number
  pulled_count: number
  conflict_count: number
  error?: string
  duration_ms: number
}

/**
 * 推送本地更改到服务器
 */
export async function pushSync(req: SyncRequest): Promise<SyncResponse> {
  const response = await apiClient.post<SyncResponse>('/sync/push', req)
  return response.data
}

/**
 * 从服务器拉取更改
 */
export async function pullSync(lastSyncAt?: number): Promise<SyncResponse> {
  const response = await apiClient.post<SyncResponse>('/sync/pull', {
    last_sync_at: lastSyncAt,
  })
  return response.data
}

/**
 * 完整同步（推送 + 拉取）
 */
export async function fullSync(notes: Note[], folders: Folder[]): Promise<SyncReport> {
  const startTime = Date.now()

  try {
    // 1. Push
    const pushRes = await pushSync({ notes, folders })

    // 2. Pull
    const pullRes = await pullSync()

    const duration_ms = Date.now() - startTime

    return {
      success: true,
      pushed_count: notes.length,
      pulled_count: pullRes.notes.length + pullRes.folders.length,
      conflict_count: pushRes.conflicts.length,
      duration_ms,
    }
  } catch (error) {
    const duration_ms = Date.now() - startTime
    return {
      success: false,
      pushed_count: 0,
      pulled_count: 0,
      conflict_count: 0,
      error: error instanceof Error ? error.message : 'Unknown error',
      duration_ms,
    }
  }
}
