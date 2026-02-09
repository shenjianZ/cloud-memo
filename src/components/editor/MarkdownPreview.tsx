import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkMath from 'remark-math'
import rehypeHighlight from 'rehype-highlight'
import rehypeKatex from 'rehype-katex'
import rehypeRaw from 'rehype-raw'
import { cn } from '@/lib/utils'
import type { MarkdownPreviewStyle } from '@/services/editorSettingsApi'
// 导入 KaTeX 样式
import 'katex/dist/katex.min.css'
// highlight.js 样式已在 main.tsx 中导入

interface MarkdownPreviewProps {
  content: string
  className?: string
  styleMode?: MarkdownPreviewStyle
}

/**
 * 根据样式模式获取 prose 类名
 */
function getProseClasses(styleMode: MarkdownPreviewStyle = 'default'): string {
  switch (styleMode) {
    case 'minimal':
      // 朴素模式：最小化样式，只保留基本格式
      return cn(
        'max-w-none',
        // 基础样式 - 极简
        'prose-p:my-1 prose-p:leading-normal',
        'prose-h1:my-2 prose-h1:text-xl prose-h1:font-semibold',
        'prose-h2:my-2 prose-h2:text-lg prose-h2:font-semibold',
        'prose-h3:my-1 prose-h3:text-base prose-h3:font-medium',
        'prose-ul:list-disc prose-ul:my-1 prose-ul:pl-4',
        'prose-ol:list-decimal prose-ol:my-1 prose-ol:pl-4',
        'prose-li:my-0.5',
        // 代码样式 - 简单
        'prose-code:text-sm prose-code:bg-muted prose-code:px-1 prose-code:py-0.5 prose-code:rounded',
        'prose-pre:bg-muted prose-pre:border prose-pre:border-border',
        // 链接样式 - 基础
        'prose-a:text-primary prose-a:underline',
        // 引用样式 - 简单
        'prose-blockquote:border-l-2 prose-blockquote:border-muted-foreground prose-blockquote:pl-3',
        // 表格样式 - 基础
        'prose-table:border-collapse prose-table:border prose-table:border-border',
        'prose-th:border prose-th:border-border prose-th:bg-muted prose-th:px-2 prose-th:py-1',
        'prose-td:border prose-td:border-border prose-td:px-2 prose-td:py-1',
      )

    case 'rich':
      // 丰富模式：更多视觉效果
      return cn(
        'max-w-none',
        // 段落样式 - 更丰富的间距
        'prose-p:my-3 prose-p:leading-8 prose-p:text-muted-foreground',
        // 标题样式 - 渐变色和装饰
        'prose-headings:font-bold prose-headings:text-foreground',
        'prose-h1:my-6 prose-h1:text-4xl prose-h1:tracking-tight',
        'prose-h2:my-5 prose-h2:text-3xl prose-h2:tracking-tight',
        'prose-h3:my-4 prose-h3:text-2xl',
        'prose-h4:my-3 prose-h4:text-xl',
        'prose-h5:my-2 prose-h5:text-lg',
        'prose-h6:my-2 prose-h6:text-base',
        // 列表样式 - 彩色标记
        'prose-ul:list-disc prose-ul:my-4 prose-ul:pl-6',
        'prose-ol:list-decimal prose-ol:my-4 prose-ol:pl-6',
        'prose-li:my-2 prose-li:marker:text-primary',
        // 代码样式 - macOS 风格
        'prose-code:text-primary prose-code:bg-muted prose-code:px-2 prose-code:py-1 prose-code:rounded prose-code:font-semibold',
        // macOS 风格代码块 - 无边框无阴影，带红黄绿三点
        'prose-pre:bg-muted prose-pre:rounded-t-xl prose-pre:rounded-b prose-pre:my-4 prose-pre:pt-6 prose-pre:px-4 prose-pre:pb-4 prose-pre:shadow-none prose-pre:relative prose-pre:border-0',
        // 链接样式 - 悬停效果
        'prose-a:text-primary prose-a:no-underline hover:prose-a:underline hover:prose-a:text-primary/80',
        // 引用样式 - 带背景色
        'prose-blockquote:border-l-4 prose-blockquote:border-primary prose-blockquote:bg-primary/5 prose-blockquote:py-4 prose-blockquote:px-4 prose-blockquote:rounded-r-lg prose-blockquote:italic prose-blockquote:text-foreground',
        // 表格样式 - 带斑马纹
        'prose-table:my-4 prose-table:w-full prose-table:border-collapse prose-table:rounded-lg prose-table:overflow-hidden',
        'prose-thead:bg-primary/10',
        'prose-th:border prose-th:border-border prose-th:px-4 prose-th:py-3 prose-th:font-semibold prose-th:text-left',
        'prose-td:border prose-td:border-border prose-td:px-4 prose-td:py-3',
        'prose-tr:bg-muted/30 prose-tr:even:bg-muted/50',
        // 图片样式 - 不加阴影
        'prose-img:rounded-xl prose-img:my-6',
        // 分隔线样式 - 渐变
        'prose-hr:my-8 prose-hr:border-t-2 prose-hr:border-border prose-hr:border-dashed',
      )

    case 'default':
    default:
      // 默认模式：平衡的样式
      return cn(
        'max-w-none',
        'prose-headings:font-bold prose-headings:text-foreground',
        'prose-h1:text-3xl prose-h1:mt-8 prose-h1:mb-4',
        'prose-h2:text-2xl prose-h2:mt-6 prose-h2:mb-3',
        'prose-h3:text-xl prose-h3:mt-4 prose-h3:mb-2',
        'prose-p:text-muted-foreground prose-p:leading-7',
        'prose-a:text-primary prose-a:no-underline hover:prose-a:underline',
        'prose-strong:text-foreground prose-strong:font-semibold',
        'prose-code:text-primary prose-code:bg-muted prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded',
        'prose-pre:bg-muted prose-pre:border prose-pre:border-border',
        'prose-blockquote:border-l-4 prose-blockquote:border-primary prose-blockquote:pl-4 prose-blockquote:italic',
        'prose-ul:list-disc prose-ul:pl-6',
        'prose-ol:list-decimal prose-ol:pl-6',
        'prose-li:marker:text-muted-foreground',
        'prose-img:rounded-lg prose-img:shadow-lg',
        'prose-hr:border-border',
      )
  }
}

export function MarkdownPreview({ content, className, styleMode = 'default' }: MarkdownPreviewProps) {
  const proseClasses = getProseClasses(styleMode)

  return (
    <div
      className={cn(
        'h-full w-full overflow-y-auto overflow-x-hidden custom-scrollbar p-6',
        // 根据模式选择是否使用 prose 插件
        styleMode === 'minimal' ? '' : 'prose dark:prose-invert',
        proseClasses,
        className
      )}
    >
      <ReactMarkdown
        remarkPlugins={[
          remarkGfm,
          [remarkMath, { singleDollarTextMath: true }]
        ]}
        rehypePlugins={[
          // 1. 必须首先运行 KaTeX，将 math 节点转换为 HTML 标签
          [rehypeKatex, {
            strict: false,        // 关闭严格模式，避免换行符警告
            throwOnError: false,  // 遇到错误不抛出异常
            trust: true,          // 允许更多 LaTeX 命令
            // 移除 displayMode: true，让 remark-math 根据语法自动检测是块级还是行内
          }],
          // 2. 然后运行 Raw，处理 HTML 混合
          rehypeRaw,
          // 3. 最后运行高亮
          rehypeHighlight,
        ]}
        skipHtml={false}
        unwrapDisallowed={false}
        components={{
          // 图片组件 - 确保图片正确渲染
          img({ node, ...props }: any) {
            return (
              <img
                {...props}
                className={cn(
                  'max-w-full h-auto',
                  styleMode === 'minimal' && 'rounded my-4',
                  styleMode === 'rich' && 'rounded-xl my-6',
                  styleMode === 'default' && 'rounded-lg shadow-lg my-4'
                )}
                alt={props.alt || ''}
                loading="lazy"
              />
            )
          },
          // macOS 风格代码块组件（仅 rich 模式）
          ...(styleMode === 'rich' ? {
            pre({ node, children, ...props }: any) {
              return (
                <pre {...props}>
                  <span className="absolute top-3 left-3 flex gap-1.5">
                    <span className="w-2 h-2 rounded-full bg-red-500"></span>
                    <span className="w-2 h-2 rounded-full bg-yellow-500"></span>
                    <span className="w-2 h-2 rounded-full bg-green-500"></span>
                  </span>
                  {children}
                </pre>
              )
            },
          } : {}),
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}
