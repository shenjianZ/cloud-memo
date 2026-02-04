import { create } from 'zustand'
import * as editorSettingsApi from '@/services/editorSettingsApi'

export interface EditorSettingsState {
  settings: editorSettingsApi.EditorSettings | null
  isLoading: boolean
  error: string | null

  // 操作
  loadSettings: () => Promise<void>
  updateSettings: (updates: editorSettingsApi.UpdateEditorSettingsRequest) => Promise<void>
}

export const useEditorSettingsStore = create<EditorSettingsState>((set) => ({
  settings: null,
  isLoading: false,
  error: null,

  loadSettings: async () => {
    set({ isLoading: true, error: null })
    try {
      const settings = await editorSettingsApi.getEditorSettings()
      set({ settings, isLoading: false })
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : '加载设置失败',
        isLoading: false,
      })
    }
  },

  updateSettings: async (updates) => {
    set({ isLoading: true, error: null })
    try {
      const updatedSettings = await editorSettingsApi.updateEditorSettings(updates)
      set({ settings: updatedSettings, isLoading: false })
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : '更新设置失败',
        isLoading: false,
      })
      throw error
    }
  },
}))
