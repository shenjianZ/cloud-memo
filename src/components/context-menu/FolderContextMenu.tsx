import { FileEdit, FolderPlus, Palette, Copy, Trash2, FolderOpen } from 'lucide-react'
import { ContextMenu, MenuItem, MenuSeparator } from './ContextMenu'
import { useNoteStore } from '@/store/noteStore'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'

interface FolderContextMenuProps {
  position: { x: number; y: number }
  isVisible: boolean
  onClose: () => void
  folderId: string | null
}

/**
 * 文件夹右键菜单
 */
export function FolderContextMenu({
  position,
  isVisible,
  onClose,
  folderId,
}: FolderContextMenuProps) {
  const { folders, createNote, createFolder, deleteFolder, updateFolder } = useNoteStore()
  const navigate = useNavigate()

  const folder = folders.find((f) => f.id === folderId)

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

  const handleCreateSubfolder = async () => {
    if (!folder) return

    const name = prompt('请输入文件夹名称:', '新建文件夹')
    if (!name || !name.trim()) return

    try {
      await createFolder(name.trim(), folder.id)
      toast.success('文件夹已创建')
      onClose()
    } catch (error) {
      console.error('Failed to create folder:', error)
      toast.error('创建文件夹失败')
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

    const hasNotes = false // TODO: 检查文件夹内是否有笔记
    const hasSubfolders = folders.some((f) => f.parentId === folder.id)

    let message = '确定要删除这个文件夹吗?'
    if (hasNotes || hasSubfolders) {
      message += ' 文件夹内的内容也将被删除.'
    }

    if (!confirm(message)) {
      return
    }

    deleteFolder(folder.id)
      .then(() => {
        toast.success('文件夹已删除')
        onClose()
      })
      .catch((error) => {
        console.error('Failed to delete folder:', error)
        toast.error('删除文件夹失败')
      })
  }

  const handleOpen = () => {
    if (!folder) return
    navigate(`/?folder=${folder.id}`)
    onClose()
  }

  return (
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

      <MenuItem
        icon={<Trash2 className="w-4 h-4" />}
        label="删除"
        onClick={handleDelete}
        danger
      />
    </ContextMenu>
  )
}
