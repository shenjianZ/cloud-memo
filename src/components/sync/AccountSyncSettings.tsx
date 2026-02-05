import { useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { useAuthStore } from '@/store/authStore'
import { useSyncStore } from '@/store/syncStore'
import { toast } from 'sonner'
import { Cloud, RefreshCw, LogOut, User, CheckCircle, XCircle, AlertCircle, Shield } from 'lucide-react'
import { formatDistanceToNow } from 'date-fns'
import { zhCN } from 'date-fns/locale'

export function AccountSyncSettings() {
  const { user, isAuthenticated, logout, checkAuth } = useAuthStore()
  const {
    status,
    lastSyncAt,
    pendingCount,
    conflictCount,
    lastError,
    syncNow,
    refreshStatus,
  } = useSyncStore()

  useEffect(() => {
    checkAuth()
    refreshStatus()
  }, [])

  const handleLogout = async () => {
    try {
      await logout()
      toast.success('已登出')
    } catch (error) {
      toast.error('登出失败')
    }
  }

  const handleSync = async () => {
    await syncNow()
    await refreshStatus()
  }

  const handleOpenAuthWindow = async () => {
    try {
      // 通过事件打开登录窗口（需要在主窗口中监听）
      // 或者直接调用打开登录窗口的命令
      toast.info('请使用登录窗口登录账户')
    } catch (error) {
      toast.error('无法打开登录窗口')
    }
  }

  // 未登录状态
  if (!isAuthenticated || !user) {
    return (
      <div className="rounded-lg border p-4">
        <div className="flex items-center gap-2 mb-4">
          <Cloud className="w-4 h-4 text-muted-foreground" />
          <h3 className="font-semibold">账户与同步</h3>
        </div>

        <div className="flex flex-col items-center justify-center py-8 space-y-4">
          <Shield className="w-12 h-12 text-muted-foreground" />
          <div className="text-center">
            <p className="font-medium">未登录云端账户</p>
            <p className="text-sm text-muted-foreground mt-1">
              请使用登录窗口登录以启用云同步功能
            </p>
          </div>
          <Button onClick={handleOpenAuthWindow}>
            打开登录窗口
          </Button>
        </div>
      </div>
    )
  }

  // 已登录状态
  return (
    <div className="rounded-lg border p-4">
      <div className="flex items-center gap-2 mb-4">
        <Cloud className="w-4 h-4 text-blue-500" />
        <h3 className="font-semibold">账户与同步</h3>
      </div>

      <div className="space-y-4">
        {/* 用户信息 */}
        <div className="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-blue-500 flex items-center justify-center">
              <User className="w-5 h-5 text-white" />
            </div>
            <div>
              <p className="font-medium">{user.email}</p>
              <p className="text-sm text-muted-foreground">{user.serverUrl}</p>
            </div>
          </div>
          <Button variant="outline" size="sm" onClick={handleLogout}>
            <LogOut className="w-4 h-4 mr-2" />
            登出
          </Button>
        </div>

        <Separator />

        {/* 同步状态 */}
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div>
              <p className="font-medium">同步状态</p>
              <div className="flex items-center gap-2 mt-1">
                {status === 'syncing' && (
                  <span className="flex items-center text-sm text-blue-500">
                    <RefreshCw className="w-3 h-3 mr-1 animate-spin" />
                    同步中...
                  </span>
                )}
                {status === 'idle' && lastSyncAt && (
                  <span className="flex items-center text-sm text-green-500">
                    <CheckCircle className="w-3 h-3 mr-1" />
                    已同步于 {formatDistanceToNow(new Date(lastSyncAt), { addSuffix: true, locale: zhCN })}
                  </span>
                )}
                {status === 'error' && (
                  <span className="flex items-center text-sm text-red-500">
                    <XCircle className="w-3 h-3 mr-1" />
                    {lastError || '同步失败'}
                  </span>
                )}
                {status === 'conflict' && (
                  <span className="flex items-center text-sm text-yellow-500">
                    <AlertCircle className="w-3 h-3 mr-1" />
                    {conflictCount} 个冲突
                  </span>
                )}
              </div>
            </div>
            <Button onClick={handleSync} disabled={status === 'syncing'} size="sm">
              <RefreshCw className={`w-4 h-4 mr-2 ${status === 'syncing' ? 'animate-spin' : ''}`} />
              同步
            </Button>
          </div>

          {pendingCount > 0 && (
            <p className="text-sm text-muted-foreground">
              {pendingCount} 项更改待同步
            </p>
          )}
        </div>
      </div>
    </div>
  )
}
