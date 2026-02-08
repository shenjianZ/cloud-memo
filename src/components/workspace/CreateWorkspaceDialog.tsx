import { useState } from 'react'
import { toast } from 'sonner'
import { useWorkspaceStore } from '@/store/workspaceStore'
import { useNoteStore } from '@/store/noteStore'
import { useTagStore } from '@/store/tagStore'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'

interface CreateWorkspaceDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CreateWorkspaceDialog({ open, onOpenChange }: CreateWorkspaceDialogProps) {
  const { createWorkspace } = useWorkspaceStore()
  const noteStore = useNoteStore()
  const tagStore = useTagStore()
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (!name.trim()) {
      toast.error('请输入工作空间名称')
      return
    }

    setIsLoading(true)
    try {
      const newWorkspace = await createWorkspace({
        name: name.trim(),
        description: description.trim() || undefined,
      })
      toast.success('工作空间创建成功')

      // 重新加载所有业务数据
      console.log('[CreateWorkspaceDialog] 重新加载业务数据')

      // 1. 重新加载工作空间列表
      // 2. 清空并重新加载笔记和文件夹
      noteStore.clearNotesState()
      await noteStore.loadNotesFromStorage()

      // 3. 重新加载标签
      await tagStore.loadTags()

      // 4. 切换到新创建的工作空间
      if (newWorkspace?.id) {
        // 这里可以切换到新工作空间，或者保持当前工作空间
        console.log('[CreateWorkspaceDialog] 新工作空间已创建:', newWorkspace.id)
      }

      console.log('[CreateWorkspaceDialog] 业务数据重新加载完成')

      setName('')
      setDescription('')
      onOpenChange(false)
    } catch (error) {
      const errorMsg = typeof error === 'string' ? error : '创建工作空间失败'
      toast.error(errorMsg)
    } finally {
      setIsLoading(false)
    }
  }

  const handleOpenChange = (open: boolean) => {
    if (!open) {
      // 重置表单
      setName('')
      setDescription('')
    }
    onOpenChange(open)
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[425px]" onOpenChange={handleOpenChange}>
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>创建工作空间</DialogTitle>
            <DialogDescription>
              创建一个新的个人空间来组织您的笔记
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="name">名称</Label>
              <Input
                id="name"
                placeholder="例如：个人笔记、工作项目"
                value={name}
                onChange={(e) => setName(e.target.value)}
                disabled={isLoading}
                autoFocus
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="description">描述（可选）</Label>
              <Textarea
                id="description"
                placeholder="简要描述这个工作空间的用途"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                disabled={isLoading}
                rows={3}
              />
            </div>
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => handleOpenChange(false)}
              disabled={isLoading}
            >
              取消
            </Button>
            <Button type="submit" disabled={isLoading}>
              {isLoading ? '创建中...' : '创建'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
