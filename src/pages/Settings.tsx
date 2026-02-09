import { Settings as SettingsIcon, ArrowLeft } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { AppearanceSettings } from '@/components/settings/AppearanceSettings'
import { EditorSettings } from '@/components/settings/EditorSettings'
import { DataSettings } from '@/components/settings/DataSettings'
import { KeybindingsSettings } from '@/components/settings/KeybindingsSettings'
import { AboutSettings } from '@/components/settings/AboutSettings'
import { FontSettings } from '@/components/FontSettings'
import { TagManager } from '@/components/tags/TagManager'
import { AccountSyncSettings } from '@/components/sync/AccountSyncSettings'
import { AppSettings } from '@/components/settings/AppSettings'

export default function Settings() {
  const navigate = useNavigate()

  return (
    <div className="p-4 sm:p-6 max-w-4xl mx-auto">
      {/* 页面标题 */}
      <div className="mb-4 sm:mb-6">
        <div className="flex items-center gap-3">
          <Button
            variant="ghost"
            size="sm"
            className="h-8 w-8 p-0"
            onClick={() => navigate(-1)}
            title="返回"
          >
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <SettingsIcon className="h-6 w-6 sm:h-8 sm:w-8" />
          <div>
            <h1 className="text-2xl sm:text-3xl font-bold">设置</h1>
            <p className="text-sm sm:text-base text-muted-foreground mt-1">
              配置应用偏好和编辑器选项
            </p>
          </div>
        </div>
      </div>

      {/* 设置选项卡 */}
      <Tabs defaultValue="appearance" className="space-y-4 sm:space-y-6">
        <TabsList className="grid w-full grid-cols-2 sm:grid-cols-4 gap-1 h-auto">
          <TabsTrigger value="appearance" className="gap-2">
            <SettingsIcon className="h-4 w-4" />
            外观
          </TabsTrigger>
          <TabsTrigger value="editor" className="gap-2">
            <SettingsIcon className="h-4 w-4" />
            编辑器
          </TabsTrigger>
          <TabsTrigger value="fonts" className="gap-2">
            <SettingsIcon className="h-4 w-4" />
            字体
          </TabsTrigger>
          <TabsTrigger value="sync" className="gap-2">
            <SettingsIcon className="h-4 w-4" />
            同步
          </TabsTrigger>
          <TabsTrigger value="tags" className="gap-2">
            <SettingsIcon className="h-4 w-4" />
            标签
          </TabsTrigger>
          <TabsTrigger value="data" className="gap-2">
            <SettingsIcon className="h-4 w-4" />
            数据
          </TabsTrigger>
          <TabsTrigger value="keybindings" className="gap-2">
            <SettingsIcon className="h-4 w-4" />
            快捷键
          </TabsTrigger>
          <TabsTrigger value="about" className="gap-2">
            <SettingsIcon className="h-4 w-4" />
            关于
          </TabsTrigger>
        </TabsList>

        {/* 外观设置 */}
        <TabsContent value="appearance" className="space-y-6">
          <AppearanceSettings />
        </TabsContent>

        {/* 编辑器设置 */}
        <TabsContent value="editor" className="space-y-6">
          <EditorSettings />
        </TabsContent>

        {/* 字体设置 */}
        <TabsContent value="fonts" className="space-y-6">
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <h2 className="text-xl font-semibold">字体设置</h2>
            </div>
            <FontSettings />
          </div>
        </TabsContent>

        {/* 同步设置 */}
        <TabsContent value="sync" className="space-y-6">
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <h2 className="text-xl font-semibold">账户与同步</h2>
            </div>
            <AccountSyncSettings />

            <div className="pt-6 border-t">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-semibold">应用设置</h2>
              </div>
              <AppSettings />
            </div>
          </div>
        </TabsContent>

        {/* 标签管理 */}
        <TabsContent value="tags" className="space-y-6">
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <h2 className="text-xl font-semibold">标签管理</h2>
            </div>
            <div className="rounded-lg border p-4">
              <TagManager />
            </div>
          </div>
        </TabsContent>

        {/* 数据管理 */}
        <TabsContent value="data" className="space-y-6">
          <DataSettings />
        </TabsContent>

        {/* 快捷键设置 */}
        <TabsContent value="keybindings" className="space-y-6">
          <KeybindingsSettings />
        </TabsContent>

        {/* 关于 */}
        <TabsContent value="about" className="space-y-6">
          <AboutSettings />
        </TabsContent>
      </Tabs>
    </div>
  )
}
