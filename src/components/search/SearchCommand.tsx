import { useEffect, useRef } from 'react';
import { Dialog, DialogContent } from '@/components/ui/dialog';
import { useSearchStore } from '@/store/searchStore';
import { useNoteStore } from '@/store/noteStore';
import { useDebounce } from '@/hooks/useDebounce';
import { useNavigate } from 'react-router-dom';
import { SearchInput } from './SearchInput';
import { SearchResultsList } from './SearchResultsList';
import { getNoteTitle } from '@/lib/noteHelpers';
import { useEditorStore } from '@/store/editorStore';

export function SearchCommand() {
  const {
    isOpen,
    closeSearch,
    query,
    results,
    setResults,
    setIsSearching,
    selectedIndex,
    setSelectedIndex,
    resetSelectedIndex,
  } = useSearchStore();

  const searchNotesApi = useNoteStore((state) => state.searchNotesApi);
  const navigate = useNavigate();
  const openNote = useEditorStore((state) => state.openNote);
  const containerRef = useRef<HTMLDivElement>(null);

  // 防抖搜索
  const debouncedQuery = useDebounce(query, 300);

  // 执行搜索
  useEffect(() => {
    const performSearch = async () => {
      if (debouncedQuery.trim()) {
        setIsSearching(true);
        try {
          const searchResults = await searchNotesApi(debouncedQuery);
          setResults(searchResults);
        } catch (error) {
          console.error('搜索失败:', error);
          setResults([]);
        } finally {
          setIsSearching(false);
        }
      } else {
        setResults([]);
      }
    };

    performSearch();
  }, [debouncedQuery, searchNotesApi, setIsSearching, setResults]);

  // 自动选中第一个结果
  useEffect(() => {
    if (results.length > 0 && selectedIndex === -1) {
      setSelectedIndex(0);
    } else if (results.length === 0) {
      setSelectedIndex(-1);
    }
  }, [results, selectedIndex, setSelectedIndex]);

  // 重置选中索引
  useEffect(() => {
    if (!isOpen) {
      resetSelectedIndex();
    }
  }, [isOpen, resetSelectedIndex]);

  // 键盘导航
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown' || e.key === 'Tab') {
      e.preventDefault();
      // 向下选择，到达最后一个后循环回到第一个
      const nextIndex = selectedIndex + 1;
      setSelectedIndex(nextIndex >= results.length ? 0 : nextIndex);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      // 向上选择，到达第一个后循环到最后一个
      const prevIndex = selectedIndex - 1;
      setSelectedIndex(prevIndex < 0 ? results.length - 1 : prevIndex);
    } else if (e.key === 'Enter' && selectedIndex >= 0 && selectedIndex < results.length) {
      e.preventDefault();
      // 打开选中的笔记并跳转
      const selectedNote = results[selectedIndex];
      if (selectedNote) {
        const title = getNoteTitle(selectedNote);
        openNote(selectedNote.id, title);
        navigate(`/editor/${selectedNote.id}`);
        closeSearch();
      }
    }
  };

  // 点击外部关闭
  const handleOpenChange = (open: boolean) => {
    if (!open) {
      closeSearch();
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleOpenChange} withoutOverlay closeOnClickOutside>
      <DialogContent
        ref={containerRef}
        onKeyDown={handleKeyDown}
        className="p-0 w-[480px] h-[440px] overflow-hidden flex flex-col rounded-2xl border border-border bg-background"
        style={{
          boxShadow: '0 0 0 1px rgba(0, 0, 0, 0.05), 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 10px 15px -3px rgba(0, 0, 0, 0.1), -4px 0 6px -1px rgba(0, 0, 0, 0.1), 4px 0 6px -1px rgba(0, 0, 0, 0.1), 0 -4px 6px -1px rgba(0, 0, 0, 0.1)'
        }}
        hideCloseButton
      >
        <SearchInput />
        <SearchResultsList />
      </DialogContent>
    </Dialog>
  );
}
