import { formatDistanceToNow } from 'date-fns'
import { zhCN } from 'date-fns/locale'
import { Calendar, Clock, User, Edit, Tag as TagIcon } from 'lucide-react'
import { cn } from '@/lib/utils'
import { useTagStore } from '@/store/tagStore'

interface NoteMetaProps {
  createdAt: number
  updatedAt: number
  author?: string
  tags?: string[]
  className?: string
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
          <span>创建于 {formatDistanceToNow(new Date(createdAt), { addSuffix: true, locale: zhCN })}</span>
        </div>

        <div className="flex items-center gap-1.5">
          <Edit className="w-3 h-3" />
          <span>更新于 {formatDistanceToNow(new Date(updatedAt), { addSuffix: true, locale: zhCN })}</span>
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
