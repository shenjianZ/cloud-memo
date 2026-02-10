import { ChevronDown, ChevronRight, Folder, FolderOpen } from 'lucide-react'
import { cn } from '@/lib/utils'
import { useContextMenuStore } from '@/store/contextMenuStore'
import { useNoteStore } from '@/store/noteStore'
import { useSearchParams } from 'react-router-dom'
import { NoteItem } from '../notes/NoteItem'
import { FolderInlineInput } from './FolderInlineInput'
import { FolderInlineRename } from './FolderInlineRename'

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
  onNoteClick?: (noteId: string) => void
  isCreatingSub?: boolean  // 是否正在创建子文件夹
  creatingSubfolderForId?: string | null  // 正在创建子文件夹的文件夹 ID
  onCreateSubfolder?: (name: string, parentId: string) => Promise<void>  // 创建子文件夹回调
  onCancelCreatingSub?: () => void  // 取消创建子文件夹回调
  isRenaming?: boolean  // 是否正在重命名
  renamingFolderId?: string | null  // 正在重命名的文件夹 ID
  onStartRename?: (folderId: string) => void  // 开始重命名回调
  onUpdateFolder?: (folderId: string, newName: string) => Promise<void>  // 更新文件夹回调
  onCancelRename?: () => void  // 取消重命名回调
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
  onNoteClick,
  isCreatingSub = false,
  creatingSubfolderForId = null,
  onCreateSubfolder,
  onCancelCreatingSub,
  isRenaming = false,
  renamingFolderId = null,
  onStartRename,
  onUpdateFolder,
  onCancelRename,
}: FolderNodeProps) {
  const { showFolderContextMenu, showNoteContextMenu } = useContextMenuStore()
  const { folders, notes } = useNoteStore()
  const [searchParams] = useSearchParams()
  const hasChildren = folder.children && folder.children.length > 0

  // 获取当前活动笔记 ID
  const activeNoteId = searchParams.get('noteId')

  // 获取完整的文件夹数据（包含颜色等属性）
  const folderData = folders.find((f) => f.id === folder.id)

  // 获取该文件夹下的笔记
  const folderNotes = notes.filter((n) => n.folder === folder.id)

  const handleClick = () => {
    // 始终展开/折叠，不进行导航
    onToggle(folder.id)
  }

  const handleToggle = (e: React.MouseEvent) => {
    e.stopPropagation()
    // 箭头图标始终展开/折叠
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
      {isRenaming && renamingFolderId === folder.id ? (
        <FolderInlineRename
          folderId={folder.id}
          currentName={folder.name}
          level={level}
          onUpdate={onUpdateFolder || (async () => {})}
          onCancel={onCancelRename || (() => {})}
        />
      ) : (
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
      )}

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
                onNoteClick={onNoteClick}
                isCreatingSub={creatingSubfolderForId === child.id}
                creatingSubfolderForId={creatingSubfolderForId}
                onCreateSubfolder={onCreateSubfolder}
                onCancelCreatingSub={onCancelCreatingSub}
                isRenaming={isRenaming}
                renamingFolderId={renamingFolderId}
                onStartRename={onStartRename}
                onUpdateFolder={onUpdateFolder}
                onCancelRename={onCancelRename}
              />
            ))}

          {/* 内联输入：创建子文件夹 */}
          {isCreatingSub && onCreateSubfolder && onCancelCreatingSub && (
            <FolderInlineInput
              parentId={folder.id}
              level={level + 1}
              onCreate={(name) => onCreateSubfolder(name, folder.id)}
              onCancel={onCancelCreatingSub}
            />
          )}

          {/* 该文件夹的笔记 */}
          {folderNotes.length > 0 && (
            <div className="space-y-1 mt-1">
              {folderNotes.map((note) => (
                <NoteItem
                  key={note.id}
                  note={note}
                  level={level + 1}
                  onClick={() => onNoteClick?.(note.id)}
                  onContextMenu={(e) =>
                    showNoteContextMenu({ x: e.clientX, y: e.clientY }, note.id)
                  }
                  isActive={activeNoteId === note.id}
                  isRenaming={isRenaming}
                  onUpdateNote={onUpdateFolder}
                  onCancelRename={onCancelRename}
                />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
