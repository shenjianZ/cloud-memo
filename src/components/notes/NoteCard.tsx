import { formatDistanceToNow } from 'date-fns'
import { zhCN } from 'date-fns/locale'
import { Star, Tag as TagIcon, CloudOff } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { Note } from '@/types/note'
import { useTagStore } from '@/store/tagStore'
import { getNoteTitle, getPlainText } from '@/lib/noteHelpers'
import { useAuthStore } from '@/store/authStore'

interface NoteCardProps {
  note: Note
  onClick: () => void
  isActive?: boolean
  onContextMenu?: (e: React.MouseEvent, noteId: string) => void
}

/**
 * 笔记卡片组件
 */
export function NoteCard({ note, onClick, isActive, onContextMenu }: NoteCardProps) {
  const { tags } = useTagStore()
  const { isAuthenticated } = useAuthStore()
  const excerpt = getPlainText(note.content)

  // 获取笔记的标签
  const noteTags = tags.filter(tag => note.tags.includes(tag.id))

  return (
    <div
      className={cn(
        "p-3 rounded-lg cursor-pointer transition-colors group hover:shadow-sm",
        isActive
          ? "bg-primary text-primary-foreground"
          : "bg-card hover:bg-muted/50 border border-transparent hover:border-border"
      )}
      onClick={onClick}
      onContextMenu={(e) => onContextMenu?.(e, note.id)}
    >
      <div className="flex items-start justify-between gap-2 mb-1">
        <h3 className={cn(
          "font-medium text-sm line-clamp-1 flex-1",
          isActive ? "text-primary-foreground" : "text-foreground"
        )}>
          {getNoteTitle(note)}
        </h3>
        {note.isFavorite && (
          <Star className={cn(
            "w-3 h-3 flex-shrink-0",
            isActive
              ? "fill-yellow-300 text-yellow-300"
              : "fill-yellow-500 text-yellow-500"
          )} />
        )}
      </div>

      {excerpt && (
        <p className={cn(
          "text-xs line-clamp-2 mb-2",
          isActive ? "text-primary-foreground/70" : "text-muted-foreground"
        )}>
          {excerpt}
        </p>
      )}

      {/* 标签 */}
      {noteTags.length > 0 && (
        <div className="flex flex-wrap gap-1 mb-2">
          {noteTags.slice(0, 3).map(tag => (
            <div
              key={tag.id}
              className={cn(
                "inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs font-medium border",
                isActive ? "border-primary-foreground/30 bg-primary-foreground/10" : "bg-muted/50"
              )}
              style={tag.color ? {
                backgroundColor: tag.color + '20',
                borderColor: tag.color,
                color: tag.color,
              } : undefined}
            >
              <TagIcon className="w-2.5 h-2.5 flex-shrink-0" />
              <span className="truncate max-w-[60px]">{tag.name}</span>
            </div>
          ))}
          {noteTags.length > 3 && (
            <span className={cn(
              "text-xs px-1.5 py-0.5",
              isActive ? "text-primary-foreground/60" : "text-muted-foreground"
            )}>
              +{noteTags.length - 3}
            </span>
          )}
        </div>
      )}

      <div className={cn(
        "flex items-center gap-1 text-xs",
        isActive ? "text-primary-foreground/60" : "text-muted-foreground"
      )}>
        <span>
          {formatDistanceToNow(new Date(note.updatedAt), {
            addSuffix: true,
            locale: zhCN,
          })}
        </span>
        {/* 待同步图标（仅登录且笔记有脏标记时显示） */}
        {isAuthenticated && note.isDirty && (
          <CloudOff className="w-3 h-3 text-orange-500" aria-label="待同步" />
        )}
      </div>
    </div>
  )
}
