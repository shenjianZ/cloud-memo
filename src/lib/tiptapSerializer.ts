import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
import TaskList from '@tiptap/extension-task-list'
import TaskItem from '@tiptap/extension-task-item'
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight'
import Image from '@tiptap/extension-image'
import Link from '@tiptap/extension-link'
import { Table } from '@tiptap/extension-table'
import TableRow from '@tiptap/extension-table-row'
import TableHeader from '@tiptap/extension-table-header'
import TableCell from '@tiptap/extension-table-cell'
import { Markdown } from 'tiptap-markdown'
import { MathExtension } from '@aarkue/tiptap-math-extension'
import { createLowlight, common } from 'lowlight'

const lowlight = createLowlight(common)

/**
 * 将 Tiptap JSON 内容转换为 Markdown
 * 用于导出、复制等场景
 */
export function tiptapJsonToMarkdown(content: any): string {
  // 如果是字符串，直接返回
  if (typeof content === 'string') {
    return content
  }

  try {
    // 创建临时编辑器实例用于转换
    const tempEditor = new Editor({
      extensions: [
        StarterKit.configure({
          codeBlock: false,
          link: false,
        }),
        TaskList,
        TaskItem,
        CodeBlockLowlight.configure({
          lowlight,
          defaultLanguage: 'plaintext',
        }),
        Image,
        Link,
        Table,
        TableRow,
        TableHeader,
        TableCell,
        Markdown,
        MathExtension.configure({
          evaluation: false,
          katexOptions: {
            strict: false,
            throwOnError: false,
            displayMode: true,
          },
        }),
      ],
      content,
      editable: false,
    })

    const markdown = (tempEditor.storage as any).markdown.getMarkdown()
    tempEditor.destroy()

    return markdown
  } catch (error) {
    console.error('Failed to convert Tiptap JSON to Markdown:', error)
    return ''
  }
}
