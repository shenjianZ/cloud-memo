import { create } from 'zustand'

interface ContextMenuPosition {
  x: number
  y: number
}

interface FolderContextMenuState {
  isVisible: boolean
  position: ContextMenuPosition
  folderId: string | null
}

interface NoteContextMenuState {
  isVisible: boolean
  position: ContextMenuPosition
  noteId: string | null
}

interface ContextMenuStore {
  // 文件夹右键菜单
  folderContextMenu: FolderContextMenuState

  // 笔记右键菜单
  noteContextMenu: NoteContextMenuState

  // 操作方法
  showFolderContextMenu: (position: ContextMenuPosition, folderId: string) => void
  hideFolderContextMenu: () => void
  showNoteContextMenu: (position: ContextMenuPosition, noteId: string) => void
  hideNoteContextMenu: () => void
  hideAll: () => void
}

export const useContextMenuStore = create<ContextMenuStore>((set) => ({
  folderContextMenu: {
    isVisible: false,
    position: { x: 0, y: 0 },
    folderId: null,
  },

  noteContextMenu: {
    isVisible: false,
    position: { x: 0, y: 0 },
    noteId: null,
  },

  showFolderContextMenu: (position, folderId) => set({
    folderContextMenu: { isVisible: true, position, folderId },
    noteContextMenu: { isVisible: false, position: { x: 0, y: 0 }, noteId: null },
  }),

  hideFolderContextMenu: () => set((state) => ({
    folderContextMenu: { ...state.folderContextMenu, isVisible: false },
  })),

  showNoteContextMenu: (position, noteId) => set({
    noteContextMenu: { isVisible: true, position, noteId },
    folderContextMenu: { isVisible: false, position: { x: 0, y: 0 }, folderId: null },
  }),

  hideNoteContextMenu: () => set((state) => ({
    noteContextMenu: { ...state.noteContextMenu, isVisible: false },
  })),

  hideAll: () => set((state) => ({
    folderContextMenu: { ...state.folderContextMenu, isVisible: false },
    noteContextMenu: { ...state.noteContextMenu, isVisible: false },
  })),
}))
