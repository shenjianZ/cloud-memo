import { invoke } from '@tauri-apps/api/core'

/**
 * 标签模型
 */
export interface Tag {
  id: string
  name: string
  color: string | null
  createdAt: number
  updatedAt: number
}

/**
 * 创建标签请求
 */
export interface CreateTagRequest {
  name: string
  color?: string
}

/**
 * 更新标签请求
 */
export interface UpdateTagRequest {
  name?: string
  color?: string
}

/**
 * 笔记标签关联请求
 */
export interface NoteTagRequest {
  noteId: string
  tagId: string
}

/**
 * 获取所有标签
 */
export async function getAllTags(): Promise<Tag[]> {
  return invoke('get_all_tags')
}

/**
 * 根据 ID 获取标签
 */
export async function getTag(id: string): Promise<Tag> {
  return invoke('get_tag', { id })
}

/**
 * 获取笔记的所有标签
 */
export async function getNoteTags(noteId: string): Promise<Tag[]> {
  return invoke('get_note_tags', { noteId })
}

/**
 * 创建标签
 */
export async function createTag(req: CreateTagRequest): Promise<Tag> {
  return invoke('create_tag', { req })
}

/**
 * 更新标签
 */
export async function updateTag(id: string, req: UpdateTagRequest): Promise<Tag> {
  return invoke('update_tag', { id, req })
}

/**
 * 删除标签
 */
export async function deleteTag(id: string): Promise<void> {
  return invoke('delete_tag', { id })
}

/**
 * 为笔记添加标签
 */
export async function addTagToNote(req: NoteTagRequest): Promise<void> {
  return invoke('add_tag_to_note', { req })
}

/**
 * 从笔记移除标签
 */
export async function removeTagFromNote(noteId: string, tagId: string): Promise<void> {
  return invoke('remove_tag_from_note', { noteId, tagId })
}

/**
 * 设置笔记的标签（替换所有标签）
 */
export async function setNoteTags(noteId: string, tagIds: string[]): Promise<void> {
  return invoke('set_note_tags', { noteId, tagIds })
}
