import type { Note } from '@/types/note';
import { getNoteTitle } from '@/lib/noteHelpers';

/**
 * 从 Tiptap JSON 内容中提取纯文本
 */
function getPlainText(content: any): string {
  if (typeof content === 'string') {
    return content;
  }

  if (!content || typeof content !== 'object') {
    return '';
  }

  // 处理 Tiptap JSON 结构
  if (content.type === 'doc' && Array.isArray(content.content)) {
    return extractTextFromNodes(content.content);
  }

  if (Array.isArray(content.content)) {
    return extractTextFromNodes(content.content);
  }

  return '';
}

/**
 * 递归提取节点中的文本
 */
function extractTextFromNodes(nodes: any[]): string {
  let text = '';
  for (const node of nodes) {
    if (node.text) {
      text += node.text;
    }
    if (node.content) {
      text += extractTextFromNodes(node.content);
    }
  }
  return text;
}

/**
 * 高亮匹配关键词
 * @param text 原始文本
 * @param query 搜索关键词
 * @returns 带有高亮标记的 HTML 字符串
 */
export function highlightMatch(text: string, query: string): string {
  if (!query.trim()) {
    return text;
  }

  const regex = new RegExp(`(${escapeRegex(query)})`, 'gi');
  return text.replace(regex, '<mark>$1</mark>');
}

/**
 * 转义正则表达式特殊字符
 */
function escapeRegex(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * 格式化时间显示
 * @param timestamp 时间戳（毫秒）
 * @returns 格式化的时间字符串
 */
export function formatSearchTime(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;

  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) {
    return '刚刚';
  } else if (minutes < 60) {
    return `${minutes} 分钟前`;
  } else if (hours < 24) {
    return `${hours} 小时前`;
  } else if (days < 7) {
    return `${days} 天前`;
  } else {
    const date = new Date(timestamp);
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
    });
  }
}

/**
 * 获取笔记摘要
 * @param note 笔记对象
 * @param maxLength 最大长度
 * @returns 摘要文本
 */
export function getNoteExcerpt(note: Note, maxLength: number = 100): string {
  const title = getNoteTitle(note);
  const content = getPlainText(note.content);

  // 如果标题不为空且不是"未命名笔记"，返回标题
  if (title && title !== '未命名笔记') {
    return title;
  }

  // 否则返回内容摘要
  if (content.length > maxLength) {
    return content.substring(0, maxLength) + '...';
  }

  return content || '空笔记';
}

/**
 * 检查笔记是否匹配搜索关键词
 * @param note 笔记对象
 * @param query 搜索关键词
 * @returns 是否匹配
 */
export function isNoteMatch(note: Note, query: string): boolean {
  if (!query.trim()) {
    return true;
  }

  const title = getNoteTitle(note).toLowerCase();
  const content = getPlainText(note.content).toLowerCase();
  const searchTerm = query.toLowerCase();

  return title.includes(searchTerm) || content.includes(searchTerm);
}
