import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { tiptapJsonToMarkdown } from './tiptapMarkdown'
import type { TiptapContent, Note } from '@/types/note'
import { toast } from 'sonner'
import { getNoteTitle } from './noteHelpers'

/**
 * 导出笔记为 Markdown 文件
 */
export async function exportToMarkdown(note: Note) {
  try {
    // 将内容转换为 Markdown
    const title = getNoteTitle(note)
    const markdown = `# ${title}\n\n${tiptapJsonToMarkdown(note.content)}`

    // 打开保存对话框
    const filePath = await save({
      filters: [
        {
          name: 'Markdown',
          extensions: ['md'],
        },
      ],
      defaultPath: `${title}.md`,
    })

    if (filePath) {
      await writeTextFile(filePath, markdown)
      toast.success('导出成功', { description: `已导出到 ${filePath}` })
    }
  } catch (error) {
    console.error('Failed to export note:', error)
    toast.error('导出失败', { description: error instanceof Error ? error.message : '未知错误' })
  }
}

/**
 * 导出笔记为 PDF（使用浏览器的打印功能）
 */
export async function exportToPdf(note: Note) {
  try {
    // 创建一个新窗口用于打印
    const printWindow = window.open('', '_blank')
    if (!printWindow) {
      toast.error('无法打开打印窗口')
      return
    }

    // 将内容转换为 HTML
    const html = generatePrintHtml(note)

    // 写入打印窗口
    printWindow.document.write(html)
    printWindow.document.close()

    // 等待内容加载后触发打印
    printWindow.onload = () => {
      setTimeout(() => {
        printWindow.print()
      }, 250)
    }

    toast.success('准备打印', { description: '请在打印对话框中选择"另存为 PDF"' })
  } catch (error) {
    console.error('Failed to export PDF:', error)
    toast.error('导出失败', { description: error instanceof Error ? error.message : '未知错误' })
  }
}

/**
 * 生成用于打印的 HTML
 */
function generatePrintHtml(note: Note) {
  // 将 Tiptap JSON 转换为简单的 HTML
  const bodyHtml = convertTiptapJsonToHtml(note.content)
  const title = getNoteTitle(note)

  return `<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${escapeHtml(title)}</title>
  <style>
    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }

    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
      line-height: 1.6;
      color: #333;
      max-width: 800px;
      margin: 0 auto;
      padding: 40px 20px;
    }

    h1, h2, h3, h4, h5, h6 {
      margin-top: 24px;
      margin-bottom: 16px;
      font-weight: 600;
      line-height: 1.25;
    }

    h1 {
      font-size: 2em;
      border-bottom: 1px solid #eaecef;
      padding-bottom: 0.3em;
    }

    h2 {
      font-size: 1.5em;
      border-bottom: 1px solid #eaecef;
      padding-bottom: 0.3em;
    }

    h3 {
      font-size: 1.25em;
    }

    p {
      margin-bottom: 16px;
    }

    ul, ol {
      padding-left: 2em;
      margin-bottom: 16px;
    }

    li {
      margin-bottom: 4px;
    }

    code {
      background-color: #f6f8fa;
      border-radius: 3px;
      font-size: 85%;
      margin: 0;
      padding: 0.2em 0.4em;
      font-family: 'Courier New', Courier, monospace;
    }

    pre {
      background-color: #f6f8fa;
      border-radius: 3px;
      font-size: 85%;
      line-height: 1.45;
      overflow: auto;
      padding: 16px;
      margin-bottom: 16px;
    }

    pre code {
      background-color: transparent;
      border: 0;
      display: inline;
      line-height: inherit;
      margin: 0;
      max-width: auto;
      overflow: visible;
      padding: 0;
      word-wrap: normal;
    }

    blockquote {
      border-left: 4px solid #dfe2e5;
      padding: 0 16px;
      color: #6a737d;
      margin-bottom: 16px;
    }

    table {
      border-collapse: collapse;
      width: 100%;
      margin-bottom: 16px;
    }

    table th, table td {
      border: 1px solid #dfe2e5;
      padding: 6px 13px;
    }

    table th {
      background-color: #f6f8fa;
      font-weight: 600;
    }

    table tr:nth-child(2n) {
      background-color: #f6f8fa;
    }

    hr {
      border: 0;
      border-top: 1px solid #eaecef;
      height: 0;
      margin: 24px 0;
    }

    img {
      max-width: 100%;
      height: auto;
    }

    a {
      color: #0366d6;
      text-decoration: none;
    }

    a:hover {
      text-decoration: underline;
    }

    @media print {
      body {
        padding: 0;
      }
    }
  </style>
</head>
<body>
  <h1>${escapeHtml(title)}</h1>
  ${bodyHtml}
</body>
</html>`
}

/**
 * 将 Tiptap JSON 转换为 HTML
 */
function convertTiptapJsonToHtml(content: TiptapContent): string {
  if (typeof content === 'string') {
    return `<p>${escapeHtml(content)}</p>`
  }

  if (content.type === 'doc' && content.content) {
    return convertNodesToHtml(content.content)
  }

  return ''
}

/**
 * 递归转换节点为 HTML
 */
function convertNodesToHtml(nodes: any[]): string {
  let html = ''

  for (const node of nodes) {
    if (!node) continue

    switch (node.type) {
      case 'paragraph':
        html += `<p>${convertInlineNodesToHtml(node.content || [])}</p>\n`
        break

      case 'heading':
        const level = node.attrs?.level || 1
        html += `<h${level}>${convertInlineNodesToHtml(node.content || [])}</h${level}>\n`
        break

      case 'bulletList':
        html += '<ul>\n'
        if (node.content) {
          for (const item of node.content) {
            if (item.type === 'listItem') {
              html += `<li>${convertInlineNodesToHtml(item.content || [])}</li>\n`
            }
          }
        }
        html += '</ul>\n'
        break

      case 'orderedList':
        html += '<ol>\n'
        if (node.content) {
          for (const item of node.content) {
            if (item.type === 'listItem') {
              html += `<li>${convertInlineNodesToHtml(item.content || [])}</li>\n`
            }
          }
        }
        html += '</ol>\n'
        break

      case 'taskList':
        html += '<ul>\n'
        if (node.content) {
          for (const item of node.content) {
            if (item.type === 'taskItem') {
              const checked = item.attrs?.checked ? ' checked' : ''
              html += `<li><input type="checkbox"${checked} disabled> ${convertInlineNodesToHtml(item.content || [])}</li>\n`
            }
          }
        }
        html += '</ul>\n'
        break

      case 'codeBlock':
        const language = node.attrs?.language || ''
        const code = node.content?.map((c: any) => escapeHtml(c.text || '')).join('') || ''
        html += `<pre><code class="language-${language}">${code}</code></pre>\n`
        break

      case 'blockquote':
        html += `<blockquote>${convertInlineNodesToHtml(node.content || [])}</blockquote>\n`
        break

      case 'horizontalRule':
        html += '<hr>\n'
        break

      case 'table':
        html += '<table>\n'
        if (node.content) {
          for (const row of node.content) {
            if (row.type === 'tableRow') {
              html += '<tr>\n'
              if (row.content) {
                for (const cell of row.content) {
                  if (cell.type === 'tableHeader') {
                    html += `<th>${convertInlineNodesToHtml(cell.content || [])}</th>\n`
                  } else if (cell.type === 'tableCell') {
                    html += `<td>${convertInlineNodesToHtml(cell.content || [])}</td>\n`
                  }
                }
              }
              html += '</tr>\n'
            }
          }
        }
        html += '</table>\n'
        break

      case 'hardBreak':
        html += '<br>\n'
        break

      case 'image':
        const src = node.attrs?.src || ''
        const alt = node.attrs?.alt || ''
        html += `<img src="${src}" alt="${alt}">\n`
        break

      default:
        if (node.content) {
          html += convertNodesToHtml(node.content)
        }
        break
    }
  }

  return html
}

/**
 * 转换内联节点为 HTML
 */
function convertInlineNodesToHtml(nodes: any[]): string {
  let html = ''

  for (const node of nodes) {
    if (!node) continue

    switch (node.type) {
      case 'text':
        let text = escapeHtml(node.text || '')
        if (node.marks) {
          for (const mark of node.marks) {
            switch (mark.type) {
              case 'bold':
                text = `<strong>${text}</strong>`
                break
              case 'italic':
                text = `<em>${text}</em>`
                break
              case 'strike':
                text = `<s>${text}</s>`
                break
              case 'code':
                text = `<code>${text}</code>`
                break
              case 'link':
                const href = mark.attrs?.href || ''
                text = `<a href="${href}">${text}</a>`
                break
            }
          }
        }
        html += text
        break

      case 'hardBreak':
        html += '<br>'
        break

      case 'mention':
        html += `<span class="mention">${escapeHtml(node.attrs?.id || '')}</span>`
        break

      default:
        if (node.content) {
          html += convertInlineNodesToHtml(node.content)
        }
        break
    }
  }

  return html
}

/**
 * HTML 转义
 */
function escapeHtml(text: string): string {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}
