import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import * as syncApi from '@/services/syncApi'

export type SyncStatusType = 'idle' | 'syncing' | 'error' | 'conflict'

interface SyncState {
  status: SyncStatusType
  lastSyncAt: number | null
  pendingCount: number
  conflictCount: number
  lastError: string | null
  isAutoSyncEnabled: boolean

  // Actions
  syncNow: () => Promise<void>
  refreshStatus: () => Promise<void>
  clearError: () => void
  setAutoSync: (enabled: boolean) => void
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
          const report = await syncApi.syncNow()
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
        } catch (error) {
          set({
            status: 'error',
            lastError: error instanceof Error ? error.message : '同步失败',
          })
        }
      },

      refreshStatus: async () => {
        try {
          const status = await syncApi.getSyncStatus()
          set({
            lastSyncAt: status.last_sync_at,
            pendingCount: status.pending_count,
            conflictCount: status.conflict_count,
            lastError: status.last_error,
          })
        } catch (error) {
          console.error('Failed to refresh sync status:', error)
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
