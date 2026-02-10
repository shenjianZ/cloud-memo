import { useState, useCallback, useRef, useEffect, useMemo } from 'react'
import { Search, Settings, PanelLeftClose, PanelLeftOpen, Plus, FolderPlus, Home } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useNoteStore } from '@/store/noteStore'
import { useEditorStore } from '@/store/editorStore'
import { useSidebarStore } from '@/store/sidebarStore'
import { useSearchStore } from '@/store/searchStore'
import { useContextMenuStore } from '@/store/contextMenuStore'
import { cn } from '@/lib/utils'
import { useNavigate, useSearchParams, useLocation } from 'react-router-dom'
import { NoteItem } from '../notes/NoteItem'
import { FolderNode } from './FolderNode'
import { FolderContextMenu, NoteContextMenu } from '@/components/context-menu'
import { FolderInlineInput } from './FolderInlineInput'
import { updateTitleInContent } from '@/lib/noteHelpers'

interface FolderTree {
  id: string
  name: string
  parentId: string | null
  children: FolderTree[]
}

/**
 * 合并的侧边栏组件
 * 包含：顶部工具栏 + 文件夹树 + 笔记列表
 */
export function Sidebar() {
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set())
  const { folders, notes, createNote, createFolder, loadNotesFromStorage, updateFolder, updateNote } = useNoteStore()
  const { openNote } = useEditorStore()
  const { isCollapsed, toggleSidebar } = useSidebarStore()
  const openSearch = useSearchStore((state) => state.openSearch)

  // 重命名状态
  const [isRenaming, setIsRenaming] = useState(false)
  const [renamingFolderId, setRenamingFolderId] = useState<string | null>(null)
  const [renamingNoteId, setRenamingNoteId] = useState<string | null>(null)
  const {
    folderContextMenu,
    noteContextMenu,
    hideFolderContextMenu,
    hideNoteContextMenu,
    showNoteContextMenu,
  } = useContextMenuStore()
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const location = useLocation()
  const [isLoading, setIsLoading] = useState(false)
  const isCreatingRef = useRef(false) // 使用 ref 防止重复创建
  const [currentPath, setCurrentPath] = useState(window.location.hash.slice(1) || '/')

  // 内联输入状态
  const [isCreatingRoot, setIsCreatingRoot] = useState(false)
  const [creatingSubfolderForId, setCreatingSubfolderForId] = useState<string | null>(null)

  // 获取当前活动笔记 ID - 从路由路径中提取
  const activeNoteId = useMemo(() => {
    // 匹配 /editor/:noteId 格式的路由
    const match = location.pathname.match(/^\/editor\/([^/]+)$/)
    return match ? match[1] : null
  }, [location.pathname])

  // 获取当前路径
  const folderFilter = searchParams.get('folder')
  const isHomePage = currentPath === '/'

  // 加载笔记数据
  useEffect(() => {
    // 清除旧的 localStorage 文件夹数据（迁移到数据库后不再需要）
    const storage = localStorage.getItem('markdown-notes-storage')
    if (storage) {
      try {
        const parsed = JSON.parse(storage)
        if (parsed.state && parsed.state.folders) {
          delete parsed.state.folders
          localStorage.setItem('markdown-notes-storage', JSON.stringify(parsed))
        }
      } catch (e) {
        console.error('Failed to clear old folders from localStorage:', e)
      }
    }

    loadNotesFromStorage()
  }, [loadNotesFromStorage])

  // 监听路由变化以更新高亮状态
  useEffect(() => {
    const handleHashChange = () => {
      setCurrentPath(window.location.hash.slice(1) || '/')
    }

    window.addEventListener('hashchange', handleHashChange)

    return () => {
      window.removeEventListener('hashchange', handleHashChange)
    }
  }, [])

  // 当打开笔记时，自动展开其所在文件夹
  useEffect(() => {
    if (activeNoteId) {
      const note = notes.find(n => n.id === activeNoteId)
      if (note?.folder) {
        // 展开笔记所在的文件夹
        setExpandedFolders(prev => new Set([...prev, note.folder!]))
        // 递归展开所有祖先文件夹
        const expandAncestors = (folderId: string) => {
          const folder = folders.find(f => f.id === folderId)
          if (folder?.parentId) {
            setExpandedFolders(prev => new Set([...prev, folder.parentId!]))
            expandAncestors(folder.parentId!)
          }
        }
        expandAncestors(note.folder)
      }
    }
  }, [activeNoteId, notes, folders])

  // 构建文件夹树结构
  const buildTree = (): FolderTree[] => {
    const map = new Map<string, FolderTree>()
    const roots: FolderTree[] = []

    // 第一步：创建所有文件夹节点
    folders.forEach(folder => {
      map.set(folder.id, { id: folder.id, name: folder.name, parentId: folder.parentId, children: [] })
    })

    // 第二步：构建父子关系
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

  const renderFolder = (folder: FolderTree, level: number = 0) => {
    const isExpanded = expandedFolders.has(folder.id)
    const isActive = folderFilter === folder.id
    const isCreatingSub = creatingSubfolderForId === folder.id

    return (
      <FolderNode
        key={folder.id}
        folder={folder}
        level={level}
        isActive={isActive}
        isExpanded={isExpanded}
        expandedFolders={expandedFolders}
        onToggle={toggleFolder}
        onClick={(folderId) => {
          navigate(`/?folder=${folderId}`)
        }}
        onNoteClick={handleNoteClick}
        isCreatingSub={isCreatingSub}
        creatingSubfolderForId={creatingSubfolderForId}
        onCreateSubfolder={handleCreateSubfolder}
        onCancelCreatingSub={() => setCreatingSubfolderForId(null)}
        isRenaming={isRenaming}
        renamingFolderId={renamingFolderId}
        onStartRename={handleStartRenameFolder}
        onUpdateFolder={handleUpdateFolder}
        onCancelRename={handleCancelRename}
      />
    )
  }

  const tree = buildTree()

  // 按更新时间排序（基于所有笔记）
  const sortedNotes = [...notes].sort(
    (a, b) => b.updatedAt - a.updatedAt
  )

  // 创建根文件夹（内联输入）
  const handleCreateRootFolder = async (name: string) => {
    try {
      await createFolder(name)
      setIsCreatingRoot(false)
    } catch (error) {
      console.error('Failed to create folder:', error)
      throw error
    }
  }

  // 创建子文件夹（内联输入）
  const handleCreateSubfolder = async (name: string, parentId: string) => {
    try {
      await createFolder(name, parentId)
      setCreatingSubfolderForId(null)
      // 自动展开父文件夹
      setExpandedFolders(prev => new Set([...prev, parentId]))
    } catch (error) {
      console.error('Failed to create subfolder:', error)
      throw error
    }
  }

  // 开始创建子文件夹（由右键菜单触发）
  const startCreatingSubfolder = useCallback((folderId: string) => {
    setCreatingSubfolderForId(folderId)
    // 自动展开父文件夹以显示输入框
    setExpandedFolders(prev => new Set([...prev, folderId]))
    // 隐藏右键菜单
    hideFolderContextMenu()
  }, [hideFolderContextMenu])

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
        content: {
          type: 'doc',
          content: [
            {
              type: 'heading',
              attrs: { level: 1 },
              content: [{ type: 'text', text: '未命名笔记' }],
            },
          ],
        },
      })
      // 打开 Tab 并导航
      openNote(newNote.id, newNote.title || '未命名笔记')
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
  }, [isLoading, createNote, navigate, openNote])

  // 处理笔记点击 - 打开新 Tab 并导航
  const handleNoteClick = useCallback(
    (noteId: string) => {
      const note = notes.find((n) => n.id === noteId)
      if (note) {
        openNote(noteId, note.title || '未命名笔记')
        navigate(`/editor/${noteId}`)
      }
    },
    [notes, openNote, navigate],
  )

  // 开始重命名文件夹
  const handleStartRenameFolder = useCallback((folderId: string) => {
    setRenamingFolderId(folderId)
    setRenamingNoteId(null)
    setIsRenaming(true)
    hideFolderContextMenu()
  }, [hideFolderContextMenu])

  // 更新文件夹名称
  const handleUpdateFolder = useCallback(async (folderId: string, newName: string) => {
    try {
      await updateFolder(folderId, { name: newName })
      setIsRenaming(false)
      setRenamingFolderId(null)
    } catch (error) {
      console.error('Failed to rename folder:', error)
      throw error
    }
  }, [updateFolder])

  // 开始重命名笔记
  const handleStartRenameNote = useCallback((noteId: string) => {
    setRenamingNoteId(noteId)
    setRenamingFolderId(null)
    setIsRenaming(true)
    hideNoteContextMenu()
  }, [hideNoteContextMenu])

  // 更新笔记标题
  const handleUpdateNote = useCallback(async (noteId: string, newTitle: string) => {
    try {
      // 获取当前笔记
      const note = notes.find(n => n.id === noteId)
      if (!note) {
        throw new Error('Note not found')
      }

      // 更新内容中的标题（同步更新 Tiptap JSON 中的 H1）
      const updatedContent = updateTitleInContent(note.content, newTitle)

      // 同时更新标题和内容
      await updateNote(noteId, {
        title: newTitle,
        content: updatedContent
      })

      setIsRenaming(false)
      setRenamingNoteId(null)
    } catch (error) {
      console.error('Failed to rename note:', error)
      throw error
    }
  }, [updateNote, notes])

  // 取消重命名
  const handleCancelRename = useCallback(() => {
    setIsRenaming(false)
    setRenamingFolderId(null)
    setRenamingNoteId(null)
  }, [])

  return (
    <aside
      className={cn(
        "bg-muted/30 border-r border-border h-screen flex flex-col transition-all duration-300",
        isCollapsed ? "w-12" : "w-80"
      )}
    >
      {/* 顶部工具栏 */}
      <div className="h-14 border-b border-border flex items-center justify-between px-3">
        {!isCollapsed ? (
          <>
            <Button
              variant="ghost"
              size="sm"
              className="h-9 w-9 p-0"
              onClick={openSearch}
              title="搜索笔记 (Ctrl+K)"
            >
              <Search className="w-4 h-4" />
            </Button>

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
                onClick={() => toggleSidebar()}
                title="折叠侧边栏"
              >
                <PanelLeftClose className="w-4 h-4" />
              </Button>
            </div>
          </>
        ) : (
          <div className="flex items-center justify-center w-full">
            <Button
              variant="ghost"
              size="sm"
              className="h-8 w-8 p-0"
              onClick={() => toggleSidebar()}
              title="展开侧边栏"
            >
              <PanelLeftOpen className="w-4 h-4" />
            </Button>
          </div>
        )}
      </div>

      {/* 内容区域 */}
      <div className="flex-1 overflow-y-auto custom-scrollbar">
        {!isCollapsed && (
          <>
            {/* 首页入口 */}
            <div className="px-2 py-2 border-b border-border/50">
              <div
                className={cn(
                  "flex items-center gap-2 px-3 py-2 hover:bg-muted/50 cursor-pointer rounded-lg transition-colors text-sm",
                  isHomePage && "bg-accent text-accent-foreground"
                )}
                onClick={() => navigate('/')}
              >
                <Home className="w-4 h-4 text-blue-500" />
                <span className="font-medium">首页</span>
              </div>
            </div>

            {/* 笔记列表（包含文件夹和笔记） */}
            <div className="px-2 py-2">
              <div className="flex items-center justify-between mb-2 px-1">
                <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
                  笔记列表
                </span>
                <div className="flex items-center gap-1">
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
                    type="button"
                  >
                    <Plus className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 w-7 p-0"
                    onClick={(e) => {
                      e.preventDefault()
                      e.stopPropagation()
                      setIsCreatingRoot(true)
                    }}
                    title="新建文件夹"
                    type="button"
                  >
                    <FolderPlus className="w-4 h-4" />
                  </Button>
                </div>
              </div>

              {/* 文件夹树（包含文件夹和笔记） */}
              {tree.length > 0 ? (
                <div className="space-y-1">
                  {tree.map((folder) => renderFolder(folder))}

                  {/* 根文件夹内联输入 */}
                  {isCreatingRoot && (
                    <FolderInlineInput
                      parentId={null}
                      onCreate={handleCreateRootFolder}
                      onCancel={() => setIsCreatingRoot(false)}
                    />
                  )}
                </div>
              ) : (
                <>
                  <div className="text-center text-muted-foreground text-xs py-4">
                    暂无文件夹
                  </div>

                  {/* 根文件夹内联输入（无文件夹时也显示） */}
                  {isCreatingRoot && (
                    <FolderInlineInput
                      parentId={null}
                      onCreate={handleCreateRootFolder}
                      onCancel={() => setIsCreatingRoot(false)}
                    />
                  )}
                </>
              )}

              {/* 根目录笔记（不在任何文件夹中的笔记） */}
              {(() => {
                const rootNotes = sortedNotes.filter((n) => !n.folder)
                if (rootNotes.length === 0) return null

                return (
                  <div className="mt-2 space-y-1">
                    {rootNotes.map((note) => (
                      <NoteItem
                        key={note.id}
                        note={note}
                        level={0}
                        onClick={() => handleNoteClick(note.id)}
                        onContextMenu={(e) =>
                          showNoteContextMenu({ x: e.clientX, y: e.clientY }, note.id)
                        }
                        isActive={activeNoteId === note.id}
                        isRenaming={isRenaming && renamingNoteId === note.id}
                        onUpdateNote={handleUpdateNote}
                        onCancelRename={handleCancelRename}
                      />
                    ))}
                  </div>
                )
              })()}
            </div>
          </>
        )}
      </div>

      {/* 右键菜单容器 */}
      <FolderContextMenu
        position={folderContextMenu.position}
        isVisible={folderContextMenu.isVisible}
        onClose={hideFolderContextMenu}
        folderId={folderContextMenu.folderId}
        onCreateSubfolder={startCreatingSubfolder}
        onStartRename={handleStartRenameFolder}
      />
      <NoteContextMenu
        position={noteContextMenu.position}
        isVisible={noteContextMenu.isVisible}
        onClose={hideNoteContextMenu}
        noteId={noteContextMenu.noteId}
        onStartRename={handleStartRenameNote}
      />
    </aside>
  )
}
