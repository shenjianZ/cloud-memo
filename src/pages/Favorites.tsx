import { useNavigate } from 'react-router-dom'
import { Star, FileText } from 'lucide-react'
import { useNoteStore } from '@/store/noteStore'
import { getNoteTitle, getPlainText } from '@/lib/noteHelpers'
import { formatDistanceToNow } from 'date-fns'
import { zhCN } from 'date-fns/locale'

export default function Favorites() {
  const navigate = useNavigate()
  const { notes } = useNoteStore()

  // 获取所有收藏的笔记
  const favoriteNotes = notes.filter(note => note.isFavorite)

  // 按更新时间排序
  const sortedNotes = [...favoriteNotes].sort(
    (a, b) => b.updatedAt - a.updatedAt
  )

  return (
    <div className="space-y-6 px-8 py-6">
      {/* 标题 */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight flex items-center gap-2">
          <Star className="w-8 h-8 text-yellow-500" />
          收藏的笔记
        </h1>
        <p className="text-muted-foreground mt-2">
          共 {favoriteNotes.length} 篇收藏的笔记
        </p>
      </div>

      {/* 笔记列表 */}
      {sortedNotes.length === 0 ? (
        <div className="rounded-lg border p-12 text-center">
          <Star className="w-16 h-16 text-muted-foreground mx-auto mb-4" />
          <h3 className="text-lg font-semibold mb-2">暂无收藏的笔记</h3>
          <p className="text-sm text-muted-foreground mb-4">
            点击笔记卡片上的星标图标即可收藏
          </p>
        </div>
      ) : (
        <div className="grid gap-4">
          {sortedNotes.map((note) => (
            <button
              key={note.id}
              onClick={() => navigate(`/editor/${note.id}`)}
              className="w-full text-left rounded-lg border p-4 hover:bg-muted/50 transition-colors group"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-2">
                    <FileText className="w-4 h-4 text-muted-foreground flex-shrink-0" />
                    <h3 className="font-semibold text-lg truncate">
                      {getNoteTitle(note)}
                    </h3>
                    <Star className="w-4 h-4 text-yellow-500 flex-shrink-0" />
                  </div>
                  <p className="text-sm text-muted-foreground line-clamp-2">
                    {getPlainText(note.content) || '暂无内容'}
                  </p>
                  <div className="mt-2 text-xs text-muted-foreground">
                    更新于 {formatDistanceToNow(new Date(note.updatedAt), { addSuffix: true, locale: zhCN })}
                  </div>
                </div>
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
