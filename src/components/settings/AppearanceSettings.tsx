import { useState } from 'react'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Slider } from '@/components/ui/slider'
import { ModeToggle } from '@/components/mode-toggle'

export function AppearanceSettings() {
  const [fontSize, setFontSize] = useState(16)

  return (
    <div className="space-y-6">
      {/* 顶部标题 */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">外观设置</h2>
      </div>

      <div className="space-y-4">
        {/* 主题切换 */}
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label htmlFor="theme">主题模式</Label>
            <p className="text-sm text-muted-foreground">
              选择应用的主题外观
            </p>
          </div>
          <ModeToggle />
        </div>

        <Separator />

        {/* 编辑器字体大小 */}
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
  )
}
