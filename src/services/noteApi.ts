import { invoke } from '@tauri-apps/api/core';

/**
 * 笔记模型
 */
export interface Note {
  id: string;
  title: string;  // 后端数据库中是必填字段
  content: string;  // Tiptap JSON 或 Markdown 字符串
  excerpt?: string;
  markdownCache?: string;  // Markdown 缓存（用于导出/兼容）
  folderId?: string;
  isFavorite: boolean;
  isDeleted: boolean;
  isPinned: boolean;
  author?: string;
  createdAt: number;
  updatedAt: number;
  deletedAt?: number;
  wordCount: number;
  readTimeMinutes: number;
}

/**
 * 创建笔记请求
 */
export interface CreateNoteRequest {
  title: string;  // 后端要求必填
  content: string;
  folderId?: string;
}

/**
 * 更新笔记请求
 */
export interface UpdateNoteRequest {
  id: string;
  title?: string;
  content?: string;
  folderId?: string;
  isFavorite?: boolean;
  isPinned?: boolean;
  author?: string;
}

/**
 * 创建笔记
 */
export async function createNote(req: CreateNoteRequest): Promise<Note> {
  return invoke('create_note', { req });
}

/**
 * 获取单个笔记
 */
export async function getNote(id: string): Promise<Note> {
  return invoke('get_note', { id });
}

/**
 * 更新笔记
 */
export async function updateNote(req: UpdateNoteRequest): Promise<Note> {
  return invoke('update_note', { req });
}

/**
 * 删除笔记（软删除）
 */
export async function deleteNote(id: string): Promise<void> {
  return invoke('delete_note', { id });
}

/**
 * 恢复已删除的笔记到"已恢复笔记"文件夹
 *
 * @param id - 笔记 ID
 * @returns 恢复后的笔记对象
 *
 * @example
 * ```typescript
 * const note = await restoreNote('note-id');
 * console.log('笔记已恢复到:', note.folderId);
 * ```
 */
export async function restoreNote(id: string): Promise<Note> {
  return invoke('restore_note', { id });
}

/**
 * 批量恢复已删除的笔记到"已恢复笔记"文件夹
 *
 * @param noteIds - 笔记 ID 列表
 * @returns 成功恢复的笔记列表
 *
 * @example
 * ```typescript
 * const restored = await restoreNotes(['note-1', 'note-2', 'note-3']);
 * console.log(`成功恢复 ${restored.length} 个笔记`);
 * ```
 */
export async function restoreNotes(noteIds: string[]): Promise<Note[]> {
  return invoke('restore_notes', { noteIds });
}

/**
 * 获取所有笔记
 */
export async function listNotes(): Promise<Note[]> {
  return invoke('list_notes');
}

/**
 * 获取所有已删除的笔记（回收站）
 *
 * @returns 已删除的笔记列表，按删除时间倒序排列
 *
 * @example
 * ```typescript
 * const deletedNotes = await listDeletedNotes();
 * console.log(`回收站中有 ${deletedNotes.length} 篇笔记`);
 * ```
 */
export async function listDeletedNotes(): Promise<Note[]> {
  return invoke('list_deleted_notes');
}

/**
 * 搜索笔记
 */
export async function searchNotes(query: string): Promise<Note[]> {
  return invoke('search_notes', { query });
}
