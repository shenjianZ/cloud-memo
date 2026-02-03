import { useState } from 'react'
import { ModeToggle } from '@/components/mode-toggle'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { Slider } from '@/components/ui/slider'
import { useNoteStore } from '@/store/noteStore'
import { toast } from 'sonner'
import { Download, Trash2, Keyboard, Database, Info, Type, Tag as TagIcon } from 'lucide-react'
import { FontSettings } from '@/components/FontSettings'
import { TagManager } from '@/components/tags/TagManager'

export default function Settings() {
  const { notes, exportAllNotes, clearAllNotes } = useNoteStore()
  const [fontSize, setFontSize] = useState(16)
  const [autoSaveInterval, setAutoSaveInterval] = useState(1000)
  const [showLineNumbers, setShowLineNumbers] = useState(false)
  const [spellCheck, setSpellCheck] = useState(true)

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
    <div className="space-y-6 max-w-4xl">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">设置</h1>
        <p className="text-muted-foreground">管理应用偏好设置</p>
      </div>

      <div className="space-y-4">
        {/* 外观设置 */}
        <div className="rounded-lg border p-4">
          <div className="flex items-center gap-2 mb-4">
            <Info className="w-4 h-4" />
            <h3 className="font-semibold">外观</h3>
          </div>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">主题</p>
                <p className="text-sm text-muted-foreground">切换浅色/深色主题</p>
              </div>
              <ModeToggle />
            </div>

            <Separator />

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label htmlFor="font-size">编辑器字体大小</Label>
                <span className="text-sm text-muted-foreground">{fontSize}px</span>
              </div>
              <Slider
                id="font-size"
                min={12}
                max={24}
                step={1}
                value={[fontSize]}
                onValueChange={(value) => setFontSize(value[0])}
                className="w-full"
              />
            </div>
          </div>
        </div>

        {/* 字体设置 */}
        <div className="rounded-lg border p-4">
          <div className="flex items-center gap-2 mb-4">
            <Type className="w-4 h-4" />
            <h3 className="font-semibold">字体设置</h3>
          </div>
          <FontSettings />
        </div>

        {/* 编辑器设置 */}
        <div className="rounded-lg border p-4">
          <div className="flex items-center gap-2 mb-4">
            <Keyboard className="w-4 h-4" />
            <h3 className="font-semibold">编辑器</h3>
          </div>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label htmlFor="line-numbers">显示行号</Label>
                <p className="text-sm text-muted-foreground">在代码块中显示行号</p>
              </div>
              <Switch
                id="line-numbers"
                checked={showLineNumbers}
                onCheckedChange={setShowLineNumbers}
              />
            </div>

            <Separator />

            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label htmlFor="spell-check">拼写检查</Label>
                <p className="text-sm text-muted-foreground">启用浏览器拼写检查</p>
              </div>
              <Switch
                id="spell-check"
                checked={spellCheck}
                onCheckedChange={setSpellCheck}
              />
            </div>

            <Separator />

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label htmlFor="auto-save">自动保存间隔</Label>
                <span className="text-sm text-muted-foreground">{autoSaveInterval / 1000}s</span>
              </div>
              <Slider
                id="auto-save"
                min={500}
                max={5000}
                step={500}
                value={[autoSaveInterval]}
                onValueChange={(value) => setAutoSaveInterval(value[0])}
                className="w-full"
              />
            </div>
          </div>
        </div>

        {/* 标签管理 */}
        <div className="rounded-lg border p-4">
          <div className="flex items-center gap-2 mb-4">
            <TagIcon className="w-4 h-4" />
            <h3 className="font-semibold">标签管理</h3>
          </div>
          <TagManager />
        </div>

        {/* 数据管理 */}
        <div className="rounded-lg border p-4">
          <div className="flex items-center gap-2 mb-4">
            <Database className="w-4 h-4" />
            <h3 className="font-semibold">数据管理</h3>
          </div>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
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

        {/* 快捷键 */}
        <div className="rounded-lg border p-4">
          <div className="flex items-center gap-2 mb-4">
            <Keyboard className="w-4 h-4" />
            <h3 className="font-semibold">快捷键</h3>
          </div>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">新建笔记</span>
              <kbd className="px-2 py-1 bg-muted rounded text-xs">Ctrl + N</kbd>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">保存</span>
              <kbd className="px-2 py-1 bg-muted rounded text-xs">Ctrl + S</kbd>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">查找</span>
              <kbd className="px-2 py-1 bg-muted rounded text-xs">Ctrl + F</kbd>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">斜杠命令</span>
              <kbd className="px-2 py-1 bg-muted rounded text-xs">/</kbd>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">粗体</span>
              <kbd className="px-2 py-1 bg-muted rounded text-xs">Ctrl + B</kbd>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">斜体</span>
              <kbd className="px-2 py-1 bg-muted rounded text-xs">Ctrl + I</kbd>
            </div>
          </div>
        </div>

        {/* 关于 */}
        <div className="rounded-lg border p-4">
          <div className="flex items-center gap-2 mb-4">
            <Info className="w-4 h-4" />
            <h3 className="font-semibold">关于</h3>
          </div>
          <div className="space-y-2 text-sm">
            <p><strong>版本:</strong> 0.1.0</p>
            <p><strong>技术栈:</strong> React + TypeScript + Tauri 2 + Tiptap</p>
            <p><strong>数据库:</strong> SQLite</p>
            <p><strong>开源协议:</strong> MIT</p>
          </div>
        </div>
      </div>
    </div>
  )
}
