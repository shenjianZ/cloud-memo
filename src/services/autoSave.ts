/**
 * 自动保存服务
 * 使用防抖机制避免频繁保存
 */
export class AutoSaveService {
  private saveInterval: number = 500 // 500ms 防抖
  private pendingSaves: Map<string, { timer: NodeJS.Timeout; content: string }>

  constructor() {
    this.pendingSaves = new Map()
  }

  /**
   * 队列保存操作
   * @param noteId 笔记 ID
   * @param content Markdown 内容
   * @param saveCallback 保存回调函数
   */
  queueSave(
    noteId: string,
    content: string,
    saveCallback: (noteId: string, content: string) => void
  ): void {
    // 清除之前的定时器
    const existing = this.pendingSaves.get(noteId)
    if (existing) {
      clearTimeout(existing.timer)
    }

    // 设置新的定时器
    const timer = setTimeout(() => {
      this.performSave(noteId, content, saveCallback)
    }, this.saveInterval)

    this.pendingSaves.set(noteId, { timer, content })
  }

  /**
   * 立即执行保存操作
   */
  private performSave(
    noteId: string,
    content: string,
    saveCallback: (noteId: string, content: string) => void
  ): void {
    this.pendingSaves.delete(noteId)
    saveCallback(noteId, content)
  }

  /**
   * 立即保存指定笔记（跳过防抖）
   */
  saveNow(
    noteId: string,
    content: string,
    saveCallback: (noteId: string, content: string) => void
  ): void {
    // 清除待处理的保存
    const existing = this.pendingSaves.get(noteId)
    if (existing) {
      clearTimeout(existing.timer)
      this.pendingSaves.delete(noteId)
    }

    // 立即保存
    this.performSave(noteId, content, saveCallback)
  }

  /**
   * 取消待处理的保存操作
   */
  cancelSave(noteId: string): void {
    const existing = this.pendingSaves.get(noteId)
    if (existing) {
      clearTimeout(existing.timer)
      this.pendingSaves.delete(noteId)
    }
  }

  /**
   * 清理所有待处理的保存操作
   */
  cleanup(): void {
    this.pendingSaves.forEach(({ timer }) => {
      clearTimeout(timer)
    })
    this.pendingSaves.clear()
  }
}

// 导出单例实例
export const autoSaveService = new AutoSaveService()
