import { useState } from 'react'
import { AlertTriangle, Clock, FileText, Check, X, Eye } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'

interface Conflict {
  id: string
  entityType: 'note' | 'folder'
  title: string
  localVersion: {
    content: string
    updatedAt: number
  }
  serverVersion: {
    content: string
    updatedAt: number
  }
}

// 模拟数据
const mockConflicts: Conflict[] = [
  {
    id: '1',
    entityType: 'note',
    title: '项目计划',
    localVersion: {
      content: '这是本地版本的笔记内容...',
      updatedAt: Date.now() - 1000 * 60 * 10
    },
    serverVersion: {
      content: '这是服务器版本的笔记内容...',
      updatedAt: Date.now() - 1000 * 60 * 5
    }
  }
]

export function ConflictResolver() {
  const [conflicts, setConflicts] = useState<Conflict[]>(mockConflicts)
  const [selectedConflict, setSelectedConflict] = useState<Conflict | null>(null)
  const [previewMode, setPreviewMode] = useState<'local' | 'server' | null>(null)

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

  const handleResolve = (conflictId: string, _strategy: 'server' | 'local') => {
    // TODO: 调用 API 解决冲突，使用选定的策略
    console.log(`Resolving conflict ${conflictId} with strategy: ${_strategy}`)
    setConflicts(conflicts.filter(c => c.id !== conflictId))
    if (selectedConflict?.id === conflictId) {
      setSelectedConflict(null)
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <AlertTriangle className="size-5 text-orange-500" />
          冲突解决
        </CardTitle>
      </CardHeader>
      <CardContent>
        {conflicts.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <AlertTriangle className="mx-auto size-12 mb-4 text-green-500" />
            <p>暂无冲突</p>
          </div>
        ) : (
          <div className="space-y-4">
            {conflicts.map((conflict) => (
              <div key={conflict.id} className="border rounded-lg p-4">
                {/* 标题和基本信息 */}
                <div className="flex items-start justify-between mb-4">
                  <div className="flex items-center gap-2">
                    <FileText className="size-5 text-muted-foreground" />
                    <div>
                      <h4 className="font-semibold">{conflict.title}</h4>
                      <p className="text-sm text-muted-foreground">
                        {conflict.entityType === 'note' ? '笔记' : '文件夹'}
                      </p>
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setSelectedConflict(conflict)}
                    >
                      查看详情
                    </Button>
                  </div>
                </div>

                {/* 快速对比 */}
                <div className="grid grid-cols-2 gap-4 mb-4">
                  {/* 本地版本 */}
                  <div className="p-3 bg-blue-50 dark:bg-blue-950/20 rounded-lg border border-blue-200 dark:border-blue-800">
                    <div className="flex items-center gap-2 mb-2">
                      <FileText className="size-4 text-blue-500" />
                      <span className="text-sm font-medium">本地版本</span>
                    </div>
                    <div className="flex items-center gap-2 text-xs text-muted-foreground mb-2">
                      <Clock className="size-3" />
                      <span>{formatTimeAgo(conflict.localVersion.updatedAt)}</span>
                    </div>
                    <div className="text-sm line-clamp-2">{conflict.localVersion.content}</div>
                  </div>

                  {/* 服务器版本 */}
                  <div className="p-3 bg-green-50 dark:bg-green-950/20 rounded-lg border border-green-200 dark:border-green-800">
                    <div className="flex items-center gap-2 mb-2">
                      <FileText className="size-4 text-green-500" />
                      <span className="text-sm font-medium">服务器版本</span>
                    </div>
                    <div className="flex items-center gap-2 text-xs text-muted-foreground mb-2">
                      <Clock className="size-3" />
                      <span>{formatTimeAgo(conflict.serverVersion.updatedAt)}</span>
                    </div>
                    <div className="text-sm line-clamp-2">{conflict.serverVersion.content}</div>
                  </div>
                </div>

                {/* 操作按钮 */}
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleResolve(conflict.id, 'local')}
                    className="gap-2"
                  >
                    <X className="size-4" />
                    保留本地（创建副本）
                  </Button>
                  <Button
                    variant="default"
                    size="sm"
                    onClick={() => handleResolve(conflict.id, 'server')}
                    className="gap-2"
                  >
                    <Check className="size-4" />
                    保留服务器版本
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* 详情对话框 */}
        {selectedConflict && (
          <div
            className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 overflow-y-auto"
            onClick={() => {
              setSelectedConflict(null)
              setPreviewMode(null)
            }}
          >
            <div
              className="bg-background rounded-lg p-6 max-w-5xl w-full mx-4 my-8"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="flex items-center justify-between mb-4">
                <div>
                  <h3 className="text-lg font-semibold">{selectedConflict.title}</h3>
                  <p className="text-sm text-muted-foreground">
                    {selectedConflict.entityType === 'note' ? '笔记' : '文件夹'} 冲突
                  </p>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => {
                    setSelectedConflict(null)
                    setPreviewMode(null)
                  }}
                >
                  关闭
                </Button>
              </div>

              <div className="grid grid-cols-2 gap-6">
                {/* 本地版本详情 */}
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <h4 className="font-medium flex items-center gap-2">
                      <FileText className="size-4 text-blue-500" />
                      本地版本
                    </h4>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setPreviewMode(previewMode === 'local' ? null : 'local')}
                    >
                      <Eye className="size-4" />
                    </Button>
                  </div>
                  <div className="text-sm text-muted-foreground">
                    修改时间: {new Date(selectedConflict.localVersion.updatedAt).toLocaleString()}
                  </div>
                  {previewMode === 'local' ? (
                    <div className="p-4 bg-muted rounded-lg max-h-96 overflow-y-auto">
                      <pre className="text-sm whitespace-pre-wrap">{selectedConflict.localVersion.content}</pre>
                    </div>
                  ) : (
                    <div className="p-4 bg-blue-50 dark:bg-blue-950/20 rounded-lg border border-blue-200 dark:border-blue-800 text-sm line-clamp-6">
                      {selectedConflict.localVersion.content}
                    </div>
                  )}
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full"
                    onClick={() => handleResolve(selectedConflict.id, 'local')}
                  >
                    <X className="size-4 mr-2" />
                    使用本地版本
                  </Button>
                </div>

                {/* 服务器版本详情 */}
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <h4 className="font-medium flex items-center gap-2">
                      <FileText className="size-4 text-green-500" />
                      服务器版本
                    </h4>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setPreviewMode(previewMode === 'server' ? null : 'server')}
                    >
                      <Eye className="size-4" />
                    </Button>
                  </div>
                  <div className="text-sm text-muted-foreground">
                    修改时间: {new Date(selectedConflict.serverVersion.updatedAt).toLocaleString()}
                  </div>
                  {previewMode === 'server' ? (
                    <div className="p-4 bg-muted rounded-lg max-h-96 overflow-y-auto">
                      <pre className="text-sm whitespace-pre-wrap">{selectedConflict.serverVersion.content}</pre>
                    </div>
                  ) : (
                    <div className="p-4 bg-green-50 dark:bg-green-950/20 rounded-lg border border-green-200 dark:border-green-800 text-sm line-clamp-6">
                      {selectedConflict.serverVersion.content}
                    </div>
                  )}
                  <Button
                    variant="default"
                    size="sm"
                    className="w-full"
                    onClick={() => handleResolve(selectedConflict.id, 'server')}
                  >
                    <Check className="size-4 mr-2" />
                    使用服务器版本
                  </Button>
                </div>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
