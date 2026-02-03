import { invoke } from '@tauri-apps/api/core'

/**
 * 编辑器设置模型
 */
export interface EditorSettings {
  id: number
  contentFontFamily: string
  contentFontSize: number
  contentFontWeight: number
  contentLineHeight: number
  headingFontFamily: string
  headingFontWeight: number
  codeFontFamily: string
  codeFontSize: number
  updatedAt: number
}

/**
 * 更新编辑器设置请求
 */
export interface UpdateEditorSettingsRequest {
  contentFontFamily?: string
  contentFontSize?: number
  contentFontWeight?: number
  contentLineHeight?: number
  headingFontFamily?: string
  headingFontWeight?: number
  codeFontFamily?: string
  codeFontSize?: number
}

/**
 * 获取编辑器设置
 */
export async function getEditorSettings(): Promise<EditorSettings> {
  return invoke('get_editor_settings')
}

/**
 * 更新编辑器设置
 */
export async function updateEditorSettings(
  req: UpdateEditorSettingsRequest
): Promise<EditorSettings> {
  return invoke('update_editor_settings', { req })
}
