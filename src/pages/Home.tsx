import { useNavigate } from 'react-router-dom'
import { FileText, Plus, Keyboard } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useNoteStore } from '@/store/noteStore'
import { getNoteTitle, getPlainText } from '@/lib/noteHelpers'

export default function Home() {
  const navigate = useNavigate()
  const { notes } = useNoteStore()

  const handleCreateNote = () => {
    navigate('/editor/new')
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">欢迎使用 Markdown Notes</h1>
        <p className="text-muted-foreground">
          一个现代化的 Markdown 笔记应用，支持本地存储和云端同步。
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <div className="rounded-lg border p-4">
          <h3 className="font-semibold mb-2 flex items-center gap-2">
            <Plus className="w-4 h-4" />
            快速开始
          </h3>
          <p className="text-sm text-muted-foreground mb-4">
            创建你的第一篇笔记
          </p>
          <Button onClick={handleCreateNote} className="w-full">
            <Plus className="w-4 h-4 mr-2" />
            新建笔记
          </Button>
          <p className="text-xs text-muted-foreground mt-2">
            或按 Ctrl+N 快速创建
          </p>
        </div>

        <div className="rounded-lg border p-4">
          <h3 className="font-semibold mb-2 flex items-center gap-2">
            <FileText className="w-4 h-4" />
            笔记统计
          </h3>
          <div className="text-2xl font-bold">{notes.length}</div>
          <p className="text-sm text-muted-foreground">
            篇笔记
          </p>
        </div>

        <div className="rounded-lg border p-4">
          <h3 className="font-semibold mb-2 flex items-center gap-2">
            <Keyboard className="w-4 h-4" />
            快捷键
          </h3>
          <div className="space-y-1 text-sm text-muted-foreground">
            <div className="flex justify-between">
              <span>新建笔记</span>
              <span className="font-mono">Ctrl+N</span>
            </div>
            <div className="flex justify-between">
              <span>保存</span>
              <span className="font-mono">Ctrl+S</span>
            </div>
            <div className="flex justify-between">
              <span>切换预览</span>
              <span className="font-mono">Ctrl+D</span>
            </div>
          </div>
        </div>
      </div>

      {notes.length > 0 && (
        <div className="rounded-lg border p-4">
          <h3 className="font-semibold mb-4">最近笔记</h3>
          <div className="space-y-2">
            {notes.slice(0, 5).map((note) => (
              <button
                key={note.id}
                onClick={() => navigate(`/editor/${note.id}`)}
                className="w-full text-left p-3 rounded-lg hover:bg-muted/50 transition-colors"
              >
                <div className="font-medium">{getNoteTitle(note)}</div>
                <div className="text-sm text-muted-foreground truncate">
                  {getPlainText(note.content)}
                </div>
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
