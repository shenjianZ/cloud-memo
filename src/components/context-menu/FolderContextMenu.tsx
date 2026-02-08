import { FileEdit, FolderPlus, Palette, Copy, Trash2, FolderOpen, Cloud } from 'lucide-react'
import { useState } from 'react'
import { ContextMenu, MenuItem, MenuSeparator } from './ContextMenu'
import { useNoteStore } from '@/store/noteStore'
import { useSyncStore } from '@/store/syncStore'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'
import { useAuthStore } from '@/store/authStore'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'

interface FolderContextMenuProps {
  position: { x: number; y: number }
  isVisible: boolean
  onClose: () => void
  folderId: string | null
  onCreateSubfolder?: (folderId: string) => void  // 新建子文件夹回调
}

/**
 * 文件夹右键菜单
 */
export function FolderContextMenu({
  position,
  isVisible,
  onClose,
  folderId,
  onCreateSubfolder,
}: FolderContextMenuProps) {
  const { folders, createNote, createFolder, deleteFolder, updateFolder, notes } = useNoteStore()
  const navigate = useNavigate()
  const { isAuthenticated } = useAuthStore()
  const { syncSingleFolder } = useSyncStore()

  const folder = folders.find((f) => f.id === folderId)

  // 删除确认对话框状态
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false)
  const [isDeleting, setIsDeleting] = useState(false)

  const handleCreateNote = async () => {
    if (!folder) return

    try {
      const newNote = await createNote({
        title: '未命名笔记',
        content: { type: 'doc', content: [{ type: 'heading', attrs: { level: 1 }, content: [{ type: 'text', text: '未命名笔记' }] }] },
        folder: folder.id,
      })
      navigate(`/editor/${newNote.id}`)
      toast.success('笔记已创建')
      onClose()
    } catch (error) {
      console.error('Failed to create note:', error)
      toast.error('创建笔记失败')
    }
  }

  const handleCreateSubfolder = () => {
    if (!folder) return

    // 使用回调触发内联输入（而不是使用 prompt）
    if (onCreateSubfolder) {
      onCreateSubfolder(folder.id)
    } else {
      // 如果没有提供回调，回退到原来的方式（保持向后兼容）
      const name = prompt('请输入文件夹名称:', '新建文件夹')
      if (!name || !name.trim()) return

      createFolder(name.trim(), folder.id)
        .then(() => {
          toast.success('文件夹已创建')
          onClose()
        })
        .catch((error) => {
          console.error('Failed to create folder:', error)
          toast.error('创建文件夹失败')
        })
    }
  }

  const handleRename = () => {
    if (!folder) return

    const newName = prompt('请输入新名称:', folder.name)
    if (!newName || !newName.trim()) return

    updateFolder(folder.id, { name: newName.trim() })
      .then(() => {
        toast.success('重命名成功')
        onClose()
      })
      .catch((error) => {
        console.error('Failed to rename folder:', error)
        toast.error('重命名失败')
      })
  }

  const handleSetColor = () => {
    if (!folder) return

    // 使用原生颜色选择器
    const input = document.createElement('input')
    input.type = 'color'
    input.value = folder.color || '#3b82f6'

    input.onchange = (e) => {
      const color = (e.target as HTMLInputElement).value
      updateFolder(folder.id, { color })
        .then(() => {
          toast.success('颜色已设置')
          onClose()
        })
        .catch((error) => {
          console.error('Failed to set color:', error)
          toast.error('设置颜色失败')
        })
    }

    input.click()
  }

  const handleCopyPath = () => {
    if (!folder) return

    // 构建文件夹路径
    const buildPath = (folderId: string, path: string[] = []): string[] => {
      const f = folders.find((fol) => fol.id === folderId)
      if (!f) return path

      const newPath = [f.name, ...path]
      if (f.parentId) {
        return buildPath(f.parentId, newPath)
      }
      return newPath
    }

    const path = buildPath(folder.id).join('/')

    navigator.clipboard
      .writeText(path)
      .then(() => {
        toast.success('路径已复制到剪贴板')
        onClose()
      })
      .catch(() => {
        toast.error('复制路径失败')
      })
  }

  const handleDelete = () => {
    if (!folder) return
    // 打开确认对话框
    setIsDeleteDialogOpen(true)
    // 关闭右键菜单
    onClose()
  }

  const confirmDelete = async () => {
    if (!folder) return

    setIsDeleting(true)
    try {
      await deleteFolder(folder.id)
      toast.success('文件夹已删除')
      setIsDeleteDialogOpen(false)
    } catch (error) {
      console.error('Failed to delete folder:', error)
      toast.error('删除文件夹失败')
    } finally {
      setIsDeleting(false)
    }
  }

  const cancelDelete = () => {
    setIsDeleteDialogOpen(false)
    setIsDeleting(false)
  }

  // 检查文件夹是否有内容
  const getFolderContentMessage = () => {
    if (!folder) return ''

    const hasNotes = notes.some((n) => n.folder === folder.id)
    const hasSubfolders = folders.some((f) => f.parentId === folder.id)

    if (hasNotes && hasSubfolders) {
      return '该文件夹包含笔记和子文件夹，删除后这些内容也将被删除。'
    } else if (hasNotes) {
      return '该文件夹包含笔记，删除后这些笔记也将被删除。'
    } else if (hasSubfolders) {
      return '该文件夹包含子文件夹，删除后这些子文件夹也将被删除。'
    }
    return ''
  }

  const handleOpen = () => {
    if (!folder) return
    navigate(`/?folder=${folder.id}`)
    onClose()
  }

  const handleSyncToCloud = async () => {
    if (!folder) return

    // 检查是否已登录
    if (!isAuthenticated) {
      toast.error('请先登录云端账户')
      return
    }

    const toastId = toast.loading('正在同步文件夹到云端...', { id: `sync-${folder.id}` })

    try {
      // 使用 syncStore 的方法（会自动刷新笔记和文件夹列表并显示详细统计）
      await syncSingleFolder(folder.id)
      // toast 已由 syncStore 管理，这里只需关闭 loading
      toast.dismiss(toastId)
    } catch (error) {
      console.error('Sync single folder failed:', error)
      toast.error('同步失败', { id: toastId })
    } finally {
      onClose()
    }
  }

  return (
    <>
      <ContextMenu position={position} isVisible={isVisible} onClose={onClose}>
        <MenuItem icon={<FolderOpen className="w-4 h-4" />} label="打开" onClick={handleOpen} />
        <MenuItem icon={<FileEdit className="w-4 h-4" />} label="新建笔记" onClick={handleCreateNote} />
        <MenuItem
          icon={<FolderPlus className="w-4 h-4" />}
          label="新建子文件夹"
          onClick={handleCreateSubfolder}
        />

        <MenuSeparator />

        <MenuItem icon={<FileEdit className="w-4 h-4" />} label="重命名" onClick={handleRename} />
        <MenuItem icon={<Palette className="w-4 h-4" />} label="设置颜色" onClick={handleSetColor} />

        <MenuSeparator />

        <MenuItem icon={<Copy className="w-4 h-4" />} label="复制路径" onClick={handleCopyPath} />

        <MenuSeparator />

        {/* 云同步功能（仅登录时显示） */}
        {isAuthenticated && (
          <MenuItem
            icon={<Cloud className="w-4 h-4" />}
            label="同步到云端"
            onClick={handleSyncToCloud}
          />
        )}

        <MenuSeparator />

        <MenuItem
          icon={<Trash2 className="w-4 h-4" />}
          label="删除"
          onClick={handleDelete}
          danger
        />
      </ContextMenu>

      {/* 删除确认对话框 */}
      <ConfirmDialog
        isOpen={isDeleteDialogOpen}
        title="删除文件夹"
        description={
          getFolderContentMessage()
            ? `确定要删除文件夹 "${folder?.name}" 吗？${getFolderContentMessage()}此操作无法撤销。`
            : `确定要删除文件夹 "${folder?.name}" 吗？此操作无法撤销。`
        }
        confirmLabel="删除"
        cancelLabel="取消"
        onConfirm={confirmDelete}
        onCancel={cancelDelete}
        isLoading={isDeleting}
      />
    </>
  )
}
