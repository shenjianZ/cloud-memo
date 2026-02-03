import { invoke } from '@tauri-apps/api/core'
import type { Note } from '@/types/note'
import { getNoteTitle } from '@/lib/noteHelpers'

/**
 * 笔记存储服务
 * 提供文件系统操作的抽象层
 */
export class NoteStorageService {
  /**
   * 初始化笔记目录
   */
  async initializeNotesDir(): Promise<void> {
    // 后端会在第一次操作时自动创建目录
    // 这个方法保留用于未来的扩展
  }

  /**
   * 读取笔记文件
   */
  async readNoteFile(noteId: string): Promise<string> {
    // 这个方法实际上通过 noteStore 直接调用后端
    // 保留是为了提供一致的接口
    const notes = await invoke<Note[]>('note_load_all')
    const note = notes.find((n) => n.id === noteId)
    // 将 content 转换为字符串
    if (!note?.content) return ''
    return typeof note.content === 'string' ? note.content : JSON.stringify(note.content)
  }

  /**
   * 写入笔记文件
   */
  async writeNoteFile(_noteId: string, _content: string): Promise<void> {
    // 通过 noteStore 处理
    // 这个方法保留是为了提供一致的接口
  }

  /**
   * 删除笔记文件
   */
  async deleteNoteFile(noteId: string): Promise<void> {
    await invoke('note_delete', { noteId })
  }

  /**
   * 列出所有笔记
   */
  async listAllNotes(): Promise<Note[]> {
    return await invoke('note_load_all')
  }

  /**
   * 搜索笔记
   */
  async searchNotes(query: string): Promise<Note[]> {
    const notes = await this.listAllNotes()
    const queryLower = query.toLowerCase()
    return notes.filter((note) => {
      const title = getNoteTitle(note).toLowerCase()
      return title.includes(queryLower) ||
        (typeof note.content === 'string' && note.content.toLowerCase().includes(queryLower))
    })
  }
}

// 导出单例实例
export const noteStorageService = new NoteStorageService()
