import { create } from 'zustand'
import type { EditorTab, EditorInstance } from '@/types/editor'

interface EditorStore {
  tabs: EditorTab[]
  editorInstances: Map<string, EditorInstance>

  // 标签页操作
  openNote: (noteId: string, title: string) => string
  closeTab: (tabId: string) => void
  setActiveTab: (tabId: string) => void
  updateTabDirty: (tabId: string, isDirty: boolean) => void
  updateTabTitle: (tabId: string, title: string) => void
  clearTabs: () => void

  // 查询
  getActiveTab: () => EditorTab | undefined
  getTabByNoteId: (noteId: string) => EditorTab | undefined

  // 编辑器实例管理
  getEditorInstance: (noteId: string) => EditorInstance | undefined
  setEditorInstance: (noteId: string, instance: EditorInstance) => void
  removeEditorInstance: (noteId: string) => void
}

export const useEditorStore = create<EditorStore>((set, get) => ({
  tabs: [],
  editorInstances: new Map(),

  openNote: (noteId, title) => {
    // 检查是否已经打开了这个笔记
    const existingTab = get().tabs.find((t) => t.noteId === noteId)

    if (existingTab) {
      // 如果已存在，激活它
      get().setActiveTab(existingTab.id)
      return existingTab.id
    }

    // 创建新标签页
    const id = crypto.randomUUID()
    set((state) => ({
      tabs: [
        ...state.tabs.map((t) => ({ ...t, isActive: false })),
        {
          id,
          noteId,
          title,
          isActive: true,
          isDirty: false,
        },
      ],
    }))

    return id
  },

  closeTab: (tabId) => {
    const removedTab = get().tabs.find((t) => t.id === tabId)
    const isActive = removedTab?.isActive

    set((state) => {
      const newTabs = state.tabs.filter((t) => t.id !== tabId)

      // 如果移除的是活动标签页，激活最后一个标签页
      if (isActive && newTabs.length > 0) {
        newTabs[newTabs.length - 1].isActive = true

        // 触发路由更新事件
        const nextTab = newTabs[newTabs.length - 1]
        window.dispatchEvent(
          new CustomEvent('tab-switched', {
            detail: { noteId: nextTab.noteId },
          }),
        )
      }

      return { tabs: newTabs }
    })

    // 清理编辑器实例
    if (removedTab) {
      get().removeEditorInstance(removedTab.noteId)
    }
  },

  setActiveTab: (tabId) => {
    set((state) => ({
      tabs: state.tabs.map((t) => ({
        ...t,
        isActive: t.id === tabId,
      })),
    }))
  },

  updateTabDirty: (tabId, isDirty) => {
    set((state) => ({
      tabs: state.tabs.map((t) =>
        t.id === tabId ? { ...t, isDirty } : t
      ),
    }))
  },

  updateTabTitle: (tabId, title) => {
    set((state) => ({
      tabs: state.tabs.map((t) =>
        t.id === tabId ? { ...t, title } : t
      ),
    }))
  },

  clearTabs: () => {
    set({ tabs: [] })
    // 清理所有编辑器实例
    get().editorInstances.forEach((instance) => {
      if (instance.view) {
        instance.view.destroy()
      }
    })
    set({ editorInstances: new Map() })
  },

  getActiveTab: () => {
    return get().tabs.find((t) => t.isActive)
  },

  getTabByNoteId: (noteId) => {
    return get().tabs.find((t) => t.noteId === noteId)
  },

  getEditorInstance: (noteId) => {
    return get().editorInstances.get(noteId)
  },

  setEditorInstance: (noteId, instance) => {
    set((state) => {
      const newInstances = new Map(state.editorInstances)
      newInstances.set(noteId, instance)
      return { editorInstances: newInstances }
    })
  },

  removeEditorInstance: (noteId) => {
    const instance = get().editorInstances.get(noteId)
    if (instance?.view) {
      instance.view.destroy()
    }

    set((state) => {
      const newInstances = new Map(state.editorInstances)
      newInstances.delete(noteId)
      return { editorInstances: newInstances }
    })
  },
}))
