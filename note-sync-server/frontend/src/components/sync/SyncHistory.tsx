import { useState, useEffect } from 'react'
import { History, Clock, ArrowUp, ArrowDown, AlertCircle, CheckCircle2 } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import * as historyApi from '@/api/history'
import type { SyncHistoryEntry } from '@/api/history'
import { toast } from 'react-toastify'

export function SyncHistory() {
  const [history, setHistory] = useState<SyncHistoryEntry[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [selectedEntry, setSelectedEntry] = useState<SyncHistoryEntry | null>(null)

  useEffect(() => {
    loadHistory()
  }, [])

  const loadHistory = async () => {
    setIsLoading(true)
    try {
      const data = await historyApi.getSyncHistory(50)
      setHistory(data)
    } catch (error) {
      toast.error('加载历史记录失败')
    } finally {
      setIsLoading(false)
    }
  }

  const formatTimeAgo = (timestamp: number) => {
    const seconds = Math.floor((Date.now() - timestamp) / 1000)
    if (seconds < 60) return `${seconds}秒前`
    const minutes = Math.floor(seconds / 60)
    if (minutes < 60) return `${minutes}分钟前`
    const hours = Math.floor(minutes / 60)
    if (hours < 24) return `${hours}小时前`
    const days = Math.floor(hours / 24)
    return `${days}天前`
  }

  const getTypeIcon = (type: SyncHistoryEntry['sync_type']) => {
    switch (type) {
      case 'push':
        return <ArrowUp className="size-4 text-blue-500" />
      case 'pull':
        return <ArrowDown className="size-4 text-green-500" />
      case 'full':
        return <CheckCircle2 className="size-4 text-purple-500" />
      default:
        return null
    }
  }

  const getTypeText = (type: SyncHistoryEntry['sync_type']) => {
    switch (type) {
      case 'push':
        return '推送'
      case 'pull':
        return '拉取'
      case 'full':
        return '完全同步'
      default:
        return '未知'
    }
  }

  const getStatusIcon = (entry: SyncHistoryEntry) => {
    if (entry.error) {
      return <AlertCircle className="size-4 text-red-500" />
    }
    if (entry.conflict_count > 0) {
      return <AlertCircle className="size-4 text-orange-500" />
    }
    return <CheckCircle2 className="size-4 text-green-500" />
  }

  const handleClearHistory = async () => {
    if (!confirm('确定要清空历史记录吗？')) return

    try {
      await historyApi.clearSyncHistory()
      setHistory([])
      toast.success('历史记录已清空')
    } catch (error) {
      toast.error('清空失败')
    }
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <History className="size-5" />
            同步历史
          </CardTitle>
          <Button
            variant="outline"
            size="sm"
            onClick={handleClearHistory}
            disabled={history.length === 0}
          >
            清空历史
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="text-center py-8 text-muted-foreground">
            加载中...
          </div>
        ) : history.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            暂无历史记录
          </div>
        ) : (
          <div className="space-y-3">
            {history.map((entry) => (
              <div
                key={entry.id}
                className="p-4 bg-muted/50 rounded-lg hover:bg-muted/70 transition-colors cursor-pointer"
                onClick={() => setSelectedEntry(entry)}
              >
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    {getTypeIcon(entry.sync_type)}
                    <span className="font-medium">{getTypeText(entry.sync_type)}</span>
                    {getStatusIcon(entry)}
                  </div>
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Clock className="size-3" />
                    <span>{formatTimeAgo(entry.created_at)}</span>
                  </div>
                </div>

                <div className="grid grid-cols-4 gap-4 text-sm">
                  <div>
                    <span className="text-muted-foreground">推送:</span>{' '}
                    <span className="font-medium">{entry.pushed_count}</span>
                  </div>
                  <div>
                    <span className="text-muted-foreground">拉取:</span>{' '}
                    <span className="font-medium">{entry.pulled_count}</span>
                  </div>
                  <div>
                    <span className="text-muted-foreground">冲突:</span>{' '}
                    <span className="font-medium">{entry.conflict_count}</span>
                  </div>
                  <div>
                    <span className="text-muted-foreground">耗时:</span>{' '}
                    <span className="font-medium">{entry.duration_ms}ms</span>
                  </div>
                </div>

                {entry.error && (
                  <div className="mt-2 text-sm text-red-500">
                    错误: {entry.error}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}

        {/* 详情对话框 */}
        {selectedEntry && (
          <div
            className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
            onClick={() => setSelectedEntry(null)}
          >
            <div
              className="bg-background rounded-lg p-6 max-w-2xl w-full mx-4"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold">同步详情</h3>
                <Button variant="ghost" size="sm" onClick={() => setSelectedEntry(null)}>
                  关闭
                </Button>
              </div>

              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <span className="text-sm text-muted-foreground">类型:</span>
                    <p className="font-medium">{getTypeText(selectedEntry.sync_type)}</p>
                  </div>
                  <div>
                    <span className="text-sm text-muted-foreground">时间:</span>
                    <p className="font-medium">{new Date(selectedEntry.created_at).toLocaleString()}</p>
                  </div>
                  <div>
                    <span className="text-sm text-muted-foreground">推送数量:</span>
                    <p className="font-medium">{selectedEntry.pushed_count}</p>
                  </div>
                  <div>
                    <span className="text-sm text-muted-foreground">拉取数量:</span>
                    <p className="font-medium">{selectedEntry.pulled_count}</p>
                  </div>
                  <div>
                    <span className="text-sm text-muted-foreground">冲突数量:</span>
                    <p className="font-medium">{selectedEntry.conflict_count}</p>
                  </div>
                  <div>
                    <span className="text-sm text-muted-foreground">耗时:</span>
                    <p className="font-medium">{selectedEntry.duration_ms}ms</p>
                  </div>
                </div>

                {selectedEntry.error && (
                  <div className="p-3 bg-destructive/10 rounded-lg">
                    <span className="text-sm text-muted-foreground">错误信息:</span>
                    <p className="text-sm text-destructive mt-1">{selectedEntry.error}</p>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
