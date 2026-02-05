// KeybindingsSettings.tsx

export function KeybindingsSettings() {
  const shortcuts = [
    { name: '新建笔记', key: 'Ctrl + N' },
    { name: '保存', key: 'Ctrl + S' },
    { name: '查找', key: 'Ctrl + F' },
    { name: '斜杠命令', key: '/' },
    { name: '粗体', key: 'Ctrl + B' },
    { name: '斜体', key: 'Ctrl + I' },
  ]

  return (
    <div className="space-y-6">
      {/* 顶部标题 */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">快捷键</h2>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        {shortcuts.map((shortcut) => (
          <div
            key={shortcut.name}
            className="flex items-center justify-between rounded-lg border p-3 bg-muted/20"
          >
            <span className="text-sm font-medium">{shortcut.name}</span>
            <kbd className="px-2 py-1 bg-background rounded text-xs font-mono border">
              {shortcut.key}
            </kbd>
          </div>
        ))}
      </div>

      <div className="rounded-lg border p-4 bg-muted/20">
        <p className="text-sm text-muted-foreground">
          💡 提示：快捷键可以帮助你更快速地操作编辑器。更多快捷键可以在编辑器中按
          <kbd className="px-1 py-0.5 bg-background rounded text-xs font-mono border mx-1">?</kbd>
          查看。
        </p>
      </div>
    </div>
  )
}
