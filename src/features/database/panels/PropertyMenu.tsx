// Property settings (rename, options, number and date formats, formula, relation, rollup), the formula editor and the "new property" picker.

import { useEffect, useMemo, useRef, useState } from "react";
import { ArrowDownAZ, ArrowUpAZ, Copy, EyeOff, Plus, Trash2, X } from "lucide-react";
import type { Database, DatabaseMeta, OptionColor, PropKind, Property, SelectOption } from "@/lib/database-types";
import { cn } from "@/lib/utils";
import { KindIcon } from "../icons";
import {
  COLOR_NAMES, DATE_FORMATS, KINDS, KIND_ORDER, NUMBER_FORMATS, OPTION_COLORS, STATUS_GROUPS, blankOption, blankProperty, calcsFor, CALC_LABELS, colorForNew, hasOptions,
  reorderList,
} from "../model";
import { loadDatabase, useDatabase } from "../dbStore";
import ActionsEditor from "./ActionsEditor";
import { PSelect, PanelTitle, PropSelect, Row, SortableList, SortableRow, SwitchRow, fieldCls } from "./ui";
import { Popover } from "radix-ui";
import { optClass, optDot } from "../model";

/* ---------- options ---------- */

function ColorDot({ color, onChange }: { color: OptionColor; onChange: (c: OptionColor) => void }) {
  return (
    <Popover.Root>
      <Popover.Trigger asChild>
        <button type="button" aria-label={`Color: ${COLOR_NAMES[color]}`} className="grid h-6 w-6 shrink-0 place-items-center rounded hover:bg-surface-2">
          <span className="h-3.5 w-3.5 rounded-full" style={{ background: optDot(color) }} />
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content align="start" sideOffset={4} className="z-[90] w-40 rounded-lg border border-line bg-elevated p-1 shadow-float">
          {OPTION_COLORS.map((c) => (
            <button key={c || "default"} type="button" onClick={() => onChange(c)} className="flex w-full items-center gap-2 rounded-md px-2 py-1 text-sm hover:bg-surface-2">
              <span className="h-3.5 w-3.5 rounded-full" style={{ background: optDot(c) }} />
              {COLOR_NAMES[c]}
              {c === color && <span className="ms-auto text-faint">✓</span>}
            </button>
          ))}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

export function OptionsEditor({ prop, onChange }: { prop: Property; onChange: (opts: SelectOption[]) => void }) {
  const [draft, setDraft] = useState("");
  const status = prop.kind === "status";
  const setOpt = (id: string, patch: Partial<SelectOption>) => onChange(prop.options.map((o) => (o.id === id ? { ...o, ...patch } : o)));
  const add = (group: SelectOption["group"] = status ? "todo" : "") => {
    const n = draft.trim();
    if (!n) return;
    onChange([...prop.options, blankOption(n, colorForNew(prop.options.length), group)]);
    setDraft("");
  };
  return (
    <div>
      <PanelTitle>Options</PanelTitle>
      <SortableList ids={prop.options.map((o) => o.id)} onReorder={(f, t) => onChange(reorderList(prop.options, f, t))}>
        {prop.options.map((o) => (
          <SortableRow key={o.id} id={o.id}>
            {(handle) => (
              <div className="flex items-center gap-1 py-0.5">
                {handle}
                <ColorDot color={o.color} onChange={(c) => setOpt(o.id, { color: c })} />
                <input
                  aria-label="Option name"
                  dir="auto"
                  value={o.name}
                  onChange={(e) => setOpt(o.id, { name: e.target.value })}
                  className={cn(fieldCls, "h-7 min-w-0 flex-1")}
                />
                {status && (
                  <PSelect value={o.group || "todo"} label="Status group" className="w-[7.5rem]" onChange={(v) => setOpt(o.id, { group: v as SelectOption["group"] })}>
                    {STATUS_GROUPS.map((g) => (
                      <option key={g.id} value={g.id}>{g.label}</option>
                    ))}
                  </PSelect>
                )}
                <button type="button" aria-label={`Delete ${o.name}`} onClick={() => onChange(prop.options.filter((x) => x.id !== o.id))} className="rounded p-1 text-faint hover:text-danger">
                  <X size={14} />
                </button>
              </div>
            )}
          </SortableRow>
        ))}
      </SortableList>
      <div className="mt-1 flex items-center gap-1.5">
        <input
          aria-label="New option"
          dir="auto"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && (e.preventDefault(), add())}
          placeholder="Add an option"
          className={cn(fieldCls, "h-7 flex-1")}
        />
        <button type="button" onClick={() => add()} className="rounded-md p-1.5 text-muted hover:bg-surface-2 hover:text-fg" aria-label="Add option">
          <Plus size={15} />
        </button>
      </div>
      {prop.options.length > 0 && (
        <div className="mt-2 flex flex-wrap gap-1">
          {prop.options.map((o) => (
            <span key={o.id} className={optClass(o.color)}><span>{o.name || "…"}</span></span>
          ))}
        </div>
      )}
    </div>
  );
}

/* ---------- formula ---------- */

const FORMULA_HELP: { group: string; items: [string, string, string][] }[] = [
  { group: "Logic", items: [
    ["if", 'if(condition, then, else)', "Chooses a value. Also: condition ? a : b"],
    ["ifs", 'ifs(c1, v1, c2, v2, else)', "The first true condition wins"],
    ["and / or / not", 'a and b, a or b, not a', "Combine true/false values"],
    ["empty", 'empty(value)', "True when a value is blank"],
  ] },
  { group: "Text", items: [
    ["concat", 'concat("a", "b")', "Joins text (or use +)"],
    ["join", 'join(list, ", ")', "Joins a list into text"],
    ["length", 'length(text)', "Number of characters (or list items)"],
    ["lower / upper / trim", 'upper(text)', "Change case, trim spaces"],
    ["replace", 'replace(text, "old", "new")', "Replace the first match (replaceAll: all)"],
    ["substring", 'substring(text, 0, 3)', "Part of a text"],
    ["contains", 'contains(text, "x")', "True if the text has it"],
    ["format", 'format(value)', "Any value as text"],
  ] },
  { group: "Numbers", items: [
    ["round / floor / ceil", 'round(n, 2)', "Rounding"],
    ["abs / sqrt / sign", 'abs(n)', "Absolute value, root, sign"],
    ["min / max / sum / mean / median", 'max(1, 2, 3)', "Of several numbers or a list"],
    ["pow / mod", '2 ^ 3, 7 % 2', "Power and remainder"],
    ["toNumber", 'toNumber("12")', "Text to number"],
  ] },
  { group: "Dates", items: [
    ["now / today", 'now()', "Current date and time"],
    ["dateBetween", 'dateBetween(end, start, "days")', "Difference in days, weeks, months…"],
    ["dateAdd / dateSubtract", 'dateAdd(date, 7, "days")', "Move a date"],
    ["formatDate", 'formatDate(date, "MMM D, YYYY")', "A date as text"],
    ["year / month / date / day", 'year(date)', "Parts of a date"],
    ["start / end", 'start(range)', "Ends of a date range"],
  ] },
  { group: "Lists", items: [
    ["first / last / at", 'first(list)', "Pick an item"],
    ["count", 'count(list)', "Number of items"],
    ["sort / reverse / unique", 'sort(list)', "Order and clean a list"],
    ["map / filter", 'filter(list, current > 1)', "Use current and index inside"],
    ["some / every / includes", 'includes(list, x)', "Test items"],
  ] },
  { group: "Other", items: [
    ["prop", 'prop("Name")', "The value of another property"],
    ["let", 'let(x, 2, x * 3)', "Name a value"],
  ] },
];

function quickCheck(src: string): string | null {
  let depth = 0;
  let quote = "";
  for (let i = 0; i < src.length; i++) {
    const c = src[i];
    if (quote) {
      if (c === "\\") i++;
      else if (c === quote) quote = "";
    } else if (c === '"' || c === "'") quote = c;
    else if (c === "(") depth++;
    else if (c === ")" && --depth < 0) return "There is an extra )";
  }
  if (quote) return "A text is not closed";
  if (depth > 0) return "A ( is not closed";
  return null;
}

export function FormulaEditor({ prop, db, sample, onChange }: { prop: Property; db: Database; sample?: { t: string; text: string } | null; onChange: (formula: string) => void }) {
  const ta = useRef<HTMLTextAreaElement>(null);
  const [help, setHelp] = useState(false);
  const local = useMemo(() => quickCheck(prop.formula), [prop.formula]);
  const error = local ?? (sample?.t === "error" ? sample.text.replace(/^⚠\s*/, "") : null);
  const insert = (text: string) => {
    const el = ta.current;
    const v = prop.formula;
    const a = el?.selectionStart ?? v.length;
    const b = el?.selectionEnd ?? v.length;
    onChange(v.slice(0, a) + text + v.slice(b));
    requestAnimationFrame(() => {
      el?.focus();
      el?.setSelectionRange(a + text.length, a + text.length);
    });
  };
  return (
    <div>
      <PanelTitle>Formula</PanelTitle>
      <textarea
        ref={ta}
        value={prop.formula}
        dir="ltr"
        spellCheck={false}
        onChange={(e) => onChange(e.target.value)}
        aria-label="Formula"
        placeholder='prop("Price") * 2'
        rows={4}
        className={cn(fieldCls, "h-auto w-full resize-y py-1.5 font-mono text-[0.8rem]", error && "border-danger focus:border-danger")}
      />
      <div className={cn("mt-1 min-h-[1.25rem] text-xs", error ? "text-danger" : "text-faint")} role="status">
        {error ? error : sample ? <>Preview (first row): <span className="text-fg" dir="auto">{sample.text || "(empty)"}</span></> : "Write a formula; the first row shows its result."}
      </div>
      <div className="mt-1.5 flex flex-wrap gap-1">
        {db.props.filter((p) => p.id !== prop.id).slice(0, 12).map((p) => (
          <button key={p.id} type="button" onClick={() => insert(`prop("${p.name}")`)} className="inline-flex items-center gap-1 rounded-md bg-surface-2 px-1.5 py-0.5 text-xs hover:bg-line" title="Insert this property">
            <KindIcon kind={p.kind} size={11} className="text-muted" /> <span dir="auto" className="max-w-[8rem] truncate">{p.name}</span>
          </button>
        ))}
      </div>
      <button type="button" onClick={() => setHelp((h) => !h)} className="mt-2 rounded-md px-1 text-xs text-accent-strong underline-offset-2 hover:underline" aria-expanded={help}>
        {help ? "Hide functions" : "Show functions and help"}
      </button>
      {help && (
        <div className="mt-1.5 max-h-56 overflow-y-auto rounded-lg border border-line bg-surface p-2 text-xs">
          <p className="pb-1.5 text-muted">Operators: + - * / % ^ == != &lt; &gt; &lt;= &gt;= and or not, and a ? b : c. Click an example to insert it.</p>
          {FORMULA_HELP.map((g) => (
            <div key={g.group} className="pb-1.5">
              <div className="font-semibold text-muted">{g.group}</div>
              {g.items.map(([name, example, desc]) => (
                <button key={name} type="button" onClick={() => insert(example)} className="block w-full rounded px-1 py-0.5 text-start hover:bg-surface-2">
                  <span className="font-mono text-fg">{name}</span> <span className="text-faint">{desc}</span>
                  <div className="font-mono text-[0.72rem] text-accent-strong">{example}</div>
                </button>
              ))}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

/* ---------- rollup ---------- */

function RollupEditor({ db, prop, onChange }: { db: Database; prop: Property; onChange: (p: Property) => void }) {
  const rels = db.props.filter((p) => p.kind === "relation");
  const rel = rels.find((p) => p.id === prop.rollupRelation);
  const target = useDatabase(rel?.relationDb ?? "");
  useEffect(() => {
    if (rel && !target) void loadDatabase(rel.relationDb).catch(() => undefined);
  }, [rel, target]);
  const tp = target?.props.find((p) => p.id === prop.rollupProp);
  const calcs = calcsFor(tp?.kind ?? "text", true);
  return (
    <div>
      <PanelTitle>Rollup</PanelTitle>
      <Row label="Relation">
        <PropSelect props={rels} value={prop.rollupRelation} label="Relation" className="w-[11rem]" onChange={(v) => onChange({ ...prop, rollupRelation: v, rollupProp: "" })} />
      </Row>
      <Row label="Property">
        <PropSelect props={target?.props ?? []} value={prop.rollupProp} label="Rolled up property" className="w-[11rem]" onChange={(v) => onChange({ ...prop, rollupProp: v })} />
      </Row>
      <Row label="Calculate">
        <PSelect value={prop.rollupCalc || "showOriginal"} label="Calculation" className="w-[11rem]" onChange={(v) => onChange({ ...prop, rollupCalc: v })}>
          {calcs.map((c) => (
            <option key={c} value={c}>{CALC_LABELS[c]}</option>
          ))}
        </PSelect>
      </Row>
      {rels.length === 0 && <p className="pt-1 text-xs text-faint">Add a relation property first.</p>}
    </div>
  );
}

/* ---------- the menu ---------- */

interface MenuProps {
  db: Database;
  prop: Property;
  databases: DatabaseMeta[];
  locked: boolean;
  sample?: { t: string; text: string } | null;
  onChange: (p: Property) => void;
  onDelete: () => void;
  onDuplicate: () => void;
  onHide?: () => void;
  onSort?: (descending: boolean) => void;
}

export function PropertyMenu({ db, prop, databases, locked, sample, onChange, onDelete, onDuplicate, onHide, onSort }: MenuProps) {
  const isTitle = prop.kind === "title";
  const set = (patch: Partial<Property>) => onChange({ ...prop, ...patch });
  const target = databases.find((d) => d.id === prop.relationDb);
  return (
    <div className="space-y-3">
      <div>
        <div className="flex items-center gap-2">
          <KindIcon kind={prop.kind} size={16} className="shrink-0 text-muted" />
          <input
            aria-label="Property name"
            dir="auto"
            value={prop.name}
            disabled={locked}
            onChange={(e) => set({ name: e.target.value })}
            className={cn(fieldCls, "h-9 flex-1 font-medium")}
          />
        </div>
        <div className="mt-1 ps-6 text-xs text-faint">Type: {KINDS[prop.kind].label}{prop.kind === "relation" && target ? ` → ${target.title || "Untitled"}` : ""}</div>
      </div>
      {!isTitle && (
        <input aria-label="Description" dir="auto" value={prop.description} disabled={locked} onChange={(e) => set({ description: e.target.value })} placeholder="Add a description" className={cn(fieldCls, "w-full")} />
      )}

      {hasOptions(prop.kind) && !locked && <OptionsEditor prop={prop} onChange={(options) => set({ options })} />}
      {prop.kind === "number" && (
        <div>
          <Row label="Format">
            <PSelect value={prop.numberFormat || "number"} label="Number format" className="w-[11rem]" onChange={(v) => set({ numberFormat: v })}>
              {NUMBER_FORMATS.map(([k, l]) => (
                <option key={k} value={k}>{l}</option>
              ))}
            </PSelect>
          </Row>
          <Row label="Show as">
            <PSelect value={prop.numberDisplay} label="Show number as" className="w-[11rem]" onChange={(v) => set({ numberDisplay: v })}>
              <option value="">Number</option>
              <option value="bar">Bar</option>
              <option value="ring">Ring</option>
            </PSelect>
          </Row>
          {prop.numberDisplay && (
            <Row label="Full at">
              <input type="number" aria-label="Value that fills the bar or ring" value={prop.numberMax || ""} placeholder="100" onChange={(e) => set({ numberMax: Number(e.target.value) || 0 })} className={cn(fieldCls, "w-[11rem]")} />
            </Row>
          )}
        </div>
      )}
      {(prop.kind === "date" || prop.kind === "createdTime" || prop.kind === "lastEditedTime") && (
        <Row label="Date format">
          <PSelect value={prop.dateFormat || "full"} label="Date format" className="w-[13rem]" onChange={(v) => set({ dateFormat: v })}>
            {DATE_FORMATS.map(([k, l]) => (
              <option key={k} value={k}>{l}</option>
            ))}
          </PSelect>
        </Row>
      )}
      {prop.kind === "formula" && <FormulaEditor prop={prop} db={db} sample={sample} onChange={(formula) => set({ formula })} />}
      {prop.kind === "formula" && (
        <Row label="Number format">
          <PSelect value={prop.numberFormat || "number"} label="Formula number format" className="w-[11rem]" onChange={(v) => set({ numberFormat: v })}>
            {NUMBER_FORMATS.map(([k, l]) => (
              <option key={k} value={k}>{l}</option>
            ))}
          </PSelect>
        </Row>
      )}
      {prop.kind === "rollup" && <RollupEditor db={db} prop={prop} onChange={onChange} />}
      {prop.kind === "relation" && (
        <div>
          <Row label="Related database">
            <span className="text-sm">{target?.title || (prop.relationDb === db.id ? db.title : "Untitled")}</span>
          </Row>
          <SwitchRow label="Limit to one row" checked={prop.relationLimitOne} onChange={(v) => set({ relationLimitOne: v })} />
        </div>
      )}
      {prop.kind === "button" && (
        <div className="space-y-2">
          <Row label="Label">
            <input aria-label="Button label" dir="auto" value={prop.buttonLabel} maxLength={60} placeholder={prop.name} onChange={(e) => set({ buttonLabel: e.target.value })} className={cn(fieldCls, "w-[11rem]")} />
          </Row>
          <PanelTitle>When pressed</PanelTitle>
          <ActionsEditor db={db} actions={prop.actions} onChange={(actions) => set({ actions })} />
        </div>
      )}
      {prop.kind === "id" && (
        <Row label="Prefix">
          <input aria-label="ID prefix" value={prop.idPrefix} maxLength={20} placeholder="TASK" onChange={(e) => set({ idPrefix: e.target.value })} className={cn(fieldCls, "w-[11rem]")} />
        </Row>
      )}

      <div className="border-t border-line pt-1.5">
        {onSort && (
          <>
            <MenuButton icon={<ArrowUpAZ size={14} />} onClick={() => onSort(false)}>Sort ascending</MenuButton>
            <MenuButton icon={<ArrowDownAZ size={14} />} onClick={() => onSort(true)}>Sort descending</MenuButton>
          </>
        )}
        {onHide && !isTitle && <MenuButton icon={<EyeOff size={14} />} onClick={onHide}>Hide in view</MenuButton>}
        {!isTitle && !locked && <MenuButton icon={<Copy size={14} />} onClick={onDuplicate}>Duplicate property</MenuButton>}
        {!isTitle && !locked && <MenuButton icon={<Trash2 size={14} />} danger onClick={onDelete}>Delete property</MenuButton>}
      </div>
    </div>
  );
}

function MenuButton({ icon, children, onClick, danger }: { icon: React.ReactNode; children: React.ReactNode; onClick: () => void; danger?: boolean }) {
  return (
    <button type="button" onClick={onClick} className={cn("flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-start text-sm hover:bg-surface-2", danger && "text-danger")}>
      <span className="opacity-70">{icon}</span>
      {children}
    </button>
  );
}

/* ---------- new property ---------- */

export function NewPropertyPicker({
  db, databases, onCreate, onClose,
}: { db: Database; databases: DatabaseMeta[]; onCreate: (p: Property, openMenu: boolean) => void; onClose: () => void }) {
  const [kind, setKind] = useState<PropKind | null>(null);
  const [q, setQ] = useState("");
  const [name, setName] = useState("");
  const [relDb, setRelDb] = useState(databases.find((d) => !d.inline && d.trashedAt === null)?.id ?? db.id);
  const [both, setBoth] = useState(true);
  const [roll, setRoll] = useState({ relation: "", prop: "", calc: "countAll" });
  const rels = db.props.filter((p) => p.kind === "relation");
  const rollRel = rels.find((p) => p.id === roll.relation) ?? rels[0];
  const target = useDatabase(rollRel?.relationDb ?? "");
  useEffect(() => {
    if (rollRel && !target) void loadDatabase(rollRel.relationDb).catch(() => undefined);
  }, [rollRel, target]);

  const options = KIND_ORDER.filter((k) => !q || KINDS[k].label.toLowerCase().includes(q.toLowerCase()));

  const pick = (k: PropKind) => {
    if (k === "relation" || k === "rollup") {
      setKind(k);
      setName(KINDS[k].label);
      return;
    }
    const p = blankProperty(k);
    if (k === "status") {
      p.options = [blankOption("Not started", "gray", "todo"), blankOption("In progress", "blue", "inProgress"), blankOption("Done", "green", "complete")];
    } else if (k === "select" || k === "multiSelect") p.options = [];
    onCreate(p, k === "formula" || k === "button" || hasOptions(k));
    onClose();
  };

  if (kind === "relation") {
    const dbs = databases.filter((d) => d.trashedAt === null && (!d.inline || d.id === db.id));
    return (
      <div className="space-y-2.5">
        <PanelTitle>New relation</PanelTitle>
        <input aria-label="Property name" dir="auto" value={name} onChange={(e) => setName(e.target.value)} className={cn(fieldCls, "w-full")} />
        <Row label="Related to">
          <PSelect value={relDb} label="Related database" className="w-[12rem]" onChange={setRelDb}>
            {dbs.map((d) => (
              <option key={d.id} value={d.id}>{d.icon} {d.title || "Untitled"}{d.id === db.id ? " (this one)" : ""}</option>
            ))}
          </PSelect>
        </Row>
        <SwitchRow label="Show on that database too" checked={both} onChange={setBoth} />
        <div className="flex justify-end gap-2">
          <button type="button" onClick={() => setKind(null)} className="rounded-md px-3 py-1.5 text-sm text-muted hover:bg-surface-2">Back</button>
          <button
            type="button"
            className="rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-fg hover:opacity-90"
            onClick={() => {
              onCreate({ ...blankProperty("relation", name.trim() || "Relation"), relationDb: relDb, relationBack: both ? "+" : "" }, false);
              onClose();
            }}
          >
            Create
          </button>
        </div>
      </div>
    );
  }
  if (kind === "rollup") {
    const tp = target?.props.find((p) => p.id === roll.prop);
    const calcs = calcsFor(tp?.kind ?? "text", true);
    return (
      <div className="space-y-2.5">
        <PanelTitle>New rollup</PanelTitle>
        <input aria-label="Property name" dir="auto" value={name} onChange={(e) => setName(e.target.value)} className={cn(fieldCls, "w-full")} />
        {rels.length === 0 ? (
          <p className="text-sm text-muted">A rollup reads a property of related rows. Add a relation property first.</p>
        ) : (
          <>
            <Row label="Relation"><PropSelect props={rels} value={rollRel?.id ?? ""} label="Relation" className="w-[12rem]" onChange={(v) => setRoll({ relation: v, prop: "", calc: "countAll" })} /></Row>
            <Row label="Property"><PropSelect props={target?.props ?? []} value={roll.prop} label="Rolled up property" className="w-[12rem]" onChange={(v) => setRoll({ ...roll, prop: v })} /></Row>
            <Row label="Calculate">
              <PSelect value={calcs.includes(roll.calc) ? roll.calc : calcs[0]} label="Calculation" className="w-[12rem]" onChange={(v) => setRoll({ ...roll, calc: v })}>
                {calcs.map((c) => (
                  <option key={c} value={c}>{CALC_LABELS[c]}</option>
                ))}
              </PSelect>
            </Row>
          </>
        )}
        <div className="flex justify-end gap-2">
          <button type="button" onClick={() => setKind(null)} className="rounded-md px-3 py-1.5 text-sm text-muted hover:bg-surface-2">Back</button>
          <button
            type="button"
            disabled={!rollRel || !roll.prop}
            className="rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-fg hover:opacity-90 disabled:opacity-40"
            onClick={() => {
              onCreate({ ...blankProperty("rollup", name.trim() || "Rollup"), rollupRelation: rollRel!.id, rollupProp: roll.prop, rollupCalc: calcs.includes(roll.calc) ? roll.calc : calcs[0] }, false);
              onClose();
            }}
          >
            Create
          </button>
        </div>
      </div>
    );
  }
  return (
    <div>
      <input autoFocus aria-label="Search property types" value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search for a type…" className={cn(fieldCls, "mb-2 w-full")} />
      <div className="grid max-h-[min(400px,60vh)] grid-cols-2 gap-0.5 overflow-y-auto">
        {options.map((k) => (
          <button key={k} type="button" onClick={() => pick(k)} className="flex items-center gap-2 rounded-md px-2 py-1.5 text-start text-sm hover:bg-surface-2">
            <KindIcon kind={k} className="text-muted" /> {KINDS[k].label}
          </button>
        ))}
      </div>
    </div>
  );
}
