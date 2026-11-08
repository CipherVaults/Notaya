// Dev-only in-memory stand-in for the database commands (A35), used by `npm run mock`.
// A simplified engine: it follows the contract of src/lib/database-types.ts, not every detail of the Rust engine.
// Seed data (a Tasks and a Projects database) is added when the URL has ?db=1.

import type {
  ChartData, Cond, Database, DbRow, Filter, GroupBy, GroupOut, Property, PropKind, Template, View, ViewResult,
} from "../src/lib/database-types";

const params = new URLSearchParams(location.search);
const now0 = Date.now();

let idn = 0;
const hex = () => {
  const a = new Uint8Array(16);
  crypto.getRandomValues(a);
  return Array.from(a, (b) => b.toString(16).padStart(2, "0")).join("");
};
const nid = () => (idn++, hex());

interface MRow {
  id: string; db: string; title: string; content: string; text: string; cells: Record<string, unknown>;
  icon: string; cover: string; order: number; favorite: boolean; number: number;
  createdBy: string; editedBy: string; trashedAt: number | null; created: number; updated: number;
}

const dbs: Database[] = [];
const rows: MRow[] = [];
let rowOrder = 0;
const files = new Map<string, { name: string; mime: string; size: number }>();

// ---------------------------------------------------------------- values

type Val = null | number | string | boolean | DateV | Val[];
interface DateV { d: true; start: number; end?: number; hasTime: boolean }
const isDate = (v: unknown): v is DateV => typeof v === "object" && v !== null && !Array.isArray(v) && (v as DateV).d === true;
const DAY = 1440;

function parseDate(s: unknown): DateV | null {
  if (typeof s !== "string") return null;
  const m = /^(\d{4})-(\d{2})-(\d{2})(?:[T ](\d{2}):(\d{2}))?/.exec(s);
  if (!m) return null;
  const t = Date.UTC(+m[1], +m[2] - 1, +m[3], m[4] ? +m[4] : 0, m[5] ? +m[5] : 0) / 60000;
  return { d: true, start: t, hasTime: !!m[4] };
}
const pad = (n: number) => String(n).padStart(2, "0");
function isoOf(min: number, time: boolean): string {
  const d = new Date(min * 60000);
  const day = `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}`;
  return time ? `${day}T${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}` : day;
}
const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
function fmtDate1(min: number, time: boolean, style: string): string {
  const d = new Date(min * 60000);
  const y = d.getUTCFullYear(), mo = d.getUTCMonth(), da = d.getUTCDate();
  const date =
    style === "short" ? `${MONTHS[mo].slice(0, 3)} ${da}, ${y}`
      : style === "dmy" ? `${pad(da)}/${pad(mo + 1)}/${y}`
        : style === "mdy" ? `${pad(mo + 1)}/${pad(da)}/${y}`
          : style === "ymd" ? `${y}/${pad(mo + 1)}/${pad(da)}`
            : `${MONTHS[mo]} ${da}, ${y}`;
  return time ? `${date} ${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}` : date;
}
const fmtDate = (d: DateV, style: string) => (d.end != null ? `${fmtDate1(d.start, d.hasTime, style)} → ${fmtDate1(d.end, d.hasTime, style)}` : fmtDate1(d.start, d.hasTime, style));
const nowMin = () => Math.floor(Date.now() / 60000);
const startOfDay = (m: number) => Math.floor(m / DAY) * DAY;
const startOfWeek = (m: number) => { const s = startOfDay(m); const wd = (new Date(s * 60000).getUTCDay() + 6) % 7; return s - wd * DAY; };
const startOfMonth = (m: number) => { const d = new Date(m * 60000); return Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), 1) / 60000; };
const startOfYear = (m: number) => Date.UTC(new Date(m * 60000).getUTCFullYear(), 0, 1) / 60000;
const addMonths = (m: number, n: number) => { const d = new Date(m * 60000); return Date.UTC(d.getUTCFullYear(), d.getUTCMonth() + n, d.getUTCDate(), d.getUTCHours(), d.getUTCMinutes()) / 60000; };

const tidy = (n: number) => Math.round(n * 1e9) / 1e9;
function numStr(n: number): string { return String(tidy(n)); }
const commas = (s: string) => { const [i, f] = s.split("."); return i.replace(/\B(?=(\d{3})+(?!\d))/g, ",") + (f ? "." + f : ""); };
function fmtNum(n: number, f: string): string {
  const base = numStr(n);
  const money = (sym: string) => `${n < 0 ? "-" : ""}${sym}${commas(numStr(Math.abs(n)))}`;
  switch (f) {
    case "numberWithCommas": return commas(base);
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
function textOf(v: Val): string {
  if (v == null) return "";
  if (typeof v === "number") return numStr(v);
  if (typeof v === "boolean") return v ? "true" : "false";
  if (typeof v === "string") return v;
  if (isDate(v)) return fmtDate(v, "full");
  return v.map(textOf).join(", ");
}
const isEmptyV = (v: Val) => v == null || v === "" || (Array.isArray(v) && v.length === 0);
function cmp(a: Val, b: Val): number {
  if (typeof a === "number" && typeof b === "number") return a - b;
  if (isDate(a) && isDate(b)) return a.start - b.start;
  return textOf(a).localeCompare(textOf(b), undefined, { numeric: true, sensitivity: "base" });
}

// ---------------------------------------------------------------- engine context

const rowsOf = (dbId: string) => rows.filter((r) => r.db === dbId && r.trashedAt == null).sort((a, b) => a.order - b.order || a.created - b.created);
const rowById = (id: string) => rows.find((r) => r.id === id);
const dbById = (id: string) => dbs.find((d) => d.id === id);

function relIds(r: MRow, p: Property): string[] {
  const v = r.cells[p.id];
  return Array.isArray(v) ? (v as string[]).filter((id) => { const t = rowById(id); return t && t.trashedAt == null; }) : [];
}

const memo = new Map<string, Val | Error>();
const busy = new Set<string>();

function valueOf(db: Database, r: MRow, p: Property): Val {
  switch (p.kind) {
    case "formula": case "rollup": {
      const key = r.id + "|" + p.id;
      if (memo.has(key)) { const m = memo.get(key)!; if (m instanceof Error) throw m; return m; }
      if (busy.has(key)) throw new Error("Circular reference");
      busy.add(key);
      try {
        const v = p.kind === "formula" ? runFormula(p.formula, db, r) : rollup(db, r, p);
        memo.set(key, v);
        return v;
      } catch (e) {
        memo.set(key, e as Error);
        throw e;
      } finally { busy.delete(key); }
    }
    default: return stored(db, r, p);
  }
}
const safeValue = (db: Database, r: MRow, p: Property): Val => { try { return valueOf(db, r, p); } catch { return null; } };

function stored(_db: Database, r: MRow, p: Property): Val {
  const raw = r.cells[p.id];
  switch (p.kind) {
    case "title": return r.title || null;
    case "text": case "url": case "email": case "phone": return typeof raw === "string" && raw ? raw : null;
    case "number": return typeof raw === "number" ? raw : null;
    case "select": case "status": return typeof raw === "string" ? (p.options.find((o) => o.id === raw)?.name ?? null) : null;
    case "multiSelect": return Array.isArray(raw) ? (raw as string[]).map((id) => p.options.find((o) => o.id === id)?.name).filter((x): x is string => !!x) : [];
    case "date": {
      const o = raw as { start?: string; end?: string } | undefined;
      const d = o && parseDate(o.start);
      if (!d) return null;
      const e = parseDate(o?.end);
      if (e && e.start >= d.start) d.end = e.start;
      return d;
    }
    case "person": return Array.isArray(raw) ? (raw as string[]) : [];
    case "files": return Array.isArray(raw) ? (raw as { name: string }[]).map((f) => f.name) : [];
    case "checkbox": return raw === true;
    case "relation": return relIds(r, p).map((id) => rowById(id)!.title || "Untitled");
    case "place": { const a = (raw as { address?: string } | undefined)?.address; return a ? a : null; }
    case "id": return r.number > 0 ? (p.idPrefix ? `${p.idPrefix}-${r.number}` : String(r.number)) : null;
    case "createdTime": return { d: true, start: Math.floor(r.created / 60000), hasTime: true };
    case "lastEditedTime": return { d: true, start: Math.floor(r.updated / 60000), hasTime: true };
    case "createdBy": return r.createdBy || null;
    case "lastEditedBy": return r.editedBy || null;
    default: return null;
  }
}

const CALCS_PCT = new Set(["percentEmpty", "percentNotEmpty", "percentChecked", "percentUnchecked"]);
function calc(name: string, vals: Val[]): Val {
  const nums = vals.filter((v): v is number => typeof v === "number");
  const dates = vals.filter(isDate);
  const n = vals.length || 1;
  switch (name) {
    case "countAll": return vals.length;
    case "countValues": case "countNotEmpty": return vals.filter((v) => !isEmptyV(v)).length;
    case "countEmpty": return vals.filter(isEmptyV).length;
    case "countUnique": return new Set(vals.flatMap((v) => (Array.isArray(v) ? v : [v])).filter((v) => !isEmptyV(v)).map(textOf)).size;
    case "percentEmpty": return vals.filter(isEmptyV).length / n;
    case "percentNotEmpty": return vals.filter((v) => !isEmptyV(v)).length / n;
    case "sum": return tidy(nums.reduce((a, b) => a + b, 0));
    case "avg": return nums.length ? tidy(nums.reduce((a, b) => a + b, 0) / nums.length) : null;
    case "median": { if (!nums.length) return null; const s = [...nums].sort((a, b) => a - b); const m = s.length >> 1; return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2; }
    case "min": return nums.length ? Math.min(...nums) : null;
    case "max": return nums.length ? Math.max(...nums) : null;
    case "range": return nums.length ? Math.max(...nums) - Math.min(...nums) : null;
    case "earliest": return dates.length ? dates.reduce((a, b) => (b.start < a.start ? b : a)) : null;
    case "latest": return dates.length ? dates.reduce((a, b) => (b.start > a.start ? b : a)) : null;
    case "dateRange": return dates.length ? (Math.max(...dates.map((d) => d.end ?? d.start)) - Math.min(...dates.map((d) => d.start))) / DAY : null;
    case "checked": return vals.filter((v) => v === true).length;
    case "unchecked": return vals.filter((v) => v !== true).length;
    case "percentChecked": return vals.filter((v) => v === true).length / n;
    case "percentUnchecked": return vals.filter((v) => v !== true).length / n;
    case "showOriginal": return vals.flatMap((v) => (Array.isArray(v) ? v : [v]));
    case "showUnique": return [...new Set(vals.flatMap((v) => (Array.isArray(v) ? v : [v])).filter((v) => !isEmptyV(v)).map(textOf))];
    default: return null;
  }
}
function calcShown(name: string, res: Val, p: Property): string {
  if (typeof res === "number") {
    if (CALCS_PCT.has(name)) return numStr(Math.round(res * 1000) / 10) + "%";
    if (["sum", "avg", "median", "min", "max", "range"].includes(name)) return fmtNum(res, p.numberFormat);
    if (name === "dateRange") return numStr(res) + " days";
    return numStr(res);
  }
  if (isDate(res)) return fmtDate(res, p.dateFormat);
  return textOf(res);
}

function rollup(db: Database, r: MRow, p: Property): Val {
  const rel = db.props.find((x) => x.id === p.rollupRelation && x.kind === "relation");
  if (!rel) throw new Error("Pick a relation to roll up");
  const tdb = dbById(rel.relationDb);
  if (!tdb) throw new Error("The related database is gone");
  const tp = tdb.props.find((x) => x.id === p.rollupProp);
  if (!tp) throw new Error("Pick a property to roll up");
  const vals = relIds(r, rel).map((id) => valueOf(tdb, rowById(id)!, tp));
  return calc(p.rollupCalc || "showOriginal", vals);
}

// ---------------------------------------------------------------- formulas

type Tok = { t: "num" | "str" | "id" | "op" | "("  | ")" | "," | "?" | ":"; v: string };
function lex(src: string): Tok[] {
  const out: Tok[] = [];
  let i = 0;
  while (i < src.length) {
    const c = src[i];
    if (/\s/.test(c)) { i++; continue; }
    if (/[0-9]/.test(c) || (c === "." && /[0-9]/.test(src[i + 1] ?? ""))) {
      let j = i; while (j < src.length && /[0-9.]/.test(src[j])) j++;
      out.push({ t: "num", v: src.slice(i, j) }); i = j; continue;
    }
    if (c === '"' || c === "'") {
      let j = i + 1; let s = "";
      while (j < src.length && src[j] !== c) { if (src[j] === "\\") j++; s += src[j]; j++; }
      if (j >= src.length) throw new Error("Text is not closed");
      out.push({ t: "str", v: s }); i = j + 1; continue;
    }
    if (/[\p{L}_]/u.test(c)) { let j = i; while (j < src.length && /[\p{L}\p{N}_]/u.test(src[j])) j++; out.push({ t: "id", v: src.slice(i, j) }); i = j; continue; }
    const two = src.slice(i, i + 2);
    if (["==", "!=", "<=", ">=", "&&", "||"].includes(two)) { out.push({ t: "op", v: two }); i += 2; continue; }
    if ("+-*/%^<>!=".includes(c)) { out.push({ t: "op", v: c }); i++; continue; }
    if (c === "(" || c === ")" || c === "," || c === "?" || c === ":") { out.push({ t: c, v: c } as Tok); i++; continue; }
    throw new Error(`Unexpected character ${c}`);
  }
  return out;
}
type Ex = { k: "lit"; v: Val } | { k: "id"; n: string } | { k: "call"; n: string; a: Ex[] } | { k: "un"; o: string; a: Ex } | { k: "bin"; o: string; a: Ex; b: Ex } | { k: "tern"; c: Ex; a: Ex; b: Ex };
function parse(src: string): Ex {
  const t = lex(src);
  let i = 0;
  const peek = () => t[i];
  const eat = (k: string, v?: string) => (t[i] && t[i].t === k && (v == null || t[i].v === v) ? t[i++] : null);
  const levels: string[][] = [["||", "or"], ["&&", "and"], ["==", "!="], ["<", ">", "<=", ">="], ["+", "-"], ["*", "/", "%"], ["^"]];
  const ternary = (): Ex => {
    const c = bin(0);
    if (eat("?")) { const a = ternary(); if (!eat(":")) throw new Error("Expected : in a ? b : c"); const b = ternary(); return { k: "tern", c, a, b }; }
    return c;
  };
  const bin = (lv: number): Ex => {
    if (lv >= levels.length) return unary();
    let l = bin(lv + 1);
    for (;;) {
      const p = peek();
      if (p && (p.t === "op" || p.t === "id") && levels[lv].includes(p.v)) { i++; l = { k: "bin", o: p.v === "or" ? "||" : p.v === "and" ? "&&" : p.v, a: l, b: bin(lv + 1) }; } else return l;
    }
  };
  const unary = (): Ex => {
    const p = peek();
    if (p && ((p.t === "op" && (p.v === "-" || p.v === "!")) || (p.t === "id" && p.v === "not"))) { i++; return { k: "un", o: p.v === "not" ? "!" : p.v, a: unary() }; }
    return atom();
  };
  const atom = (): Ex => {
    const p = t[i++];
    if (!p) throw new Error("The formula ends too soon");
    if (p.t === "num") return { k: "lit", v: parseFloat(p.v) };
    if (p.t === "str") return { k: "lit", v: p.v };
    if (p.t === "(") { const e = ternary(); if (!eat(")")) throw new Error("Missing )"); return e; }
    if (p.t === "id") {
      if (p.v === "true") return { k: "lit", v: true };
      if (p.v === "false") return { k: "lit", v: false };
      if (eat("(")) {
        const a: Ex[] = [];
        if (!eat(")")) { do a.push(ternary()); while (eat(",")); if (!eat(")")) throw new Error("Missing )"); }
        return { k: "call", n: p.v, a };
      }
      return { k: "id", n: p.v };
    }
    throw new Error(`Unexpected ${p.v}`);
  };
  const e = ternary();
  if (i < t.length) throw new Error(`Unexpected ${t[i].v}`);
  return e;
}
const num = (v: Val): number => {
  if (typeof v === "number") return v;
  if (typeof v === "boolean") return v ? 1 : 0;
  if (typeof v === "string" && v.trim() !== "" && !isNaN(Number(v))) return Number(v);
  if (v == null) return 0;
  throw new Error("Expected a number");
};
const truthy = (v: Val) => (typeof v === "boolean" ? v : typeof v === "number" ? v !== 0 : !isEmptyV(v));
function evalEx(e: Ex, db: Database, r: MRow): Val {
  switch (e.k) {
    case "lit": return e.v;
    case "id": if (e.n === "e") return Math.E; if (e.n === "pi") return Math.PI; throw new Error(`Unknown name ${e.n}`);
    case "un": { const a = evalEx(e.a, db, r); return e.o === "-" ? -num(a) : !truthy(a); }
    case "tern": return truthy(evalEx(e.c, db, r)) ? evalEx(e.a, db, r) : evalEx(e.b, db, r);
    case "bin": {
      if (e.o === "&&") return truthy(evalEx(e.a, db, r)) && truthy(evalEx(e.b, db, r));
      if (e.o === "||") return truthy(evalEx(e.a, db, r)) || truthy(evalEx(e.b, db, r));
      const a = evalEx(e.a, db, r), b = evalEx(e.b, db, r);
      switch (e.o) {
        case "+": if (typeof a === "string" || typeof b === "string") return textOf(a) + textOf(b); if (isDate(a)) return { ...a, start: a.start + num(b) * DAY, end: undefined }; return tidy(num(a) + num(b));
        case "-": if (isDate(a) && isDate(b)) return a.start - b.start; return tidy(num(a) - num(b));
        case "*": return tidy(num(a) * num(b));
        case "/": { const d = num(b); if (d === 0) throw new Error("Division by zero"); return tidy(num(a) / d); }
        case "%": return num(a) % num(b);
        case "^": return Math.pow(num(a), num(b));
        case "==": return textOf(a) === textOf(b);
        case "!=": return textOf(a) !== textOf(b);
        case "<": return cmp(a, b) < 0;
        case ">": return cmp(a, b) > 0;
        case "<=": return cmp(a, b) <= 0;
        case ">=": return cmp(a, b) >= 0;
      }
      throw new Error("Bad operator");
    }
    case "call": {
      const n = e.n;
      if (n === "if") { if (e.a.length !== 3) throw new Error("if needs 3 arguments"); return truthy(evalEx(e.a[0], db, r)) ? evalEx(e.a[1], db, r) : evalEx(e.a[2], db, r); }
      const a = e.a.map((x) => evalEx(x, db, r));
      switch (n) {
        case "prop": { const name = textOf(a[0]); const p = db.props.find((x) => x.name === name) ?? db.props.find((x) => x.name.toLowerCase() === name.toLowerCase()); if (!p) throw new Error(`No property named ${name}`); return valueOf(db, r, p); }
        case "concat": return a.map(textOf).join("");
        case "round": { const k = a[1] != null ? num(a[1]) : 0; const f = Math.pow(10, k); return Math.round(num(a[0]) * f) / f; }
        case "floor": return Math.floor(num(a[0]));
        case "ceil": return Math.ceil(num(a[0]));
        case "abs": return Math.abs(num(a[0]));
        case "sqrt": return Math.sqrt(num(a[0]));
        case "min": return Math.min(...a.flat().map(num));
        case "max": return Math.max(...a.flat().map(num));
        case "sum": return tidy(a.flat().map(num).reduce((x, y) => x + y, 0));
        case "length": return Array.isArray(a[0]) ? a[0].length : textOf(a[0]).length;
        case "upper": return textOf(a[0]).toUpperCase();
        case "lower": return textOf(a[0]).toLowerCase();
        case "trim": return textOf(a[0]).trim();
        case "contains": return Array.isArray(a[0]) ? a[0].map(textOf).includes(textOf(a[1])) : textOf(a[0]).includes(textOf(a[1]));
        case "empty": return isEmptyV(a[0]);
        case "not": return !truthy(a[0]);
        case "and": return a.every(truthy);
        case "or": return a.some(truthy);
        case "format": return textOf(a[0]);
        case "toNumber": return num(a[0]);
        case "now": return { d: true, start: nowMin(), hasTime: true };
        case "today": return { d: true, start: startOfDay(nowMin()), hasTime: false };
        case "dateBetween": {
          if (!isDate(a[0]) || !isDate(a[1])) throw new Error("dateBetween needs two dates");
          const u = textOf(a[2] ?? "days"); const diff = a[0].start - a[1].start;
          const div = u.startsWith("minute") ? 1 : u.startsWith("hour") ? 60 : u.startsWith("week") ? 7 * DAY : DAY;
          return Math.trunc(diff / div);
        }
        case "formatDate": { if (!isDate(a[0])) throw new Error("Expected a date"); return fmtDate1(a[0].start, a[0].hasTime, "full"); }
        case "year": return isDate(a[0]) ? new Date(a[0].start * 60000).getUTCFullYear() : 0;
        case "month": return isDate(a[0]) ? new Date(a[0].start * 60000).getUTCMonth() + 1 : 0;
        case "date": return isDate(a[0]) ? new Date(a[0].start * 60000).getUTCDate() : 0;
        case "count": return Array.isArray(a[0]) ? a[0].length : 1;
        case "first": return Array.isArray(a[0]) ? (a[0][0] ?? null) : a[0];
        case "last": return Array.isArray(a[0]) ? (a[0][a[0].length - 1] ?? null) : a[0];
        case "join": return (Array.isArray(a[0]) ? a[0] : [a[0]]).map(textOf).join(textOf(a[1] ?? ", "));
        case "replace": case "replaceAll": return textOf(a[0]).split(textOf(a[1])).join(textOf(a[2]));
        case "substring": case "slice": return textOf(a[0]).slice(num(a[1]), a[2] != null ? num(a[2]) : undefined);
        case "id": return r.id;
      }
      throw new Error(`Unknown function ${n}`);
    }
  }
}
function runFormula(src: string, db: Database, r: MRow): Val {
  if (!src.trim()) return null;
  return evalEx(parse(src), db, r);
}
const typeName = (v: Val): string => (v == null ? "null" : typeof v === "number" ? "number" : typeof v === "boolean" ? "bool" : typeof v === "string" ? "text" : isDate(v) ? "date" : "list");
function valJson(v: Val): unknown {
  if (isDate(v)) return { start: isoOf(v.start, v.hasTime), end: v.end != null ? isoOf(v.end, v.hasTime) : null, hasTime: v.hasTime };
  if (Array.isArray(v)) return v.map(valJson);
  return v;
}
function computed(p: Property, db: Database, r: MRow) {
  try {
    const v = valueOf(db, r, p);
    const text = typeof v === "number" ? fmtNum(v, p.numberFormat) : isDate(v) ? fmtDate(v, p.dateFormat) : textOf(v);
    return { t: typeName(v), v: valJson(v), text };
  } catch (e) {
    return { t: "error", v: null, text: "⚠ " + (e as Error).message };
  }
}

// ---------------------------------------------------------------- filters

function dateRange(c: Cond): [number, number] | null {
  const today = startOfDay(nowMin());
  switch (c.rel) {
    case "": case "exact": { const d = parseDate(c.value); if (!d) return null; return d.hasTime ? [d.start, d.start + 1] : [d.start, d.start + DAY]; }
    case "today": return [today, today + DAY];
    case "tomorrow": return [today + DAY, today + 2 * DAY];
    case "yesterday": return [today - DAY, today];
    case "oneWeekAgo": return [today - 7 * DAY, today - 6 * DAY];
    case "oneWeekFromNow": return [today + 7 * DAY, today + 8 * DAY];
    case "oneMonthAgo": { const t = addMonths(today, -1); return [t, t + DAY]; }
    case "oneMonthFromNow": { const t = addMonths(today, 1); return [t, t + DAY]; }
    case "thisWeek": { const s = startOfWeek(today); return [s, s + 7 * DAY]; }
    case "pastWeek": return [today - 6 * DAY, today + DAY];
    case "nextWeek": return [today + DAY, today + 8 * DAY];
    case "pastMonth": return [today - 29 * DAY, today + DAY];
    case "nextMonth": return [today + DAY, today + 31 * DAY];
    case "thisMonth": { const s = startOfMonth(today); return [s, addMonths(s, 1)]; }
    case "thisYear": { const s = startOfYear(today); return [s, addMonths(s, 12)]; }
    case "pastYear": return [addMonths(today, -12) + DAY, today + DAY];
    case "nextYear": return [today + DAY, addMonths(today, 12) + DAY];
  }
  return null;
}

function test(_db: Database, r: MRow, p: Property, v: Val, c: Cond): boolean {
  if (c.op === "isEmpty") return isEmptyV(v);
  if (c.op === "isNotEmpty") return !isEmptyV(v);
  const wanted = () => [...c.values, ...(c.value ? [c.value] : [])].map((id) => p.options.find((o) => o.id === id)?.name ?? id);
  switch (p.kind) {
    case "select": case "status": {
      const hit = wanted().some((n) => typeof v === "string" && v.toLowerCase() === n.toLowerCase());
      return ["is", "in", "contains"].includes(c.op) ? hit : ["isNot", "notIn", "notContains"].includes(c.op) ? !hit : true;
    }
    case "multiSelect": case "person": case "files": {
      const names = p.kind === "multiSelect" ? wanted() : [c.value];
      const items = Array.isArray(v) ? v.map(textOf) : [];
      const hit = names.some((n) => items.some((i) => i.toLowerCase() === n.toLowerCase() || (p.kind !== "multiSelect" && i.toLowerCase().includes(n.toLowerCase()))));
      return ["contains", "is", "in"].includes(c.op) ? hit : ["notContains", "isNot", "notIn"].includes(c.op) ? !hit : true;
    }
    case "relation": {
      const ids = relIds(r, p);
      const hit = [...c.values, ...(c.value ? [c.value] : [])].some((w) => ids.includes(w));
      return ["contains", "is"].includes(c.op) ? hit : ["notContains", "isNot"].includes(c.op) ? !hit : true;
    }
    case "checkbox": return c.op === "is" ? (v === true) === (c.value === "true") : c.op === "isNot" ? (v === true) !== (c.value === "true") : true;
  }
  if (typeof v === "number") {
    const w = parseFloat(c.value);
    if (isNaN(w)) return true;
    switch (c.op) {
      case "eq": case "equals": case "is": return tidy(v) === tidy(w);
      case "neq": case "notEquals": case "isNot": return tidy(v) !== tidy(w);
      case "gt": return v > w; case "gte": return v >= w; case "lt": return v < w; case "lte": return v <= w;
    }
    return true;
  }
  if (isDate(v)) {
    let lo: number, hi: number;
    if (c.op === "between") {
      const a = parseDate(c.value), b = parseDate(c.value2);
      if (!a || !b) return true;
      lo = Math.min(a.start, b.start); hi = startOfDay(Math.max(a.start, b.start)) + DAY;
    } else { const t = dateRange(c); if (!t) return true; [lo, hi] = t; }
    const end = v.end ?? v.start;
    const overlaps = v.start < hi && end >= lo;
    switch (c.op) {
      case "is": case "between": return overlaps;
      case "before": return v.start < lo;
      case "after": return end >= hi;
      case "onOrBefore": return v.start < hi;
      case "onOrAfter": return end >= lo;
      case "isNot": return !overlaps;
    }
    return true;
  }
  if (typeof v === "boolean") return c.op === "is" ? v === (c.value === "true") : c.op === "isNot" ? v !== (c.value === "true") : true;
  if (p.kind === "number") return ["neq", "notEquals"].includes(c.op);
  if (["date", "createdTime", "lastEditedTime"].includes(p.kind)) return false;
  const t = textOf(v).toLowerCase(), w = c.value.toLowerCase();
  switch (c.op) {
    case "equals": case "is": return t === w;
    case "notEquals": case "isNot": return t !== w;
    case "contains": return t.includes(w);
    case "notContains": return !t.includes(w);
    case "startsWith": return t.startsWith(w);
    case "endsWith": return t.endsWith(w);
  }
  return true;
}
function filterEmpty(f: Filter): boolean { return f.conds.length === 0 && f.groups.every(filterEmpty); }
function matches(db: Database, r: MRow, f: Filter): boolean {
  const res: boolean[] = [];
  for (const c of f.conds) { const p = db.props.find((x) => x.id === c.prop); if (p) res.push(test(db, r, p, safeValue(db, r, p), c)); }
  for (const g of f.groups) if (!filterEmpty(g)) res.push(matches(db, r, g));
  if (f.op === "or") return res.length === 0 || res.some(Boolean);
  return res.every(Boolean);
}

// ---------------------------------------------------------------- sort / group

function sortKey(db: Database, r: MRow, p: Property): Val {
  const v = safeValue(db, r, p);
  if (p.kind === "select" || p.kind === "status") return typeof v === "string" ? p.options.findIndex((o) => o.name === v) : null;
  if (p.kind === "id") return r.number;
  return v;
}
function sorted(db: Database, list: MRow[], sorts: View["sorts"]): MRow[] {
  const ps = sorts.map((s) => ({ p: db.props.find((x) => x.id === s.prop), d: s.descending })).filter((x): x is { p: Property; d: boolean } => !!x.p);
  if (!ps.length) return list;
  const keyed = new Map(list.map((r) => [r.id, ps.map((x) => sortKey(db, r, x.p))]));
  return [...list].sort((a, b) => {
    for (let i = 0; i < ps.length; i++) {
      const x = keyed.get(a.id)![i], y = keyed.get(b.id)![i];
      if (x == null && y == null) continue;
      if (x == null) return 1;
      if (y == null) return -1;
      const o = cmp(x, y);
      if (o !== 0) return ps[i].d ? -o : o;
    }
    return 0;
  });
}

function groupKeys(db: Database, r: MRow, p: Property, g: GroupBy): [string, string][] {
  const none: [string, string][] = [["", `No ${p.name}`]];
  const v = safeValue(db, r, p);
  switch (p.kind) {
    case "select": case "status": {
      if (typeof v !== "string") return none;
      const o = p.options.find((x) => x.name === v);
      if (p.kind === "status" && g.dateBy === "group") { const grp = o?.group || "todo"; return [[grp, grp === "inProgress" ? "In progress" : grp === "complete" ? "Complete" : "To-do"]]; }
      return o ? [[o.id, o.name]] : none;
    }
    case "multiSelect": return Array.isArray(v) && v.length ? (v as string[]).map((n) => p.options.find((o) => o.name === n)).filter((o): o is NonNullable<typeof o> => !!o).map((o) => [o.id, o.name] as [string, string]) : none;
    case "checkbox": return [v === true ? ["true", "Checked"] : ["false", "Unchecked"]];
  }
  if (v == null) return none;
  if (isDate(v)) {
    if (g.dateBy === "week") { const s = startOfWeek(v.start); return [[String(s + 2 ** 40).padStart(20, "0"), "Week of " + fmtDate1(s, false, "short")]]; }
    if (g.dateBy === "month") { const s = startOfMonth(v.start); return [[String(s + 2 ** 40).padStart(20, "0"), new Date(s * 60000).toLocaleString("en", { month: "long", year: "numeric", timeZone: "UTC" })]]; }
    if (g.dateBy === "year") { const s = startOfYear(v.start); return [[String(s + 2 ** 40).padStart(20, "0"), String(new Date(s * 60000).getUTCFullYear())]]; }
    if (g.dateBy === "relative") {
      const today = startOfDay(nowMin()), dd = startOfDay(v.start);
      const [rank, label] = dd === today ? [0, "Today"] : dd === today + DAY ? [1, "Tomorrow"] : dd === today - DAY ? [-1, "Yesterday"] : dd < today ? [-2, "Earlier"] : dd < startOfWeek(today) + 7 * DAY ? [2, "This week"] : dd < startOfWeek(today) + 14 * DAY ? [3, "Next week"] : [4, "Later"];
      return [[String(rank + 10).padStart(3, "0"), label as string]];
    }
    const s = startOfDay(v.start);
    return [[String(s + 2 ** 40).padStart(20, "0"), fmtDate1(s, false, "short")]];
  }
  if (typeof v === "number") {
    if (g.numberStep > 0) { const lo = Math.floor(v / g.numberStep) * g.numberStep; return [[String(lo + 1e12).padStart(24, "0"), `${numStr(lo)} – ${numStr(lo + g.numberStep)}`]]; }
    return [[String(v + 1e12).padStart(24, "0"), numStr(v)]];
  }
  if (Array.isArray(v)) return v.length ? v.map((x) => [textOf(x).toLowerCase(), textOf(x)] as [string, string]) : none;
  const t = textOf(v);
  return t ? [[t.toLowerCase(), t]] : none;
}
function buildGroups(db: Database, list: MRow[], g: GroupBy): GroupOut[] {
  const p = db.props.find((x) => x.id === g.prop);
  if (!p) return [];
  const map = new Map<string, GroupOut>();
  const order: string[] = [];
  const add = (key: string, label: string, color: string) => { order.push(key); map.set(key, { key, label, color, count: 0, rows: [], subs: [], calcs: {} }); };
  let seeded = false;
  if (p.kind === "select" || p.kind === "multiSelect" || p.kind === "status") {
    seeded = true;
    add("", `No ${p.name}`, "");
    if (p.kind === "status" && g.dateBy === "group") { add("todo", "To-do", "gray"); add("inProgress", "In progress", "blue"); add("complete", "Complete", "green"); } else for (const o of p.options) add(o.id, o.name, o.color);
  } else if (p.kind === "checkbox") { seeded = true; add("false", "Unchecked", ""); add("true", "Checked", ""); }
  for (const r of list) for (const [k, label] of groupKeys(db, r, p, g)) {
    if (!map.has(k)) add(k, label, "");
    const e = map.get(k)!; e.rows.push(r.id); e.count++;
  }
  let out = order.map((k) => map.get(k)!);
  const manual = !g.sort || g.sort === "manual";
  if (!manual || !seeded) {
    out = [...out].sort((a, b) => {
      if ((a.key === "") !== (b.key === "")) return a.key === "" ? 1 : -1;
      const o = a.key < b.key && /^\d/.test(a.key) ? -1 : a.key > b.key && /^\d/.test(a.key) ? 1 : a.label.localeCompare(b.label, undefined, { numeric: true });
      return g.sort === "desc" ? -o : o;
    });
  }
  if (g.hideEmpty) out = out.filter((x) => x.count > 0);
  return out.filter((x) => !g.hidden.includes(x.key));
}
function footer(db: Database, calcs: View["calcs"], list: MRow[]): Record<string, string> {
  const out: Record<string, string> = {};
  for (const c of calcs) {
    if (!c.calc || c.calc === "none") continue;
    const p = db.props.find((x) => x.id === c.prop);
    if (!p) continue;
    out[c.prop] = calcShown(c.calc, calc(c.calc, list.map((r) => safeValue(db, r, p))), p);
  }
  return out;
}

function rowOut(db: Database, r: MRow): DbRow {
  const values: Record<string, unknown> = {};
  for (const p of db.props) {
    if (p.kind === "title") continue;
    if (["formula", "rollup", "createdTime", "lastEditedTime", "createdBy", "lastEditedBy", "id"].includes(p.kind)) values[p.id] = computed(p, db, r);
    else if (r.cells[p.id] !== undefined) values[p.id] = r.cells[p.id];
  }
  return { id: r.id, title: r.title, icon: r.icon, number: r.number, created: r.created, updated: r.updated, favorite: r.favorite, values };
}

function runView(db: Database, view: View): ViewResult & { _rows: MRow[] } {
  memo.clear();
  const all = rowsOf(db.id);
  const f = all.filter((r) => filterEmpty(view.filter) || matches(db, r, view.filter));
  const list = sorted(db, f, view.sorts);
  const calcs = footer(db, view.calcs, list);
  const byId = new Map(list.map((r) => [r.id, r]));
  const groups = view.group ? buildGroups(db, list, view.group).map((g) => {
    const mem = g.rows.map((id) => byId.get(id)!).filter(Boolean);
    g.calcs = footer(db, view.calcs, mem);
    if (view.subGroup) g.subs = buildGroups(db, mem, view.subGroup).map((s) => { s.calcs = footer(db, view.calcs, s.rows.map((id) => byId.get(id)!).filter(Boolean)); return s; });
    return g;
  }) : null;
  return { rows: list.map((r) => rowOut(db, r)), total: all.length, filtered: list.length, truncated: false, groups, calcs, _rows: list };
}

// ---------------------------------------------------------------- charts

function chartData(db: Database, view: View): ChartData {
  const c = view.chart;
  const res = runView(db, { ...view, group: null });
  const xp = db.props.find((p) => p.id === c.xProp);
  const gp = db.props.find((p) => p.id === c.groupProp);
  const yp = db.props.find((p) => p.id === c.yProp);
  const palette = ["blue", "green", "orange", "purple", "red", "yellow", "pink", "brown", "gray"];
  const xkeys: [string, string][] = [];
  const buckets = new Map<string, Map<string, MRow[]>>();
  const skeys: [string, string][] = [];
  for (const r of res._rows) {
    const xs = xp ? groupKeys(db, r, xp, { prop: xp.id, sort: "", dateBy: c.xBy || "month", numberStep: c.xStep, hideEmpty: false, hidden: [], collapsed: [] }) : [["", "All"] as [string, string]];
    const gs = gp ? groupKeys(db, r, gp, { prop: gp.id, sort: "", dateBy: "month", numberStep: 0, hideEmpty: false, hidden: [], collapsed: [] }) : [["", "Rows"] as [string, string]];
    for (const [xk, xl] of xs) {
      if (c.hideEmpty && xk === "") continue;
      if (!xkeys.some((k) => k[0] === xk)) xkeys.push([xk, xl]);
      for (const [gk, gl] of gs) {
        if (!skeys.some((k) => k[0] === gk)) skeys.push([gk, gl]);
        if (!buckets.has(xk)) buckets.set(xk, new Map());
        const b = buckets.get(xk)!;
        if (!b.has(gk)) b.set(gk, []);
        b.get(gk)!.push(r);
      }
    }
  }
  if (xp && (!xp.options.length || c.xSort)) xkeys.sort((a, b) => (a[0] === "" ? 1 : b[0] === "" ? -1 : c.xSort === "desc" ? b[0].localeCompare(a[0]) : a[0].localeCompare(b[0], undefined, { numeric: true })));
  else if (xp && xp.options.length) xkeys.sort((a, b) => (a[0] === "" ? 1 : b[0] === "" ? -1 : xp.options.findIndex((o) => o.id === a[0]) - xp.options.findIndex((o) => o.id === b[0])));
  const calcName = c.yCalc || "count";
  const value = (list: MRow[]): number => {
    if (calcName === "count") return list.length;
    const vals = list.map((r) => (yp ? safeValue(db, r, yp) : null));
    const v = calc(calcName === "unique" ? "countUnique" : calcName, vals);
    return typeof v === "number" ? v : 0;
  };
  const series = skeys.map(([gk, gl], i) => {
    const color = gp?.options.find((o) => o.id === gk)?.color || palette[i % palette.length];
    return {
      name: gl, color,
      points: xkeys.map(([xk, xl]) => {
        const v = value(buckets.get(xk)?.get(gk) ?? []);
        return { label: xl, value: v, text: yp ? fmtNum(v, yp.numberFormat) : numStr(v), color: xp?.options.find((o) => o.id === xk)?.color || color };
      }),
    };
  });
  const total = c.kind === "number" ? (yp ? fmtNum(value(res._rows), yp.numberFormat) : numStr(value(res._rows))) : null;
  return { labels: xkeys.map((k) => k[1]), series, total, rows: res._rows.length };
}

// ---------------------------------------------------------------- database building blocks

const blankProp = (name: string, kind: PropKind, extra: Partial<Property> = {}): Property => ({
  id: nid(), name, kind, options: [], numberFormat: "number", numberDisplay: "", numberMax: 0, dateFormat: "full", formula: "", relationDb: "", relationBack: "",
  relationLimitOne: false, rollupRelation: "", rollupProp: "", rollupCalc: "", idPrefix: "", description: "", ...extra,
});
const opt = (name: string, color: string, group = "") => ({ id: nid(), name, color: color as never, group: group as never });
const blankView = (name: string, kind: View["kind"], extra: Partial<View> = {}): View => ({
  id: nid(), name, kind, filter: { op: "and", conds: [], groups: [] }, sorts: [], group: null, subGroup: null, columns: [], calcs: [], freeze: 1, wrap: false,
  rowNumbers: false, openIn: "full", cardSize: "medium", cardCover: "", cardFit: false, dateProp: "", endProp: "", calendarMode: "month", timelineZoom: "month",
  showDeps: true, showSubItems: true,
  chart: { kind: "bar", xProp: "", xBy: "month", xStep: 0, xSort: "", groupProp: "", yCalc: "count", yProp: "", stacked: false, horizontal: false, showLabels: true, hideEmpty: false, color: "", title: "" },
  ...extra,
});

function newDb(sectionId: string, title: string, parent = "", inline = false): Database {
  const t = Date.now();
  const db: Database = {
    id: nid(), sectionId, parent, inline, title, icon: "", description: "", order: dbs.length, favorite: false, locked: false,
    props: [blankProp("Name", "title", { id: "title" })], views: [blankView("Table", "table")], templates: [], defaultTemplate: "", nextId: 1,
    subParentProp: "", subChildProp: "", blockedByProp: "", blockingProp: "", trashedAt: null, created: t, updated: t,
  };
  return db;
}

function sanitize(db: Database) {
  const ids = db.props.map((p) => p.id);
  if (!db.props.some((p) => p.kind === "title")) db.props.unshift(blankProp("Name", "title", { id: "title" }));
  const ti = db.props.findIndex((p) => p.kind === "title");
  if (ti > 0) db.props.unshift(...db.props.splice(ti, 1));
  db.props[0].id = "title";
  for (const p of db.props) { if (!p.id) p.id = nid(); for (const o of p.options) if (!o.id) o.id = nid(); }
  if (!db.views.length) db.views.push(blankView("Table", "table"));
  for (const v of db.views) {
    if (!v.id) v.id = nid();
    const have = new Set<string>();
    v.columns = v.columns.filter((c) => db.props.some((p) => p.id === c.prop) && !have.has(c.prop) && have.add(c.prop));
    for (const p of db.props) if (!have.has(p.id)) v.columns.push({ prop: p.id, visible: true, width: 0 });
    v.sorts = v.sorts.filter((s) => db.props.some((p) => p.id === s.prop));
    v.calcs = v.calcs.filter((s) => db.props.some((p) => p.id === s.prop));
    if (v.group && !db.props.some((p) => p.id === v.group!.prop)) v.group = null;
    if (v.subGroup && (!v.group || !db.props.some((p) => p.id === v.subGroup!.prop))) v.subGroup = null;
  }
  // Like the backend: an action that changes a property the database does not have is dropped when it is saved.
  const dropBlank = (acts: { kind: string; prop: string }[] | undefined) => acts && acts.splice(0, acts.length, ...acts.filter((a) => a.kind !== "setProp" || ids.includes(a.prop)));
  for (const a of db.automations ?? []) dropBlank(a.actions);
  for (const p of db.props) dropBlank(p.actions);
}

function meta(d: Database) {
  return { id: d.id, sectionId: d.sectionId, parent: d.parent, inline: d.inline, title: d.title, icon: d.icon, favorite: d.favorite, locked: d.locked, order: d.order, rows: rowsOf(d.id).length, trashedAt: d.trashedAt, created: d.created, updated: d.updated };
}
export const databaseMetas = () => dbs.map(meta);

/** The real reorder command shares one order across pages and databases. */
export function reorderDatabases(ids: string[]) {
  ids.forEach((id, order) => {
    const db = dbs.find((d) => d.id === id);
    if (db) db.order = order;
  });
  dbs.sort((a, b) => a.order - b.order);
}

function newRow(db: Database, title: string): MRow {
  const t = Date.now();
  const hasId = db.props.some((p) => p.kind === "id");
  const r: MRow = { id: nid(), db: db.id, title, content: "", text: "", cells: {}, icon: "", cover: "", order: ++rowOrder, favorite: false, number: hasId ? db.nextId++ : 0, createdBy: "You", editedBy: "You", trashedAt: null, created: t, updated: t };
  rows.push(r);
  return r;
}

function cleanCell(p: Property, v: unknown): unknown | undefined {
  if (v === null) return undefined;
  switch (p.kind) {
    case "text": case "url": case "email": case "phone": return typeof v === "string" && v ? v : undefined;
    case "number": { const n = typeof v === "number" ? v : parseFloat(String(v)); if (!isFinite(n)) throw "That value does not fit the property \"" + p.name + "\"."; return n; }
    case "select": case "status": if (v === "") return undefined; if (!p.options.some((o) => o.id === v)) throw "That value does not fit the property \"" + p.name + "\"."; return v;
    case "multiSelect": { const a = (v as string[]); if (!a.every((id) => p.options.some((o) => o.id === id))) throw "That value does not fit the property \"" + p.name + "\"."; return a.length ? [...new Set(a)] : undefined; }
    case "date": { const o = v as { start?: string; end?: string; reminder?: number }; if (!parseDate(o.start)) throw "That value does not fit the property \"" + p.name + "\"."; if (o.end && parseDate(o.end)!.start < parseDate(o.start)!.start) throw "The end date is before the start date."; return { start: o.start, ...(o.end ? { end: o.end } : {}), ...(o.reminder != null ? { reminder: o.reminder } : {}) }; }
    case "person": { const a = (v as string[]).map((s) => s.trim()).filter(Boolean); return a.length ? a : undefined; }
    case "files": return (v as unknown[]).length ? v : undefined;
    case "checkbox": return !!v;
    case "relation": { const a = [...new Set(v as string[])]; for (const id of a) if (!rowById(id)) throw "A linked row no longer exists."; const x = p.relationLimitOne ? a.slice(0, 1) : a; return x.length ? x : undefined; }
    case "place": { const o = v as { address?: string; lat?: number | null; lng?: number | null }; if (!o.address && o.lat == null && o.lng == null) return undefined; return { address: o.address ?? "", lat: o.lat ?? null, lng: o.lng ?? null }; }
    case "title": throw "The title is set with the row's name.";
    default: throw `"${p.name}" is worked out by Notaya and cannot be typed in.`;
  }
}

function syncRelations(db: Database, row: MRow, old: Record<string, unknown> | null) {
  for (const p of db.props) {
    if (p.kind !== "relation" || !p.relationBack) continue;
    const before = (old?.[p.id] as string[] | undefined) ?? [];
    const after = (row.cells[p.id] as string[] | undefined) ?? [];
    for (const tid of [...before.filter((x) => !after.includes(x)), ...after.filter((x) => !before.includes(x))]) {
      const t = rowById(tid);
      if (!t) continue;
      const list = ((t.cells[p.relationBack] as string[] | undefined) ?? []).filter((x) => x !== row.id);
      if (after.includes(tid)) list.push(row.id);
      if (list.length) t.cells[p.relationBack] = list; else delete t.cells[p.relationBack];
    }
  }
}

// ---------------------------------------------------------------- seed

function seed() {
  const sectionId = "s1";
  const projects = newDb(sectionId, "Projects");
  projects.icon = "🗂️";
  projects.props[0].name = "Project";
  const pstatus = blankProp("Status", "status", { options: [opt("Planned", "gray", "todo"), opt("Active", "blue", "inProgress"), opt("Done", "green", "complete")] });
  projects.props.push(pstatus);
  const tasks = newDb(sectionId, "Tasks");
  tasks.icon = "✅";
  tasks.props[0].name = "Task";
  const status = blankProp("Status", "status", { options: [opt("Not started", "gray", "todo"), opt("In progress", "blue", "inProgress"), opt("Review", "orange", "inProgress"), opt("Done", "green", "complete")] });
  const due = blankProp("Due", "date", { dateFormat: "short" });
  const prio = blankProp("Priority", "select", { options: [opt("Low", "gray"), opt("Medium", "yellow"), opt("High", "red")] });
  const tags = blankProp("Tags", "multiSelect", { options: [opt("Design", "purple"), opt("Backend", "blue"), opt("Bug", "red"), opt("Docs", "brown"), opt("Research", "green")] });
  const done = blankProp("Done", "checkbox");
  const est = blankProp("Estimate", "number", { numberDisplay: "bar", numberMax: 13 });
  const cost = blankProp("Cost", "number", { numberFormat: "dollar" });
  const project = blankProp("Project", "relation", { relationDb: projects.id, relationLimitOne: true });
  const projBack = blankProp("Tasks", "relation", { relationDb: tasks.id, relationBack: project.id });
  project.relationBack = projBack.id;
  const score = blankProp("Score", "formula", { formula: 'prop("Estimate") * 10 + if(prop("Priority") == "High", 20, 0)' });
  const link = blankProp("Link", "url");
  const owner = blankProp("Owner", "person");
  const createdAt = blankProp("Created", "createdTime");
  const idp = blankProp("ID", "id", { idPrefix: "TASK" });
  tasks.props.push(status, due, prio, tags, done, est, cost, project, score, link, owner, createdAt, idp);
  projects.props.push(projBack);
  const projRoll = blankProp("Total estimate", "rollup", { rollupRelation: projBack.id, rollupProp: est.id, rollupCalc: "sum" });
  projects.props.push(projRoll);
  tasks.nextId = 1;
  const tableV = tasks.views[0];
  tableV.name = "All tasks";
  tableV.calcs = [{ prop: est.id, calc: "sum" }, { prop: done.id, calc: "percentChecked" }];
  const board = blankView("Board", "board", { group: { prop: status.id, sort: "manual", dateBy: "", numberStep: 0, hideEmpty: false, hidden: [], collapsed: [] } });
  const cal = blankView("Calendar", "calendar", { dateProp: due.id });
  const tl = blankView("Timeline", "timeline", { dateProp: due.id });
  const chart = blankView("Chart", "chart");
  chart.chart.xProp = status.id; chart.chart.groupProp = prio.id; chart.chart.kind = "bar";
  tasks.views.push(board, cal, tl, blankView("List", "list"), blankView("Gallery", "gallery"), chart);
  tasks.templates.push({ id: nid(), name: "Bug report", icon: "🐞", cells: JSON.stringify({ [prio.id]: prio.options[2].id, [tags.id]: [tags.options[2].id] }), content: "", text: "", repeat: "", repeatLast: 0 } as Template);
  projects.templates = [];
  for (const d of [projects, tasks]) { sanitize(d); dbs.push(d); }
  // rows
  const pr = ["Website relaunch", "Mobile app", "Research notes", "Office move"].map((t, i) => {
    const r = newRow(projects, t);
    r.cells[pstatus.id] = pstatus.options[i % 3].id;
    return r;
  });
  const names = [
    "Design the landing page", "Write the API", "Fix login bug", "Update the docs", "Interview users", "Set up CI", "Review pull requests", "Plan sprint",
    "Draft release notes", "Create icon set", "Migrate database", "Add dark mode", "Write tests", "Customer call", "Refactor search", "Optimize images",
    "Prepare demo", "Backup strategy", "Accessibility audit", "Translate strings", "Onboarding flow", "Pricing page", "Error tracking", "Sync engine", "Cleanup backlog",
  ];
  const day = (n: number) => isoOf(startOfDay(nowMin()) + n * DAY, false);
  names.forEach((n, i) => {
    const r = newRow(tasks, n);
    r.created = now0 - (30 - i) * 86400000;
    r.cells[status.id] = status.options[i % 4].id;
    if (i % 5 !== 4) r.cells[due.id] = { start: day(i - 8), ...(i % 6 === 0 ? { end: day(i - 5) } : {}) };
    r.cells[prio.id] = prio.options[i % 3].id;
    if (i % 3 !== 2) r.cells[tags.id] = [tags.options[i % 5].id, ...(i % 4 === 0 ? [tags.options[(i + 2) % 5].id] : [])];
    r.cells[done.id] = i % 4 === 3;
    r.cells[est.id] = [1, 2, 3, 5, 8, 13][i % 6];
    if (i % 2 === 0) r.cells[cost.id] = 100 * (i + 1) + 0.5;
    const p = pr[i % 4];
    r.cells[project.id] = [p.id];
    p.cells[projBack.id] = [...((p.cells[projBack.id] as string[]) ?? []), r.id];
    if (i % 7 === 0) r.cells[owner.id] = ["Mohamed"];
    if (i === 3) r.cells[link.id] = "https://example.com/docs";
  });
}

// ---------------------------------------------------------------- commands

type A = Record<string, unknown>;
export const NOT_HANDLED = Symbol("not handled");

function dbFull(id: unknown): Database {
  const d = dbById(String(id));
  if (!d) throw "Not found.";
  return d;
}
function rowFull(id: unknown): MRow {
  const r = rowById(String(id));
  if (!r) throw "Not found.";
  return r;
}
function clone<T>(x: T): T { return JSON.parse(JSON.stringify(x)); }
function treeNow(base: Record<string, unknown>) { return { ...base, databases: databaseMetas() }; }
function rowAsPage(r: MRow) {
  const d = dbById(r.db);
  return { id: r.id, sectionId: d?.sectionId ?? "", title: r.title, content: r.content, text: r.text, order: r.order, favorite: r.favorite, icon: r.icon, cover: r.cover, parent: r.db, trashedAt: r.trashedAt, created: r.created, updated: r.updated };
}

let firstDb = true;
let csvCount = 0;

/** Handles the database commands; returns NOT_HANDLED for anything else. `base` is the notes tree of the mock. */

// ---- automations and buttons (a small copy of src-tauri/src/commands/automation.rs, enough for the screens)
const notices: unknown[] = [];
function autoValue(p: Property, spec: string, src: MRow, cur: MRow | null): unknown {
  const s = spec.trim();
  if (s === "@clear") return null;
  switch (p.kind) {
    case "number": { if (s.startsWith("@add:")) return (Number(cur?.cells[p.id] ?? 0) || 0) + Number(s.slice(5)); return s === "" ? null : Number(s); }
    case "select": case "status": { const o = p.options.find((x) => x.name.toLowerCase() === s.toLowerCase()); if (!o) throw "\"" + spec + "\" does not fit \"" + p.name + "\""; return o.id; }
    case "checkbox": return s === "@toggle" ? cur?.cells[p.id] !== true : ["true", "yes", "1"].includes(s.toLowerCase());
    case "date": { const d = new Date(); if (s === "@now") return { start: d.toISOString().slice(0, 16) }; const m = /^@today([+-]\d+)?$/.exec(s); if (m) { d.setDate(d.getDate() + Number(m[1] ?? 0)); return { start: d.toISOString().slice(0, 10) }; } return { start: s }; }
    case "relation": return s === "@self" ? [src.id] : (() => { throw "bad"; })();
    case "person": case "multiSelect": return s.split(",").map((x) => x.trim()).filter(Boolean);
    default: return s.replace("{title}", src.title);
  }
}
function runActions(name: string, actions: any[], row: MRow, depth: number) {
  let left = 50;
  for (const a of actions) {
    if (left-- <= 0) break;
    try {
      const d = dbFull(row.db);
      if (a.kind === "setProp") {
        const old = clone(row.cells);
        if (a.prop === "title") row.title = String(a.value).replace("{title}", row.title);
        else { const p = d.props.find((x) => x.id === a.prop); if (!p) continue; const v = autoValue(p, a.value, row, row); if (v === null) delete row.cells[p.id]; else row.cells[p.id] = v; }
        syncRelations(d, row, old);
        if (depth < 3 && JSON.stringify(old) !== JSON.stringify(row.cells)) fireAuto(d, row, "changed", Object.keys({ ...old, ...row.cells }).filter((k) => JSON.stringify(old[k]) !== JSON.stringify(row.cells[k])), depth + 1);
      } else if (a.kind === "addRow") {
        const td = dbFull(a.database || row.db);
        const nr = newRow(td, String(a.title ?? "").replace("{title}", row.title));
        for (const c of a.cells ?? []) { const p = td.props.find((x) => x.id === c.prop); if (!p) continue; const v = autoValue(p, c.value, row, null); if (v !== null) nr.cells[p.id] = v; }
        syncRelations(td, nr, null);
        if (depth < 3) fireAuto(td, nr, "added", [], depth + 1);
      } else if (a.kind === "notify") {
        notices.push({ key: "auto:" + nid(), rowId: row.id, title: String(a.title || name).replace("{title}", row.title), databaseId: d.id, databaseTitle: d.title, when: "", message: name });
      } else if (a.kind === "trash") row.trashedAt = Date.now();
    } catch (e) {
      notices.push({ key: "auto:" + nid(), rowId: row.id, title: "An action did not run: " + String(e), databaseId: row.db, databaseTitle: "", when: "", message: name });
    }
  }
}
function fireAuto(d: Database, row: MRow, ev: "added" | "changed", changed: string[], depth = 0) {
  for (const a of ((d as any).automations ?? []) as any[]) {
    if (!a.enabled) continue;
    const hit = (a.trigger.kind === "rowAdded" && ev === "added") || (a.trigger.kind === "propChanged" && ev === "changed" && (!a.trigger.prop || changed.includes(a.trigger.prop)));
    if (hit) runActions(a.name, a.actions, row, depth);
  }
}

export function databaseCommand(cmd: string, args: A, base: Record<string, unknown>, options?: { headers?: Record<string, string> }): unknown {
  switch (cmd) {
    case "list_notes": return treeNow(base);
    case "create_database": {
      const t = args.title ? String(args.title) : "Untitled database";
      const d = newDb(String(args.sectionId), t, String(args.parent ?? ""), !!args.inline);
      starter(d, String(args.starterKind));
      sanitize(d);
      dbs.push(d);
      const notice = firstDb ? "Notaya made a backup copy first. After this, an older Notaya (V1.4.0 or earlier) cannot open this vault." : null;
      firstDb = false;
      return { database: clone(d), notice };
    }
    case "get_database": return clone(dbFull(args.id));
    case "save_database": {
      const inc = JSON.parse(String(args.databaseJson)) as Database;
      const old = dbFull(inc.id);
      if (old.locked && inc.locked) throw "This database is locked. Unlock it to change its properties.";
      inc.created = old.created; inc.sectionId = old.sectionId; inc.parent = old.parent; inc.inline = old.inline; inc.order = old.order; inc.favorite = old.favorite; inc.trashedAt = old.trashedAt; inc.nextId = old.nextId;
      for (const p of inc.props) {
        const op = old.props.find((x) => x.id === p.id);
        if (op && op.kind !== p.kind) throw "A property's type cannot be changed here.";
        if (!p.id) p.id = nid();
        if (p.kind === "relation" && p.relationBack === "+") {
          const target = p.relationDb === inc.id ? inc : dbById(p.relationDb);
          if (!target) throw `The relation "${p.name}" needs a database to link to.`;
          const back = blankProp(`Related to ${inc.title || "Untitled"} (${p.name})`, "relation", { relationDb: inc.id, relationBack: p.id });
          p.relationBack = back.id;
          target.props.push(back);
          if (target !== inc) { sanitize(target); target.updated = Date.now(); }
        }
        if (p.kind === "relation" && !dbById(p.relationDb) && p.relationDb !== inc.id) throw `The relation "${p.name}" needs a database to link to.`;
      }
      const removed = old.props.filter((p) => !inc.props.some((q) => q.id === p.id));
      for (const rm of removed) for (const r of rows.filter((x) => x.db === inc.id)) delete r.cells[rm.id];
      if (inc.props.some((p) => p.kind === "id") && !old.props.some((p) => p.kind === "id")) {
        let n = inc.nextId;
        for (const r of rowsOf(inc.id).sort((a, b) => a.created - b.created)) if (!r.number) r.number = n++;
        inc.nextId = n;
      }
      inc.updated = Date.now();
      sanitize(inc);
      dbs[dbs.findIndex((d) => d.id === inc.id)] = inc;
      return clone(inc);
    }
    case "query_view": {
      const req = args.request as { databaseId: string; viewId: string; view?: View };
      const d = dbFull(req.databaseId);
      const v = req.view ?? d.views.find((x) => x.id === req.viewId) ?? d.views[0];
      const { _rows, ...res } = runView(d, v);
      void _rows;
      return res;
    }
    case "chart_data": {
      const req = args.request as { databaseId: string; viewId: string; view?: View };
      const d = dbFull(req.databaseId);
      return chartData(d, req.view ?? d.views.find((x) => x.id === req.viewId) ?? d.views[0]);
    }
    case "create_row": {
      const d = dbFull(args.databaseId);
      if (d.locked && false) throw "locked";
      const r = newRow(d, String(args.title ?? ""));
      const tid = args.templateId === "-" ? "" : args.templateId ? String(args.templateId) : d.defaultTemplate;
      const t = d.templates.find((x) => x.id === tid);
      if (t) { try { r.cells = JSON.parse(t.cells || "{}"); } catch { /* ignore */ } r.content = t.content; r.text = t.text; if (t.icon) r.icon = t.icon; }
      const given = JSON.parse(String(args.cells || "{}")) as A;
      for (const [k, v] of Object.entries(given)) {
        const p = d.props.find((x) => x.id === k);
        if (!p) throw "Not found.";
        const c = cleanCell(p, v);
        if (c === undefined) delete r.cells[k]; else r.cells[k] = c;
      }
      syncRelations(d, r, null);
      fireAuto(d, r, "added", []);
      memo.clear();
      return { row: rowOut(d, r) };
    }
    case "press_button": {
      const r = rowFull(args.rowId);
      const d = dbFull(r.db);
      const p = d.props.find((x) => x.id === args.propId && x.kind === "button");
      if (!p) throw "Not found.";
      runActions(p.name, (p as any).actions ?? [], r, 0);
      memo.clear();
      return { row: rowOut(d, r) };
    }
    case "due_reminders_now": return notices.splice(0);
    case "set_cells": {
      const r = rowFull(args.rowId);
      const d = dbFull(r.db);
      const old = clone(r.cells);
      const patch = JSON.parse(String(args.patch || "{}")) as A;
      for (const [k, v] of Object.entries(patch)) {
        const p = d.props.find((x) => x.id === k);
        if (!p) throw "Not found.";
        const c = cleanCell(p, v);
        if (c === undefined) delete r.cells[k]; else r.cells[k] = c;
      }
      if (args.title != null) r.title = String(args.title);
      r.updated = Date.now(); r.editedBy = "You";
      syncRelations(d, r, old);
      fireAuto(d, r, "changed", Object.keys(patch).filter((k) => JSON.stringify(old[k]) !== JSON.stringify(r.cells[k])));
      memo.clear();
      return { row: rowOut(d, r) };
    }
    case "get_row": { const r = rowFull(args.rowId); memo.clear(); return { row: rowOut(dbFull(r.db), r) }; }
    case "duplicate_row": {
      const s = rowFull(args.rowId);
      const d = dbFull(s.db);
      const r = newRow(d, s.title ? s.title + " (copy)" : "");
      r.content = s.content; r.text = s.text; r.icon = s.icon; r.cover = s.cover;
      r.cells = clone(s.cells);
      for (const p of d.props) if (["formula", "rollup", "id", "createdTime", "createdBy", "lastEditedTime", "lastEditedBy"].includes(p.kind)) delete r.cells[p.id];
      syncRelations(d, r, null);
      memo.clear();
      return { row: rowOut(d, r) };
    }
    case "reorder_rows": { (args.ids as string[]).forEach((id, i) => { const r = rowById(id); if (r) r.order = i + 1; }); rowOrder = Math.max(rowOrder, (args.ids as string[]).length); return null; }
    case "search_rows": {
      const q = String(args.query ?? "").toLowerCase();
      return rowsOf(String(args.databaseId)).filter((r) => r.title.toLowerCase().includes(q)).slice(0, Number(args.limitN ?? 50)).map((r) => ({ id: r.id, title: r.title, icon: r.icon }));
    }
    case "row_titles": return (args.ids as string[]).map((id) => rowById(id)).filter((r): r is MRow => !!r).map((r) => ({ id: r.id, title: r.title, icon: r.icon }));
    case "set_database_look": {
      const d = dbFull(args.id);
      d.title = String(args.title); d.icon = String(args.icon); d.favorite = !!args.favorite; d.locked = !!args.locked; d.updated = Date.now();
      return meta(d);
    }
    case "move_database": { dbFull(args.id).sectionId = String(args.sectionId); return null; }
    case "enable_sub_items": {
      const d = dbFull(args.id);
      if (!d.subParentProp) {
        const child = blankProp("Sub-items", "relation", { relationDb: d.id });
        const parent = blankProp("Parent item", "relation", { relationDb: d.id, relationBack: child.id, relationLimitOne: true });
        child.relationBack = parent.id;
        d.props.push(parent, child); d.subParentProp = parent.id; d.subChildProp = child.id; sanitize(d);
      }
      return clone(d);
    }
    case "enable_dependencies": {
      const d = dbFull(args.id);
      if (!d.blockedByProp) {
        const blocking = blankProp("Blocking", "relation", { relationDb: d.id });
        const by = blankProp("Blocked by", "relation", { relationDb: d.id, relationBack: blocking.id });
        blocking.relationBack = by.id;
        d.props.push(by, blocking); d.blockedByProp = by.id; d.blockingProp = blocking.id; sanitize(d);
      }
      return clone(d);
    }
    case "trash_page": case "restore_page": case "delete_page": {
      const id = String(args.id);
      const t = cmd === "trash_page" ? Date.now() : null;
      const d = dbById(id);
      if (d) {
        if (cmd === "delete_page") { dbs.splice(dbs.indexOf(d), 1); for (let i = rows.length - 1; i >= 0; i--) if (rows[i].db === id) rows.splice(i, 1); } else d.trashedAt = t;
        return treeNow(base);
      }
      const r = rowById(id);
      if (r) {
        if (cmd === "delete_page") rows.splice(rows.indexOf(r), 1); else r.trashedAt = t;
        return treeNow(base);
      }
      return NOT_HANDLED;
    }
    case "get_page": { const r = rowById(String(args.id)); return r ? rowAsPage(r) : NOT_HANDLED; }
    case "save_page": {
      const r = rowById(String(args.id));
      if (!r) return NOT_HANDLED;
      r.title = String(args.title); r.content = String(args.content); r.text = String(args.text); r.updated = Date.now();
      const p = rowAsPage(r);
      return { ...p, preview: r.text.slice(0, 80), words: r.text.split(/\s+/).filter(Boolean).length };
    }
    case "set_page_look": { const r = rowById(String(args.id)); if (!r) return NOT_HANDLED; r.icon = String(args.icon); r.cover = String(args.cover); return rowAsPage(r); }
    case "set_page_favorite": { const r = rowById(String(args.id)); if (r) { r.favorite = !!args.favorite; return treeNow(base); } const d = dbById(String(args.id)); if (d) { d.favorite = !!args.favorite; return treeNow(base); } return NOT_HANDLED; }
    case "move_page": { const d = dbById(String(args.id)); if (d) { d.sectionId = String(args.sectionId); return treeNow(base); } return NOT_HANDLED; }
    case "search": {
      const q = String(args.query ?? "").toLowerCase();
      if (!q) return [];
      const hits: unknown[] = [];
      for (const d of dbs) if (!d.trashedAt && d.title.toLowerCase().includes(q)) hits.push({ kind: "database", id: d.id, title: d.title, subtitle: "Database", snippet: "", score: 2 });
      for (const r of rows) if (!r.trashedAt && r.title.toLowerCase().includes(q)) hits.push({ kind: "row", id: r.id, title: r.title, subtitle: dbById(r.db)?.title ?? "", snippet: "", score: 1 });
      return hits.slice(0, 20);
    }
    case "empty_trash": {
      for (let i = dbs.length - 1; i >= 0; i--) if (dbs[i].trashedAt) dbs.splice(i, 1);
      for (let i = rows.length - 1; i >= 0; i--) if (rows[i].trashedAt) rows.splice(i, 1);
      return treeNow(base);
    }
    case "import_csv": {
      const dbId = args.databaseId as string | null;
      const csv = [["Name", "Status", "Amount"], ["Alpha", "Open", "10"], ["Beta", "Closed", "25"], ["Gamma", "Open", "7"]];
      let d = dbId ? dbFull(dbId) : null;
      let created = false;
      const skipped: string[] = [];
      if (!d) {
        d = newDb("s1", `Imported CSV ${++csvCount}`);
        d.props[0].name = "Name";
        d.props.push(blankProp("Status", "select"), blankProp("Amount", "number"));
        sanitize(d); dbs.push(d); created = true;
      } else skipped.push("Column \"Amount\" was skipped: it is a formula.");
      const stp = d.props.find((p) => p.name === "Status");
      for (const line of csv.slice(1)) {
        const r = newRow(d, line[0]);
        if (stp) {
          let o = stp.options.find((x) => x.name === line[1]);
          if (!o && stp.kind === "select") { o = opt(line[1], "gray"); stp.options.push(o); }
          if (o) r.cells[stp.id] = o.id;
        }
        const am = d.props.find((p) => p.name === "Amount" && p.kind === "number");
        if (am) r.cells[am.id] = Number(line[2]);
      }
      return { databaseId: d.id, created, rows: csv.length - 1, skipped };
    }
    case "export_csv": { const d = dbFull(args.databaseId); return { path: `C:\\Users\\you\\Documents\\${d.title}.csv`, rows: rowsOf(d.id).length }; }
    case "attach_file": {
      const h = options?.headers ?? {};
      const r = rowFull(h["x-row-id"]);
      const d = dbFull(r.db);
      const p = d.props.find((x) => x.id === h["x-prop-id"]);
      if (!p || p.kind !== "files") throw "Not found.";
      const size = (args as unknown as Uint8Array).length ?? 0;
      if (size > 50 * 1024 * 1024) throw "That file is larger than 50 MB.";
      const name = decodeURIComponent(h["x-file-name"] ?? "file");
      const mime = /\.(png|jpe?g|gif|webp)$/i.test(name) ? "image/" + name.split(".").pop()!.toLowerCase().replace("jpg", "jpeg") : "application/octet-stream";
      const f = { id: hex(), name, mime, size };
      files.set(f.id, f);
      r.cells[p.id] = [...((r.cells[p.id] as unknown[]) ?? []), f];
      return r.cells[p.id];
    }
    case "remove_file": {
      const r = rowFull(args.rowId);
      const list = ((r.cells[String(args.propId)] as { id: string }[]) ?? []).filter((f) => f.id !== args.fileId);
      if (list.length) r.cells[String(args.propId)] = list; else delete r.cells[String(args.propId)];
      return list;
    }
    case "open_file": return null;
    case "save_file_as": return "C:\\Users\\you\\Downloads\\file";
  }
  return NOT_HANDLED;
}

function starter(d: Database, kind: string) {
  const sel = (n: string, o: [string, string, string?][], k: PropKind = "status") => blankProp(n, k, { options: o.map(([a, b, c]) => opt(a, b, c ?? "")) });
  if (kind === "tasks") {
    d.props[0].name = "Task";
    const status = sel("Status", [["Not started", "gray", "todo"], ["In progress", "blue", "inProgress"], ["Done", "green", "complete"]]);
    const due = blankProp("Due", "date");
    d.props.push(status, due, sel("Priority", [["Low", "gray"], ["Medium", "yellow"], ["High", "red"]], "select"), blankProp("Done", "checkbox"));
    d.views.push(blankView("Board", "board", { group: { prop: status.id, sort: "manual", dateBy: "", numberStep: 0, hideEmpty: false, hidden: [], collapsed: [] } }), blankView("Calendar", "calendar", { dateProp: due.id }));
  } else if (kind === "reading") {
    d.props[0].name = "Title";
    d.props.push(blankProp("Author", "text"), sel("Status", [["To read", "gray", "todo"], ["Reading", "blue", "inProgress"], ["Finished", "green", "complete"]]), blankProp("Rating", "number", { numberDisplay: "ring", numberMax: 5 }), sel("Tags", [["Fiction", "purple"], ["Non-fiction", "orange"], ["Reference", "brown"]], "multiSelect"));
    d.views.push(blankView("Gallery", "gallery"));
  } else if (kind === "projects") {
    d.props[0].name = "Project";
    const dates = blankProp("Dates", "date");
    d.props.push(sel("Status", [["Planned", "gray", "todo"], ["Active", "blue", "inProgress"], ["Done", "green", "complete"]]), dates, blankProp("Notes", "text"));
    d.views.push(blankView("Timeline", "timeline", { dateProp: dates.id }));
  }
}

if (params.get("db") === "1") { seed(); firstDb = false; }


// C2's mock uses the same hierarchy for inherited bins and container cleanup.
export function coordinationItems() { return [...dbs.map((d) => ({ ...d, kind: "database" })), ...rows.map((r) => ({ ...r, kind: "row" }))]; }
export function removeCoordinatedItems(ids: Set<string>) {
  for (let i = dbs.length - 1; i >= 0; i--) if (ids.has(dbs[i].id)) dbs.splice(i, 1);
  for (let i = rows.length - 1; i >= 0; i--) if (ids.has(rows[i].id)) rows.splice(i, 1);
}
