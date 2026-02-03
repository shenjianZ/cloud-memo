import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { FileText, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { TiptapEditor } from '@/components/editor/TiptapEditor'
import { useNoteStore } from '@/store/noteStore'
import { toast } from 'sonner'

/**
 * 编辑器页面
 * 使用 Tiptap 所见即所得编辑器
 */
export function Editor() {
  const { noteId } = useParams<{ noteId: string }>()
  const navigate = useNavigate()
  const [isLoading, setIsLoading] = useState(true)

  const { getNote, loadNotesFromStorage, createNote, isStorageLoaded } = useNoteStore()

  // 初始化：加载笔记
  useEffect(() => {
    const initializeEditor = async () => {
      setIsLoading(true)
      try {
        // 加载所有笔记
        if (!isStorageLoaded) {
          await loadNotesFromStorage()
        }

        // 如果指定了 noteId，打开该笔记
        if (noteId && noteId !== 'new') {
          const note = getNote(noteId)
          if (!note) {
            toast.error('笔记不存在')
            navigate('/')
          }
        } else if (noteId === 'new') {
          // 创建新笔记
          const newNote = await createNote({
            content: { type: 'doc', content: [] },
          })
          // 更新 URL
          navigate(`/editor/${newNote.id}`, { replace: true })
        }
      } catch (error) {
        console.error('Failed to initialize editor:', error)
        toast.error('加载笔记失败')
      } finally {
        setIsLoading(false)
      }
    }

    initializeEditor()
  }, [noteId, isStorageLoaded, loadNotesFromStorage, getNote, createNote, navigate])

  // 获取当前笔记
  const currentNote = noteId ? getNote(noteId) : null

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!currentNote) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <FileText className="w-16 h-16 text-muted-foreground" />
        <div className="text-muted-foreground">笔记不存在</div>
        <Button onClick={() => navigate('/')}>返回首页</Button>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full bg-background">
      {/* Tiptap 编辑器 */}
      <div className="flex-1 overflow-hidden">
        <TiptapEditor noteId={currentNote.id} content={currentNote.content} />
      </div>
    </div>
  )
}
