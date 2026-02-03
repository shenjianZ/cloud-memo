import { useState, useEffect, useRef } from 'react'
import { X, Tag as TagIcon, Plus, Palette } from 'lucide-react'
import { useTagStore } from '@/store/tagStore'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'

const PRESET_COLORS = [
  '#ef4444', '#f97316', '#eab308', '#22c55e', '#06b6d4',
  '#3b82f6', '#8b5cf6', '#ec4899', '#f43f5e', '#64748b',
]

interface TagPopoverProps {
  noteId: string
  selectedTags: string[]
  onTagsChange: (tagIds: string[]) => void
  onClose: () => void
}

export function TagPopover({ noteId, selectedTags, onTagsChange, onClose }: TagPopoverProps) {
  const { tags, loadTags, createTag } = useTagStore()
  const [searchQuery, setSearchQuery] = useState('')
  const [isCreating, setIsCreating] = useState(false)
  const [newTagName, setNewTagName] = useState('')
  const [selectedColor, setSelectedColor] = useState(PRESET_COLORS[5])
  const [showColorPalette, setShowColorPalette] = useState(false)
  const popoverRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    loadTags()
  }, [loadTags])

  // 点击外部关闭
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (popoverRef.current && !popoverRef.current.contains(event.target as Node)) {
        onClose()
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [onClose])

  const filteredTags = tags.filter(tag =>
    tag.name.toLowerCase().includes(searchQuery.toLowerCase()) &&
    !selectedTags.includes(tag.id)
  )

  const selectedTagObjects = tags.filter(tag => selectedTags.includes(tag.id))

  const handleAddTag = (tagId: string) => {
    onTagsChange([...selectedTags, tagId])
    setSearchQuery('')
  }

  const handleRemoveTag = (tagId: string) => {
    onTagsChange(selectedTags.filter(id => id !== tagId))
  }

  const handleCreateTag = async () => {
    if (!newTagName.trim()) return

    try {
      setIsCreating(true)
      const newTag = await createTag({
        name: newTagName.trim(),
        color: selectedColor
      })
      onTagsChange([...selectedTags, newTag.id])
      setNewTagName('')
      setSearchQuery('')
      setSelectedColor(PRESET_COLORS[5])
      setShowColorPalette(false)
    } catch (error) {
      console.error('Failed to create tag:', error)
    } finally {
      setIsCreating(false)
    }
  }

  return (
    <div
      ref={popoverRef}
      className="absolute right-16 top-12 z-50 w-72 bg-popover border border-border rounded-lg shadow-lg p-3"
      onClick={(e) => e.stopPropagation()}
    >
      {/* 已选标签 */}
      {selectedTagObjects.length > 0 && (
        <div className="mb-3 pb-3 border-b border-border/50">
          <div className="text-xs text-muted-foreground mb-2">已选标签</div>
          <div className="flex flex-wrap gap-1">
            {selectedTagObjects.map(tag => (
              <div
                key={tag.id}
                className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium border"
                style={{
                  backgroundColor: tag.color ? `${tag.color}20` : undefined,
                  borderColor: tag.color || 'hsl(var(--border))',
                  color: tag.color || undefined,
                }}
              >
                <TagIcon className="w-3 h-3 flex-shrink-0" />
                <span>{tag.name}</span>
                <button
                  onClick={() => handleRemoveTag(tag.id)}
                  className="ml-0.5 hover:opacity-70"
                >
                  <X className="w-3 h-3" />
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 搜索框 */}
      <Input
        placeholder="搜索标签..."
        value={searchQuery}
        onChange={(e) => setSearchQuery(e.target.value)}
        className="mb-2 h-8 text-sm"
        autoFocus
      />

      {/* 标签列表 */}
      <div className="max-h-40 overflow-y-auto mb-2">
        {searchQuery && !filteredTags.some(t => t.name.toLowerCase() === searchQuery.toLowerCase()) && (
          <div
            className="flex items-center gap-2 px-2 py-1.5 hover:bg-muted/50 cursor-pointer rounded text-sm"
            onClick={handleCreateTag}
          >
            <Plus className="w-3 h-3" />
            <span className="text-muted-foreground">创建 "{searchQuery}"</span>
          </div>
        )}

        {filteredTags.length > 0 ? (
          filteredTags.map(tag => (
            <div
              key={tag.id}
              className="flex items-center gap-2 px-2 py-1.5 hover:bg-muted/50 cursor-pointer rounded text-sm"
              onClick={() => handleAddTag(tag.id)}
            >
              <div
                className="w-3 h-3 rounded-full flex-shrink-0"
                style={{ backgroundColor: tag.color || '#64748b' }}
              />
              <span>{tag.name}</span>
            </div>
          ))
        ) : (
          <div className="text-center text-muted-foreground text-xs py-4">
            {searchQuery ? '没有匹配的标签' : '暂无可用标签'}
          </div>
        )}
      </div>

      {/* 创建新标签 */}
      <div className="pt-2 border-t border-border/50">
        <div className="flex gap-2 mb-2">
          <Input
            placeholder="新标签名称"
            value={newTagName}
            onChange={(e) => setNewTagName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                handleCreateTag()
              }
            }}
            className="h-8 text-sm flex-1"
          />
          <Button
            variant="outline"
            size="sm"
            className="h-8 w-8 p-0 relative"
            onClick={() => setShowColorPalette(!showColorPalette)}
            title="选择颜色"
          >
            <Palette className="w-4 h-4" />
            <div
              className="absolute bottom-0 right-0 w-2 h-2 rounded-full border border-background"
              style={{ backgroundColor: selectedColor }}
            />
          </Button>
          <Button
            size="sm"
            onClick={handleCreateTag}
            disabled={isCreating || !newTagName.trim()}
            className="h-8"
          >
            <Plus className="w-4 h-4" />
          </Button>
        </div>

        {/* 颜色选择器 */}
        {showColorPalette && (
          <div className="flex gap-1 p-2 bg-muted/30 rounded-lg">
            {PRESET_COLORS.map(color => (
              <button
                key={color}
                className={cn(
                  "w-5 h-5 rounded-full border-2 transition-all hover:scale-110",
                  selectedColor === color ? "border-foreground scale-110" : "border-transparent"
                )}
                style={{ backgroundColor: color }}
                onClick={() => setSelectedColor(color)}
                title={color}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
