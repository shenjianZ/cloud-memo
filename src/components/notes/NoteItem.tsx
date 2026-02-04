import { FileText } from 'lucide-react'
import { cn } from '@/lib/utils'
import { getNoteTitle } from '@/lib/noteHelpers'
import type { Note } from '@/types/note'

interface NoteItemProps {
  note: Note
  onClick: () => void
  isActive?: boolean
  onContextMenu?: (e: React.MouseEvent, noteId: string) => void
  level?: number
}

/**
 * 简洁的笔记项组件（类似文件夹项样式）
 * 只显示标题和图标，与文件夹项对齐
 */
export function NoteItem({ note, onClick, isActive, onContextMenu, level = 0 }: NoteItemProps) {
  const title = getNoteTitle(note)

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation()
    onClick()
  }

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault() // 阻止浏览器默认的右键菜单
    e.stopPropagation() // 阻止事件冒泡到父元素
    onContextMenu?.(e, note.id)
  }

  return (
    <div
      className={cn(
        "flex items-center gap-2 px-3 py-2 hover:bg-muted/50 cursor-pointer rounded-lg transition-colors text-sm group",
        isActive && "bg-accent text-accent-foreground"
      )}
      style={{ paddingLeft: `${level * 12 + 12}px` }}
      onClick={handleClick}
      onContextMenu={handleContextMenu}
    >
      {/* 占位空间 - 与文件夹的展开箭头对齐 */}
      <div className="w-4 h-4 flex-shrink-0" />

      {/* 笔记图标 */}
      <FileText className="w-4 h-4 flex-shrink-0 text-muted-foreground group-hover:text-foreground" />

      {/* 笔记标题 */}
      <span className="font-medium truncate flex-1">{title}</span>
    </div>
  )
}
