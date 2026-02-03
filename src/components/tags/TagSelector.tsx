import { useState, useEffect } from 'react'
import { X, Tag as TagIcon, Plus, Palette } from 'lucide-react'
import { useTagStore } from '@/store/tagStore'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'

const PRESET_COLORS = [
  '#ef4444', // red
  '#f97316', // orange
  '#eab308', // yellow
  '#22c55e', // green
  '#06b6d4', // cyan
  '#3b82f6', // blue
  '#8b5cf6', // violet
  '#ec4899', // pink
  '#f43f5e', // rose
  '#64748b', // slate
]

interface TagSelectorProps {
  noteId: string
  selectedTags: string[]
  onTagsChange: (tagIds: string[]) => void
  showColorPicker?: boolean
}

export function TagSelector({ noteId, selectedTags, onTagsChange, showColorPicker = true }: TagSelectorProps) {
  const { tags, loadTags, createTag } = useTagStore()
  const [isOpen, setIsOpen] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [isCreating, setIsCreating] = useState(false)
  const [newTagName, setNewTagName] = useState('')
  const [selectedColor, setSelectedColor] = useState(PRESET_COLORS[5])
  const [showColorPalette, setShowColorPalette] = useState(false)

  useEffect(() => {
    loadTags()
  }, [loadTags])

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
        color: showColorPicker ? selectedColor : undefined
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
    <div className="relative">
      {/* 已选标签 */}
      <div className="flex flex-wrap gap-2 mb-2">
        {selectedTagObjects.map(tag => (
          <TagBadge
            key={tag.id}
            tag={tag}
            onRemove={() => handleRemoveTag(tag.id)}
          />
        ))}
        <Button
          variant="outline"
          size="sm"
          className="h-6 px-2 text-xs"
          onClick={() => setIsOpen(!isOpen)}
        >
          <Plus className="w-3 h-3 mr-1" />
          添加标签
        </Button>
      </div>

      {/* 下拉菜单 */}
      {isOpen && (
        <div className="absolute z-10 w-full min-w-[200px] bg-background border border-border rounded-lg shadow-lg p-2">
          {/* 搜索框 */}
          <Input
            placeholder="搜索或创建标签..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="mb-2 h-8 text-sm"
            autoFocus
          />

          {/* 标签列表 */}
          <div className="max-h-48 overflow-y-auto">
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
                  <TagIcon className="w-3 h-3 flex-shrink-0" style={{ color: tag.color || undefined }} />
                  <span>{tag.name}</span>
                </div>
              ))
            ) : (
              <div className="text-center text-muted-foreground text-xs py-4">
                {searchQuery ? '没有匹配的标签' : '暂无标签'}
              </div>
            )}
          </div>

          {/* 创建新标签 */}
          <div className="mt-2 pt-2 border-t border-border/50">
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
              {showColorPicker && (
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
              )}
              <Button
                size="sm"
                onClick={handleCreateTag}
                disabled={isCreating || !newTagName.trim()}
                className="h-8"
              >
                创建
              </Button>
            </div>

            {/* 颜色选择器 */}
            {showColorPicker && showColorPalette && (
              <div className="flex gap-1 p-2 bg-muted/30 rounded-lg">
                {PRESET_COLORS.map(color => (
                  <button
                    key={color}
                    className={cn(
                      "w-6 h-6 rounded-full border-2 transition-all hover:scale-110",
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
      )}
    </div>
  )
}

interface TagBadgeProps {
  tag: {
    id: string
    name: string
    color: string | null
  }
  onRemove?: () => void
  onClick?: () => void
}

export function TagBadge({ tag, onRemove, onClick }: TagBadgeProps) {
  const tagStyle = tag.color ? {
    backgroundColor: `${tag.color}20`,
    borderColor: tag.color,
    color: tag.color,
  } : {}

  return (
    <div
      className={cn(
        "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium border transition-colors",
        onClick && "cursor-pointer hover:opacity-80"
      )}
      style={tagStyle}
      onClick={onClick}
    >
      <TagIcon className="w-3 h-3 flex-shrink-0" />
      <span>{tag.name}</span>
      {onRemove && (
        <button
          onClick={(e) => {
            e.stopPropagation()
            onRemove()
          }}
          className="ml-0.5 hover:opacity-70"
        >
          <X className="w-3 h-3" />
        </button>
      )}
    </div>
  )
}
