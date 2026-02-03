import { X } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useEditorStore } from '@/store/editorStore'

export function EditorTabBar() {
  const { tabs, setActiveTab, closeTab } = useEditorStore()

  if (tabs.length === 0) {
    return null
  }

  return (
    <div className="flex items-center gap-1 bg-muted/30 border-b border-border px-2 py-1">
      {tabs.map((tab) => (
        <div
          key={tab.id}
          className={`
            group flex items-center gap-2 px-3 py-1.5 rounded-t-lg
            border-b-2 transition-all cursor-pointer select-none
            min-w-[120px] max-w-[200px]
            ${
              tab.isActive
                ? 'bg-background border-primary text-foreground'
                : 'border-transparent text-muted-foreground hover:bg-muted/40'
            }
          `}
          onClick={() => {
            if (!tab.isActive) {
              setActiveTab(tab.id)
            }
          }}
        >
          {/* 脏标记 */}
          {tab.isDirty && (
            <span className="w-2 h-2 rounded-full bg-primary" />
          )}

          <span className="text-sm font-medium truncate flex-1">
            {tab.title}
          </span>

          <Button
            variant="ghost"
            size="sm"
            className="h-4 w-4 p-0 opacity-0 group-hover:opacity-100 transition-opacity"
            onClick={(e) => {
              e.stopPropagation()
              closeTab(tab.id)
            }}
          >
            <X className="h-3 w-3" />
          </Button>
        </div>
      ))}
    </div>
  )
}
