import { useState, useEffect } from 'react'
import { useAuthStore } from '@/store/authStore'
import { Check, Trash2, Users, Plus } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { toast } from 'sonner'
import { openAuthWindow } from '@/lib/authWindow'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'

interface AccountManagerProps {
  open?: boolean
  onOpenChange?: (open: boolean) => void
}

export function AccountManager({ open: controlledOpen, onOpenChange: controlledOnOpenChange }: AccountManagerProps) {
  const { user, allAccounts, switchAccount, removeAccount, listAccounts } = useAuthStore()
  const [internalOpen, setInternalOpen] = useState(false)

  // 支持受控和非受控模式
  const open = controlledOpen !== undefined ? controlledOpen : internalOpen
  const setOpen = controlledOnOpenChange || setInternalOpen

  const [isLoading, setIsLoading] = useState(false)

  // 对话框打开时刷新账号列表
  useEffect(() => {
    if (open) {
      listAccounts()
    }
  }, [open, listAccounts])

  const handleSwitch = async (userId: string) => {
    if (userId === user?.id) return // 已经是当前账号

    setIsLoading(true)
    try {
      await switchAccount(userId)
      toast.success('账号已切换')
      setOpen(false)
    } catch (error) {
      toast.error('切换账号失败: ' + (error instanceof Error ? error.message : '未知错误'))
    } finally {
      setIsLoading(false)
    }
  }

  const handleRemove = async (userId: string, email: string) => {
    if (userId === user?.id) {
      toast.error('无法删除当前账号，请先切换到其他账号')
      return
    }

    setIsLoading(true)
    try {
      await removeAccount(userId)
      toast.success(`账号 ${email} 已删除`)
    } catch (error) {
      toast.error('删除账号失败: ' + (error instanceof Error ? error.message : '未知错误'))
    } finally {
      setIsLoading(false)
    }
  }

  const handleAddAccount = async () => {
    try {
      // 打开登录窗口
      await openAuthWindow()

      // 关闭当前对话框
      setOpen(false)

      toast.info('请在登录窗口中登录新账号')
    } catch (error) {
      console.error('打开登录窗口失败:', error)
      toast.error('打开登录窗口失败')
    }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="sm:max-w-md" onOpenChange={setOpen}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            账号管理
          </DialogTitle>
          <DialogDescription>
            管理已登录的账号，快速切换或删除
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-2 mt-4">
          {allAccounts.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              暂无已登录账号
            </div>
          ) : (
            allAccounts.map((account) => {
              const isCurrent = account.id === user?.id
              // 优先使用 profile.username，否则使用 email 前缀
              const displayName = account.profile?.username || account.email.split('@')[0] || 'User'

              // 生成用户名首字母作为头像 fallback
              const getInitials = (name: string) => {
                return name
                  .split(' ')
                  .map(n => n[0])
                  .join('')
                  .toUpperCase()
                  .slice(0, 2)
              }

              // 获取头像源（从 profile 中获取 Base64 数据）
              const getAvatarSrc = () => {
                if (account.profile?.avatarData && account.profile?.avatarMimeType) {
                  return `data:${account.profile.avatarMimeType};base64,${account.profile.avatarData}`
                }
                return undefined
              }

              return (
                <div
                  key={account.id}
                  className={`flex items-center justify-between p-3 rounded-lg border transition-colors ${
                    isCurrent
                      ? 'bg-primary/10 border-primary/30'
                      : 'bg-muted/30 hover:bg-muted/50'
                  }`}
                >
                  <div className="flex items-center gap-3 flex-1 min-w-0">
                    {/* 头像 - 使用 Avatar 组件显示真实头像或 fallback */}
                    <Avatar className="h-10 w-10">
                      <AvatarImage src={getAvatarSrc()} />
                      <AvatarFallback className={`${
                        isCurrent
                          ? 'bg-primary text-primary-foreground'
                          : 'bg-muted-foreground/20 text-muted-foreground'
                      }`}>
                        {getInitials(displayName)}
                      </AvatarFallback>
                    </Avatar>

                    {/* 账号信息 */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <p className="text-sm font-medium truncate">{displayName}</p>
                        {isCurrent && (
                          <span className="flex items-center gap-1 text-xs text-primary">
                            <Check className="h-3 w-3" />
                            当前
                          </span>
                        )}
                      </div>
                      <p className="text-xs text-muted-foreground truncate">{account.email}</p>
                    </div>
                  </div>

                  {/* 操作按钮 */}
                  {!isCurrent && (
                    <div className="flex items-center gap-1">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleSwitch(account.id)}
                        disabled={isLoading}
                        className="h-8 px-3"
                      >
                        切换
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleRemove(account.id, account.email)}
                        disabled={isLoading}
                        className="h-8 w-8 p-0 text-destructive hover:text-destructive hover:bg-destructive/10"
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  )}
                </div>
              )
            })
          )}
        </div>

        <div className="flex justify-between items-center pt-4">
          <Button
            variant="outline"
            onClick={handleAddAccount}
            className="flex items-center gap-2"
          >
            <Plus className="h-4 w-4" />
            添加账号
          </Button>
          <Button variant="outline" onClick={() => setOpen(false)}>
            关闭
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
