// Timeline view (Gantt): a bar per row from its start date to its end date, a list of titles on the left,
// zoom from days to years, a today line, dependency arrows and sub-item nesting. Drag a bar to move it, drag an edge to resize.

import { useEffect, useMemo, useRef, useState, type KeyboardEvent as ReactKeyboard, type PointerEvent as ReactPointer } from "react";
import { CalendarRange, ChevronDown, ChevronRight, Plus } from "lucide-react";
import type { DbRow, GroupOut, Property, View } from "@/lib/database-types";
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
  weekStart,
  withDays,
  ymdOf,
} from "./shared";

const LEFT_W = 244;
const ROW_H = 34;
const HEAD_H = 52;
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
  | { kind: "row"; row: DbRow; depth: number; kids: number; open: boolean };

type Mode = "move" | "start" | "end";
interface Drag {
  id: string;
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

function TimelineInner({ db, view, result, rowById, openRow, setCells, createRow, updateView, dp, dateProps }: ViewProps & { dp: Property; dateProps: Property[] }) {
  const zoom: Zoom = (ZOOMS as readonly string[]).includes(view.timelineZoom) ? (view.timelineZoom as Zoom) : "week";
  const px = PX[zoom];
  const today = todayNum();
  const ep = view.endProp && view.endProp !== dp.id ? propOf(db, view.endProp) : undefined;
  const endProp = ep && ep.kind === "date" ? ep : undefined;
  const cp = useMemo(() => colorProp(db), [db]);
  const subProp = view.showSubItems ? db.subParentProp : "";
  const blockedProp = view.showDeps ? db.blockedByProp : "";
  const scroller = useRef<HTMLDivElement>(null);
  const [closedRows, setClosedRows] = useState<Set<string>>(new Set());
  const [drag, setDrag] = useState<Drag | null>(null);
  const dragRef = useRef<Drag | null>(null);
  const [setting, setSetting] = useState<string | null>(null);
  const [showFoot, setShowFoot] = useState(true);

  /* ----- dates of rows ----- */
  const bounds = (row: DbRow): Bounds | null => {
    const a = spanOf(row.values[dp.id]);
    if (!a) return null;
    let e = a.end;
    if (endProp) {
      const b = spanOf(row.values[endProp.id]);
      if (b) e = b.end;
    }
    return { s: a.start, e: Math.max(e, a.start) };
  };

  const { dated, unscheduled } = useMemo(() => {
    const dated = new Map<string, Bounds>();
    const unscheduled: DbRow[] = [];
    for (const r of result.rows) {
      const a = spanOf(r.values[dp.id]);
      if (!a) {
        unscheduled.push(r);
        continue;
      }
      let e = a.end;
      if (endProp) {
        const b = spanOf(r.values[endProp.id]);
        if (b) e = b.end;
      }
      dated.set(r.id, { s: a.start, e: Math.max(e, a.start) });
    }
    return { dated, unscheduled };
  }, [result.rows, dp.id, endProp]);

  /* ----- the lines on the left ----- */
  const lines: Line[] = useMemo(() => {
    const build = (ids: string[]): Line[] => {
      const set = new Set(ids);
      const kids = new Map<string, string[]>();
      const roots: string[] = [];
      for (const id of ids) {
        const row = rowById.get(id);
        const parents = subProp && row ? (Array.isArray(row.values[subProp]) ? (row.values[subProp] as string[]) : []) : [];
        const parent = parents.find((p) => set.has(p) && p !== id);
        if (parent) kids.set(parent, [...(kids.get(parent) ?? []), id]);
        else roots.push(id);
      }
      const out: Line[] = [];
      const seen = new Set<string>();
      const walk = (id: string, depth: number) => {
        if (seen.has(id)) return;
        seen.add(id);
        const row = rowById.get(id);
        if (!row) return;
        const k = kids.get(id) ?? [];
        const open = !closedRows.has(id);
        out.push({ kind: "row", row, depth, kids: k.length, open });
        if (open) k.forEach((c) => walk(c, depth + 1));
        else hide(k);
      };
      // The sub-items of a collapsed row stay hidden (they count as seen).
      const hide = (list: string[]) =>
        list.forEach((c) => {
          if (seen.has(c)) return;
          seen.add(c);
          hide(kids.get(c) ?? []);
        });
      roots.forEach((r) => walk(r, 0));
      // Rows in a parent loop are never reached from a root: show them at the top level.
      ids.forEach((i) => walk(i, 0));
      return out;
    };
    if (result.groups && view.group) {
      const out: Line[] = [];
      for (const g of result.groups) {
        const closed = view.group.collapsed.includes(g.key);
        out.push({ kind: "group", key: g.key, g, closed });
        if (!closed) out.push(...build(g.rows));
      }
      return out;
    }
    return build(result.rows.map((r) => r.id));
  }, [result.groups, result.rows, view.group, rowById, subProp, closedRows]);

  /* ----- the time axis ----- */
  const range = useMemo(() => {
    let lo = today;
    let hi = today;
    dated.forEach((b) => {
      lo = Math.min(lo, b.s);
      hi = Math.max(hi, b.e);
    });
    const start = weekStart(lo - PAD[zoom]);
    let end = hi + PAD[zoom];
    const minDays = Math.ceil(1100 / PX[zoom]);
    if (end - start < minDays) end = start + minDays;
    return { start, end };
  }, [dated, today, zoom]);
  const total = Math.ceil((range.end - range.start + 1) * px);
  const x = (day: number) => (day - range.start) * px;
  const ticks = useMemo(() => axis(zoom, range.start, range.end), [zoom, range.start, range.end]);

  /* ----- positions while dragging ----- */
  const live = (id: string): Bounds | null => {
    const b = dated.get(id);
    if (!b) return null;
    if (!drag || drag.id !== id || !drag.delta) return b;
    if (drag.mode === "move") return { s: b.s + drag.delta, e: b.e + drag.delta };
    if (drag.mode === "start") return { s: Math.min(b.s + drag.delta, b.e), e: b.e };
    return { s: b.s, e: Math.max(b.e + drag.delta, b.s) };
  };

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
    const d = { id, mode, x0: e.clientX, delta: 0, moved: false };
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
  const scrollToToday = (smooth = false) => {
    const el = scroller.current;
    if (!el) return;
    const target = Math.max(0, LEFT_W + x(today) - el.clientWidth / 3);
    el.scrollTo({ left: target, behavior: smooth ? "smooth" : "auto" });
  };
  useEffect(() => {
    scrollToToday();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [zoom, range.start]);

  /* ----- dependency arrows ----- */
  const lineIndex = useMemo(() => {
    const m = new Map<string, number>();
    lines.forEach((l, i) => l.kind === "row" && m.set(l.row.id, i));
    return m;
  }, [lines]);
  const arrows = useMemo(() => {
    if (!blockedProp) return [];
    const out: { id: string; d: string }[] = [];
    for (const l of lines) {
      if (l.kind !== "row") continue;
      const v = l.row.values[blockedProp];
      if (!Array.isArray(v)) continue;
      const to = live(l.row.id);
      const yi = lineIndex.get(l.row.id);
      if (!to || yi === undefined) continue;
      for (const by of v as string[]) {
        const from = live(by);
        const yj = lineIndex.get(by);
        if (!from || yj === undefined) continue;
        const x1 = x(from.e + 1);
        const x2 = x(to.s);
        const y1 = yj * ROW_H + ROW_H / 2;
        const y2 = yi * ROW_H + ROW_H / 2;
        const dir = y2 >= y1 ? 1 : -1;
        const path =
          x2 >= x1 + 16
            ? `M${x1},${y1} H${x1 + 8} V${y2} H${x2 - 1}`
            : `M${x1},${y1} H${x1 + 8} V${y1 + (dir * ROW_H) / 2} H${x2 - 10} V${y2} H${x2 - 1}`;
        out.push({ id: `${by}>${l.row.id}`, d: path });
      }
    }
    return out;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [lines, blockedProp, drag, range.start, px, dated, lineIndex]);

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
    const p = setCollapsed(view as View, key, closed);
    if (p) updateView(p);
  };

  const bodyH = lines.length * ROW_H;
  const noRows = !result.rows.length;

  return (
    <div className="flex h-full min-h-0 flex-col" role="region" aria-label="Timeline" dir="ltr">
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
        <div ref={scroller} className="relative min-h-0 flex-1 overflow-auto border-y border-line" data-scroller>
          <div className="relative" style={{ width: LEFT_W + total }}>
            {/* header */}
            <div className="sticky top-0 z-30 flex bg-bg" style={{ height: HEAD_H }}>
              <div className="sticky left-0 z-40 flex shrink-0 items-end border-b border-e border-line bg-bg px-3 pb-1.5 text-[0.72rem] font-medium uppercase tracking-wide text-muted" style={{ width: LEFT_W }}>
                Name
              </div>
              <div className="relative border-b border-line" style={{ width: total }}>
                {ticks.top.map((t) => (
                  <div key={t.key} className="absolute top-0 flex h-[26px] items-center overflow-hidden border-s border-line px-2 text-[0.72rem] font-medium text-muted" style={{ left: x(t.from), width: (t.to - t.from + 1) * px }}>
                    <span className="sticky left-[252px] whitespace-nowrap">{t.label}</span>
                  </div>
                ))}
                {ticks.bottom.map((t) => (
                  <div key={t.key} className={cn("absolute flex h-[26px] items-center overflow-hidden border-s border-line px-1 text-[0.7rem] text-faint", t.weekend && "bg-surface")} style={{ top: 26, left: x(t.from), width: (t.to - t.from + 1) * px }}>
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
                  <div key={t.key} className={cn("absolute inset-y-0 border-s border-line/60", t.weekend && "bg-surface/70")} style={{ left: x(t.from), width: (t.to - t.from + 1) * px }} />
                ))}
                <div className="absolute inset-y-0 z-[5] w-px bg-accent" style={{ left: x(today) + px / 2 }} data-today-line />
              </div>

              {lines.map((l) => (
                <div key={l.kind === "group" ? "g:" + l.key : l.row.id} className="relative flex" style={{ height: ROW_H }} data-line={l.kind === "row" ? l.row.id : undefined}>
                  <div className="sticky left-0 z-20 flex shrink-0 items-center border-b border-e border-line bg-bg" style={{ width: LEFT_W }}>
                    {l.kind === "group" ? (
                      <button type="button" aria-expanded={!l.closed} onClick={() => setGroupClosed(l.key, !l.closed)} className="flex h-full w-full items-center gap-1.5 px-2 text-start hover:bg-surface">
                        {l.closed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                        <GroupChip g={l.g} />
                        <span className="text-xs tabular-nums text-faint">{l.g.count}</span>
                      </button>
                    ) : (
                      <div className="flex h-full min-w-0 flex-1 items-center" style={{ paddingInlineStart: 6 + l.depth * 16 }}>
                        {l.kids ? (
                          <button
                            type="button"
                            aria-label={l.open ? "Collapse sub-items" : "Expand sub-items"}
                            aria-expanded={l.open}
                            onClick={() =>
                              setClosedRows((s) => {
                                const n = new Set(s);
                                if (n.has(l.row.id)) n.delete(l.row.id);
                                else n.add(l.row.id);
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
                        <button type="button" onClick={() => openRow(l.row.id)} className="min-w-0 flex-1 rounded px-1 py-1 text-start text-[0.82rem] hover:bg-surface-2" data-title-of={l.row.id}>
                          <RowTitle row={l.row} />
                        </button>
                      </div>
                    )}
                  </div>
                  <div className={cn("relative border-b border-line/70", l.kind === "group" && "bg-surface/60")} style={{ width: total }}>
                    {l.kind === "row" ? (
                      <BarFor
                        row={l.row}
                        b={live(l.row.id)}
                        x={x}
                        px={px}
                        color={rowColor(l.row, cp)}
                        dragging={drag?.id === l.row.id}
                        begin={begin}
                        track={track}
                        finish={finish}
                        keys={keys}
                      />
                    ) : null}
                  </div>
                </div>
              ))}

              {arrows.length ? (
                <svg className="pointer-events-none absolute top-0 z-10 text-muted" style={{ left: LEFT_W }} width={total} height={bodyH} aria-hidden data-deps={arrows.length}>
                  <defs>
                    <marker id="dep-arrow" markerWidth="7" markerHeight="7" refX="6" refY="3.5" orient="auto">
                      <path d="M0,0.5 L6.5,3.5 L0,6.5 z" fill="currentColor" />
                    </marker>
                  </defs>
                  {arrows.map((a) => (
                    <path key={a.id} d={a.d} fill="none" stroke="currentColor" strokeWidth="1.3" markerEnd="url(#dep-arrow)" data-dep={a.id} />
                  ))}
                </svg>
              ) : null}
            </div>
          </div>
        </div>
      )}

      {unscheduled.length ? (
        <div className="shrink-0 px-4 py-2" data-unscheduled-box>
          <button type="button" aria-expanded={showFoot} onClick={() => setShowFoot((v) => !v)} className="flex items-center gap-1.5 text-[0.8rem] font-medium text-muted hover:text-fg">
            {showFoot ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
            No dates
            <span className="text-xs font-normal tabular-nums text-faint">{unscheduled.length}</span>
          </button>
          {showFoot ? (
            <div className="mt-1.5 flex max-h-36 flex-col gap-0.5 overflow-y-auto">
              {unscheduled.map((r) => (
                <div key={r.id} className="flex items-center gap-2 rounded-lg px-1 py-0.5 hover:bg-surface" data-unscheduled={r.id}>
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
                </div>
              ))}
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

function axis(zoom: Zoom, start: number, end: number): { top: Tick[]; bottom: Tick[] } {
  const top: Tick[] = [];
  const bottom: Tick[] = [];
  const monthLabel = (d: number) => fmt({ month: "long", year: "numeric" }).format(dateAt(d));
  const eachMonth = (cb: (from: number, to: number, y: number, m: number) => void) => {
    const s = ymdOf(start);
    for (let y = s.y, m = s.m; ; m++) {
      if (m > 12) {
        m = 1;
        y++;
      }
      const from = dayNum(y, m, 1);
      if (from > end) break;
      cb(Math.max(from, start), Math.min(dayNum(y, m + 1, 0), end), y, m);
    }
  };
  if (zoom === "day" || zoom === "week") {
    eachMonth((from, to) => top.push({ key: `m${from}`, label: monthLabel(from), from, to }));
    if (zoom === "day") {
      for (let d = start; d <= end; d++) {
        const wd = (((d + 3) % 7) + 7) % 7;
        bottom.push({ key: `d${d}`, label: `${ymdOf(d).d} ${fmt({ weekday: "narrow" }).format(dateAt(d))}`, from: d, to: d, weekend: wd >= 5 });
      }
    } else {
      for (let w = start; w <= end; w += 7) bottom.push({ key: `w${w}`, label: shortDate(w), from: w, to: Math.min(w + 6, end) });
    }
  } else if (zoom === "month") {
    for (let y = ymdOf(start).y; y <= ymdOf(end).y; y++) top.push({ key: `y${y}`, label: String(y), from: Math.max(dayNum(y, 1, 1), start), to: Math.min(dayNum(y, 12, 31), end) });
    eachMonth((from, to) => bottom.push({ key: `m${from}`, label: fmt({ month: "short" }).format(dateAt(from)), from, to }));
  } else if (zoom === "quarter") {
    for (let y = ymdOf(start).y; y <= ymdOf(end).y; y++) {
      top.push({ key: `y${y}`, label: String(y), from: Math.max(dayNum(y, 1, 1), start), to: Math.min(dayNum(y, 12, 31), end) });
      for (let q = 0; q < 4; q++) {
        const from = dayNum(y, q * 3 + 1, 1);
        const to = dayNum(y, q * 3 + 4, 0);
        if (to >= start && from <= end) bottom.push({ key: `q${y}${q}`, label: `Q${q + 1}`, from: Math.max(from, start), to: Math.min(to, end) });
      }
    }
  } else {
    for (let y = ymdOf(start).y; y <= ymdOf(end).y; y++) bottom.push({ key: `y${y}`, label: String(y), from: Math.max(dayNum(y, 1, 1), start), to: Math.min(dayNum(y, 12, 31), end) });
  }
  return { top, bottom };
}
