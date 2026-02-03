import {
  Heading1,
  Heading2,
  Heading3,
  Bold,
  Italic,
  Strikethrough,
  Code,
  List,
  ListOrdered,
  ListTodo,
  Quote,
  Undo,
  Redo,
  Code2,
  Minus,
  Table,
  RemoveFormatting,
  Tag as TagIcon,
  MoreVertical,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { cn } from '@/lib/utils'

interface ToolbarProps {
  editor: any
  onToggleTags?: () => void
  onToggleMore?: () => void
}

/**
 * Tiptap 编辑器工具栏
 * 顶部固定工具栏，提供常用格式按钮
 */
export function Toolbar({ editor, onToggleTags, onToggleMore }: ToolbarProps) {
  if (!editor) return null

  const toolbarGroups = [
    // 标题
    {
      buttons: [
        {
          icon: Heading1,
          title: '标题 1',
          action: () => editor.chain().focus().toggleHeading({ level: 1 }).run(),
          isActive: () => editor.isActive('heading', { level: 1 }),
        },
        {
          icon: Heading2,
          title: '标题 2',
          action: () => editor.chain().focus().toggleHeading({ level: 2 }).run(),
          isActive: () => editor.isActive('heading', { level: 2 }),
        },
        {
          icon: Heading3,
          title: '标题 3',
          action: () => editor.chain().focus().toggleHeading({ level: 3 }).run(),
          isActive: () => editor.isActive('heading', { level: 3 }),
        },
      ],
    },
    // 基础格式
    {
      buttons: [
        {
          icon: Bold,
          title: '粗体 (Ctrl+B)',
          action: () => editor.chain().focus().toggleBold().run(),
          isActive: () => editor.isActive('bold'),
        },
        {
          icon: Italic,
          title: '斜体 (Ctrl+I)',
          action: () => editor.chain().focus().toggleItalic().run(),
          isActive: () => editor.isActive('italic'),
        },
        {
          icon: Strikethrough,
          title: '删除线',
          action: () => editor.chain().focus().toggleStrike().run(),
          isActive: () => editor.isActive('strike'),
        },
        {
          icon: Code,
          title: '行内代码',
          action: () => editor.chain().focus().toggleCode().run(),
          isActive: () => editor.isActive('code'),
        },
        {
          icon: RemoveFormatting,
          title: '清除格式',
          action: () => editor.chain().focus().unsetAllMarks().run(),
          isActive: () => false,
        },
      ],
    },
    // 列表和引用
    {
      buttons: [
        {
          icon: List,
          title: '无序列表',
          action: () => editor.chain().focus().toggleBulletList().run(),
          isActive: () => editor.isActive('bulletList'),
        },
        {
          icon: ListOrdered,
          title: '有序列表',
          action: () => editor.chain().focus().toggleOrderedList().run(),
          isActive: () => editor.isActive('orderedList'),
        },
        {
          icon: ListTodo,
          title: '任务列表',
          action: () => editor.chain().focus().toggleTaskList().run(),
          isActive: () => editor.isActive('taskList'),
        },
        {
          icon: Quote,
          title: '引用',
          action: () => editor.chain().focus().toggleBlockquote().run(),
          isActive: () => editor.isActive('blockquote'),
        },
      ],
    },
    // 代码块和分割线
    {
      buttons: [
        {
          icon: Code2,
          title: '代码块',
          action: () => editor.chain().focus().toggleCodeBlock().run(),
          isActive: () => editor.isActive('codeBlock'),
        },
        {
          icon: Minus,
          title: '水平分割线',
          action: () => editor.chain().focus().setHorizontalRule().run(),
          isActive: () => false,
        },
        {
          icon: Table,
          title: '插入表格',
          action: () => editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(),
          isActive: () => editor.isActive('table'),
        },
      ],
    },
    // 操作
    {
      buttons: [
        {
          icon: Undo,
          title: '撤销 (Ctrl+Z)',
          action: () => editor.chain().focus().undo().run(),
          isActive: () => false,
          disabled: () => !editor.can().undo(),
        },
        {
          icon: Redo,
          title: '重做 (Ctrl+Shift+Z)',
          action: () => editor.chain().focus().redo().run(),
          isActive: () => false,
          disabled: () => !editor.can().redo(),
        },
      ],
    },
  ]

  return (
    <div className="border-b border-border bg-card sticky top-0 z-10">
      <div className="flex items-center gap-1 px-3 py-2 flex-wrap">
        {toolbarGroups.map((group, groupIndex) => (
          <div key={groupIndex} className="flex items-center gap-1">
            {group.buttons.map((button, buttonIndex) => (
              <ToolbarButton
                key={buttonIndex}
                icon={button.icon}
                title={button.title}
                isActive={button.isActive()}
                onClick={button.action}
                disabled={'disabled' in button ? (button as any).disabled?.() || false : false}
              />
            ))}
            {groupIndex < toolbarGroups.length - 1 && (
              <Separator orientation="vertical" className="h-6 mx-1" />
            )}
          </div>
        ))}

        {/* 标签按钮 */}
        <Separator orientation="vertical" className="h-6 mx-1" />
        <ToolbarButton
          icon={TagIcon}
          title="标签"
          isActive={false}
          onClick={onToggleTags || (() => {})}
        />

        {/* 更多菜单按钮 */}
        <Separator orientation="vertical" className="h-6 mx-1" />
        <ToolbarButton
          icon={MoreVertical}
          title="更多"
          isActive={false}
          onClick={onToggleMore || (() => {})}
        />
      </div>
    </div>
  )
}

interface ToolbarButtonProps {
  icon: React.ComponentType<{ className?: string }>
  title: string
  isActive: boolean
  onClick: () => void
  disabled?: boolean
}

function ToolbarButton({ icon: Icon, title, isActive, onClick, disabled = false }: ToolbarButtonProps) {
  return (
    <Button
      variant="ghost"
      size="sm"
      onClick={onClick}
      disabled={disabled}
      className={cn(
        "h-8 w-8 p-0",
        isActive && "bg-accent text-accent-foreground",
        disabled && "opacity-50 cursor-not-allowed"
      )}
      title={title}
    >
      <Icon className="w-4 h-4" />
    </Button>
  )
}
