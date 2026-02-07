import { useEffect, useState } from 'react'
import { useAppSettingsStore } from '@/store/appSettingsStore'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { toast } from 'sonner'
import { Loader2, Server, RefreshCw } from 'lucide-react'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

export function AppSettings() {
  const { settings, isLoading, fetchSettings, updateSettings, resetSettings } =
    useAppSettingsStore()

  const [serverUrl, setServerUrl] = useState('')
  const [autoSyncEnabled, setAutoSyncEnabled] = useState(true)
  const [syncInterval, setSyncInterval] = useState('5')
  const [isSaving, setIsSaving] = useState(false)

  useEffect(() => {
    fetchSettings()
  }, [fetchSettings])

  useEffect(() => {
    if (settings) {
      setServerUrl(settings.defaultServerUrl)
      setAutoSyncEnabled(settings.autoSyncEnabled)
      setSyncInterval(settings.syncIntervalMinutes.toString())
    }
  }, [settings])

  const handleSave = async () => {
    setIsSaving(true)
    try {
      await updateSettings({
        defaultServerUrl: serverUrl,
        autoSyncEnabled: autoSyncEnabled,
        syncIntervalMinutes: parseInt(syncInterval),
      })
      toast.success('设置已保存')
    } catch (error) {
      toast.error('保存设置失败', {
        description: error instanceof Error ? error.message : '未知错误',
      })
    } finally {
      setIsSaving(false)
    }
  }

  const handleReset = async () => {
    if (confirm('确定要重置为默认设置吗？')) {
      setIsSaving(true)
      try {
        const reset = await resetSettings()
        setServerUrl(reset.defaultServerUrl)
        setAutoSyncEnabled(reset.autoSyncEnabled)
        setSyncInterval(reset.syncIntervalMinutes.toString())
        toast.success('设置已重置')
      } catch (error) {
        toast.error('重置设置失败', {
          description: error instanceof Error ? error.message : '未知错误',
        })
      } finally {
        setIsSaving(false)
      }
    }
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* 服务器设置 */}
      <div className="space-y-4">
        <div className="flex items-center gap-2 pb-2 border-b">
          <Server className="h-5 w-5" />
          <h3 className="text-lg font-semibold">服务器设置</h3>
        </div>

        <div className="space-y-2">
          <Label htmlFor="server-url">默认服务器地址</Label>
          <Input
            id="server-url"
            placeholder="https://api.noteapp.com"
            value={serverUrl}
            onChange={(e) => setServerUrl(e.target.value)}
            className="font-mono text-sm"
          />
          <p className="text-xs text-muted-foreground">
            登录时如果不指定服务器，将使用此默认地址
          </p>
        </div>
      </div>

      {/* 同步设置 */}
      <div className="space-y-4">
        <div className="flex items-center gap-2 pb-2 border-b">
          <RefreshCw className="h-5 w-5" />
          <h3 className="text-lg font-semibold">同步设置</h3>
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label htmlFor="auto-sync">自动同步</Label>
            <p className="text-xs text-muted-foreground">
              自动将笔记同步到云端服务器
            </p>
          </div>
          <Switch
            id="auto-sync"
            checked={autoSyncEnabled}
            onCheckedChange={setAutoSyncEnabled}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="sync-interval">同步间隔（分钟）</Label>
          <Select value={syncInterval} onValueChange={setSyncInterval}>
            <SelectTrigger id="sync-interval">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="1">1 分钟</SelectItem>
              <SelectItem value="5">5 分钟</SelectItem>
              <SelectItem value="10">10 分钟</SelectItem>
              <SelectItem value="15">15 分钟</SelectItem>
              <SelectItem value="30">30 分钟</SelectItem>
              <SelectItem value="60">60 分钟</SelectItem>
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            自动同步的时间间隔
          </p>
        </div>
      </div>

      {/* 操作按钮 */}
      <div className="flex gap-3 pt-4 border-t">
        <Button onClick={handleSave} disabled={isSaving} className="flex-1">
          {isSaving && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
          {isSaving ? '保存中...' : '保存设置'}
        </Button>
        <Button
          onClick={handleReset}
          disabled={isSaving}
          variant="outline"
          className="flex-1"
        >
          <RefreshCw className="h-4 w-4 mr-2" />
          重置默认
        </Button>
      </div>
    </div>
  )
}
