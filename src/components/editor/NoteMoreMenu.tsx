import { useRef, useEffect, useState } from 'react'
import { Download, Trash2, Copy, ExternalLink } from 'lucide-react'
import { useNoteStore } from '@/store/noteStore'
import { toast } from 'sonner'
import { tiptapJsonToMarkdown } from '@/lib/tiptapSerializer'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'

interface NoteMoreMenuProps {
  noteId: string
  content: any
  onClose: () => void
}

export function NoteMoreMenu({ noteId, content, onClose }: NoteMoreMenuProps) {
  const { deleteNote, exportNote } = useNoteStore()
  const menuRef = useRef<HTMLDivElement>(null)

  // 删除确认对话框状态
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false)
  const [isDeleting, setIsDeleting] = useState(false)

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose()
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [onClose])

  const handleExport = async () => {
    try {
      await exportNote(noteId)
      onClose()
    } catch (error) {
      console.error('Export failed:', error)
    }
  }

  const handleCopy = async () => {
    try {
      const markdown = tiptapJsonToMarkdown(content)
      await navigator.clipboard.writeText(markdown)
      toast.success('已复制到剪贴板')
      onClose()
    } catch (error) {
      toast.error('复制失败')
      console.error('Copy failed:', error)
    }
  }

  const handleDelete = () => {
    // 打开确认对话框
    setIsDeleteDialogOpen(true)
    onClose()
  }

  const confirmDelete = async () => {
    setIsDeleting(true)
    try {
      await deleteNote(noteId)
      toast.success('笔记已删除')
      setIsDeleteDialogOpen(false)
      // 导航回首页
      window.location.href = '/'
    } catch (error) {
      toast.error('删除失败')
      console.error('Delete failed:', error)
    } finally {
      setIsDeleting(false)
    }
  }

  const cancelDelete = () => {
    setIsDeleteDialogOpen(false)
    setIsDeleting(false)
  }

  const handleOpenInNewTab = () => {
    const markdown = tiptapJsonToMarkdown(content)
    const blob = new Blob([markdown], { type: 'text/markdown' })
    const url = URL.createObjectURL(blob)
    window.open(url, '_blank')
    onClose()
  }

  return (
    <>
      <div
        ref={menuRef}
        className="absolute right-3 top-12 z-50 w-48 bg-popover border border-border rounded-lg shadow-lg py-1"
        onClick={(e) => e.stopPropagation()}
      >
        <div
          className="flex items-center gap-2 px-3 py-2 text-sm hover:bg-muted/50 cursor-pointer"
          onClick={handleCopy}
        >
          <Copy className="w-4 h-4" />
          <span>复制 Markdown</span>
        </div>

        <div
          className="flex items-center gap-2 px-3 py-2 text-sm hover:bg-muted/50 cursor-pointer"
          onClick={handleExport}
        >
          <Download className="w-4 h-4" />
          <span>导出</span>
        </div>

        <div
          className="flex items-center gap-2 px-3 py-2 text-sm hover:bg-muted/50 cursor-pointer"
          onClick={handleOpenInNewTab}
        >
          <ExternalLink className="w-4 h-4" />
          <span>在新标签页打开</span>
        </div>

        <div className="border-t border-border/50 my-1" />

        <div
          className="flex items-center gap-2 px-3 py-2 text-sm hover:bg-destructive/10 text-destructive cursor-pointer"
          onClick={handleDelete}
        >
          <Trash2 className="w-4 h-4" />
          <span>删除笔记</span>
        </div>
      </div>

      {/* 删除确认对话框 */}
      <ConfirmDialog
        isOpen={isDeleteDialogOpen}
        title="删除笔记"
        description="确定要删除这篇笔记吗？此操作无法撤销。"
        confirmLabel="删除"
        cancelLabel="取消"
        onConfirm={confirmDelete}
        onCancel={cancelDelete}
        isLoading={isDeleting}
      />
    </>
  )
}
