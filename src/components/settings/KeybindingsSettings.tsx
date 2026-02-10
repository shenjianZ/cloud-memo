// KeybindingsSettings.tsx
import { useKeybindingStore } from '@/store/keybindingStore'
import { serializeKeyBinding } from '@/lib/keybindingParser'
import { useEffect, useState } from 'react'

export function KeybindingsSettings() {
  const { keybindings, isLoaded } = useKeybindingStore()
  const [mounted, setMounted] = useState(false)

  // 确保组件已挂载
  useEffect(() => {
    setMounted(true)
  }, [])

  // 从后端加载的快捷键中过滤出笔记软件相关的（global 和 note 开头）
  const noteKeybindings = Object.entries(keybindings)
    .filter(([actionId]) => actionId.startsWith('global.') || actionId.startsWith('note.'))
    .reduce((acc, [actionId, binding]) => {
      const category = actionId.startsWith('global.') ? '全局' : '编辑器'
      if (!acc[category]) {
        acc[category] = []
      }
      acc[category].push({
        id: actionId,
        key: serializeKeyBinding(binding)
      })
      return acc
    }, {} as Record<string, Array<{ id: string; key: string }>>)

  // 快捷键名称映射
  const keybindingNames: Record<string, string> = {
    'global.newNote': '新建笔记',
    'global.openSearch': '搜索笔记',
    'global.openSettings': '打开设置',
    'global.toggleSidebar': '切换侧边栏',
    'note.save': '保存笔记',
    'note.find': '查找',
    'note.closeTab': '关闭标签页',
    'note.togglePreview': '切换预览',
    'note.zoomIn': '放大字体',
    'note.zoomOut': '缩小字体',
    'note.zoomReset': '重置字体',
  }

  if (!mounted || !isLoaded) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold">快捷键设置</h2>
        </div>
        <div className="text-sm text-muted-foreground">加载中...</div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* 顶部标题 */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">快捷键设置</h2>
      </div>

      {/* 按分类显示快捷键 */}
      {Object.entries(noteKeybindings)
        .sort(([, a], [, b]) => b.length - a.length) // 按数量降序排列
        .map(([category, bindings]) => (
          <div key={category} className="space-y-3">
            <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wide">
              {category}快捷键
            </h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {bindings.map((binding) => (
                <div
                  key={binding.id}
                  className="flex items-center justify-between rounded-lg border p-3 bg-muted/20"
                >
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium truncate">
                      {keybindingNames[binding.id] || binding.id}
                    </div>
                    <div className="text-xs text-muted-foreground truncate">{binding.id}</div>
                  </div>
                  <kbd className="ml-2 px-2 py-1 bg-background rounded text-xs font-mono border flex-shrink-0">
                    {binding.key}
                  </kbd>
                </div>
              ))}
            </div>
          </div>
        ))}

      <div className="rounded-lg border p-4 bg-muted/20">
        <p className="text-sm text-muted-foreground">
          💡 提示：这些快捷键配置从后端加载，存储在
          <code className="px-1 py-0.5 bg-background rounded text-xs font-mono border mx-1">
            ~/.notes-data/keybindings.json
          </code>
          中。你可以直接编辑该文件来自定义快捷键。
        </p>
      </div>
    </div>
  )
}
