// The list of actions of a button or an automation. Every action stays inside the vault:
// set a property, add a row, show a message in Notaya, or move the row to the trash.

import { useEffect } from "react";
import { Plus, Trash2 } from "lucide-react";
import type { Action, Database, Property, SetCell } from "@/lib/database-types";
import { cn } from "@/lib/utils";
import { useApp } from "@/store/app";
import { loadDatabase, useDatabase } from "../dbStore";
import { isComputedKind } from "../model";
import { PSelect, PropSelect, fieldCls } from "./ui";

export const blankAction = (kind: Action["kind"] = "setProp"): Action => ({ kind, prop: "", value: "", database: "", title: "", cells: [] });

const KIND_LABELS: Record<Action["kind"], string> = {
  setProp: "Change a property",
  addRow: "Add a row",
  notify: "Show a message",
  trash: "Move this row to the trash",
};

const DATE_PRESETS: [string, string][] = [
  ["@now", "Now"],
  ["@today", "Today"],
  ["@today+1", "Tomorrow"],
  ["@today+7", "In a week"],
  ["@today-1", "Yesterday"],
];

/** Properties an action can set (everything Notaya does not work out itself, and not files or places). */
export const settable = (p: Property) => !isComputedKind(p.kind) && p.kind !== "files" && p.kind !== "id" && p.kind !== "place";

/** The value of one property in an action: a control that fits the property. */
export function ValueInput({ prop, value, onChange, title }: { prop: Property | undefined; value: string; onChange: (v: string) => void; title?: boolean }) {
  if (title || !prop) {
    return <input aria-label="Value" dir="auto" value={value} maxLength={300} placeholder="Text ({title} = this row's name)" onChange={(e) => onChange(e.target.value)} className={cn(fieldCls, "min-w-0 flex-1")} />;
  }
  const mode = value === "@clear" ? "clear" : value.startsWith("@add:") ? "add" : "set";
  const withMode = (modes: [string, string][], body: React.ReactNode) => (
    <span className="flex min-w-0 flex-1 items-center gap-1.5">
      <PSelect value={mode} label="How" className="w-[6.5rem] shrink-0" onChange={(m) => onChange(m === "clear" ? "@clear" : m === "add" ? "@add:" : "")}>
        {modes.map(([k, l]) => (
          <option key={k} value={k}>{l}</option>
        ))}
      </PSelect>
      {mode !== "clear" && body}
    </span>
  );
  switch (prop.kind) {
    case "select":
    case "status":
      return withMode(
        [["set", "Set to"], ["clear", "Clear"]],
        <PSelect value={prop.options.some((o) => o.name === value) ? value : ""} label="Option" className="min-w-0 flex-1" onChange={onChange}>
          <option value="">Choose…</option>
          {prop.options.map((o) => (
            <option key={o.id} value={o.name}>{o.name}</option>
          ))}
        </PSelect>,
      );
    case "multiSelect":
      return withMode(
        [["set", "Set to"], ["add", "Add"], ["clear", "Clear"]],
        <input aria-label="Options" dir="auto" value={value.replace(/^@add:/, "")} placeholder={prop.options.map((o) => o.name).slice(0, 3).join(", ") || "Option, option"} onChange={(e) => onChange(mode === "add" ? `@add:${e.target.value}` : e.target.value)} className={cn(fieldCls, "min-w-0 flex-1")} />,
      );
    case "checkbox":
      return (
        <PSelect value={value || "true"} label="Checkbox" className="min-w-0 flex-1" onChange={onChange}>
          <option value="true">Checked</option>
          <option value="false">Unchecked</option>
          <option value="@toggle">Switch it</option>
        </PSelect>
      );
    case "number":
      return withMode(
        [["set", "Set to"], ["add", "Add"], ["clear", "Clear"]],
        <input aria-label="Number" type="number" value={value.replace(/^@add:/, "")} onChange={(e) => onChange(mode === "add" ? `@add:${e.target.value}` : e.target.value)} className={cn(fieldCls, "min-w-0 flex-1")} />,
      );
    case "date": {
      const preset = DATE_PRESETS.some(([k]) => k === value);
      const custom = value !== "" && !preset && value !== "@clear";
      return withMode(
        [["set", "Set to"], ["clear", "Clear"]],
        <span className="flex min-w-0 flex-1 items-center gap-1.5">
          <PSelect value={preset ? value : custom ? "custom" : ""} label="Date" className="min-w-0 flex-1" onChange={(v) => onChange(v === "custom" ? new Date().toISOString().slice(0, 10) : v)}>
            <option value="">Choose…</option>
            {DATE_PRESETS.map(([k, l]) => (
              <option key={k} value={k}>{l}</option>
            ))}
            <option value="custom">A date…</option>
          </PSelect>
          {custom && <input aria-label="Date" type="date" value={value.slice(0, 10)} onChange={(e) => onChange(e.target.value)} className={cn(fieldCls, "shrink-0")} />}
        </span>,
      );
    }
    case "relation":
      return (
        <PSelect value={value || "@self"} label="Link" className="min-w-0 flex-1" onChange={onChange}>
          <option value="@self">Link to this row</option>
          <option value="@clear">Clear</option>
        </PSelect>
      );
    case "person":
      return withMode([["set", "Set to"], ["clear", "Clear"]], <input aria-label="Names" dir="auto" value={value} placeholder="Name, name" onChange={(e) => onChange(e.target.value)} className={cn(fieldCls, "min-w-0 flex-1")} />);
    default:
      return withMode([["set", "Set to"], ["clear", "Clear"]], <input aria-label="Text" dir="auto" value={value} maxLength={2000} placeholder="Text ({title} = this row's name)" onChange={(e) => onChange(e.target.value)} className={cn(fieldCls, "min-w-0 flex-1")} />);
  }
}

function AddRowEditor({ db, action, onChange }: { db: Database; action: Action; onChange: (a: Action) => void }) {
  const databases = useApp((s) => s.tree?.databases ?? []).filter((d) => d.trashedAt === null && (!d.inline || d.id === db.id));
  const targetId = action.database || db.id;
  const loaded = useDatabase(targetId);
  useEffect(() => {
    if (!loaded) void loadDatabase(targetId).catch(() => undefined);
  }, [loaded, targetId]);
  const props = (loaded?.props ?? []).filter((p) => settable(p) && p.kind !== "title");
  const setCell = (i: number, c: SetCell) => onChange({ ...action, cells: action.cells.map((x, j) => (j === i ? c : x)) });
  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-1.5">
        <PSelect value={action.database} label="Database" className="w-[10rem] shrink-0" onChange={(v) => onChange({ ...action, database: v === db.id ? "" : v, cells: [] })}>
          <option value="">This database</option>
          {databases.filter((d) => d.id !== db.id).map((d) => (
            <option key={d.id} value={d.id}>{d.title || "Untitled"}</option>
          ))}
        </PSelect>
        <input aria-label="Name of the new row" dir="auto" value={action.title} maxLength={300} placeholder="Name ({title} = this row's name)" onChange={(e) => onChange({ ...action, title: e.target.value })} className={cn(fieldCls, "min-w-0 flex-1")} />
      </div>
      {action.cells.map((c, i) => {
        const p = props.find((x) => x.id === c.prop);
        return (
          <div key={i} className="flex items-center gap-1.5 ps-3">
            <PropSelect props={props} value={c.prop} label="Property" className="w-[9rem] shrink-0" onChange={(v) => setCell(i, { prop: v, value: "" })} />
            <ValueInput prop={p} value={c.value} onChange={(v) => setCell(i, { ...c, value: v })} />
            <button type="button" aria-label="Remove this value" onClick={() => onChange({ ...action, cells: action.cells.filter((_, j) => j !== i) })} className="grid h-7 w-7 shrink-0 place-items-center rounded text-faint hover:bg-surface-2 hover:text-fg">
              <Trash2 size={13} />
            </button>
          </div>
        );
      })}
      <button type="button" disabled={props.length === 0} onClick={() => onChange({ ...action, cells: [...action.cells, { prop: props[0]?.id ?? "", value: "" }] })} className="ms-3 inline-flex items-center gap-1 text-xs text-accent-strong hover:underline disabled:opacity-50">
        <Plus size={12} /> Set a value
      </button>
    </div>
  );
}

export default function ActionsEditor({ db, actions, onChange }: { db: Database; actions: Action[]; onChange: (a: Action[]) => void }) {
  const props = db.props.filter(settable);
  const set = (i: number, a: Action) => onChange(actions.map((x, j) => (j === i ? a : x)));
  return (
    <div className="space-y-2">
      {actions.map((a, i) => {
        const p = db.props.find((x) => x.id === a.prop);
        return (
          <div key={i} className="rounded-lg border border-line p-2">
            <div className="flex items-center gap-1.5">
              <PSelect value={a.kind} label="Action" className="min-w-0 flex-1" onChange={(k) => set(i, { ...blankAction(k as Action["kind"]) })}>
                {(Object.keys(KIND_LABELS) as Action["kind"][]).map((k) => (
                  <option key={k} value={k}>{KIND_LABELS[k]}</option>
                ))}
              </PSelect>
              <button type="button" aria-label="Remove this action" onClick={() => onChange(actions.filter((_, j) => j !== i))} className="grid h-7 w-7 shrink-0 place-items-center rounded text-faint hover:bg-surface-2 hover:text-fg">
                <Trash2 size={13} />
              </button>
            </div>
            <div className="mt-1.5">
              {a.kind === "setProp" && (
                <div className="flex items-center gap-1.5">
                  <PropSelect props={props} value={a.prop} label="Property" className="w-[9rem] shrink-0" onChange={(v) => set(i, { ...a, prop: v, value: "" })} />
                  <ValueInput prop={p} title={a.prop === "title"} value={a.value} onChange={(v) => set(i, { ...a, value: v })} />
                </div>
              )}
              {a.kind === "addRow" && <AddRowEditor db={db} action={a} onChange={(n) => set(i, n)} />}
              {a.kind === "notify" && (
                <input aria-label="Message" dir="auto" value={a.title} maxLength={300} placeholder="Message ({title} = this row's name)" onChange={(e) => set(i, { ...a, title: e.target.value })} className={cn(fieldCls, "w-full")} />
              )}
              {a.kind === "trash" && <p className="px-0.5 text-xs text-muted">The row goes to the trash and can be restored from there.</p>}
            </div>
          </div>
        );
      })}
      <button type="button" disabled={actions.length >= 20} onClick={() => onChange([...actions, blankAction()])} className="inline-flex items-center gap-1 text-sm text-accent-strong hover:underline disabled:opacity-50">
        <Plus size={14} /> Add an action
      </button>
    </div>
  );
}
