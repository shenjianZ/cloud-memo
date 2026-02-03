/**
 * 笔记元数据
 */
export interface NoteMetadata {
  id: string
  title: string  // 后端数据库中是必填字段，总是有值（至少是 "未命名笔记"）
  author?: string
  createdAt: number
  updatedAt: number
  tags: string[]
  folder?: string
  isPinned: boolean
  isFavorite: boolean
}

/**
 * Tiptap 内容类型
 * 支持 Markdown 字符串（向后兼容）或 Tiptap JSON 对象
 */
export type TiptapContent = string | {
  type: 'doc'
  content?: Array<{
    type: string
    attrs?: Record<string, any>
    content?: any[]
    text?: string
    marks?: any[]
  }>
}

/**
 * 笔记内容
 */
export interface NoteContent {
  markdown: string  // 保留向后兼容
}

/**
 * 完整笔记对象
 * 使用 TiptapContent 作为内容类型
 */
export interface Note extends NoteMetadata {
  content: TiptapContent
  markdownCache?: string  // 用于导出/预览的 Markdown 缓存
}

/**
 * 笔记文件夹
 */
export interface NoteFolder {
  id: string
  name: string
  parentId: string | null
  createdAt: number
}

/**
 * 搜索过滤器
 */
export interface NoteFilter {
  query: string
  tags: string[]
  folder?: string
  favorites: boolean
}
