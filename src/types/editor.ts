import { EditorView } from '@codemirror/view'

/**
 * 编辑器视图模式
 */
export type EditorViewMode = 'edit' | 'preview' | 'split'

/**
 * 编辑器状态配置
 */
export interface EditorState {
  viewMode: EditorViewMode
  fontSize: number
  lineNumbers: boolean
  wordWrap: boolean
  spellCheck: boolean
  theme: 'light' | 'dark' | 'one-dark'
}

/**
 * 编辑器标签页
 */
export interface EditorTab {
  id: string
  noteId: string
  title: string
  isActive: boolean
  isDirty: boolean
}

/**
 * 编辑器实例包装
 */
export interface EditorInstance {
  view: EditorView | null
  containerElement: HTMLElement | null
}
