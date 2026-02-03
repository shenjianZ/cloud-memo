import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import rehypeHighlight from 'rehype-highlight'
import { cn } from '@/lib/utils'
import 'highlight.js/styles/github-dark.css'

interface MarkdownPreviewProps {
  content: string
  className?: string
}

export function MarkdownPreview({ content, className }: MarkdownPreviewProps) {
  return (
    <div
      className={cn(
        'h-full w-full overflow-y-auto custom-scrollbar p-6',
        'prose prose-dark max-w-none',
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
        className
      )}
    >
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeHighlight]}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}
