// List view: one compact line per row (icon, title, a few properties on the far side), checkboxes toggle in place.

import { memo, useCallback, useMemo, useRef, useState, type KeyboardEvent } from "react";
import { Check, ChevronDown, ChevronRight, List as ListIcon } from "lucide-react";
import type { DbRow, GroupBy, GroupOut, Property } from "@/lib/database-types";
import type { ViewProps } from "../viewTypes";
import { cn } from "@/lib/utils";
import { EmptyState, GroupChip, NewButton, NewTitleInput, RowTitle, groupPatch, groupProp, setCollapsed, titleMap, valueNode, visibleProps } from "./shared";

export default function ListView({ db, view, result, rowById, openRow, setCells, createRow, updateView }: ViewProps) {
  const g = view.group;
  const gp = groupProp(db, g);
  const shown = useMemo(() => visibleProps(db, view, [g?.prop]), [db, view, g?.prop]);
  const { lead, rest } = useMemo(() => {
    const lead = shown.find((p) => p.kind === "checkbox");
    return { lead, rest: shown.filter((p) => p !== lead) };
  }, [shown]);
  const titles = useMemo(() => titleMap(result.rows), [result.rows]);
  const [adding, setAdding] = useState<string | null>(null);
  const host = useRef<HTMLDivElement>(null);

  const toggle = useCallback((row: DbRow, p: Property) => void setCells(row.id, { [p.id]: row.values[p.id] !== true }), [setCells]);

  const moveFocus = useCallback((e: KeyboardEvent<HTMLElement>, delta: number) => {
    const all = Array.from(host.current?.querySelectorAll<HTMLElement>("[data-list-row]") ?? []);
    const i = all.indexOf(e.currentTarget);
    all[i + delta]?.focus();
  }, []);

  const renderRow = (row: DbRow) => <ListRow key={row.id} row={row} lead={lead} rest={rest} titles={titles} openRow={openRow} toggle={toggle} moveFocus={moveFocus} />;

  const cellsFor = (gr: GroupOut | null, gg: GroupBy | null): Record<string, unknown> => {
    if (!gr || !gg) return {};
    const patch = groupPatch(gp, gg, gr.key, gr.label) ?? {};
    return Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null));
  };

  const footer = (id: string, gr: GroupOut | null) =>
    adding === id ? (
      <div className="px-2 py-1">
        <NewTitleInput keepOpen onSubmit={(t) => void createRow(cellsFor(gr, g), t)} onCancel={() => setAdding((a) => (a === id ? null : a))} />
      </div>
    ) : (
      <NewButton onClick={() => setAdding(id)} className="ms-1" />
    );

  if (!result.rows.length && !(result.groups && result.groups.length)) {
    return (
      <EmptyState icon={<ListIcon size={28} />} title="No rows yet">
        <NewButton onClick={() => setAdding("all")} />
        {adding === "all" ? <NewTitleInput keepOpen onSubmit={(t) => void createRow({}, t)} onCancel={() => setAdding(null)} /> : null}
      </EmptyState>
    );
  }

  return (
    <div ref={host} className="h-full overflow-y-auto px-4 pb-8 pt-1" role="list" aria-label="List">
      <div className="mx-auto max-w-[56rem]">
        {result.groups && g ? (
          result.groups.map((gr) => {
            const closed = g.collapsed.includes(gr.key);
            return (
              <section key={gr.key} className="mb-3" data-group={gr.key}>
                <button
                  type="button"
                  aria-expanded={!closed}
                  onClick={() => {
                    const p = setCollapsed(view, gr.key, !closed);
                    if (p) updateView(p);
                  }}
                  className="flex h-8 w-full items-center gap-1.5 rounded-lg px-1.5 text-start hover:bg-surface"
                >
                  {closed ? <ChevronRight size={14} className="rtl:rotate-180" /> : <ChevronDown size={14} />}
                  <GroupChip g={gr} />
                  <span className="text-xs tabular-nums text-faint" data-count>{gr.count}</span>
                </button>
                {closed ? null : (
                  <div className="mt-0.5">
                    {gr.rows.map((id) => rowById.get(id)).filter((r): r is DbRow => !!r).map(renderRow)}
                    {footer("g:" + gr.key, gr)}
                  </div>
                )}
              </section>
            );
          })
        ) : (
          <>
            {result.rows.map(renderRow)}
            {footer("all", null)}
          </>
        )}
      </div>
    </div>
  );
}

function CheckBox({ checked, label, onToggle, small }: { checked: boolean; label: string; onToggle: () => void; small?: boolean }) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={checked}
      aria-label={label}
      onClick={(e) => {
        e.stopPropagation();
        onToggle();
      }}
      onKeyDown={(e) => e.stopPropagation()}
      className={cn(
        "grid shrink-0 place-items-center rounded border transition-colors",
        small ? "h-4 w-4" : "h-[18px] w-[18px]",
        checked ? "border-accent bg-accent text-accent-fg" : "border-faint/70 hover:border-fg",
      )}
    >
      {checked ? <Check size={small ? 11 : 13} strokeWidth={3} /> : null}
    </button>
  );
}

const ListRow = memo(function ListRow({ row, lead, rest, titles, openRow, toggle, moveFocus }: {
  row: DbRow; lead: Property | undefined; rest: Property[]; titles: Map<string, string>;
  openRow: (id: string) => void; toggle: (row: DbRow, prop: Property) => void;
  moveFocus: (e: KeyboardEvent<HTMLElement>, delta: number) => void;
}) {
  return (
    <div
      key={row.id}
      role="listitem"
      tabIndex={0}
      data-list-row={row.id}
      onClick={() => openRow(row.id)}
      onKeyDown={(e) => {
        if (e.target !== e.currentTarget) return;
        if (e.key === "Enter") {
          e.preventDefault();
          openRow(row.id);
        } else if (e.key === "ArrowDown") {
          e.preventDefault();
          moveFocus(e, 1);
        } else if (e.key === "ArrowUp") {
          e.preventDefault();
          moveFocus(e, -1);
        }
      }}
      className="group/row flex min-h-9 cursor-pointer items-center gap-2 rounded-lg px-2 py-1 text-[0.88rem] outline-none hover:bg-surface focus-visible:ring-2 focus-visible:ring-accent"
    >
      {lead ? <CheckBox checked={row.values[lead.id] === true} label={`${lead.name}: ${row.title || "Untitled"}`} onToggle={() => toggle(row, lead)} /> : null}
      <RowTitle row={row} className={cn("flex-1", lead && row.values[lead.id] === true && "text-muted line-through")} />
      <div className="flex min-w-0 max-w-[65%] shrink-0 items-center justify-end gap-3 text-[0.78rem] text-muted">
        {rest.map((p) => {
          if (p.kind === "checkbox") {
            return <CheckBox key={p.id} checked={row.values[p.id] === true} label={`${p.name}: ${row.title || "Untitled"}`} onToggle={() => toggle(row, p)} small />;
          }
          const node = valueNode(p, row.values[p.id], titles);
          return node ? (
            <span key={p.id} className="min-w-0 max-w-[12rem] truncate" title={p.name} data-prop={p.id}>
              {node}
            </span>
          ) : null;
        })}
      </div>
    </div>
  );
});
