import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Note } from '@/types/note';

interface SearchStore {
  // 面板状态
  isOpen: boolean;
  openSearch: () => void;
  closeSearch: () => void;

  // 搜索状态
  query: string;
  setQuery: (query: string) => void;
  results: Note[];
  setResults: (results: Note[]) => void;
  isSearching: boolean;
  setIsSearching: (isSearching: boolean) => void;

  // 键盘导航
  selectedIndex: number;
  setSelectedIndex: (selectedIndex: number) => void;
  resetSelectedIndex: () => void;
}

export const useSearchStore = create<SearchStore>()(
  persist(
    (set) => ({
      // 面板状态
      isOpen: false,
      openSearch: () => set({ isOpen: true }),
      closeSearch: () =>
        set({
          isOpen: false,
          query: '',
          results: [],
          selectedIndex: -1,
          isSearching: false,
        }),

      // 搜索状态
      query: '',
      setQuery: (query) => set({ query }),
      results: [],
      setResults: (results) => set({ results }),
      isSearching: false,
      setIsSearching: (isSearching) => set({ isSearching }),

      // 键盘导航
      selectedIndex: -1,
      setSelectedIndex: (selectedIndex) => set({ selectedIndex }),
      resetSelectedIndex: () => set({ selectedIndex: -1 }),
    }),
    {
      name: 'search-store',
      // 不持久化任何状态，搜索是临时的
      partialize: () => ({}),
    }
  )
);
