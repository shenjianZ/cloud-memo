import { useEffect, useState } from 'react'
import { X, Plus, Settings, Briefcase, Check } from 'lucide-react'
import { useWorkspaceStore } from '@/store/workspaceStore'
import { useNoteStore } from '@/store/noteStore'
import { useTagStore } from '@/store/tagStore'
import { CreateWorkspaceDialog } from './CreateWorkspaceDialog'
import { ManageWorkspacesDialog } from './ManageWorkspacesDialog'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

interface WorkspaceDrawerProps {
  open: boolean
  onClose: () => void
}

export function WorkspaceDrawer({ open, onClose }: WorkspaceDrawerProps) {
  const { workspaces, currentWorkspace, loadWorkspaces, switchWorkspace } = useWorkspaceStore()
  const noteStore = useNoteStore()
  const tagStore = useTagStore()
  const [createDialogOpen, setCreateDialogOpen] = useState(false)
  const [manageDialogOpen, setManageDialogOpen] = useState(false)

  // 加载工作空间列表
  useEffect(() => {
    if (open) {
      loadWorkspaces()
    }
  }, [open, loadWorkspaces])

  // 切换工作空间
  const handleSwitchWorkspace = async (id: string) => {
    try {
      await switchWorkspace(id)
      toast.success('工作空间已切换')

      // 重新加载所有业务数据
      console.log('[WorkspaceDrawer] 重新加载业务数据')

      // 1. 清空并重新加载笔记和文件夹
      noteStore.clearNotesState()
      await noteStore.loadNotesFromStorage()

      // 2. 重新加载标签
      await tagStore.loadTags()

      console.log('[WorkspaceDrawer] 业务数据重新加载完成')

      onClose() // 切换后关闭抽屉
    } catch (error) {
      const errorMsg = typeof error === 'string' ? error : '切换工作空间失败'
      toast.error(errorMsg)
    }
  }

  if (!open) return null

  return (
    <>
      {/* 遮罩层 */}
      <div
        className="fixed inset-0 bg-black/50 z-50 transition-opacity"
        onClick={onClose}
      />

      {/* 右侧抽屉 */}
      <div className="fixed right-0 top-0 h-full w-80 bg-background border-l shadow-lg z-50 flex flex-col animate-slide-in-from-right">
        {/* 头部 */}
        <div className="flex items-center justify-between p-4 border-b">
          <div className="flex items-center gap-2">
            <Briefcase className="h-5 w-5" />
            <h2 className="text-lg font-semibold">工作空间</h2>
          </div>
          <Button
            variant="ghost"
            size="icon"
            onClick={onClose}
            className="h-8 w-8"
          >
            <X className="h-4 w-4" />
          </Button>
        </div>

        {/* 内容区域 */}
        <div className="flex-1 overflow-y-auto p-4">
          {/* 当前工作空间 */}
          {currentWorkspace && (
            <div className="mb-6">
              <p className="text-sm text-muted-foreground mb-2">当前工作空间</p>
              <div className="flex items-center justify-between p-3 rounded-lg border bg-accent/50">
                <div className="flex items-center gap-2 flex-1 min-w-0">
                  {currentWorkspace.icon && (
                    <span className="text-xl">{currentWorkspace.icon}</span>
                  )}
                  <span className="font-medium truncate">{currentWorkspace.name}</span>
                </div>
                <Check className="h-4 w-4 text-primary shrink-0" />
              </div>
            </div>
          )}

          {/* 工作空间列表 */}
          <div className="mb-6">
            <p className="text-sm text-muted-foreground mb-2">
              所有工作空间 ({workspaces.length})
            </p>
            {workspaces.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                暂无工作空间
              </div>
            ) : (
              <div className="space-y-1">
                {workspaces.map((workspace) => (
                  <button
                    key={workspace.id}
                    onClick={() => handleSwitchWorkspace(workspace.id)}
                    className={cn(
                      "w-full flex items-center justify-between p-3 rounded-lg border transition-colors hover:bg-accent/50 text-left",
                      workspace.isCurrent && "bg-accent border-primary"
                    )}
                  >
                    <div className="flex items-center gap-2 flex-1 min-w-0">
                      {workspace.icon && (
                        <span className="text-xl shrink-0">{workspace.icon}</span>
                      )}
                      <div className="flex-1 min-w-0">
                        <div className="font-medium truncate">{workspace.name}</div>
                        {workspace.description && (
                          <div className="text-xs text-muted-foreground truncate">
                            {workspace.description}
                          </div>
                        )}
                      </div>
                    </div>
                    {workspace.isCurrent && (
                      <Check className="h-4 w-4 text-primary shrink-0 ml-2" />
                    )}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* 操作按钮 */}
          <div className="space-y-2">
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => setCreateDialogOpen(true)}
            >
              <Plus className="mr-2 h-4 w-4" />
              创建工作空间
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => setManageDialogOpen(true)}
            >
              <Settings className="mr-2 h-4 w-4" />
              管理工作空间
            </Button>
          </div>
        </div>
      </div>

      {/* 对话框 */}
      <CreateWorkspaceDialog
        open={createDialogOpen}
        onOpenChange={setCreateDialogOpen}
      />

      <ManageWorkspacesDialog
        open={manageDialogOpen}
        onOpenChange={setManageDialogOpen}
      />
    </>
  )
}
