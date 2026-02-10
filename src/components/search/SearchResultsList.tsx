import { useSearchStore } from '@/store/searchStore';
import { SearchResultItem } from './SearchResultItem';
import { useEditorStore } from '@/store/editorStore';
import { useNavigate } from 'react-router-dom';
import { getNoteTitle } from '@/lib/noteHelpers';

export function SearchResultsList() {
  const { results, selectedIndex, closeSearch } = useSearchStore();
  const openNote = useEditorStore((state) => state.openNote);
  const navigate = useNavigate();

  const handleSelectNote = (noteId: string) => {
    // 在新标签页中打开笔记
    const note = results.find(n => n.id === noteId);
    if (note) {
      const title = getNoteTitle(note);
      openNote(noteId, title);
      // 跳转到编辑器路由
      navigate(`/editor/${noteId}`);
    }
    closeSearch();
  };

  if (results.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground text-sm px-4">
        <div className="text-center">
          <p>没有找到匹配的笔记</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto search-scroll py-2">
      <div className="px-2 space-y-1">
        {results.map((note, index) => (
          <SearchResultItem
            key={note.id}
            note={note}
            isActive={selectedIndex === index}
            onClick={() => handleSelectNote(note.id)}
          />
        ))}
      </div>
    </div>
  );
}
