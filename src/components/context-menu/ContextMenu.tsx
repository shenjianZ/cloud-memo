import { useRef, useEffect, ReactNode } from 'react'
import { cn } from '@/lib/utils'

interface ContextMenuProps {
  position: { x: number; y: number }
  isVisible: boolean
  onClose: () => void
  children: ReactNode
}

/**
 * 通用右键菜单组件
 * 自动处理位置计算、边界检测、点击外部关闭
 */
export function ContextMenu({ position, isVisible, onClose, children }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null)

  // 点击外部关闭菜单
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose()
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [onClose])

  // ESC 键关闭菜单
  useEffect(() => {
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose()
      }
    }
    document.addEventListener('keydown', handleEscape)
    return () => document.removeEventListener('keydown', handleEscape)
  }, [onClose])

  // 调整位置避免超出视口
  useEffect(() => {
    if (menuRef.current && isVisible) {
      const rect = menuRef.current.getBoundingClientRect()
      const viewportWidth = window.innerWidth
      const viewportHeight = window.innerHeight

      let adjustedX = position.x
      let adjustedY = position.y

      // 右边界检测
      if (position.x + rect.width > viewportWidth) {
        adjustedX = viewportWidth - rect.width - 8
      }

      // 下边界检测
      if (position.y + rect.height > viewportHeight) {
        adjustedY = viewportHeight - rect.height - 8
      }

      // 上边界检测
      if (adjustedY < 8) {
        adjustedY = 8
      }

      // 左边界检测
      if (adjustedX < 8) {
        adjustedX = 8
      }

      menuRef.current.style.left = `${adjustedX}px`
      menuRef.current.style.top = `${adjustedY}px`
    }
  }, [position.x, position.y, isVisible])

  if (!isVisible) return null

  return (
    <div
      ref={menuRef}
      className="fixed z-50 min-w-[180px] bg-popover border border-border rounded-lg shadow-lg py-1"
      style={{ left: `${position.x}px`, top: `${position.y}px` }}
      onClick={(e) => e.stopPropagation()}
    >
      {children}
    </div>
  )
}

/**
 * 右键菜单项组件
 */
interface MenuItemProps {
  icon?: React.ReactNode
  label: string
  onClick: () => void
  danger?: boolean
  disabled?: boolean
}

export function MenuItem({ icon, label, onClick, danger, disabled }: MenuItemProps) {
  return (
    <div
      className={cn(
        "flex items-center gap-2 px-3 py-2 text-sm cursor-pointer transition-colors",
        danger
          ? "text-destructive hover:bg-destructive/10"
          : "hover:bg-muted/50",
        disabled && "opacity-50 cursor-not-allowed hover:bg-transparent"
      )}
      onClick={() => {
        if (!disabled) {
          onClick()
        }
      }}
    >
      {icon && <span className="w-4 h-4 flex-shrink-0">{icon}</span>}
      <span>{label}</span>
    </div>
  )
}

/**
 * 右键菜单分隔符
 */
export function MenuSeparator() {
  return <div className="border-t border-border/50 my-1 mx-2" />
}
