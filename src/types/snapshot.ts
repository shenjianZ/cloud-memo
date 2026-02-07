/**
 * 快照相关类型定义
 */

/**
 * 创建快照请求
 */
export interface CreateSnapshotRequest {
  noteId: string;
  title: string;
  content: string;
  snapshotName?: string;
}

/**
 * 笔记快照
 */
export interface NoteSnapshot {
  id: string;
  noteId: string;
  title: string;
  content: string;
  snapshotName?: string;
  createdAt: number;
}

/**
 * 快照列表项
 */
export interface SnapshotListItem {
  id: string;
  noteId: string;
  title: string;
  snapshotName?: string;
  createdAt: number;
  createdAtDisplay: string;
}
