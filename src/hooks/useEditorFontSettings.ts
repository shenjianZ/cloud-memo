import { useEffect } from 'react'
import { useEditorSettingsStore } from '@/store/editorSettingsStore'

/**
 * 应用编辑器字体设置
 * 将编辑器设置应用到全局 CSS 变量
 */
export function useEditorFontSettings() {
  const { settings } = useEditorSettingsStore()

  useEffect(() => {
    if (!settings) return

    // 创建或更新 CSS 样式
    let styleElement = document.getElementById('editor-font-settings') as HTMLStyleElement

    if (!styleElement) {
      styleElement = document.createElement('style')
      styleElement.id = 'editor-font-settings'
      document.head.appendChild(styleElement)
    }

    // 生成 CSS 规则
    const cssRules = `
      /* Tiptap 编辑器字体设置 */
      .ProseMirror {
        font-family: ${settings.contentFontFamily};
        font-size: ${settings.contentFontSize}px;
        font-weight: ${settings.contentFontWeight};
        line-height: ${settings.contentLineHeight};
      }

      /* 标题字体 */
      .ProseMirror h1,
      .ProseMirror h2,
      .ProseMirror h3,
      .ProseMirror h4,
      .ProseMirror h5,
      .ProseMirror h6 {
        font-family: ${settings.headingFontFamily};
        font-weight: ${settings.headingFontWeight};
      }

      /* 代码块字体 */
      .ProseMirror pre {
        font-family: ${settings.codeFontFamily};
        font-size: ${settings.codeFontSize}px;
      }

      /* 行内代码字体 */
      .ProseMirror :not(pre) > code {
        font-family: ${settings.codeFontFamily};
      }
    `

    styleElement.textContent = cssRules
  }, [settings])
}

/**
 * 初始化编辑器设置
 * 在应用启动时加载设置
 */
export function useInitEditorSettings() {
  const loadSettings = useEditorSettingsStore((state) => state.loadSettings)

  useEffect(() => {
    loadSettings()
  }, [loadSettings])
}
