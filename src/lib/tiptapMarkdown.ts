import type { TiptapContent } from '@/types/note'

/**
 * 将 Tiptap JSON 内容转换为 Markdown
 */
export function tiptapJsonToMarkdown(content: TiptapContent): string {
  // 如果内容是纯文本/Markdown 字符串，直接返回
  if (typeof content === 'string') {
    return content
  }

  try {
    // 将 JSON 转换为 Markdown
    // 注意：这需要一个完整的文档结构
    if (content.type === 'doc' && content.content) {
      // 简单的 JSON 到 Markdown 转换
      return convertNode(content.content)
    }

    return ''
  } catch (error) {
    console.error('Failed to convert Tiptap JSON to Markdown:', error)
    return ''
  }
}

/**
 * 递归转换节点为 Markdown
 */
function convertNode(nodes: any[], depth = 0): string {
  let markdown = ''

  for (const node of nodes) {
    switch (node.type) {
      case 'paragraph':
        markdown += convertInlineContent(node.content) + '\n\n'
        break

      case 'heading':
        const level = node.attrs?.level || 1
        const text = convertInlineContent(node.content)
        markdown += `${'#'.repeat(level)} ${text}\n\n`
        break

      case 'bulletList':
        markdown += convertBulletList(node.content, depth)
        break

      case 'orderedList':
        markdown += convertOrderedList(node.content, node.attrs?.start || 1)
        break

      case 'taskList':
        markdown += convertTaskList(node.content)
        break

      case 'codeBlock':
        const language = node.attrs?.language || ''
        const code = node.content?.map((c: any) => c.text).join('') || ''
        markdown += `\`\`\`${language}\n${code}\n\`\`\`\n\n`
        break

      case 'blockquote':
        const quoteText = convertInlineContent(node.content)
        markdown += quoteText.split('\n').map((line) => `> ${line}`).join('\n') + '\n\n'
        break

      case 'horizontalRule':
        markdown += '---\n\n'
        break

      case 'table':
        markdown += convertTable(node.content)
        break

      case 'hardBreak':
        markdown += '\n'
        break

      default:
        // 递归处理未知节点
        if (node.content) {
          markdown += convertNode(node.content, depth)
        }
        break
    }
  }

  return markdown
}

/**
 * 转换内联内容
 */
function convertInlineContent(content: any[] = []): string {
  let text = ''

  for (const node of content) {
    if (!node) continue

    switch (node.type) {
      case 'text':
        // 处理文本标记（粗体、斜体等）
        if (node.marks) {
          let markedText = node.text
          for (const mark of node.marks) {
            switch (mark.type) {
              case 'bold':
                markedText = `**${markedText}**`
                break
              case 'italic':
                markedText = `*${markedText}*`
                break
              case 'strike':
                markedText = `~~${markedText}~~`
                break
              case 'code':
                markedText = `\`${markedText}\``
                break
              case 'link':
                markedText = `[${markedText}](${mark.attrs?.href || ''})`
                break
            }
          }
          text += markedText
        } else {
          text += node.text
        }
        break

      case 'hardBreak':
        text += '\n'
        break

      default:
        // 递归处理
        if (node.content) {
          text += convertInlineContent(node.content)
        }
        break
    }
  }

  return text.trim()
}

/**
 * 转换无序列表
 */
function convertBulletList(items: any[], depth: number): string {
  let markdown = ''
  const indent = '  '.repeat(depth)

  for (const item of items) {
    if (item.type === 'listItem' || item.type === 'taskItem') {
      const checked = item.attrs?.checked
      const prefix = checked !== undefined ? (checked ? '- [x] ' : '- [ ] ') : '- '
      const content = convertInlineContent(item.content)
      markdown += `${indent}${prefix}${content}\n`

      // 处理嵌套列表
      if (item.content) {
        for (const child of item.content) {
          if (child.type === 'bulletList') {
            markdown += convertBulletList(child.content, depth + 1)
          } else if (child.type === 'orderedList') {
            markdown += convertOrderedList(child.content, 1, depth + 1)
          }
        }
      }
    }
  }

  return markdown
}

/**
 * 转换有序列表
 */
function convertOrderedList(items: any[], start: number = 1, depth: number = 0): string {
  let markdown = ''
  const indent = '  '.repeat(depth)

  items.forEach((item, index) => {
    if (item.type === 'listItem') {
      const content = convertInlineContent(item.content)
      markdown += `${indent}${start + index}. ${content}\n`

      // 处理嵌套列表
      if (item.content) {
        for (const child of item.content) {
          if (child.type === 'bulletList') {
            markdown += convertBulletList(child.content, depth + 1)
          } else if (child.type === 'orderedList') {
            markdown += convertOrderedList(child.content, 1, depth + 1)
          }
        }
      }
    }
  })

  return markdown
}

/**
 * 转换任务列表
 */
function convertTaskList(items: any[]): string {
  return convertBulletList(items, 0)
}

/**
 * 转换表格
 */
function convertTable(rows: any[]): string {
  let markdown = ''

  rows.forEach((row, rowIndex) => {
    if (row.type === 'tableRow') {
      const cells = row.content?.filter((c: any) => c.type === 'tableCell' || c.type === 'tableHeader')
      if (cells) {
        markdown += '| ' + cells.map((cell: any) => {
          return convertInlineContent(cell.content)
        }).join(' | ') + ' |\n'

        // 添加分隔行（在表头后）
        if (rowIndex === 0) {
          markdown += '|' + cells.map(() => '---').join('|') + '|\n'
        }
      }
    }
  })

  return markdown + '\n'
}
