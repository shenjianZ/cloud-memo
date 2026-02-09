import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useNoteStore } from '@/store/noteStore'
import { getNoteTitle } from '@/lib/noteHelpers'
import { formatDistanceToNow } from 'date-fns'
import { zhCN } from 'date-fns/locale'
import {
  Trash2,
  RotateCcw,
  CheckSquare,
  Square,
  AlertCircle,
  ArrowLeft,
} from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { listDeletedNotes, permanentlyDeleteNote, permanentlyDeleteNotes } from '@/services/noteApi'

interface Note {
  id: string
  title: string
  content: any
  markdownCache?: string
  createdAt: number
  updatedAt: number
  deletedAt?: number
  folder?: string
  tags: string[]
  isFavorite: boolean
  isPinned: boolean
}

/**
 * 回收站页面
 *
 * 功能：
 * - 显示所有已删除的笔记（is_deleted = true）
 * - 恢复单个笔记到"已恢复笔记"文件夹
 * - 批量恢复笔记
 * - 永久删除笔记
 */
export default function Trash() {
  const navigate = useNavigate()
  const { restoreNote, restoreNotes } = useNoteStore()
  const [trashNotes, setTrashNotes] = useState<Note[]>([])
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())
  const [isLoading, setIsLoading] = useState(true)
  const [isRestoring, setIsRestoring] = useState(false)
  const [isDeleting, setIsDeleting] = useState(false)

  // 加载回收站笔记
  useEffect(() => {
    loadTrashNotes()
  }, [])

  const loadTrashNotes = async () => {
    setIsLoading(true)
    try {
      // 直接调用后端 API 获取已删除的笔记
      const apiNotes = await listDeletedNotes()

      const deletedNotes = apiNotes.map((apiNote) => ({
        id: apiNote.id,
        title: apiNote.title,
        content: apiNote.content,
        markdownCache: apiNote.markdownCache,
        createdAt: apiNote.createdAt * 1000,
        updatedAt: apiNote.updatedAt * 1000,
        deletedAt: apiNote.deletedAt ? apiNote.deletedAt * 1000 : undefined,
        folder: apiNote.folderId,
        tags: [],
        isFavorite: apiNote.isFavorite,
        isPinned: apiNote.isPinned,
      }))

      setTrashNotes(deletedNotes)
    } catch (error) {
      console.error('Failed to load trash notes:', error)
      toast.error('加载回收站失败')
    } finally {
      setIsLoading(false)
    }
  }

  // 恢复单个笔记
  const handleRestore = async (id: string) => {
    setIsRestoring(true)
    try {
      await restoreNote(id)
      // 从列表中移除已恢复的笔记
      setTrashNotes((prev) => prev.filter((n) => n.id !== id))
      setSelectedIds((prev) => {
        const next = new Set(prev)
        next.delete(id)
        return next
      })
    } catch (error) {
      console.error('Failed to restore note:', error)
    } finally {
      setIsRestoring(false)
    }
  }

  // 批量恢复选中的笔记
  const handleBatchRestore = async () => {
    if (selectedIds.size === 0) {
      toast.warning('请先选择要恢复的笔记')
      return
    }

    setIsRestoring(true)
    try {
      const ids = Array.from(selectedIds)
      await restoreNotes(ids)
      // 从列表中移除已恢复的笔记
      setTrashNotes((prev) => prev.filter((n) => !selectedIds.has(n.id)))
      setSelectedIds(new Set())
    } catch (error) {
      console.error('Failed to batch restore:', error)
    } finally {
      setIsRestoring(false)
    }
  }

  // 单个硬删除
  const handlePermanentDelete = async (id: string) => {
    if (!confirm('确定要永久删除这篇笔记吗？此操作无法撤销！')) {
      return
    }

    setIsDeleting(true)
    try {
      await permanentlyDeleteNote(id)
      setTrashNotes((prev) => prev.filter((n) => n.id !== id))
      setSelectedIds((prev) => {
        const next = new Set(prev)
        next.delete(id)
        return next
      })
      toast.success('笔记已永久删除')
    } catch (error) {
      toast.error('删除失败', {
        description: error instanceof Error ? error.message : '未知错误',
      })
    } finally {
      setIsDeleting(false)
    }
  }

  // 批量硬删除
  const handleBatchPermanentDelete = async () => {
    if (selectedIds.size === 0) {
      toast.warning('请先选择要删除的笔记')
      return
    }

    if (!confirm(`确定要永久删除选中的 ${selectedIds.size} 篇笔记吗？此操作无法撤销！`)) {
      return
    }

    setIsDeleting(true)
    try {
      const ids = Array.from(selectedIds)
      const count = await permanentlyDeleteNotes(ids)
      setTrashNotes((prev) => prev.filter((n) => !selectedIds.has(n.id)))
      setSelectedIds(new Set())
      toast.success(`已永久删除 ${count} 篇笔记`)
    } catch (error) {
      toast.error('批量删除失败', {
        description: error instanceof Error ? error.message : '未知错误',
      })
    } finally {
      setIsDeleting(false)
    }
  }

  // 切换选择状态
  const toggleSelect = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  // 全选/取消全选
  const toggleSelectAll = () => {
    if (selectedIds.size === trashNotes.length) {
      setSelectedIds(new Set())
    } else {
      setSelectedIds(new Set(trashNotes.map((n) => n.id)))
    }
  }

  if (isLoading) {
    return (
      <div className="flex h-64 items-center justify-center px-4 sm:px-6">
        <div className="text-muted-foreground">加载中...</div>
      </div>
    )
  }

  return (
    <div className="space-y-6 px-4 sm:px-6 py-4 sm:py-6 max-w-5xl mx-auto">
      {/* 标题和操作栏 */}
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
          <h1 className="text-2xl sm:text-3xl font-bold tracking-tight">回收站</h1>
          <p className="text-muted-foreground">
            {trashNotes.length > 0
              ? `共 ${trashNotes.length} 篇已删除的笔记`
              : '回收站为空'}
          </p>
        </div>

        {/* 批量操作按钮 */}
        {trashNotes.length > 0 && selectedIds.size > 0 && (
          <div className="flex gap-2 flex-wrap">
            <button
              onClick={handleBatchRestore}
              disabled={isRestoring || isDeleting}
              className="inline-flex items-center gap-2 rounded-lg bg-primary px-3 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              <RotateCcw className="h-4 w-4" />
              <span className="hidden sm:inline">恢复选中</span>
              <span className="sm:hidden">恢复</span>
              ({selectedIds.size})
            </button>
            <button
              onClick={handleBatchPermanentDelete}
              disabled={isDeleting}
              className="inline-flex items-center gap-2 rounded-lg bg-red-600 px-3 py-2 text-sm font-medium text-white hover:bg-red-700 disabled:opacity-50"
            >
              <Trash2 className="h-4 w-4" />
              <span className="hidden sm:inline">永久删除</span>
              <span className="sm:hidden">删除</span>
              ({selectedIds.size})
            </button>
            <button
              onClick={() => setSelectedIds(new Set())}
              className="inline-flex items-center gap-2 rounded-lg border px-3 py-2 text-sm font-medium hover:bg-accent"
            >
              取消选择
            </button>
          </div>
        )}
      </div>

      {/* 全选按钮 */}
      {trashNotes.length > 0 && (
        <div className="flex items-center gap-2">
          <button
            onClick={toggleSelectAll}
            className="inline-flex items-center gap-2 rounded-lg border px-3 py-2 text-sm font-medium hover:bg-accent"
          >
            {selectedIds.size === trashNotes.length ? (
              <>
                <Square className="h-4 w-4" />
                取消全选
              </>
            ) : (
              <>
                <CheckSquare className="h-4 w-4" />
                全选
              </>
            )}
          </button>
        </div>
      )}

      {/* 回收站为空 */}
      {trashNotes.length === 0 && (
        <div className="flex flex-col items-center justify-center rounded-lg border border-dashed py-16">
          <Trash2 className="mb-4 h-16 w-16 text-muted-foreground/50" />
          <h3 className="mb-2 text-lg font-semibold">回收站为空</h3>
          <p className="text-center text-sm text-muted-foreground">
            删除的笔记会在这里保留，<br />
            可以随时恢复到"已恢复笔记"文件夹
          </p>
        </div>
      )}

      {/* 已删除笔记卡片网格 */}
      {trashNotes.length > 0 && (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {trashNotes.map((note) => (
            <div
              key={note.id}
              className={`group rounded-lg border bg-card p-4 transition-all hover:shadow-md ${
                selectedIds.has(note.id) ? 'border-primary bg-accent' : ''
              }`}
            >
              {/* 顶部：选择框和标题 */}
              <div className="flex items-start gap-3 mb-3">
                <button
                  onClick={() => toggleSelect(note.id)}
                  className="mt-1 flex-shrink-0"
                  title={selectedIds.has(note.id) ? '取消选择' : '选择'}
                >
                  {selectedIds.has(note.id) ? (
                    <CheckSquare className="h-5 w-5 text-primary" />
                  ) : (
                    <Square className="h-5 w-5 text-muted-foreground" />
                  )}
                </button>
                <div className="flex-1 min-w-0">
                  <h3 className="font-semibold text-base line-clamp-1 pr-2">{getNoteTitle(note)}</h3>
                  <div className="mt-1 flex items-center gap-1 text-xs text-muted-foreground">
                    <AlertCircle className="h-3 w-3" />
                    {note.deletedAt && (
                      <span className="truncate">
                        {formatDistanceToNow(note.deletedAt, { addSuffix: true, locale: zhCN })}
                      </span>
                    )}
                  </div>
                </div>
              </div>

              {/* 笔记摘要 */}
              {note.markdownCache && (
                <p className="text-sm text-muted-foreground line-clamp-2 mb-3 pl-8">
                  {note.markdownCache.slice(0, 150)}
                </p>
              )}

              {/* 操作按钮 */}
              <div className="flex gap-2 pl-8">
                <button
                  onClick={() => handleRestore(note.id)}
                  disabled={isRestoring || isDeleting}
                  className="inline-flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium text-green-700 hover:bg-green-100 disabled:opacity-50 transition-colors"
                  title="恢复笔记"
                >
                  <RotateCcw className="h-4 w-4" />
                  恢复
                </button>
                <button
                  onClick={() => handlePermanentDelete(note.id)}
                  disabled={isDeleting}
                  className="inline-flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium text-red-700 hover:bg-red-100 disabled:opacity-50 transition-colors"
                  title="永久删除"
                >
                  <Trash2 className="h-4 w-4" />
                  删除
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* 底部提示 */}
      {trashNotes.length > 0 && (
        <div className="rounded-lg border border-blue-200 bg-blue-50 p-4 text-sm text-blue-900 dark:border-blue-800 dark:bg-blue-950 dark:text-blue-100">
          <div className="flex items-start gap-2">
            <AlertCircle className="mt-0.5 h-4 w-4 flex-shrink-0" />
            <div>
              <p className="font-semibold">关于回收站</p>
              <ul className="mt-1 space-y-1 text-blue-800 dark:text-blue-200">
                <li>• 恢复的笔记会移至"已恢复笔记"文件夹</li>
                <li>• 您可以手动将恢复的笔记移动到其他文件夹</li>
                <li>• 删除的笔记会永久保留在回收站中</li>
              </ul>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
