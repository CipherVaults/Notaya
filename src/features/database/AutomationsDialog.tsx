// Automations of a database: "when this happens to a row, and it matches, do these actions".
// Triggers: a row is added, a property changes, a date arrives. Actions stay inside the vault (see ActionsEditor).

import { useState } from "react";
import { Plus, Trash2, Zap } from "lucide-react";
import type { Automation, Database, Filter, Trigger } from "@/lib/database-types";
import { cn } from "@/lib/utils";
import { Modal } from "@/components/ui";
import { editDatabase, useDatabase } from "./dbStore";
import { emptyFilter, newId } from "./model";
import ActionsEditor, { blankAction } from "./panels/ActionsEditor";
import { FilterGroup } from "./panels/FilterPanel";
import { PSelect, PropSelect, SwitchRow, fieldCls } from "./panels/ui";

const TRIGGERS: [Trigger["kind"], string][] = [
  ["rowAdded", "A row is added"],
  ["propChanged", "A property changes"],
  ["dateReached", "A date arrives"],
];

const blankAutomation = (): Automation => ({
  id: newId(),
  name: "New automation",
  enabled: true,
  trigger: { kind: "rowAdded", prop: "", offsetMins: 0 },
  filter: emptyFilter(),
  actions: [blankAction()],
  lastRun: 0,
});

function unitOf(mins: number): [number, "minutes" | "hours" | "days"] {
  const a = Math.abs(mins);
  if (a >= 1440 && a % 1440 === 0) return [a / 1440, "days"];
  if (a >= 60 && a % 60 === 0) return [a / 60, "hours"];
  return [a, "minutes"];
}
const MULT = { minutes: 1, hours: 60, days: 1440 } as const;

export default function AutomationsDialog({ db: initial, onClose, onSaved }: { db: Database; onClose: () => void; onSaved: () => void }) {
  const live = useDatabase(initial.id) ?? initial;
  const autos = live.automations ?? [];
  const [sel, setSel] = useState<string | null>(autos[0]?.id ?? null);
  const a = autos.find((x) => x.id === sel) ?? null;
  const edit = (patch: Partial<Automation>, now = false) => {
    void editDatabase(live.id, (d) => ({ ...d, automations: (d.automations ?? []).map((x) => (x.id === sel ? { ...x, ...patch } : x)) }), now);
    onSaved();
  };
  const add = () => {
    const na = blankAutomation();
    void editDatabase(live.id, (d) => ({ ...d, automations: [...(d.automations ?? []), na] }), true);
    setSel(na.id);
  };
  const remove = () => {
    if (!a) return;
    const rest = autos.filter((x) => x.id !== a.id);
    void editDatabase(live.id, (d) => ({ ...d, automations: rest }), true);
    setSel(rest[0]?.id ?? null);
  };
  const setTrigger = (t: Partial<Trigger>) => a && edit({ trigger: { ...a.trigger, ...t } });
  const dateProps = live.props.filter((p) => p.kind === "date");
  const [amount, unit] = a ? unitOf(a.trigger.offsetMins) : [0, "minutes" as const];
  const before = !!a && a.trigger.offsetMins < 0;
  const setOffset = (n: number, u: keyof typeof MULT, b: boolean) => setTrigger({ offsetMins: Math.max(0, Math.round(n || 0)) * MULT[u] * (b ? -1 : 1) });
  const hasFilter = !!a && (a.filter.conds.length > 0 || a.filter.groups.length > 0);
  const [showFilter, setShowFilter] = useState(false);

  return (
    <Modal open onOpenChange={(o) => !o && onClose()} title="Automations" description="Notaya does these things for you while it is open. They only change this vault: nothing is sent anywhere." width={780}>
      <div className="flex min-h-[340px] gap-4">
        <div className="w-52 shrink-0 space-y-1">
          {autos.map((x) => (
            <button
              key={x.id}
              type="button"
              onClick={() => setSel(x.id)}
              className={cn("flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-start text-sm", x.id === sel ? "bg-surface-2 font-medium" : "hover:bg-surface")}
            >
              <Zap size={13} className={cn("shrink-0", x.enabled ? "text-accent-strong" : "text-faint")} aria-hidden />
              <span dir="auto" className={cn("min-w-0 flex-1 truncate", !x.enabled && "text-faint line-through")}>{x.name || "Automation"}</span>
            </button>
          ))}
          <button type="button" onClick={add} className="inline-flex items-center gap-1.5 px-2 py-1.5 text-sm text-accent-strong hover:underline">
            <Plus size={14} /> New automation
          </button>
        </div>
        <div className="min-w-0 flex-1 space-y-3">
          {!a ? (
            <p className="pt-8 text-center text-sm text-muted">No automations yet. For example: when a task is marked Done, tick its checkbox and set the finished date.</p>
          ) : (
            <>
              <div className="flex items-center gap-2">
                <input aria-label="Automation name" dir="auto" value={a.name} maxLength={100} onChange={(e) => edit({ name: e.target.value })} className={cn(fieldCls, "min-w-0 flex-1 font-medium")} />
                <button type="button" aria-label="Delete this automation" onClick={remove} className="grid h-8 w-8 shrink-0 place-items-center rounded-md text-faint hover:bg-surface-2 hover:text-danger">
                  <Trash2 size={15} />
                </button>
              </div>
              <SwitchRow label="On" checked={a.enabled} onChange={(v) => edit({ enabled: v })} />

              <section className="space-y-1.5">
                <h3 className="text-[0.7rem] font-semibold uppercase tracking-wider text-faint">When</h3>
                <PSelect value={a.trigger.kind} label="Trigger" className="w-full" onChange={(k) => setTrigger({ kind: k as Trigger["kind"], prop: k === "dateReached" ? (dateProps[0]?.id ?? "") : "", offsetMins: 0 })}>
                  {TRIGGERS.map(([k, l]) => (
                    <option key={k} value={k}>{l}</option>
                  ))}
                </PSelect>
                {a.trigger.kind === "propChanged" && (
                  <PropSelect props={live.props.filter((p) => p.kind !== "button")} value={a.trigger.prop} allowNone noneLabel="Any property" label="Property that changes" onChange={(v) => setTrigger({ prop: v })} />
                )}
                {a.trigger.kind === "dateReached" &&
                  (dateProps.length === 0 ? (
                    <p className="text-xs text-danger">This database has no date property. Add one first.</p>
                  ) : (
                    <div className="flex flex-wrap items-center gap-1.5">
                      <PropSelect props={dateProps} value={a.trigger.prop} label="Date property" className="w-[9rem]" onChange={(v) => setTrigger({ prop: v })} />
                      <input aria-label="How long" type="number" min={0} value={amount} onChange={(e) => setOffset(Number(e.target.value), unit, before)} className={cn(fieldCls, "w-16")} />
                      <PSelect value={unit} label="Unit" className="w-[6.5rem]" onChange={(u) => setOffset(amount, u as keyof typeof MULT, before)}>
                        <option value="minutes">minutes</option>
                        <option value="hours">hours</option>
                        <option value="days">days</option>
                      </PSelect>
                      <PSelect value={before ? "before" : "after"} label="Before or after" className="w-[6rem]" onChange={(v) => setOffset(amount, unit, v === "before")}>
                        <option value="before">before</option>
                        <option value="after">after</option>
                      </PSelect>
                    </div>
                  ))}
                {a.trigger.kind === "dateReached" && <p className="text-xs text-muted">A date without a time counts as 09:00. Notaya runs it while it is open (and catches up on up to a week when it was closed).</p>}
              </section>

              <section className="space-y-1.5">
                <div className="flex items-center justify-between">
                  <h3 className="text-[0.7rem] font-semibold uppercase tracking-wider text-faint">Only if</h3>
                  {!hasFilter && !showFilter && (
                    <button type="button" onClick={() => setShowFilter(true)} className="text-xs text-accent-strong hover:underline">Add a condition</button>
                  )}
                </div>
                {(hasFilter || showFilter) && <FilterGroup filter={a.filter} props={live.props.filter((p) => p.kind !== "button")} depth={1} onChange={(f: Filter) => edit({ filter: f })} />}
                {!hasFilter && !showFilter && <p className="text-xs text-muted">Every row counts.</p>}
              </section>

              <section className="space-y-1.5">
                <h3 className="text-[0.7rem] font-semibold uppercase tracking-wider text-faint">Do</h3>
                <ActionsEditor db={live} actions={a.actions} onChange={(actions) => edit({ actions })} />
              </section>
            </>
          )}
        </div>
      </div>
    </Modal>
  );
}
