import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import * as workspaceApi from '@/services/workspaceApi'
import type { Workspace, CreateWorkspaceRequest, UpdateWorkspaceRequest } from '@/types/workspace'

interface WorkspaceState {
  workspaces: Workspace[]
  currentWorkspace: Workspace | null
  isLoading: boolean
  error: string | null

  // Actions
  loadWorkspaces: () => Promise<void>
  loadCurrentWorkspace: () => Promise<void>
  createWorkspace: (req: CreateWorkspaceRequest) => Promise<Workspace>
  updateWorkspace: (req: UpdateWorkspaceRequest) => Promise<void>
  deleteWorkspace: (id: string) => Promise<void>
  setDefaultWorkspace: (id: string) => Promise<void>
  switchWorkspace: (id: string) => Promise<void>
  clearError: () => void
  clearWorkspaceState: () => void  // 新增：清空工作空间状态（用于切换账号）
}

export const useWorkspaceStore = create<WorkspaceState>()(
  persist(
    (set, get) => ({
      workspaces: [],
      currentWorkspace: null,
      isLoading: false,
      error: null,

      loadWorkspaces: async () => {
        set({ isLoading: true, error: null })
        try {
          const workspaces = await workspaceApi.listWorkspaces()
          set({ workspaces, isLoading: false })
        } catch (error) {
          const errorMsg = typeof error === 'string' ? error : String(error)
          set({ error: errorMsg, isLoading: false })
          throw error
        }
      },

      loadCurrentWorkspace: async () => {
        set({ isLoading: true, error: null })
        try {
          const workspace = await workspaceApi.getCurrentWorkspace()
          set({ currentWorkspace: workspace, isLoading: false })
        } catch (error) {
          const errorMsg = typeof error === 'string' ? error : String(error)
          set({ error: errorMsg, isLoading: false })
          throw error
        }
      },

      createWorkspace: async (req: CreateWorkspaceRequest) => {
        set({ isLoading: true, error: null })
        try {
          const workspace = await workspaceApi.createWorkspace(req)
          const workspaces = get().workspaces
          set({
            workspaces: [...workspaces, workspace],
            isLoading: false,
          })
          return workspace
        } catch (error) {
          const errorMsg = typeof error === 'string' ? error : String(error)
          set({ error: errorMsg, isLoading: false })
          throw error
        }
      },

      updateWorkspace: async (req: UpdateWorkspaceRequest) => {
        set({ isLoading: true, error: null })
        try {
          const updatedWorkspace = await workspaceApi.updateWorkspace(req)
          const workspaces = get().workspaces.map(ws =>
            ws.id === req.id ? updatedWorkspace : ws
          )
          const currentWorkspace = get().currentWorkspace
          set({
            workspaces,
            currentWorkspace: currentWorkspace?.id === req.id ? updatedWorkspace : currentWorkspace,
            isLoading: false,
          })
        } catch (error) {
          const errorMsg = typeof error === 'string' ? error : String(error)
          set({ error: errorMsg, isLoading: false })
          throw error
        }
      },

      deleteWorkspace: async (id: string) => {
        set({ isLoading: true, error: null })
        try {
          await workspaceApi.deleteWorkspace(id)
          const workspaces = get().workspaces.filter(ws => ws.id !== id)
          const currentWorkspace = get().currentWorkspace
          set({
            workspaces,
            currentWorkspace: currentWorkspace?.id === id ? null : currentWorkspace,
            isLoading: false,
          })
        } catch (error) {
          const errorMsg = typeof error === 'string' ? error : String(error)
          set({ error: errorMsg, isLoading: false })
          throw error
        }
      },

      setDefaultWorkspace: async (id: string) => {
        set({ isLoading: true, error: null })
        try {
          await workspaceApi.setDefaultWorkspace(id)
          const workspaces = get().workspaces.map(ws => ({
            ...ws,
            isDefault: ws.id === id,
          }))
          set({ workspaces, isLoading: false })
        } catch (error) {
          const errorMsg = typeof error === 'string' ? error : String(error)
          set({ error: errorMsg, isLoading: false })
          throw error
        }
      },

      switchWorkspace: async (id: string) => {
        set({ isLoading: true, error: null })
        try {
          await workspaceApi.switchWorkspace(id)
          const workspace = get().workspaces.find(ws => ws.id === id)
          if (workspace) {
            const workspaces = get().workspaces.map(ws => ({
              ...ws,
              isCurrent: ws.id === id,
            }))
            set({
              workspaces,
              currentWorkspace: { ...workspace, isCurrent: true },
              isLoading: false,
            })
          }
        } catch (error) {
          const errorMsg = typeof error === 'string' ? error : String(error)
          set({ error: errorMsg, isLoading: false })
          throw error
        }
      },

      clearError: () => set({ error: null }),

      // 新增：清空工作空间状态（用于切换账号）
      clearWorkspaceState: () => {
        set({ workspaces: [], currentWorkspace: null })
      },
    }),
    {
      name: 'workspace-storage',
      partialize: (state) => ({
        workspaces: state.workspaces,
        currentWorkspace: state.currentWorkspace,
      }),
    }
  )
)
