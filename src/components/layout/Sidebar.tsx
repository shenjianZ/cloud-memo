import { useState, useCallback, useRef, useEffect } from 'react'
import { Search, Settings, ChevronDown, ChevronRight, Folder, FolderOpen, Plus, Inbox, Star } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useNoteStore } from '@/store/noteStore'
import { cn } from '@/lib/utils'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { NoteCard } from '../notes/NoteCard'
import { getNoteTitle } from '@/lib/noteHelpers'

interface FolderNode {
  id: string
  name: string
  parentId: string | null
  children: FolderNode[]
}

/**
 * 合并的侧边栏组件
 * 包含：顶部工具栏 + 文件夹树 + 笔记列表
 */
export function Sidebar() {
  const [searchQuery, setSearchQuery] = useState('')
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set())
  const { folders, notes, createNote, loadNotesFromStorage } = useNoteStore()
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false)
  const [foldersCollapsed, setFoldersCollapsed] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const isCreatingRef = useRef(false) // 使用 ref 防止重复创建

  // 获取当前活动笔记 ID
  const activeNoteId = searchParams.get('noteId')
  const folderFilter = searchParams.get('folder')
  const favoritesFilter = searchParams.get('filter') === 'favorites'

  // 加载笔记数据
  useEffect(() => {
    loadNotesFromStorage()
  }, [loadNotesFromStorage])

  // 构建文件夹树结构
  const buildTree = (): FolderNode[] => {
    const map = new Map<string, FolderNode>()
    const roots: FolderNode[] = []

    folders.forEach(folder => {
      map.set(folder.id, { ...folder, children: [] })
    })

    folders.forEach(folder => {
      const node = map.get(folder.id)!
      if (folder.parentId) {
        const parent = map.get(folder.parentId)
        if (parent) {
          parent.children.push(node)
        }
      } else {
        roots.push(node)
      }
    })

    return roots
  }

  const toggleFolder = (folderId: string) => {
    setExpandedFolders(prev => {
      const next = new Set(prev)
      if (next.has(folderId)) {
        next.delete(folderId)
      } else {
        next.add(folderId)
      }
      return next
    })
  }

  const renderFolder = (folder: FolderNode, level: number = 0) => {
    const isExpanded = expandedFolders.has(folder.id)
    const hasChildren = folder.children.length > 0
    const isActive = folderFilter === folder.id

    return (
      <div key={folder.id}>
        <div
          className={cn(
            "flex items-center gap-2 px-3 py-2 hover:bg-muted/50 cursor-pointer rounded-lg transition-colors text-sm",
            isActive && "bg-accent text-accent-foreground"
          )}
          style={{ paddingLeft: `${level * 12 + 12}px` }}
          onClick={() => {
            if (hasChildren) {
              toggleFolder(folder.id)
            }
            navigate(`/?folder=${folder.id}`)
          }}
        >
          {hasChildren && (
            isExpanded ? (
              <ChevronDown className="w-3 h-3 text-muted-foreground flex-shrink-0" />
            ) : (
              <ChevronRight className="w-3 h-3 text-muted-foreground flex-shrink-0" />
            )
          )}
          {isExpanded ? (
            <FolderOpen className="w-4 h-4 text-blue-500 flex-shrink-0" />
          ) : (
            <Folder className="w-4 h-4 text-blue-500 flex-shrink-0" />
          )}
          <span className="font-medium truncate flex-1">{folder.name}</span>
        </div>

        {isExpanded && hasChildren && (
          <div>
            {folder.children.map(child => renderFolder(child, level + 1))}
          </div>
        )}
      </div>
    )
  }

  const tree = buildTree()

  // 过滤笔记
  const filteredNotes = (() => {
    let result = notes

    if (searchQuery) {
      const query = searchQuery.toLowerCase()
      result = result.filter(note => {
        const title = getNoteTitle(note).toLowerCase()
        return title.includes(query) ||
          (typeof note.content === 'string' && note.content.toLowerCase().includes(query))
      })
    }

    if (folderFilter) {
      result = result.filter(n => n.folder === folderFilter)
    }

    if (favoritesFilter) {
      result = result.filter(n => n.isFavorite)
    }

    return result
  })()

  // 按更新时间排序
  const sortedNotes = [...filteredNotes].sort(
    (a, b) => b.updatedAt - a.updatedAt
  )

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
        title: '未命名笔记',
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
        isSidebarCollapsed ? "w-12" : "w-80"
      )}
    >
      {/* 顶部工具栏 */}
      <div className="h-14 border-b border-border flex items-center justify-between px-3">
        {!isSidebarCollapsed ? (
          <>
            <div className="flex items-center gap-2 flex-1">
              <div className="relative flex-1">
                <Search className="absolute left-2 top-1/2 transform -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                <Input
                  placeholder="搜索..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-8 h-9 text-sm"
                />
              </div>
            </div>

            <div className="flex items-center gap-1">
              <Button
                variant="ghost"
                size="sm"
                className="h-8 w-8 p-0"
                onClick={() => navigate('/settings')}
                title="设置"
              >
                <Settings className="w-4 h-4" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="h-8 w-8 p-0"
                onClick={() => setIsSidebarCollapsed(!isSidebarCollapsed)}
                title="折叠侧边栏"
              >
                <ChevronDown className="w-4 h-4" />
              </Button>
            </div>
          </>
        ) : (
          <div className="flex items-center justify-center w-full">
            <Button
              variant="ghost"
              size="sm"
              className="h-8 w-8 p-0"
              onClick={() => setIsSidebarCollapsed(false)}
              title="展开侧边栏"
            >
              <ChevronRight className="w-4 h-4" />
            </Button>
          </div>
        )}
      </div>

      {/* 内容区域 */}
      <div className="flex-1 overflow-y-auto custom-scrollbar">
        {!isSidebarCollapsed && (
          <>
            {/* 快捷入口 + 文件夹区域 */}
            <div className="border-b border-border/50">
              {/* 快捷入口 */}
              <div className="px-2 py-2 space-y-1">
                <div
                  className={cn(
                    "flex items-center gap-2 px-3 py-2 hover:bg-muted/50 cursor-pointer rounded-lg transition-colors text-sm",
                    !folderFilter && !favoritesFilter && "bg-accent text-accent-foreground"
                  )}
                  onClick={() => navigate('/')}
                >
                  <Inbox className="w-4 h-4 text-muted-foreground" />
                  <span className="font-medium">全部笔记</span>
                  <span className="ml-auto text-xs text-muted-foreground">{notes.length}</span>
                </div>

                <div
                  className={cn(
                    "flex items-center gap-2 px-3 py-2 hover:bg-muted/50 cursor-pointer rounded-lg transition-colors text-sm",
                    favoritesFilter && "bg-accent text-accent-foreground"
                  )}
                  onClick={() => navigate('/?filter=favorites')}
                >
                  <Star className="w-4 h-4 text-yellow-500" />
                  <span className="font-medium">收藏</span>
                  <span className="ml-auto text-xs text-muted-foreground">
                    {notes.filter(n => n.isFavorite).length}
                  </span>
                </div>
              </div>

              {/* 文件夹折叠按钮 */}
              <div
                className="flex items-center gap-2 px-3 py-2 hover:bg-muted/50 cursor-pointer text-xs font-medium text-muted-foreground uppercase tracking-wider"
                onClick={() => setFoldersCollapsed(!foldersCollapsed)}
              >
                {foldersCollapsed ? <ChevronRight className="w-3 h-3" /> : <ChevronDown className="w-3 h-3" />}
                文件夹
              </div>

              {/* 文件夹树 */}
              {!foldersCollapsed && tree.length > 0 ? (
                <div className="pb-2">
                  {tree.map(folder => renderFolder(folder))}
                </div>
              ) : (
                !foldersCollapsed && tree.length === 0 && (
                  <div className="text-center text-muted-foreground text-xs py-4">
                    暂无文件夹
                  </div>
                )
              )}
            </div>

            {/* 笔记列表 */}
            <div className="px-2 py-2">
              <div className="flex items-center justify-between mb-2 px-1">
                <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
                  笔记
                </span>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 w-7 p-0"
                  onClick={(e) => {
                    e.preventDefault()
                    e.stopPropagation()
                    handleCreateNote()
                  }}
                  disabled={isLoading}
                  title="新建笔记"
                  type="button" // 确保不会触发表单提交
                >
                  <Plus className="w-4 h-4" />
                </Button>
              </div>

              <div className="space-y-1">
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
                      : favoritesFilter
                      ? '暂无收藏的笔记'
                      : folderFilter
                      ? '此文件夹为空'
                      : '暂无笔记'}
                  </div>
                )}
              </div>
            </div>
          </>
        )}
      </div>
    </aside>
  )
}
