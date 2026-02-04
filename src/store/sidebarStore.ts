import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface SidebarState {
  isCollapsed: boolean;
  toggleSidebar: () => void;
  setCollapsed: (collapsed: boolean) => void;

  // 文件夹展开状态
  expandedFolders: Set<string>;
  toggleFolderExpanded: (folderId: string) => void;
  setFolderExpanded: (folderId: string, expanded: boolean) => void;
  expandAllFolders: () => void;
  collapseAllFolders: () => void;
}

export const useSidebarStore = create<SidebarState>()(
  persist(
    (set) => ({
      isCollapsed: false,
      expandedFolders: new Set<string>(),

      toggleSidebar: () => set((state) => ({ isCollapsed: !state.isCollapsed })),
      setCollapsed: (collapsed) => set({ isCollapsed: collapsed }),

      toggleFolderExpanded: (folderId) => set((state) => {
        const newExpanded = new Set(state.expandedFolders);
        if (newExpanded.has(folderId)) {
          newExpanded.delete(folderId);
        } else {
          newExpanded.add(folderId);
        }
        return { expandedFolders: newExpanded };
      }),

      setFolderExpanded: (folderId, expanded) => set((state) => {
        const newExpanded = new Set(state.expandedFolders);
        if (expanded) {
          newExpanded.add(folderId);
        } else {
          newExpanded.delete(folderId);
        }
        return { expandedFolders: newExpanded };
      }),

      expandAllFolders: () => set({ expandedFolders: new Set<string>() }),
      collapseAllFolders: () => set({ expandedFolders: new Set() }),
    }),
    {
      name: 'sidebar-storage',
      partialize: (state) => ({
        isCollapsed: state.isCollapsed,
        expandedFolders: Array.from(state.expandedFolders),
      }),
      merge: (persistedState, currentState) => ({
        ...currentState,
        ...(persistedState as SidebarState),
        expandedFolders: new Set((persistedState as SidebarState).expandedFolders as unknown as string[]),
      }),
    }
  )
);
