import { formatDistanceToNow } from 'date-fns'
import { zhCN } from 'date-fns/locale'
import { Calendar, User, Edit, Tag as TagIcon } from 'lucide-react'
import { cn } from '@/lib/utils'
import { useTagStore } from '@/store/tagStore'

interface NoteMetaProps {
  createdAt: number
  updatedAt: number
  author?: string
  tags?: string[]
  className?: string
}

// 安全地格式化日期距离
function safeFormatDistanceToNow(timestamp: number | null | undefined): string {
  // 检查时间戳是否有效
  if (!timestamp || timestamp <= 0 || !Number.isFinite(timestamp)) {
    return '未知时间'
  }

  try {
    const date = new Date(timestamp)
    // 检查日期是否有效（检查年份是否在合理范围内）
    const year = date.getFullYear()
    if (year < 2000 || year > 2100) {
      return '未知时间'
    }

    return formatDistanceToNow(date, { addSuffix: true, locale: zhCN })
  } catch {
    return '未知时间'
  }
}

export function NoteMeta({ createdAt, updatedAt, author, tags = [], className }: NoteMetaProps) {
  const { tags: allTags } = useTagStore()
  const noteTags = allTags.filter(tag => tags.includes(tag.id))

  return (
    <div className={cn(
      "flex items-center justify-between text-xs text-muted-foreground px-4 py-2 border-b border-border/50 bg-muted/30",
      className
    )}>
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-1.5">
          <User className="w-3 h-3" />
          <span>{author || '匿名用户'}</span>
        </div>

        <div className="flex items-center gap-1.5">
          <Calendar className="w-3 h-3" />
          <span>创建于 {safeFormatDistanceToNow(createdAt)}</span>
        </div>

        <div className="flex items-center gap-1.5">
          <Edit className="w-3 h-3" />
          <span>更新于 {safeFormatDistanceToNow(updatedAt)}</span>
        </div>
      </div>

      {/* 标签显示 */}
      {noteTags.length > 0 && (
        <div className="flex items-center gap-1.5">
          <TagIcon className="w-3 h-3" />
          <div className="flex items-center gap-1">
            {noteTags.slice(0, 3).map(tag => (
              <div
                key={tag.id}
                className="px-1.5 py-0.5 rounded text-xs font-medium border"
                style={{
                  backgroundColor: tag.color ? `${tag.color}20` : undefined,
                  borderColor: tag.color || 'hsl(var(--border))',
                  color: tag.color || undefined,
                }}
              >
                {tag.name}
              </div>
            ))}
            {noteTags.length > 3 && (
              <span className="text-muted-foreground">+{noteTags.length - 3}</span>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
