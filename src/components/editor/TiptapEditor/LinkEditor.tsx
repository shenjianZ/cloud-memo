import { useState, useEffect, useRef } from 'react'
import { Link as LinkIcon, ExternalLink, Trash2, Check, X } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

interface LinkEditorProps {
  editor: any
  onClose: () => void
  position: { x: number; y: number }
}

export function LinkEditor({ editor, onClose, position }: LinkEditorProps) {
  const [url, setUrl] = useState(editor.getAttributes('link').href || '')
  const inputRef = useRef<HTMLInputElement>(null)
  const popoverRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    // 自动聚焦并选中输入框内容
    if (inputRef.current) {
      inputRef.current.focus()
      inputRef.current.select()
    }
  }, [])

  // 点击外部关闭
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (popoverRef.current && !popoverRef.current.contains(event.target as Node)) {
        handleSave()
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [url])

  const handleSave = () => {
    if (url.trim()) {
      editor.chain().focus().setLink({ href: url.trim() }).run()
    } else {
      editor.chain().focus().unsetLink().run()
    }
    onClose()
  }

  const handleRemove = () => {
    editor.chain().focus().unsetLink().run()
    onClose()
  }

  const handleOpen = () => {
    const href = editor.getAttributes('link').href
    if (href) {
      // 使用 window.open 打开链接
      window.open(href, '_blank', 'noopener,noreferrer')
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSave()
    } else if (e.key === 'Escape') {
      onClose()
    }
  }

  return (
    <div
      ref={popoverRef}
      className="fixed z-50 bg-popover border border-border rounded-lg shadow-lg p-3 w-80"
      style={{ left: position.x, top: position.y }}
      onClick={(e) => e.stopPropagation()}
      onMouseDown={(e) => e.stopPropagation()}
    >
      <div className="space-y-3">
        {/* 标题 */}
        <div className="flex items-center gap-2 text-sm font-medium">
          <LinkIcon className="w-4 h-4" />
          <span>编辑链接</span>
        </div>

        {/* URL 输入框 */}
        <Input
          ref={inputRef}
          type="url"
          placeholder="https://example.com"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={handleKeyDown}
          className="h-9 text-sm"
        />

        {/* 操作按钮 */}
        <div className="flex items-center gap-2">
          {/* 保存按钮 */}
          <Button
            size="sm"
            onClick={handleSave}
            className="flex-1 h-8"
            disabled={!url.trim()}
          >
            <Check className="w-3 h-3 mr-1" />
            保存
          </Button>

          {/* 打开链接按钮 */}
          {url.trim() && (
            <Button
              size="sm"
              variant="outline"
              onClick={handleOpen}
              className="h-8 px-2"
              title="在浏览器中打开"
            >
              <ExternalLink className="w-3 h-3" />
            </Button>
          )}

          {/* 移除链接按钮 */}
          {url.trim() && (
            <Button
              size="sm"
              variant="outline"
              onClick={handleRemove}
              className="h-8 px-2 text-destructive hover:text-destructive"
              title="移除链接"
            >
              <Trash2 className="w-3 h-3" />
            </Button>
          )}

          {/* 取消按钮 */}
          <Button
            size="sm"
            variant="ghost"
            onClick={onClose}
            className="h-8 px-2"
          >
            <X className="w-3 h-3" />
          </Button>
        </div>

        {/* 提示文本 */}
        <div className="text-xs text-muted-foreground">
          按 Enter 保存，Esc 取消
        </div>
      </div>
    </div>
  )
}
