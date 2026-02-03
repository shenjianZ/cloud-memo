import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { EditorState } from '@/types/editor'

const DEFAULT_EDITOR_CONFIG: EditorState = {
  viewMode: 'edit',
  fontSize: 14,
  lineNumbers: true,
  wordWrap: true,
  spellCheck: false,
  theme: 'one-dark',
}

interface EditorConfigStore {
  config: EditorState

  // 操作
  setConfig: (updates: Partial<EditorState>) => void
  setViewMode: (mode: EditorState['viewMode']) => void
  setFontSize: (size: number) => void
  setTheme: (theme: EditorState['theme']) => void
  resetConfig: () => void

  // 预设配置
  applyPreset: (preset: 'minimal' | 'standard' | 'full') => void
}

export const useEditorConfigStore = create<EditorConfigStore>()(
  persist(
    (set, get) => ({
      config: DEFAULT_EDITOR_CONFIG,

      setConfig: (updates) => {
        set((state) => ({
          config: { ...state.config, ...updates },
        }))
      },

      setViewMode: (mode) => {
        get().setConfig({ viewMode: mode })
      },

      setFontSize: (size) => {
        get().setConfig({ fontSize: size })
      },

      setTheme: (theme) => {
        get().setConfig({ theme })
      },

      resetConfig: () => {
        set({ config: DEFAULT_EDITOR_CONFIG })
      },

      applyPreset: (preset) => {
        switch (preset) {
          case 'minimal':
            set({
              config: {
                ...DEFAULT_EDITOR_CONFIG,
                lineNumbers: false,
                wordWrap: false,
              },
            })
            break
          case 'standard':
            set({ config: DEFAULT_EDITOR_CONFIG })
            break
          case 'full':
            set({
              config: {
                ...DEFAULT_EDITOR_CONFIG,
                lineNumbers: true,
                wordWrap: true,
                spellCheck: true,
              },
            })
            break
        }
      },
    }),
    {
      name: 'markdown-editor-config',
    }
  )
)
