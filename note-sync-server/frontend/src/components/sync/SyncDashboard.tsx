import { CheckCircle2, Cloud, HardDrive, AlertTriangle } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useSyncStore } from '@/store/syncStore'

export function SyncDashboard() {
  const { status, lastSyncAt, pendingCount, conflictCount } = useSyncStore()
  const formatTimeAgo = (timestamp: number | null) => {
    if (!timestamp) return '从未同步'
    const seconds = Math.floor((Date.now() - timestamp) / 1000)
    if (seconds < 60) return `${seconds}秒前`
    const minutes = Math.floor(seconds / 60)
    if (minutes < 60) return `${minutes}分钟前`
    const hours = Math.floor(minutes / 60)
    if (hours < 24) return `${hours}小时前`
    const days = Math.floor(hours / 24)
    return `${days}天前`
  }

  const getStatusText = () => {
    switch (status) {
      case 'idle':
        return '已同步'
      case 'syncing':
        return '同步中'
      case 'error':
        return '错误'
      case 'conflict':
        return '有冲突'
      default:
        return '未知'
    }
  }

  const getStatusIcon = () => {
    switch (status) {
      case 'idle':
        return <CheckCircle2 className="size-5 text-green-500" />
      case 'syncing':
        return <Cloud className="size-5 text-yellow-500 animate-pulse" />
      case 'error':
        return <AlertTriangle className="size-5 text-red-500" />
      case 'conflict':
        return <AlertTriangle className="size-5 text-orange-500" />
      default:
        return null
    }
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      {/* 同步状态卡片 */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">同步状态</CardTitle>
          {getStatusIcon()}
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{getStatusText()}</div>
          <p className="text-xs text-muted-foreground">
            上次同步：{formatTimeAgo(lastSyncAt)}
          </p>
        </CardContent>
      </Card>

      {/* 待同步数量 */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">待同步</CardTitle>
          <Cloud className="size-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{pendingCount}</div>
          <p className="text-xs text-muted-foreground">项更改</p>
        </CardContent>
      </Card>

      {/* 冲突数量 */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">冲突</CardTitle>
          <AlertTriangle className="size-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{conflictCount}</div>
          <p className="text-xs text-muted-foreground">需要解决</p>
        </CardContent>
      </Card>

      {/* 云端存储 */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">云端存储</CardTitle>
          <HardDrive className="size-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">--</div>
          <p className="text-xs text-muted-foreground">未实现</p>
        </CardContent>
      </Card>
    </div>
  )
}
