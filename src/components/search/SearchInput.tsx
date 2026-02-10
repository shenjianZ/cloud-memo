import { useEffect, useRef } from 'react';
import { Search, X } from 'lucide-react';
import { useSearchStore } from '@/store/searchStore';

export function SearchInput() {
  const { query, setQuery, closeSearch, isOpen } = useSearchStore();
  const inputRef = useRef<HTMLInputElement>(null);

  // 自动聚焦到输入框
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  const handleClear = () => {
    setQuery('');
    if (inputRef.current) {
      inputRef.current.focus();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Escape') {
      closeSearch();
    }
    // 方向键和Enter键不阻止默认行为，让事件冒泡到父组件处理
    // 这样可以在输入框中直接使用方向键选择结果，Enter确认
  };

  return (
    <div className="flex items-center gap-2 px-4 h-16 border-b border-border">
      <Search className="w-4 h-4 text-muted-foreground shrink-0" />
      <input
        ref={inputRef}
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="搜索笔记..."
        className="flex-1 bg-transparent border-none outline-none text-sm text-foreground placeholder:text-muted-foreground"
      />
      {query && (
        <button
          onClick={handleClear}
          className="p-1 rounded-full hover:bg-muted transition-colors"
          aria-label="清除搜索"
        >
          <X className="w-4 h-4 text-muted-foreground" />
        </button>
      )}
    </div>
  );
}
