import { useState, useEffect, useLayoutEffect, useRef, useCallback } from 'react'
import { Heading1, Heading2, Heading3, List, ListOrdered, CheckSquare, Code, Quote, Minus, Table } from 'lucide-react'
import type { Editor } from '@tiptap/react'

interface SlashCommandMenuProps {
  editor: Editor | null
}

interface Command {
  title: string
  description: string
  icon: any
  command: () => void
  keywords?: string[]
}

/**
 * Tiptap Slash Command Menu
 * 输入 "/" 时显示的命令菜单
 */
export function SlashCommandMenu({ editor }: SlashCommandMenuProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [position, setPosition] = useState({ x: 0, y: 0 })
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)
  const menuRef = useRef<HTMLDivElement>(null)
  const slashPosRef = useRef<number | null>(null)

  const commands: Command[] = [
    // 标题
    {
      title: '标题 1',
      description: '大标题',
      icon: Heading1,
      command: () => editor?.chain().focus().setHeading({ level: 1 }).run(),
      keywords: ['h', 'h1', 'heading', 'heading1', 'title', '大标题', '一级标题'],
    },
    {
      title: '标题 2',
      description: '中标题',
      icon: Heading2,
      command: () => editor?.chain().focus().setHeading({ level: 2 }).run(),
      keywords: ['h2', 'heading', 'heading2', 'title', '中标题', '二级标题'],
    },
    {
      title: '标题 3',
      description: '小标题',
      icon: Heading3,
      command: () => editor?.chain().focus().setHeading({ level: 3 }).run(),
      keywords: ['h3', 'heading', 'heading3', 'title', '小标题', '三级标题'],
    },
    // 列表
    {
      title: '无序列表',
      description: '创建简单的无序列表',
      icon: List,
      command: () => editor?.chain().focus().toggleBulletList().run(),
      keywords: ['ul', 'bullet', 'list', '无序', '圆点', '列表'],
    },
    {
      title: '有序列表',
      description: '创建带编号的列表',
      icon: ListOrdered,
      command: () => editor?.chain().focus().toggleOrderedList().run(),
      keywords: ['ol', 'ordered', 'number', '有序', '数字', '编号', '列表'],
    },
    {
      title: '任务列表',
      description: '创建待办事项',
      icon: CheckSquare,
      command: () => editor?.chain().focus().toggleTaskList().run(),
      keywords: ['todo', 'task', 'check', '待办', '任务', '清单', '复选框'],
    },
    // 代码
    {
      title: '代码块',
      description: '插入代码块',
      icon: Code,
      command: () => editor?.chain().focus().toggleCodeBlock().run(),
      keywords: ['code', 'pre', 'program', '代码', '编程', '代码块'],
    },
    // 其他
    {
      title: '引用',
      description: '创建引用块',
      icon: Quote,
      command: () => editor?.chain().focus().toggleBlockquote().run(),
      keywords: ['quote', 'block', '引用', '引用块', 'blockquote'],
    },
    {
      title: '分割线',
      description: '水平分割线',
      icon: Minus,
      command: () => editor?.chain().focus().setHorizontalRule().run(),
      keywords: ['hr', 'divider', 'rule', '分割', '分割线', '水平线', '分隔线'],
    },
    {
      title: '表格',
      description: '插入表格',
      icon: Table,
      command: () => editor?.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(),
      keywords: ['table', 'grid', '表格', '表', 'matrix'],
    },
  ]

  // 过滤命令
  const filteredCommands = commands.filter((command) => {
    const query = searchQuery.toLowerCase()
    return (
      command.title.toLowerCase().includes(query) ||
      command.description.toLowerCase().includes(query) ||
      command.keywords?.some((keyword) => keyword.toLowerCase().includes(query))
    )
  })

  // 按类别分组过滤后的命令
  const basicBlocks = filteredCommands.filter(cmd =>
    ['标题 1', '标题 2', '标题 3'].includes(cmd.title)
  )
  const listBlocks = filteredCommands.filter(cmd =>
    ['无序列表', '有序列表', '任务列表'].includes(cmd.title)
  )
  const otherBlocks = filteredCommands.filter(cmd =>
    !['标题 1', '标题 2', '标题 3', '无序列表', '有序列表', '任务列表'].includes(cmd.title)
  )

  // 当菜单打开时，重置选中索引到第一项（在浏览器绘制前执行）
  useLayoutEffect(() => {
    if (isOpen && filteredCommands.length > 0) {
      setSelectedIndex(0)
    }
  }, [isOpen])

  // 确保 selectedIndex 不会超出范围（当搜索导致过滤结果变化时）
  useEffect(() => {
    if (isOpen && filteredCommands.length > 0 && selectedIndex >= filteredCommands.length) {
      setSelectedIndex(Math.min(selectedIndex, filteredCommands.length - 1))
    }
  }, [isOpen, filteredCommands.length, selectedIndex])

  // 检查是否应该触发菜单
  const checkSlashTrigger = useCallback(() => {
    if (!editor) return false

    const { state } = editor
    const { selection } = state
    const { $from } = selection

    // 获取当前行的文本
    const textBefore = $from.nodeBefore?.textContent || ''

    // 检查光标前的字符是否是独立的 /
    const lastChar = textBefore.slice(-1)

    // 只有当 / 是独立字符时才触发（不是单词的一部分）
    if (lastChar === '/') {
      slashPosRef.current = selection.from - 1

      // 检查是否在代码块中
      const $pos = state.doc.resolve(selection.from)
      const isInCodeBlock = $pos.parent.type.name === 'codeBlock'

      if (!isInCodeBlock) {
        // 获取坐标
        const coords = editor.view.coordsAtPos(selection.from)
        setPosition({
          x: coords.left,
          y: coords.bottom + 5,
        })
        setSearchQuery('')
        setIsOpen(true)
        return true
      }
    }

    return false
  }, [editor])

  // 监听编辑器变化
  useEffect(() => {
    if (!editor) return

    const handleUpdate = () => {
      if (isOpen) {
        // 如果菜单已打开，更新搜索查询
        const { state } = editor
        const { selection } = state
        const { $from } = selection

        // 获取从 slash 位置到当前位置的文本
        if (slashPosRef.current !== null) {
          const searchText = state.doc.textBetween(slashPosRef.current, selection.from)
          // 去掉开头的 /
          const query = searchText.startsWith('/') ? searchText.slice(1) : searchText
          setSearchQuery(query)
        }

        // 检查是否应该关闭菜单
        const textBefore = $from.nodeBefore?.textContent || ''
        const lastChar = textBefore.slice(-1)

        // 如果删除了 / 或光标移到了其他位置，关闭菜单
        if (lastChar !== '/' || (slashPosRef.current !== null && selection.from < slashPosRef.current)) {
          // 但要确保我们还在有效范围内
          if (selection.from < slashPosRef.current) {
            closeMenu()
          }
        }
      } else {
        // 检查是否应该打开菜单
        checkSlashTrigger()
      }
    }

    editor.on('update', handleUpdate)
    editor.on('selectionUpdate', handleUpdate)

    return () => {
      editor.off('update', handleUpdate)
      editor.off('selectionUpdate', handleUpdate)
    }
  }, [editor, isOpen, checkSlashTrigger])

  // 处理键盘导航
  useEffect(() => {
    if (!isOpen || !editor) return

    const handleKeyDown = (event: KeyboardEvent) => {
      switch (event.key) {
        case 'ArrowDown':
          event.preventDefault()
          setSelectedIndex((prev) => (prev + 1) % filteredCommands.length)
          break
        case 'ArrowUp':
          event.preventDefault()
          setSelectedIndex((prev) => (prev - 1 + filteredCommands.length) % filteredCommands.length)
          break
        case 'Enter':
          event.preventDefault()
          if (filteredCommands[selectedIndex]) {
            executeCommand(filteredCommands[selectedIndex])
          }
          break
        case 'Escape':
          event.preventDefault()
          closeMenu()
          break
      }
    }

    editor.view.dom.addEventListener('keydown', handleKeyDown)

    return () => {
      editor.view.dom.removeEventListener('keydown', handleKeyDown)
    }
  }, [editor, isOpen, selectedIndex, filteredCommands])

  // 点击外部关闭菜单
  useEffect(() => {
    if (!isOpen) return

    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        closeMenu()
      }
    }

    const handleSelectionChange = () => {
      // 如果选择改变，关闭菜单
      if (editor && !editor.state.selection.empty) {
        closeMenu()
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    document.addEventListener('selectionchange', handleSelectionChange)

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      document.removeEventListener('selectionchange', handleSelectionChange)
    }
  }, [isOpen, editor])

  const closeMenu = () => {
    setIsOpen(false)
    setSearchQuery('')
    setSelectedIndex(0)
    slashPosRef.current = null
  }

  const executeCommand = (command: Command) => {
    if (!editor || slashPosRef.current === null) return

    // 删除 / 和搜索文本
    const { state } = editor
    const { selection } = state

    editor.chain().focus().deleteRange({ from: slashPosRef.current, to: selection.from }).run()
    command.command()
    closeMenu()
  }

  // 滚动到选中项
  useEffect(() => {
    if (isOpen && menuRef.current) {
      const selectedElement = menuRef.current.querySelector('[data-selected="true"]') as HTMLElement
      if (selectedElement) {
        selectedElement.scrollIntoView({ block: 'nearest' })
      }
    }
  }, [selectedIndex, isOpen])

  if (!isOpen || !editor) return null

  return (
    <div
      ref={menuRef}
      className="fixed z-50 w-72 bg-popover border border-border rounded-lg shadow-lg max-h-96 overflow-y-auto"
      style={{ left: position.x, top: position.y }}
      onMouseDown={(e) => e.stopPropagation()}
    >
      {filteredCommands.length === 0 ? (
        <div className="p-4 text-sm text-muted-foreground text-center">
          没有找到匹配的命令
        </div>
      ) : (
        <div className="p-2 space-y-1">
          {/* 标题分组 */}
          {basicBlocks.length > 0 && (
            <>
              <div className="text-xs text-muted-foreground px-2 py-1 font-medium">
                标题
              </div>
              {basicBlocks.map((command, index) => (
                <CommandButton
                  key={`heading-${index}`}
                  command={command}
                  index={index}
                  selectedIndex={selectedIndex}
                  onSelect={() => executeCommand(command)}
                  onMouseEnter={() => setSelectedIndex(index)}
                />
              ))}
            </>
          )}

          {/* 列表分组 */}
          {listBlocks.length > 0 && (
            <>
              {basicBlocks.length > 0 && <div className="my-1 border-t border-border" />}
              <div className="text-xs text-muted-foreground px-2 py-1 font-medium">
                列表
              </div>
              {listBlocks.map((command, index) => (
                <CommandButton
                  key={`list-${index}`}
                  command={command}
                  index={basicBlocks.length + index}
                  selectedIndex={selectedIndex}
                  onSelect={() => executeCommand(command)}
                  onMouseEnter={() => setSelectedIndex(basicBlocks.length + index)}
                />
              ))}
            </>
          )}

          {/* 其他分组 */}
          {otherBlocks.length > 0 && (
            <>
              {(basicBlocks.length > 0 || listBlocks.length > 0) && <div className="my-1 border-t border-border" />}
              <div className="text-xs text-muted-foreground px-2 py-1 font-medium">
                其他
              </div>
              {otherBlocks.map((command, index) => (
                <CommandButton
                  key={`other-${index}`}
                  command={command}
                  index={basicBlocks.length + listBlocks.length + index}
                  selectedIndex={selectedIndex}
                  onSelect={() => executeCommand(command)}
                  onMouseEnter={() => setSelectedIndex(basicBlocks.length + listBlocks.length + index)}
                />
              ))}
            </>
          )}

          {/* 提示信息 */}
          <div className="mt-2 pt-2 border-t border-border">
            <div className="text-xs text-muted-foreground px-2">
              使用 <kbd className="px-1 py-0.5 bg-muted rounded text-xs">↑</kbd>
              <kbd className="px-1 py-0.5 bg-muted rounded text-xs">↓</kbd> 导航，
              <kbd className="px-1 py-0.5 bg-muted rounded text-xs">Enter</kbd> 选择
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

interface CommandButtonProps {
  command: Command
  index: number
  selectedIndex: number
  onSelect: () => void
  onMouseEnter: () => void
}

function CommandButton({ command, index, selectedIndex, onSelect, onMouseEnter }: CommandButtonProps) {
  const isSelected = index === selectedIndex

  return (
    <button
      className="w-full flex items-center gap-3 px-2 py-2 rounded-lg text-left transition-colors hover:bg-muted/50"
      style={{
        backgroundColor: isSelected ? 'hsl(var(--muted) / 0.5)' : 'transparent',
      }}
      data-selected={isSelected ? 'true' : undefined}
      onClick={onSelect}
      onMouseEnter={onMouseEnter}
    >
      <div className="w-8 h-8 rounded bg-muted flex items-center justify-center flex-shrink-0">
        <command.icon className="w-4 h-4" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="text-sm font-medium truncate">{command.title}</div>
        <div className="text-xs text-muted-foreground truncate">{command.description}</div>
      </div>
    </button>
  )
}
