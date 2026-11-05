import { useCallback, useEffect, useState } from "react";
import { Command } from "cmdk";
import { History, Trash2, X } from "lucide-react";
import { api } from "@/lib/ipc";

/**
 * The last five searches, shown in the search box while it is empty. They are kept in the encrypted vault
 * (see `commands/search_history.rs`): a click puts one back into the box, the X forgets it, Clear forgets all.
 */
export function useRecentSearches(open: boolean) {
  const [recent, setRecent] = useState<string[]>([]);

  // Read again each time the box opens, so another window's changes show up too.
  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    api
      .searchHistory()
      .then((h) => !cancelled && setRecent(h))
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [open]);

  /** Remembers what was searched for once the person opens a result of it. */
  const remember = useCallback((term: string) => {
    const t = term.trim();
    if (!t) return;
    // Shown at once; the saved list answers with the same result (or, if saving failed, it is just not kept).
    setRecent((cur) => [t, ...cur.filter((x) => x.toLowerCase() !== t.toLowerCase())].slice(0, 5));
    api.addSearchTerm(t).then(setRecent).catch(() => {});
  }, []);

  const forget = useCallback((term: string) => {
    setRecent((cur) => cur.filter((x) => x !== term));
    api.removeSearchTerm(term).then(setRecent).catch(() => {});
  }, []);

  const clear = useCallback(() => {
    setRecent([]);
    api.clearSearchHistory().catch(() => api.searchHistory().then(setRecent).catch(() => {}));
  }, []);

  return { recent, remember, forget, clear };
}

export function RecentSearches({
  recent,
  onPick,
  onForget,
  onClear,
  itemClass,
  headingClass,
}: {
  recent: string[];
  onPick: (term: string) => void;
  onForget: (term: string) => void;
  onClear: () => void;
  itemClass: string;
  headingClass: string;
}) {
  if (recent.length === 0) return null;
  return (
    <Command.Group heading="Recent searches" className={headingClass}>
      {recent.map((term) => (
        <Command.Item key={term} value={`recent-${term}`} onSelect={() => onPick(term)} className={`${itemClass} group/recent`}>
          <span className="text-muted">
            <History size={16} />
          </span>
          <span dir="auto" className="min-w-0 flex-1 truncate">
            {term}
          </span>
          <button
            type="button"
            tabIndex={-1}
            aria-label={`Forget "${term}"`}
            title="Forget this search"
            onClick={(e) => {
              e.stopPropagation();
              onForget(term);
            }}
            className="grid h-6 w-6 place-items-center rounded-md text-faint hover:bg-line hover:text-fg"
          >
            <X size={14} />
          </button>
        </Command.Item>
      ))}
      <Command.Item value="recent-clear" onSelect={onClear} className={itemClass}>
        <span className="text-muted">
          <Trash2 size={16} />
        </span>
        <span className="flex-1">Clear recent searches</span>
      </Command.Item>
    </Command.Group>
  );
}
