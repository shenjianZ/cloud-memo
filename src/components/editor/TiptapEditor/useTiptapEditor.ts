import { useEditor } from '@tiptap/react'
import StarterKit from '@tiptap/starter-kit'
import Image from '@tiptap/extension-image'
import Link from '@tiptap/extension-link'
import TaskList from '@tiptap/extension-task-list'
import TaskItem from '@tiptap/extension-task-item'
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight'
import { common, createLowlight } from 'lowlight'
import Placeholder from '@tiptap/extension-placeholder'
import CharacterCount from '@tiptap/extension-character-count'
import Color from '@tiptap/extension-color'
import { TextStyle } from '@tiptap/extension-text-style'
import Typography from '@tiptap/extension-typography'
import { Table } from '@tiptap/extension-table'
import TableRow from '@tiptap/extension-table-row'
import TableHeader from '@tiptap/extension-table-header'
import TableCell from '@tiptap/extension-table-cell'
import type { TiptapContent } from '@/types/note'
import { TableCommands } from './tableExtensions'

// 创建 lowlight 实例并注册常用语言
const lowlight = createLowlight(common)

interface UseTiptapEditorProps {
  content: TiptapContent
  onUpdate?: (json: any, text: string) => void
  editable?: boolean
}

/**
 * Tiptap 编辑器 Hook
 * 配置所有扩展和功能
 */
export function useTiptapEditor({
  content,
  onUpdate,
  editable = true,
}: UseTiptapEditorProps) {
  // 准备初始内容
  const initialContent = (() => {
    if (typeof content === 'string') {
      // 如果是 Markdown 字符串，转换为段落
      return content || '<p></p>'
    }
    // 如果是 Tiptap JSON 对象，直接使用
    return content
  })()

  const editor = useEditor({
    extensions: [
      // 基础工具包 - 包含了大部分常用扩展
      // Document, Paragraph, Text, Bold, Italic, Strike, Code, Heading
      // BulletList, OrderedList, Blockquote, HorizontalRule, HardBreak
      // Dropcursor, Gapcursor, History, Link
      StarterKit.configure({
        // 禁用 StarterKit 中的 codeBlock，使用 CodeBlockLowlight 替代
        codeBlock: false,

        // 禁用 StarterKit 中的 link，使用自定义配置的 Link 扩展
        link: false,

        // 标题配置
        heading: {
          levels: [1, 2, 3, 4, 5, 6],
        },

        // 列表配置 - 移除 Tailwind 类，使用自定义 CSS
        bulletList: {
          keepMarks: true,
          keepAttributes: false,
          HTMLAttributes: {},
        },
        orderedList: {
          keepMarks: true,
          keepAttributes: false,
          HTMLAttributes: {},
        },

        // 区块引用配置
        blockquote: {
          HTMLAttributes: {},
        },

        // 代码块配置（行内代码）
        code: {
          HTMLAttributes: {},
        },

        // 水平分割线
        horizontalRule: {
          HTMLAttributes: {},
        },
      }),

      // 图片扩展
      Image.configure({
        inline: true,
        allowBase64: true,
        HTMLAttributes: {
          class: 'rounded-lg max-w-full h-auto',
        },
      }),

      // 链接扩展（StarterKit 已包含，这里进行自定义配置）
      Link.configure({
        // 禁用默认的点击打开行为，让我们自己处理
        openOnClick: false,
        HTMLAttributes: {
          class: 'text-blue-500 underline hover:text-blue-600 cursor-pointer',
        },
        autolink: true,
        linkOnPaste: true,
      }),

      // 任务列表
      TaskList.configure({
        HTMLAttributes: {},
      }),
      TaskItem.configure({
        nested: true,
        HTMLAttributes: {},
      }),

      // 代码块（带语法高亮）
      CodeBlockLowlight.configure({
        lowlight,
        defaultLanguage: 'plaintext',
        HTMLAttributes: {
          class: 'hljs',
        },
      }),

      // 占位符
      Placeholder.configure({
        placeholder: ({ node }) => {
          if (node.type.name === 'heading') {
            return '标题...'
          }
          return '输入 / 唤起命令菜单，或开始输入...'
        },
        includeChildren: false,
      }),

      // 字符统计
      CharacterCount,

      // 颜色和文本样式
      Color,
      TextStyle,

      // 排版增强（智能引号、破折号等）
      Typography,

      // 表格（使用官方扩展 + 自定义增强命令）
      Table.configure({
        resizable: true,
        allowTableNodeSelection: true,
        HTMLAttributes: {
          class: 'border-collapse table-auto w-full my-4',
        },
      }),
      TableRow.configure({
        HTMLAttributes: {
          class: 'border-b',
        },
      }),
      TableHeader.configure({
        HTMLAttributes: {
          class: 'border border-gray-300 px-4 py-2 bg-gray-50 font-semibold text-left',
        },
      }),
      TableCell.configure({
        HTMLAttributes: {
          class: 'border border-gray-300 px-4 py-2',
        },
      }),

      // 表格增强命令
      TableCommands,
    ],

    content: initialContent,
    editable,

    editorProps: {
      attributes: {
        class: 'max-w-none focus:outline-none min-h-[500px] px-4 py-3',
      },
      handleDrop: (view, event) => {
        // 处理图片拖拽上传
        if (event.dataTransfer && event.dataTransfer.files) {
          const files = Array.from(event.dataTransfer.files)
          const images = files.filter(file => /image/i.test(file.type))

          if (images.length > 0) {
            images.forEach(async (image) => {
              const reader = new FileReader()
              reader.onload = () => {
                const { schema } = view.state
                const coordinates = view.posAtCoords({
                  left: event.clientX,
                  top: event.clientY,
                })

                if (coordinates) {
                  const node = schema.nodes.image.create({
                    src: reader.result,
                  })
                  const transaction = view.state.tr.insert(coordinates.pos, node)
                  view.dispatch(transaction)
                }
              }
              reader.readAsDataURL(image)
            })
            return true
          }
        }
        return false
      },
    },

    onUpdate: ({ editor }) => {
      const json = editor.getJSON()
      const text = editor.getText()
      onUpdate?.(json, text)
    },
  })

  return editor
}
