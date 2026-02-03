import { useState, useEffect } from 'react'
import { Tag as TagIcon, Plus, Edit2, Trash2, X, Check } from 'lucide-react'
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

export function TagManager() {
  const { tags, loadTags, createTag, updateTag, deleteTag, isLoading } = useTagStore()
  const [newTagName, setNewTagName] = useState('')
  const [newTagColor, setNewTagColor] = useState(PRESET_COLORS[5])
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editingName, setEditingName] = useState('')
  const [editingColor, setEditingColor] = useState('')

  useEffect(() => {
    loadTags()
  }, [loadTags])

  const handleCreate = async () => {
    if (!newTagName.trim()) return

    try {
      await createTag({ name: newTagName.trim(), color: newTagColor })
      setNewTagName('')
      setNewTagColor(PRESET_COLORS[5])
    } catch (error) {
      console.error('Failed to create tag:', error)
    }
  }

  const handleStartEdit = (tagId: string, name: string, color: string | null) => {
    setEditingId(tagId)
    setEditingName(name)
    setEditingColor(color || PRESET_COLORS[5])
  }

  const handleSaveEdit = async () => {
    if (!editingId) return

    try {
      await updateTag(editingId, { name: editingName, color: editingColor })
      setEditingId(null)
      setEditingName('')
      setEditingColor('')
    } catch (error) {
      console.error('Failed to update tag:', error)
    }
  }

  const handleCancelEdit = () => {
    setEditingId(null)
    setEditingName('')
    setEditingColor('')
  }

  const handleDelete = async (tagId: string) => {
    if (!confirm('确定要删除这个标签吗？')) return

    try {
      await deleteTag(tagId)
    } catch (error) {
      console.error('Failed to delete tag:', error)
    }
  }

  return (
    <div className="space-y-4">
      {/* 创建新标签 */}
      <div className="space-y-2">
        <h4 className="text-sm font-medium">创建新标签</h4>
        <div className="flex gap-2">
          <Input
            placeholder="标签名称"
            value={newTagName}
            onChange={(e) => setNewTagName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                handleCreate()
              }
            }}
            className="flex-1"
          />
          <div className="flex gap-1">
            {PRESET_COLORS.map(color => (
              <button
                key={color}
                className={cn(
                  "w-6 h-6 rounded border-2 transition-all",
                  newTagColor === color ? "border-foreground scale-110" : "border-transparent"
                )}
                style={{ backgroundColor: color }}
                onClick={() => setNewTagColor(color)}
              />
            ))}
          </div>
          <Button onClick={handleCreate} disabled={isLoading || !newTagName.trim()}>
            <Plus className="w-4 h-4" />
          </Button>
        </div>
      </div>

      {/* 标签列表 */}
      <div className="space-y-2">
        <h4 className="text-sm font-medium">所有标签 ({tags.length})</h4>
        <div className="space-y-1">
          {tags.length === 0 ? (
            <div className="text-center text-muted-foreground text-sm py-4">
              暂无标签
            </div>
          ) : (
            tags.map(tag => (
              <div
                key={tag.id}
                className="flex items-center gap-2 p-2 rounded-lg border bg-card"
              >
                {editingId === tag.id ? (
                  <>
                    <Input
                      value={editingName}
                      onChange={(e) => setEditingName(e.target.value)}
                      className="flex-1 h-8"
                    />
                    <div className="flex gap-1">
                      {PRESET_COLORS.map(color => (
                        <button
                          key={color}
                          className={cn(
                            "w-5 h-5 rounded border-2 transition-all",
                            editingColor === color ? "border-foreground scale-110" : "border-transparent"
                          )}
                          style={{ backgroundColor: color }}
                          onClick={() => setEditingColor(color)}
                        />
                      ))}
                    </div>
                    <Button size="sm" variant="ghost" onClick={handleSaveEdit}>
                      <Check className="w-4 h-4" />
                    </Button>
                    <Button size="sm" variant="ghost" onClick={handleCancelEdit}>
                      <X className="w-4 h-4" />
                    </Button>
                  </>
                ) : (
                  <>
                    <div
                      className="flex items-center gap-2 px-2 py-1 rounded-full text-xs font-medium border"
                      style={{
                        backgroundColor: tag.color ? `${tag.color}20` : undefined,
                        borderColor: tag.color || undefined,
                        color: tag.color || undefined,
                      }}
                    >
                      <TagIcon className="w-3 h-3" />
                      {tag.name}
                    </div>
                    <div className="ml-auto flex gap-1">
                      <Button
                        size="sm"
                        variant="ghost"
                        className="h-8 w-8 p-0"
                        onClick={() => handleStartEdit(tag.id, tag.name, tag.color)}
                      >
                        <Edit2 className="w-3 h-3" />
                      </Button>
                      <Button
                        size="sm"
                        variant="ghost"
                        className="h-8 w-8 p-0 text-destructive"
                        onClick={() => handleDelete(tag.id)}
                      >
                        <Trash2 className="w-3 h-3" />
                      </Button>
                    </div>
                  </>
                )}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  )
}
