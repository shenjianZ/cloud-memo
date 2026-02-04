import { ChevronDown, ChevronRight, Folder, FolderOpen } from 'lucide-react'
import { cn } from '@/lib/utils'
import { useContextMenuStore } from '@/store/contextMenuStore'
import { useNoteStore } from '@/store/noteStore'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { NoteItem } from '../notes/NoteItem'

interface FolderTreeNode {
  id: string
  name: string
  parentId: string | null
  children?: FolderTreeNode[]
}

interface FolderNodeProps {
  folder: FolderTreeNode
  level?: number
  isActive?: boolean
  isExpanded?: boolean
  expandedFolders: Set<string>
  onToggle: (folderId: string) => void
  onClick: (folderId: string) => void
}

/**
 * 文件夹节点组件（递归）
 * 包含：文件夹 + 子文件夹 + 该文件夹的笔记
 */
export function FolderNode({
  folder,
  level = 0,
  isActive,
  isExpanded,
  expandedFolders,
  onToggle,
  onClick,
}: FolderNodeProps) {
  const { showFolderContextMenu, showNoteContextMenu } = useContextMenuStore()
  const { folders, notes } = useNoteStore()
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const hasChildren = folder.children && folder.children.length > 0

  // 获取当前活动笔记 ID
  const activeNoteId = searchParams.get('noteId')

  // 获取完整的文件夹数据（包含颜色等属性）
  const folderData = folders.find((f) => f.id === folder.id)

  // 获取该文件夹下的笔记
  const folderNotes = notes.filter((n) => n.folder === folder.id)

  const handleClick = () => {
    onClick(folder.id)
  }

  const handleToggle = (e: React.MouseEvent) => {
    e.stopPropagation()
    onToggle(folder.id)
  }

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault()
    e.stopPropagation()
    showFolderContextMenu({ x: e.clientX, y: e.clientY }, folder.id)
  }

  return (
    <div>
      {/* 文件夹行 */}
      <div
        className={cn(
          "flex items-center gap-2 px-3 py-2 hover:bg-muted/50 cursor-pointer rounded-lg transition-colors text-sm group",
          isActive && "bg-accent text-accent-foreground"
        )}
        style={{ paddingLeft: `${level * 12 + 12}px` }}
        onClick={handleClick}
        onContextMenu={handleContextMenu}
      >
        {/* 展开/折叠箭头 */}
        <div
          className="flex-shrink-0 hover:bg-muted rounded p-0.5"
          onClick={handleToggle}
        >
          {isExpanded || (hasChildren && expandedFolders.has(folder.id)) ? (
            <ChevronDown className="w-3 h-3 text-muted-foreground" />
          ) : (
            <ChevronRight className="w-3 h-3 text-muted-foreground" />
          )}
        </div>

        {/* 文件夹图标 */}
        {(isExpanded || expandedFolders.has(folder.id)) ? (
          <FolderOpen
            className="w-4 h-4 flex-shrink-0"
            style={{ color: folderData?.color || '#3b82f6' }}
          />
        ) : (
          <Folder
            className="w-4 h-4 flex-shrink-0"
            style={{ color: folderData?.color || '#3b82f6' }}
          />
        )}

        {/* 文件夹名称 */}
        <span className="font-medium truncate flex-1">{folder.name}</span>
      </div>

      {/* 子文件夹和笔记 */}
      {(isExpanded || expandedFolders.has(folder.id)) && (
        <div>
          {/* 子文件夹 */}
          {hasChildren &&
            folder.children!.map((child) => (
              <FolderNode
                key={child.id}
                folder={child}
                level={level + 1}
                isActive={isActive}
                isExpanded={expandedFolders.has(child.id)}
                expandedFolders={expandedFolders}
                onToggle={onToggle}
                onClick={onClick}
              />
            ))}

          {/* 该文件夹的笔记 */}
          {folderNotes.length > 0 && (
            <div className="space-y-1 mt-1">
              {folderNotes.map((note) => (
                <NoteItem
                  key={note.id}
                  note={note}
                  level={level + 1}
                  onClick={() => navigate(`/editor/${note.id}`)}
                  onContextMenu={(e) =>
                    showNoteContextMenu({ x: e.clientX, y: e.clientY }, note.id)
                  }
                  isActive={activeNoteId === note.id}
                />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
