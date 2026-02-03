/**
 * 快捷键动作执行器
 * 负责将快捷键事件转换为实际的动作执行
 */
export class KeybindingActionExecutor {
  /**
   * 执行指定的动作
   * @param actionId 动作 ID
   * @returns 是否成功执行
   */
  async execute(actionId: string): Promise<boolean> {
    switch (actionId) {
      case 'global.newNote':
        return this.executeGlobalNewNote()

      case 'global.openSettings':
        return this.executeGlobalOpenSettings()

      case 'global.toggleSidebar':
        return this.executeGlobalToggleSidebar()

      case 'note.save':
        return this.executeNoteSave()

      case 'note.find':
        return this.executeNoteFind()

      case 'note.closeTab':
        return this.executeNoteCloseTab()

      case 'note.togglePreview':
        return this.executeNoteTogglePreview()

      case 'note.zoomIn':
      case 'note.zoomOut':
      case 'note.zoomReset':
        // 这些由编辑器内部处理
        return false

      default:
        console.warn(`Unknown keybinding action: ${actionId}`)
        return false
    }
  }

  private async executeGlobalNewNote(): Promise<boolean> {
    const event = new CustomEvent('keybinding-note-new')
    window.dispatchEvent(event)
    return true
  }

  private async executeGlobalOpenSettings(): Promise<boolean> {
    const event = new CustomEvent('keybinding-open-settings')
    window.dispatchEvent(event)
    return true
  }

  private async executeGlobalToggleSidebar(): Promise<boolean> {
    const event = new CustomEvent('keybinding-toggle-sidebar')
    window.dispatchEvent(event)
    return true
  }

  private async executeNoteSave(): Promise<boolean> {
    const event = new CustomEvent('keybinding-note-save')
    window.dispatchEvent(event)
    return true
  }

  private async executeNoteFind(): Promise<boolean> {
    const event = new CustomEvent('keybinding-note-find')
    window.dispatchEvent(event)
    return true
  }

  private async executeNoteCloseTab(): Promise<boolean> {
    const event = new CustomEvent('keybinding-note-close-tab')
    window.dispatchEvent(event)
    return true
  }

  private async executeNoteTogglePreview(): Promise<boolean> {
    const event = new CustomEvent('keybinding-note-toggle-preview')
    window.dispatchEvent(event)
    return true
  }
}

// 导出单例实例
export const keybindingActionExecutor = new KeybindingActionExecutor()
