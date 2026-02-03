import { useEffect, useState, useRef } from 'react'
import {
  Bold,
  Italic,
  Strikethrough,
  Code,
  Link as LinkIcon,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { cn } from '@/lib/utils'
import { LinkEditor } from './LinkEditor'

interface BubbleMenuProps {
  editor: any
}

/**
 * Tiptap Bubble Menu
 * 选中文字时显示的浮动菜单，显示在选区的右上角
 */
export function BubbleMenu({ editor }: BubbleMenuProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [position, setPosition] = useState({ x: 0, y: 0 })
  const [showLinkEditor, setShowLinkEditor] = useState(false)
  const [linkEditorPosition, setLinkEditorPosition] = useState({ x: 0, y: 0 })
  const menuRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!editor) return

    const updatePosition = () => {
      const { from, to, empty } = editor.state.selection

      // 只有在选中文字时才显示
      if (empty || from === to) {
        setIsOpen(false)
        return
      }

      try {
        const { view } = editor

        // 获取选区起始和结束的坐标
        const startCoords = view.coordsAtPos(from)
        const endCoords = view.coordsAtPos(to)

        // 计算选区右上角的位置
        // 使用结束位置的右侧作为 X 坐标
        // 使用起始位置的顶部作为 Y 坐标，向上移动更多距离
        const x = endCoords.right + 8
        const y = startCoords.top - 50 // 向上移动更多距离

        setPosition({ x, y })
        setIsOpen(true)
      } catch (error) {
        console.error('Error calculating bubble menu position:', error)
        setIsOpen(false)
      }
    }

    // 监听选择更新
    editor.on('selectionUpdate', updatePosition)
    editor.on('transaction', updatePosition)

    // 失去焦点时隐藏
    const handleBlur = () => {
      // 延迟隐藏，避免点击菜单时立即消失
      setTimeout(() => {
        if (document.activeElement && !menuRef.current?.contains(document.activeElement as Node)) {
          setIsOpen(false)
        }
      }, 100)
    }

    editor.on('blur', handleBlur)

    return () => {
      editor.off('selectionUpdate', updatePosition)
      editor.off('transaction', updatePosition)
      editor.off('blur', handleBlur)
    }
  }, [editor])

  if (!isOpen || !editor) return null

  const menuItems = [
    {
      icon: Bold,
      title: '粗体',
      action: () => {
        editor.chain().focus().toggleBold().run()
      },
      isActive: () => editor.isActive('bold'),
    },
    {
      icon: Italic,
      title: '斜体',
      action: () => {
        editor.chain().focus().toggleItalic().run()
      },
      isActive: () => editor.isActive('italic'),
    },
    {
      icon: Strikethrough,
      title: '删除线',
      action: () => {
        editor.chain().focus().toggleStrike().run()
      },
      isActive: () => editor.isActive('strike'),
    },
    {
      icon: Code,
      title: '行内代码',
      action: () => {
        editor.chain().focus().toggleCode().run()
      },
      isActive: () => editor.isActive('code'),
    },
  ]

  const secondaryItems = [
    {
      icon: LinkIcon,
      title: '链接',
      action: () => {
        // 计算链接编辑器的位置（在 BubbleMenu 下方）
        setLinkEditorPosition({
          x: position.x,
          y: position.y + 50, // 在 BubbleMenu 下方显示
        })
        setShowLinkEditor(true)
      },
      isActive: () => editor.isActive('link'),
    },
  ]

  return (
    <div
      ref={menuRef}
      className="fixed z-50 bg-popover border border-border rounded-lg shadow-lg p-1"
      style={{ left: position.x, top: position.y }}
      onMouseDown={(e) => {
        // 阻止默认行为，防止失去焦点
        e.preventDefault()
      }}
    >
      <div className="flex items-center gap-0.5">
        {menuItems.map((item, index) => (
          <BubbleButton
            key={index}
            icon={item.icon}
            title={item.title}
            isActive={item.isActive()}
            onClick={(e) => {
              e.preventDefault()
              e.stopPropagation()
              item.action()
            }}
          />
        ))}

        <Separator orientation="vertical" className="h-6 mx-1" />

        {secondaryItems.map((item, index) => (
          <BubbleButton
            key={index}
            icon={item.icon}
            title={item.title}
            isActive={item.isActive()}
            onClick={(e) => {
              e.preventDefault()
              e.stopPropagation()
              item.action()
            }}
          />
        ))}
      </div>

      {/* 链接编辑器 */}
      {showLinkEditor && (
        <LinkEditor
          editor={editor}
          position={linkEditorPosition}
          onClose={() => setShowLinkEditor(false)}
        />
      )}
    </div>
  )
}

interface BubbleButtonProps {
  icon: React.ComponentType<{ className?: string }>
  title: string
  isActive: boolean
  onClick: (e: React.MouseEvent) => void
}

function BubbleButton({ icon: Icon, title, isActive, onClick }: BubbleButtonProps) {
  return (
    <Button
      variant="ghost"
      size="sm"
      onClick={onClick}
      className={cn(
        "h-7 w-7 p-0",
        isActive && "bg-accent text-accent-foreground"
      )}
      title={title}
    >
      <Icon className="w-4 h-4" />
    </Button>
  )
}
