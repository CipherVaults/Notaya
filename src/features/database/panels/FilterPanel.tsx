// Filters: conditions joined by And/Or, with groups nested up to three levels (the same rule as the backend).

import { Popover } from "radix-ui";
import { Plus, Trash2, X } from "lucide-react";
import type { Cond, Filter, Property } from "@/lib/database-types";
import { cn } from "@/lib/utils";
import { DATE_PRESETS, blankCond, opsFor } from "../model";
import { OptionChip, StatusChip } from "../cells/CellDisplay";
import { RelationPicker } from "../cells/pickers";
import { PSelect, PropSelect, fieldCls } from "./ui";
import { useRowTitles } from "../rowTitles";

const MAX_DEPTH = 3;

interface GroupProps {
  filter: Filter;
  props: Property[];
  depth: number;
  onChange: (f: Filter) => void;
  onRemove?: () => void;
}

export function FilterGroup({ filter, props, depth, onChange, onRemove }: GroupProps) {
  const first = props[0];
  const setCond = (i: number, c: Cond) => onChange({ ...filter, conds: filter.conds.map((x, j) => (j === i ? c : x)) });
  const rows: React.ReactNode[] = [];
  const total = filter.conds.length + filter.groups.length;
  const joiner = (i: number) =>
    i === 0 ? <span className="w-[4.5rem] shrink-0 pe-2 text-sm text-muted">Where</span>
      : i === 1 ? (
        <PSelect value={filter.op} onChange={(v) => onChange({ ...filter, op: v as "and" | "or" })} label="Combine with" className="w-[4.5rem] shrink-0">
          <option value="and">And</option>
          <option value="or">Or</option>
        </PSelect>
      ) : <span className="w-[4.5rem] shrink-0 pe-2 text-sm text-muted">{filter.op === "and" ? "And" : "Or"}</span>;

  filter.conds.forEach((c, i) => {
    const p = props.find((x) => x.id === c.prop);
    rows.push(
      <div key={`c${i}`} className="flex items-start gap-1.5 py-1">
        {joiner(i)}
        <div className="flex min-w-0 flex-1 flex-wrap items-center gap-1.5">
          <PropSelect
            props={props}
            value={c.prop}
            label="Property"
            className="w-[8.5rem]"
            onChange={(id) => {
              const np = props.find((x) => x.id === id);
              if (np) setCond(i, blankCond(np));
            }}
          />
          {p && <CondEditor cond={c} prop={p} onChange={(n) => setCond(i, n)} />}
        </div>
        <button type="button" aria-label="Remove filter" onClick={() => onChange({ ...filter, conds: filter.conds.filter((_, j) => j !== i) })} className="mt-1 rounded p-1 text-faint hover:text-danger">
          <X size={14} />
        </button>
      </div>,
    );
  });
  filter.groups.forEach((g, gi) => {
    const i = filter.conds.length + gi;
    rows.push(
      <div key={`g${gi}`} className="flex items-start gap-1.5 py-1">
        {joiner(i)}
        <div className="min-w-0 flex-1 rounded-lg border border-line bg-surface p-2">
          <FilterGroup
            filter={g}
            props={props}
            depth={depth + 1}
            onChange={(n) => onChange({ ...filter, groups: filter.groups.map((x, j) => (j === gi ? n : x)) })}
            onRemove={() => onChange({ ...filter, groups: filter.groups.filter((_, j) => j !== gi) })}
          />
        </div>
      </div>,
    );
  });

  return (
    <div>
      {total === 0 && <div className="py-1 text-sm text-faint">No filters yet.</div>}
      {rows}
      <div className="flex flex-wrap items-center gap-1 pt-1">
        <button
          type="button"
          disabled={!first}
          onClick={() => first && onChange({ ...filter, conds: [...filter.conds, blankCond(first)] })}
          className="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-sm text-muted hover:bg-surface-2 hover:text-fg"
        >
          <Plus size={14} /> Add filter
        </button>
        {depth < MAX_DEPTH && (
          <button
            type="button"
            disabled={!first}
            onClick={() => first && onChange({ ...filter, groups: [...filter.groups, { op: "and", conds: [blankCond(first)], groups: [] }] })}
            className="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-sm text-muted hover:bg-surface-2 hover:text-fg"
          >
            <Plus size={14} /> Add filter group
          </button>
        )}
        {onRemove && (
          <button type="button" onClick={onRemove} className="ms-auto inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-sm text-muted hover:bg-surface-2 hover:text-danger">
            <Trash2 size={13} /> Delete group
          </button>
        )}
      </div>
    </div>
  );
}

function CondEditor({ cond, prop, onChange }: { cond: Cond; prop: Property; onChange: (c: Cond) => void }) {
  const ops = opsFor(prop.kind);
  const op = ops.find((o) => o.id === cond.op) ?? ops[0];
  const isDate = prop.kind === "date" || prop.kind === "createdTime" || prop.kind === "lastEditedTime";
  return (
    <>
      <PSelect value={op.id} onChange={(v) => onChange({ ...cond, op: v })} label="Condition" className="w-[8.5rem]">
        {ops.map((o) => (
          <option key={o.id} value={o.id}>{o.label}</option>
        ))}
      </PSelect>
      {!op.none && (
        <>
          {isDate && (
            <PSelect value={cond.rel || "exact"} onChange={(v) => onChange({ ...cond, rel: v })} label="Date" className="w-[8.5rem]">
              {(op.id === "between" ? DATE_PRESETS.filter(([k]) => k === "exact") : DATE_PRESETS).map(([k, l]) => (
                <option key={k} value={k}>{l}</option>
              ))}
            </PSelect>
          )}
          <ValueEditor cond={cond} prop={prop} two={op.two} onChange={onChange} />
        </>
      )}
    </>
  );
}

function ValueEditor({ cond, prop, two, onChange }: { cond: Cond; prop: Property; two?: boolean; onChange: (c: Cond) => void }) {
  switch (prop.kind) {
    case "checkbox":
      return (
        <PSelect value={cond.value || "true"} onChange={(v) => onChange({ ...cond, value: v })} label="Value" className="w-[7rem]">
          <option value="true">Checked</option>
          <option value="false">Unchecked</option>
        </PSelect>
      );
    case "select": case "status": case "multiSelect":
      return <OptionsValue cond={cond} prop={prop} onChange={onChange} />;
    case "relation":
      return <RelationValue cond={cond} prop={prop} onChange={onChange} />;
    case "date": case "createdTime": case "lastEditedTime":
      if ((cond.rel || "exact") !== "exact") return null;
      return (
        <>
          <input type="date" aria-label="Date" value={cond.value.slice(0, 10)} onChange={(e) => onChange({ ...cond, value: e.target.value })} className={cn(fieldCls, "w-[9.5rem]")} />
          {two && <input type="date" aria-label="End date" value={cond.value2.slice(0, 10)} onChange={(e) => onChange({ ...cond, value2: e.target.value })} className={cn(fieldCls, "w-[9.5rem]")} />}
        </>
      );
    case "number":
      return <input type="number" aria-label="Value" value={cond.value} onChange={(e) => onChange({ ...cond, value: e.target.value })} className={cn(fieldCls, "w-[7rem]")} />;
    default:
      return <input aria-label="Value" dir="auto" value={cond.value} onChange={(e) => onChange({ ...cond, value: e.target.value })} placeholder="Value" className={cn(fieldCls, "w-[10rem]")} />;
  }
}

function OptionsValue({ cond, prop, onChange }: { cond: Cond; prop: Property; onChange: (c: Cond) => void }) {
  const picked = cond.values;
  const toggle = (id: string) => onChange({ ...cond, value: "", values: picked.includes(id) ? picked.filter((x) => x !== id) : [...picked, id] });
  return (
    <Popover.Root>
      <Popover.Trigger asChild>
        <button type="button" aria-label="Choose options" className={cn(fieldCls, "inline-flex min-w-[6rem] max-w-[14rem] items-center gap-1 overflow-hidden text-start")}>
          {picked.length === 0 ? <span className="text-faint">Choose…</span> : picked.map((id) => {
            const o = prop.options.find((x) => x.id === id);
            return o ? <OptionChip key={id} option={o} /> : null;
          })}
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content align="start" sideOffset={4} className="z-[80] max-h-64 w-56 overflow-y-auto rounded-lg border border-line bg-elevated p-1 shadow-float">
          {prop.options.map((o) => (
            <label key={o.id} className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 hover:bg-surface-2">
              <input type="checkbox" checked={picked.includes(o.id)} onChange={() => toggle(o.id)} className="h-4 w-4 accent-[var(--accent)]" />
              {prop.kind === "status" ? <StatusChip option={o} /> : <OptionChip option={o} />}
            </label>
          ))}
          {!prop.options.length && <div className="px-2 py-2 text-sm text-faint">No options yet</div>}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

function RelationValue({ cond, prop, onChange }: { cond: Cond; prop: Property; onChange: (c: Cond) => void }) {
  const titles = useRowTitles(cond.values);
  return (
    <Popover.Root>
      <Popover.Trigger asChild>
        <button type="button" aria-label="Choose rows" className={cn(fieldCls, "inline-flex min-w-[6rem] max-w-[14rem] items-center gap-1 overflow-hidden text-start")}>
          {cond.values.length === 0 ? <span className="text-faint">Choose rows…</span> : <span dir="auto" className="truncate">{cond.values.map((id) => titles.get(id)?.title || "Untitled").join(", ")}</span>}
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content align="start" sideOffset={4} className="z-[80] rounded-lg border border-line bg-elevated shadow-float">
          <RelationPicker prop={{ ...prop, relationLimitOne: false }} value={cond.values} onChange={(v) => onChange({ ...cond, value: "", values: v ?? [] })} />
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
