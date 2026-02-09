import { useState } from 'react'
import { toast } from 'sonner'
import { useWorkspaceStore } from '@/store/workspaceStore'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Pencil, Trash2, Star, Loader2 } from 'lucide-react'

interface ManageWorkspacesDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function ManageWorkspacesDialog({ open, onOpenChange }: ManageWorkspacesDialogProps) {
  const { workspaces, updateWorkspace, deleteWorkspace, setDefaultWorkspace, isLoading } = useWorkspaceStore()
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editingName, setEditingName] = useState('')
  const [deletingId, setDeletingId] = useState<string | null>(null)

  const handleStartEdit = (id: string, name: string) => {
    setEditingId(id)
    setEditingName(name)
  }

  const handleSaveEdit = async (id: string) => {
    if (!editingName.trim()) {
      toast.error('工作空间名称不能为空')
      return
    }

    try {
      await updateWorkspace({ id, name: editingName.trim() })
      toast.success('工作空间已重命名')
      setEditingId(null)
      setEditingName('')
    } catch (error) {
      const errorMsg = typeof error === 'string' ? error : '重命名失败'
      toast.error(errorMsg)
    }
  }

  const handleCancelEdit = () => {
    setEditingId(null)
    setEditingName('')
  }

  const handleSetDefault = async (id: string) => {
    try {
      await setDefaultWorkspace(id)
      toast.success('已设置为默认工作空间')
    } catch (error) {
      const errorMsg = typeof error === 'string' ? error : '设置失败'
      toast.error(errorMsg)
    }
  }

  const handleDelete = async (id: string) => {
    const workspace = workspaces.find(w => w.id === id)
    if (workspace?.isDefault) {
      toast.error('不允许删除默认工作空间')
      return
    }

    setDeletingId(id)
    try {
      await deleteWorkspace(id)
      toast.success('工作空间已删除')
    } catch (error) {
      const errorMsg = typeof error === 'string' ? error : '删除失败'
      toast.error(errorMsg)
    } finally {
      setDeletingId(null)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]" onOpenChange={onOpenChange}>
        <DialogHeader>
          <DialogTitle>管理工作空间</DialogTitle>
          <DialogDescription>
            重命名、删除或设置默认工作空间
          </DialogDescription>
        </DialogHeader>
        <div className="max-h-[400px] overflow-y-auto">
          {workspaces.length === 0 ? (
            <div className="py-8 text-center text-muted-foreground">
              暂无工作空间
            </div>
          ) : (
            <div className="space-y-2">
              {workspaces.map((workspace) => (
                <div
                  key={workspace.id}
                  className="flex items-center gap-2 p-3 border rounded-lg group hover:bg-accent/50"
                >
                  {editingId === workspace.id ? (
                    <>
                      <Input
                        value={editingName}
                        onChange={(e) => setEditingName(e.target.value)}
                        className="flex-1"
                        disabled={isLoading}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') {
                            handleSaveEdit(workspace.id)
                          } else if (e.key === 'Escape') {
                            handleCancelEdit()
                          }
                        }}
                        autoFocus
                      />
                      <Button
                        size="sm"
                        onClick={() => handleSaveEdit(workspace.id)}
                        disabled={isLoading}
                      >
                        保存
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={handleCancelEdit}
                        disabled={isLoading}
                      >
                        取消
                      </Button>
                    </>
                  ) : (
                    <>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="font-medium truncate">{workspace.name}</span>
                          {workspace.isDefault && (
                            <Badge variant="secondary" className="text-xs">
                              <Star className="w-3 h-3 mr-1" />
                              默认
                            </Badge>
                          )}
                          {workspace.isCurrent && (
                            <Badge variant="outline" className="text-xs">
                              当前
                            </Badge>
                          )}
                        </div>
                        {workspace.description && (
                          <p className="text-sm text-muted-foreground truncate">
                            {workspace.description}
                          </p>
                        )}
                      </div>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => handleStartEdit(workspace.id, workspace.name)}
                        disabled={isLoading}
                      >
                        <Pencil className="h-4 w-4" />
                      </Button>
                      {!workspace.isDefault && (
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={() => handleSetDefault(workspace.id)}
                          disabled={isLoading}
                          title="设为默认"
                        >
                          <Star className="h-4 w-4" />
                        </Button>
                      )}
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => handleDelete(workspace.id)}
                        disabled={isLoading || workspace.isDefault || deletingId === workspace.id}
                        className="text-destructive hover:text-destructive"
                      >
                        {deletingId === workspace.id ? (
                          <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                          <Trash2 className="h-4 w-4" />
                        )}
                      </Button>
                    </>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
