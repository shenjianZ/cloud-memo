import { useEffect, useRef } from 'react'
import { EditorView } from '@codemirror/view'
import { EditorState } from '@codemirror/state'
import { markdown } from '@codemirror/lang-markdown'
import { languages } from '@codemirror/language-data'
import { oneDark } from '@codemirror/theme-one-dark'
import { keymap } from '@codemirror/view'
import { defaultKeymap, historyKeymap } from '@codemirror/commands'
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search'
import { lineNumbers } from '@codemirror/view'
import { useNoteStore } from '@/store/noteStore'
import { useEditorConfigStore } from '@/store/editorConfigStore'
import { useEditorStore } from '@/store/editorStore'
import { autoSaveService } from '@/services/autoSave'

interface CodeMirrorEditorProps {
  noteId: string
  content: string
  onChange?: (content: string) => void
  onSave?: () => void
  readOnly?: boolean
}

export function CodeMirrorEditor({
  noteId,
  content,
  onChange,
  onSave,
  readOnly = false,
}: CodeMirrorEditorProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const viewRef = useRef<EditorView | null>(null)
  const updateNote = useNoteStore((state) => state.updateNote)
  const { config } = useEditorConfigStore()
  const { setEditorInstance, updateTabDirty } = useEditorStore()

  useEffect(() => {
    if (!containerRef.current) return

    // 创建编辑器状态
    const state = EditorState.create({
      doc: content,
      extensions: [
        lineNumbers(),
        markdown({ codeLanguages: languages }),
        // 主题配置
        config.theme === 'one-dark' ? oneDark : [],
        // 自动换行
        EditorView.theme({
          '&': { fontSize: `${config.fontSize}px` },
          '.cm-content': { fontFamily: 'JetBrains Mono, monospace' },
          '.cm-line': { padding: '0 0' },
        }),
        // 只读模式
        EditorState.readOnly.of(readOnly),
        // 内容变化监听
        EditorView.updateListener.of((update: any) => {
          if (update.docChanged) {
            const newContent = update.state.doc.toString()
            onChange?.(newContent)
            updateTabDirty(noteId, true)

            // 已禁用自动保存，只保留手动保存（Ctrl+S）
            // autoSaveService.queueSave(noteId, newContent, async (id, markdown) => {
            //   await updateNote(id, { markdown })
            //   updateTabDirty(id, false)
            // })
          }
        }),
        // 快捷键
        keymap.of([
          ...defaultKeymap,
          ...historyKeymap,
          ...searchKeymap,
          {
            key: 'Ctrl-s',
            run: () => {
              autoSaveService.saveNow(noteId, viewRef.current?.state.doc.toString() || '', async (id, content) => {
                await updateNote(id, { content })
                updateTabDirty(id, false)
                onSave?.()
              })
              return true
            },
          },
        ]),
        highlightSelectionMatches(),
      ],
    })

    // 创建编辑器视图
    const view = new EditorView({
      state,
      parent: containerRef.current,
    })

    viewRef.current = view

    // 存储编辑器实例到 store
    setEditorInstance(noteId, {
      view,
      containerElement: containerRef.current,
    })

    return () => {
      view.destroy()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [noteId])

  // 更新内容和配置
  useEffect(() => {
    if (!viewRef.current) return

    const view = viewRef.current

    // 更新内容（仅当内容真正改变时）
    if (view.state.doc.toString() !== content) {
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: content,
        },
      })
    }
  }, [content])

  return (
    <div
      ref={containerRef}
      className="h-full w-full overflow-hidden"
      style={{ fontSize: `${config.fontSize}px` }}
    />
  )
}
