// A small searchable list of existing pages, used by the "Link to page" entry of the "/" menu.

import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Search } from "lucide-react";
import { cn } from "@/lib/utils";
import { useApp } from "@/store/app";

const MAX_SHOWN = 50;

export interface PickerPlacement {
  left: number;
  top: number;
  bottom: number;
}

export default function PagePicker({
  placement,
  onPick,
  onClose,
  includeCurrent = false,
}: {
  placement: PickerPlacement;
  onPick: (pageId: string) => void;
  onClose: () => void;
  /** Also list the page being edited (to link to one of its headings). */
  includeCurrent?: boolean;
}) {
  const tree = useApp((s) => s.tree);
  const activePageId = useApp((s) => s.activePageId);
  const [query, setQuery] = useState("");
  const [index, setIndex] = useState(0);
  const [pos, setPos] = useState<{ left: number; top: number } | null>(null);
  const boxRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const pages = useMemo(() => {
    const q = query.trim().toLowerCase();
    const sections = new Map(tree.sections.map((x) => [x.id, x]));
    const notebooks = new Map(tree.notebooks.map((x) => [x.id, x]));
    return tree.pages
      .filter((p) => p.trashedAt === null && (includeCurrent || p.id !== activePageId))
      .filter((p) => !q || (p.title || "Untitled").toLowerCase().includes(q))
      .sort((a, b) => {
        if (q) {
          const as = (a.title || "").toLowerCase().startsWith(q) ? 0 : 1;
          const bs = (b.title || "").toLowerCase().startsWith(q) ? 0 : 1;
          if (as !== bs) return as - bs;
        }
        return b.updated - a.updated;
      })
      .slice(0, MAX_SHOWN)
      .map((p) => {
        const sec = sections.get(p.sectionId);
        const nb = sec ? notebooks.get(sec.notebookId) : undefined;
        return { page: p, where: p.id === activePageId ? "This page" : [nb?.name, sec?.name].filter(Boolean).join(" / ") };
      });
  }, [tree, activePageId, query, includeCurrent]);

  useEffect(() => setIndex(0), [query]);

  useLayoutEffect(() => {
    const h = boxRef.current?.offsetHeight ?? 340;
    const below = window.innerHeight - placement.bottom;
    const top = below < h + 16 ? Math.max(8, placement.top - h - 8) : placement.bottom + 6;
    setPos({ left: Math.max(8, Math.min(placement.left, window.innerWidth - 328)), top });
  }, [placement, pages.length]);

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
      if (pages.length) setIndex((i) => (i + (e.key === "ArrowDown" ? 1 : -1) + pages.length) % pages.length);
    } else if (e.key === "Enter") {
      e.preventDefault();
      const hit = pages[Math.min(index, pages.length - 1)];
      if (hit) onPick(hit.page.id);
    }
  };

  return createPortal(
    <>
      <div className="fixed inset-0 z-[79]" onMouseDown={onClose} />
      <div
        ref={boxRef}
        role="dialog"
        aria-label="Link to a page"
        style={{ left: pos?.left ?? placement.left, top: pos?.top ?? placement.bottom + 6 }}
        className="fixed z-[80] w-[320px] overflow-hidden rounded-xl border border-line bg-elevated shadow-float"
        onKeyDown={onKeyDown}
      >
        <div className="flex items-center gap-2 border-b border-line px-3">
          <Search size={15} className="shrink-0 text-faint" />
          <input
            autoFocus
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search for a page to link"
            aria-label="Search pages"
            className="h-10 w-full bg-transparent text-sm text-fg outline-none placeholder:text-faint"
          />
        </div>
        <div ref={listRef} className="max-h-[300px] overflow-y-auto p-1.5">
          {pages.length === 0 && <div className="px-3 py-6 text-center text-sm text-muted">{query.trim() ? "No pages match." : "There are no other pages yet."}</div>}
          {pages.map(({ page, where }, i) => (
            <button
              key={page.id}
              data-active={i === index}
              onMouseMove={() => setIndex(i)}
              onClick={() => onPick(page.id)}
              className={cn("flex w-full items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-left", i === index ? "bg-surface-2" : "")}
            >
              <span className="grid h-6 w-6 shrink-0 place-items-center text-base leading-none">{page.icon || "📄"}</span>
              <span className="min-w-0 flex-1">
                <span className="block truncate text-sm text-fg">{page.title || "Untitled"}</span>
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

export interface HeadingEntry {
  level: number;
  text: string;
  anchor: string | null;
  /** Position in the open page, when the heading is in the page being edited. */
  pos?: number;
}

/** The headings of one page, to link to one of them (or to the page as a whole). */
export function HeadingList({
  placement,
  pageTitle,
  headings,
  onPick,
  onBack,
  onClose,
}: {
  placement: PickerPlacement;
  pageTitle: string;
  headings: HeadingEntry[];
  /** null means the page itself. */
  onPick: (heading: HeadingEntry | null) => void;
  onBack: () => void;
  onClose: () => void;
}) {
  const [index, setIndex] = useState(0);
  const [pos, setPos] = useState<{ left: number; top: number } | null>(null);
  const boxRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const count = headings.length + 1;
  const minLevel = Math.min(4, ...headings.map((h) => h.level));

  useLayoutEffect(() => {
    const h = boxRef.current?.offsetHeight ?? 340;
    const below = window.innerHeight - placement.bottom;
    const top = below < h + 16 ? Math.max(8, placement.top - h - 8) : placement.bottom + 6;
    setPos({ left: Math.max(8, Math.min(placement.left, window.innerWidth - 328)), top });
  }, [placement, count]);

  useEffect(() => {
    boxRef.current?.focus();
  }, []);

  useEffect(() => {
    listRef.current?.querySelector<HTMLElement>('[data-active="true"]')?.scrollIntoView({ block: "nearest" });
  }, [index]);

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      onClose();
    } else if (e.key === "Backspace" || e.key === "ArrowLeft") {
      e.preventDefault();
      onBack();
    } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      setIndex((i) => (i + (e.key === "ArrowDown" ? 1 : -1) + count) % count);
    } else if (e.key === "Enter") {
      e.preventDefault();
      onPick(index === 0 ? null : headings[index - 1]);
    }
  };

  const row = "flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm text-fg";
  return createPortal(
    <>
      <div className="fixed inset-0 z-[79]" onMouseDown={onClose} />
      <div
        ref={boxRef}
        tabIndex={-1}
        role="dialog"
        aria-label="Link to a section"
        style={{ left: pos?.left ?? placement.left, top: pos?.top ?? placement.bottom + 6 }}
        className="fixed z-[80] w-[320px] overflow-hidden rounded-xl border border-line bg-elevated shadow-float outline-none"
        onKeyDown={onKeyDown}
      >
        <div className="flex items-center gap-2 border-b border-line px-3 py-2 text-xs text-muted">
          <button onClick={onBack} aria-label="Back to the page list" className="rounded px-1 text-fg hover:bg-surface-2">
            &larr;
          </button>
          <span className="truncate">{pageTitle || "Untitled"}</span>
        </div>
        <div ref={listRef} className="max-h-[300px] overflow-y-auto p-1.5">
          <button data-active={index === 0} onMouseMove={() => setIndex(0)} onClick={() => onPick(null)} className={cn(row, index === 0 ? "bg-surface-2" : "")}>
            <span className="truncate">The whole page</span>
          </button>
          {headings.length === 0 && <div className="px-3 py-3 text-xs text-faint">This page has no headings yet.</div>}
          {headings.map((h, i) => (
            <button
              key={i}
              data-active={index === i + 1}
              onMouseMove={() => setIndex(i + 1)}
              onClick={() => onPick(h)}
              style={{ paddingLeft: `${0.625 + (h.level - minLevel) * 0.9}rem` }}
              className={cn(row, index === i + 1 ? "bg-surface-2" : "")}
            >
              <span className="w-5 shrink-0 text-[0.7rem] text-faint">H{h.level}</span>
              <span className="truncate">{h.text || "Untitled heading"}</span>
            </button>
          ))}
        </div>
      </div>
    </>,
    document.body,
  );
}
