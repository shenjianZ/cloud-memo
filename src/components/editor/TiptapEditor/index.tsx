import { useEffect, useRef, useState } from 'react'
import { EditorContent } from '@tiptap/react'
import { useTiptapEditor } from './useTiptapEditor'
import { BubbleMenu } from './BubbleMenu'
import { SlashCommandMenu } from './SlashCommandMenu'
import { CodeBlockLanguageSelector } from './CodeBlockLanguageSelector'
import { Toolbar } from './Toolbar'
import { MarkdownPreview } from '../MarkdownPreview'
import { NoteMeta } from '@/components/editor/NoteMeta'
import { TagPopover } from '@/components/tags/TagPopover'
import { NoteMoreMenu } from '@/components/editor/NoteMoreMenu'
import './tiptap-editor.css'
import { useNoteStore } from '@/store/noteStore'
import { useTiptapStore } from '@/store/tiptapStore'
import { useEditorFontSettings } from '@/hooks/useEditorFontSettings'
import type { TiptapContent } from '@/types/note'
import { Loader2, Edit, Eye, Split } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { tiptapJsonToMarkdown } from '@/lib/tiptapMarkdown'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

interface TiptapEditorProps {
  noteId: string
  content: TiptapContent
}

/**
 * Tiptap 编辑器主组件
 * 包含工具栏、编辑区、Bubble Menu 和 Slash Command Menu
 * 支持编辑/预览/分屏三种视图模式
 */
export function TiptapEditor({ noteId, content }: TiptapEditorProps) {
  const { updateNote, getNote, setNoteTags } = useNoteStore()
  const { setEditor, clearEditor, updateCounts, viewMode, setViewMode } = useTiptapStore()
  const saveTimeoutRef = useRef<NodeJS.Timeout | undefined>(undefined)
  const [selectedTags, setSelectedTags] = useState<string[]>([])
  const [showTagPopover, setShowTagPopover] = useState(false)
  const [showMoreMenu, setShowMoreMenu] = useState(false)

  // 应用编辑器字体设置
  useEditorFontSettings()

  // 获取当前笔记
  const currentNote = getNote(noteId)

  // 加载笔记的标签
  useEffect(() => {
    if (currentNote) {
      setSelectedTags(currentNote.tags || [])
    }
  }, [noteId, currentNote])

  // 处理标签变更
  const handleTagsChange = async (tagIds: string[]) => {
    setSelectedTags(tagIds)
    await setNoteTags(noteId, tagIds)
  }

  const editor = useTiptapEditor({
    content,
    onUpdate: (json, text) => {
      // 更新字符统计
      const wordCount = text.split(/\s+/).filter(Boolean).length
      const characterCount = text.length
      updateCounts(wordCount, characterCount)

      // 防抖自动保存
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current)
      }

      saveTimeoutRef.current = setTimeout(async () => {
        try {
          await updateNote(noteId, { content: json })
        } catch (error) {
          console.error('Failed to save note:', error)
        }
      }, 1000) // 1秒防抖
    },
    editable: viewMode === 'edit' || viewMode === 'split',
  })

  // 注册/注销编辑器实例
  useEffect(() => {
    if (editor) {
      setEditor(editor)
    }
    return () => {
      clearEditor()
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current)
      }
    }
  }, [editor, setEditor, clearEditor])

  // 将 Tiptap JSON 转换为 Markdown 用于预览
  const markdownContent = tiptapJsonToMarkdown(content)

  // 视图模式图标
  const viewModeIcons = {
    edit: Edit,
    preview: Eye,
    split: Split,
  }

  const CurrentIcon = viewModeIcons[viewMode]

  if (!editor) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full bg-background">
      {/* 工具栏 */}
      <div className="border-b border-border bg-card">
        <Toolbar
          editor={editor}
          onToggleTags={() => setShowTagPopover(!showTagPopover)}
          onToggleMore={() => setShowMoreMenu(!showMoreMenu)}
        />
        {/* 代码块语言选择器 - 当光标在代码块中时显示 */}
        <CodeBlockLanguageSelector editor={editor} />
      </div>

      {/* 内容区域 */}
      <div className="flex-1 overflow-hidden flex flex-col relative">
        {/* 元信息栏 - 显示作者、创建时间、更新时间、标签 */}
        {currentNote && (
          <NoteMeta
            createdAt={currentNote.createdAt}
            updatedAt={currentNote.updatedAt}
            author={currentNote.author}
            tags={selectedTags}
          />
        )}

        {/* 编辑器/预览内容 */}
        <div className="flex-1 overflow-hidden">
          {viewMode === 'edit' && (
            <div className="h-full overflow-y-auto">
              <EditorContent editor={editor} />
            </div>
          )}

          {viewMode === 'preview' && (
            <MarkdownPreview content={markdownContent} className="h-full" />
          )}

          {viewMode === 'split' && (
            <div className="flex h-full">
              <div className="flex-1 overflow-y-auto border-r border-border">
                <EditorContent editor={editor} />
              </div>
              <div className="flex-1 overflow-y-auto">
                <MarkdownPreview content={markdownContent} className="h-full" />
              </div>
            </div>
          )}
        </div>

        {/* 标签弹窗 */}
        {showTagPopover && (
          <TagPopover
            noteId={noteId}
            selectedTags={selectedTags}
            onTagsChange={handleTagsChange}
            onClose={() => setShowTagPopover(false)}
          />
        )}

        {/* 更多菜单 */}
        {showMoreMenu && (
          <NoteMoreMenu
            noteId={noteId}
            content={content}
            onClose={() => setShowMoreMenu(false)}
          />
        )}
      </div>

      {/* 状态栏 */}
      <div className="h-8 border-t border-border flex items-center justify-between px-4 text-xs text-muted-foreground bg-muted/30">
        <div>自动保存已启用</div>

        {/* 视图模式切换 */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="sm" className="h-6 gap-1">
              <CurrentIcon className="w-3 h-3" />
              <span className="capitalize">
                {viewMode === 'edit' && '编辑'}
                {viewMode === 'preview' && '预览'}
                {viewMode === 'split' && '分屏'}
              </span>
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => setViewMode('edit')}>
              <Edit className="w-4 h-4 mr-2" />
              编辑模式
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => setViewMode('preview')}>
              <Eye className="w-4 h-4 mr-2" />
              预览模式
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => setViewMode('split')}>
              <Split className="w-4 h-4 mr-2" />
              分屏模式
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {/* Bubble Menu - 选中时显示（仅在编辑模式下） */}
      {(viewMode === 'edit' || viewMode === 'split') && <BubbleMenu editor={editor} />}

      {/* Slash Command Menu（仅在编辑模式下） */}
      {(viewMode === 'edit' || viewMode === 'split') && <SlashCommandMenu editor={editor} />}
    </div>
  )
}
