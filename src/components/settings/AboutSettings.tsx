import { Separator } from '@/components/ui/separator'
import { FileText, Github } from 'lucide-react'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Button } from '@/components/ui/button'

export function AboutSettings() {
  return (
    <div className="space-y-6">
      {/* 顶部标题 */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">关于</h2>
      </div>

      {/* 应用信息卡片 */}
      <div className="rounded-lg border p-6 bg-muted/20 text-center">
        <div className="w-16 h-16 rounded-2xl bg-primary/10 flex items-center justify-center mx-auto mb-4">
          <FileText className="h-8 w-8 text-primary" />
        </div>
        <h2 className="text-2xl font-bold">Markdown Notes</h2>
        <p className="text-muted-foreground mt-1">版本 0.1.0</p>
        <p className="text-sm text-muted-foreground mt-4 max-w-md mx-auto">
          基于 Tauri 2.0 和 React 构建的现代化 Markdown 笔记应用
        </p>
      </div>

      <Separator />

      {/* 技术栈 */}
      <div className="space-y-3">
        <h3 className="font-semibold">技术栈</h3>
        <div className="grid grid-cols-2 sm:grid-cols-2 gap-2 text-sm">
          <div className="rounded border p-3 bg-muted/20">
            <p className="font-medium">前端</p>
            <p className="text-muted-foreground text-xs">React 19</p>
          </div>
          <div className="rounded border p-3 bg-muted/20">
            <p className="font-medium">后端</p>
            <p className="text-muted-foreground text-xs">Tauri 2.0</p>
          </div>
          <div className="rounded border p-3 bg-muted/20">
            <p className="font-medium">UI 库</p>
            <p className="text-muted-foreground text-xs">shadcn/ui</p>
          </div>
          <div className="rounded border p-3 bg-muted/20">
            <p className="font-medium">编辑器</p>
            <p className="text-muted-foreground text-xs">Tiptap</p>
          </div>
        </div>
      </div>

      <Separator />

      {/* 开源仓库 */}
      <div className="space-y-3">
        <h3 className="font-semibold">开源仓库</h3>
        <div className="rounded-lg border p-4 bg-muted/20">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Github className="h-5 w-5" />
              <div>
                <p className="font-medium">GitHub 仓库</p>
                <p className="text-sm text-muted-foreground">查看源代码</p>
              </div>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => openUrl('https://github.com')}
              className="gap-2"
            >
              <Github className="h-4 w-4" />
              访问
            </Button>
          </div>
        </div>
      </div>

      <Separator />

      {/* 版权信息 */}
      <div className="text-center text-sm text-muted-foreground">
        <p>© 2025 Markdown Notes. All rights reserved.</p>
        <p className="mt-1">
          Made with ❤️ using Tauri and React
        </p>
      </div>
    </div>
  )
}
