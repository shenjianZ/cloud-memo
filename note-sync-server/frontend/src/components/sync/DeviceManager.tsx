import { useEffect, useState } from 'react'
import { Laptop, Smartphone, Monitor, Tablet, Trash2, RefreshCw } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { useDeviceStore } from '@/store/deviceStore'
import { toast } from 'react-toastify'
import type { Device } from '@/api/devices'

export function DeviceManager() {
  const { devices, isLoading, error, fetchDevices, revokeDevice, clearError } = useDeviceStore()
  const [currentDeviceId, setCurrentDeviceId] = useState<string | null>(null)

  useEffect(() => {
    loadDevices()
  }, [])

  // 从 localStorage 获取当前设备 ID
  useEffect(() => {
    const savedDeviceId = localStorage.getItem('current_device_id')
    if (savedDeviceId) {
      setCurrentDeviceId(savedDeviceId)
    }
  }, [])

  const loadDevices = async () => {
    try {
      await fetchDevices()
    } catch (err) {
      toast.error('获取设备列表失败')
    }
  }

  const formatTimeAgo = (timestamp: number) => {
    const seconds = Math.floor((Date.now() - timestamp) / 1000)
    if (seconds < 60) return '刚刚活跃'
    const minutes = Math.floor(seconds / 60)
    if (minutes < 60) return `${minutes}分钟前活跃`
    const hours = Math.floor(minutes / 60)
    if (hours < 24) return `${hours}小时前活跃`
    const days = Math.floor(hours / 24)
    return `${days}天前活跃`
  }

  const getDeviceIcon = (deviceType: Device['device_type']) => {
    switch (deviceType) {
      case 'desktop':
        return <Monitor className="size-4" />
      case 'laptop':
        return <Laptop className="size-4" />
      case 'tablet':
        return <Tablet className="size-4" />
      case 'mobile':
        return <Smartphone className="size-4" />
      default:
        return <Monitor className="size-4" />
    }
  }

  const handleRefresh = async () => {
    clearError()
    await loadDevices()
  }

  const handleRevoke = async (deviceId: string) => {
    if (!confirm('确定要登出此设备吗？')) return

    try {
      await revokeDevice(deviceId)
      toast.success('设备已登出')
    } catch (err) {
      toast.error('登出设备失败')
    }
  }

  const isCurrentDevice = (device: Device) => {
    return device.id === currentDeviceId || device.revoked === false
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <Laptop className="size-5" />
            已登录设备
          </CardTitle>
          <Button
            variant="outline"
            size="sm"
            onClick={handleRefresh}
            disabled={isLoading}
          >
            <RefreshCw className={`size-4 ${isLoading ? 'animate-spin' : ''}`} />
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {error && (
          <div className="mb-4 text-sm text-red-500 bg-red-50 dark:bg-red-950/20 p-3 rounded">
            {error}
          </div>
        )}

        {devices.length === 0 && !isLoading ? (
          <div className="text-center py-8 text-muted-foreground">
            暂无设备登录
          </div>
        ) : (
          <div className="space-y-2">
            {devices.map((device) => (
              <div
                key={device.id}
                className={`flex items-center justify-between p-4 rounded-lg transition-colors ${
                  device.revoked
                    ? 'bg-muted/30 opacity-60'
                    : 'bg-muted/50 hover:bg-muted/70'
                }`}
              >
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-background rounded-lg">
                    {getDeviceIcon(device.device_type)}
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <p className="font-medium">{device.device_name}</p>
                      {isCurrentDevice(device) && !device.revoked && (
                        <span className="text-xs px-2 py-0.5 bg-primary/10 text-primary rounded-full">
                          当前设备
                        </span>
                      )}
                      {device.revoked && (
                        <span className="text-xs px-2 py-0.5 bg-muted text-muted-foreground rounded-full">
                          已撤销
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-muted-foreground">
                      {formatTimeAgo(device.last_seen_at)}
                    </p>
                  </div>
                </div>

                {!device.revoked && device.id !== currentDeviceId && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleRevoke(device.id)}
                    className="text-destructive hover:text-destructive hover:bg-destructive/10"
                  >
                    <Trash2 className="size-4" />
                  </Button>
                )}
              </div>
            ))}
          </div>
        )}

        {/* 设备统计 */}
        {devices.length > 0 && (
          <div className="mt-6 pt-6 border-t">
            <div className="grid grid-cols-3 gap-4 text-center">
              <div>
                <p className="text-2xl font-bold">{devices.length}</p>
                <p className="text-sm text-muted-foreground">总设备数</p>
              </div>
              <div>
                <p className="text-2xl font-bold">
                  {devices.filter(d => Date.now() - d.last_seen_at < 1000 * 60 * 60 * 24 && !d.revoked).length}
                </p>
                <p className="text-sm text-muted-foreground">活跃设备</p>
              </div>
              <div>
                <p className="text-2xl font-bold">
                  {devices.filter(d => d.id !== currentDeviceId && !d.revoked).length}
                </p>
                <p className="text-sm text-muted-foreground">其他设备</p>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
