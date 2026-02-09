import { useEffect, useState } from 'react'
import { TiptapEditor } from './TiptapEditor'
import { useNoteStore } from '@/store/noteStore'
import type { EditorViewMode } from '@/types/editor'

interface TabEditorProps {
  tabId: string
  noteId: string
  isActive: boolean
}

/**
 * Tab 编辑器容器组件
 * 负责:
 * 1. 隔离每个 Tab 的编辑器状态
 * 2. 处理挂载/卸载逻辑
 * 3. 监听预览切换事件
 */
export function TabEditor({ tabId, noteId, isActive }: TabEditorProps) {
  const { getNote } = useNoteStore()

  // 每个 Tab 独立的 viewMode 状态
  const [viewMode, setViewMode] = useState<EditorViewMode>('edit')

  // 监听预览切换事件
  useEffect(() => {
    const handleTogglePreview = (e: Event) => {
      const customEvent = e as CustomEvent<{ tabId: string }>
      console.log('[TabEditor] handleTogglePreview called, tabId:', tabId, 'event.detail.tabId:', customEvent.detail.tabId)
      if (customEvent.detail.tabId === tabId) {
        console.log('[TabEditor] Tab matched, switching viewMode from:', viewMode)
        setViewMode((prev) => {
          const modes: EditorViewMode[] = ['edit', 'preview', 'split']
          const currentIndex = modes.indexOf(prev)
          const newMode = modes[(currentIndex + 1) % modes.length]
          console.log('[TabEditor] Switching to viewMode:', newMode)
          return newMode
        })
      }
    }

    window.addEventListener('toggle-preview-tab', handleTogglePreview)
    console.log('[TabEditor] Registered toggle-preview-tab listener for tabId:', tabId)

    return () => {
      window.removeEventListener('toggle-preview-tab', handleTogglePreview)
      console.log('[TabEditor] Unregistered toggle-preview-tab listener for tabId:', tabId)
    }
  }, [tabId, viewMode])

  const note = getNote(noteId)

  if (!note) {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        笔记不存在
      </div>
    )
  }

  // 非 Tab 使用 CSS 隐藏(保留 DOM,避免卸载)
  return (
    <div
      className={`h-full ${isActive ? 'block' : 'hidden'}`}
      data-tab-id={tabId}
      data-note-id={noteId}
    >
      <TiptapEditor
        noteId={noteId}
        content={note.content}
        viewMode={viewMode}
        onViewModeChange={setViewMode}
      />
    </div>
  )
}
