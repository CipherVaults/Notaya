// Small pieces shared by the database views (board, calendar, timeline, list, gallery, chart).
// They only display values and build the patches the shell's setCells understands.

import { useMemo, useRef, type CSSProperties, type ReactNode } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Check, Paperclip, Plus } from "lucide-react";
import type { Database, DateCell, DbRow, FileCell, GroupBy, GroupOut, OptionColor, Property, SelectOption, View } from "@/lib/database-types";
import { cn } from "@/lib/utils";
import { formatNumber } from "../model";

/* ---------- Colors ---------- */

export const HUES: Record<string, string> = {
  gray: "#8b8680",
  brown: "#a9744f",
  orange: "#ee8a2b",
  yellow: "#d9a514",
  green: "#2f9e6a",
  blue: "#3b82f6",
  purple: "#8b5cf6",
  pink: "#ec4899",
  red: "#e5484d",
};

/** The solid hue of an option color ("" has none). */
export function hueOf(color: string | undefined): string | null {
  return color && HUES[color] ? HUES[color] : null;
}

/** A soft chip that reads on light and dark looks (it mixes the hue into transparent / the text color). */
export function chipStyle(color: string | undefined): CSSProperties {
  const h = hueOf(color);
  if (!h) return { background: "var(--surface-2)", color: "var(--text)" };
  return { background: `color-mix(in oklab, ${h} 24%, transparent)`, color: `color-mix(in oklab, ${h} 58%, var(--text))` };
}

export function Chip({ color, children, className }: { color?: OptionColor | string; children: ReactNode; className?: string }) {
  return (
    <span className={cn("inline-flex max-w-full items-center rounded-md px-1.5 py-px text-[0.72rem] leading-snug", className)} style={chipStyle(color)}>
      <span dir="auto" className="block truncate">{children}</span>
    </span>
  );
}

/* ---------- Properties and rows ---------- */

export function propOf(db: Database, id: string | undefined | null): Property | undefined {
  return id ? db.props.find((p) => p.id === id) : undefined;
}

export function optionOf(p: Property | undefined, id: unknown): SelectOption | undefined {
  return p && typeof id === "string" ? p.options.find((o) => o.id === id) : undefined;
}

export const titleOf = (row: DbRow) => row.title.trim();

export function RowTitle({ row, className, muted }: { row: DbRow; className?: string; muted?: boolean }) {
  const t = titleOf(row);
  return (
    // dir="auto": an Arabic or Hebrew title starts at the right edge and puts its icon there too.
    <span dir="auto" className={cn("flex min-w-0 items-center gap-1.5", className)}>
      {row.icon ? <span className="shrink-0 leading-none">{row.icon}</span> : null}
      <span className={cn("block min-w-0 truncate", !t && "text-faint", muted && "text-muted")}>{t || "Untitled"}</span>
    </span>
  );
}

/** Property columns the view shows on a card or list line (never the title, never `skip`). */
export function visibleProps(db: Database, view: View, skip: (string | undefined | null)[] = []): Property[] {
  const out: Property[] = [];
  for (const c of view.columns) {
    if (!c.visible || skip.includes(c.prop)) continue;
    const p = propOf(db, c.prop);
    if (p && p.kind !== "title") out.push(p);
  }
  return out;
}

/** The select / status property whose option color tints a bar or chip: the first status, else the first select. */
export function colorProp(db: Database): Property | undefined {
  return db.props.find((p) => p.kind === "status") ?? db.props.find((p) => p.kind === "select");
}

export function rowColor(row: DbRow, p: Property | undefined): string {
  return p ? optionOf(p, row.values[p.id])?.color ?? "" : "";
}

export function isComputed(v: unknown): v is { t: string; v: unknown; text: string } {
  return !!v && typeof v === "object" && !Array.isArray(v) && "t" in v && "text" in v;
}

/* ---------- Dates ---------- */

/** Days since 1970-01-01 of a calendar day (no time zone involved). */
export const dayNum = (y: number, m: number, d: number) => Math.floor(Date.UTC(y, m - 1, d) / 86400000);

export function ymdOf(day: number): { y: number; m: number; d: number } {
  const dt = new Date(day * 86400000);
  return { y: dt.getUTCFullYear(), m: dt.getUTCMonth() + 1, d: dt.getUTCDate() };
}

const p2 = (n: number) => String(n).padStart(2, "0");

export function isoDay(day: number): string {
  const { y, m, d } = ymdOf(day);
  return `${String(y).padStart(4, "0")}-${p2(m)}-${p2(d)}`;
}

export interface ParsedIso {
  day: number;
  /** "HH:MM" or null for a date without time. */
  time: string | null;
}

export function parseIso(s: unknown): ParsedIso | null {
  if (typeof s !== "string") return null;
  const m = /^(\d{4})-(\d{2})-(\d{2})(?:[T ](\d{2}):(\d{2}))?/.exec(s);
  if (!m) return null;
  return { day: dayNum(+m[1], +m[2], +m[3]), time: m[4] !== undefined ? `${m[4]}:${m[5]}` : null };
}

export const toIso = (day: number, time: string | null) => (time ? `${isoDay(day)}T${time}` : isoDay(day));

export function todayNum(): number {
  const n = new Date();
  return dayNum(n.getFullYear(), n.getMonth() + 1, n.getDate());
}

export function readDate(v: unknown): DateCell | null {
  if (v && typeof v === "object" && !Array.isArray(v) && typeof (v as DateCell).start === "string" && parseIso((v as DateCell).start)) return v as DateCell;
  return null;
}

export interface DateSpan {
  start: number;
  end: number;
  startTime: string | null;
  endTime: string | null;
  hasEnd: boolean;
}

/** The days a date cell covers (end defaults to start). */
export function spanOf(v: unknown): DateSpan | null {
  const c = readDate(v);
  if (!c) return null;
  const s = parseIso(c.start)!;
  const e = c.end ? parseIso(c.end) : null;
  const end = e && e.day >= s.day ? e : null;
  return { start: s.day, end: end ? end.day : s.day, startTime: s.time, endTime: end ? end.time : null, hasEnd: !!end };
}

/** The same cell moved by `delta` days (times, the end and the reminder are kept). */
export function shiftCell(c: DateCell, delta: number): DateCell {
  const s = parseIso(c.start)!;
  const out: DateCell = { ...c, start: toIso(s.day + delta, s.time) };
  const e = c.end ? parseIso(c.end) : null;
  if (e) out.end = toIso(e.day + delta, e.time);
  return out;
}

/** The cell with a new first and last day (the end is dropped when it equals the start and has no time). */
export function withDays(c: DateCell | null, start: number, end: number): DateCell {
  const s = c ? parseIso(c.start) : null;
  const e = c?.end ? parseIso(c.end) : null;
  const out: DateCell = { start: toIso(start, s?.time ?? null) };
  if (c?.reminder !== undefined) out.reminder = c.reminder;
  if (end > start || e?.time) out.end = toIso(Math.max(end, start), e?.time ?? s?.time ?? null);
  return out;
}

const fmtCache = new Map<string, Intl.DateTimeFormat>();
export function fmt(opts: Intl.DateTimeFormatOptions): Intl.DateTimeFormat {
  const k = JSON.stringify(opts);
  let f = fmtCache.get(k);
  if (!f) fmtCache.set(k, (f = new Intl.DateTimeFormat(undefined, { ...opts, timeZone: "UTC" })));
  return f;
}
export const dateAt = (day: number) => new Date(day * 86400000);

export function shortDate(day: number, withYear = false): string {
  return fmt({ day: "numeric", month: "short", ...(withYear ? { year: "numeric" } : {}) }).format(dateAt(day));
}

/** "Mar 3 → Mar 5" or "Mar 3 10:00" for a date cell. */
export function dateText(v: unknown): string {
  const sp = spanOf(v);
  if (!sp) return "";
  const withYear = ymdOf(sp.start).y !== new Date().getFullYear();
  const a = shortDate(sp.start, withYear) + (sp.startTime ? ` ${sp.startTime}` : "");
  if (!sp.hasEnd || (sp.end === sp.start && !sp.endTime)) return a;
  return `${a} → ${sp.end === sp.start ? "" : shortDate(sp.end, withYear)}${sp.endTime ? ` ${sp.endTime}` : ""}`.trim();
}

/** Weekday names starting Monday. */
export function weekdayNames(style: "short" | "narrow" | "long" = "short"): string[] {
  // 1970-01-05 is a Monday.
  return Array.from({ length: 7 }, (_, i) => fmt({ weekday: style }).format(dateAt(4 + i)));
}

/** Monday of the week of a day number (1970-01-01 was a Thursday). */
export const weekStart = (day: number) => day - ((((day + 3) % 7) + 7) % 7);

/* ---------- Files ---------- */

export function filesOf(v: unknown): FileCell[] {
  return Array.isArray(v) ? (v.filter((x) => x && typeof x === "object" && typeof (x as FileCell).id === "string") as FileCell[]) : [];
}

export function imageUrl(id: string): string {
  return convertFileSrc(id, "clatt");
}

export function firstImage(row: DbRow, propId: string): FileCell | null {
  return filesOf(row.values[propId]).find((f) => (f.mime ?? "").startsWith("image/")) ?? null;
}

/* ---------- Value display (compact) ---------- */

/** One value of a property as a small inline piece; null when empty. */
export function valueNode(prop: Property, value: unknown, titles?: Map<string, string>, numberText?: string): ReactNode {
  if (value === undefined || value === null || value === "") return null;
  if (isComputed(value)) {
    if (value.t === "null" || !value.text) return null;
    if (value.t === "error") return <span className="text-danger">{value.text}</span>;
    return <span dir="auto" className="block truncate">{value.text}</span>;
  }
  switch (prop.kind) {
    case "text":
    case "url":
    case "email":
    case "phone":
      return typeof value === "string" ? <span dir="auto" className="block truncate">{value}</span> : null;
    case "number":
      return typeof value === "number" ? <span className="tabular-nums">{numberText ?? formatNumber(value, prop.numberFormat)}</span> : null;
    case "select":
    case "status": {
      const o = optionOf(prop, value);
      return o ? <Chip color={o.color}>{o.name}</Chip> : null;
    }
    case "multiSelect": {
      const ids = Array.isArray(value) ? value : [];
      const opts = ids.map((i) => optionOf(prop, i)).filter(Boolean) as SelectOption[];
      return opts.length ? (
        <span className="flex min-w-0 flex-wrap gap-1">
          {opts.map((o) => <Chip key={o.id} color={o.color}>{o.name}</Chip>)}
        </span>
      ) : null;
    }
    case "date": {
      const t = dateText(value);
      return t ? <span className="whitespace-nowrap">{t}</span> : null;
    }
    case "person":
      return Array.isArray(value) && value.length ? <span dir="auto" className="block truncate">{value.join(", ")}</span> : null;
    case "files": {
      const n = filesOf(value).length;
      return n ? <span className="inline-flex items-center gap-1"><Paperclip size={11} />{n}</span> : null;
    }
    case "checkbox":
      return value === true ? <Check size={13} aria-label="Checked" /> : null;
    case "relation": {
      const ids = Array.isArray(value) ? (value as string[]) : [];
      if (!ids.length) return null;
      const names = titles ? ids.map((i) => titles.get(i)).filter(Boolean) : [];
      return <span dir="auto" className="block truncate">{names.length ? names.join(", ") : `${ids.length} linked`}</span>;
    }
    case "place":
      return (value as { address?: string }).address ? <span dir="auto" className="block truncate">{(value as { address: string }).address}</span> : null;
    default:
      return typeof value === "string" || typeof value === "number" ? <span className="block truncate">{String(value)}</span> : null;
  }
}

/** The properties of a row as small lines for a card; empty values are left out. */
export function PropLines({ row, props, compact, titles }: { row: DbRow; props: Property[]; compact?: boolean; titles?: Map<string, string> }) {
  const lines = props.map((p) => ({ p, node: valueNode(p, row.values[p.id], titles, row.numberText?.[p.id]) })).filter((x) => x.node !== null);
  if (!lines.length) return null;
  return (
    <div className={cn("flex min-w-0 flex-col text-[0.78rem] text-muted", compact ? "gap-0.5" : "gap-1")}>
      {lines.map(({ p, node }) => (
        <div key={p.id} className="flex min-w-0 items-center gap-1.5" title={p.name} data-prop={p.id}>
          <span className="min-w-0 flex-1 truncate">{node}</span>
        </div>
      ))}
    </div>
  );
}

/* ---------- Group helpers ---------- */

export function groupProp(db: Database, g: GroupBy | null | undefined): Property | undefined {
  return g ? propOf(db, g.prop) : undefined;
}

/** Kinds that can be dragged between groups and get a starting value in a new row. */
export function canGroupEdit(p: Property | undefined, g: GroupBy | null | undefined): boolean {
  if (!p || !g) return false;
  return ["select", "status", "multiSelect", "checkbox", "person", "text"].includes(p.kind);
}

/**
 * The cells that put a row into the group `key` of `g` (what a new row or a dropped card gets).
 * `from` is the group the row came from; for multi-value properties it is taken out first, so a move keeps other tags.
 * Returns null when the property can't be edited by dropping.
 */
export function groupPatch(
  p: Property | undefined,
  g: GroupBy | null | undefined,
  key: string,
  label: string,
  row?: DbRow | null,
  from?: { key: string; label: string },
): Record<string, unknown> | null {
  if (!p || !g || !canGroupEdit(p, g)) return null;
  const cur = row?.values[p.id];
  switch (p.kind) {
    case "select":
      return { [p.id]: key || null };
    case "status": {
      if (g.dateBy === "group") {
        if (!key) return { [p.id]: null };
        const o = p.options.find((x) => x.group === key);
        return o ? { [p.id]: o.id } : null;
      }
      return { [p.id]: key || null };
    }
    case "multiSelect": {
      let ids = Array.isArray(cur) ? (cur as string[]).filter((x) => x !== from?.key) : [];
      if (key && !ids.includes(key)) ids = [...ids, key];
      return { [p.id]: ids.length ? ids : null };
    }
    case "checkbox":
      return { [p.id]: key === "true" };
    case "person": {
      let names = Array.isArray(cur) ? (cur as string[]).filter((x) => x.toLowerCase() !== (from?.key ?? "\u0000")) : [];
      if (key && !names.some((x) => x.toLowerCase() === key)) names = [...names, label];
      return { [p.id]: names.length ? names : null };
    }
    case "text":
      return { [p.id]: key ? label : null };
    default:
      return null;
  }
}

export function setCollapsed(view: View, key: string, collapsed: boolean): Partial<View> | null {
  if (!view.group) return null;
  const set = new Set(view.group.collapsed);
  if (collapsed) set.add(key);
  else set.delete(key);
  return { group: { ...view.group, collapsed: [...set] } };
}

/** The label of a group (as a chip with its option color when it has one). */
export function GroupChip({ g }: { g: GroupOut }) {
  return g.color ? <Chip color={g.color}>{g.label}</Chip> : <span dir="auto" className="min-w-0 truncate text-[0.82rem] font-medium">{g.label}</span>;
}

/* ---------- Small controls ---------- */

export function PropSelect({
  value,
  onChange,
  props,
  placeholder,
  label,
  className,
  allowNone,
}: {
  value: string;
  onChange: (id: string) => void;
  props: Property[];
  placeholder: string;
  label: string;
  className?: string;
  allowNone?: string;
}) {
  return (
    <select
      aria-label={label}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className={cn("h-8 max-w-[14rem] rounded-lg border border-line bg-surface px-2 text-[0.8rem] text-fg outline-none focus:border-accent", className)}
    >
      <option value="" disabled={!allowNone}>{allowNone ?? placeholder}</option>
      {props.map((p) => <option key={p.id} value={p.id}>{p.name}</option>)}
    </select>
  );
}

export function EmptyState({ icon, title, children }: { icon?: ReactNode; title: string; children?: ReactNode }) {
  return (
    <div className="flex min-h-[14rem] flex-col items-center justify-center gap-2 px-6 py-10 text-center">
      {icon ? <div className="text-faint">{icon}</div> : null}
      <div className="text-[0.95rem] font-medium">{title}</div>
      {children ? <div className="flex max-w-md flex-col items-center gap-2 text-[0.82rem] text-muted">{children}</div> : null}
    </div>
  );
}

export function NewButton({ onClick, label = "New", className }: { onClick: () => void; label?: string; className?: string }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn("flex h-8 items-center gap-1.5 rounded-lg px-2 text-[0.8rem] text-muted transition-colors hover:bg-surface-2 hover:text-fg", className)}
    >
      <Plus size={14} />
      {label}
    </button>
  );
}

/** A one-line title field for a row that is about to be created: Enter adds, Esc or an empty blur cancels. */
export function NewTitleInput({
  onSubmit,
  onCancel,
  placeholder = "Untitled",
  className,
  keepOpen,
}: {
  onSubmit: (title: string) => void;
  onCancel: () => void;
  placeholder?: string;
  className?: string;
  /** Stay open after Enter (to add several). */
  keepOpen?: boolean;
}) {
  const ref = useRef<HTMLInputElement>(null);
  const done = useRef(false);
  const submit = (title: string, allowEmpty: boolean) => {
    if (done.current) return;
    if (!title.trim() && !allowEmpty) {
      done.current = true;
      onCancel();
      return;
    }
    if (!keepOpen) done.current = true;
    onSubmit(title.trim());
    if (keepOpen && ref.current) ref.current.value = "";
  };
  return (
    <input
      ref={ref}
      autoFocus
      dir="auto"
      aria-label="Title of the new row"
      placeholder={placeholder}
      spellCheck={false}
      className={cn("h-8 w-full rounded-lg border border-accent bg-elevated px-2 text-[0.85rem] text-fg outline-none placeholder:text-faint", className)}
      onClick={(e) => e.stopPropagation()}
      onKeyDown={(e) => {
        e.stopPropagation();
        if (e.key === "Enter") {
          e.preventDefault();
          submit(e.currentTarget.value, true);
        } else if (e.key === "Escape") {
          done.current = true;
          onCancel();
        }
      }}
      onBlur={(e) => submit(e.currentTarget.value, false)}
    />
  );
}

/** Row titles for relation values: a map built from the rows in view (other rows show as a count). */
export function titleMap(rows: DbRow[]): Map<string, string> {
  return new Map(rows.map((r) => [r.id, r.title || "Untitled"]));
}

/** Whether the click that ended a drag should be ignored (a card dropped on itself must not open). */
export function useDragClickGuard() {
  const last = useRef(0);
  return useMemo(() => ({
    mark: () => {
      last.current = Date.now();
    },
    recent: () => Date.now() - last.current < 250,
  }), []);
}
