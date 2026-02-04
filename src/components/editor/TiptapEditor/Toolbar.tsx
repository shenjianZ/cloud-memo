import { useState, useEffect } from 'react'
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
  Plus,
  Trash2,
  CornerDownLeft,
  CornerDownRight,
  CornerUpLeft,
  CornerUpRight,
  Copy,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { cn } from '@/lib/utils'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

interface DropdownButton {
  icon: React.ComponentType<{ className?: string }>
  title: string
  type: 'dropdown'
  isActive: () => boolean
  dropdownContent: React.ReactNode
}

interface ActionButton {
  icon: React.ComponentType<{ className?: string }>
  title: string
  type?: 'action'
  action: () => void
  isActive: () => boolean
  disabled?: () => boolean
}

type ToolbarButton = ActionButton | DropdownButton

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

  // 跟踪光标是否在表格内
  const [isInTable, setIsInTable] = useState(false)

  // 监听编辑器的选择变化，实时更新状态
  useEffect(() => {
    const updateTableStatus = () => {
      setIsInTable(editor.isActive('table'))
    }

    // 初始化状态
    updateTableStatus()

    // 监听选择更新和事务变化
    editor.on('selectionUpdate', updateTableStatus)
    editor.on('transaction', updateTableStatus)

    return () => {
      editor.off('selectionUpdate', updateTableStatus)
      editor.off('transaction', updateTableStatus)
    }
  }, [editor])

  const toolbarGroups: { buttons: ToolbarButton[] }[] = [
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
          title: '表格',
          type: 'dropdown',
          isActive: () => isInTable,
          dropdownContent: (
            <>
              <DropdownMenuItem onClick={() => editor.chain().focus().insertTable({ rows: 2, cols: 2, withHeaderRow: true }).run()}>
                <Plus className="w-4 h-4 mr-2" />
                插入表格 (2x2)
              </DropdownMenuItem>
              {isInTable && (
                <>
                  <div className="border-t border-border/50 my-1" />
                  <DropdownMenuItem onClick={() => editor.chain().focus().addRowBefore().run()}>
                    <CornerUpLeft className="w-4 h-4 mr-2" />
                    在上方添加行
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => editor.chain().focus().addRowAfter().run()}>
                    <CornerUpRight className="w-4 h-4 mr-2" />
                    在下方添加行
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => editor.chain().focus().duplicateRow().run()}>
                    <Copy className="w-4 h-4 mr-2" />
                    复制当前行
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => editor.chain().focus().deleteRow().run()}>
                    <Trash2 className="w-4 h-4 mr-2" />
                    删除当前行
                  </DropdownMenuItem>
                  <div className="border-t border-border/50 my-1" />
                  <DropdownMenuItem onClick={() => editor.chain().focus().addColumnBefore().run()}>
                    <CornerDownLeft className="w-4 h-4 mr-2" />
                    在左侧添加列
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => editor.chain().focus().addColumnAfter().run()}>
                    <CornerDownRight className="w-4 h-4 mr-2" />
                    在右侧添加列
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => editor.chain().focus().duplicateColumn().run()}>
                    <Copy className="w-4 h-4 mr-2" />
                    复制当前列
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => editor.chain().focus().deleteColumn().run()}>
                    <Trash2 className="w-4 h-4 mr-2" />
                    删除当前列
                  </DropdownMenuItem>
                  <div className="border-t border-border/50 my-1" />
                  <DropdownMenuItem onClick={() => editor.chain().focus().toggleHeaderColumn().run()}>
                    切换表头列
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => editor.chain().focus().toggleHeaderRow().run()}>
                    切换表头行
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => editor.chain().focus().toggleCellMerge().run()}>
                    合并/拆分单元格
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => editor.chain().focus().deleteTable().run()}>
                    <Trash2 className="w-4 h-4 mr-2" />
                    删除表格
                  </DropdownMenuItem>
                </>
              )}
            </>
          ),
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
            {group.buttons.map((button, buttonIndex) => {
              if (button.type === 'dropdown') {
                return (
                  <DropdownMenu key={buttonIndex}>
                    <DropdownMenuTrigger asChild>
                      <Button
                        variant="ghost"
                        size="sm"
                        className={cn(
                          "h-8 w-8 p-0",
                          button.isActive() && "bg-accent text-accent-foreground"
                        )}
                        title={button.title}
                      >
                        <button.icon className="w-4 h-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      {button.dropdownContent}
                    </DropdownMenuContent>
                  </DropdownMenu>
                )
              }

              return (
                <ToolbarButton
                  key={buttonIndex}
                  icon={button.icon}
                  title={button.title}
                  isActive={button.isActive()}
                  onClick={button.action}
                  disabled={button.disabled?.() || false}
                />
              )
            })}
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
