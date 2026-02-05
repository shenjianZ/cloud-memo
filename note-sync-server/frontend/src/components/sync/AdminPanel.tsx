import { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Button } from '@/components/ui/button'
import { SyncDashboard } from '@/components/sync/SyncDashboard'
import { SyncHistory } from '@/components/sync/SyncHistory'
import { ConflictResolver } from '@/components/sync/ConflictResolver'
import { DeviceManager } from '@/components/sync/DeviceManager'
import { RefreshCw, Settings, LogOut } from 'lucide-react'
import { useAuthStore } from '@/store/authStore'
import { useSyncStore } from '@/store/syncStore'
import { useDeviceStore } from '@/store/deviceStore'
import { toast } from 'react-toastify'

export default function AdminPanel() {
  const { user, logout } = useAuthStore()
  const { syncNow, status } = useSyncStore()
  const { fetchDevices } = useDeviceStore()

  const [isSyncing, setIsSyncing] = useState(false)

  useEffect(() => {
    if (user) {
      fetchDevices()
    }
  }, [user, fetchDevices])

  const handleSync = async () => {
    setIsSyncing(true)
    try {
      await syncNow()
      await fetchDevices()
      toast.success('同步完成')
    } catch (error) {
      toast.error('同步失败')
    } finally {
      setIsSyncing(false)
    }
  }

  const handleLogout = async () => {
    try {
      await logout()
      toast.success('已登出')
    } catch (error) {
      toast.error('登出失败')
    }
  }

  return (
    <div className="container min-h-screen py-8">
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Note Sync Server</h1>
          <p className="text-muted-foreground">
            欢迎回来，{user?.email}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={handleLogout}
        >
          <LogOut className="size-4 mr-2" />
          登出
        </Button>
      </div>

      <Tabs defaultValue="overview" className="space-y-4">
        <TabsList>
          <TabsTrigger value="overview">概览</TabsTrigger>
          <TabsTrigger value="history">历史</TabsTrigger>
          <TabsTrigger value="conflicts">冲突</TabsTrigger>
          <TabsTrigger value="devices">设备</TabsTrigger>
          <TabsTrigger value="settings">设置</TabsTrigger>
        </TabsList>

        <TabsContent value="overview">
          <div className="space-y-6">
            {/* 操作栏 */}
            <Card>
              <CardContent className="pt-6">
                <div className="flex items-center gap-4">
                  <Button
                    onClick={handleSync}
                    disabled={isSyncing || status === 'syncing'}
                    className="gap-2"
                  >
                    <RefreshCw className={`size-4 ${isSyncing || status === 'syncing' ? 'animate-spin' : ''}`} />
                    {isSyncing || status === 'syncing' ? '同步中...' : '立即同步'}
                  </Button>
                </div>
              </CardContent>
            </Card>

            {/* 同步仪表板 */}
            <SyncDashboard />
          </div>
        </TabsContent>

        <TabsContent value="history">
          <SyncHistory />
        </TabsContent>

        <TabsContent value="conflicts">
          <ConflictResolver />
        </TabsContent>

        <TabsContent value="devices">
          <DeviceManager />
        </TabsContent>

        <TabsContent value="settings">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Settings className="size-5" />
                服务器设置
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="rounded-lg border p-4">
                  <h3 className="font-semibold mb-4">服务器信息</h3>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">服务器地址:</span>
                      <code className="bg-muted px-2 py-1 rounded">ws://localhost:3000</code>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">状态:</span>
                      <span className="text-green-500">运行中</span>
                    </div>
                  </div>
                </div>

                <div className="rounded-lg border p-4">
                  <h3 className="font-semibold mb-4">数据库信息</h3>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">数据库:</span>
                      <code className="bg-muted px-2 py-1 rounded">MySQL 8.0</code>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">连接池:</span>
                      <code className="bg-muted px-2 py-1 rounded">10 连接</code>
                    </div>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
