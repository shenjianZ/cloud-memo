import { useState, useEffect, useRef } from 'react'
import { Check, X, Folder } from 'lucide-react'
import { cn } from '@/lib/utils'

interface FolderInlineInputProps {
  parentId: string | null
  level?: number
  onCreate: (name: string, parentId: string | null) => Promise<void>
  onCancel: () => void
}

/**
 * VSCode 风格的内联文件夹输入框
 * 在文件夹列表中直接显示，用于创建新文件夹
 */
export function FolderInlineInput({
  parentId,
  level = 0,
  onCreate,
  onCancel,
}: FolderInlineInputProps) {
  const [name, setName] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)
  const [isValid, setIsValid] = useState(true)

  // 自动聚焦输入框
  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.focus()
      // 全选文本，方便直接输入
      inputRef.current.select()
    }
  }, [])

  const handleSubmit = async (e?: React.FormEvent) => {
    e?.preventDefault()

    const trimmedName = name.trim()

    // 验证：不能为空
    if (!trimmedName) {
      setIsValid(false)
      return
    }

    // 验证：不能包含特殊字符
    if (/[<>:"/\\|?*]/.test(trimmedName)) {
      setIsValid(false)
      return
    }

    try {
      await onCreate(trimmedName, parentId)
    } catch (error) {
      console.error('Failed to create folder:', error)
      setIsValid(false)
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    setIsValid(true)

    if (e.key === 'Enter') {
      e.preventDefault() // 阻止表单默认提交行为，避免重复调用
      handleSubmit()
    } else if (e.key === 'Escape') {
      onCancel()
    }
  }

  return (
    <div
      className={cn(
        "flex items-center gap-2 px-3 py-2 rounded-lg transition-colors",
        "bg-background",
        isValid ? "border border-border" : "border border-destructive"
      )}
      style={{ paddingLeft: `${level * 12 + 12}px` }}
    >
      {/* 文件夹图标 */}
      <Folder className="w-4 h-4 flex-shrink-0" style={{ color: '#3b82f6' }} />

      {/* 输入框 */}
      <form onSubmit={handleSubmit} className="flex-1 flex items-center gap-1">
        <input
          ref={inputRef}
          type="text"
          value={name}
          onChange={(e) => {
            setName(e.target.value)
            setIsValid(true)
          }}
          onKeyDown={handleKeyDown}
          placeholder="文件夹名称"
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
