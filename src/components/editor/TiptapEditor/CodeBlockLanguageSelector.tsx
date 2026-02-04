import { useState, useEffect, useRef } from 'react'
import { Code, ChevronDown } from 'lucide-react'
import type { Editor } from '@tiptap/react'

// 支持的编程语言列表
const SUPPORTED_LANGUAGES = [
  { name: '自动检测', value: 'plaintext' },
  { name: 'JavaScript', value: 'javascript' },
  { name: 'TypeScript', value: 'typescript' },
  { name: 'Python', value: 'python' },
  { name: 'Java', value: 'java' },
  { name: 'C/C++', value: 'cpp' },
  { name: 'C#', value: 'csharp' },
  { name: 'Go', value: 'go' },
  { name: 'Rust', value: 'rust' },
  { name: 'Ruby', value: 'ruby' },
  { name: 'PHP', value: 'php' },
  { name: 'Swift', value: 'swift' },
  { name: 'Kotlin', value: 'kotlin' },
  { name: 'Dart', value: 'dart' },
  { name: 'HTML', value: 'html' },
  { name: 'CSS', value: 'css' },
  { name: 'SCSS', value: 'scss' },
  { name: 'JSON', value: 'json' },
  { name: 'XML', value: 'xml' },
  { name: 'SQL', value: 'sql' },
  { name: 'Bash', value: 'bash' },
  { name: 'Shell', value: 'shell' },
  { name: 'PowerShell', value: 'powershell' },
  { name: 'Markdown', value: 'markdown' },
  { name: 'YAML', value: 'yaml' },
  { name: 'TOML', value: 'toml' },
  { name: 'Dockerfile', value: 'dockerfile' },
]

interface CodeBlockLanguageSelectorProps {
  editor: Editor | null
}

/**
 * 代码块语言选择器
 * 当光标在代码块中时，在工具栏下方显示语言选择栏
 */
export function CodeBlockLanguageSelector({ editor }: CodeBlockLanguageSelectorProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [currentLanguage, setCurrentLanguage] = useState('plaintext')
  const [searchQuery, setSearchQuery] = useState('')
  const [isInCodeBlock, setIsInCodeBlock] = useState(false)
  const menuRef = useRef<HTMLDivElement>(null)

  // 获取当前代码块的语言
  useEffect(() => {
    if (!editor) return

    const updateCurrentLanguage = () => {
      const { state } = editor
      const { selection } = state
      const { $from } = selection

      // 检查是否在代码块中
      const inCodeBlock = $from.parent.type.name === 'codeBlock'
      setIsInCodeBlock(inCodeBlock)

      if (inCodeBlock) {
        const language = $from.parent.attrs.language || 'plaintext'
        setCurrentLanguage(language)
      }
    }

    // 初始化
    updateCurrentLanguage()

    // 监听选择变化
    editor.on('selectionUpdate', updateCurrentLanguage)
    editor.on('transaction', updateCurrentLanguage)

    return () => {
      editor.off('selectionUpdate', updateCurrentLanguage)
      editor.off('transaction', updateCurrentLanguage)
    }
  }, [editor])

  // 点击外部关闭菜单
  useEffect(() => {
    if (!isOpen) return

    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false)
        setSearchQuery('')
      }
    }

    document.addEventListener('mousedown', handleClickOutside)

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [isOpen])

  if (!editor || !isInCodeBlock) return null

  const filteredLanguages = SUPPORTED_LANGUAGES.filter((lang) =>
    lang.name.toLowerCase().includes(searchQuery.toLowerCase())
  )

  const handleLanguageSelect = (languageValue: string) => {
    editor
      .chain()
      .focus()
      .updateAttributes('codeBlock', { language: languageValue })
      .run()
    setCurrentLanguage(languageValue)
    setIsOpen(false)
    setSearchQuery('')
  }

  const getCurrentLanguageName = () => {
    const lang = SUPPORTED_LANGUAGES.find((l) => l.value === currentLanguage)
    return lang?.name || '自动检测'
  }

  return (
    <div className="border-b border-border bg-muted/30 px-4 py-2" ref={menuRef}>
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Code className="w-4 h-4 text-muted-foreground" />
          <span className="text-sm font-medium">代码语言:</span>
          <span className="text-sm text-primary font-semibold">{getCurrentLanguageName()}</span>
        </div>

        <button
          onClick={() => setIsOpen(!isOpen)}
          className="flex items-center gap-1 px-2 py-1 text-xs bg-background border border-border rounded hover:bg-muted transition-colors"
        >
          切换语言
          <ChevronDown className={`w-3 h-3 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
        </button>
      </div>

      {/* 语言选择菜单 */}
      {isOpen && (
        <div className="mt-2 bg-background border border-border rounded-lg shadow-lg p-2">
          {/* 搜索框 */}
          <div className="mb-2">
            <input
              type="text"
              placeholder="搜索语言..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full px-3 py-2 text-sm bg-background border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-ring"
              autoFocus
            />
          </div>

          {/* 语言列表 */}
          <div className="max-h-48 overflow-y-auto grid grid-cols-3 gap-1">
            {filteredLanguages.length === 0 ? (
              <div className="col-span-3 px-3 py-8 text-sm text-muted-foreground text-center">
                没有找到匹配的语言
              </div>
            ) : (
              filteredLanguages.map((lang) => (
                <button
                  key={lang.value}
                  onClick={() => handleLanguageSelect(lang.value)}
                  className={`px-2 py-1.5 text-xs text-left rounded transition-colors ${
                    lang.value === currentLanguage
                      ? 'bg-primary text-primary-foreground font-medium'
                      : 'hover:bg-muted'
                  }`}
                >
                  {lang.name}
                </button>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  )
}
