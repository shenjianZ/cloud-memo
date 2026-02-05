import { invoke } from '@tauri-apps/api/core'

export interface CreateSnapshotRequest {
  note_id: string
  title: string
  content: string
  snapshot_name?: string
}

export interface NoteSnapshot {
  id: string
  note_id: string
  title: string
  content: string
  snapshot_name?: string
  created_at: number
}

export interface SnapshotListItem {
  id: string
  note_id: string
  title: string
  snapshot_name?: string
  created_at: number
  created_at_display: string
}

/**
 * 创建快照
 */
export async function createSnapshot(req: CreateSnapshotRequest): Promise<NoteSnapshot> {
  return await invoke<NoteSnapshot>('create_snapshot', { req })
}

/**
 * 列出笔记的所有快照
 */
export async function listSnapshots(noteId: string): Promise<SnapshotListItem[]> {
  return await invoke<SnapshotListItem[]>('list_snapshots', { noteId })
}

/**
 * 获取单个快照详情
 */
export async function getSnapshot(snapshotId: string): Promise<NoteSnapshot> {
  return await invoke<NoteSnapshot>('get_snapshot', { snapshotId })
}

/**
 * 删除快照
 */
export async function deleteSnapshot(snapshotId: string): Promise<void> {
  return await invoke('delete_snapshot', { snapshotId })
}

/**
 * 从快照恢复（返回快照内容，由前端调用 update_note）
 */
export async function restoreFromSnapshot(snapshotId: string): Promise<NoteSnapshot> {
  return await invoke<NoteSnapshot>('restore_from_snapshot', { snapshotId })
}
