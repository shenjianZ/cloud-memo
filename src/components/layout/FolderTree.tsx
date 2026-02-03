import { useState } from 'react'
import { ChevronLeft, ChevronRight, ChevronDown, Folder, FolderOpen, Plus, Inbox, Star } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useNoteStore } from '@/store/noteStore'
import { cn } from '@/lib/utils'
import { useNavigate, useSearchParams } from 'react-router-dom'

interface FolderNode {
  id: string
  name: string
  parentId: string | null
  children: FolderNode[]
}

/**
 * 文件夹树组件
 * 左侧侧边栏，显示文件夹层次结构
 */
export function FolderTree() {
  const [isCollapsed, setIsCollapsed] = useState(false)
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set())
  const { folders } = useNoteStore()
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()

  // 构建文件夹树结构
  const buildTree = (): FolderNode[] => {
    const map = new Map<string, FolderNode>()
    const roots: FolderNode[] = []

    // 初始化所有节点
    folders.forEach(folder => {
      map.set(folder.id, { ...folder, children: [] })
    })

    // 构建父子关系
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
    const isActive = searchParams.get('folder') === folder.id

    return (
      <div key={folder.id}>
        <div
          className={cn(
            "flex items-center gap-2 px-3 py-2 hover:bg-muted/50 cursor-pointer rounded-lg transition-colors",
            isActive && "bg-accent text-accent-foreground"
          )}
          style={{ paddingLeft: `${level * 16 + 12}px` }}
          onClick={() => {
            if (hasChildren) {
              toggleFolder(folder.id)
            }
            navigate(`/?folder=${folder.id}`)
          }}
        >
          {hasChildren && (
            isExpanded ? (
              <ChevronDown className="w-4 h-4 text-muted-foreground" />
            ) : (
              <ChevronRight className="w-4 h-4 text-muted-foreground" />
            )
          )}
          {isExpanded ? (
            <FolderOpen className="w-4 h-4 text-blue-500" />
          ) : (
            <Folder className="w-4 h-4 text-blue-500" />
          )}
          {!isCollapsed && (
            <span className="text-sm font-medium truncate flex-1">
              {folder.name}
            </span>
          )}
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

  return (
    <aside
      className={cn(
        "bg-card border-r border-border h-screen flex flex-col transition-all duration-300",
        isCollapsed ? "w-12" : "w-64"
      )}
    >
      {/* 头部 */}
      <div className="h-14 border-b border-border flex items-center justify-between px-3">
        {!isCollapsed && (
          <h2 className="font-semibold text-sm">文件夹</h2>
        )}
        <Button
          variant="ghost"
          size="sm"
          className="h-8 w-8 p-0"
          onClick={() => setIsCollapsed(!isCollapsed)}
        >
          {isCollapsed ? (
            <ChevronRight className="w-4 h-4" />
          ) : (
            <ChevronLeft className="w-4 h-4" />
          )}
        </Button>
      </div>

      {/* 文件夹树 */}
      <div className="flex-1 overflow-y-auto custom-scrollbar p-2">
        {!isCollapsed ? (
          <>
            {/* 全部笔记 */}
            <div
              className={cn(
                "flex items-center gap-2 px-3 py-2 hover:bg-muted/50 cursor-pointer rounded-lg mb-1",
                !searchParams.get('folder') && !searchParams.get('filter') && "bg-accent text-accent-foreground"
              )}
              onClick={() => navigate('/')}
            >
              <Inbox className="w-4 h-4 text-muted-foreground" />
              <span className="text-sm">全部笔记</span>
            </div>

            {/* 收藏 */}
            <div
              className={cn(
                "flex items-center gap-2 px-3 py-2 hover:bg-muted/50 cursor-pointer rounded-lg mb-2",
                searchParams.get('filter') === 'favorites' && "bg-accent text-accent-foreground"
              )}
              onClick={() => navigate('/?filter=favorites')}
            >
              <Star className="w-4 h-4 text-yellow-500" />
              <span className="text-sm">收藏</span>
            </div>

            {/* 文件夹树 */}
            {tree.length > 0 ? (
              tree.map(folder => renderFolder(folder))
            ) : (
              <div className="text-center text-muted-foreground text-xs py-4">
                暂无文件夹
              </div>
            )}
          </>
        ) : (
          <div className="space-y-2">
            <div
              className="h-8 flex items-center justify-center hover:bg-muted/50 rounded cursor-pointer"
              onClick={() => navigate('/')}
              title="全部笔记"
            >
              <Inbox className="w-4 h-4" />
            </div>
            <div
              className="h-8 flex items-center justify-center hover:bg-muted/50 rounded cursor-pointer"
              onClick={() => navigate('/?filter=favorites')}
              title="收藏"
            >
              <Star className="w-4 h-4 text-yellow-500" />
            </div>
          </div>
        )}
      </div>

      {/* 底部操作 */}
      {!isCollapsed && (
        <div className="p-2 border-t border-border">
          <Button
            variant="ghost"
            size="sm"
            className="w-full justify-start"
            onClick={() => {
              // TODO: 实现新建文件夹对话框
            }}
          >
            <Plus className="w-4 h-4 mr-2" />
            新建文件夹
          </Button>
        </div>
      )}
    </aside>
  )
}
