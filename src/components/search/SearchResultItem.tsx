import { FileText } from 'lucide-react';
import type { Note } from '@/types/note';
import { getNoteExcerpt, formatSearchTime } from '@/lib/searchHelpers';

interface SearchResultItemProps {
  note: Note;
  isActive: boolean;
  onClick: () => void;
}

export function SearchResultItem({ note, isActive, onClick }: SearchResultItemProps) {
  const excerpt = getNoteExcerpt(note, 80);
  const time = formatSearchTime(note.updatedAt);

  return (
    <div
      onClick={onClick}
      className={`
        group flex items-start gap-3 px-3 py-3 rounded-xl cursor-pointer transition-all
        ${isActive
          ? 'bg-primary/10 border border-primary/20 shadow-sm'
          : 'hover:bg-muted/50 border border-transparent'
        }
      `}
    >
      <FileText className={`w-4 h-4 shrink-0 mt-0.5 ${isActive ? 'text-primary' : 'text-muted-foreground'}`} />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <div className={`text-sm font-medium truncate ${isActive ? 'text-foreground' : ''}`}>{excerpt}</div>
        </div>
        <div className={`text-xs mt-0.5 ${isActive ? 'text-primary/70' : 'text-muted-foreground'}`}>{time}</div>
      </div>
    </div>
  );
}
