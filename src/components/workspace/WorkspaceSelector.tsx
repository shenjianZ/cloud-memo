import { useEffect, useState } from 'react'
import { ChevronsUpDown, Plus, Settings, Check } from 'lucide-react'
import { useWorkspaceStore } from '@/store/workspaceStore'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { CreateWorkspaceDialog } from './CreateWorkspaceDialog'
import { ManageWorkspacesDialog } from './ManageWorkspacesDialog'
import { toast } from 'sonner'

export function WorkspaceSelector() {
  const { workspaces, currentWorkspace, loadWorkspaces, switchWorkspace, isLoading } = useWorkspaceStore()
  const [createDialogOpen, setCreateDialogOpen] = useState(false)
  const [manageDialogOpen, setManageDialogOpen] = useState(false)

  useEffect(() => {
    loadWorkspaces()
  }, [loadWorkspaces])

  const handleSwitchWorkspace = async (id: string) => {
    try {
      await switchWorkspace(id)
      toast.success('工作空间已切换')
      // 重新加载笔记列表
      window.location.reload()
    } catch (error) {
      const errorMsg = typeof error === 'string' ? error : '切换工作空间失败'
      toast.error(errorMsg)
    }
  }

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="outline"
            role="combobox"
            className="h-8 w-[200px] justify-between"
            disabled={isLoading}
          >
            {currentWorkspace ? (
              <span className="truncate">{currentWorkspace.name}</span>
            ) : (
              <span className="text-muted-foreground">选择工作空间</span>
            )}
            <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-[200px]">
          <DropdownMenuLabel>工作空间</DropdownMenuLabel>
          <DropdownMenuSeparator />
          {workspaces.length === 0 ? (
            <div className="px-2 py-1.5 text-sm text-muted-foreground">
              暂无工作空间
            </div>
          ) : (
            workspaces.map((workspace) => (
              <DropdownMenuItem
                key={workspace.id}
                onClick={() => handleSwitchWorkspace(workspace.id)}
                className="flex items-center justify-between"
              >
                <span className="truncate">{workspace.name}</span>
                {workspace.isCurrent && (
                  <Check className="h-4 w-4 text-primary" />
                )}
                {workspace.isDefault && !workspace.isCurrent && (
                  <span className="text-xs text-muted-foreground">默认</span>
                )}
              </DropdownMenuItem>
            ))
          )}
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={() => setCreateDialogOpen(true)}>
            <Plus className="mr-2 h-4 w-4" />
            创建工作空间
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => setManageDialogOpen(true)}>
            <Settings className="mr-2 h-4 w-4" />
            管理工作空间
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

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
