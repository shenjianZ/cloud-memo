import { useEffect, useState, useCallback } from 'react'
import { Label } from '@/components/ui/label'
import { Slider } from '@/components/ui/slider'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { useEditorSettingsStore } from '@/store/editorSettingsStore'
import { Type, Heading, Code2, RotateCcw } from 'lucide-react'
import { toast } from 'sonner'

/**
 * 字体设置组件
 * 允许用户自定义编辑器的字体样式
 * 任何更改都会立即保存到后端
 */
export function FontSettings() {
  const { settings, loadSettings, updateSettings } = useEditorSettingsStore()
  const [localSettings, setLocalSettings] = useState(settings)
  const [isSaving, setIsSaving] = useState(false)

  useEffect(() => {
    if (!settings) {
      loadSettings()
    } else {
      setLocalSettings(settings)
    }
  }, [settings, loadSettings])

  // 立即保存设置到后端
  const saveSetting = useCallback(async (key: string, value: any) => {
    if (!localSettings) return

    setIsSaving(true)
    try {
      const updatedSettings = { ...localSettings, [key]: value }
      await updateSettings(updatedSettings)
      setLocalSettings(updatedSettings)
    } catch (error) {
      toast.error('保存失败')
      console.error('Failed to save setting:', error)
    } finally {
      setIsSaving(false)
    }
  }, [localSettings, updateSettings])

  if (!localSettings) {
    return <div className="text-sm text-muted-foreground">加载中...</div>
  }

  const handleReset = async () => {
    const defaultSettings = {
      contentFontFamily: 'Inter, Avenir, Helvetica, Arial, sans-serif',
      contentFontSize: 16,
      contentFontWeight: 400,
      contentLineHeight: 1.7,
      headingFontFamily: 'Inter, Avenir, Helvetica, Arial, sans-serif',
      headingFontWeight: 600,
      codeFontFamily: "'JetBrains Mono', 'Fira Code', Consolas, 'Courier New', monospace",
      codeFontSize: 14,
    }

    try {
      await updateSettings(defaultSettings)
      setLocalSettings({ ...localSettings, ...defaultSettings, id: localSettings.id, updatedAt: localSettings.updatedAt })
      toast.success('已重置为默认设置')
    } catch (error) {
      toast.error('重置失败')
    }
  }

  return (
    <div className="space-y-6">
      {/* 保存状态提示 */}
      {isSaving && (
        <div className="text-xs text-muted-foreground">正在保存...</div>
      )}

      {/* 正文设置 */}
      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <Type className="w-4 h-4" />
          <h4 className="font-semibold">正文文字</h4>
        </div>

        <div className="space-y-4 pl-6">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label htmlFor="content-font-size">字体大小</Label>
              <span className="text-sm text-muted-foreground">{localSettings.contentFontSize}px</span>
            </div>
            <Slider
              id="content-font-size"
              min={12}
              max={24}
              step={1}
              value={[localSettings.contentFontSize]}
              onValueChange={([value]) => saveSetting('contentFontSize', value)}
              disabled={isSaving}
              className="w-full"
            />
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label htmlFor="content-font-weight">字重</Label>
              <span className="text-sm text-muted-foreground">{localSettings.contentFontWeight}</span>
            </div>
            <Slider
              id="content-font-weight"
              min={100}
              max={900}
              step={100}
              value={[localSettings.contentFontWeight]}
              onValueChange={([value]) => saveSetting('contentFontWeight', value)}
              disabled={isSaving}
              className="w-full"
            />
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label htmlFor="content-line-height">行高</Label>
              <span className="text-sm text-muted-foreground">{localSettings.contentLineHeight}</span>
            </div>
            <Slider
              id="content-line-height"
              min={1.0}
              max={2.5}
              step={0.1}
              value={[localSettings.contentLineHeight]}
              onValueChange={([value]) => saveSetting('contentLineHeight', value)}
              disabled={isSaving}
              className="w-full"
            />
          </div>
        </div>
      </div>

      <Separator />

      {/* 标题设置 */}
      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <Heading className="w-4 h-4" />
          <h4 className="font-semibold">标题文字</h4>
        </div>

        <div className="space-y-4 pl-6">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label htmlFor="heading-font-weight">标题字重</Label>
              <span className="text-sm text-muted-foreground">{localSettings.headingFontWeight}</span>
            </div>
            <Slider
              id="heading-font-weight"
              min={100}
              max={900}
              step={100}
              value={[localSettings.headingFontWeight]}
              onValueChange={([value]) => saveSetting('headingFontWeight', value)}
              disabled={isSaving}
              className="w-full"
            />
          </div>
        </div>
      </div>

      <Separator />

      {/* 代码设置 */}
      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <Code2 className="w-4 h-4" />
          <h4 className="font-semibold">代码文字</h4>
        </div>

        <div className="space-y-4 pl-6">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label htmlFor="code-font-size">代码字体大小</Label>
              <span className="text-sm text-muted-foreground">{localSettings.codeFontSize}px</span>
            </div>
            <Slider
              id="code-font-size"
              min={10}
              max={20}
              step={1}
              value={[localSettings.codeFontSize]}
              onValueChange={([value]) => saveSetting('codeFontSize', value)}
              disabled={isSaving}
              className="w-full"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="code-font-family">代码字体族</Label>
            <select
              id="code-font-family"
              value={localSettings.codeFontFamily}
              onChange={(e) => saveSetting('codeFontFamily', e.target.value)}
              disabled={isSaving}
              className="w-full px-3 py-2 bg-background border border-border rounded-md text-sm"
            >
              <option value="'JetBrains Mono', 'Fira Code', Consolas, 'Courier New', monospace">
                JetBrains Mono, Fira Code
              </option>
              <option value="'Fira Code', 'JetBrains Mono', Consolas, 'Courier New', monospace">
                Fira Code, JetBrains Mono
              </option>
              <option value="'Consolas', 'Monaco', 'Courier New', monospace">
                Consolas, Monaco
              </option>
              <option value="'Courier New', Courier, monospace">
                Courier New
              </option>
              <option value="monospace">
                系统等宽字体
              </option>
            </select>
          </div>
        </div>
      </div>

      <Separator />

      {/* 操作按钮 */}
      <div className="flex gap-2">
        <Button variant="outline" onClick={handleReset} disabled={isSaving} className="gap-2">
          <RotateCcw className="w-4 h-4" />
          重置为默认
        </Button>
      </div>

      {/* 预览 */}
      <div className="rounded-lg border p-4 space-y-3">
        <div className="text-sm text-muted-foreground">预览</div>
        <div style={{
          fontFamily: localSettings.contentFontFamily,
          fontSize: `${localSettings.contentFontSize}px`,
          fontWeight: localSettings.contentFontWeight,
          lineHeight: localSettings.contentLineHeight,
        }}>
          这是一段正文文字示例。The quick brown fox jumps over the lazy dog.
        </div>
        <div style={{
          fontFamily: localSettings.headingFontFamily,
          fontWeight: localSettings.headingFontWeight,
          fontSize: `${localSettings.contentFontSize * 1.5}px`,
        }}>
          这是标题示例
        </div>
        <code style={{
          fontFamily: localSettings.codeFontFamily,
          fontSize: `${localSettings.codeFontSize}px`,
          backgroundColor: 'rgb(255, 255, 255)',
          color: 'rgb(255, 0, 0)',
          padding: '0.125em 0.375em',
          borderRadius: '0.25rem',
        }}>
          const example = "代码示例";
        </code>
      </div>
    </div>
  )
}
