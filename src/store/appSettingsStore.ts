import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import * as appSettingsApi from '@/services/appSettingsApi'

export interface AppSettings {
  id: number
  default_server_url: string
  auto_sync_enabled: boolean
  sync_interval_minutes: number
  theme: 'system' | 'light' | 'dark'
  language: string
  updated_at: number
}

interface AppSettingsStore {
  settings: AppSettings | null
  isLoading: boolean
  error: string | null

  fetchSettings: () => Promise<void>
  updateSettings: (updates: Partial<AppSettings>) => Promise<void>
  resetSettings: () => Promise<AppSettings>
  getDefaultServerUrl: () => Promise<string>
}

export const useAppSettingsStore = create<AppSettingsStore>()(
  persist(
    (set) => ({
      settings: null,
      isLoading: false,
      error: null,

      fetchSettings: async () => {
        set({ isLoading: true, error: null })
        try {
          const settings = await appSettingsApi.getSettings()
          set({ settings, isLoading: false })
        } catch (error: any) {
          console.error('Failed to fetch app settings:', error)
          set({ error: error.message || 'Failed to fetch settings', isLoading: false })
        }
      },

      updateSettings: async (updates) => {
        set({ isLoading: true, error: null })
        try {
          const updated = await appSettingsApi.updateSettings(updates)
          set({ settings: updated, isLoading: false })
        } catch (error: any) {
          console.error('Failed to update app settings:', error)
          set({ error: error.message || 'Failed to update settings', isLoading: false })
          throw error
        }
      },

      resetSettings: async () => {
        set({ isLoading: true, error: null })
        try {
          const settings = await appSettingsApi.resetSettings()
          set({ settings, isLoading: false })
          return settings
        } catch (error: any) {
          console.error('Failed to reset app settings:', error)
          set({ error: error.message || 'Failed to reset settings', isLoading: false })
          throw error
        }
      },

      getDefaultServerUrl: async () => {
        try {
          return await appSettingsApi.getDefaultServerUrl()
        } catch (error: any) {
          console.error('Failed to get default server URL:', error)
          throw error
        }
      },
    }),
    {
      name: 'app-settings-storage',
      partialize: (state) => ({
        settings: state.settings,
      }),
    }
  )
)
