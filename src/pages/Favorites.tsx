import { useNavigate } from 'react-router-dom'
import { Star, FileText, ArrowLeft, Clock } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useNoteStore } from '@/store/noteStore'
import { getNoteTitle, getPlainText } from '@/lib/noteHelpers'
import { formatDistanceToNow } from 'date-fns'
import { zhCN } from 'date-fns/locale'

export default function Favorites() {
  const navigate = useNavigate()
  const { notes } = useNoteStore()

  // 获取所有收藏的笔记（排除已删除的）
  const favoriteNotes = notes.filter(note => note.isFavorite && !note.isDeleted)

  // 按更新时间排序
  const sortedNotes = [...favoriteNotes].sort(
    (a, b) => b.updatedAt - a.updatedAt
  )

  return (
    <div className="space-y-6 px-4 sm:px-6 py-4 sm:py-6 max-w-6xl mx-auto">
      {/* 标题 */}
      <div className="flex items-center gap-3">
        <Button
          variant="ghost"
          size="sm"
          className="h-8 w-8 p-0"
          onClick={() => navigate(-1)}
          title="返回"
        >
          <ArrowLeft className="h-4 w-4" />
        </Button>
        <div className="flex-1">
          <h1 className="text-2xl sm:text-3xl font-bold tracking-tight flex items-center gap-2">
            <Star className="w-6 h-6 sm:w-8 sm:h-8 text-yellow-500" />
            收藏的笔记
          </h1>
          <p className="text-muted-foreground mt-1 sm:mt-2">
            共 {favoriteNotes.length} 篇收藏的笔记
          </p>
        </div>
      </div>

      {/* 笔记卡片网格 */}
      {sortedNotes.length === 0 ? (
        <div className="rounded-lg border p-12 text-center">
          <Star className="w-16 h-16 text-muted-foreground mx-auto mb-4" />
          <h3 className="text-lg font-semibold mb-2">暂无收藏的笔记</h3>
          <p className="text-sm text-muted-foreground mb-4">
            点击笔记卡片上的星标图标即可收藏
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {sortedNotes.map((note) => (
            <button
              key={note.id}
              onClick={() => navigate(`/editor/${note.id}`)}
              className="text-left rounded-lg border bg-card p-4 hover:bg-accent/50 transition-all group hover:shadow-md"
            >
              <div className="flex items-start gap-3">
                <FileText className="w-5 h-5 text-muted-foreground flex-shrink-0 mt-0.5" />
                <div className="flex-1 min-w-0">
                  <h3 className="font-semibold text-base mb-2 line-clamp-1 group-hover:text-primary transition-colors">
                    {getNoteTitle(note)}
                  </h3>
                  <p className="text-sm text-muted-foreground line-clamp-2 mb-3">
                    {getPlainText(note.content) || '暂无内容'}
                  </p>
                  <div className="flex items-center gap-2 text-xs text-muted-foreground">
                    <Clock className="w-3 h-3" />
                    <span>
                      {formatDistanceToNow(new Date(note.updatedAt), { addSuffix: true, locale: zhCN })}
                    </span>
                  </div>
                </div>
                <Star className="w-4 h-4 text-yellow-500 flex-shrink-0" />
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
