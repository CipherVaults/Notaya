// Gallery view: a grid of cards with an optional cover picture (the first image of a files property),
// the title and the visible properties. Groups become sections. The last card is "+ New".

import { memo, useCallback, useContext, useLayoutEffect, useMemo, useRef, useState } from "react";
import { defaultRangeExtractor, observeElementRect, useVirtualizer, type Range, type VirtualItem } from "@tanstack/react-virtual";
import { ChevronDown, ChevronRight, Image as ImageIcon, LayoutGrid, Plus } from "lucide-react";
import type { DbRow, GroupOut, Property } from "@/lib/database-types";
import { DbShellContext } from "../context";
import type { ViewProps } from "../viewTypes";
import { cn } from "@/lib/utils";
import { EmptyState, GroupChip, NewTitleInput, PropLines, RowTitle, firstImage, groupPatch, groupProp, imageUrl, propOf, setCollapsed, titleMap, useViewRows, visibleProps } from "./shared";

const MIN_W: Record<string, number> = { small: 168, medium: 236, large: 320 };
const COVER_H: Record<string, number> = { small: 92, medium: 132, large: 190 };
const GRID_GAP = 12;
const SCROLL_HEIGHT_BUDGET = 24_000_000;

type GalleryItem = { key: string; group: GroupOut | null } & (
  | { kind: "header"; group: GroupOut; closed: boolean }
  | { kind: "cards"; first: number; ids: readonly string[]; footer: boolean; gap: number }
);

export default function GalleryView({ db, view, result, rowById, rowTitles, tableRowIds, loadTableRows, openRow, createRow, updateView }: ViewProps) {
  const compact = useContext(DbShellContext)?.compact;
  const size = MIN_W[view.cardSize] ? view.cardSize : "medium";
  const g = view.group;
  const gp = groupProp(db, g);
  const coverProp = propOf(db, view.cardCover);
  const cover = coverProp?.kind === "files" ? coverProp.id : "";
  const wantsCover = view.cardCover !== "none";
  const shown = useMemo(() => visibleProps(db, view, [g?.prop]), [db, view, g?.prop]);
  const titles = useMemo(() => rowTitles ?? titleMap(result.rows), [rowTitles, result.rows]);
  const [adding, setAdding] = useState<string | null>(null);
  const [width, setWidth] = useState(0);
  const host = useRef<HTMLDivElement>(null);
  const content = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    const el = content.current;
    if (!el) return;
    const sync = () => setWidth(el.clientWidth);
    sync();
    const observer = new ResizeObserver(sync);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  const measured = width > 0;
  const columns = Math.max(1, Math.floor((width + GRID_GAP) / (MIN_W[size] + GRID_GAP)));
  const gridStyle = { gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`, gap: GRID_GAP };
  const orderedIds = useMemo(() => tableRowIds ?? result.rows.map((row) => row.id), [tableRowIds, tableRowIds ? undefined : result.rows]);
  const groups = g ? result.groups : null;
  const empty = !orderedIds.length && !groups?.some((group) => group.count);
  const { items, footerIndices } = useMemo(() => {
    const items: GalleryItem[] = [];
    const footerIndices = new Map<string, number>();
    if (!measured || empty) return { items, footerIndices };
    const collapsed = new Set(g?.collapsed);
    const cards = (ids: readonly string[], group: GroupOut | null, prefix: string) => {
      // The extra slot packs the footer into the last card row when there is room.
      for (let first = 0; first <= ids.length; first += columns) {
        const footer = first + columns > ids.length;
        if (footer) footerIndices.set(prefix, items.length);
        const gap = footer ? (group ? 20 : 0) : GRID_GAP;
        items.push({ kind: "cards", group, first, ids, footer, gap, key: `${prefix}:cards:${first}` });
      }
    };
    if (groups) {
      for (const group of groups) {
        const prefix = `g:${group.key}`;
        const closed = collapsed.has(group.key);
        items.push({ kind: "header", group, closed, key: `${prefix}:header` });
        if (!closed) cards(group.rows, group, prefix);
      }
    } else cards(orderedIds, null, "all");
    return { items, footerIndices };
  }, [columns, empty, g?.collapsed, groups, measured, orderedIds]);
  const coverHeight = wantsCover ? COVER_H[size] : 0;
  const estimateCardHeight = 46 + coverHeight + shown.length * 20;
  const newHeight = wantsCover ? Math.max(80, coverHeight / 2 + 48) : 80;
  // At one card per row, a large gallery can exceed the browser's maximum
  // scroll height. Compress scrollbar coordinates and paint cards at normal size.
  const scale = Math.min(1, SCROLL_HEIGHT_BUDGET / (Math.max(80, estimateCardHeight) + 24) / Math.max(1, items.length));
  const scaleRef = useRef(scale);
  scaleRef.current = scale;
  const refreshViewport = useRef<(() => void) | null>(null);
  const footerIndex = adding === null ? undefined : footerIndices.get(adding);
  const footerItem = footerIndex === undefined ? undefined : items[footerIndex];
  const footerRowCount = footerItem?.kind === "cards" ? footerItem.ids.length : 0;
  const getScrollElement = useCallback(() => host.current, []);
  const getItemKey = useCallback((i: number) => items[i].key, [items]);
  const estimateSize = useCallback((i: number) => {
    const item = items[i];
    if (item.kind === "header") return (item.closed ? 60 : 40) * scale;
    return ((item.first === item.ids.length ? newHeight : estimateCardHeight) + item.gap) * scale;
  }, [estimateCardHeight, items, newHeight, scale]);
  const rangeExtractor = useCallback((range: Range) => {
    const indices = defaultRangeExtractor(range);
    if (footerIndex !== undefined && !indices.includes(footerIndex)) return [...indices, footerIndex].sort((a, b) => a - b);
    return indices;
  }, [footerIndex]);
  const virtualizer = useVirtualizer<HTMLDivElement, HTMLDivElement>({
    count: items.length, getScrollElement, getItemKey, estimateSize, rangeExtractor,
    enabled: !empty && measured, useFlushSync: false, overscan: 4, paddingStart: 8 * scale, paddingEnd: 32 * scale,
    observeElementRect: (instance, callback) => {
      const cleanup = observeElementRect(instance, (rect) => {
        refreshViewport.current = () => callback({ ...rect, height: rect.height * scaleRef.current });
        refreshViewport.current();
      });
      return () => {
        refreshViewport.current = null;
        cleanup?.();
      };
    },
    measureElement: (el, entry) => (entry?.borderBoxSize?.[0]?.blockSize ?? el.offsetHeight) * scale,
  });
  useLayoutEffect(() => {
    virtualizer.measure();
    content.current?.querySelectorAll<HTMLDivElement>("[data-index]").forEach(virtualizer.measureElement);
  }, [virtualizer, columns, estimateCardHeight, scale]);
  useLayoutEffect(() => {
    if (footerIndex === undefined) return;
    // Creating a row can repack the open editor into a different card row.
    // Align through the virtualizer because native focus scrolls in physical pixels.
    const frame = requestAnimationFrame(() => {
      const viewport = host.current?.getBoundingClientRect();
      const field = host.current?.querySelector(`[data-index="${footerIndex}"] input`)?.getBoundingClientRect();
      if (viewport && field && (scale < 1 || field.top < viewport.top || field.bottom > viewport.bottom)) {
        virtualizer.scrollToIndex(footerIndex, { align: "start" });
      }
    });
    return () => cancelAnimationFrame(frame);
  }, [footerIndex, footerRowCount, scale, virtualizer]);
  const virtualItems = virtualizer.getVirtualItems();
  useViewRows(loadTableRows, virtualItems.flatMap(({ index }) => {
    const item = items[index];
    return item.kind === "cards" ? item.ids.slice(item.first, item.first + columns) : [];
  }));

  const setScrollPosition = (el: HTMLDivElement) => el.style.setProperty("--gallery-scroll", `${el.scrollTop}px`);
  useLayoutEffect(() => {
    refreshViewport.current?.();
    if (host.current) setScrollPosition(host.current);
  }, [scale]);
  const translateTop = (start: number) => scale === 1
    ? start
    : `calc(${start}px + (${start}px - var(--gallery-scroll, 0px)) * ${1 / scale - 1})`;
  // The virtualizer's viewport is scaled, while the native scrollbar uses the
  // actual viewport. Extend the spacer by the difference so the last row is reachable.
  const viewportPadding = (virtualizer.scrollRect?.height ?? 0) * (1 / scale - 1);

  const cellsFor = (gr: GroupOut | null): Record<string, unknown> => {
    if (!gr || !g) return {};
    const patch = groupPatch(gp, g, gr.key, gr.label) ?? {};
    return Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null));
  };

  const renderNew = (key: string, group: GroupOut | null) => adding === key ? (
    <div className="flex min-h-[5rem] items-start rounded-xl border border-dashed border-accent p-2">
      <NewTitleInput keepOpen onSubmit={(t) => void createRow(cellsFor(group), t)} onCancel={() => setAdding((a) => (a === key ? null : a))} />
    </div>
  ) : (
    <button
      type="button"
      onClick={() => setAdding(key)}
      data-new-card
      className="flex min-h-[5rem] items-center justify-center gap-1.5 rounded-xl border border-dashed border-line text-[0.82rem] text-muted transition-colors hover:border-accent hover:bg-surface hover:text-fg"
      style={{ minHeight: newHeight }}
    >
      <Plus size={15} />
      New
    </button>
  );

  const sections = new Map<GroupOut | null, { header?: VirtualItem; rows: VirtualItem[] }>();
  for (const virtual of virtualItems) {
    const item = items[virtual.index];
    let section = sections.get(item.group);
    if (!section) sections.set(item.group, section = { rows: [] });
    if (item.kind === "header") section.header = virtual;
    else section.rows.push(virtual);
  }
  const renderRows = (rows: VirtualItem[], key: string, groupStart?: number) => {
    if (!rows.length) return null;
    const first = rows[0].start;
    const top = groupStart === undefined ? translateTop(first) : (first - groupStart) / scale;
    const height = (rows[rows.length - 1].end - first) / scale;
    return (
      <div key={key} data-grid={key} className="absolute inset-x-0 grid" style={{ ...gridStyle, top, height }}>
        {rows.map(({ index, start }) => {
          const item = items[index];
          if (item.kind !== "cards") return null;
          return (
            <div key={item.key} data-index={index} ref={virtualizer.measureElement} className="absolute inset-x-0 grid"
              style={{ ...gridStyle, top: (start - first) / scale, paddingBottom: item.gap }}>
              {item.ids.slice(item.first, item.first + columns).map((id) => {
                const row = rowById.get(id);
                return row ? <Card key={id} row={row} cover={cover} wantsCover={wantsCover} fit={view.cardFit} coverH={coverHeight} props={shown} titles={titles} onOpen={openRow} /> : (
                  <div key={id} style={{ minHeight: estimateCardHeight }} className="rounded-xl border border-line bg-surface" aria-hidden="true" />
                );
              })}
              {item.footer ? renderNew(key, item.group) : null}
            </div>
          );
        })}
      </div>
    );
  };
  const renderHeader = (virtual: VirtualItem) => {
    const item = items[virtual.index];
    if (item.kind !== "header") return null;
    const { group, closed } = item;
    return (
      <button type="button" aria-expanded={!closed}
        onClick={() => {
          const patch = setCollapsed(view, group.key, !closed);
          if (patch) updateView(patch);
        }}
        className="absolute start-0 top-0 flex h-8 items-center gap-1.5 rounded-lg px-1.5 hover:bg-surface"
      >
        {closed ? <ChevronRight size={14} className="rtl:rotate-180" /> : <ChevronDown size={14} />}
        <GroupChip g={group} />
        <span className="text-xs tabular-nums text-faint" data-count>{group.count}</span>
      </button>
    );
  };

  return (
    <div ref={host} className={cn("h-full overflow-y-auto px-4 [overflow-anchor:none]", compact && "max-h-[520px]")}
      role="region" aria-label="Gallery" onScroll={scale === 1 ? undefined : (event) => setScrollPosition(event.currentTarget)}>
      <div ref={content}>
        {empty ? (
          <div className="pb-8 pt-2">
            <EmptyState icon={<LayoutGrid size={28} />} title="No rows yet"><span>Add a row to start the gallery.</span></EmptyState>
            <div className="grid" data-grid="all" style={gridStyle}>{renderNew("all", null)}</div>
          </div>
        ) : (
          <div className={cn("relative", scale < 1 && "overflow-clip")} style={{ height: virtualizer.getTotalSize() + viewportPadding }}>
            {[...sections].map(([group, section]) => {
              if (!group) return renderRows(section.rows, "all");
              const first = section.header ?? section.rows[0];
              const last = section.rows[section.rows.length - 1] ?? first;
              return (
                <section key={group.key} data-group={group.key} className="absolute inset-x-0" style={{ top: translateTop(first.start), height: (last.end - first.start) / scale }}>
                  {section.header ? renderHeader(section.header) : null}
                  {renderRows(section.rows, `g:${group.key}`, first.start)}
                </section>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

const Card = memo(function Card({ row, cover, wantsCover, fit, coverH, props, titles, onOpen }: { row: DbRow; cover: string; wantsCover: boolean; fit: boolean; coverH: number; props: Property[]; titles: Map<string, string>; onOpen: (id: string) => void }) {
  const img = cover ? firstImage(row, cover) : null;
  const [broken, setBroken] = useState(false);
  return (
    <div
      role="button"
      tabIndex={0}
      data-card={row.id}
      aria-label={row.title || "Untitled"}
      onClick={() => onOpen(row.id)}
      onKeyDown={(e) => {
        if (e.key === "Enter" && e.target === e.currentTarget) {
          e.preventDefault();
          onOpen(row.id);
        }
      }}
      className="group/card flex min-w-0 cursor-pointer flex-col overflow-hidden rounded-xl border border-line bg-elevated outline-none transition-shadow hover:shadow-float focus-visible:ring-2 focus-visible:ring-accent"
    >
      {wantsCover ? (
        img && !broken ? (
          <img src={imageUrl(img.id)} alt="" draggable={false} loading="lazy" onError={() => setBroken(true)} className={cn("w-full border-b border-line", fit ? "bg-surface object-contain" : "object-cover")} style={{ height: coverH }} data-cover />
        ) : (
          <div className="grid w-full place-items-center border-b border-line bg-surface text-faint" style={{ height: coverH }} data-cover-empty>
            {row.icon ? <span className="text-3xl leading-none">{row.icon}</span> : <ImageIcon size={22} strokeWidth={1.5} />}
          </div>
        )
      ) : null}
      <div className="flex min-w-0 flex-col gap-1.5 px-3 py-2.5">
        <RowTitle row={row} className="text-[0.9rem] font-medium" />
        <PropLines row={row} props={props} titles={titles} />
      </div>
    </div>
  );
});
