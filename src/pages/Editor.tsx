import { useEffect, useState, useRef } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { FileText, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { EditorTabBar } from '@/components/editor/EditorTabBar'
import { TabEditor } from '@/components/editor/TabEditor'
import { useNoteStore } from '@/store/noteStore'
import { useEditorStore } from '@/store/editorStore'
import { toast } from 'sonner'

/**
 * 编辑器页面 - 支持多 Tab
 * 使用 Tiptap 所见即所得编辑器
 */
export function Editor() {
  const { noteId } = useParams<{ noteId: string }>()
  const navigate = useNavigate()
  const [isLoading, setIsLoading] = useState(true)
  const [initialized, setInitialized] = useState(false)
  const isUpdatingUrlRef = useRef(false)

  const {
    getNote,
    loadNotesFromStorage,
    createNote,
    isStorageLoaded,
  } = useNoteStore()

  const {
    tabs,
    openNote,
    closeTab,
    getActiveTab,
  } = useEditorStore()

  // 初始化：打开 URL 中的笔记
  useEffect(() => {
    const initializeEditor = async () => {
      if (initialized) return // 防止重复初始化

      setIsLoading(true)
      try {
        // 加载所有笔记
        if (!isStorageLoaded) {
          await loadNotesFromStorage()
        }

        // 处理 noteId
        if (noteId && noteId !== 'new') {
          const note = getNote(noteId)
          if (note) {
            openNote(note.id, note.title || '未命名笔记')
          } else {
            toast.error('笔记不存在')
            navigate('/')
          }
        } else if (noteId === 'new') {
          const newNote = await createNote({
            title: '未命名笔记',
            content: {
              type: 'doc',
              content: [
                {
                  type: 'heading',
                  attrs: { level: 1 },
                  content: [{ type: 'text', text: '未命名笔记' }],
                },
              ],
            },
          })
          openNote(newNote.id, newNote.title || '未命名笔记')
          navigate(`/editor/${newNote.id}`, { replace: true })
        }

        setInitialized(true)
      } catch (error) {
        console.error('Failed to initialize editor:', error)
        toast.error('加载笔记失败')
      } finally {
        setIsLoading(false)
      }
    }

    initializeEditor()
  }, [noteId, isStorageLoaded])

  // Tab 切换 → 更新 URL
  const activeTab = getActiveTab()
  useEffect(() => {
    if (!initialized || !activeTab || isUpdatingUrlRef.current) return

    if (activeTab.noteId !== noteId) {
      isUpdatingUrlRef.current = true
      navigate(`/editor/${activeTab.noteId}`, { replace: true })
      // 延迟重置标志位，避免循环
      setTimeout(() => {
        isUpdatingUrlRef.current = false
      }, 0)
    }
  }, [activeTab?.id, initialized, noteId, navigate])

  // 监听 Tab 切换事件（来自 closeTab）
  useEffect(() => {
    const handleTabSwitched = (e: Event) => {
      const customEvent = e as CustomEvent<{ noteId: string }>
      if (!isUpdatingUrlRef.current) {
        isUpdatingUrlRef.current = true
        navigate(`/editor/${customEvent.detail.noteId}`, { replace: true })
        setTimeout(() => {
          isUpdatingUrlRef.current = false
        }, 0)
      }
    }

    window.addEventListener('tab-switched', handleTabSwitched)
    return () => {
      window.removeEventListener('tab-switched', handleTabSwitched)
    }
  }, [navigate])

  // 快捷键监听
  useEffect(() => {
    const handleCloseTab = () => {
      console.log('[Editor.tsx] handleCloseTab called')
      const currentActiveTab = getActiveTab()
      console.log('[Editor.tsx] currentActiveTab:', currentActiveTab)
      if (currentActiveTab) {
        closeTab(currentActiveTab.id)
      }
    }

    const handleTogglePreview = () => {
      console.log('[Editor.tsx] handleTogglePreview called')
      const currentActiveTab = getActiveTab()
      console.log('[Editor.tsx] currentActiveTab:', currentActiveTab)
      if (currentActiveTab) {
        window.dispatchEvent(
          new CustomEvent('toggle-preview-tab', {
            detail: { tabId: currentActiveTab.id, noteId: currentActiveTab.noteId },
          }),
        )
      }
    }

    window.addEventListener('keybinding-note-close-tab', handleCloseTab)
    window.addEventListener('keybinding-note-toggle-preview', handleTogglePreview)

    console.log('[Editor.tsx] 快捷键监听器已注册')

    return () => {
      window.removeEventListener('keybinding-note-close-tab', handleCloseTab)
      window.removeEventListener('keybinding-note-toggle-preview', handleTogglePreview)
      console.log('[Editor.tsx] 快捷键监听器已移除')
    }
  }, [getActiveTab, closeTab])

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (tabs.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <FileText className="w-16 h-16 text-muted-foreground" />
        <div className="text-muted-foreground">没有打开的标签页</div>
        <Button onClick={() => navigate('/')}>返回首页</Button>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full bg-background">
      {/* Tab 栏 */}
      <EditorTabBar />

      {/* 编辑器区域 */}
      <div className="flex-1 overflow-hidden">
        {tabs.map((tab) => (
          <TabEditor
            key={tab.id}
            tabId={tab.id}
            noteId={tab.noteId}
            isActive={tab.isActive}
          />
        ))}
      </div>
    </div>
  )
}
