import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { useNoteStore } from '@/store/noteStore'
import { toast } from 'sonner'
import { Download, Trash2 } from 'lucide-react'

export function DataSettings() {
  const { notes, exportAllNotes, clearAllNotes } = useNoteStore()

  // 导出所有笔记
  const handleExportAll = async () => {
    try {
      await exportAllNotes()
      toast.success('所有笔记已导出')
    } catch (error) {
      toast.error('导出失败')
      console.error(error)
    }
  }

  // 清除所有数据
  const handleClearAll = async () => {
    try {
      await clearAllNotes()
      toast.success('所有数据已清除')
    } catch (error) {
      toast.error('清除失败')
      console.error(error)
    }
  }

  return (
    <div className="space-y-6">
      {/* 顶部标题 */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">数据管理</h2>
      </div>

      <div className="space-y-4">
        {/* 导出笔记 */}
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <p className="font-medium">导出所有笔记</p>
            <p className="text-sm text-muted-foreground">
              当前共有 {notes.length} 篇笔记
            </p>
          </div>
          <Button variant="outline" size="sm" onClick={handleExportAll}>
            <Download className="w-4 h-4 mr-2" />
            导出
          </Button>
        </div>

        <Separator />

        {/* 清除数据 */}
        <div className="rounded-lg border border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-950/20 p-4">
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              <Trash2 className="w-4 h-4 text-red-600 dark:text-red-400" />
              <p className="font-medium text-red-900 dark:text-red-100">危险区域</p>
            </div>
            <p className="text-sm text-red-700 dark:text-red-300">
              此操作将删除所有笔记且无法撤销。请确保已导出重要数据。
            </p>
            <Button
              variant="destructive"
              size="sm"
              onClick={() => {
                const confirmed = window.confirm('确定要清除所有数据吗？此操作将删除所有笔记且无法撤销。请确保已导出重要数据。')
                if (confirmed) {
                  handleClearAll()
                }
              }}
            >
              <Trash2 className="w-4 h-4 mr-2" />
              清除所有数据
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}
