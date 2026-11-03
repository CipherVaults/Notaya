// Dev-only harness: renders the database views (src/features/database/views/*) with hand-made data in a plain
// browser, with a tiny stand-in for the shell (grouping, setCells, createRow...). Everything the views ask the
// shell to do is logged to window.__h.calls so scripted checks can assert on it.
//   npx vite --config vite.views.config.ts      (serves http://127.0.0.1:1423/dev-mock/database-views-harness.html)
//   ?view=board|calendar|timeline|list|gallery|chart  &theme=sunrise|sunrise-dark  &rtl=1  &data=empty|many  &group=none
import React, { useEffect, useMemo, useReducer, useState } from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/inter";
import "../src/styles/app.css";
import type { ChartData, Column, Database, DbRow, GroupBy, GroupOut, Property, View, ViewKind, ViewResult } from "@/lib/database-types";
import type { ViewProps } from "@/features/database/viewTypes";

const q = new URLSearchParams(location.search);
const kind = (q.get("view") ?? "board") as ViewKind;
const theme = q.get("theme") ?? "sunrise";
const rtl = q.has("rtl");
document.documentElement.dataset.theme = theme;
document.documentElement.dataset.mode = theme.includes("dark") ? "dark" : "light";
document.documentElement.style.setProperty("--accent", theme.includes("dark") ? "#ff7a5c" : "#e8553f");
document.documentElement.style.setProperty("--accent-fg", "#ffffff");
if (rtl) {
  document.documentElement.dir = "rtl";
  document.documentElement.lang = "ar";
}

/* ---------- Data ---------- */

const today = (() => {
  const n = new Date();
  return Math.floor(Date.UTC(n.getFullYear(), n.getMonth(), n.getDate()) / 86400000);
})();
const iso = (off: number, time?: string) => {
  const d = new Date((today + off) * 86400000);
  const s = `${d.getUTCFullYear()}-${String(d.getUTCMonth() + 1).padStart(2, "0")}-${String(d.getUTCDate()).padStart(2, "0")}`;
  return time ? `${s}T${time}` : s;
};
const range = (a: number, b: number) => ({ start: iso(a), end: iso(b) });

const emptyProp = (id: string, name: string, kind: Property["kind"], extra: Partial<Property> = {}): Property => ({
  id, name, kind, options: [], numberFormat: "number", numberDisplay: "", numberMax: 100, dateFormat: "full", formula: "",
  relationDb: "", relationBack: "", relationLimitOne: false, rollupRelation: "", rollupProp: "", rollupCalc: "", idPrefix: "", description: "", ...extra,
});
const opt = (id: string, name: string, color: string, group: "" | "todo" | "inProgress" | "complete" = "") => ({ id, name, color: color as never, group });

const props: Property[] = [
  emptyProp("title", "Name", "title"),
  emptyProp("status", "Status", "status", {
    options: [opt("s1", "Not started", "gray", "todo"), opt("s2", "In progress", "blue", "inProgress"), opt("s3", "Done", "green", "complete")],
  }),
  emptyProp("prio", "Priority", "select", { options: [opt("hi", "High", "red"), opt("md", "Medium", "orange"), opt("lo", "Low", "gray")] }),
  emptyProp("tags", "Tags", "multiSelect", {
    options: [opt("t1", "design", "purple"), opt("t2", "bug", "red"), opt("t3", "docs", "yellow"), opt("t4", "meeting", "blue")],
  }),
  emptyProp("due", "When", "date"),
  emptyProp("end", "Deadline", "date"),
  emptyProp("owner", "Owner", "person"),
  emptyProp("done", "Done", "checkbox"),
  emptyProp("est", "Estimate", "number"),
  emptyProp("cover", "Cover", "files"),
  emptyProp("blocked", "Blocked by", "relation"),
  emptyProp("parent", "Parent item", "relation"),
];

const file = (n: number) => [{ id: n.toString(16).padStart(32, "a"), name: `pic${n}.png`, mime: "image/png", size: 1000 }];
const mk = (i: number, title: string, values: Record<string, unknown>, icon = ""): DbRow => ({
  id: `r${i}`, title, icon, number: i, created: 1, updated: 1, favorite: false, values,
});

function sample(): DbRow[] {
  const rows: DbRow[] = [
    mk(1, "Design the home screen", { status: "s2", prio: "hi", tags: ["t1"], due: range(-2, 3), owner: ["Mohamed"], est: 5, cover: file(1) }, "🎨"),
    mk(2, "اجتماع الفريق الأسبوعي", { status: "s1", prio: "md", tags: ["t4"], due: { start: iso(1, "10:00") }, owner: ["أحمد"], done: false, cover: file(2) }, "📅"),
    mk(3, "Write release notes", { status: "s3", prio: "lo", tags: ["t3"], due: range(-6, -4), done: true }),
    mk(4, "Fix sync bug", { status: "s2", prio: "hi", tags: ["t2"], due: range(1, 8), est: 8, cover: file(3) }, "🐞"),
    mk(5, "تحديث الوثائق", { status: "s1", prio: "lo", tags: ["t3"], due: { start: iso(5) }, owner: ["سارة"] }),
    mk(6, "Plan Q1 roadmap", { status: "s1", prio: "md", due: range(10, 24), est: 13 }),
    mk(7, "Review pull requests", { status: "s2", prio: "md", due: { start: iso(0) }, owner: ["Mohamed"] }),
    mk(8, "Buy domain", { status: "s3", prio: "lo", due: range(-10, -9), done: true }),
    mk(9, "Interview candidates", { status: "s1", prio: "hi", due: range(3, 4), tags: ["t4"] }),
    mk(10, "Backlog grooming", { prio: "lo" }),
    mk(11, "Prepare demo", { status: "s1", prio: "hi", due: range(12, 15), blocked: ["r4"], tags: ["t1", "t4"], cover: file(4) }),
    mk(12, "Demo script", { status: "s1", prio: "md", due: range(12, 13), parent: ["r11"] }),
    mk(13, "Slides", { status: "s1", prio: "md", due: range(13, 15), parent: ["r11"] }),
    mk(14, "ترجمة الواجهة", { status: "s1", prio: "md", tags: ["t1", "t3"] }),
    mk(15, "A very long title that should be cut off at the end of the line because it keeps going and going and going", { status: "s3", prio: "lo", due: { start: iso(-3) }, done: true }),
    mk(16, "Archive old notes", { status: "s3", prio: "lo", due: { start: iso(-1) }, tags: ["t3"] }),
    mk(17, "Customer call", { status: "s1", prio: "hi", due: { start: iso(1, "14:30") }, tags: ["t4"] }),
    mk(18, "Team lunch", { status: "s1", prio: "lo", due: { start: iso(1) } }),
    mk(19, "Update dependencies", { status: "s2", prio: "md", due: { start: iso(1) }, tags: ["t2"] }),
    mk(20, "Security review", { status: "s1", prio: "hi", due: { start: iso(1) } }),
  ];
  if (q.get("data") === "empty") return [];
  if (q.get("data") === "many") {
    for (let i = 21; i < 220; i++) rows.push(mk(i, `Task number ${i}`, { status: ["s1", "s2", "s3"][i % 3], prio: ["hi", "md", "lo"][i % 3], due: range((i % 40) - 10, (i % 40) - 10 + (i % 5)), est: i % 17 }));
  }
  return rows;
}

const columns: Column[] = [...(q.has("check") ? ["done"] : []), "status", "prio", "tags", "due", "owner", "est"].map((prop) => ({ prop, visible: true, width: 0 }));

const group = (prop: string, extra: Partial<GroupBy> = {}): GroupBy => ({ prop, sort: "manual", dateBy: "", numberStep: 0, hideEmpty: false, hidden: [], collapsed: [], ...extra });

const baseView = (id: string, k: ViewKind, patch: Partial<View> = {}): View => ({
  id, name: k, kind: k, filter: { op: "and", conds: [], groups: [] }, sorts: [], group: null, subGroup: null, columns,
  calcs: [], freeze: 0, wrap: false, rowNumbers: false, openIn: "side", cardSize: "medium", cardCover: "", cardFit: false,
  dateProp: "due", endProp: "", calendarMode: "month", timelineZoom: "week", showDeps: true, showSubItems: true,
  chart: { kind: "bar", xProp: "status", xBy: "", xStep: 0, xSort: "manual", groupProp: "", yCalc: "count", yProp: "", stacked: false, horizontal: false, showLabels: true, hideEmpty: false, color: "", title: "" },
  ...patch,
});

const noGroup = q.get("group") === "none";
const views: Record<string, View> = {
  board: baseView("v-board", "board", { group: noGroup ? null : group(q.get("groupby") ?? "status", q.get("dateby") ? { dateBy: q.get("dateby")! } : {}), subGroup: q.get("sub") ? group(q.get("sub")!) : null, cardCover: q.get("cover") ?? "", cardSize: q.get("size") ?? "medium" }),
  calendar: baseView("v-cal", "calendar", { calendarMode: q.get("mode") ?? "month", dateProp: q.get("noprop") ? "" : "due" }),
  timeline: baseView("v-tl", "timeline", { timelineZoom: q.get("zoom") ?? "week", dateProp: q.get("noprop") ? "" : "due", group: q.has("grouped") ? group("status") : null }),
  list: baseView("v-list", "list", { group: q.has("grouped") ? group("status") : null }),
  gallery: baseView("v-gal", "gallery", { cardCover: q.get("cover") ?? "cover", group: q.has("grouped") ? group("status") : null }),
  chart: (() => {
    const v = baseView("v-chart", "chart");
    v.chart = { ...v.chart, kind: q.get("ck") ?? "bar", xProp: q.get("cx") ?? "status", groupProp: q.get("cg") ?? "", horizontal: q.has("horiz"), stacked: q.has("stack"), xBy: q.get("by") ?? "", yCalc: q.get("calc") ?? "countAll", yProp: q.get("cy") ?? "" };
    return v;
  })(),
  table: baseView("v-table", "table"),
};

const database: Database = {
  id: "db1", sectionId: "s1", parent: "", inline: false, title: "Tasks", icon: "", description: "", order: 0, favorite: false, locked: q.has("locked"),
  props, views: Object.values(views), templates: [], defaultTemplate: "", nextId: 100, subParentProp: "parent", subChildProp: "", blockedByProp: "blocked", blockingProp: "",
  trashedAt: null, created: 1, updated: 1,
};

/* ---------- A tiny shell ---------- */

interface Call { fn: string; args: unknown[] }
const calls: Call[] = [];
const log = (fn: string, ...args: unknown[]) => calls.push({ fn, args: JSON.parse(JSON.stringify(args)) });

let rows = sample();
let view = views[kind] ?? views.board;
let nextRow = 1000;
const listeners = new Set<() => void>();
const bump = () => listeners.forEach((l) => l());

function groupsOf(g: GroupBy, ids: string[]): GroupOut[] {
  const p = props.find((x) => x.id === g.prop);
  if (!p) return [];
  const byId = new Map(rows.map((r) => [r.id, r]));
  const out: GroupOut[] = [];
  const mkg = (key: string, label: string, color: string): GroupOut => ({ key, label, color, count: 0, rows: [], subs: [], calcs: {} });
  const find = (key: string, label: string, color = "") => {
    let o = out.find((x) => x.key === key);
    if (!o) out.push((o = mkg(key, label, color)));
    return o;
  };
  if (p.kind === "select" || p.kind === "status" || p.kind === "multiSelect") {
    find("", `No ${p.name}`);
    if (p.kind === "status" && g.dateBy === "group") {
      find("todo", "To-do", "gray"); find("inProgress", "In progress", "blue"); find("complete", "Complete", "green");
    } else p.options.forEach((o) => find(o.id, o.name, o.color));
  }
  if (p.kind === "checkbox") { find("false", "Unchecked"); find("true", "Checked"); }
  for (const id of ids) {
    const r = byId.get(id);
    if (!r) continue;
    const v = r.values[p.id];
    let keys: [string, string, string][] = [];
    if (p.kind === "select") keys = [[typeof v === "string" ? v : "", "", ""]];
    else if (p.kind === "status") {
      const o = p.options.find((x) => x.id === v);
      keys = [[g.dateBy === "group" ? (o?.group ?? "") : (o?.id ?? ""), "", ""]];
    } else if (p.kind === "multiSelect") keys = Array.isArray(v) && v.length ? (v as string[]).map((x) => [x, "", ""]) : [["", "", ""]];
    else if (p.kind === "checkbox") keys = [[v === true ? "true" : "false", "", ""]];
    else if (p.kind === "person") keys = Array.isArray(v) && v.length ? (v as string[]).map((x) => [x.toLowerCase(), x, ""]) : [["", `No ${p.name}`, ""]];
    else keys = [[typeof v === "string" ? v.toLowerCase() : "", typeof v === "string" ? v : `No ${p.name}`, ""]];
    for (const [k, l, c] of keys) {
      const o = find(k, l || `No ${p.name}`, c);
      o.rows.push(id);
      o.count++;
    }
  }
  return out.filter((x) => !g.hidden.includes(x.key) && !(g.hideEmpty && x.count === 0));
}

function run(): ViewResult {
  const ids = rows.map((r) => r.id);
  const groups = view.group ? groupsOf(view.group, ids) : null;
  if (groups && view.subGroup) {
    for (const gr of groups) gr.subs = groupsOf(view.subGroup, gr.rows);
  }
  return { rows, total: rows.length, filtered: rows.length, truncated: false, groups, calcs: {} };
}

function chartData(v: View): ChartData {
  const c = v.chart;
  const p = props.find((x) => x.id === c.xProp);
  const palette = ["blue", "orange", "green", "purple", "pink", "yellow", "red", "brown", "gray"];
  const col = (x: string, i: number) => x || palette[i % palette.length];
  const calc = (rs: DbRow[]) => {
    const vals = c.yProp ? rs.map((r) => Number(r.values[c.yProp]) || 0) : [];
    const n = c.yCalc === "sum" ? vals.reduce((a, b) => a + b, 0) : c.yCalc === "avg" ? (vals.length ? vals.reduce((a, b) => a + b, 0) / vals.length : 0) : rs.length;
    return { value: n, text: String(Math.round(n * 100) / 100) };
  };
  if (c.kind === "number" || !p) {
    const r = calc(rows);
    return { labels: [], series: [{ name: c.title, color: col(c.color, 0), points: [{ label: "", value: r.value, text: r.text, color: col(c.color, 0) }] }], total: r.text, rows: rows.length };
  }
  let labels: string[] = [];
  let colors: string[] = [];
  let keyOf: (r: DbRow) => string = () => "";
  if (p.kind === "select" || p.kind === "status") {
    labels = [...p.options.map((o) => o.name), "No " + p.name];
    colors = [...p.options.map((o) => o.color), ""];
    keyOf = (r) => p.options.find((o) => o.id === r.values[p.id])?.name ?? "No " + p.name;
  } else if (p.kind === "multiSelect") {
    labels = p.options.map((o) => o.name);
    colors = p.options.map((o) => o.color);
    keyOf = (r) => ((r.values[p.id] as string[] | undefined) ?? []).map((id) => p.options.find((o) => o.id === id)?.name).join("|");
  } else if (p.kind === "date") {
    const set = new Set<string>();
    rows.forEach((r) => { const d = r.values[p.id] as { start: string } | undefined; if (d) set.add(c.xBy === "year" ? d.start.slice(0, 4) : d.start.slice(0, 7)); });
    labels = [...set].sort();
    if (c.xSort === "desc") labels.reverse();
    colors = labels.map(() => "");
    keyOf = (r) => { const d = (r.values[p.id] as { start: string } | undefined)?.start; return d ? (c.xBy === "year" ? d.slice(0, 4) : d.slice(0, 7)) : ""; };
  } else {
    labels = ["A", "B"];
    colors = ["", ""];
    keyOf = (r) => (r.number % 2 ? "A" : "B");
  }
  const gp = props.find((x) => x.id === c.groupProp);
  const names = gp ? gp.options.map((o) => o.name) : [c.title || "Count"];
  const sKey = (r: DbRow) => (gp ? gp.options.find((o) => o.id === r.values[gp.id])?.name ?? "" : names[0]);
  const keepIdx = labels.map((l, i) => (c.hideEmpty && !rows.some((r) => keyOf(r).split("|").includes(l)) ? -1 : i)).filter((i) => i >= 0);
  const series = names.map((name, si) => ({
    name,
    color: gp ? col(gp.options[si].color, si) : col(c.color, 0),
    points: keepIdx.map((li) => {
      const label = labels[li];
      const r = calc(rows.filter((x) => keyOf(x).split("|").includes(label) && sKey(x) === name));
      return { label, value: r.value, text: r.text, color: gp ? col(gp.options[si].color, si) : c.kind === "donut" || colors[li] ? col(colors[li], li) : col(c.color, 0) };
    }),
  }));
  return { labels: keepIdx.map((i) => labels[i]), series, total: c.kind === "donut" ? calc(rows).text : null, rows: rows.length };
}

(window as unknown as { __invoke: (cmd: string, args: Record<string, unknown>) => Promise<unknown> }).__invoke = async (cmd, args) => {
  await new Promise((r) => setTimeout(r, 15));
  if (cmd === "chart_data") {
    log("chartData", (args.request as { view?: View }).view?.chart);
    if (q.get("chart") === "error") throw new Error("The chart could not be calculated.");
    return chartData(((args.request as { view?: View }).view ?? view) as View);
  }
  throw new Error(`harness: unknown command ${cmd}`);
};

const H = {
  calls,
  rows: () => rows,
  view: () => view,
  row: (id: string) => rows.find((r) => r.id === id),
  clear: () => { calls.length = 0; },
};
(window as unknown as { __h: typeof H }).__h = H;

function Harness() {
  const [, force] = useReducer((x: number) => x + 1, 0);
  const [ready, setReady] = useState(false);
  useEffect(() => {
    const l = () => force();
    listeners.add(l);
    setReady(true);
    return () => void listeners.delete(l);
  }, []);
  const res = useMemo(run, [rows, view, ready, force]);
  const rowById = new Map(res.rows.map((r) => [r.id, r]));
  const p: ViewProps = {
    db: database,
    view,
    result: res,
    rowById,
    locked: database.locked,
    openRow: (id) => log("openRow", id),
    setCells: async (id, patch) => {
      log("setCells", id, patch);
      rows = rows.map((r) => {
        if (r.id !== id) return r;
        const values = { ...r.values };
        for (const [k, v] of Object.entries(patch)) {
          if (v === null) delete values[k];
          else values[k] = v;
        }
        return { ...r, values };
      });
      bump();
    },
    setTitle: async (id, title) => {
      log("setTitle", id, title);
      rows = rows.map((r) => (r.id === id ? { ...r, title } : r));
      bump();
    },
    createRow: async (cells = {}, title = "") => {
      log("createRow", cells, title);
      const row = mk(nextRow++, title, { ...cells });
      rows = [...rows, row];
      bump();
      return row;
    },
    updateView: (patch) => {
      log("updateView", patch);
      view = { ...view, ...patch };
      bump();
    },
    reorderRows: async (ids) => {
      log("reorderRows", ids);
      const by = new Map(rows.map((r) => [r.id, r]));
      rows = ids.map((i) => by.get(i)!).filter(Boolean);
      bump();
    },
  };
  const modules = import.meta.glob("/src/features/database/views/*View.tsx", { eager: true }) as Record<string, { default: React.ComponentType<ViewProps> }>;
  const names: Record<string, string> = { board: "BoardView", calendar: "CalendarView", timeline: "TimelineView", list: "ListView", gallery: "GalleryView", chart: "ChartView" };
  const mod = Object.entries(modules).find(([k]) => k.endsWith(`/${names[kind]}.tsx`))?.[1];
  const View = mod?.default;
  return (
    <div className="flex h-full flex-col bg-bg text-fg" data-ready={ready}>
      <div className="flex shrink-0 items-center gap-3 border-b border-line px-4 py-2 text-sm">
        <span className="font-medium">{database.title}</span>
        {Object.keys(names).map((k) => (
          <a key={k} href={`?view=${k}`} className={k === kind ? "text-accent" : "text-muted"}>{k}</a>
        ))}
        <span className="ms-auto text-xs text-faint" id="harness-count">{res.rows.length} rows</span>
      </div>
      <div className="min-h-0 flex-1" id="view-host">{View ? <View {...p} /> : <div className="p-6">No component for {kind}</div>}</div>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(<Harness />);
