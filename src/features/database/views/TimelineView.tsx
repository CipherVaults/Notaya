// Timeline view (Gantt): a bar per row from its start date to its end date, a list of titles on the left,
// zoom from days to years, a today line, dependency arrows and sub-item nesting. Drag a bar to move it, drag an edge to resize.

import { useCallback, useContext, useEffect, useId, useLayoutEffect, useMemo, useRef, useState, type KeyboardEvent as ReactKeyboard, type PointerEvent as ReactPointer } from "react";
import { defaultRangeExtractor, useVirtualizer, type Range } from "@tanstack/react-virtual";
import { CalendarRange, ChevronDown, ChevronRight, Plus } from "lucide-react";
import type { DbRow, GroupOut, Property } from "@/lib/database-types";
import { DbShellContext } from "../context";
import type { ViewProps } from "../viewTypes";
import { cn } from "@/lib/utils";
import {
  EmptyState,
  GroupChip,
  NewButton,
  PropSelect,
  RowTitle,
  colorProp,
  dateAt,
  dayNum,
  fmt,
  hueOf,
  isoDay,
  parseIso,
  propOf,
  readDate,
  rowColor,
  setCollapsed,
  shiftCell,
  shortDate,
  spanOf,
  todayNum,
  useViewRows,
  weekStart,
  withDays,
  ymdOf,
} from "./shared";

const LEFT_W = 244;
const ROW_H = 34;
const HEAD_H = 52;
const OVERSCAN = 8;
const ZOOMS = ["day", "week", "month", "quarter", "year"] as const;
type Zoom = (typeof ZOOMS)[number];
const PX: Record<Zoom, number> = { day: 44, week: 16, month: 5, quarter: 2, year: 0.7 };
const PAD: Record<Zoom, number> = { day: 7, week: 14, month: 45, quarter: 90, year: 200 };

interface Bounds {
  s: number;
  e: number;
}

type Line =
  | { kind: "group"; key: string; g: GroupOut; closed: boolean }
  | { kind: "row"; id: string; group: string | null; depth: number; kids: number; open: boolean };

type ParentLinks = ReadonlyMap<string, readonly string[]>;
const noParents: ParentLinks = new Map();

interface KnownTracks {
  scope: string;
  parents: ParentLinks;
}

function trackLines(ids: readonly string[], parents: ParentLinks, closed: ReadonlySet<string>, out: Line[], group: string | null = null) {
  if (!parents.size) {
    for (const id of ids) out.push({ kind: "row", id, group, depth: 0, kids: 0, open: true });
    return;
  }
  const membership = new Set(ids);
  const children = new Map<string, string[]>();
  const nested = new Set<string>();
  for (const id of ids) {
    const parent = parents.get(id)?.find((p) => p !== id && membership.has(p));
    if (!parent) continue;
    nested.add(id);
    const siblings = children.get(parent);
    if (siblings) siblings.push(id);
    else children.set(parent, [id]);
  }
  const seen = new Set<string>();
  const walk = (root: string) => {
    const stack = [{ id: root, depth: 0, visible: true }];
    while (stack.length) {
      const { id, depth, visible } = stack.pop()!;
      if (seen.has(id)) continue;
      seen.add(id);
      const kids = children.get(id) ?? [];
      const open = !closed.has(id);
      if (visible) out.push({ kind: "row", id, group, depth, kids: kids.length, open });
      for (let i = kids.length - 1; i >= 0; i--) {
        stack.push({ id: kids[i], depth: depth + 1, visible: visible && open });
      }
    }
  };
  for (const id of ids) {
    if (nested.has(id)) continue;
    if (children.has(id)) walk(id);
    else {
      seen.add(id);
      out.push({ kind: "row", id, group, depth: 0, kids: 0, open: true });
    }
  }
  // Parent cycles have no root. Include each track once even in that case.
  for (const id of ids) if (!seen.has(id)) walk(id);
}

type Mode = "move" | "start" | "end";
interface Drag {
  id: string;
  index: number;
  mode: Mode;
  x0: number;
  delta: number;
  moved: boolean;
}

export default function TimelineView(props: ViewProps) {
  const { db, view, updateView } = props;
  const dp = propOf(db, view.dateProp);
  const dateProps = db.props.filter((p) => p.kind === "date");
  if (!dp || dp.kind !== "date") {
    return (
      <EmptyState icon={<CalendarRange size={28} />} title="Choose a date property for the timeline">
        {dateProps.length ? (
          <>
            <span>Each row becomes a bar that starts on this property's date.</span>
            <PropSelect label="Start date property" placeholder="Date property…" props={dateProps} value="" onChange={(id) => updateView({ dateProp: id })} />
          </>
        ) : (
          <span>Add a date property to the database to use the timeline.</span>
        )}
      </EmptyState>
    );
  }
  return <TimelineInner {...props} dp={dp} dateProps={dateProps} />;
}

function TimelineInner({ db, view, result, rowById, tableRowIds, timelineMeta, loadTableRows, openRow, setCells, createRow, updateView, dp, dateProps }: ViewProps & { dp: Property; dateProps: Property[] }) {
  const compact = useContext(DbShellContext)?.compact;
  const zoom: Zoom = (ZOOMS as readonly string[]).includes(view.timelineZoom) ? (view.timelineZoom as Zoom) : "week";
  const px = PX[zoom];
  const today = todayNum();
  const ep = view.endProp && view.endProp !== dp.id ? propOf(db, view.endProp) : undefined;
  const endProp = ep && ep.kind === "date" ? ep : undefined;
  const cp = useMemo(() => colorProp(db), [db]);
  const subProp = view.showSubItems ? db.subParentProp : "";
  const blockedProp = view.showDeps ? db.blockedByProp : "";
  const scroller = useRef<HTMLDivElement>(null);
  const footerScroller = useRef<HTMLDivElement>(null);
  const marker = useId();
  const [closedRows, setClosedRows] = useState<Set<string>>(new Set());
  const [drag, setDrag] = useState<Drag | null>(null);
  const dragRef = useRef<Drag | null>(null);
  const [setting, setSetting] = useState<string | null>(null);
  const [showFoot, setShowFoot] = useState(true);
  const [known, setKnown] = useState<KnownTracks>();

  /* ----- dates of rows ----- */
  const bounds = useCallback((row: DbRow): Bounds | null => {
    const a = spanOf(row.values[dp.id]);
    if (!a) return null;
    let e = a.end;
    if (endProp) {
      const b = spanOf(row.values[endProp.id]);
      if (b) e = b.end;
    }
    return { s: a.start, e: Math.max(e, a.start) };
  }, [dp.id, endProp?.id]);

  const allRows = tableRowIds ? undefined : result.rows;
  const ids = useMemo(() => tableRowIds ?? allRows?.map((r) => r.id) ?? [], [tableRowIds, allRows]);
  const windowIds = useMemo(() => tableRowIds ? new Set(tableRowIds) : null, [tableRowIds]);
  const snapshotParents = tableRowIds && subProp ? timelineMeta?.subParents : undefined;
  const snapshotLinks = useMemo(() => snapshotParents ? new Map(Object.entries(snapshotParents)) : undefined, [snapshotParents]);
  const { dated, unscheduled, parents, extent } = useMemo(() => {
    const dated = new Map<string, Bounds>();
    const unscheduled: string[] = [];
    const parents = new Map<string, readonly string[]>();
    const extent = { s: today, e: today };
    for (const row of result.rows) {
      if (windowIds && !windowIds.has(row.id)) continue;
      const b = bounds(row);
      if (b) {
        dated.set(row.id, b);
        extent.s = Math.min(extent.s, b.s);
        extent.e = Math.max(extent.e, b.e);
      } else if (!timelineMeta) {
        unscheduled.push(row.id);
      }
      const links = subProp && !snapshotLinks ? row.values[subProp] : null;
      if (Array.isArray(links) && links.length) parents.set(row.id, links as string[]);
    }
    return { dated, unscheduled, parents, extent };
  }, [result.rows, windowIds, bounds, subProp, snapshotLinks, today, timelineMeta]);
  const unscheduledIds = timelineMeta?.unscheduledIds ?? unscheduled;

  const scope = JSON.stringify([db.id, view.id, dp.id, endProp?.id, subProp]);
  const knowledge = known?.scope === scope ? known : undefined;
  // Without snapshot links, keep observed parent links when the shell evicts row values.
  useEffect(() => {
    if (!windowIds || !subProp || snapshotLinks) return;
    setKnown((previous) => {
      const prior = previous?.scope === scope ? previous : undefined;
      const oldLinks = prior?.parents ?? noParents;
      let nextLinks: Map<string, readonly string[]> | undefined;
      for (const row of result.rows) {
        if (!windowIds.has(row.id)) continue;
        const before = oldLinks.get(row.id);
        const after = parents.get(row.id);
        if ((!before && !after) || (before && after && before.length === after.length && before.every((id, i) => id === after[i]))) continue;
        nextLinks ??= new Map(oldLinks);
        if (after) nextLinks.set(row.id, after);
        else nextLinks.delete(row.id);
      }
      if (!nextLinks) return previous;
      return { scope, parents: nextLinks };
    });
  }, [windowIds, result.rows, scope, subProp, parents, snapshotLinks]);

  /* ----- the lines on the left ----- */
  const parentLinks = tableRowIds ? snapshotLinks ?? knowledge?.parents ?? noParents : parents;
  const lines: Line[] = useMemo(() => {
    const out: Line[] = [];
    if (result.groups && view.group) {
      const collapsed = new Set(view.group.collapsed);
      for (const g of result.groups) {
        const closed = collapsed.has(g.key);
        out.push({ kind: "group", key: g.key, g, closed });
        if (!closed) trackLines(g.rows, parentLinks, closedRows, out, g.key);
      }
    } else {
      trackLines(ids, parentLinks, closedRows, out);
    }
    return out;
  }, [result.groups, ids, view.group, parentLinks, closedRows]);
  const noRows = !ids.length;
  const lineIndex = useMemo(() => {
    const byId = new Map<string, number>();
    lines.forEach((line, index) => {
      if (line.kind === "row") byId.set(line.id, index);
    });
    return byId;
  }, [lines]);
  const source = drag ? lines[drag.index] : undefined;
  const pinnedIndex = drag ? (source?.kind === "row" && source.id === drag.id ? drag.index : lineIndex.get(drag.id)) : undefined;
  const rangeExtractor = useCallback((range: Range) => {
    const indices = defaultRangeExtractor(range);
    if (pinnedIndex !== undefined && !indices.includes(pinnedIndex)) indices.push(pinnedIndex);
    return indices;
  }, [pinnedIndex]);
  const tracks = useVirtualizer({
    count: lines.length,
    getScrollElement: () => scroller.current,
    estimateSize: () => ROW_H,
    overscan: OVERSCAN,
    rangeExtractor,
  });
  const virtualTracks = tracks.getVirtualItems();
  const requestedIds = virtualTracks.flatMap(({ index }) => {
    const line = lines[index];
    return line?.kind === "row" ? [line.id] : [];
  });
  const settingIndex = setting ? unscheduledIds.indexOf(setting) : -1;
  const footerRange = useCallback((range: Range) => {
    const indices = defaultRangeExtractor(range);
    if (settingIndex >= 0 && !indices.includes(settingIndex)) indices.push(settingIndex);
    return indices;
  }, [settingIndex]);
  const footerKey = useCallback((index: number) => unscheduledIds[index], [unscheduledIds]);
  const footer = useVirtualizer({
    count: showFoot ? unscheduledIds.length : 0,
    getScrollElement: () => footerScroller.current,
    getItemKey: footerKey,
    estimateSize: () => ROW_H,
    overscan: 2,
    rangeExtractor: footerRange,
  });
  const footerItems = footer.getVirtualItems();
  for (const { index } of footerItems) requestedIds.push(unscheduledIds[index]);
  useViewRows(loadTableRows, requestedIds);

  const live = (id: string): Bounds | null => {
    const b = dated.get(id);
    if (!b) return null;
    if (!drag || drag.id !== id || !drag.delta) return b;
    if (drag.mode === "move") return { s: b.s + drag.delta, e: b.e + drag.delta };
    if (drag.mode === "start") return { s: Math.min(b.s + drag.delta, b.e), e: b.e };
    return { s: b.s, e: Math.max(b.e + drag.delta, b.s) };
  };

  /* ----- the time axis ----- */
  const preview = drag && live(drag.id);
  const lo = Math.min(timelineMeta?.minDay ?? today, extent.s, preview?.s ?? today);
  const hi = Math.max(timelineMeta?.maxDay ?? today, extent.e, preview?.e ?? today);
  const range = useMemo(() => {
    const start = weekStart(lo - PAD[zoom]);
    let end = hi + PAD[zoom];
    const minDays = Math.ceil(1100 / PX[zoom]);
    if (end - start < minDays) end = start + minDays;
    return { start, end };
  }, [lo, hi, zoom]);
  const total = Math.ceil((range.end - range.start + 1) * px);
  const x = (day: number) => (day - range.start) * px;
  const [scroll, setScroll] = useState({ left: 0, width: 0 });
  const syncHorizontal = useCallback(() => {
    const el = scroller.current;
    if (!el) return;
    const left = el.scrollLeft;
    const width = el.clientWidth;
    setScroll((prior) => prior.left === left && prior.width === width ? prior : { left, width });
  }, []);
  useLayoutEffect(() => {
    const el = scroller.current;
    if (!el) return;
    const observer = new ResizeObserver(syncHorizontal);
    observer.observe(el);
    return () => observer.disconnect();
  }, [noRows, syncHorizontal]);
  const visibleWidth = Math.max(scroll.width - LEFT_W, 1);
  const over = Math.max(visibleWidth, 720);
  const x0 = Math.max(0, scroll.left - over);
  const x1 = Math.min(total, scroll.left + visibleWidth + over);
  const fromDay = range.start + Math.floor(x0 / px);
  const toDay = Math.min(range.end, range.start + Math.floor(x1 / px));
  const ticks = useMemo(() => axis(zoom, range.start, range.end, fromDay, toDay), [zoom, range.start, range.end, fromDay, toDay]);
  const intersects = (left: number, right: number) => right >= x0 && left <= x1;

  /* ----- positions while dragging ----- */
  const commit = (id: string, mode: Mode, delta: number) => {
    const row = rowById.get(id);
    const b = row && bounds(row);
    if (!row || !b || !delta) return;
    const a = readDate(row.values[dp.id]);
    if (!a) return;
    const spanA = spanOf(a)!;
    const patch: Record<string, unknown> = {};
    const bc = endProp ? readDate(row.values[endProp.id]) : null;
    if (mode === "move") {
      patch[dp.id] = shiftCell(a, delta);
      if (endProp && bc) patch[endProp.id] = shiftCell(bc, delta);
    } else if (mode === "start") {
      const s2 = Math.min(b.s + delta, b.e);
      patch[dp.id] = endProp ? withDays(a, s2, Math.max(spanA.end, s2)) : withDays(a, s2, b.e);
    } else {
      const e2 = Math.max(b.e + delta, b.s);
      if (endProp) patch[endProp.id] = bc ? withDays(bc, e2, e2) : { start: isoDay(e2) };
      else patch[dp.id] = withDays(a, b.s, e2);
    }
    void setCells(id, patch);
  };

  const begin = (e: ReactPointer<HTMLElement>, id: string, mode: Mode) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    e.currentTarget.setPointerCapture(e.pointerId);
    const index = Number(e.currentTarget.closest<HTMLElement>("[data-track-index]")?.dataset.trackIndex);
    const d = { id, index, mode, x0: e.clientX, delta: 0, moved: false };
    dragRef.current = d;
    setDrag(d);
  };
  const track = (e: ReactPointer<HTMLElement>) => {
    const d = dragRef.current;
    if (!d) return;
    const dx = e.clientX - d.x0;
    const next = { ...d, delta: Math.round(dx / px), moved: d.moved || Math.abs(dx) > 3 };
    dragRef.current = next;
    setDrag(next);
  };
  const finish = (e: ReactPointer<HTMLElement>, cancel = false) => {
    const d = dragRef.current;
    dragRef.current = null;
    setDrag(null);
    if (!d) return;
    if (e.currentTarget.hasPointerCapture?.(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId);
    if (cancel) return;
    if (!d.moved) {
      if (d.mode === "move") openRow(d.id);
      return;
    }
    commit(d.id, d.mode, d.delta);
  };
  const keys = (e: ReactKeyboard<HTMLElement>, id: string) => {
    if (e.key === "Enter") {
      e.preventDefault();
      openRow(id);
    } else if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      e.preventDefault();
      commit(id, e.shiftKey ? "end" : "move", e.key === "ArrowLeft" ? -1 : 1);
    }
  };

  /* ----- scrolling to today ----- */
  const scrollToToday = useCallback((smooth = false) => {
    const el = scroller.current;
    if (!el) return;
    const target = Math.max(0, LEFT_W + (today - range.start) * px - el.clientWidth / 3);
    el.scrollTo({ left: target, behavior: smooth ? "smooth" : "auto" });
  }, [px, range.start, today]);
  const anchor = useRef<{ start: number; zoom: Zoom } | undefined>(undefined);
  useLayoutEffect(() => {
    const el = scroller.current;
    const previous = anchor.current;
    if (!el) {
      anchor.current = undefined;
      return;
    }
    if (!previous || previous.zoom !== zoom) scrollToToday();
    else if (previous.start !== range.start) el.scrollLeft += (previous.start - range.start) * px;
    anchor.current = { start: range.start, zoom };
    syncHorizontal();
  }, [noRows, px, range.start, scrollToToday, syncHorizontal, zoom]);

  /* ----- dependency arrows ----- */
  const bodyH = tracks.getTotalSize();
  const minY = Math.max(0, (tracks.scrollOffset ?? 0) - OVERSCAN * ROW_H);
  const maxY = Math.min(bodyH, (tracks.scrollOffset ?? 0) + (tracks.scrollRect?.height ?? 0) + OVERSCAN * ROW_H);
  const arrows: { id: string; d: string }[] = [];
  if (blockedProp) {
    const visible = new Map<string, number>();
    for (const { index, start } of virtualTracks) {
      const l = lines[index];
      if (l?.kind === "row" && start + ROW_H >= minY && start < maxY) visible.set(l.id, start + ROW_H / 2);
    }
    for (const [id, y2] of visible) {
      const dependencies = rowById.get(id)?.values[blockedProp];
      const to = live(id);
      if (!Array.isArray(dependencies) || !to) continue;
      for (const by of dependencies as string[]) {
        const y1 = visible.get(by);
        if (y1 === undefined) continue;
        const from = live(by);
        if (!from) continue;
        const fromX = x(from.e + 1);
        const toX = x(to.s);
        if (!intersects(Math.min(fromX, toX) - 10, Math.max(fromX, toX) + 8)) continue;
        const dir = y2 >= y1 ? 1 : -1;
        const path =
          toX >= fromX + 16
            ? `M${fromX},${y1} H${fromX + 8} V${y2} H${toX - 1}`
            : `M${fromX},${y1} H${fromX + 8} V${y1 + (dir * ROW_H) / 2} H${toX - 10} V${y2} H${toX - 1}`;
        arrows.push({ id: `${by}>${id}`, d: path });
      }
    }
  }

  /* ----- creating and setting dates ----- */
  const newRow = () => void createRow({ [dp.id]: { start: isoDay(today) } });
  const saveDates = (row: DbRow, start: string, end: string) => {
    const s = parseIso(start);
    const en = parseIso(end || start);
    if (!s) return;
    const e2 = en && en.day >= s.day ? en.day : s.day;
    const patch: Record<string, unknown> = {};
    if (endProp) {
      patch[dp.id] = { start: isoDay(s.day) };
      patch[endProp.id] = { start: isoDay(e2) };
    } else patch[dp.id] = e2 > s.day ? { start: isoDay(s.day), end: isoDay(e2) } : { start: isoDay(s.day) };
    setSetting(null);
    void setCells(row.id, patch);
  };

  const setGroupClosed = (key: string, closed: boolean) => {
    const p = setCollapsed(view, key, closed);
    if (p) updateView(p);
  };

  return (
    <div className={cn("flex h-full min-h-0 flex-col", compact && "max-h-[520px]")} role="region" aria-label="Timeline" dir="ltr">
      <div className="flex shrink-0 flex-wrap items-center gap-2 px-4 pb-2 pt-1">
        <div className="flex rounded-lg border border-line p-0.5" role="group" aria-label="Zoom">
          {ZOOMS.map((z) => (
            <button key={z} type="button" aria-pressed={zoom === z} onClick={() => updateView({ timelineZoom: z })} className={cn("h-7 rounded-md px-2.5 text-[0.8rem] capitalize", zoom === z ? "bg-accent-soft text-accent" : "text-muted hover:text-fg")}>
              {z}
            </button>
          ))}
        </div>
        <button type="button" onClick={() => scrollToToday(true)} className="h-8 rounded-lg border border-line px-3 text-[0.8rem] hover:bg-surface-2">Today</button>
        <span className="ms-auto flex items-center gap-2">
          {dateProps.length > 1 ? (
            <>
              <PropSelect label="Start date property" placeholder="Start" props={dateProps} value={dp.id} onChange={(id) => updateView({ dateProp: id, endProp: view.endProp === id ? "" : view.endProp })} />
              <PropSelect label="End date property" placeholder="End" allowNone="End: from the date range" props={dateProps.filter((p) => p.id !== dp.id)} value={endProp?.id ?? ""} onChange={(id) => updateView({ endProp: id })} />
            </>
          ) : null}
          <NewButton onClick={newRow} label="New" />
        </span>
      </div>

      {noRows ? (
        <EmptyState icon={<CalendarRange size={28} />} title="No rows yet">
          <span>Add a row to see it on the timeline.</span>
          <button type="button" onClick={newRow} className="inline-flex h-8 items-center gap-1.5 rounded-lg bg-accent px-3 text-[0.8rem] text-accent-fg"><Plus size={14} />New</button>
        </EmptyState>
      ) : (
        <div ref={scroller} onScroll={syncHorizontal} className="relative min-h-0 flex-1 overflow-auto border-y border-line [overflow-anchor:none]" data-scroller>
          <div className="relative" style={{ width: LEFT_W + total }}>
            {/* header */}
            <div className="sticky top-0 z-30 flex bg-bg" style={{ height: HEAD_H }}>
              <div className="sticky left-0 z-40 flex shrink-0 items-end border-b border-e border-line bg-bg px-3 pb-1.5 text-[0.72rem] font-medium uppercase tracking-wide text-muted" style={{ width: LEFT_W }}>
                Name
              </div>
              <div className="relative border-b border-line" style={{ width: total }}>
                {ticks.top.map((t) => (
                  <div key={t.key} data-timeline-tick={t.key} className="absolute top-0 flex h-[26px] items-center overflow-hidden border-s border-line px-2 text-[0.72rem] font-medium text-muted" style={{ left: x(t.from), width: (t.to - t.from + 1) * px }}>
                    <span className="sticky left-[252px] whitespace-nowrap">{t.label}</span>
                  </div>
                ))}
                {ticks.bottom.map((t) => (
                  <div key={t.key} data-timeline-tick={t.key} className={cn("absolute flex h-[26px] items-center overflow-hidden border-s border-line px-1 text-[0.7rem] text-faint", t.weekend && "bg-surface")} style={{ top: 26, left: x(t.from), width: (t.to - t.from + 1) * px }}>
                    <span className="truncate">{t.label}</span>
                  </div>
                ))}
                <div className="absolute z-10 -translate-x-1/2 rounded-full bg-accent px-1.5 text-[0.62rem] font-medium leading-4 text-accent-fg" style={{ left: x(today) + px / 2, top: 5 }}>
                  Today
                </div>
              </div>
            </div>

            {/* body */}
            <div className="relative" style={{ height: bodyH }}>
              <div className="pointer-events-none absolute top-0" style={{ left: LEFT_W, width: total, height: bodyH }} aria-hidden>
                {ticks.bottom.map((t) => (
                  <div key={t.key} data-timeline-gridline={t.key} className={cn("absolute inset-y-0 border-s border-line/60", t.weekend && "bg-surface/70")} style={{ left: x(t.from), width: (t.to - t.from + 1) * px }} />
                ))}
                <div className="absolute inset-y-0 z-[5] w-px bg-accent" style={{ left: x(today) + px / 2 }} data-today-line />
              </div>

              {virtualTracks.map(({ index, start }) => {
                const l = lines[index];
                if (!l) return null;
                const row = l.kind === "row" ? rowById.get(l.id) : undefined;
                const b = row ? live(row.id) : null;
                const dragging = !!row && drag?.id === row.id;
                const showBar = b && (dragging || intersects(x(b.s), Math.max(x(b.e + 1), x(b.s) + 8)));
                return (
                  <div key={l.kind === "group" ? `g:${l.key}` : `r:${l.group}:${l.id}`} className="absolute inset-x-0 flex" style={{ top: start, height: ROW_H }} data-track-index={index} data-line={l.kind === "row" ? l.id : undefined}>
                    <div className="sticky left-0 z-20 flex shrink-0 items-center border-b border-e border-line bg-bg" style={{ width: LEFT_W }}>
                      {l.kind === "group" ? (
                        <button type="button" aria-expanded={!l.closed} onClick={() => setGroupClosed(l.key, !l.closed)} className="flex h-full w-full items-center gap-1.5 px-2 text-start hover:bg-surface">
                          {l.closed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                          <GroupChip g={l.g} />
                          <span data-count className="text-xs tabular-nums text-faint">{l.g.count}</span>
                        </button>
                      ) : row ? (
                        <div className="flex h-full min-w-0 flex-1 items-center" style={{ paddingInlineStart: 6 + Math.min(l.depth, 11) * 16 }}>
                          {l.kids ? (
                            <button
                              type="button"
                              aria-label={l.open ? "Collapse sub-items" : "Expand sub-items"}
                              aria-expanded={l.open}
                              onClick={() =>
                                setClosedRows((s) => {
                                  const n = new Set(s);
                                  if (n.has(l.id)) n.delete(l.id);
                                  else n.add(l.id);
                                  return n;
                                })
                              }
                              className="grid h-5 w-5 shrink-0 place-items-center rounded text-faint hover:bg-surface-2 hover:text-fg"
                            >
                              {l.open ? <ChevronDown size={13} /> : <ChevronRight size={13} />}
                            </button>
                          ) : (
                            <span className="w-5 shrink-0" />
                          )}
                          <button type="button" onClick={() => openRow(row.id)} className="min-w-0 flex-1 rounded px-1 py-1 text-start text-[0.82rem] hover:bg-surface-2" data-title-of={row.id}>
                            <RowTitle row={row} />
                          </button>
                        </div>
                      ) : <span aria-hidden className="ms-8 h-2 w-24 rounded bg-surface-2" />}
                    </div>
                    <div className={cn("relative border-b border-line/70", l.kind === "group" && "bg-surface/60")} style={{ width: total }}>
                      {row && showBar ? (
                        <BarFor
                          row={row}
                          b={b}
                          x={x}
                          px={px}
                          color={rowColor(row, cp)}
                          dragging={dragging}
                          begin={begin}
                          track={track}
                          finish={finish}
                          keys={keys}
                        />
                      ) : null}
                    </div>
                  </div>
                );
              })}

              {arrows.length ? (
                <svg className="pointer-events-none absolute z-10 text-muted" style={{ left: LEFT_W + x0, top: minY }} width={x1 - x0} height={maxY - minY} viewBox={`${x0} ${minY} ${x1 - x0} ${maxY - minY}`} aria-hidden data-deps={arrows.length}>
                  <defs>
                    <marker id={marker} markerWidth="7" markerHeight="7" refX="6" refY="3.5" orient="auto">
                      <path d="M0,0.5 L6.5,3.5 L0,6.5 z" fill="currentColor" />
                    </marker>
                  </defs>
                  {arrows.map((a) => (
                    <path key={a.id} d={a.d} fill="none" stroke="currentColor" strokeWidth="1.3" markerEnd={`url(#${marker})`} data-dep={a.id} />
                  ))}
                </svg>
              ) : null}
            </div>
          </div>
        </div>
      )}

      {unscheduledIds.length ? (
        <div className="shrink-0 px-4 py-2" data-unscheduled-box>
          <button type="button" aria-expanded={showFoot} onClick={() => setShowFoot((v) => !v)} className="flex items-center gap-1.5 text-[0.8rem] font-medium text-muted hover:text-fg">
            {showFoot ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
            No dates
            <span className="text-xs font-normal tabular-nums text-faint">{unscheduledIds.length}</span>
          </button>
          {showFoot ? (
            <div ref={footerScroller} className="mt-1.5 max-h-36 overflow-y-auto">
              <div className="relative" style={{ height: footer.getTotalSize() }}>
                {footerItems.map(({ key, index, start }) => {
                  const id = unscheduledIds[index];
                  const r = rowById.get(id);
                  return (
                    <div key={key} className="absolute inset-x-0 flex items-center gap-2 rounded-lg px-1 py-0.5 hover:bg-surface" style={{ top: start, height: ROW_H }} data-unscheduled={id}>
                      {r ? (
                        <>
                          <button type="button" onClick={() => openRow(r.id)} className="min-w-0 flex-1 rounded px-1 py-1 text-start text-[0.82rem]">
                            <RowTitle row={r} />
                          </button>
                          {setting === r.id ? (
                            <DatesEditor today={isoDay(today)} onSave={(s, e) => saveDates(r, s, e)} onCancel={() => setSetting(null)} />
                          ) : (
                            <button type="button" onClick={() => setSetting(r.id)} className="h-7 shrink-0 rounded-lg border border-line px-2.5 text-[0.78rem] text-muted hover:bg-surface-2 hover:text-fg">
                              Set dates
                            </button>
                          )}
                        </>
                      ) : (
                        <span aria-hidden className="ms-2 h-2 w-24 rounded bg-surface-2" />
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}

/* ---------- A bar ---------- */

function BarFor({
  row,
  b,
  x,
  px,
  color,
  dragging,
  begin,
  track,
  finish,
  keys,
}: {
  row: DbRow;
  b: Bounds | null;
  x: (d: number) => number;
  px: number;
  color: string;
  dragging: boolean;
  begin: (e: ReactPointer<HTMLElement>, id: string, mode: Mode) => void;
  track: (e: ReactPointer<HTMLElement>) => void;
  finish: (e: ReactPointer<HTMLElement>, cancel?: boolean) => void;
  keys: (e: ReactKeyboard<HTMLElement>, id: string) => void;
}) {
  if (!b) return null;
  const left = x(b.s);
  const width = Math.max((b.e - b.s + 1) * px, 8);
  const hue = hueOf(color);
  const fill = hue ? `color-mix(in oklab, ${hue} 70%, var(--bg))` : "color-mix(in oklab, var(--accent) 75%, var(--bg))";
  const label = `${row.title || "Untitled"}: ${shortDate(b.s, true)} to ${shortDate(b.e, true)}`;
  const inside = width >= 90;
  const move = (e: ReactPointer<HTMLElement>) => begin(e, row.id, "move");
  return (
    <>
      <div
        role="button"
        tabIndex={0}
        data-bar={row.id}
        data-start={isoDay(b.s)}
        data-end={isoDay(b.e)}
        aria-label={label}
        title={label}
        onPointerDown={move}
        onPointerMove={track}
        onPointerUp={(e) => finish(e)}
        onPointerCancel={(e) => finish(e, true)}
        onKeyDown={(e) => keys(e, row.id)}
        className={cn("group/bar absolute top-1/2 z-[6] flex h-[22px] -translate-y-1/2 cursor-grab touch-none items-center rounded-md text-[0.74rem] text-white shadow-sm outline-none focus-visible:ring-2 focus-visible:ring-accent", dragging && "cursor-grabbing opacity-85 shadow-float")}
        style={{ left, width, background: fill }}
      >
        <span
          role="presentation"
          data-handle="start"
          className="absolute inset-y-0 start-0 z-10 w-2 cursor-ew-resize rounded-s-md hover:bg-white/30"
          onPointerDown={(e) => begin(e, row.id, "start")}
          onPointerMove={track}
          onPointerUp={(e) => finish(e)}
          onPointerCancel={(e) => finish(e, true)}
        />
        {inside ? (
          <span dir="auto" className="pointer-events-none block min-w-0 flex-1 truncate px-2.5 drop-shadow-[0_1px_0_rgba(0,0,0,0.25)]">
            {row.icon ? `${row.icon} ` : ""}
            {row.title || "Untitled"}
          </span>
        ) : null}
        <span
          role="presentation"
          data-handle="end"
          className="absolute inset-y-0 end-0 z-10 w-2 cursor-ew-resize rounded-e-md hover:bg-white/30"
          onPointerDown={(e) => begin(e, row.id, "end")}
          onPointerMove={track}
          onPointerUp={(e) => finish(e)}
          onPointerCancel={(e) => finish(e, true)}
        />
      </div>
      {!inside ? (
        <span dir="auto" className="pointer-events-none absolute top-1/2 z-[6] max-w-[16rem] -translate-y-1/2 truncate text-[0.76rem] text-muted" style={{ left: left + width + 6 }}>
          {row.icon ? `${row.icon} ` : ""}
          {row.title || "Untitled"}
        </span>
      ) : null}
    </>
  );
}

function DatesEditor({ today, onSave, onCancel }: { today: string; onSave: (start: string, end: string) => void; onCancel: () => void }) {
  const [s, setS] = useState(today);
  const [e, setE] = useState(today);
  return (
    <span className="flex shrink-0 items-center gap-1.5 text-[0.78rem]">
      <input type="date" aria-label="Start date" value={s} onChange={(ev) => setS(ev.target.value)} className="h-7 rounded-md border border-line bg-surface px-1.5 outline-none focus:border-accent" />
      <span className="text-faint">→</span>
      <input type="date" aria-label="End date" value={e} min={s} onChange={(ev) => setE(ev.target.value)} className="h-7 rounded-md border border-line bg-surface px-1.5 outline-none focus:border-accent" />
      <button type="button" onClick={() => onSave(s, e)} className="h-7 rounded-md bg-accent px-2.5 text-accent-fg">Save</button>
      <button type="button" onClick={onCancel} className="h-7 rounded-md px-2 text-muted hover:bg-surface-2">Cancel</button>
    </span>
  );
}

/* ---------- The axis ---------- */

interface Tick {
  key: string;
  label: string;
  from: number;
  to: number;
  weekend?: boolean;
}

function axis(zoom: Zoom, start: number, end: number, first: number, last: number): { top: Tick[]; bottom: Tick[] } {
  const top: Tick[] = [];
  const bottom: Tick[] = [];
  const firstYear = ymdOf(first).y;
  const lastYear = ymdOf(last).y;
  const monthLabel = (d: number) => fmt({ month: "long", year: "numeric" }).format(dateAt(d));
  const eachMonth = (cb: (from: number, to: number, y: number, m: number) => void) => {
    const s = ymdOf(first);
    for (let y = s.y, m = s.m; ; m++) {
      if (m > 12) {
        m = 1;
        y++;
      }
      const from = dayNum(y, m, 1);
      if (from > last) break;
      cb(Math.max(from, start), Math.min(dayNum(y, m + 1, 0), end), y, m);
    }
  };
  if (zoom === "day" || zoom === "week") {
    eachMonth((from, to) => top.push({ key: `m${from}`, label: monthLabel(from), from, to }));
    if (zoom === "day") {
      for (let d = first; d <= last; d++) {
        const wd = (((d + 3) % 7) + 7) % 7;
        bottom.push({ key: `d${d}`, label: `${ymdOf(d).d} ${fmt({ weekday: "narrow" }).format(dateAt(d))}`, from: d, to: d, weekend: wd >= 5 });
      }
    } else {
      for (let w = weekStart(first); w <= last; w += 7) bottom.push({ key: `w${w}`, label: shortDate(w), from: w, to: Math.min(w + 6, end) });
    }
  } else if (zoom === "month") {
    for (let y = firstYear; y <= lastYear; y++) top.push({ key: `y${y}`, label: String(y), from: Math.max(dayNum(y, 1, 1), start), to: Math.min(dayNum(y, 12, 31), end) });
    eachMonth((from, to) => bottom.push({ key: `m${from}`, label: fmt({ month: "short" }).format(dateAt(from)), from, to }));
  } else if (zoom === "quarter") {
    for (let y = firstYear; y <= lastYear; y++) {
      top.push({ key: `y${y}`, label: String(y), from: Math.max(dayNum(y, 1, 1), start), to: Math.min(dayNum(y, 12, 31), end) });
      for (let q = 0; q < 4; q++) {
        const from = dayNum(y, q * 3 + 1, 1);
        const to = dayNum(y, q * 3 + 4, 0);
        if (to >= first && from <= last) bottom.push({ key: `q${y}${q}`, label: `Q${q + 1}`, from: Math.max(from, start), to: Math.min(to, end) });
      }
    }
  } else {
    for (let y = firstYear; y <= lastYear; y++) bottom.push({ key: `y${y}`, label: String(y), from: Math.max(dayNum(y, 1, 1), start), to: Math.min(dayNum(y, 12, 31), end) });
  }
  return { top, bottom };
}
