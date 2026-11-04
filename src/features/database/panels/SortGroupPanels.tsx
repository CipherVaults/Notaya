// Sort and Group panels.

import { Plus, Trash2, X } from "lucide-react";
import type { GroupBy, Property, Sort } from "@/lib/database-types";
import { blankGroup } from "../model";
import { PSelect, PropSelect, SwitchRow, PanelTitle, Row, fieldCls } from "./ui";
import { cn } from "@/lib/utils";

export function SortPanel({ sorts, props, onChange }: { sorts: Sort[]; props: Property[]; onChange: (s: Sort[]) => void }) {
  const first = props.find((p) => !sorts.some((s) => s.prop === p.id)) ?? props[0];
  return (
    <div>
      {sorts.length === 0 && <div className="py-1 text-sm text-faint">No sorts. Rows keep your own order and can be dragged.</div>}
      {sorts.map((s, i) => (
        <div key={i} className="flex items-center gap-1.5 py-1">
          <PropSelect props={props} value={s.prop} label="Sort by" className="min-w-0 flex-1" onChange={(id) => onChange(sorts.map((x, j) => (j === i ? { ...x, prop: id } : x)))} />
          <PSelect value={s.descending ? "desc" : "asc"} label="Direction" className="w-[8.5rem]" onChange={(v) => onChange(sorts.map((x, j) => (j === i ? { ...x, descending: v === "desc" } : x)))}>
            <option value="asc">Ascending</option>
            <option value="desc">Descending</option>
          </PSelect>
          <button type="button" aria-label="Remove sort" onClick={() => onChange(sorts.filter((_, j) => j !== i))} className="rounded p-1 text-faint hover:text-danger">
            <X size={14} />
          </button>
        </div>
      ))}
      <div className="flex items-center gap-1 pt-1">
        <button type="button" disabled={!first || sorts.length >= 10} onClick={() => first && onChange([...sorts, { prop: first.id, descending: false }])} className="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-sm text-muted hover:bg-surface-2 hover:text-fg">
          <Plus size={14} /> Add sort
        </button>
        {sorts.length > 0 && (
          <button type="button" onClick={() => onChange([])} className="ms-auto inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-sm text-muted hover:bg-surface-2 hover:text-danger">
            <Trash2 size={13} /> Delete sort
          </button>
        )}
      </div>
    </div>
  );
}

const GROUPABLE = (p: Property) => p.kind !== "files" && p.kind !== "title";

export function GroupEditor({ group, props, onChange, title }: { group: GroupBy | null; props: Property[]; onChange: (g: GroupBy | null) => void; title: string }) {
  const p = group ? props.find((x) => x.id === group.prop) : undefined;
  const set = (patch: Partial<GroupBy>) => group && onChange({ ...group, ...patch });
  const isDate = p && (p.kind === "date" || p.kind === "createdTime" || p.kind === "lastEditedTime");
  return (
    <div>
      <PanelTitle>{title}</PanelTitle>
      <Row label="Group by">
        <PropSelect
          props={props}
          value={group?.prop ?? ""}
          label={title}
          allowNone
          noneLabel="None"
          filter={GROUPABLE}
          className="w-[11rem]"
          onChange={(id) => onChange(id ? (group && group.prop === id ? group : blankGroup(id)) : null)}
        />
      </Row>
      {group && p && (
        <>
          {p.kind === "status" && (
            <Row label="Group status by">
              <PSelect value={group.dateBy === "group" ? "group" : ""} label="Group status by" className="w-[11rem]" onChange={(v) => set({ dateBy: v })}>
                <option value="">Each status</option>
                <option value="group">To-do / In progress / Complete</option>
              </PSelect>
            </Row>
          )}
          {isDate && (
            <Row label="Date by">
              <PSelect value={group.dateBy || "day"} label="Group date by" className="w-[11rem]" onChange={(v) => set({ dateBy: v })}>
                <option value="day">Day</option>
                <option value="week">Week</option>
                <option value="month">Month</option>
                <option value="year">Year</option>
                <option value="relative">Relative</option>
              </PSelect>
            </Row>
          )}
          {(p.kind === "number" || p.kind === "formula" || p.kind === "rollup") && (
            <Row label="Group size">
              <input
                type="number"
                min={0}
                aria-label="Number group size"
                value={group.numberStep || ""}
                placeholder="Each value"
                onChange={(e) => set({ numberStep: Math.max(0, Number(e.target.value) || 0) })}
                className={cn(fieldCls, "w-[11rem]")}
              />
            </Row>
          )}
          <Row label="Sort groups">
            <PSelect value={group.sort || "manual"} label="Sort groups" className="w-[11rem]" onChange={(v) => set({ sort: v })}>
              <option value="manual">Manual</option>
              <option value="asc">Ascending</option>
              <option value="desc">Descending</option>
            </PSelect>
          </Row>
          <SwitchRow label="Hide empty groups" checked={group.hideEmpty} onChange={(v) => set({ hideEmpty: v })} />
          {group.hidden.length > 0 && (
            <button type="button" onClick={() => set({ hidden: [] })} className="mt-1 rounded-md px-2 py-1 text-sm text-muted hover:bg-surface-2 hover:text-fg">
              Show {group.hidden.length} hidden {group.hidden.length === 1 ? "group" : "groups"}
            </button>
          )}
        </>
      )}
    </div>
  );
}

export function GroupPanel({ group, subGroup, props, onChange, allowSub }: { group: GroupBy | null; subGroup: GroupBy | null; props: Property[]; onChange: (g: GroupBy | null, sub: GroupBy | null) => void; allowSub: boolean }) {
  return (
    <div className="space-y-3">
      <GroupEditor group={group} props={props} onChange={(g) => onChange(g, g ? subGroup : null)} title="Group" />
      {allowSub && group && (
        <div className="border-t border-line pt-3">
          <GroupEditor group={subGroup} props={props.filter((p) => p.id !== group.prop)} onChange={(g) => onChange(group, g)} title="Sub-group" />
        </div>
      )}
    </div>
  );
}
