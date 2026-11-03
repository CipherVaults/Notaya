// A small month calendar (no library) and the date editor of a Date property: start, optional end, optional time, reminder.

import { useState } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import type { DateCell } from "@/lib/database-types";
import { cn } from "@/lib/utils";
import { DAY_NAMES, MONTH_NAMES, daysInMonth, parseYmd, todayString, weekdayMon, ymdString } from "../model";

interface GridProps {
  /** Selected day (YYYY-MM-DD or with time); only the day part is used. */
  start?: string;
  end?: string;
  onPick: (ymd: string) => void;
  /** Month to show first (YYYY-MM-DD); defaults to the selection or today. */
  focus?: string;
}

export function MonthGrid({ start, end, onPick, focus }: GridProps) {
  const base = parseYmd(start) ?? parseYmd(focus) ?? parseYmd(todayString())!;
  const [ym, setYm] = useState({ y: base.y, m: base.m });
  const today = todayString();
  const s = start?.slice(0, 10);
  const e = end?.slice(0, 10);
  const lead = weekdayMon(ym.y, ym.m, 1);
  const total = daysInMonth(ym.y, ym.m);
  const cells: (number | null)[] = [...Array(lead).fill(null), ...Array.from({ length: total }, (_, i) => i + 1)];
  while (cells.length % 7) cells.push(null);
  const go = (delta: number) => {
    const d = new Date(ym.y, ym.m - 1 + delta, 1);
    setYm({ y: d.getFullYear(), m: d.getMonth() + 1 });
  };
  return (
    <div className="w-[248px] select-none">
      <div className="mb-1 flex items-center justify-between px-1">
        <span className="text-sm font-medium" dir="auto">
          {MONTH_NAMES[ym.m - 1]} {ym.y}
        </span>
        <span className="flex">
          <button type="button" onClick={() => go(-1)} aria-label="Previous month" className="rounded-md p-1 text-muted hover:bg-surface-2 hover:text-fg">
            <ChevronLeft size={15} className="rtl:rotate-180" />
          </button>
          <button type="button" onClick={() => go(1)} aria-label="Next month" className="rounded-md p-1 text-muted hover:bg-surface-2 hover:text-fg">
            <ChevronRight size={15} className="rtl:rotate-180" />
          </button>
        </span>
      </div>
      <div className="grid grid-cols-7 text-center text-[0.68rem] text-faint">
        {DAY_NAMES.map((d) => (
          <span key={d} className="py-1">{d.slice(0, 2)}</span>
        ))}
      </div>
      <div className="grid grid-cols-7 gap-y-0.5" role="grid" aria-label={`${MONTH_NAMES[ym.m - 1]} ${ym.y}`}>
        {cells.map((d, i) => {
          if (d == null) return <span key={i} />;
          const ymd = ymdString(ym.y, ym.m, d);
          const isStart = ymd === s;
          const isEnd = ymd === e;
          const inRange = !!s && !!e && ymd > s && ymd < e;
          return (
            <button
              key={i}
              type="button"
              onClick={() => onPick(ymd)}
              aria-label={ymd}
              aria-pressed={isStart || isEnd}
              className={cn(
                "h-8 text-[0.8rem] tabular-nums transition-colors",
                inRange && "bg-accent-soft",
                (isStart || isEnd) ? "rounded-md bg-accent font-semibold text-accent-fg" : "rounded-md hover:bg-surface-2",
                ymd === today && !isStart && !isEnd && "font-semibold text-accent-strong ring-1 ring-inset ring-accent/40",
              )}
            >
              {d}
            </button>
          );
        })}
      </div>
    </div>
  );
}

const REMINDERS: [number, string][] = [
  [-1, "None"], [0, "At time of event"], [5, "5 minutes before"], [10, "10 minutes before"], [15, "15 minutes before"], [30, "30 minutes before"],
  [60, "1 hour before"], [120, "2 hours before"], [1440, "1 day before"], [2880, "2 days before"], [10080, "1 week before"],
];

const inputCls =
  "h-8 min-w-0 flex-1 rounded-md border border-line bg-surface px-2 text-sm text-fg outline-none focus:border-accent focus:ring-2 focus:ring-accent-soft";

/** Edits a date cell. `onChange(null)` clears it. */
export function DateEditor({ value, onChange }: { value: DateCell | undefined; onChange: (v: DateCell | null) => void }) {
  const cur: DateCell = value ?? { start: "" };
  const hasEnd = cur.end !== undefined;
  const sp = parseYmd(cur.start);
  const ep = parseYmd(cur.end);
  const timeOn = !!(sp?.time);
  const [picking, setPicking] = useState<"start" | "end">("start");

  const commit = (next: DateCell) => {
    if (!next.start) {
      onChange(null);
      return;
    }
    const out: DateCell = { start: next.start };
    if (next.end !== undefined && next.end >= next.start) out.end = next.end;
    else if (next.end !== undefined) out.end = next.start;
    if (next.reminder !== undefined && next.reminder >= 0) out.reminder = next.reminder;
    onChange(out);
  };
  const withTime = (ymd: string, on: boolean, from?: { hh: number; mm: number }) => (on ? `${ymd.slice(0, 10)}T${String(from?.hh ?? 9).padStart(2, "0")}:${String(from?.mm ?? 0).padStart(2, "0")}` : ymd.slice(0, 10));

  const pick = (ymd: string) => {
    if (hasEnd && picking === "end" && cur.start) {
      commit({ ...cur, end: withTime(ymd, timeOn, ep ?? undefined) });
      setPicking("start");
    } else {
      const start = withTime(ymd, timeOn, sp ?? undefined);
      commit({ ...cur, start, end: hasEnd ? (cur.end && cur.end >= start ? cur.end : start) : undefined });
      if (hasEnd) setPicking("end");
    }
  };
  const setTime = (which: "start" | "end", t: string) => {
    const [hh, mm] = t.split(":").map(Number);
    if (Number.isNaN(hh)) return;
    const day = (which === "start" ? cur.start : cur.end ?? cur.start).slice(0, 10);
    const s = `${day}T${String(hh).padStart(2, "0")}:${String(mm || 0).padStart(2, "0")}`;
    commit(which === "start" ? { ...cur, start: s } : { ...cur, end: s });
  };
  const timeOf = (p: ReturnType<typeof parseYmd>) => (p?.time ? `${String(p.hh).padStart(2, "0")}:${String(p.mm).padStart(2, "0")}` : "");

  return (
    <div className="space-y-2.5 p-1">
      <div className="flex gap-1.5">
        <input
          type="date"
          aria-label="Start date"
          className={inputCls}
          value={cur.start.slice(0, 10)}
          onChange={(e) => e.target.value && commit({ ...cur, start: withTime(e.target.value, timeOn, sp ?? undefined) })}
        />
        {timeOn && <input type="time" aria-label="Start time" className={cn(inputCls, "max-w-[6.5rem]")} value={timeOf(sp)} onChange={(e) => setTime("start", e.target.value)} />}
      </div>
      {hasEnd && (
        <div className="flex gap-1.5">
          <input
            type="date"
            aria-label="End date"
            className={inputCls}
            value={(cur.end ?? "").slice(0, 10)}
            onChange={(e) => e.target.value && commit({ ...cur, end: withTime(e.target.value, timeOn, ep ?? undefined) })}
          />
          {timeOn && <input type="time" aria-label="End time" className={cn(inputCls, "max-w-[6.5rem]")} value={timeOf(ep)} onChange={(e) => setTime("end", e.target.value)} />}
        </div>
      )}
      <MonthGrid start={cur.start || undefined} end={cur.end} onPick={pick} />
      <div className="space-y-1.5 border-t border-line pt-2 text-sm">
        <label className="flex cursor-pointer items-center justify-between">
          <span>End date</span>
          <input
            type="checkbox"
            checked={hasEnd}
            disabled={!cur.start}
            onChange={(e) => {
              if (e.target.checked) {
                commit({ ...cur, end: cur.start });
                setPicking("end");
              } else commit({ ...cur, end: undefined });
            }}
            className="h-4 w-4 accent-[var(--accent)]"
          />
        </label>
        <label className="flex cursor-pointer items-center justify-between">
          <span>Include time</span>
          <input
            type="checkbox"
            checked={timeOn}
            disabled={!cur.start}
            onChange={(e) => {
              const on = e.target.checked;
              commit({ ...cur, start: withTime(cur.start, on, sp ?? undefined), end: cur.end !== undefined ? withTime(cur.end, on, ep ?? undefined) : undefined });
            }}
            className="h-4 w-4 accent-[var(--accent)]"
          />
        </label>
        <label className="flex items-center justify-between gap-2">
          <span>Remind</span>
          <select
            value={cur.reminder ?? -1}
            disabled={!cur.start}
            onChange={(e) => commit({ ...cur, reminder: Number(e.target.value) })}
            className="h-8 max-w-[10.5rem] rounded-md border border-line bg-surface px-1.5 text-sm text-fg outline-none focus:border-accent"
            aria-label="Reminder"
          >
            {REMINDERS.map(([m, l]) => (
              <option key={m} value={m}>{l}</option>
            ))}
          </select>
        </label>
      </div>
      <button type="button" onClick={() => onChange(null)} className="w-full rounded-md py-1.5 text-center text-sm text-muted hover:bg-surface-2 hover:text-fg">
        Clear
      </button>
    </div>
  );
}
