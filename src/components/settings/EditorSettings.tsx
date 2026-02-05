import { useState } from 'react'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { Slider } from '@/components/ui/slider'

export function EditorSettings() {
  const [showLineNumbers, setShowLineNumbers] = useState(false)
  const [spellCheck, setSpellCheck] = useState(true)
  const [autoSaveInterval, setAutoSaveInterval] = useState(1000)

  return (
    <div className="space-y-6">
      {/* 顶部标题 */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">编辑器设置</h2>
      </div>

      <div className="space-y-4">
        {/* 显示行号 */}
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label htmlFor="line-numbers">显示行号</Label>
            <p className="text-sm text-muted-foreground">
              在代码块中显示行号
            </p>
          </div>
          <Switch
            id="line-numbers"
            checked={showLineNumbers}
            onCheckedChange={setShowLineNumbers}
          />
        </div>

        <Separator />

        {/* 拼写检查 */}
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label htmlFor="spell-check">拼写检查</Label>
            <p className="text-sm text-muted-foreground">
              启用浏览器拼写检查
            </p>
          </div>
          <Switch
            id="spell-check"
            checked={spellCheck}
            onCheckedChange={setSpellCheck}
          />
        </div>

        <Separator />

        {/* 自动保存间隔 */}
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
  )
}
