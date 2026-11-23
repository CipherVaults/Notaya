// List view: one compact line per row (icon, title, a few properties on the far side), checkboxes toggle in place.

import { memo, useCallback, useContext, useLayoutEffect, useMemo, useRef, useState } from "react";
import { defaultRangeExtractor, useVirtualizer, type Range, type VirtualItem } from "@tanstack/react-virtual";
import { Check, ChevronDown, ChevronRight, List as ListIcon } from "lucide-react";
import type { DbRow, GroupOut, Property } from "@/lib/database-types";
import { DbShellContext } from "../context";
import type { ViewProps } from "../viewTypes";
import { cn } from "@/lib/utils";
import { EmptyState, GroupChip, NewButton, NewTitleInput, RowTitle, groupPatch, groupProp, setCollapsed, titleMap, useViewRows, valueNode, visibleProps } from "./shared";

type ListItem = { key: string; group: GroupOut | null } & (
  | { kind: "header"; group: GroupOut; closed: boolean }
  | { kind: "row"; id: string; rowIndex: number }
  | { kind: "new" }
);

export default function ListView({ db, view, result, rowById, rowTitles, tableRowIds, loadTableRows, openRow, setCells, createRow, updateView }: ViewProps) {
  const compact = useContext(DbShellContext)?.compact;
  const g = view.group;
  const gp = groupProp(db, g);
  const shown = useMemo(() => visibleProps(db, view, [g?.prop]), [db, view, g?.prop]);
  const { lead, rest } = useMemo(() => {
    const lead = shown.find((p) => p.kind === "checkbox");
    return { lead, rest: shown.filter((p) => p !== lead) };
  }, [shown]);
  const titles = useMemo(() => rowTitles ?? titleMap(result.rows), [rowTitles, result.rows]);
  const [adding, setAdding] = useState<string | null>(null);
  const [focused, setFocused] = useState<string | null>(null);
  const [focusTarget, setFocusTarget] = useState<string | null>(null);
  const host = useRef<HTMLDivElement>(null);

  const orderedIds = useMemo(() => tableRowIds ?? result.rows.map((row) => row.id), [tableRowIds, tableRowIds ? undefined : result.rows]);
  const groups = g ? result.groups : null;
  const { items, rowIndices, itemIndices } = useMemo(() => {
    const items: ListItem[] = [];
    const rowIndices: number[] = [];
    const itemIndices = new Map<string, number>();
    const collapsed = new Set(g?.collapsed);
    const append = (item: ListItem) => {
      itemIndices.set(item.key, items.length);
      if (item.kind === "row") rowIndices.push(items.length);
      items.push(item);
    };
    const addRows = (ids: readonly string[], group: GroupOut | null, prefix: string) => {
      for (const id of ids) append({ kind: "row", id, group, key: `${prefix}row:${id}`, rowIndex: rowIndices.length });
      append({ kind: "new", group, key: `${prefix}new` });
    };
    if (groups) {
      for (const group of groups) {
        const prefix = `group:${group.key.length}:${group.key}:`;
        const closed = collapsed.has(group.key);
        append({ kind: "header", group, closed, key: `${prefix}header` });
        if (!closed) addRows(group.rows, group, prefix);
      }
    } else addRows(orderedIds, null, "all:");
    return { items, rowIndices, itemIndices };
  }, [g?.collapsed, groups, orderedIds]);
  const rangeExtractor = useCallback((range: Range) => {
    const indices = new Set(defaultRangeExtractor(range));
    for (const key of [focused, focusTarget, adding]) {
      const index = key === null ? undefined : itemIndices.get(key);
      if (index !== undefined) indices.add(index);
    }
    return [...indices].sort((a, b) => a - b);
  }, [adding, focused, focusTarget, itemIndices]);
  const getScrollElement = useCallback(() => host.current, []);
  const getItemKey = useCallback((i: number) => items[i].key, [items]);
  const estimateSize = useCallback((i: number) => items[i].kind === "new" && items[i].group ? 44 : 36, [items]);
  const virtualizer = useVirtualizer<HTMLDivElement, HTMLDivElement>({
    count: items.length, getScrollElement, getItemKey, estimateSize, rangeExtractor,
    overscan: 8, paddingStart: 4, paddingEnd: 32,
  });
  const addingIndex = adding === null ? undefined : itemIndices.get(adding);
  useLayoutEffect(() => {
    if (addingIndex !== undefined) virtualizer.scrollToIndex(addingIndex, { align: "auto" });
  }, [addingIndex, virtualizer]);
  const virtualItems = virtualizer.getVirtualItems();
  useViewRows(loadTableRows, virtualItems.flatMap(({ index }) => {
    const item = items[index];
    return item.kind === "row" ? [item.id] : [];
  }));

  const toggle = useCallback((row: DbRow, p: Property) => void setCells(row.id, { [p.id]: row.values[p.id] !== true }), [setCells]);

  const focusItem = useCallback((index: number) => {
    const node = host.current?.querySelector<HTMLElement>(`[data-index="${index}"] [data-list-row]`);
    node?.focus({ preventScroll: true });
    return !!node;
  }, []);
  const moveFocus = (item: Extract<ListItem, { kind: "row" }>, delta: number) => {
    const pendingIndex = focusTarget === null ? undefined : itemIndices.get(focusTarget);
    const pending = pendingIndex === undefined ? undefined : items[pendingIndex];
    const current = pending?.kind === "row" ? pending : item;
    const next = rowIndices[current.rowIndex + delta];
    if (next === undefined) return;
    virtualizer.scrollToIndex(next, { align: "auto" });
    setFocusTarget(focusItem(next) ? null : items[next].key);
  };
  useLayoutEffect(() => {
    if (focusTarget === null) return;
    const index = itemIndices.get(focusTarget);
    if (index === undefined || focusItem(index)) setFocusTarget(null);
  }, [focusItem, focusTarget, itemIndices, rowById, virtualItems]);

  const cellsFor = (gr: GroupOut | null): Record<string, unknown> => {
    if (!gr || !g) return {};
    const patch = groupPatch(gp, g, gr.key, gr.label) ?? {};
    return Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null));
  };

  const footer = (id: string, gr: GroupOut | null) =>
    adding === id ? (
      <div className="px-2 py-1">
        <NewTitleInput keepOpen onSubmit={(t) => void createRow(cellsFor(gr), t)} onCancel={() => setAdding((a) => (a === id ? null : a))} />
      </div>
    ) : (
      <NewButton onClick={() => setAdding(id)} className="ms-1" />
    );

  const visibleSections = new Map<GroupOut | null, VirtualItem[]>();
  for (const virtual of virtualItems) {
    const group = items[virtual.index].group;
    const section = visibleSections.get(group);
    if (section) section.push(virtual);
    else visibleSections.set(group, [virtual]);
  }
  const renderItem = ({ index, start }: VirtualItem, origin: number) => {
    const item = items[index];
    const position = { top: start - origin };
    if (item.kind === "header") {
      const { group, closed } = item;
      return (
        <button key={item.key} type="button" aria-expanded={!closed} style={{ top: position.top + 2 }}
          onClick={() => {
            const patch = setCollapsed(view, group.key, !closed);
            if (patch) updateView(patch);
          }}
          className="absolute inset-x-0 flex h-8 w-full items-center gap-1.5 rounded-lg px-1.5 text-start hover:bg-surface"
        >
          {closed ? <ChevronRight size={14} className="rtl:rotate-180" /> : <ChevronDown size={14} />}
          <GroupChip g={group} />
          <span className="text-xs tabular-nums text-faint" data-count>{group.count}</span>
        </button>
      );
    }
    if (item.kind === "new") {
      return (
        <div key={item.key} data-index={index} ref={virtualizer.measureElement} style={position} className={cn("absolute inset-x-0", item.group && "pb-3")}>
          {footer(item.key, item.group)}
        </div>
      );
    }
    const row = rowById.get(item.id);
    return (
      <div key={item.key} data-index={index} ref={virtualizer.measureElement} style={position} className="absolute inset-x-0">
        {row ? <ListRow row={row} lead={lead} rest={rest} titles={titles} openRow={openRow} toggle={toggle}
          onFocus={() => setFocused(item.key)} moveFocus={(delta) => moveFocus(item, delta)} /> : (
          <div className="flex h-9 items-center px-2" aria-hidden="true"><div className="h-3 w-32 rounded bg-surface-2" /></div>
        )}
      </div>
    );
  };

  return (
    <div ref={host} className={cn("h-full overflow-y-auto px-4 [overflow-anchor:none]", compact && "max-h-[520px]")}
      role="list" aria-label="List" onBlurCapture={(event) => {
        if (event.relatedTarget instanceof Element && event.currentTarget.contains(event.relatedTarget) && event.relatedTarget.closest("[data-list-row]")) return;
        setFocused(null);
        setFocusTarget(null);
      }}>
      {!orderedIds.length && !groups?.length ? (
        <EmptyState icon={<ListIcon size={28} />} title="No rows yet">
          <NewButton onClick={() => setAdding("all:new")} />
          {adding === "all:new" ? <NewTitleInput keepOpen onSubmit={(t) => void createRow({}, t)} onCancel={() => setAdding(null)} /> : null}
        </EmptyState>
      ) : (
        <div className="relative mx-auto max-w-[56rem]" style={{ height: virtualizer.getTotalSize() }}>
          {[...visibleSections].map(([group, section]) => {
            if (!group) return section.map((item) => renderItem(item, 0));
            const top = section[0].start;
            const height = section[section.length - 1].end - top;
            return (
              <section key={group.key} className="absolute inset-x-0" style={{ top, height }} data-group={group.key}>
                {section.map((item) => renderItem(item, top))}
              </section>
            );
          })}
        </div>
      )}
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

const ListRow = memo(function ListRow({ row, lead, rest, titles, openRow, toggle, onFocus, moveFocus }: {
  row: DbRow; lead: Property | undefined; rest: Property[]; titles: Map<string, string>;
  openRow: (id: string) => void; toggle: (row: DbRow, prop: Property) => void;
  onFocus: () => void;
  moveFocus: (delta: number) => void;
}) {
  return (
    <div
      key={row.id}
      role="listitem"
      tabIndex={0}
      data-list-row={row.id}
      onFocus={onFocus}
      onClick={() => openRow(row.id)}
      onKeyDown={(e) => {
        if (e.target !== e.currentTarget) return;
        if (e.key === "Enter") {
          e.preventDefault();
          openRow(row.id);
        } else if (e.key === "ArrowDown") {
          e.preventDefault();
          moveFocus(1);
        } else if (e.key === "ArrowUp") {
          e.preventDefault();
          moveFocus(-1);
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
