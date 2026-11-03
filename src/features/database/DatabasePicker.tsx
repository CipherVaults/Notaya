// A small searchable list of databases, used by "Linked view of database…" in the "/" menu.

import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Search, Table2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { useApp } from "@/store/app";

export interface DatabasePickerPlacement {
  left: number;
  top: number;
  bottom: number;
}

export default function DatabasePicker({ placement, onPick, onClose }: { placement: DatabasePickerPlacement; onPick: (databaseId: string) => void; onClose: () => void }) {
  const tree = useApp((s) => s.tree);
  const [query, setQuery] = useState("");
  const [index, setIndex] = useState(0);
  const [pos, setPos] = useState<{ left: number; top: number } | null>(null);
  const boxRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const dbs = useMemo(() => {
    const q = query.trim().toLowerCase();
    const sections = new Map(tree.sections.map((x) => [x.id, x]));
    const notebooks = new Map(tree.notebooks.map((x) => [x.id, x]));
    return tree.databases
      .filter((d) => d.trashedAt === null && (!q || (d.title || "Untitled database").toLowerCase().includes(q)))
      .sort((a, b) => b.updated - a.updated)
      .slice(0, 50)
      .map((d) => {
        const sec = sections.get(d.sectionId);
        const nb = sec ? notebooks.get(sec.notebookId) : undefined;
        return { db: d, where: [nb?.name, sec?.name].filter(Boolean).join(" / ") };
      });
  }, [tree, query]);

  useEffect(() => setIndex(0), [query]);

  useLayoutEffect(() => {
    const h = boxRef.current?.offsetHeight ?? 340;
    const below = window.innerHeight - placement.bottom;
    const top = below < h + 16 ? Math.max(8, placement.top - h - 8) : placement.bottom + 6;
    setPos({ left: Math.max(8, Math.min(placement.left, window.innerWidth - 328)), top });
  }, [placement, dbs.length]);

  useEffect(() => {
    listRef.current?.querySelector<HTMLElement>('[data-active="true"]')?.scrollIntoView({ block: "nearest" });
  }, [index]);

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      onClose();
    } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      if (dbs.length) setIndex((i) => (i + (e.key === "ArrowDown" ? 1 : -1) + dbs.length) % dbs.length);
    } else if (e.key === "Enter") {
      e.preventDefault();
      const hit = dbs[Math.min(index, dbs.length - 1)];
      if (hit) onPick(hit.db.id);
    }
  };

  return createPortal(
    <>
      <div className="fixed inset-0 z-[79]" onMouseDown={onClose} />
      <div
        ref={boxRef}
        role="dialog"
        aria-label="Choose a database"
        style={{ left: pos?.left ?? placement.left, top: pos?.top ?? placement.bottom + 6 }}
        className="fixed z-[80] w-[320px] overflow-hidden rounded-xl border border-line bg-elevated shadow-float"
        onKeyDown={onKeyDown}
      >
        <div className="flex items-center gap-2 border-b border-line px-3">
          <Search size={15} className="shrink-0 text-faint" />
          <input
            autoFocus
            dir="auto"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search for a database"
            aria-label="Search databases"
            className="h-10 w-full bg-transparent text-sm text-fg outline-none placeholder:text-faint"
          />
        </div>
        <div ref={listRef} className="max-h-[300px] overflow-y-auto p-1.5">
          {dbs.length === 0 && <div className="px-3 py-6 text-center text-sm text-muted">{query.trim() ? "No databases match." : "There are no databases yet."}</div>}
          {dbs.map(({ db, where }, i) => (
            <button
              key={db.id}
              type="button"
              data-active={i === index}
              onMouseMove={() => setIndex(i)}
              onClick={() => onPick(db.id)}
              className={cn("flex w-full items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-start", i === index ? "bg-surface-2" : "")}
            >
              <span className="grid h-6 w-6 shrink-0 place-items-center text-base leading-none">{db.icon || <Table2 size={16} className="text-muted" />}</span>
              <span className="min-w-0 flex-1">
                <span dir="auto" className="block truncate text-sm text-fg">{db.title || "Untitled database"}</span>
                {where && <span className="block truncate text-xs text-faint">{where}</span>}
              </span>
            </button>
          ))}
        </div>
      </div>
    </>,
    document.body,
  );
}
