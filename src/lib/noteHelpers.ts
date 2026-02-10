import type { Note } from '@/types/note'

/**
 * 从 Tiptap JSON 或 Markdown 内容中提取标题
 * 优先使用第一个 H1，如果不存在则返回空字符串
 */
export function extractTitleFromContent(content: Note['content']): string {
  if (typeof content === 'string') {
    // Markdown 格式：查找第一个 H1 (# 标题)
    const h1Match = content.match(/^#\s+(.+)$/m)
    return h1Match ? h1Match[1].trim() : ''
  }

  // Tiptap JSON 格式：查找第一个 heading 节点
  if (content?.type === 'doc' && content?.content) {
    for (const node of content.content) {
      if (node.type === 'heading' && node.attrs?.level === 1) {
        // 递归提取文本内容
        if (node.content) {
          return extractTextFromNode(node).trim()
        }
      }
    }
  }

  return ''
}

/**
 * 从 Tiptap 节点中提取文本
 */
function extractTextFromNode(node: any): string {
  if (node.text) return node.text
  if (node.content) {
    return node.content.map(extractTextFromNode).join('')
  }
  return ''
}

/**
 * 更新 Tiptap JSON 或 Markdown 内容中的第一个一级标题
 * 如果不存在一级标题，则在开头插入
 */
export function updateTitleInContent(content: Note['content'], newTitle: string): Note['content'] {
  if (typeof content === 'string') {
    // Markdown 格式
    const h1Regex = /^#\s+(.+)$/m
    if (h1Regex.test(content)) {
      // 替换第一个 H1
      return content.replace(h1Regex, `# ${newTitle}`)
    } else {
      // 在开头插入 H1
      return `# ${newTitle}\n\n${content}`
    }
  }

  // Tiptap JSON 格式
  if (content?.type === 'doc' && content?.content) {
    const newContent = {
      type: content.type,
      content: [...content.content]
    }
    let updated = false

    // 查找并替换第一个一级标题
    for (let i = 0; i < newContent.content.length; i++) {
      const node = newContent.content[i]
      if (node?.type === 'heading' && node.attrs?.level === 1) {
        // 替换标题文本
        newContent.content[i] = {
          type: 'heading',
          attrs: { level: 1 },
          content: [{ type: 'text', text: newTitle }]
        }
        updated = true
        break
      }
    }

    // 如果没有找到一级标题，在开头插入
    if (!updated) {
      newContent.content.unshift({
        type: 'heading',
        attrs: { level: 1 },
        content: [{ type: 'text', text: newTitle }]
      })
    }

    return newContent
  }

  return content
}

/**
 * 获取笔记的显示标题
 * note.title 在后端是必填字段，总是有值（至少是 "未命名笔记"）
 * 如果 title 是默认值且内容中有 H1，则优先使用 H1
 */
export function getNoteTitle(note: Note): string {
  // 如果标题不是 "未命名笔记"，直接使用存储的标题
  if (note.title && note.title !== '未命名笔记') {
    return note.title
  }

  // 如果标题是默认值，尝试从内容中提取第一个 H1
  const contentTitle = extractTitleFromContent(note.content)
  if (contentTitle) {
    return contentTitle
  }

  // 使用存储的标题（至少是 "未命名笔记"）
  return note.title || '未命名笔记'
}

/**
 * 从 Tiptap JSON 或 Markdown 提取纯文本摘要
 */
export function getPlainText(content: Note['content'], maxLength = 100): string {
  if (typeof content === 'string') {
    // Markdown 格式：移除 Markdown 语法
    return content
      .replace(/^#+\s+/gm, '') // 标题
      .replace(/\*\*/g, '') // 粗体
      .replace(/\*/g, '') // 斜体
      .replace(/`/g, '') // 代码
      .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1') // 链接
      .replace(/\n/g, ' ') // 换行
      .trim()
      .slice(0, maxLength)
  }

  // Tiptap JSON 格式：递归提取文本
  if (content?.type === 'doc' && content?.content) {
    return extractTextFromNodes(content.content).slice(0, maxLength)
  }

  return ''
}

function extractTextFromNodes(nodes: any[]): string {
  return nodes
    .map(node => {
      if (node.text) return node.text
      if (node.content) return extractTextFromNodes(node.content)
      return ''
    })
    .join(' ')
    .trim()
}
