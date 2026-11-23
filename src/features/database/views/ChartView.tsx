// Chart view: bar (vertical or horizontal, grouped or stacked), line, donut and a single number, drawn as
// hand-written SVG from `chart_data` (the numbers are calculated in Rust). The settings live in the view header.

import { useEffect, useId, useMemo, useRef, useState, type ReactNode } from "react";
import { BarChart3, ChartLine, ChartPie, Hash, RefreshCw, Settings2, Table2 } from "lucide-react";
import { api } from "@/lib/ipc";
import type { ChartConfig, ChartData, ChartSeries, Database, GroupBy, Property } from "@/lib/database-types";
import type { ViewProps } from "../viewTypes";
import { cn } from "@/lib/utils";
import { isComputedKind } from "../model";
import ChartDrill, { type Drill } from "./ChartDrill";
import { axisFraction, niceTicks } from "./chartScale";
import { EmptyState, HUES, PropSelect, propOf } from "./shared";

const KINDS = [
  { id: "bar", name: "Bar", icon: BarChart3 },
  { id: "line", name: "Line", icon: ChartLine },
  { id: "donut", name: "Donut", icon: ChartPie },
  { id: "number", name: "Number", icon: Hash },
] as const;

const CALCS: { id: string; name: string; needsProp: boolean; numeric?: boolean }[] = [
  { id: "countAll", name: "Count of rows", needsProp: false },
  { id: "countValues", name: "Count of values", needsProp: true },
  { id: "countUnique", name: "Count of unique values", needsProp: true },
  { id: "countEmpty", name: "Count of empty", needsProp: true },
  { id: "countNotEmpty", name: "Count of not empty", needsProp: true },
  { id: "percentNotEmpty", name: "Percent not empty", needsProp: true },
  { id: "sum", name: "Sum", needsProp: true, numeric: true },
  { id: "avg", name: "Average", needsProp: true, numeric: true },
  { id: "median", name: "Median", needsProp: true, numeric: true },
  { id: "min", name: "Minimum", needsProp: true, numeric: true },
  { id: "max", name: "Maximum", needsProp: true, numeric: true },
  { id: "range", name: "Range", needsProp: true, numeric: true },
];

const X_KINDS = ["select", "status", "multiSelect", "person", "checkbox", "date", "number", "text", "createdTime", "lastEditedTime", "formula", "url", "email", "phone"];
const NUMERIC_KINDS = ["number", "formula", "rollup"];
const PALETTE = ["blue", "orange", "green", "purple", "pink", "yellow", "red", "brown", "gray"];

/** A color name of an option ("blue") or a CSS color becomes a CSS color. */
function paint(c: string | undefined, i = 0): string {
  if (c && (c.startsWith("#") || c.startsWith("var(") || c.startsWith("rgb") || c.startsWith("hsl"))) return c;
  return HUES[c && HUES[c] ? c : PALETTE[i % PALETTE.length]];
}

/** Which part of a chart was clicked: a label (a bar, a slice, a point), a series, or both. Nothing = every row of the chart. */
export interface ChartPick {
  li?: number;
  si?: number;
}

export default function ChartView({ db, view, result, updateView, openRow }: ViewProps) {
  const cfg = view.chart;
  const [data, setData] = useState<ChartData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [table, setTable] = useState(false);
  const [panel, setPanel] = useState(!cfg.xProp && cfg.kind !== "number");
  const [retry, setRetry] = useState(0);
  const [drill, setDrill] = useState<Drill | null>(null);
  const seq = useRef(0);
  const drillSeq = useRef(0);
  const key = JSON.stringify([db.id, view.id, view.chart, view.filter]);

  useEffect(() => {
    const mine = ++seq.current;
    setLoading(true);
    const t = setTimeout(() => {
      api
        .chartData({ databaseId: db.id, viewId: view.id, view })
        .then((d) => {
          if (seq.current !== mine) return;
          setData(d);
          setError(null);
          setLoading(false);
        })
        .catch((e) => {
          if (seq.current !== mine) return;
          setError(typeof e === "string" ? e : e instanceof Error ? e.message : (e as { message?: string })?.message ?? "The chart could not be calculated.");
          setLoading(false);
        });
    }, 40);
    return () => clearTimeout(t);
    // The view object changes on every settings edit; `key` and the rows are what matter.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, result.rows, retry]);

  const patch = (p: Partial<ChartConfig>) => updateView({ chart: { ...cfg, ...p } });
  const needsX = cfg.kind !== "number";

  // Click on a part of the chart: ask the backend for the rows with the same grouping the chart used, so the list
  // always agrees with the numbers (dates by month, numbers in steps, options, empty values).
  const openDrill = (pick: ChartPick) => {
    if (!data) return;
    const x = needsX ? propOf(db, cfg.xProp) : undefined;
    const g = propOf(db, cfg.groupProp);
    const label = pick.li !== undefined ? data.labels[pick.li] ?? "" : undefined;
    const series = pick.si !== undefined && data.series.length > 1 ? data.series[pick.si]?.name ?? "" : undefined;
    const chips: string[] = [];
    if (x && label !== undefined) chips.push(`${x.name} is ${label || "empty"}`);
    if (g && series !== undefined) chips.push(`${g.name} is ${series || "empty"}`);
    if (!chips.length) chips.push(cfg.kind === "number" ? "All rows counted in this number" : "All rows in this chart");
    const title = label !== undefined ? label : series !== undefined ? series : cfg.title || defaultTitle(db, cfg);
    const others = db.props.filter((p) => p.kind !== "title" && p.id !== x?.id && p.id !== g?.id && !isComputedKind(p.kind) && p.kind !== "button" && p.kind !== "files");
    const cols = [...(x ? [x] : []), ...(g ? [g] : []), ...others].slice(0, 3);
    const mine = ++drillSeq.current;
    setDrill({ title, chips, rows: [], cols, loading: true, error: null });
    const xg: GroupBy | null = x ? { prop: x.id, sort: cfg.xSort || "manual", dateBy: cfg.xBy, numberStep: cfg.xStep, hideEmpty: cfg.hideEmpty, hidden: [], collapsed: [] } : null;
    const sg: GroupBy | null = x && g ? { prop: g.id, sort: "manual", dateBy: "", numberStep: 0, hideEmpty: false, hidden: [], collapsed: [] } : null;
    api
      .queryView({ databaseId: db.id, viewId: view.id, view: { ...view, sorts: [], calcs: [], group: xg, subGroup: sg } })
      .then((r) => {
        if (drillSeq.current !== mine) return;
        const groups = r.groups ?? [];
        const grp = label !== undefined ? groups.find((q) => q.label === label) ?? groups[pick.li ?? -1] : undefined;
        let ids: string[];
        if (series !== undefined && sg) {
          const pool = grp ? [grp] : groups;
          ids = [...new Set(pool.flatMap((q) => q.subs.filter((sub) => sub.label === series).flatMap((sub) => sub.rows)))];
        } else if (label !== undefined) ids = grp?.rows ?? [];
        else ids = r.rows.map((row) => row.id);
        const byId = new Map(r.rows.map((row) => [row.id, row]));
        setDrill((d) => (d ? { ...d, rows: ids.map((id) => byId.get(id)).filter((row): row is NonNullable<typeof row> => !!row), loading: false } : d));
      })
      .catch((e) => {
        if (drillSeq.current !== mine) return;
        setDrill((d) => (d ? { ...d, loading: false, error: typeof e === "string" ? e : e instanceof Error ? e.message : "The rows could not be loaded." } : d));
      });
  };

  return (
    <div className="flex h-full min-h-0 flex-col overflow-y-auto px-4 pb-6 pt-1" role="region" aria-label="Chart">
      <div className="mx-auto flex w-full max-w-[60rem] flex-col gap-3">
        <div className="flex flex-wrap items-center gap-2">
          <h2 className="me-auto min-w-0 truncate text-[1rem] font-semibold" dir="auto">{cfg.title || defaultTitle(db, cfg)}</h2>
          <button type="button" aria-expanded={panel} onClick={() => setPanel((v) => !v)} className={cn("inline-flex h-8 items-center gap-1.5 rounded-lg border border-line px-2.5 text-[0.8rem] hover:bg-surface-2", panel && "bg-accent-soft text-accent")}>
            <Settings2 size={14} />
            Customize
          </button>
          {data && !error && data.series.length && cfg.kind !== "number" ? (
            <button type="button" aria-pressed={table} onClick={() => setTable((v) => !v)} className={cn("inline-flex h-8 items-center gap-1.5 rounded-lg border border-line px-2.5 text-[0.8rem] hover:bg-surface-2", table && "bg-accent-soft text-accent")}>
              <Table2 size={14} />
              {table ? "Show chart" : "Show as table"}
            </button>
          ) : null}
        </div>

        {panel ? <Config db={db} cfg={cfg} patch={patch} /> : null}

        <div className="min-h-[18rem] rounded-xl border border-line bg-surface/40 p-3" aria-busy={loading}>
          {error ? (
            <EmptyState title="The chart could not be drawn">
              <span role="alert">{error}</span>
              <button type="button" onClick={() => setRetry((n) => n + 1)} className="inline-flex h-8 items-center gap-1.5 rounded-lg border border-line px-3 text-[0.8rem] hover:bg-surface-2">
                <RefreshCw size={13} />
                Try again
              </button>
            </EmptyState>
          ) : !data ? (
            <div className="grid min-h-[16rem] place-items-center text-sm text-muted">Calculating…</div>
          ) : needsX && !propOf(db, cfg.xProp) ? (
            <EmptyState icon={<BarChart3 size={28} />} title="Choose what to chart">
              <span>Pick a property for the horizontal axis in Customize.</span>
              {!panel ? <button type="button" onClick={() => setPanel(true)} className="h-8 rounded-lg bg-accent px-3 text-[0.8rem] text-accent-fg">Customize</button> : null}
            </EmptyState>
          ) : (
            <Chart data={data} cfg={cfg} db={db} table={table} dim={loading} open={openDrill} />
          )}
        </div>
      </div>
      {drill ? <ChartDrill drill={drill} onClose={() => (drillSeq.current++, setDrill(null))} onOpenRow={(id) => (drillSeq.current++, setDrill(null), openRow(id))} /> : null}
    </div>
  );
}

function defaultTitle(db: Database, cfg: ChartConfig): string {
  const x = propOf(db, cfg.xProp);
  const y = propOf(db, cfg.yProp);
  const calc = CALCS.find((c) => c.id === cfg.yCalc)?.name ?? "Count of rows";
  if (cfg.kind === "number") return y && cfg.yCalc && cfg.yCalc !== "countAll" ? `${calc} of ${y.name}` : "Rows";
  return x ? `${y && cfg.yCalc !== "countAll" && cfg.yCalc ? `${calc} of ${y.name}` : "Rows"} by ${x.name}` : "Chart";
}

/* ---------- Settings ---------- */

function Config({ db, cfg, patch }: { db: Database; cfg: ChartConfig; patch: (p: Partial<ChartConfig>) => void }) {
  const xProps = db.props.filter((p) => X_KINDS.includes(p.kind));
  const gProps = db.props.filter((p) => ["select", "status", "multiSelect", "person", "checkbox"].includes(p.kind));
  const calc = CALCS.find((c) => c.id === (cfg.yCalc || "countAll")) ?? CALCS[0];
  const yProps = db.props.filter((p) => p.kind !== "title" && (!calc.numeric || NUMERIC_KINDS.includes(p.kind)));
  const x = propOf(db, cfg.xProp);
  const id = useId();
  const field = (label: string, node: ReactNode) => (
    <div className="flex min-w-0 flex-col gap-1">
      <span className="text-[0.72rem] text-muted">{label}</span>
      {node}
    </div>
  );
  const sel = "h-8 w-full rounded-lg border border-line bg-surface px-2 text-[0.8rem] text-fg outline-none focus:border-accent";
  const toggle = (label: string, on: boolean, set: (v: boolean) => void, disabled?: boolean) => (
    <label className={cn("flex items-center gap-2 text-[0.8rem]", disabled && "opacity-45")}>
      <input type="checkbox" checked={on} disabled={disabled} onChange={(e) => set(e.target.checked)} className="h-4 w-4 accent-[var(--accent)]" />
      {label}
    </label>
  );
  return (
    <div className="grid gap-3 rounded-xl border border-line p-3 sm:grid-cols-2 lg:grid-cols-4" data-chart-config>
      {field(
        "Chart",
        <div className="flex rounded-lg border border-line p-0.5" role="group" aria-label="Chart type">
          {KINDS.map((k) => (
            <button key={k.id} type="button" aria-pressed={cfg.kind === k.id} aria-label={k.name} title={k.name} onClick={() => patch({ kind: k.id })} className={cn("grid h-7 flex-1 place-items-center rounded-md", cfg.kind === k.id ? "bg-accent-soft text-accent" : "text-muted hover:text-fg")}>
              <k.icon size={15} />
            </button>
          ))}
        </div>,
      )}
      {cfg.kind !== "number"
        ? field("X axis", <PropSelect label="X axis" placeholder="Choose a property" props={xProps} value={cfg.xProp} onChange={(v) => patch({ xProp: v, xBy: "", xStep: 0 })} className="max-w-none" />)
        : null}
      {cfg.kind !== "number" && x?.kind === "date" ? (
        field(
          "Group dates by",
          <select aria-label="Group dates by" className={sel} value={cfg.xBy || "day"} onChange={(e) => patch({ xBy: e.target.value })}>
            {["day", "week", "month", "year"].map((b) => <option key={b} value={b}>{b[0].toUpperCase() + b.slice(1)}</option>)}
          </select>,
        )
      ) : null}
      {cfg.kind !== "number" && x?.kind === "number"
        ? field("Bucket size", <input aria-label="Bucket size" type="number" min={0} className={sel} value={cfg.xStep || 0} onChange={(e) => patch({ xStep: Math.max(0, Number(e.target.value) || 0) })} />)
        : null}
      {cfg.kind !== "number"
        ? field(
            "Order",
            <select aria-label="Order" className={sel} value={cfg.xSort || "manual"} onChange={(e) => patch({ xSort: e.target.value })}>
              <option value="manual">Default</option>
              <option value="asc">Ascending</option>
              <option value="desc">Descending</option>
            </select>,
          )
        : null}
      {cfg.kind !== "number" && cfg.kind !== "donut"
        ? field("Split into series by", <PropSelect label="Group by" placeholder="None" allowNone="None" props={gProps.filter((p) => p.id !== cfg.xProp)} value={cfg.groupProp} onChange={(v) => patch({ groupProp: v, stacked: v ? cfg.stacked : false })} className="max-w-none" />)
        : null}
      {field(
        "Value",
        <select aria-label="Value" className={sel} value={calc.id} onChange={(e) => patch({ yCalc: e.target.value, yProp: e.target.value === "countAll" ? "" : cfg.yProp })}>
          {CALCS.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
        </select>,
      )}
      {calc.needsProp ? field("Of property", <PropSelect label="Of property" placeholder="Choose a property" props={yProps} value={cfg.yProp} onChange={(v) => patch({ yProp: v })} className="max-w-none" />) : null}
      {field(
        "Color",
        <div className="flex flex-wrap items-center gap-1.5" role="group" aria-label="Color">
          <button type="button" aria-pressed={!cfg.color} onClick={() => patch({ color: "" })} className={cn("h-6 rounded-md border px-1.5 text-[0.72rem]", !cfg.color ? "border-accent text-accent" : "border-line text-muted")}>Auto</button>
          {PALETTE.map((c) => (
            <button key={c} type="button" aria-label={c} aria-pressed={cfg.color === c} onClick={() => patch({ color: c })} className={cn("h-5 w-5 rounded-full ring-offset-1 ring-offset-[var(--bg)]", cfg.color === c && "ring-2 ring-[var(--text)]")} style={{ background: HUES[c] }} />
          ))}
        </div>,
      )}
      {field(
        "Title",
        <input id={id} aria-label="Chart title" dir="auto" placeholder="Automatic" className={sel} value={cfg.title} onChange={(e) => patch({ title: e.target.value })} />,
      )}
      <div className="flex flex-wrap items-end gap-x-4 gap-y-2 sm:col-span-2 lg:col-span-4">
        {cfg.kind === "bar" ? toggle("Horizontal", cfg.horizontal, (v) => patch({ horizontal: v })) : null}
        {cfg.kind === "bar" ? toggle("Stacked", cfg.stacked, (v) => patch({ stacked: v }), !cfg.groupProp) : null}
        {cfg.kind !== "number" ? toggle("Show values", cfg.showLabels, (v) => patch({ showLabels: v })) : null}
        {cfg.kind !== "number" ? toggle("Hide empty groups", cfg.hideEmpty, (v) => patch({ hideEmpty: v })) : null}
      </div>
    </div>
  );
}

/* ---------- Drawing ---------- */

interface Tip {
  x: number;
  y: number;
  title: string;
  rows: { color: string; name: string; text: string }[];
}

const HINT = "Click to view data";
/** Calculations whose result is not a share of a whole, so a percent next to them would mean nothing. */
const NO_PERCENT = new Set(["avg", "median", "min", "max", "range", "percentNotEmpty"]);

function useWidth<T extends HTMLElement>(): [React.RefObject<T | null>, number] {
  const ref = useRef<T>(null);
  const [w, setW] = useState(640);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const ro = new ResizeObserver(() => setW(Math.max(240, Math.floor(el.clientWidth))));
    ro.observe(el);
    setW(Math.max(240, Math.floor(el.clientWidth)));
    return () => ro.disconnect();
  }, []);
  return [ref, w];
}

const compact = (n: number) => {
  const magnitude = Math.abs(n);
  if (magnitude >= 1e21 || (magnitude > 0 && magnitude < 0.01)) return n.toExponential(1);
  return new Intl.NumberFormat(undefined, magnitude >= 10000
    ? { notation: "compact", maximumFractionDigits: 1 }
    : { maximumFractionDigits: 2 }).format(n);
};
const cut = (s: string, n: number) => (s.length > n ? s.slice(0, Math.max(1, n - 1)) + "…" : s);

function Chart({ data, cfg, db, table, dim, open }: { data: ChartData; cfg: ChartConfig; db: Database; table: boolean; dim: boolean; open: (p: ChartPick) => void }) {
  const [box, width] = useWidth<HTMLDivElement>();
  const [tip, setTip] = useState<Tip | null>(null);
  const y = propOf(db, cfg.yProp);
  const summary = useMemo(() => summarize(data, cfg), [data, cfg]);
  const empty = cfg.kind === "number" ? false : !data.series.length || !data.labels.length || data.series.every((s) => s.points.every((p) => !p.value));

  const show = (e: React.MouseEvent, title: string, rows: Tip["rows"]) => {
    const r = box.current?.getBoundingClientRect();
    if (!r) return;
    setTip({ x: e.clientX - r.left, y: e.clientY - r.top, title, rows });
  };
  const hide = () => setTip(null);

  let body: ReactNode;
  if (cfg.kind === "number") {
    body = <NumberTile data={data} cfg={cfg} y={y} open={() => open({})} />;
  } else if (empty) {
    body = (
      <EmptyState icon={<BarChart3 size={28} />} title="Nothing to chart yet">
        <span>{data.rows ? "The values here are all zero." : "No rows match this view."}</span>
      </EmptyState>
    );
  } else if (table) {
    body = <DataTable data={data} />;
  } else if (cfg.kind === "donut") {
    body = <Donut data={data} cfg={cfg} width={width} summary={summary} show={show} hide={hide} open={open} />;
  } else if (cfg.kind === "line") {
    body = <Lines data={data} cfg={cfg} width={width} summary={summary} show={show} hide={hide} open={open} />;
  } else {
    body = <Bars data={data} cfg={cfg} width={width} summary={summary} show={show} hide={hide} open={open} />;
  }

  return (
    <div ref={box} className={cn("relative transition-opacity", dim && "opacity-60")}>
      {body}
      {tip ? (
        <div className="pointer-events-none absolute z-20 max-w-[16rem] rounded-lg border border-line bg-elevated px-2.5 py-1.5 text-[0.78rem] shadow-float" style={{ left: Math.min(tip.x + 12, Math.max(0, width - 200)), top: Math.max(0, tip.y - 12) }} role="tooltip" data-tooltip>
          <div dir="auto" className="mb-0.5 font-medium">{tip.title || "No value"}</div>
          {tip.rows.map((r, i) => (
            <div key={i} className="flex items-center gap-1.5 text-muted">
              <span className="h-2 w-2 shrink-0 rounded-full" style={{ background: r.color }} />
              <span dir="auto" className="min-w-0 truncate">{r.name}</span>
              <span className="ms-auto ps-2 tabular-nums text-fg">{r.text}</span>
            </div>
          ))}
          {!table ? <div className="mt-1 border-t border-line pt-1 text-[0.72rem] text-faint" data-tooltip-hint>{HINT}</div> : null}
        </div>
      ) : null}
    </div>
  );
}

function summarize(data: ChartData, cfg: ChartConfig): string {
  const kind = cfg.kind === "donut" ? "Donut chart" : cfg.kind === "line" ? "Line chart" : `${cfg.horizontal ? "Horizontal bar" : "Bar"} chart`;
  const first = data.series[0];
  const items = data.labels.slice(0, 6).map((l, i) => `${l || "No value"}: ${first?.points[i]?.text ?? ""}`);
  const more = data.labels.length > 6 ? ` and ${data.labels.length - 6} more` : "";
  return `${cfg.title ? cfg.title + ". " : ""}${kind} of ${data.rows} rows in ${data.labels.length} groups${data.series.length > 1 ? ` and ${data.series.length} series` : ""}. ${items.join("; ")}${more}.`;
}

interface Draw {
  data: ChartData;
  cfg: ChartConfig;
  width: number;
  summary: string;
  show: (e: React.MouseEvent, title: string, rows: Tip["rows"]) => void;
  hide: () => void;
  /** A bar, slice or point was clicked: show the rows behind it. */
  open: (p: ChartPick) => void;
}

const clickKeys = (fn: () => void) => (e: React.KeyboardEvent) => {
  if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    fn();
  }
};

const seriesColor = (s: ChartSeries, i: number, cfg: ChartConfig) => paint(s.color || cfg.color, i);

function tipRows(data: ChartData, cfg: ChartConfig, li: number, only?: number): Tip["rows"] {
  const pct = !NO_PERCENT.has(cfg.yCalc || "countAll");
  return data.series
    .map((s, si) => ({ s, si }))
    .filter(({ si }) => only === undefined || si === only)
    .map(({ s, si }) => {
      const total = s.points.reduce((a, p) => a + Math.max(0, p.value), 0);
      const v = s.points[li]?.value ?? 0;
      const share = pct && total > 0 && v >= 0 ? ` · ${Math.round((v / total) * 1000) / 10}%` : "";
      return { color: s.points[li]?.color ? paint(s.points[li].color, li) : seriesColor(s, si, cfg), name: s.name, text: `${s.points[li]?.text ?? ""}${share}` };
    });
}

function Legend({ data, cfg }: { data: ChartData; cfg: ChartConfig }) {
  if (data.series.length < 2) return null;
  return (
    <ul className="mt-2 flex flex-wrap justify-center gap-x-4 gap-y-1 text-[0.78rem] text-muted" aria-label="Legend">
      {data.series.map((s, i) => (
        <li key={i} className="flex items-center gap-1.5">
          <span className="h-2.5 w-2.5 rounded-sm" style={{ background: seriesColor(s, i, cfg) }} />
          <span dir="auto">{s.name || "No value"}</span>
        </li>
      ))}
    </ul>
  );
}

function Bars({ data, cfg, width, summary, show, hide, open }: Draw) {
  const horizontal = cfg.horizontal;
  const n = data.labels.length;
  const k = data.series.length;
  const hasNeg = data.series.some((s) => s.points.some((p) => p.value < 0));
  const maxLabel = Math.max(...data.labels.map((l) => (l || "No value").length), 1);
  const stackSums = cfg.stacked && k > 1 && !hasNeg
    ? data.labels.map((_, i) => data.series.reduce((a, s) => a + (s.points[i]?.value ?? 0), 0))
    : [];
  // If a stack has no finite endpoint, every point can still be drawn as a separate bar.
  const stacked = stackSums.length > 0 && stackSums.every(Number.isFinite);
  const all = data.series.flatMap((s) => s.points.map((p) => p.value));
  const hi = (stacked ? stackSums : all).reduce((a, b) => Math.max(a, b), 0);
  const lo = stacked ? 0 : all.reduce((a, b) => Math.min(a, b), 0);
  const ticks = niceTicks(lo, hi, 5, all.every(Number.isInteger) && (!stacked || stackSums.every(Number.isInteger)));
  const vmin = ticks[0];
  const vmax = ticks[ticks.length - 1];
  const tickW = Math.max(...ticks.map((t) => compact(t).length)) * 6.5 + 12;

  // Geometry. Vertical: bands along x. Horizontal: bands along y.
  const rowH = horizontal ? Math.max(26, Math.min(48, (k > 1 && !stacked ? k * 16 : 28) + 10)) : 0;
  const labW = horizontal ? Math.min(180, maxLabel * 6.8 + 12) : 0;
  const m = horizontal ? { l: labW + 8, r: 28, t: 8, b: 28 } : { l: tickW + 6, r: 12, t: 16, b: 0 };
  const innerW = width - m.l - m.r;
  const band = horizontal ? rowH : innerW / Math.max(n, 1);
  const rotate = !horizontal && band < Math.min(90, maxLabel * 6.5 + 8);
  const bottom = horizontal ? 28 : rotate ? Math.min(90, maxLabel * 5 + 20) : 30;
  const height = horizontal ? m.t + n * rowH + bottom : 320;
  const innerH = horizontal ? n * rowH : height - m.t - bottom;
  const vx = (v: number) => m.l + axisFraction(v, vmin, vmax) * innerW;
  const vy = (v: number) => m.t + innerH - axisFraction(v, vmin, vmax) * innerH;
  const groupW = band * 0.72;
  const barW = stacked ? Math.min(groupW, 56) : Math.min(groupW / k, 44);
  const bars: ReactNode[] = [];

  data.labels.forEach((label, li) => {
    const c0 = (horizontal ? m.t : m.l) + band * li + band / 2;
    let acc = 0;
    data.series.forEach((s, si) => {
      const p = s.points[li];
      if (!p) return;
      const v = p.value;
      const color = k === 1 && p.color ? paint(p.color, li) : seriesColor(s, si, cfg);
      const off = stacked ? 0 : (si - (k - 1) / 2) * barW;
      const base = stacked ? acc : 0;
      const top = stacked ? acc + v : v;
      if (stacked) acc = top;
      const a = Math.min(vy(base), vy(top));
      const hgt = Math.max(Math.abs(vy(top) - vy(base)), v === 0 ? 0 : 1.5);
      const xA = Math.min(vx(base), vx(top));
      const wid = Math.max(Math.abs(vx(top) - vx(base)), v === 0 ? 0 : 1.5);
      const labelOk = cfg.showLabels && (horizontal ? true : band / (stacked ? 1 : k) >= 30);
      bars.push(
        <g
          key={`${li}-${si}`}
          role="button"
          tabIndex={0}
          aria-label={`${label || "No value"}: ${p.text}. Click to view data`}
          onMouseMove={(e) => show(e, label, tipRows(data, cfg, li, stacked || k === 1 ? undefined : si))}
          onMouseLeave={hide}
          onFocus={() => hide()}
          onClick={() => (hide(), open({ li, si: stacked || k === 1 ? undefined : si }))}
          onKeyDown={clickKeys(() => open({ li, si: stacked || k === 1 ? undefined : si }))}
          className="cursor-pointer outline-none focus-visible:[&>rect]:stroke-[var(--text)] focus-visible:[&>rect]:stroke-2"
          data-bar-point
        >
          <rect
            data-bar
            x={horizontal ? xA : c0 + off - barW / 2}
            y={horizontal ? c0 + off - barW / 2 : a}
            width={horizontal ? wid : barW}
            height={horizontal ? barW : hgt}
            rx={3}
            fill={color}
            className="transition-opacity hover:opacity-80"
          />
          {labelOk && v !== 0 && !(stacked && (horizontal ? wid : hgt) < 16) ? (
            stacked ? (
              <text x={horizontal ? xA + wid / 2 : c0 + off} y={horizontal ? c0 + off + 4 : a + hgt / 2 + 4} textAnchor="middle" fontSize="11" fill="#fff" className="pointer-events-none" style={{ paintOrder: "stroke", stroke: "rgba(0,0,0,.25)", strokeWidth: 2 }}>
                {p.text}
              </text>
            ) : horizontal ? (
              <text x={vx(Math.max(v, 0)) + 5} y={c0 + off + 4} fontSize="11" fill="currentColor" className="pointer-events-none text-muted">{p.text}</text>
            ) : (
              <text x={c0 + off} y={(v >= 0 ? a : a + hgt) + (v >= 0 ? -5 : 13)} textAnchor="middle" fontSize="11" fill="currentColor" className="pointer-events-none text-muted">{p.text}</text>
            )
          ) : null}
        </g>,
      );
    });
  });

  return (
    <div>
      <svg style={{ direction: "ltr" }} width={width} height={height} role="img" aria-label={summary} data-chart="bar" data-bars={n * k}>
        <title>{summary}</title>
        {ticks.map((t) => (
          <g key={t} className="text-muted">
            {horizontal ? (
              <>
                <line x1={vx(t)} x2={vx(t)} y1={m.t} y2={m.t + innerH} stroke="var(--border)" strokeWidth={t === 0 ? 1.5 : 1} />
                <text x={vx(t)} y={m.t + innerH + 16} textAnchor="middle" fontSize="11" fill="currentColor">{compact(t)}</text>
              </>
            ) : (
              <>
                <line x1={m.l} x2={width - m.r} y1={vy(t)} y2={vy(t)} stroke="var(--border)" strokeWidth={t === 0 ? 1.5 : 1} />
                <text x={m.l - 8} y={vy(t) + 4} textAnchor="end" fontSize="11" fill="currentColor">{compact(t)}</text>
              </>
            )}
          </g>
        ))}
        {data.labels.map((l, i) => {
          const c = (horizontal ? m.t : m.l) + band * i + band / 2;
          const text = cut(l || "No value", horizontal ? Math.floor(labW / 6.5) : rotate ? 14 : Math.max(3, Math.floor(band / 6.5)));
          return horizontal ? (
            <text key={i} x={m.l - 8} y={c + 4} textAnchor="end" fontSize="11.5" fill="currentColor"><title>{l}</title>{text}</text>
          ) : rotate ? (
            <text key={i} transform={`translate(${c + 4},${m.t + innerH + 14}) rotate(-35)`} textAnchor="end" fontSize="11.5" fill="currentColor"><title>{l}</title>{text}</text>
          ) : (
            <text key={i} x={c} y={m.t + innerH + 18} textAnchor="middle" fontSize="11.5" fill="currentColor"><title>{l}</title>{text}</text>
          );
        })}
        {bars}
      </svg>
      <Legend data={data} cfg={cfg} />
    </div>
  );
}

function Lines({ data, cfg, width, summary, show, hide, open }: Draw) {
  const n = data.labels.length;
  const all = data.series.flatMap((s) => s.points.map((p) => p.value));
  const lo = all.reduce((a, b) => Math.min(a, b), 0);
  const hi = all.reduce((a, b) => Math.max(a, b), 0);
  const ticks = niceTicks(lo, hi, 5, all.every(Number.isInteger));
  const vmin = ticks[0];
  const vmax = ticks[ticks.length - 1];
  const tickW = Math.max(...ticks.map((t) => compact(t).length)) * 6.5 + 12;
  const maxLabel = Math.max(...data.labels.map((l) => (l || "No value").length), 1);
  const m = { l: tickW + 6, r: 34, t: 16 };
  const innerW = width - m.l - m.r;
  const step = n > 1 ? innerW / (n - 1) : 0;
  const rotate = n > 1 && step < Math.min(90, maxLabel * 6.5 + 8);
  const bottom = rotate ? Math.min(90, maxLabel * 5 + 20) : 30;
  const height = 320;
  const innerH = height - m.t - bottom;
  const px = (i: number) => (n > 1 ? m.l + step * i : m.l + innerW / 2);
  const py = (v: number) => m.t + innerH - axisFraction(v, vmin, vmax) * innerH;
  const every = Math.max(1, Math.ceil(n / Math.max(2, Math.floor(innerW / (rotate ? 22 : Math.min(110, maxLabel * 6.5 + 14))))));
  return (
    <div>
      <svg style={{ direction: "ltr" }} width={width} height={height} role="img" aria-label={summary} data-chart="line" data-points={n * data.series.length}>
        <title>{summary}</title>
        {ticks.map((t) => (
          <g key={t} className="text-muted">
            <line x1={m.l} x2={width - m.r} y1={py(t)} y2={py(t)} stroke="var(--border)" strokeWidth={t === 0 ? 1.5 : 1} />
            <text x={m.l - 8} y={py(t) + 4} textAnchor="end" fontSize="11" fill="currentColor">{compact(t)}</text>
          </g>
        ))}
        {data.labels.map((l, i) =>
          i % every ? null : rotate ? (
            <text key={i} transform={`translate(${px(i) + 4},${m.t + innerH + 14}) rotate(-35)`} textAnchor="end" fontSize="11.5" fill="currentColor" className="text-muted">{cut(l || "No value", 14)}</text>
          ) : (
            <text key={i} x={px(i)} y={m.t + innerH + 18} textAnchor="middle" fontSize="11.5" fill="currentColor" className="text-muted">{cut(l || "No value", Math.max(3, Math.floor(Math.max(step * every, 60) / 6.5)))}</text>
          ),
        )}
        {data.series.map((s, si) => {
          const color = seriesColor(s, si, cfg);
          const pts = s.points.map((p, i) => [px(i), py(p.value)] as const);
          return (
            <g key={si} data-series={si}>
              <polyline fill="none" stroke={color} strokeWidth={2.2} strokeLinejoin="round" strokeLinecap="round" points={pts.map((p) => p.join(",")).join(" ")} />
              {s.points.map((p, i) => (
                <g key={i}>
                  <circle data-point cx={pts[i][0]} cy={pts[i][1]} r={3.6} fill="var(--bg)" stroke={color} strokeWidth={2} />
                  <circle
                    data-point-hit
                    cx={pts[i][0]}
                    cy={pts[i][1]}
                    r={12}
                    fill="transparent"
                    role="button"
                    tabIndex={0}
                    aria-label={`${data.labels[i] || "No value"}: ${p.text}. Click to view data`}
                    className="cursor-pointer outline-none"
                    onMouseMove={(e) => show(e, data.labels[i], tipRows(data, cfg, i))}
                    onMouseLeave={hide}
                    onClick={() => (hide(), open({ li: i, si: data.series.length > 1 ? si : undefined }))}
                    onKeyDown={clickKeys(() => open({ li: i, si: data.series.length > 1 ? si : undefined }))}
                  />
                  {cfg.showLabels && n <= 14 && data.series.length <= 2 ? (
                    <text x={pts[i][0]} y={pts[i][1] - 9} textAnchor="middle" fontSize="11" fill="currentColor" className="pointer-events-none text-muted">{p.text}</text>
                  ) : null}
                </g>
              ))}
            </g>
          );
        })}
      </svg>
      <Legend data={data} cfg={cfg} />
    </div>
  );
}

function Donut({ data, cfg, width, summary, show, hide, open }: Draw) {
  const slices = useMemo(() => {
    if (data.series.length > 1) {
      return data.series.map((s, i) => ({
        label: s.name,
        value: s.donutSlice?.value ?? 0,
        text: s.donutSlice?.text ?? "0",
        color: seriesColor(s, i, cfg),
      }));
    }
    const s = data.series[0];
    return (s?.points ?? []).map((p, i) => ({ label: p.label, value: Math.max(0, p.value), text: p.text, color: paint(p.color, i) }));
  }, [data, cfg]);
  const largest = slices.reduce((max, s) => Math.max(max, s.value), 0) || 1;
  const sum = slices.reduce((a, s) => a + s.value / largest, 0) || 1;
  const fraction = (value: number) => (value / largest) / sum;
  // One series: a slice is a label. Several series: a slice is a series (all its labels).
  const pickOfSlice = (i: number): ChartPick => (data.series.length > 1 ? { si: i } : { li: i });
  const wide = width >= 520;
  const size = wide ? 260 : Math.min(260, width - 16);
  const sw = 38;
  const r = (size - sw) / 2 - 2;
  const circ = 2 * Math.PI * r;
  let acc = 0;
  return (
    <div className={cn("flex items-center justify-center gap-6", wide ? "flex-row" : "flex-col")}>
      <svg style={{ direction: "ltr" }} width={size} height={size} role="img" aria-label={summary} data-chart="donut" data-slices={slices.filter((s) => s.value > 0).length} className="shrink-0">
        <title>{summary}</title>
        <g transform={`rotate(-90 ${size / 2} ${size / 2})`}>
          {slices.map((s, i) => {
            const len = fraction(s.value) * circ;
            const el = (
              <circle
                key={i}
                data-slice
                cx={size / 2}
                cy={size / 2}
                r={r}
                fill="none"
                stroke={s.color}
                strokeWidth={sw}
                strokeDasharray={`${Math.max(len - (slices.length > 1 && len > 2 ? 1.5 : 0), 0)} ${circ}`}
                strokeDashoffset={-acc}
                className="cursor-pointer outline-none transition-opacity hover:opacity-80"
                role="button"
                tabIndex={0}
                aria-label={`${s.label || "No value"}: ${s.text}. Click to view data`}
                onMouseMove={(e) => show(e, s.label, [{ color: s.color, name: `${Math.round(fraction(s.value) * 1000) / 10}%`, text: s.text }])}
                onMouseLeave={hide}
                onClick={() => (hide(), open(pickOfSlice(i)))}
                onKeyDown={clickKeys(() => open(pickOfSlice(i)))}
              />
            );
            acc += len;
            return el;
          })}
        </g>
        <text x={size / 2} y={size / 2 + 2} textAnchor="middle" fontSize="30" fontWeight="600" fill="currentColor" data-donut-total>{data.total ?? ""}</text>
        <text x={size / 2} y={size / 2 + 22} textAnchor="middle" fontSize="11.5" fill="currentColor" className="text-muted">{data.rows} {data.rows === 1 ? "row" : "rows"}</text>
      </svg>
      <ul className="flex min-w-0 max-w-full flex-col gap-1 text-[0.82rem]" aria-label="Legend">
        {slices.map((s, i) => (
          <li key={i} data-legend>
            <button type="button" onClick={() => open(pickOfSlice(i))} className="flex w-full items-center gap-2 rounded-md px-1.5 py-0.5 text-start hover:bg-surface-2" title={HINT}>
              <span className="h-2.5 w-2.5 shrink-0 rounded-sm" style={{ background: s.color }} />
              <span dir="auto" className="min-w-0 truncate">{s.label || "No value"}</span>
              <span className="ms-auto ps-3 tabular-nums text-muted">{s.text}</span>
              <span className="w-11 text-end tabular-nums text-faint">{Math.round(fraction(s.value) * 100)}%</span>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}

function NumberTile({ data, cfg, y, open }: { data: ChartData; cfg: ChartConfig; y: Property | undefined; open: () => void }) {
  const text = data.total ?? data.series[0]?.points[0]?.text ?? "0";
  const calc = CALCS.find((c) => c.id === (cfg.yCalc || "countAll"))?.name ?? "Count of rows";
  const color = cfg.color ? paint(cfg.color) : "var(--text)";
  return (
    <button type="button" onClick={open} title={HINT} className="flex min-h-[16rem] w-full cursor-pointer flex-col items-center justify-center gap-1 rounded-lg text-center hover:bg-surface-2/60" aria-label={`${cfg.title || calc}: ${text}. Click to view data`} data-chart="number">
      <div className="text-[4rem] font-semibold leading-none tabular-nums" style={{ color }} data-number>{text}</div>
      <div dir="auto" className="mt-2 text-[0.9rem] text-muted">{cfg.title || (y && cfg.yCalc !== "countAll" ? `${calc} of ${y.name}` : calc)}</div>
      <div className="text-xs text-faint">{data.rows} {data.rows === 1 ? "row" : "rows"} · {HINT}</div>
    </button>
  );
}

function DataTable({ data }: { data: ChartData }) {
  return (
    <div className="overflow-x-auto">
      <table className="w-full border-collapse text-[0.82rem]" data-chart-table>
        <thead>
          <tr className="text-start text-muted">
            <th className="border-b border-line px-2 py-1.5 text-start font-medium">Group</th>
            {data.series.map((s, i) => <th key={i} className="border-b border-line px-2 py-1.5 text-end font-medium"><span dir="auto">{s.name || "Value"}</span></th>)}
          </tr>
        </thead>
        <tbody>
          {data.labels.map((l, i) => (
            <tr key={i}>
              <td className="border-b border-line/60 px-2 py-1.5"><span dir="auto">{l || "No value"}</span></td>
              {data.series.map((s, si) => <td key={si} className="border-b border-line/60 px-2 py-1.5 text-end tabular-nums">{s.points[i]?.text ?? ""}</td>)}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

