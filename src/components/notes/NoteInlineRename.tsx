import { useState, useEffect, useRef } from 'react'
import { Check, X, FileText } from 'lucide-react'
import { cn } from '@/lib/utils'
import { toast } from 'sonner'

interface NoteInlineRenameProps {
  noteId: string
  currentTitle: string
  level?: number
  onUpdate: (noteId: string, newTitle: string) => Promise<void>
  onCancel: () => void
}

/**
 * 笔记内联重命名组件
 * 在笔记列表中直接显示，用于重命名笔记
 */
export function NoteInlineRename({
  noteId,
  currentTitle,
  level = 0,
  onUpdate,
  onCancel,
}: NoteInlineRenameProps) {
  const [title, setTitle] = useState(currentTitle)
  const inputRef = useRef<HTMLInputElement>(null)
  const [isValid, setIsValid] = useState(true)

  // 自动聚焦并选中文本
  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.focus()
      // 全选文本，方便直接输入
      inputRef.current.select()
    }
  }, [])

  const handleSubmit = async (e?: React.FormEvent) => {
    e?.preventDefault()

    const trimmedTitle = title.trim()

    // 如果标题没有变化，直接取消
    if (trimmedTitle === currentTitle) {
      onCancel()
      return
    }

    // 验证：不能为空
    if (!trimmedTitle) {
      setIsValid(false)
      return
    }

    try {
      await onUpdate(noteId, trimmedTitle)
      toast.success('笔记重命名成功')
    } catch (error) {
      console.error('Failed to rename note:', error)
      setIsValid(false)
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    setIsValid(true)

    if (e.key === 'Enter') {
      e.preventDefault()
      handleSubmit()
    } else if (e.key === 'Escape') {
      onCancel()
    }
  }

  return (
    <div
      className={cn(
        "flex items-center gap-2 px-2 py-1.5 rounded-lg transition-colors",
        "bg-background",
        isValid ? "border border-primary" : "border border-destructive"
      )}
      style={{ paddingLeft: `${level * 12 + 8}px` }}
    >
      {/* 笔记图标 */}
      <FileText className="w-4 h-4 flex-shrink-0 text-muted-foreground" />

      {/* 输入框 */}
      <form onSubmit={handleSubmit} className="flex-1 flex items-center gap-1">
        <input
          ref={inputRef}
          type="text"
          value={title}
          onChange={(e) => {
            setTitle(e.target.value)
            setIsValid(true)
          }}
          onKeyDown={handleKeyDown}
          placeholder="笔记标题"
          className={cn(
            "flex-1 min-w-0 px-2 py-1 text-sm bg-transparent border-0 outline-none",
            "placeholder:text-muted-foreground/50",
            !isValid && "placeholder:text-destructive/50"
          )}
          autoComplete="off"
          spellCheck={false}
        />
      </form>

      {/* 确认按钮 */}
      <button
        type="button"
        onClick={() => handleSubmit()}
        className="flex-shrink-0 p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
        title="确认 (Enter)"
      >
        <Check className="w-3.5 h-3.5" />
      </button>

      {/* 取消按钮 */}
      <button
        type="button"
        onClick={onCancel}
        className="flex-shrink-0 p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
        title="取消 (Esc)"
      >
        <X className="w-3.5 h-3.5" />
      </button>
    </div>
  )
}
