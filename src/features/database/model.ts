// Shared helpers of the database UI: ids, defaults, property kinds, operators, number and date formatting.
// Values are computed in Rust; this file only knows how to show them and how to build the objects the UI edits.

import type {
  ChartConfig, Cond, Database, DateCell, DbRow, Filter, GroupBy, OptionColor, PropKind, Property, SelectOption, View, ViewKind,
} from "@/lib/database-types";

/** A 32-character hex id, like the ones the backend makes. */
export function newId(): string {
  const a = new Uint8Array(16);
  crypto.getRandomValues(a);
  return Array.from(a, (b) => b.toString(16).padStart(2, "0")).join("");
}

// ---------------------------------------------------------------- property kinds

export interface KindInfo {
  label: string;
  /** Icon name from lucide-react (resolved in KindIcon). */
  icon: string;
  /** Computed by Notaya, never typed in. */
  computed?: boolean;
}

export const KINDS: Record<PropKind, KindInfo> = {
  title: { label: "Title", icon: "Type" },
  text: { label: "Text", icon: "AlignLeft" },
  number: { label: "Number", icon: "Hash" },
  select: { label: "Select", icon: "CircleDot" },
  multiSelect: { label: "Multi-select", icon: "Tags" },
  status: { label: "Status", icon: "Loader" },
  date: { label: "Date", icon: "Calendar" },
  person: { label: "Person", icon: "User" },
  files: { label: "Files & media", icon: "Paperclip" },
  checkbox: { label: "Checkbox", icon: "SquareCheck" },
  url: { label: "URL", icon: "Link" },
  email: { label: "Email", icon: "Mail" },
  phone: { label: "Phone", icon: "Phone" },
  formula: { label: "Formula", icon: "Sigma", computed: true },
  relation: { label: "Relation", icon: "ArrowUpRight" },
  rollup: { label: "Rollup", icon: "Layers", computed: true },
  createdTime: { label: "Created time", icon: "Clock", computed: true },
  createdBy: { label: "Created by", icon: "UserPen", computed: true },
  lastEditedTime: { label: "Last edited time", icon: "History", computed: true },
  lastEditedBy: { label: "Last edited by", icon: "UserPen", computed: true },
  id: { label: "ID", icon: "Fingerprint", computed: true },
  place: { label: "Place", icon: "MapPin" },
  button: { label: "Button", icon: "MousePointerClick", computed: true },
};

/** The order kinds are offered in the "new property" picker. */
export const KIND_ORDER: PropKind[] = [
  "text", "number", "select", "multiSelect", "status", "date", "person", "files", "checkbox", "url", "email", "phone",
  "formula", "relation", "rollup", "place", "button", "createdTime", "createdBy", "lastEditedTime", "lastEditedBy", "id",
];

export const isComputedKind = (k: PropKind) => !!KINDS[k].computed || k === "createdTime";
export const hasOptions = (k: PropKind) => k === "select" || k === "multiSelect" || k === "status";

export function blankProperty(kind: PropKind, name?: string): Property {
  return {
    id: newId(), name: name ?? KINDS[kind].label, kind, options: [], numberFormat: "number", numberDisplay: "", numberMax: 0,
    dateFormat: "full", formula: "", relationDb: "", relationBack: "", relationLimitOne: false, rollupRelation: "", rollupProp: "",
    rollupCalc: "", idPrefix: "", description: "", buttonLabel: "", actions: [],
  };
}

export function blankOption(name: string, color: OptionColor, group: SelectOption["group"] = ""): SelectOption {
  return { id: newId(), name, color, group };
}

export const OPTION_COLORS: OptionColor[] = ["", "gray", "brown", "orange", "yellow", "green", "blue", "purple", "pink", "red"];
export const COLOR_NAMES: Record<string, string> = {
  "": "Default", gray: "Gray", brown: "Brown", orange: "Orange", yellow: "Yellow", green: "Green", blue: "Blue", purple: "Purple", pink: "Pink", red: "Red",
};
/** CSS class of an option chip (see app.css, `.oc-*`). */
export const optClass = (c: string | undefined) => `oc oc-${c || "default"}`;
/** The solid dot color of an option color. */
export const optDot = (c: string | undefined) => `var(--oc-${c || "default"})`;
const NEXT_COLORS: OptionColor[] = ["gray", "blue", "green", "orange", "purple", "pink", "yellow", "red", "brown"];
export const colorForNew = (n: number): OptionColor => NEXT_COLORS[n % NEXT_COLORS.length];

export const STATUS_GROUPS: { id: "todo" | "inProgress" | "complete"; label: string }[] = [
  { id: "todo", label: "To-do" },
  { id: "inProgress", label: "In progress" },
  { id: "complete", label: "Complete" },
];

export const NUMBER_FORMATS: [string, string][] = [
  ["number", "Number"], ["numberWithCommas", "Number with commas"], ["percent", "Percent"], ["dollar", "Dollar ($)"], ["euro", "Euro (€)"],
  ["pound", "Pound (£)"], ["yen", "Yen (¥)"], ["rupee", "Rupee (₹)"], ["won", "Won (₩)"], ["yuan", "Yuan (CN¥)"], ["rial", "Rial (﷼)"],
];
export const DATE_FORMATS: [string, string][] = [
  ["full", "Full date (November 3, 2026)"], ["short", "Short (Nov 3, 2026)"], ["dmy", "Day/Month/Year"], ["mdy", "Month/Day/Year"], ["ymd", "Year/Month/Day"], ["relative", "Relative"],
];

// ---------------------------------------------------------------- calculations

export const CALC_LABELS: Record<string, string> = {
  none: "None", countAll: "Count all", countValues: "Count values", countUnique: "Count unique values", countEmpty: "Count empty",
  countNotEmpty: "Count not empty", percentEmpty: "Percent empty", percentNotEmpty: "Percent not empty", sum: "Sum", avg: "Average",
  median: "Median", min: "Min", max: "Max", range: "Range", earliest: "Earliest date", latest: "Latest date", dateRange: "Date range",
  checked: "Checked", unchecked: "Unchecked", percentChecked: "Percent checked", percentUnchecked: "Percent unchecked",
  showOriginal: "Show original", showUnique: "Show unique values",
};
const COMMON = ["none", "countAll", "countValues", "countUnique", "countEmpty", "countNotEmpty", "percentEmpty", "percentNotEmpty"];
/** Calculations that make sense for a property kind (footers, rollups). */
export function calcsFor(kind: PropKind, forRollup = false): string[] {
  const base = forRollup ? ["countAll", "countValues", "countUnique", "countEmpty", "countNotEmpty", "percentEmpty", "percentNotEmpty"] : COMMON;
  const extra = forRollup ? ["showOriginal", "showUnique"] : [];
  switch (kind) {
    case "number": case "formula": case "rollup": return [...base, "sum", "avg", "median", "min", "max", "range", ...extra];
    case "date": case "createdTime": case "lastEditedTime": return [...base, "earliest", "latest", "dateRange", ...extra];
    case "checkbox": return [...base.filter((c) => ["none", "countAll"].includes(c)), "checked", "unchecked", "percentChecked", "percentUnchecked", ...extra];
    default: return [...base, ...extra];
  }
}

// ---------------------------------------------------------------- filter operators (match Ctx::test in engine.rs)

export interface OpInfo { id: string; label: string; /** Needs no value. */ none?: boolean; /** Needs two values (between). */ two?: boolean }
const O = (id: string, label: string, extra: Partial<OpInfo> = {}): OpInfo => ({ id, label, ...extra });
const EMPTY = [O("isEmpty", "Is empty", { none: true }), O("isNotEmpty", "Is not empty", { none: true })];
const TEXT_OPS = [O("contains", "Contains"), O("notContains", "Does not contain"), O("equals", "Is"), O("notEquals", "Is not"), O("startsWith", "Starts with"), O("endsWith", "Ends with"), ...EMPTY];
const NUM_OPS = [O("eq", "="), O("neq", "≠"), O("gt", ">"), O("gte", "≥"), O("lt", "<"), O("lte", "≤"), ...EMPTY];
const DATE_OPS = [O("is", "Is"), O("before", "Is before"), O("after", "Is after"), O("onOrBefore", "Is on or before"), O("onOrAfter", "Is on or after"), O("between", "Is between", { two: true }), O("isNot", "Is not"), ...EMPTY];
const SELECT_OPS = [O("is", "Is"), O("isNot", "Is not"), ...EMPTY];
const MULTI_OPS = [O("contains", "Contains"), O("notContains", "Does not contain"), ...EMPTY];

export function opsFor(kind: PropKind): OpInfo[] {
  switch (kind) {
    case "number": return NUM_OPS;
    case "select": case "status": return SELECT_OPS;
    case "multiSelect": case "relation": return MULTI_OPS;
    case "person": case "files": return MULTI_OPS;
    case "checkbox": return [O("is", "Is"), O("isNot", "Is not")];
    case "date": case "createdTime": case "lastEditedTime": return DATE_OPS;
    case "formula": case "rollup": return [O("contains", "Contains"), O("notContains", "Does not contain"), O("equals", "Is"), O("notEquals", "Is not"), O("gt", ">"), O("gte", "≥"), O("lt", "<"), O("lte", "≤"), ...EMPTY];
    default: return TEXT_OPS;
  }
}
export const DATE_PRESETS: [string, string][] = [
  ["exact", "Exact date"], ["today", "Today"], ["tomorrow", "Tomorrow"], ["yesterday", "Yesterday"], ["oneWeekAgo", "One week ago"], ["oneWeekFromNow", "One week from now"],
  ["oneMonthAgo", "One month ago"], ["oneMonthFromNow", "One month from now"], ["thisWeek", "This week"], ["pastWeek", "Past week"], ["nextWeek", "Next week"],
  ["thisMonth", "This month"], ["pastMonth", "Past month"], ["nextMonth", "Next month"], ["thisYear", "This year"], ["pastYear", "Past year"], ["nextYear", "Next year"],
];

export const emptyFilter = (): Filter => ({ op: "and", conds: [], groups: [] });
export const filterIsEmpty = (f: Filter): boolean => f.conds.length === 0 && f.groups.every(filterIsEmpty);
export function countConds(f: Filter): number {
  return f.conds.length + f.groups.reduce((n, g) => n + countConds(g), 0);
}
export function blankCond(p: Property): Cond {
  const op = opsFor(p.kind)[0].id;
  const isDate = p.kind === "date" || p.kind === "createdTime" || p.kind === "lastEditedTime";
  return { prop: p.id, op, value: p.kind === "checkbox" ? "true" : "", value2: "", values: [], rel: isDate ? "exact" : "" };
}

// ---------------------------------------------------------------- views

export const VIEW_KINDS: { kind: ViewKind; label: string; icon: string }[] = [
  { kind: "table", label: "Table", icon: "Table" },
  { kind: "board", label: "Board", icon: "Columns3" },
  { kind: "calendar", label: "Calendar", icon: "CalendarDays" },
  { kind: "timeline", label: "Timeline", icon: "GanttChart" },
  { kind: "list", label: "List", icon: "List" },
  { kind: "gallery", label: "Gallery", icon: "LayoutGrid" },
  { kind: "chart", label: "Chart", icon: "ChartColumn" },
];

export const blankChart = (): ChartConfig => ({
  kind: "bar", xProp: "", xBy: "month", xStep: 0, xSort: "", groupProp: "", yCalc: "count", yProp: "", stacked: false, horizontal: false,
  showLabels: true, hideEmpty: false, color: "", title: "",
});

export const blankGroup = (prop: string): GroupBy => ({ prop, sort: "manual", dateBy: "", numberStep: 0, hideEmpty: false, hidden: [], collapsed: [] });

export function blankView(kind: ViewKind, name: string, db?: Database): View {
  const v: View = {
    id: newId(), name, kind, filter: emptyFilter(), sorts: [], group: null, subGroup: null,
    columns: (db?.props ?? []).map((p) => ({ prop: p.id, visible: true, width: 0 })),
    calcs: [], freeze: 1, wrap: false, rowNumbers: false, openIn: "full", cardSize: "medium", cardCover: "", cardFit: false,
    dateProp: "", endProp: "", calendarMode: "month", timelineZoom: "month", showDeps: true, showSubItems: true, chart: blankChart(),
  };
  if (db) {
    const date = db.props.find((p) => p.kind === "date");
    const grp = db.props.find((p) => p.kind === "status") ?? db.props.find((p) => p.kind === "select");
    if (kind === "calendar" || kind === "timeline") v.dateProp = date?.id ?? "";
    if (kind === "board" && grp) v.group = blankGroup(grp.id);
    if (kind === "chart") v.chart.xProp = grp?.id ?? db.props.find((p) => p.kind === "checkbox")?.id ?? "";
  }
  return v;
}

// ---------------------------------------------------------------- numbers

const groupDigits = (s: string) => s.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
export function numStr(n: number): string {
  return String(Math.round(n * 1e9) / 1e9);
}
export function formatNumber(n: number, format: string): string {
  const base = numStr(n);
  const withCommas = (s: string) => {
    const [i, f] = s.split(".");
    return groupDigits(i) + (f ? "." + f : "");
  };
  const money = (sym: string) => `${n < 0 ? "-" : ""}${sym}${withCommas(numStr(Math.abs(n)))}`;
  switch (format) {
    case "numberWithCommas": return withCommas(base);
    case "percent": return numStr(n * 100) + "%";
    case "dollar": return money("$");
    case "euro": return money("€");
    case "pound": return money("£");
    case "yen": return money("¥");
    case "rupee": return money("₹");
    case "won": return money("₩");
    case "yuan": return money("CN¥");
    case "rial": return money("﷼");
    default: return base;
  }
}

// ---------------------------------------------------------------- dates (local calendar days; no time zones)

export const pad2 = (n: number) => String(n).padStart(2, "0");
export const MONTH_NAMES = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
export const DAY_NAMES = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

export interface Ymd { y: number; m: number; d: number; hh: number; mm: number; time: boolean }
export function parseYmd(s: string | undefined | null): Ymd | null {
  if (!s) return null;
  const m = /^(\d{4})-(\d{2})-(\d{2})(?:[T ](\d{2}):(\d{2}))?/.exec(s);
  if (!m) return null;
  return { y: +m[1], m: +m[2], d: +m[3], hh: m[4] ? +m[4] : 0, mm: m[5] ? +m[5] : 0, time: !!m[4] };
}
export function ymdString(y: number, m: number, d: number, time?: { hh: number; mm: number } | null): string {
  const day = `${y}-${pad2(m)}-${pad2(d)}`;
  return time ? `${day}T${pad2(time.hh)}:${pad2(time.mm)}` : day;
}
export function todayString(): string {
  const n = new Date();
  return ymdString(n.getFullYear(), n.getMonth() + 1, n.getDate());
}
const dayNumber = (p: Ymd) => Math.floor(Date.UTC(p.y, p.m - 1, p.d) / 86400000);
export function daysFromToday(s: string): number | null {
  const p = parseYmd(s);
  if (!p) return null;
  const n = new Date();
  return dayNumber(p) - Math.floor(Date.UTC(n.getFullYear(), n.getMonth(), n.getDate()) / 86400000);
}

function oneDate(s: string, fmt: string): string {
  const p = parseYmd(s);
  if (!p) return s;
  const time = p.time ? ` ${pad2(p.hh)}:${pad2(p.mm)}` : "";
  if (fmt === "relative") {
    const d = daysFromToday(s)!;
    const rel = d === 0 ? "Today" : d === 1 ? "Tomorrow" : d === -1 ? "Yesterday" : d > 0 ? `In ${d} days` : `${-d} days ago`;
    return rel + time;
  }
  const date =
    fmt === "short" ? `${MONTH_NAMES[p.m - 1].slice(0, 3)} ${p.d}, ${p.y}`
      : fmt === "dmy" ? `${pad2(p.d)}/${pad2(p.m)}/${p.y}`
        : fmt === "mdy" ? `${pad2(p.m)}/${pad2(p.d)}/${p.y}`
          : fmt === "ymd" ? `${p.y}/${pad2(p.m)}/${pad2(p.d)}`
            : `${MONTH_NAMES[p.m - 1]} ${p.d}, ${p.y}`;
  return date + time;
}
export function formatDateCell(c: DateCell, fmt: string): string {
  const a = oneDate(c.start, fmt);
  return c.end ? `${a} → ${oneDate(c.end, fmt)}` : a;
}

export function daysInMonth(y: number, m: number): number {
  return new Date(y, m, 0).getDate();
}
/** Monday-first weekday index (0..6) of a date. */
export function weekdayMon(y: number, m: number, d: number): number {
  return (new Date(y, m - 1, d).getDay() + 6) % 7;
}

// ---------------------------------------------------------------- files

export const MAX_FILE_BYTES = 900 * 1024 * 1024;
export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${Math.round(n / 102.4) / 10} KB`;
  return `${Math.round(n / 104857.6) / 10} MB`;
}

// ---------------------------------------------------------------- misc

export function reorderList<T>(list: T[], from: number, to: number): T[] {
  const a = [...list];
  const [x] = a.splice(from, 1);
  a.splice(to, 0, x);
  return a;
}

export function isComputedValue(v: unknown): v is { t: string; v: unknown; text: string } {
  return typeof v === "object" && v !== null && !Array.isArray(v) && "t" in v && "text" in v;
}

/** Text of a cell for tooltips and plain lists. */
export function cellText(p: Property, v: unknown): string {
  if (v == null) return "";
  if (isComputedValue(v)) return String(v.text);
  if (typeof v === "string") return v;
  if (typeof v === "number") return formatNumber(v, p.numberFormat);
  if (typeof v === "boolean") return v ? "Yes" : "No";
  return "";
}

export function rowMatchesFind(props: Property[], row: DbRow, query: string): boolean {
  const has = (s: string) => s.toLowerCase().includes(query);
  return has(row.title) || props.some((p) => {
    if (p.kind === "title") return false;
    const value = row.values[p.id];
    if (has(cellText(p, value))) return true;
    const name = (p.kind === "select" || p.kind === "status") && p.options.find((o) => o.id === value)?.name;
    return typeof name === "string" && has(name);
  });
}
