import { useEffect, useState, useCallback, useRef } from 'react'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { Search, Plus, ChevronLeft, ChevronRight } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { useNoteStore } from '@/store/noteStore'
import { NoteCard } from './NoteCard'
import { ScrollArea } from '@/components/ui/scroll-area'
import { cn } from '@/lib/utils'
import { getNoteTitle } from '@/lib/noteHelpers'

/**
 * 笔记列表组件
 * 中间栏，显示笔记卡片列表
 */
export function NoteList() {
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const folderId = searchParams.get('folder')
  const filter = searchParams.get('filter')
  const { notes, loadNotesFromStorage, createNote } = useNoteStore()
  const [searchQuery, setSearchQuery] = useState('')
  const [isCollapsed, setIsCollapsed] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const isCreatingRef = useRef(false) // 使用 ref 防止重复创建

  useEffect(() => {
    loadNotesFromStorage()
  }, [loadNotesFromStorage])

  // 过滤笔记
  const filteredNotes = (() => {
    let result = notes

    // 搜索过滤
    if (searchQuery) {
      const query = searchQuery.toLowerCase()
      result = result.filter(note => {
        const title = getNoteTitle(note).toLowerCase()
        return title.includes(query) ||
          (typeof note.content === 'string' && note.content.toLowerCase().includes(query))
      })
    }

    // 文件夹过滤
    if (folderId) {
      result = result.filter(n => n.folder === folderId)
    }

    // 特殊过滤
    if (filter === 'favorites') {
      result = result.filter(n => n.isFavorite)
    }

    return result
  })()

  // 按更新时间排序
  const sortedNotes = [...filteredNotes].sort(
    (a, b) => b.updatedAt - a.updatedAt
  )

  // 获取当前活动笔记 ID
  const activeNoteId = searchParams.get('noteId')

  // 创建新笔记 - 使用严格防重复机制
  const handleCreateNote = useCallback(async () => {
    // 多重检查防止重复创建
    if (isLoading || isCreatingRef.current) {
      console.log('正在创建笔记，请勿重复点击')
      return
    }

    // 立即设置标志位
    isCreatingRef.current = true
    setIsLoading(true)

    try {
      const newNote = await createNote({
        content: { type: 'doc', content: [] },
      })
      navigate(`/editor/${newNote.id}`)
    } catch (error) {
      console.error('Failed to create note:', error)
    } finally {
      // 延迟重置，确保不会有快速重复点击
      setTimeout(() => {
        isCreatingRef.current = false
        setIsLoading(false)
      }, 500)
    }
  }, [isLoading, createNote, navigate])

  return (
    <aside
      className={cn(
        "bg-muted/30 border-r border-border h-screen flex flex-col transition-all duration-300",
        isCollapsed ? "w-12" : "w-80"
      )}
    >
      {/* 搜索栏 */}
      <div className="h-14 border-b border-border flex items-center px-3 gap-2">
        {!isCollapsed && (
          <div className="relative flex-1">
            <Search className="absolute left-2 top-1/2 transform -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              placeholder="搜索笔记..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-8 h-9 text-sm"
            />
          </div>
        )}
        <Button
          variant="ghost"
          size="sm"
          className="h-8 w-8 p-0 flex-shrink-0"
          onClick={() => setIsCollapsed(!isCollapsed)}
        >
          {isCollapsed ? (
            <ChevronRight className="w-4 h-4" />
          ) : (
            <ChevronLeft className="w-4 h-4" />
          )}
        </Button>
      </div>

      {/* 笔记列表 */}
      <ScrollArea className="flex-1">
        <div className="p-2 space-y-1">
          {!isCollapsed ? (
            <>
              {sortedNotes.length > 0 ? (
                sortedNotes.map((note) => (
                  <NoteCard
                    key={note.id}
                    note={note}
                    onClick={() => navigate(`/editor/${note.id}`)}
                    isActive={activeNoteId === note.id}
                  />
                ))
              ) : (
                <div className="text-center text-muted-foreground text-sm py-8">
                  {searchQuery
                    ? '没有找到匹配的笔记'
                    : filter === 'favorites'
                    ? '暂无收藏的笔记'
                    : folderId
                    ? '此文件夹为空'
                    : '暂无笔记'}
                </div>
              )}
            </>
          ) : (
            <div className="space-y-1">
              {sortedNotes.map((note) => {
                const title = getNoteTitle(note)
                return (
                  <div
                    key={note.id}
                    className="h-8 flex items-center justify-center hover:bg-muted/50 rounded cursor-pointer"
                    onClick={() => navigate(`/editor/${note.id}`)}
                    title={title}
                  >
                    {title?.[0]?.toUpperCase() || 'N'}
                  </div>
                )
              })}
            </div>
          )}
        </div>
      </ScrollArea>

      {/* 底部新建按钮 */}
      {!isCollapsed && (
        <div className="p-2 border-t border-border">
          <Button
            variant="default"
            size="sm"
            className="w-full justify-start"
            onClick={(e) => {
              e.preventDefault()
              e.stopPropagation()
              handleCreateNote()
            }}
            disabled={isLoading}
            type="button"
          >
            <Plus className="w-4 h-4 mr-2" />
            {isLoading ? '创建中...' : '新建笔记'}
          </Button>
        </div>
      )}
    </aside>
  )
}
