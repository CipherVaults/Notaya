// Calendar view: a month or a week, rows placed by a date property (ranges are bars across days).
// Click an empty day to add a row, drag a row to another day to move it, rows without a date wait in a side list.

import { useEffect, useMemo, useRef, useState } from "react";
import { DndContext, DragOverlay, PointerSensor, pointerWithin, useDraggable, useDroppable, useSensor, useSensors, type DragEndEvent, type DragStartEvent } from "@dnd-kit/core";
import { CalendarDays, ChevronLeft, ChevronRight } from "lucide-react";
import type { DbRow, Property } from "@/lib/database-types";
import type { ViewProps } from "../viewTypes";
import { cn } from "@/lib/utils";
import {
  EmptyState,
  NewTitleInput,
  PropSelect,
  RowTitle,
  colorProp,
  dateAt,
  dayNum,
  fmt,
  hueOf,
  isoDay,
  propOf,
  readDate,
  rowColor,
  shiftCell,
  shortDate,
  spanOf,
  todayNum,
  useDragClickGuard,
  weekStart,
  weekdayNames,
  ymdOf,
  type DateSpan,
} from "./shared";

const MAX_LANES = 3;

interface Item {
  row: DbRow;
  span: DateSpan;
  color: string;
}

interface Seg {
  item: Item;
  /** Columns 0..6 of the week. */
  from: number;
  to: number;
  before: boolean;
  after: boolean;
  lane: number;
}

export default function CalendarView(props: ViewProps) {
  const { db, view, updateView } = props;
  const dp = propOf(db, view.dateProp);
  const dateProps = db.props.filter((p) => p.kind === "date");
  if (!dp || dp.kind !== "date") {
    return (
      <EmptyState icon={<CalendarDays size={28} />} title="Choose a date property for the calendar">
        {dateProps.length ? (
          <>
            <span>Rows appear on the days of this property.</span>
            <PropSelect label="Date property" placeholder="Date property…" props={dateProps} value="" onChange={(id) => updateView({ dateProp: id })} />
          </>
        ) : (
          <span>Add a date property to the database to use the calendar.</span>
        )}
      </EmptyState>
    );
  }
  return <CalendarInner {...props} dp={dp} dateProps={dateProps} />;
}

function CalendarInner({ db, view, result, openRow, setCells, createRow, updateView, dp, dateProps }: ViewProps & { dp: Property; dateProps: Property[] }) {
  const mode = view.calendarMode === "week" ? "week" : "month";
  const today = todayNum();
  const [anchor, setAnchor] = useState(today);
  const [adding, setAdding] = useState<number | null>(null);
  const [more, setMore] = useState<number | null>(null);
  const [dragging, setDragging] = useState<DbRow | null>(null);
  const grab = useRef<number | null>(null);
  const guard = useDragClickGuard();
  const cp = useMemo(() => colorProp(db), [db]);

  const { items, unscheduled } = useMemo(() => {
    const items: Item[] = [];
    const unscheduled: DbRow[] = [];
    for (const row of result.rows) {
      const span = spanOf(row.values[dp.id]);
      if (span) items.push({ row, span, color: rowColor(row, cp) });
      else unscheduled.push(row);
    }
    return { items, unscheduled };
  }, [result.rows, dp.id, cp]);

  /* ----- the weeks on screen ----- */
  const { y, m } = ymdOf(anchor);
  const weeks: number[] = useMemo(() => {
    if (mode === "week") return [weekStart(anchor)];
    const first = dayNum(y, m, 1);
    const last = dayNum(y, m + 1, 0);
    const out: number[] = [];
    for (let w = weekStart(first); w <= last; w += 7) out.push(w);
    return out;
  }, [mode, anchor, y, m]);

  const step = (dir: -1 | 1) => setAnchor(mode === "week" ? anchor + 7 * dir : dayNum(y, m + dir, 1));
  const title = mode === "week" ? `${shortDate(weeks[0])} – ${shortDate(weeks[0] + 6, true)}` : fmt({ month: "long", year: "numeric" }).format(dateAt(dayNum(y, m, 1)));
  const names = useMemo(() => weekdayNames("short"), []);

  /* ----- drag and drop ----- */
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }));
  const onDragStart = (e: DragStartEvent) => {
    setMore(null);
    const id = String(e.active.id).replace(/^(row|un):/, "").split("|")[0];
    setDragging(result.rows.find((r) => r.id === id) ?? null);
    grab.current = null;
    const ev = e.activatorEvent as PointerEvent | undefined;
    if (ev && typeof ev.clientX === "number") {
      const el = document.elementsFromPoint(ev.clientX, ev.clientY).find((n) => (n as HTMLElement).dataset?.day);
      if (el) grab.current = Number((el as HTMLElement).dataset.day);
    }
  };
  const onDragEnd = (e: DragEndEvent) => {
    guard.mark();
    setDragging(null);
    const over = e.over ? String(e.over.id) : "";
    if (!over.startsWith("day:")) return;
    const target = Number(over.slice(4));
    const id = String(e.active.id).replace(/^(row|un):/, "").split("|")[0];
    const row = result.rows.find((r) => r.id === id);
    if (!row) return;
    const cell = readDate(row.values[dp.id]);
    const span = spanOf(row.values[dp.id]);
    if (!cell || !span) {
      void setCells(id, { [dp.id]: { start: isoDay(target) } });
      return;
    }
    const from = grab.current !== null && grab.current >= span.start && grab.current <= span.end ? grab.current : span.start;
    const delta = target - from;
    if (delta) void setCells(id, { [dp.id]: shiftCell(cell, delta) });
  };

  const create = (day: number, titleText: string) => {
    setAdding(null);
    void createRow({ [dp.id]: { start: isoDay(day) } }, titleText);
  };

  return (
    <div className="flex h-full min-h-0 flex-col" dir="ltr" role="region" aria-label="Calendar">
      <div className="flex shrink-0 flex-wrap items-center gap-2 px-4 pb-2 pt-1">
        <h2 className="me-1 min-w-[8.5rem] text-[1rem] font-semibold" aria-live="polite" data-title>{title}</h2>
        <button type="button" aria-label={mode === "week" ? "Previous week" : "Previous month"} onClick={() => step(-1)} className="grid h-8 w-8 place-items-center rounded-lg text-muted hover:bg-surface-2 hover:text-fg">
          <ChevronLeft size={16} />
        </button>
        <button type="button" onClick={() => setAnchor(today)} className="h-8 rounded-lg border border-line px-3 text-[0.8rem] hover:bg-surface-2">Today</button>
        <button type="button" aria-label={mode === "week" ? "Next week" : "Next month"} onClick={() => step(1)} className="grid h-8 w-8 place-items-center rounded-lg text-muted hover:bg-surface-2 hover:text-fg">
          <ChevronRight size={16} />
        </button>
        <div className="ms-2 flex rounded-lg border border-line p-0.5" role="group" aria-label="Calendar mode">
          {(["month", "week"] as const).map((k) => (
            <button
              key={k}
              type="button"
              aria-pressed={mode === k}
              onClick={() => updateView({ calendarMode: k })}
              className={cn("h-7 rounded-md px-2.5 text-[0.8rem] capitalize", mode === k ? "bg-accent-soft text-accent" : "text-muted hover:text-fg")}
            >
              {k}
            </button>
          ))}
        </div>
        {dateProps.length > 1 ? (
          <PropSelect label="Date property" placeholder="Date property" props={dateProps} value={dp.id} onChange={(id) => updateView({ dateProp: id })} className="ms-auto" />
        ) : null}
      </div>

      <DndContext sensors={sensors} collisionDetection={pointerWithin} onDragStart={onDragStart} onDragEnd={onDragEnd} onDragCancel={() => setDragging(null)}>
        <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-hidden px-4 pb-4 lg:flex-row">
          <div className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden rounded-xl border border-line">
            <div className="grid shrink-0 grid-cols-7 border-b border-line bg-surface text-center text-[0.72rem] font-medium uppercase tracking-wide text-muted">
              {names.map((n) => <div key={n} className="py-1.5">{n}</div>)}
            </div>
            <div className="min-h-0 flex-1 overflow-y-auto">
              {weeks.map((ws) => (
                <WeekRow
                  key={ws}
                  ws={ws}
                  items={items}
                  mode={mode}
                  month={m}
                  today={today}
                  adding={adding}
                  more={more}
                  setMore={setMore}
                  onAdd={setAdding}
                  onCreate={create}
                  onOpen={(id) => {
                    if (!guard.recent()) openRow(id);
                  }}
                  fill={mode === "week"}
                />
              ))}
            </div>
          </div>

          <aside className="flex max-h-48 shrink-0 flex-col rounded-xl border border-line lg:max-h-none lg:w-52" aria-label="Rows without a date">
            <div className="flex items-center gap-2 border-b border-line px-3 py-2 text-[0.8rem] font-medium">
              <span>No date</span>
              <span className="text-xs font-normal tabular-nums text-faint" data-unscheduled-count>{unscheduled.length}</span>
            </div>
            <div className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-1.5">
              {unscheduled.length ? (
                unscheduled.map((r) => <Unscheduled key={r.id} row={r} onOpen={(id) => !guard.recent() && openRow(id)} />)
              ) : (
                <div className="px-1.5 py-2 text-[0.78rem] text-faint">Every row has a date.</div>
              )}
            </div>
            {unscheduled.length ? <div className="border-t border-line px-3 py-1.5 text-[0.72rem] text-faint">Drag a row onto a day to give it a date.</div> : null}
          </aside>
        </div>

        <DragOverlay dropAnimation={null}>
          {dragging ? (
            <div className="max-w-[14rem] rounded-md border border-line bg-elevated px-2 py-1 text-[0.78rem] shadow-float">
              <RowTitle row={dragging} />
            </div>
          ) : null}
        </DragOverlay>
      </DndContext>
    </div>
  );
}

/* ---------- One week ---------- */

function layoutWeek(items: Item[], ws: number): Seg[] {
  const we = ws + 6;
  const segs: Seg[] = items
    .filter((i) => i.span.start <= we && i.span.end >= ws)
    .sort((a, b) => a.span.start - b.span.start || b.span.end - b.span.start - (a.span.end - a.span.start) || a.row.number - b.row.number)
    .map((item) => ({
      item,
      from: Math.max(item.span.start, ws) - ws,
      to: Math.min(item.span.end, we) - ws,
      before: item.span.start < ws,
      after: item.span.end > we,
      lane: 0,
    }));
  const ends: number[] = [];
  for (const s of segs) {
    let l = ends.findIndex((e) => e < s.from);
    if (l < 0) l = ends.length;
    ends[l] = s.to;
    s.lane = l;
  }
  return segs;
}

function WeekRow({
  ws,
  items,
  mode,
  month,
  today,
  adding,
  more,
  setMore,
  onAdd,
  onCreate,
  onOpen,
  fill,
}: {
  ws: number;
  items: Item[];
  mode: "month" | "week";
  month: number;
  today: number;
  adding: number | null;
  more: number | null;
  setMore: (d: number | null) => void;
  onAdd: (d: number | null) => void;
  onCreate: (d: number, title: string) => void;
  onOpen: (id: string) => void;
  fill: boolean;
}) {
  const segs = useMemo(() => layoutWeek(items, ws), [items, ws]);
  const limit = mode === "month" ? MAX_LANES : Infinity;
  const shown = segs.filter((s) => s.lane < limit);
  const lanes = shown.reduce((n, s) => Math.max(n, s.lane + 1), 0);
  const hiddenIn = (col: number) => segs.filter((s) => s.lane >= limit && s.from <= col && s.to >= col).length;
  const anyHidden = [0, 1, 2, 3, 4, 5, 6].some((c) => hiddenIn(c) > 0);
  const rowsCount = lanes + (anyHidden ? 1 : 0);
  const days = [0, 1, 2, 3, 4, 5, 6].map((i) => ws + i);

  return (
    <div className={cn("relative border-b border-line last:border-b-0", fill ? "min-h-[24rem]" : "min-h-[6.75rem]")} data-week={ws}>
      <div className="absolute inset-0 grid grid-cols-7">
        {days.map((d, i) => (
          <DayCell key={d} day={d} out={mode === "month" && ymdOf(d).m !== month} today={today === d} last={i === 6} adding={adding === d} onAdd={() => onAdd(d)} onCancel={() => onAdd(null)} onCreate={(t) => onCreate(d, t)} />
        ))}
      </div>
      <div className="pointer-events-none relative grid grid-cols-7 gap-y-0.5 pb-1.5" style={{ gridTemplateRows: `1.75rem repeat(${Math.max(rowsCount, 0)}, 1.375rem)` }}>
        {days.map((d, i) => {
          const { d: dd } = ymdOf(d);
          const isToday = d === today;
          return (
            <div key={d} className="flex items-center justify-start ps-1.5 pt-1" style={{ gridColumn: i + 1, gridRow: 1 }}>
              <span
                className={cn(
                  "grid h-5 min-w-5 place-items-center rounded-full px-1 text-[0.72rem] tabular-nums",
                  isToday ? "bg-accent font-semibold text-accent-fg" : ymdOf(d).m !== month && mode === "month" ? "text-faint" : "text-muted",
                )}
                data-daynum={d}
              >
                {dd === 1 ? shortDate(d) : dd}
              </span>
            </div>
          );
        })}
        {shown.map((s) => (
          <Bar key={s.item.row.id + ":" + ws} seg={s} ws={ws} onOpen={onOpen} />
        ))}
        {anyHidden
          ? days.map((d, i) => {
              const n = hiddenIn(i);
              if (!n) return null;
              return (
                <div key={d} className="pointer-events-auto relative px-1" style={{ gridColumn: i + 1, gridRow: lanes + 2 }}>
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      setMore(more === d ? null : d);
                    }}
                    className="h-5 w-full truncate rounded px-1 text-start text-[0.72rem] text-muted hover:bg-surface-2 hover:text-fg"
                    data-more={d}
                  >
                    {n} more
                  </button>
                  {more === d ? <MorePopover day={d} items={items} onOpen={onOpen} onClose={() => setMore(null)} /> : null}
                </div>
              );
            })
          : null}
      </div>
    </div>
  );
}

function DayCell({ day, out, today, last, adding, onAdd, onCancel, onCreate }: { day: number; out: boolean; today: boolean; last: boolean; adding: boolean; onAdd: () => void; onCancel: () => void; onCreate: (t: string) => void }) {
  const { setNodeRef, isOver } = useDroppable({ id: `day:${day}` });
  const label = fmt({ weekday: "long", day: "numeric", month: "long", year: "numeric" }).format(dateAt(day));
  return (
    <div
      ref={setNodeRef}
      data-day={day}
      role="button"
      tabIndex={-1}
      aria-label={`Add a row on ${label}`}
      onClick={onAdd}
      className={cn(
        "relative cursor-pointer transition-colors",
        !last && "border-e border-line",
        out && "bg-surface/60",
        today && "bg-accent-soft/50",
        isOver && "bg-accent-soft",
      )}
    >
      {adding ? (
        <div className="absolute inset-x-1 bottom-1 z-10">
          <NewTitleInput onSubmit={onCreate} onCancel={onCancel} />
        </div>
      ) : null}
    </div>
  );
}

function Bar({ seg, ws, onOpen }: { seg: Seg; ws: number; onOpen: (id: string) => void }) {
  const { row, span, color } = seg.item;
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({ id: `row:${row.id}|${ws}` });
  const hue = hueOf(color);
  const style = hue
    ? { background: `color-mix(in oklab, ${hue} 26%, var(--bg))`, color: `color-mix(in oklab, ${hue} 55%, var(--text))`, borderColor: hue }
    : { background: "var(--surface-2)", color: "var(--text)", borderColor: "var(--accent)" };
  const time = !seg.before && span.startTime ? span.startTime : "";
  return (
    <div
      ref={setNodeRef}
      {...attributes}
      {...listeners}
      role="button"
      tabIndex={0}
      data-bar={row.id}
      aria-label={row.title || "Untitled"}
      onClick={() => onOpen(row.id)}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          onOpen(row.id);
        }
      }}
      className={cn(
        "pointer-events-auto mx-1 flex min-w-0 touch-none items-center gap-1 overflow-hidden border-s-[3px] px-1.5 text-[0.76rem] leading-none outline-none focus-visible:ring-2 focus-visible:ring-accent",
        seg.before ? "ms-0 rounded-s-none border-s-0" : "rounded-s-md",
        seg.after ? "me-0 rounded-e-none" : "rounded-e-md",
        isDragging && "opacity-40",
      )}
      style={{ gridColumn: `${seg.from + 1} / ${seg.to + 2}`, gridRow: seg.lane + 2, ...style }}
    >
      {time ? <span className="shrink-0 tabular-nums opacity-70">{time}</span> : null}
      <RowTitle row={row} className="min-w-0 flex-1" />
    </div>
  );
}

function Unscheduled({ row, onOpen }: { row: DbRow; onOpen: (id: string) => void }) {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({ id: `un:${row.id}` });
  return (
    <div
      ref={setNodeRef}
      {...attributes}
      {...listeners}
      role="button"
      tabIndex={0}
      data-unscheduled={row.id}
      onClick={() => onOpen(row.id)}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          onOpen(row.id);
        }
      }}
      className={cn("touch-none rounded-md border border-line bg-elevated px-2 py-1.5 text-[0.8rem] outline-none hover:bg-surface-2 focus-visible:ring-2 focus-visible:ring-accent", isDragging && "opacity-40")}
    >
      <RowTitle row={row} />
    </div>
  );
}

function MorePopover({ day, items, onOpen, onClose }: { day: number; items: Item[]; onOpen: (id: string) => void; onClose: () => void }) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const down = (e: PointerEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    const key = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    document.addEventListener("pointerdown", down, true);
    document.addEventListener("keydown", key);
    return () => {
      document.removeEventListener("pointerdown", down, true);
      document.removeEventListener("keydown", key);
    };
  }, [onClose]);
  const list = items.filter((i) => i.span.start <= day && i.span.end >= day);
  return (
    <div ref={ref} className="absolute start-0 top-0 z-30 w-60 rounded-xl border border-line bg-elevated p-2 shadow-float" role="dialog" aria-label={`Rows on ${shortDate(day, true)}`} data-more-popover>
      <div className="px-1 pb-1.5 text-[0.78rem] font-medium">{fmt({ weekday: "long", day: "numeric", month: "long" }).format(dateAt(day))}</div>
      <div className="flex max-h-60 flex-col gap-1 overflow-y-auto">
        {list.map((i) => (
          <button
            key={i.row.id}
            type="button"
            onClick={() => {
              onClose();
              onOpen(i.row.id);
            }}
            className="flex items-center gap-1.5 rounded-md px-1.5 py-1 text-start text-[0.8rem] hover:bg-surface-2"
          >
            <span className="h-2 w-2 shrink-0 rounded-full" style={{ background: hueOf(i.color) ?? "var(--accent)" }} />
            {i.span.startTime ? <span className="shrink-0 text-xs tabular-nums text-muted">{i.span.startTime}</span> : null}
            <RowTitle row={i.row} className="min-w-0 flex-1" />
          </button>
        ))}
      </div>
    </div>
  );
}
