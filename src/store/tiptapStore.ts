import { create } from 'zustand'
import { devtools } from 'zustand/middleware'

/**
 * 视图模式
 */
export type ViewMode = 'edit' | 'preview' | 'split'

/**
 * Tiptap 编辑器状态管理
 */
export interface TiptapEditorState {
  // 编辑器实例
  editor: any | null

  // 编辑器状态
  isEditable: boolean
  wordCount: number
  characterCount: number

  // 视图模式
  viewMode: ViewMode

  // 菜单状态
  showBubbleMenu: boolean
  showSlashMenu: boolean

  // 操作
  setEditor: (editor: any) => void
  clearEditor: () => void
  setEditable: (editable: boolean) => void
  updateCounts: (wordCount: number, characterCount: number) => void
  setViewMode: (mode: ViewMode) => void
  toggleViewMode: () => void
  toggleBubbleMenu: (show: boolean) => void
  toggleSlashMenu: (show: boolean) => void
}

export const useTiptapStore = create<TiptapEditorState>()(
  devtools(
    (set) => ({
      editor: null,
      isEditable: true,
      wordCount: 0,
      characterCount: 0,
      viewMode: 'edit',
      showBubbleMenu: false,
      showSlashMenu: false,

      setEditor: (editor) => set({ editor }),

      clearEditor: () => set({ editor: null }),

      setEditable: (editable) => set({ isEditable: editable }),

      updateCounts: (wordCount, characterCount) =>
        set({ wordCount, characterCount }),

      setViewMode: (mode) => set({ viewMode: mode }),

      toggleViewMode: () => set((state) => {
        const modes: ViewMode[] = ['edit', 'preview', 'split']
        const currentIndex = modes.indexOf(state.viewMode)
        const nextIndex = (currentIndex + 1) % modes.length
        return { viewMode: modes[nextIndex] }
      }),

      toggleBubbleMenu: (show) => set({ showBubbleMenu: show }),

      toggleSlashMenu: (show) => set({ showSlashMenu: show }),
    }),
    { name: 'TiptapEditorStore' }
  )
)
