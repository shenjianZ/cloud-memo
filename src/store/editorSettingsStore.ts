import { create } from 'zustand'
import * as editorSettingsApi from '@/services/editorSettingsApi'

export interface EditorSettingsState {
  settings: editorSettingsApi.EditorSettings | null
  isLoading: boolean
  error: string | null
  _loaded: boolean // 防止重复加载的标志

  // 操作
  loadSettings: () => Promise<void>
  updateSettings: (updates: editorSettingsApi.UpdateEditorSettingsRequest) => Promise<void>
}

export const useEditorSettingsStore = create<EditorSettingsState>((set, get) => ({
  settings: null,
  isLoading: false,
  error: null,
  _loaded: false,

  loadSettings: async () => {
    // 防止重复加载
    if (get()._loaded) {
      console.log('[editorSettingsStore] 设置已加载，跳过重复加载')
      return
    }

    set({ isLoading: true, error: null })
    try {
      const settings = await editorSettingsApi.getEditorSettings()
      set({ settings, isLoading: false, _loaded: true })
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
