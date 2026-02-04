import { ExternalLink, Star, FileEdit, Copy, Download, FolderOpen, Trash2 } from 'lucide-react'
import { ContextMenu, MenuItem, MenuSeparator } from './ContextMenu'
import { useNoteStore } from '@/store/noteStore'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'
import { tiptapJsonToMarkdown } from '@/lib/tiptapMarkdown'

interface NoteContextMenuProps {
  position: { x: number; y: number }
  isVisible: boolean
  onClose: () => void
  noteId: string | null
}

/**
 * 笔记右键菜单
 */
export function NoteContextMenu({
  position,
  isVisible,
  onClose,
  noteId,
}: NoteContextMenuProps) {
  const { getNote, updateNote, deleteNote, exportNote, favoriteNote } = useNoteStore()
  const navigate = useNavigate()

  const note = noteId ? getNote(noteId) : null

  const handleOpen = () => {
    if (!note) return
    navigate(`/editor/${note.id}`)
    onClose()
  }

  const handleOpenInNewTab = () => {
    if (!note) return
    window.open(`/editor/${note.id}`, '_blank')
    onClose()
  }

  const handleToggleFavorite = () => {
    if (!note) return

    favoriteNote(note.id)
    toast.success(note.isFavorite ? '已取消收藏' : '已添加到收藏')
    onClose()
  }

  const handleRename = () => {
    if (!note) return

    const newTitle = prompt('请输入新标题:', note.title)
    if (!newTitle || !newTitle.trim()) return

    updateNote(note.id, { title: newTitle.trim() })
      .then(() => {
        toast.success('重命名成功')
        onClose()
      })
      .catch((error) => {
        console.error('Failed to rename note:', error)
        toast.error('重命名失败')
      })
  }

  const handleCopy = async () => {
    if (!note) return

    try {
      const markdown = tiptapJsonToMarkdown(note.content)
      await navigator.clipboard.writeText(markdown)
      toast.success('已复制到剪贴板')
      onClose()
    } catch (error) {
      toast.error('复制失败')
      console.error('Copy failed:', error)
    }
  }

  const handleExport = async () => {
    if (!note) return

    try {
      await exportNote(note.id)
      onClose()
    } catch (error) {
      console.error('Export failed:', error)
    }
  }

  const handleMove = () => {
    if (!note) return

    // TODO: 显示文件夹选择对话框
    toast.info('移动功能开发中...')
    onClose()
  }

  const handleDelete = () => {
    if (!note) return

    if (!confirm(`确定要删除笔记 "${note.title}" 吗？此操作无法撤销。`)) {
      return
    }

    deleteNote(note.id)
      .then(() => {
        toast.success('笔记已删除')
        onClose()
      })
      .catch((error) => {
        console.error('Failed to delete note:', error)
        toast.error('删除失败')
      })
  }

  return (
    <ContextMenu position={position} isVisible={isVisible} onClose={onClose}>
      <MenuItem icon={<FileEdit className="w-4 h-4" />} label="打开" onClick={handleOpen} />
      <MenuItem
        icon={<ExternalLink className="w-4 h-4" />}
        label="在新标签页打开"
        onClick={handleOpenInNewTab}
      />

      <MenuSeparator />

      <MenuItem
        icon={<Star className="w-4 h-4" />}
        label={note?.isFavorite ? '取消收藏' : '收藏'}
        onClick={handleToggleFavorite}
      />
      <MenuItem icon={<FileEdit className="w-4 h-4" />} label="重命名" onClick={handleRename} />

      <MenuSeparator />

      <MenuItem icon={<Copy className="w-4 h-4" />} label="复制 Markdown" onClick={handleCopy} />
      <MenuItem icon={<Download className="w-4 h-4" />} label="导出" onClick={handleExport} />
      <MenuItem icon={<FolderOpen className="w-4 h-4" />} label="移动到..." onClick={handleMove} />

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
