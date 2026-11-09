// The table view: header with property menus, frozen columns, groups and sub-groups, footers, row selection,
// drag to reorder (when no sort is set), keyboard navigation and in-place editing.

import { memo, useCallback, useEffect, useId, useLayoutEffect, useMemo, useRef, useState } from "react";
import { defaultRangeExtractor, useVirtualizer } from "@tanstack/react-virtual";
import { Popover } from "radix-ui";
import { ChevronDown, ChevronRight, Copy, ExternalLink, GripVertical, MoreHorizontal, Plus, Trash2, X } from "lucide-react";
import {
  DndContext, KeyboardSensor, PointerSensor, closestCenter, useSensor, useSensors, type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext, horizontalListSortingStrategy, sortableKeyboardCoordinates, useSortable, verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import type { Column, DbRow, GroupBy, GroupOut, Property, SelectOption } from "@/lib/database-types";
import { cn } from "@/lib/utils";
import { Menu, MenuContent, MenuItem, MenuSeparator, MenuTrigger } from "@/components/ui";
import type { ViewProps } from "../viewTypes";
import { useDbShell } from "../context";
import { KindIcon } from "../icons";
import { CALC_LABELS, calcsFor, isComputedKind } from "../model";
import { CheckBox, OptionChip } from "../cells/CellDisplay";
import EditableCell from "../cells/CellEditor";
import { NewPropertyPicker } from "../panels/PropertyMenu";

const GUTTER = 64;
const MIN_W = 80;
// Existing cell content is 34px tall plus its 1px bottom border. Wrapped rows
// are measured; unwrapped rows and the short structural rows have fixed heights.
const ROW_HEIGHT = 35;
const NO_DRAG_ROWS: string[] = [];

function defaultWidth(p: Property): number {
  switch (p.kind) {
    case "title": return 280;
    case "checkbox": return 90;
    case "number": case "id": return 130;
    case "date": case "createdTime": case "lastEditedTime": return 190;
    case "multiSelect": case "relation": case "files": return 220;
    default: return 180;
  }
}

interface Col {
  col: Column;
  prop: Property;
  width: number;
  /** Offset for a sticky (frozen) column, or null. */
  left: number | null;
}

/** The cell values a new row gets inside a group. */
function cellsForGroup(prop: Property | undefined, g: GroupBy | null | undefined, group: GroupOut | undefined): Record<string, unknown> {
  if (!prop || !group) return {};
  const key = group.key;
  switch (prop.kind) {
    case "select": case "status": {
      if (prop.kind === "status" && g?.dateBy === "group") {
        const o = prop.options.find((x) => x.group === key);
        return o ? { [prop.id]: o.id } : {};
      }
      return key ? { [prop.id]: key } : {};
    }
    case "multiSelect": return key ? { [prop.id]: [key] } : {};
    case "checkbox": return { [prop.id]: key === "true" };
    default: return {};
  }
}

interface Actions {
  select: (r: number, c: number, edit?: boolean, itemKey?: string) => void;
  setEditing: (b: boolean) => void;
  commit: (row: DbRow, prop: Property, value: unknown) => void;
  patchLocal: (rowId: string, propId: string, value: unknown) => void;
  open: (rowId: string) => void;
  toggleSelect: (rowId: string, shift: boolean) => void;
  trash: (ids: string[]) => void;
  duplicate: (id: string) => void;
  endEdit: (how: "enter" | "tab" | "blur" | "escape") => void;
}

interface RowProps {
  row: DbRow;
  index: number;
  cols: Col[];
  dbId: string;
  wrap: boolean;
  numbers: boolean;
  selected: boolean;
  activeCol: number;
  editing: boolean;
  seed: string | undefined;
  canDrag: boolean;
  actions: Actions;
  virtualIndex: number;
  ariaIndex: number;
  cellId: (rowId: string, col: number) => string;
  measure: ((node: HTMLTableRowElement | null) => void) | undefined;
  itemKey: string;
}

type TableItem =
  | { kind: "row"; key: string; row: DbRow; rowIndex: number }
  | { kind: "group"; key: string; groupKey: string; group: GroupOut; level: number }
  | { kind: "add"; key: string; cells: Record<string, unknown> }
  | { kind: "footer"; key: string; calcs: Record<string, string> };

const BG = "bg-bg";

// Only this small wrapper subscribes to dnd-kit. Its expensive cells have stable props and
// remain memoized when the drag context changes on a click or selection elsewhere.
const SortableRow = memo(function SortableRow(props: RowProps) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: props.row.id });
  const measure = useCallback((node: HTMLTableRowElement | null) => { setNodeRef(node); props.measure?.(node); }, [setNodeRef, props.measure]);
  return (
    <tr
      ref={measure}
      data-index={props.virtualIndex}
      data-row-id={props.row.id}
      aria-rowindex={props.ariaIndex}
      style={{ transform: CSS.Transform.toString(transform), transition, position: isDragging ? "relative" : undefined, zIndex: isDragging ? 20 : undefined }}
      className={cn("group/row", isDragging && "opacity-70", props.selected ? "bg-accent-soft" : "hover:bg-surface")}
      aria-selected={props.selected}
    >
      <RowGutter {...props} attributes={attributes} listeners={listeners} isDragging={isDragging} />
      <RowCells {...props} />
    </tr>
  );
});

const RowView = memo(function RowView(props: RowProps) {
  if (props.canDrag) return <SortableRow {...props} />;
  return (
    <tr ref={props.measure} data-index={props.virtualIndex} data-row-id={props.row.id} aria-rowindex={props.ariaIndex} className={cn("group/row", props.selected ? "bg-accent-soft" : "hover:bg-surface")} aria-selected={props.selected}>
      <RowGutter {...props} />
      <RowCells {...props} />
    </tr>
  );
});

const RowGutter = memo(function RowGutter({ row, index, numbers, selected, canDrag, actions, attributes, listeners, isDragging }: RowProps & Partial<Pick<ReturnType<typeof useSortable>, "attributes" | "listeners" | "isDragging">>) {
  return (
      <td className={cn("db-cell sticky z-[2] p-0", BG)} style={{ insetInlineStart: 0, width: GUTTER, minWidth: GUTTER }}>
        <div className={cn("flex h-full items-center justify-end gap-0.5 pe-1", selected && "bg-accent-soft")}>
          <button
            type="button"
            {...attributes}
            {...listeners}
            disabled={!canDrag}
            aria-label="Drag to reorder"
            className={cn("grid h-6 w-4 cursor-grab place-items-center text-faint opacity-0 hover:text-fg focus-visible:opacity-100 group-hover/row:opacity-100 disabled:hidden", isDragging && "opacity-100")}
          >
            <GripVertical size={14} />
          </button>
          <span className="grid h-6 w-6 place-items-center">
            <button
              type="button"
              role="checkbox"
              aria-checked={selected}
              aria-label={`Select row ${index + 1}`}
              onClick={(e) => actions.toggleSelect(row.id, e.shiftKey)}
              className={cn("rounded [table[data-has-selection=true]_&]:opacity-100", selected ? "" : "opacity-0 focus-visible:opacity-100 group-hover/row:opacity-100")}
            >
              <CheckBox checked={selected} />
            </button>
            {numbers && !selected && <span className="pointer-events-none absolute text-xs tabular-nums text-faint group-hover/row:hidden [table[data-has-selection=true]_&]:hidden">{index + 1}</span>}
          </span>
        </div>
      </td>
  );
});

const RowCells = memo(function RowCells({ row, index, cols, dbId, wrap, selected, activeCol, editing, seed, actions, cellId, itemKey }: RowProps) {
  return (
    <>
      {cols.map((c, ci) => {
        const isTitle = c.prop.kind === "title";
        const isActive = activeCol === ci;
        const value = isTitle ? row.title : row.values[c.prop.id];
        const sticky = c.left != null;
        return (
          <td
            key={c.prop.id}
            role="gridcell"
            id={cellId(itemKey, ci)}
            tabIndex={-1}
            aria-colindex={ci + 2}
            aria-selected={isActive}
            onMouseDown={(e) => {
              if (e.button !== 0 || (e.target as HTMLElement).closest("button,a,input,textarea")) return;
              const interactive = ["select", "status", "multiSelect", "date", "person", "files", "relation", "place"].includes(c.prop.kind) && !isComputedKind(c.prop.kind);
              // A click on a tag or a date opens its picker at once. The browser must not move the focus afterwards (to the
              // table): the picker would see that as a click outside and close itself the moment it opened.
              if (interactive) {
                e.preventDefault();
                if (isActive && editing) return; // the picker is open already; Radix closes it on a click outside
              }
              actions.select(index, ci, interactive, itemKey);
            }}
            className={cn(
              "db-cell relative align-top",
              BG,
              selected && "!bg-accent-soft",
              sticky && "sticky z-[1]",
              isActive && "db-cell-active",
            )}
            style={{ width: c.width, minWidth: c.width, maxWidth: c.width, ...(sticky ? { insetInlineStart: c.left! } : {}) }}
          >
            <div className={cn("flex min-h-[34px] items-center px-2 py-[5px] text-sm", wrap ? "" : "max-h-[34px] overflow-hidden", isTitle && "pe-14")}>
              <div className="min-w-0 flex-1">
                <EditableCell
                  dbId={dbId}
                  prop={c.prop}
                  row={row}
                  value={value}
                  editing={isActive && editing}
                  setEditing={actions.setEditing}
                  seed={isActive && editing ? seed : undefined}
                  commit={(v) => actions.commit(row, c.prop, v)}
                  onLocalChange={(v) => actions.patchLocal(row.id, c.prop.id, v)}
                  onEnd={actions.endEdit}
                  wrap={wrap}
                />
              </div>
            </div>
            {isTitle && (
              <div className="absolute inset-y-0 end-1 flex items-center gap-0.5 opacity-0 group-hover/row:opacity-100 focus-within:opacity-100">
                <button type="button" aria-label="Open row" title="Open" onClick={() => actions.open(row.id)} className="rounded-md border border-line bg-elevated p-1 text-muted hover:text-fg">
                  <ExternalLink size={12} />
                </button>
                <Menu>
                  <MenuTrigger asChild>
                    <button type="button" aria-label="Row options" className="rounded-md border border-line bg-elevated p-1 text-muted hover:text-fg">
                      <MoreHorizontal size={12} />
                    </button>
                  </MenuTrigger>
                  <MenuContent align="start">
                    <MenuItem icon={<ExternalLink size={14} />} onSelect={() => actions.open(row.id)}>Open</MenuItem>
                    <MenuItem icon={<Copy size={14} />} onSelect={() => actions.duplicate(row.id)}>Duplicate</MenuItem>
                    <MenuSeparator />
                    <MenuItem danger icon={<Trash2 size={14} />} onSelect={() => actions.trash([row.id])}>Move to trash</MenuItem>
                  </MenuContent>
                </Menu>
              </div>
            )}
          </td>
        );
      })}
      <td className="db-cell p-0" style={{ width: 44 }} />
    </>
  );
});

function HeaderCell({ c, ci, onResize, onOpen, locked, wrapTitle }: { c: Col; ci: number; onResize: (w: number, commit: boolean) => void; onOpen: () => void; locked: boolean; wrapTitle: boolean }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: c.prop.id, disabled: locked || c.prop.kind === "title" });
  const startResize = (e: React.PointerEvent) => {
    e.preventDefault();
    e.stopPropagation();
    const startX = e.clientX;
    const startW = c.width;
    const dir = getComputedStyle(e.currentTarget as HTMLElement).direction === "rtl" ? -1 : 1;
    const move = (ev: PointerEvent) => onResize(Math.max(MIN_W, startW + (ev.clientX - startX) * dir), false);
    const up = (ev: PointerEvent) => {
      onResize(Math.max(MIN_W, startW + (ev.clientX - startX) * dir), true);
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };
  const sticky = c.left != null;
  return (
    <th
      ref={setNodeRef}
      scope="col"
      className={cn("db-cell sticky top-0 border-t border-t-line p-0 text-start font-normal", BG, sticky ? "z-[4]" : "z-[3]", isDragging && "opacity-70")}
      style={{ width: c.width, minWidth: c.width, maxWidth: c.width, ...(sticky ? { insetInlineStart: c.left! } : {}), transform: CSS.Translate.toString(transform), transition }}
      data-col={ci}
    >
      <button
        type="button"
        {...attributes}
        {...listeners}
        onClick={onOpen}
        title={c.prop.description || c.prop.name}
        className="flex h-8 w-full items-center gap-1.5 px-2 text-[0.8rem] text-muted hover:bg-surface-2"
      >
        <KindIcon kind={c.prop.kind} size={13} className="shrink-0" />
        <span dir="auto" className={cn("min-w-0 flex-1 text-start", wrapTitle ? "" : "truncate")}>{c.prop.name}</span>
      </button>
      <span
        role="separator"
        aria-orientation="vertical"
        aria-label={`Resize ${c.prop.name}`}
        onPointerDown={startResize}
        className="absolute inset-y-0 end-0 z-[5] w-1.5 cursor-col-resize hover:bg-accent/40"
      />
    </th>
  );
}

export default function TableView({ db, view, result, locked, openRow, setCells, setTitle, createRow, updateView, reorderRows }: ViewProps) {
  const shell = useDbShell();
  const { editDb, editView, trashRows, duplicateRow, openProp, focusRowId, clearFocusRow, patchRowLocal } = shell;
  const [liveWidths, setLiveWidths] = useState<Record<string, number>>({});
  const [active, setActive] = useState<{ id: string; c: number; key?: string } | null>(null);
  const [editing, setEditing] = useState(false);
  const [seed, setSeed] = useState<string | undefined>(undefined);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [addOpen, setAddOpen] = useState(false);
  const lastSelected = useRef<string | null>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const headRef = useRef<HTMLTableSectionElement>(null);
  const [headerHeight, setHeaderHeight] = useState(33);
  const pendingFocus = useRef(false);
  const instanceId = useId();
  const cellId = useCallback((rowId: string, col: number) => `${instanceId}-${rowId}-${col}`, [instanceId]);
  useLayoutEffect(() => {
    const head = headRef.current;
    if (!head) return;
    const measure = () => setHeaderHeight(head.getBoundingClientRect().height);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(head);
    return () => observer.disconnect();
  }, []);

  // ----- columns
  const cols: Col[] = useMemo(() => {
    const visible = view.columns.filter((c) => c.visible && db.props.some((p) => p.id === c.prop));
    visible.sort((a, b) => (a.prop === "title" ? -1 : b.prop === "title" ? 1 : 0));
    if (!visible.some((c) => c.prop === "title")) visible.unshift({ prop: "title", visible: true, width: 0 });
    let left = GUTTER;
    return visible.map((col, i) => {
      const prop = db.props.find((p) => p.id === col.prop)!;
      const width = liveWidths[col.prop] ?? (col.width || defaultWidth(prop));
      const frozen = i < Math.max(0, view.freeze);
      const out: Col = { col, prop, width, left: frozen ? left : null };
      left += width;
      return out;
    });
  }, [view.columns, view.freeze, view.rowNumbers, db.props, liveWidths]);
  const titleIdx = cols.findIndex((c) => c.prop.kind === "title");

  // ----- the rows in the order they are shown
  const groups = result.groups;
  const rowById = useMemo(() => new Map(result.rows.map((r) => [r.id, r])), [result.rows]);
  const collapsed = useMemo(() => new Set(view.group?.collapsed ?? []), [view.group]);
  const flat: DbRow[] = useMemo(() => {
    if (!groups) return result.rows;
    const out: DbRow[] = [];
    for (const g of groups) {
      if (collapsed.has(g.key)) continue;
      const lists = g.subs.length ? g.subs.filter((s) => !collapsed.has(g.key + "/" + s.key)).map((s) => s.rows) : [g.rows];
      for (const ids of lists) for (const id of ids) {
        const r = rowById.get(id);
        if (r) out.push(r);
      }
    }
    return out;
  }, [groups, result.rows, rowById, collapsed]);
  const indexOfRow = useMemo(() => new Map(flat.map((r, i) => [r.id, i])), [flat]);
  // Build lightweight descriptors once per result/view change, never React cells for
  // the full result. Headers, New lines and group totals occupy real scroll space.
  const items = useMemo(() => {
    const out: TableItem[] = [];
    const rows = (ids: string[], prefix: string) => {
      for (const id of ids) {
        const row = rowById.get(id);
        if (row) out.push({ kind: "row", key: `${prefix}:row:${id}`, row, rowIndex: indexOfRow.get(id) ?? 0 });
      }
    };
    if (!groups) {
      for (let i = 0; i < result.rows.length; i++) out.push({ kind: "row", key: `row:${result.rows[i].id}`, row: result.rows[i], rowIndex: i });
      out.push({ kind: "add", key: "add:end", cells: {} });
    } else {
      const groupProp = db.props.find((p) => p.id === view.group?.prop);
      const subProp = db.props.find((p) => p.id === view.subGroup?.prop);
      for (const g of groups) {
        out.push({ kind: "group", key: `group:${g.key}`, groupKey: g.key, group: g, level: 0 });
        if (collapsed.has(g.key)) continue;
        if (g.subs.length) {
          for (const s of g.subs) {
            const key = `${g.key}/${s.key}`;
            out.push({ kind: "group", key: `group:${key}`, groupKey: key, group: s, level: 1 });
            if (collapsed.has(key)) continue;
            rows(s.rows, key);
            out.push({ kind: "add", key: `add:${key}`, cells: { ...cellsForGroup(groupProp, view.group, g), ...cellsForGroup(subProp, view.subGroup, s) } });
            if (view.calcs.length) out.push({ kind: "footer", key: `footer:${key}`, calcs: s.calcs });
          }
        } else {
          rows(g.rows, g.key);
          out.push({ kind: "add", key: `add:${g.key}`, cells: cellsForGroup(groupProp, view.group, g) });
          if (view.calcs.length) out.push({ kind: "footer", key: `footer:${g.key}`, calcs: g.calcs });
        }
      }
    }
    return out;
  }, [groups, result.rows, rowById, indexOfRow, collapsed, db.props, view.group, view.subGroup, view.calcs]);
  const itemIndexOfRow = useMemo(() => {
    const map = new Map<string, number>();
    items.forEach((item, i) => { if (item.kind === "row") map.set(item.row.id, i); });
    return map;
  }, [items]);
  const itemIndexOfKey = useMemo(() => new Map(items.map((item, i) => [item.key, i])), [items]);
  const activeIndex = active ? (active.key ? itemIndexOfKey.get(active.key) : itemIndexOfRow.get(active.id)) : undefined;
  const dragIndex = draggingId ? itemIndexOfRow.get(draggingId) : undefined;
  const virtualizer = useVirtualizer<HTMLDivElement, HTMLTableRowElement>({
    count: items.length,
    getScrollElement: () => wrapRef.current,
    estimateSize: (i) => items[i].kind === "row" ? ROW_HEIGHT : items[i].kind === "group" ? 37 : 33,
    getItemKey: (i) => items[i].key,
    overscan: 6,
    scrollMargin: headerHeight,
    scrollPaddingStart: headerHeight,
    // Keep the active editor and drag source alive even outside the window. Gaps
    // between these and the window get their own spacers (no shifted row heights).
    rangeExtractor: useCallback((range) => {
      const indices = new Set(defaultRangeExtractor(range));
      if (activeIndex != null) indices.add(activeIndex);
      if (dragIndex != null) indices.add(dragIndex);
      return [...indices].sort((a, b) => a - b);
    }, [activeIndex, dragIndex]),
  });
  // A width/wrap change invalidates previously measured heights, including rows
  // that are currently off screen. Native scroll anchoring must not also adjust.
  useLayoutEffect(() => {
    virtualizer.measure();
    if (view.wrap) wrapRef.current?.querySelectorAll<HTMLTableRowElement>("tr[data-index]").forEach(virtualizer.measureElement);
  }, [virtualizer, cols, view.wrap]);
  const virtualRows = virtualizer.getVirtualItems();
  const canDrag = view.sorts.length === 0;
  // dnd-kit measures only mounted rows; drop resolution below uses virtual offsets
  // so an auto-scroll can target any row, including one not mounted at pickup.
  const dragRowIds = useMemo(() => [...new Set(virtualRows.flatMap((v) => items[v.index].kind === "row" ? [(items[v.index] as Extract<TableItem, { kind: "row" }>).row.id] : []))], [virtualRows, items]);
  const columnIds = useMemo(() => cols.map((c) => c.prop.id), [cols]);

  // ----- actions
  const commit = useCallback(
    (row: DbRow, prop: Property, value: unknown) => {
      if (prop.kind === "title") void setTitle(row.id, String(value ?? ""));
      else void setCells(row.id, { [prop.id]: value ?? null });
    },
    [setCells, setTitle],
  );

  const flatRef = useRef(flat);
  flatRef.current = flat;
  const colsRef = useRef(cols);
  colsRef.current = cols;
  const virtualRef = useRef({ virtualizer, itemIndexOfRow });
  virtualRef.current = { virtualizer, itemIndexOfRow };

  const select = useCallback((r: number, c: number, edit = false, itemKey?: string) => {
    const row = flatRef.current[r];
    if (!row) return;
    setActive({ id: row.id, c, key: itemKey });
    setEditing(edit);
    setSeed(undefined);
    pendingFocus.current = true;
    const { virtualizer: v, itemIndexOfRow: indices } = virtualRef.current;
    const index = indices.get(row.id);
    if (index != null) v.scrollToIndex(index, { align: "auto" });
  }, []);
  useLayoutEffect(() => {
    if (!pendingFocus.current || !active) return;
    if (activeIndex != null) virtualizer.scrollToIndex(activeIndex, { align: "auto" });
    const item = activeIndex == null ? undefined : items[activeIndex];
    const cell = item ? document.getElementById(cellId(item.key, active.c)) : null;
    if (!cell) return;
    pendingFocus.current = false;
    const wrap = wrapRef.current;
    if (wrap && cols[active.c]?.left == null) {
      const bounds = wrap.getBoundingClientRect();
      const rect = cell.getBoundingClientRect();
      const frozen = GUTTER + cols.filter((c) => c.left != null).reduce((sum, c) => sum + c.width, 0);
      const rtl = getComputedStyle(wrap).direction === "rtl";
      const left = bounds.left + (rtl ? 0 : frozen);
      const right = bounds.left + wrap.clientWidth - (rtl ? frozen : 0);
      if (rect.left < left) wrap.scrollLeft += rect.left - left;
      else if (rect.right > right) wrap.scrollLeft += rect.right - right;
    }
    // Interactive editors take their own focus. Selecting an ordinary cell must
    // not steal focus from a picker or focus every time a row is recycled.
    if (!editing) cell.focus({ preventScroll: true });
  });
  const toggleSelect = useCallback((id: string, shift: boolean) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (shift && lastSelected.current) {
        const a = flatRef.current.findIndex((r) => r.id === lastSelected.current);
        const b = flatRef.current.findIndex((r) => r.id === id);
        if (a >= 0 && b >= 0) for (let i = Math.min(a, b); i <= Math.max(a, b); i++) next.add(flatRef.current[i].id);
      } else if (next.has(id)) next.delete(id);
      else next.add(id);
      lastSelected.current = id;
      return next;
    });
  }, []);
  const endEdit = useCallback((how: "enter" | "tab" | "blur" | "escape") => {
    wrapRef.current?.focus({ preventScroll: true });
    if (how !== "enter" && how !== "tab") return;
    setActive((a) => {
      if (!a) return a;
      const r = flatRef.current.findIndex((row) => row.id === a.id);
      const nextRow = how === "enter" || a.c + 1 >= colsRef.current.length;
      const row = flatRef.current[Math.min(flatRef.current.length - 1, r + (nextRow ? 1 : 0))];
      return row ? { id: row.id, c: how === "enter" ? a.c : nextRow ? 0 : a.c + 1 } : null;
    });
    pendingFocus.current = true;
  }, []);
  const actions: Actions = useMemo(
    () => ({ select, setEditing, commit, patchLocal: patchRowLocal, open: openRow, toggleSelect, trash: (ids) => void trashRows(ids), duplicate: (id) => void duplicateRow(id), endEdit }),
    [select, commit, patchRowLocal, openRow, toggleSelect, trashRows, duplicateRow, endEdit],
  );

  // A freshly added row: edit its title.
  useEffect(() => {
    if (!focusRowId) return;
    const i = indexOfRow.get(focusRowId);
    if (i == null) {
      // New rows can land in a collapsed group. Reveal only their containing
      // group/subgroup; filtering and hidden groups remain the engine's decision.
      const group = groups?.find((g) => g.rows.includes(focusRowId) || g.subs.some((s) => s.rows.includes(focusRowId)));
      if (group && view.group) {
        const keys = new Set([group.key, ...group.subs.filter((s) => s.rows.includes(focusRowId)).map((s) => `${group.key}/${s.key}`)]);
        const next = view.group.collapsed.filter((key) => !keys.has(key));
        if (next.length !== view.group.collapsed.length) updateView({ group: { ...view.group, collapsed: next } });
      }
      return;
    }
    if (titleIdx < 0) return;
    select(i, titleIdx, true);
    clearFocusRow();
  }, [focusRowId, indexOfRow, titleIdx, clearFocusRow, select, groups, view.group, updateView]);

  // Keep the selection to rows that still exist.
  useEffect(() => {
    setSelected((s) => {
      const keep = new Set([...s].filter((id) => rowById.has(id)));
      return keep.size === s.size ? s : keep;
    });
    if (active && (activeIndex == null || !indexOfRow.has(active.id))) {
      setActive(null);
      setEditing(false);
      // A filtered/deleted/collapsed row cannot remain an active descendant.
      if (wrapRef.current?.contains(document.activeElement)) wrapRef.current.focus({ preventScroll: true });
    }
  }, [rowById, indexOfRow, active, activeIndex]);

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (editing || !active) return;
    if ((e.target as HTMLElement).closest("input,textarea,select,[role=dialog]")) return;
    const c = active.c;
    const r = indexOfRow.get(active.id) ?? -1;
    const col = cols[c];
    const row = flat[r];
    if (!col || !row) return;
    const move = (dr: number, dc: number) => {
      e.preventDefault();
      select(Math.max(0, Math.min(flat.length - 1, r + dr)), Math.max(0, Math.min(cols.length - 1, c + dc)));
    };
    const rtl = getComputedStyle(e.currentTarget).direction === "rtl";
    switch (e.key) {
      case "ArrowDown": return move(1, 0);
      case "ArrowUp": return move(-1, 0);
      case "ArrowRight": return move(0, rtl ? -1 : 1);
      case "ArrowLeft": return move(0, rtl ? 1 : -1);
      case "Home": return move(-r, 0);
      case "End": return move(flat.length - 1 - r, 0);
      case "PageDown": return move(Math.max(1, Math.floor(((wrapRef.current?.clientHeight ?? 350) - headerHeight) / ROW_HEIGHT)), 0);
      case "PageUp": return move(-Math.max(1, Math.floor(((wrapRef.current?.clientHeight ?? 350) - headerHeight) / ROW_HEIGHT)), 0);
      case "Tab": return move(0, e.shiftKey ? -1 : 1);
      case "Escape":
        e.preventDefault();
        setActive(null);
        setSelected(new Set());
        return;
      case "Enter":
      case "F2":
        e.preventDefault();
        if (col.prop.kind === "checkbox") commit(row, col.prop, row.values[col.prop.id] !== true);
        else if (!isComputedKind(col.prop.kind)) {
          setEditing(true);
          setSeed(undefined);
        }
        return;
      case " ":
        if (col.prop.kind === "checkbox") {
          e.preventDefault();
          commit(row, col.prop, row.values[col.prop.id] !== true);
        } else if (!isComputedKind(col.prop.kind) && !["title", "text", "number", "url", "email", "phone"].includes(col.prop.kind)) {
          e.preventDefault();
          setEditing(true);
        }
        return;
      case "Delete":
      case "Backspace":
        if (selected.size > 0 && !e.metaKey) {
          e.preventDefault();
          void trashRows([...selected]);
        } else if (col.prop.kind !== "title" && !isComputedKind(col.prop.kind)) {
          e.preventDefault();
          commit(row, col.prop, null);
        }
        return;
    }
    if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey && ["title", "text", "number", "url", "email", "phone"].includes(col.prop.kind)) {
      e.preventDefault();
      setEditing(true);
      setSeed(e.key);
    }
  };

  // ----- column header actions
  const setWidth = (propId: string, w: number, done: boolean) => {
    if (!done) {
      setLiveWidths((l) => ({ ...l, [propId]: w }));
      return;
    }
    setLiveWidths((l) => {
      const n = { ...l };
      delete n[propId];
      return n;
    });
    updateView({ columns: view.columns.map((c) => (c.prop === propId ? { ...c, width: Math.round(w) } : c)) });
  };
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 6 } }), useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }));
  const onColumnDrag = (e: DragEndEvent) => {
    if (!e.over || e.active.id === e.over.id) return;
    const ids = view.columns.map((c) => c.prop);
    const from = ids.indexOf(String(e.active.id));
    const to = ids.indexOf(String(e.over.id));
    if (from < 0 || to < 0 || String(e.over.id) === "title") return;
    const cs = [...view.columns];
    const [m] = cs.splice(from, 1);
    cs.splice(to, 0, m);
    updateView({ columns: cs });
  };
  const onRowDrag = (e: DragEndEvent) => {
    if (!canDrag) return;
    let target = e.over ? String(e.over.id) : null;
    const rect = e.active.rect.current.translated;
    const wrap = wrapRef.current;
    if (rect && wrap) {
      const offset = rect.top + rect.height / 2 - wrap.getBoundingClientRect().top + wrap.scrollTop;
      const hit = virtualizer.getVirtualItemForOffset(offset);
      if (hit) {
        // A group/add/footer line is not a drop destination. Resolve to its
        // nearest data row without asking dnd-kit for an unmounted DOM node.
        for (let distance = 0; distance < items.length; distance++) {
          const before = items[hit.index - distance];
          const after = items[hit.index + distance];
          const item = after?.kind === "row" ? after : before?.kind === "row" ? before : null;
          if (item?.kind === "row") { target = item.row.id; break; }
        }
      }
    }
    if (!target || String(e.active.id) === target) return;
    const ids = result.rows.map((r) => r.id);
    const from = ids.indexOf(String(e.active.id));
    const to = ids.indexOf(target);
    if (from < 0 || to < 0) return;
    const next = [...ids];
    const [m] = next.splice(from, 1);
    next.splice(to, 0, m);
    void reorderRows(next);
  };

  const addProperty = (p: Property, openMenu: boolean) => {
    void editDb(
      (d) => ({ ...d, props: [...d.props, p], views: d.views.map((v) => ({ ...v, columns: [...v.columns, { prop: p.id, visible: v.id === view.id, width: 0 }] })) }),
      true,
    ).then(() => openMenu && openProp(p.id));
  };

  const setCalc = (propId: string, calc: string) => {
    const rest = view.calcs.filter((c) => c.prop !== propId);
    editView((v) => ({ ...v, calcs: calc === "none" ? rest : [...rest, { prop: propId, calc }] }));
  };

  // ----- rendering helpers
  const groupProp = view.group ? db.props.find((p) => p.id === view.group!.prop) : undefined;
  const subProp = view.subGroup ? db.props.find((p) => p.id === view.subGroup!.prop) : undefined;
  const totalCols = cols.length + 2;

  const toggleCollapsed = (key: string) => {
    if (!view.group) return;
    const cur = view.group.collapsed;
    updateView({ group: { ...view.group, collapsed: cur.includes(key) ? cur.filter((k) => k !== key) : [...cur, key] } });
  };
  const hideGroup = (key: string) => {
    if (view.group) updateView({ group: { ...view.group, hidden: [...view.group.hidden, key] } });
  };

  const addRow = async (cells: Record<string, unknown>) => {
    const row = await createRow(cells, "");
    if (row) shell.focusRow(row.id); // the table starts editing its title once the row is on screen
  };

  const renderRow = (item: Extract<TableItem, { kind: "row" }>, virtualIndex: number) => {
      const r = item.row;
      const idx = item.rowIndex;
      return (
        <RowView
          key={item.key}
          row={r}
          index={idx}
          cols={cols}
          dbId={db.id}
          wrap={view.wrap}
          numbers={view.rowNumbers}
          selected={selected.has(r.id)}
          activeCol={activeIndex === virtualIndex ? active!.c : -1}
          editing={activeIndex === virtualIndex && editing}
          seed={activeIndex === virtualIndex ? seed : undefined}
          canDrag={canDrag}
          actions={actions}
          virtualIndex={virtualIndex}
          ariaIndex={virtualIndex + 2}
          cellId={cellId}
          measure={view.wrap ? virtualizer.measureElement : undefined}
          itemKey={item.key}
        />
      );
  };

  const footerRow = (calcs: Record<string, string>, key?: string, index?: number) => (
    <tr key={key} data-index={index} aria-rowindex={index == null ? items.length + 2 : index + 2} data-table-footer="" ref={view.wrap && index != null ? virtualizer.measureElement : undefined}>
      <td className={cn("db-cell sticky z-[2] h-8", BG)} style={{ insetInlineStart: 0 }} />
      {cols.map((c) => {
        const cfg = view.calcs.find((x) => x.prop === c.prop.id);
        const text = calcs[c.prop.id];
        const sticky = c.left != null;
        return (
          <td key={c.prop.id} className={cn("db-cell h-8 p-0", BG, sticky && "sticky z-[1]")} style={sticky ? { insetInlineStart: c.left! } : undefined}>
            <Menu>
              <MenuTrigger asChild>
                <button type="button" aria-label={`Calculate ${c.prop.name}`} className="flex h-8 w-full items-center justify-end gap-1.5 px-2 text-xs text-muted opacity-100 hover:bg-surface-2">
                  {cfg ? (
                    <>
                      <span className="text-faint">{CALC_LABELS[cfg.calc]}</span>
                      <span className="font-medium tabular-nums text-fg" dir="auto">{text ?? ""}</span>
                    </>
                  ) : (
                    <span className="text-faint opacity-0 hover:opacity-100 focus-visible:opacity-100 [tr:hover_&]:opacity-60">Calculate</span>
                  )}
                </button>
              </MenuTrigger>
              <MenuContent align="start">
                {calcsFor(c.prop.kind).map((k) => (
                  <MenuItem key={k} onSelect={() => setCalc(c.prop.id, k)}>{CALC_LABELS[k]}</MenuItem>
                ))}
              </MenuContent>
            </Menu>
          </td>
        );
      })}
      <td className="db-cell" />
    </tr>
  );

  const groupHeader = (g: GroupOut, key: string, prop: Property | undefined, spec: GroupBy | null, level: number, index: number, itemKey: string) => {
    const isCollapsed = collapsed.has(key);
    const opt: SelectOption | undefined = prop?.options.find((o) => o.id === g.key);
    return (
      <tr key={itemKey} className="group/gh" data-index={index} aria-rowindex={index + 2} data-group-key={key} ref={view.wrap ? virtualizer.measureElement : undefined}>
        <td colSpan={totalCols} className={cn("db-cell p-0", BG)}>
          <div className="sticky flex h-9 items-center gap-1.5 ps-3 pe-3" style={{ insetInlineStart: 0, width: "max-content", paddingInlineStart: 12 + level * 22 }}>
            <button type="button" aria-expanded={!isCollapsed} aria-label={isCollapsed ? "Expand group" : "Collapse group"} onClick={() => toggleCollapsed(key)} className="rounded p-0.5 text-muted hover:bg-surface-2 hover:text-fg">
              {isCollapsed ? <ChevronRight size={14} className="rtl:rotate-180" /> : <ChevronDown size={14} />}
            </button>
            {opt ? <OptionChip option={opt} /> : g.color ? <OptionChip option={{ name: g.label, color: g.color as never }} /> : <span dir="auto" className={cn("text-sm font-medium", g.key === "" && "text-muted")}>{g.label}</span>}
            <span className="text-xs tabular-nums text-faint">{g.count}</span>
            {!locked && level === 0 && (
              <Menu>
                <MenuTrigger asChild>
                  <button type="button" aria-label="Group options" className="rounded p-0.5 text-faint opacity-0 hover:text-fg focus-visible:opacity-100 group-hover/gh:opacity-100">
                    <MoreHorizontal size={14} />
                  </button>
                </MenuTrigger>
                <MenuContent align="start">
                  <MenuItem icon={<Plus size={14} />} onSelect={() => void addRow(cellsForGroup(prop, spec, g))}>New row in this group</MenuItem>
                  <MenuItem icon={<X size={14} />} onSelect={() => hideGroup(g.key)}>Hide group</MenuItem>
                </MenuContent>
              </Menu>
            )}
          </div>
        </td>
      </tr>
    );
  };

  const addLine = (cells: Record<string, unknown>, key: string, index: number) => (
    <tr key={key} data-index={index} aria-rowindex={index + 2} ref={view.wrap ? virtualizer.measureElement : undefined}>
      <td colSpan={totalCols} className={cn("db-cell p-0", BG)}>
        <button
          type="button"
          onClick={() => void addRow(cells)}
          className="sticky flex h-8 items-center gap-1.5 px-3 text-sm text-faint hover:text-fg"
          style={{ insetInlineStart: GUTTER }}
        >
          <Plus size={14} /> New
        </button>
      </td>
    </tr>
  );

  const spacer = (height: number, key: string) => height > 0 ? (
    <tr key={key} aria-hidden="true" role="presentation" data-virtual-spacer="">
      <td colSpan={totalCols} style={{ height, padding: 0, border: 0, lineHeight: 0 }} />
    </tr>
  ) : null;
  const body: React.ReactNode[] = [];
  let previousEnd = 0;
  for (const v of virtualRows) {
    const item = items[v.index];
    body.push(spacer(v.start - headerHeight - previousEnd, `space:${item.key}`));
    if (item.kind === "row") body.push(renderRow(item, v.index));
    else if (item.kind === "group") body.push(groupHeader(item.group, item.groupKey, item.level ? subProp : groupProp, item.level ? view.subGroup : view.group, item.level, v.index, item.key));
    else if (item.kind === "add") body.push(addLine(item.cells, item.key, v.index));
    else body.push(footerRow(item.calcs, item.key, v.index));
    previousEnd = v.end - headerHeight;
  }
  body.push(spacer(virtualizer.getTotalSize() - previousEnd, "space:end"));

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {selected.size > 0 && (
        <div className="sticky top-0 z-20 flex items-center gap-2 border-b border-line bg-elevated px-4 py-1.5 text-sm">
          <span className="font-medium">{selected.size} selected</span>
          <button type="button" onClick={() => void trashRows([...selected]).then(() => setSelected(new Set()))} className="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-danger hover:bg-danger/10">
            <Trash2 size={14} /> Move to trash
          </button>
          <button type="button" onClick={() => setSelected(new Set())} className="ms-auto rounded-md px-2 py-1 text-muted hover:bg-surface-2 hover:text-fg">Clear selection</button>
        </div>
      )}
      <div ref={wrapRef} tabIndex={0} onKeyDown={onKeyDown} role="application" aria-label="Table" className="db-scroll min-h-0 flex-1 overflow-auto outline-none" style={{ overflowAnchor: "none", ...(shell.compact ? { maxHeight: 480 } : {}) }}>
        <DndContext sensors={sensors} collisionDetection={closestCenter} onDragStart={(e) => { if (itemIndexOfRow.has(String(e.active.id))) setDraggingId(String(e.active.id)); }} onDragCancel={() => setDraggingId(null)} onDragEnd={(e) => { setDraggingId(null); if (view.columns.some((c) => c.prop === String(e.active.id))) onColumnDrag(e); else onRowDrag(e); }}>
          <table className="db-table text-sm" data-has-selection={selected.size > 0} style={{ tableLayout: "fixed", width: GUTTER + cols.reduce((n, c) => n + c.width, 0) + 44 }} role="grid" aria-rowcount={items.length + 1 + (groups ? 0 : 1)} aria-colcount={totalCols}>
            <colgroup>
              <col style={{ width: GUTTER }} />
              {cols.map((c) => (
                <col key={c.prop.id} style={{ width: c.width }} />
              ))}
              <col style={{ width: 44 }} />
            </colgroup>
            <thead ref={headRef}>
              <tr aria-rowindex={1}>
                <th className={cn("db-cell sticky top-0 z-[5] border-t border-t-line p-0", BG)} style={{ insetInlineStart: 0 }}>
                  <div className="flex h-8 items-center justify-end pe-2">
                    <button
                      type="button"
                      role="checkbox"
                      aria-checked={selected.size > 0 && selected.size === flat.length}
                      aria-label="Select all rows"
                      onClick={() => setSelected(selected.size === flat.length ? new Set() : new Set(flat.map((r) => r.id)))}
                      className={cn("rounded", selected.size > 0 ? "" : "opacity-0 focus-visible:opacity-100 [table:hover_&]:opacity-60")}
                    >
                      <CheckBox checked={selected.size > 0 && selected.size === flat.length} />
                    </button>
                  </div>
                </th>
                <SortableContext items={columnIds} strategy={horizontalListSortingStrategy}>
                  {cols.map((c, ci) => (
                    <HeaderCell key={c.prop.id} c={c} ci={ci} locked={locked} wrapTitle={view.wrap} onResize={(w, done) => setWidth(c.prop.id, w, done)} onOpen={() => openProp(c.prop.id)} />
                  ))}
                </SortableContext>
                <th className={cn("db-cell sticky top-0 z-[3] border-t border-t-line p-0", BG)}>
                  {!locked && (
                    <Popover.Root open={addOpen} onOpenChange={setAddOpen}>
                      <Popover.Trigger asChild>
                        <button type="button" aria-label="Add a property" title="Add a property" className="grid h-8 w-full place-items-center text-muted hover:bg-surface-2 hover:text-fg">
                          <Plus size={15} />
                        </button>
                      </Popover.Trigger>
                      <Popover.Portal>
                        <Popover.Content align="end" sideOffset={4} collisionPadding={12} className="z-[80] w-[320px] rounded-xl border border-line bg-elevated p-2.5 shadow-float outline-none">
                          <NewPropertyPicker db={db} databases={shell.databases} onCreate={addProperty} onClose={() => setAddOpen(false)} />
                        </Popover.Content>
                      </Popover.Portal>
                    </Popover.Root>
                  )}
                </th>
              </tr>
            </thead>
            <SortableContext items={canDrag ? dragRowIds : NO_DRAG_ROWS} strategy={verticalListSortingStrategy}>
              <tbody>{body}</tbody>
            </SortableContext>
            {/* Every column offers a calculation at the foot, shown when the row is hovered or the column has one. */}
            {!groups && <tfoot>{footerRow(result.calcs ?? {})}</tfoot>}
          </table>
        </DndContext>
      </div>
      {result.rows.length === 0 && !groups && (
        <p className="shrink-0 px-4 py-3 text-sm text-faint">This table has no rows yet.</p>
      )}
      <span className="sr-only" aria-live="polite">{selected.size > 0 ? `${selected.size} rows selected` : ""}</span>
    </div>
  );
}
