import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import * as syncApi from '@/api/sync'

type SyncStatusType = 'idle' | 'syncing' | 'error' | 'conflict'

interface SyncState {
  status: SyncStatusType
  lastSyncAt: number | null
  pendingCount: number
  conflictCount: number
  lastError: string | null
  isAutoSyncEnabled: boolean

  syncNow: () => Promise<SyncReport>
  clearError: () => void
  setAutoSync: (enabled: boolean) => void
}

export interface SyncReport {
  success: boolean
  pushed_count: number
  pulled_count: number
  conflict_count: number
  error?: string
  duration_ms: number
}

export const useSyncStore = create<SyncState>()(
  persist(
    (set) => ({
      status: 'idle',
      lastSyncAt: null,
      pendingCount: 0,
      conflictCount: 0,
      lastError: null,
      isAutoSyncEnabled: true,

      syncNow: async () => {
        set({ status: 'syncing', lastError: null })

        try {
          // 这里传入空数组，因为实际同步由 Tauri 客户端处理
          // 服务器端主要用于管理界面显示状态
          const report = await syncApi.fullSync([], [])

          if (report.success) {
            set({
              status: 'idle',
              lastSyncAt: Date.now(),
              pendingCount: 0,
              conflictCount: report.conflict_count,
            })
          } else {
            set({
              status: 'error',
              lastError: report.error || '同步失败',
            })
          }

          return report
        } catch (error) {
          const message = error instanceof Error ? error.message : '同步失败'
          set({
            status: 'error',
            lastError: message,
          })
          throw error
        }
      },

      clearError: () => set({ lastError: null, status: 'idle' }),
      setAutoSync: (enabled: boolean) => set({ isAutoSyncEnabled: enabled }),
    }),
    {
      name: 'sync-storage',
      partialize: (state) => ({
        lastSyncAt: state.lastSyncAt,
        isAutoSyncEnabled: state.isAutoSyncEnabled,
      }),
    }
  )
)
