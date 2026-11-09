// Board view: one column per group of the view's "group by" property; drag a card to another column to change
// that property, drag it up or down to change the manual order. Optional swimlanes from the sub-group.

import { memo, useCallback, useMemo, useRef, useState } from "react";
import {
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  closestCorners,
  useDroppable,
  useSensor,
  useSensors,
  type DragEndEvent,
  type DragOverEvent,
  type DragStartEvent,
} from "@dnd-kit/core";
import { SortableContext, sortableKeyboardCoordinates, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { ChevronDown, ChevronRight, Columns3, EyeOff } from "lucide-react";
import type { DbRow, GroupBy, GroupOut, Property, View } from "@/lib/database-types";
import type { ViewProps } from "../viewTypes";
import { cn } from "@/lib/utils";
import {
  EmptyState,
  GroupChip,
  NewButton,
  NewTitleInput,
  PropLines,
  PropSelect,
  RowTitle,
  canGroupEdit,
  firstImage,
  groupPatch,
  groupProp,
  imageUrl,
  propOf,
  setCollapsed,
  titleMap,
  useDragClickGuard,
  visibleProps,
} from "./shared";

const SEP = "\u0001";
const WIDTHS: Record<string, number> = { small: 220, medium: 272, large: 344 };
const GROUPABLE = ["select", "status", "multiSelect", "person", "checkbox"];

interface Lane {
  key: string;
  label: string;
}

interface Cell {
  id: string;
  col: GroupOut;
  lane: Lane | null;
  /** Card ids (a row can be in several cells when the property is multi-select). */
  ids: string[];
}

export default function BoardView(props: ViewProps) {
  const { db, view, result, updateView } = props;
  const g = view.group;
  const gp = groupProp(db, g);
  if (!g || !gp) {
    const candidates = db.props.filter((p) => GROUPABLE.includes(p.kind));
    return (
      <EmptyState icon={<Columns3 size={28} />} title="Choose how to group the board">
        {candidates.length ? (
          <>
            <span>Each value of the property becomes a column.</span>
            <PropSelect
              label="Group by"
              placeholder="Group by…"
              props={candidates}
              value=""
              onChange={(id) => {
                const p = propOf(db, id);
                if (!p) return;
                updateView({ group: { prop: id, sort: "manual", dateBy: "", numberStep: 0, hideEmpty: false, hidden: [], collapsed: [] } });
              }}
            />
          </>
        ) : (
          <span>Add a select, status, multi-select, person or checkbox property to group by.</span>
        )}
      </EmptyState>
    );
  }
  if (!result.groups) return <div className="p-6 text-sm text-muted">Loading the board…</div>;
  return <BoardInner {...props} g={g} gp={gp} groups={result.groups} />;
}

function BoardInner({ db, view, result, rowById, openRow, setCells, createRow, updateView, reorderRows, g, gp, groups }: ViewProps & { g: GroupBy; gp: Property; groups: GroupOut[] }) {
  const sub = view.subGroup && groupProp(db, view.subGroup) ? view.subGroup : null;
  const subProp = sub ? groupProp(db, sub) : undefined;
  const width = WIDTHS[view.cardSize] ?? WIDTHS.medium;
  const cover = propOf(db, view.cardCover)?.kind === "files" ? view.cardCover : "";
  const shown = useMemo(() => visibleProps(db, view, [g.prop, sub?.prop]), [db, view, g.prop, sub?.prop]);
  const titles = useMemo(() => titleMap(result.rows), [result.rows]);
  const guard = useDragClickGuard();
  const openCard = useCallback((id: string) => {
    if (!guard.recent()) openRow(id);
  }, [guard, openRow]);
  const [adding, setAdding] = useState<string | null>(null);
  const [laneClosed, setLaneClosed] = useState<Set<string>>(new Set());

  const canMove = canGroupEdit(gp, g);
  const canMoveSub = !sub || canGroupEdit(subProp, sub);

  /* ----- the cells ----- */
  const lanes: Lane[] | null = useMemo(() => {
    if (!sub) return null;
    const seen = new Map<string, string>();
    for (const c of groups) for (const s of c.subs) if (!seen.has(s.key)) seen.set(s.key, s.label);
    return [...seen].map(([key, label]) => ({ key, label }));
  }, [groups, sub]);

  const cells: Cell[] = useMemo(() => {
    const out: Cell[] = [];
    const dup = new Map<string, number>();
    const add = (col: GroupOut, lane: Lane | null, rows: string[]) => {
      const id = col.key + SEP + (lane?.key ?? "");
      rows.forEach((r) => dup.set(r, (dup.get(r) ?? 0) + 1));
      out.push({ id, col, lane, ids: rows.map((r) => r) });
    };
    for (const col of groups) {
      if (lanes) for (const l of lanes) add(col, l, col.subs.find((s) => s.key === l.key)?.rows ?? []);
      else add(col, null, col.rows);
    }
    // A row that is in several cells needs one card id per cell.
    return out.map((c) => ({ ...c, ids: c.ids.map((r) => ((dup.get(r) ?? 0) > 1 ? `${r}@${c.id}` : r)) }));
  }, [groups, lanes]);

  const rowOfCard = (cardId: string) => cardId.split("@")[0];
  const base = useMemo(() => Object.fromEntries(cells.map((c) => [c.id, c.ids])) as Record<string, string[]>, [cells]);
  const [dragItems, setDragItems] = useState<Record<string, string[]> | null>(null);
  const [active, setActive] = useState<string | null>(null);
  const startCell = useRef<string>("");
  const items = dragItems ?? base;

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 6 } }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
      keyboardCodes: { start: ["Space"], cancel: ["Escape"], end: ["Space", "Enter"] },
    }),
  );

  const containerOf = useCallback(
    (id: string, from: Record<string, string[]>) => (id in from ? id : Object.keys(from).find((k) => from[k].includes(id))),
    [],
  );

  const onDragStart = (e: DragStartEvent) => {
    const id = String(e.active.id);
    setActive(id);
    startCell.current = containerOf(id, base) ?? "";
    setDragItems(Object.fromEntries(Object.entries(base).map(([k, v]) => [k, [...v]])));
  };

  const onDragOver = (e: DragOverEvent) => {
    const { active: a, over } = e;
    if (!over || !dragItems) return;
    const aid = String(a.id);
    const oid = String(over.id);
    const from = containerOf(aid, dragItems);
    const to = containerOf(oid, dragItems);
    if (!from || !to || from === to) return;
    if (!canMove || !canMoveSub) return;
    if (dragItems[to].some((x) => rowOfCard(x) === rowOfCard(aid))) return;
    setDragItems((cur) => {
      if (!cur) return cur;
      const src = cur[from].filter((x) => x !== aid);
      const dst = [...cur[to]];
      const at = dst.indexOf(oid);
      const rect = a.rect.current.translated;
      const below = !!rect && !!over.rect && rect.top > over.rect.top + over.rect.height / 2;
      dst.splice(at >= 0 ? at + (below ? 1 : 0) : dst.length, 0, aid);
      return { ...cur, [from]: src, [to]: dst };
    });
  };

  const onDragEnd = (e: DragEndEvent) => {
    guard.mark();
    const aid = String(e.active.id);
    let fin = dragItems ?? base;
    const over = e.over ? String(e.over.id) : null;
    const to0 = containerOf(aid, fin);
    // Reorder inside one cell: apply the final position here (dragOver doesn't move within a cell).
    if (over && to0 && over !== aid && fin[to0]?.includes(over)) {
      const list = fin[to0];
      const oldI = list.indexOf(aid);
      const newI = list.indexOf(over);
      if (oldI >= 0 && newI >= 0 && oldI !== newI) {
        const next = [...list];
        next.splice(oldI, 1);
        next.splice(newI, 0, aid);
        fin = { ...fin, [to0]: next };
      }
    }
    setActive(null);
    setDragItems(null);
    const to = containerOf(aid, fin);
    if (!e.over || !to) return;
    const from = startCell.current;
    const rowId = rowOfCard(aid);
    const row = rowById.get(rowId);
    if (row && to !== from) {
      const toCell = cells.find((c) => c.id === to);
      const fromCell = cells.find((c) => c.id === from);
      if (toCell && fromCell) {
        let patch: Record<string, unknown> = {};
        if (toCell.col.key !== fromCell.col.key) {
          const p = groupPatch(gp, g, toCell.col.key, toCell.col.label, row, { key: fromCell.col.key, label: fromCell.col.label });
          if (p) patch = { ...patch, ...p };
        }
        if (toCell.lane && fromCell.lane && toCell.lane.key !== fromCell.lane.key && sub) {
          const p = groupPatch(subProp, sub, toCell.lane.key, toCell.lane.label, row, { key: fromCell.lane.key, label: fromCell.lane.label });
          if (p) patch = { ...patch, ...p };
        }
        if (Object.keys(patch).length) void setCells(rowId, patch);
      }
    }
    // Manual order: the cards as laid out now, then any rows not on the board.
    if (!view.sorts.length) {
      const order: string[] = [];
      const seen = new Set<string>();
      for (const c of cells) for (const id of fin[c.id] ?? []) {
        const r = rowOfCard(id);
        if (!seen.has(r)) {
          seen.add(r);
          order.push(r);
        }
      }
      for (const r of result.rows) if (!seen.has(r.id)) order.push(r.id);
      const before = result.rows.map((r) => r.id);
      if (order.some((id, i) => id !== before[i])) void reorderRows(order);
    }
  };

  /* ----- new cards ----- */
  const cellsFor = (cell: Cell): Record<string, unknown> => {
    const a = groupPatch(gp, g, cell.col.key, cell.col.label) ?? {};
    const b = cell.lane ? groupPatch(subProp, sub, cell.lane.key, cell.lane.label) ?? {} : {};
    return Object.fromEntries(Object.entries({ ...a, ...b }).filter(([, v]) => v !== null));
  };

  const patchGroup = (p: Partial<GroupBy>) => updateView({ group: { ...g, ...p } });

  const activeRow = active ? rowById.get(rowOfCard(active)) : undefined;
  const hiddenKeys = g.hidden;
  const hiddenLabel = (key: string): string => {
    if (gp.kind === "checkbox") return key === "true" ? "Checked" : "Unchecked";
    if (gp.kind === "status" && g.dateBy === "group") return key === "inProgress" ? "In progress" : key === "complete" ? "Complete" : "To-do";
    if (!key) return `No ${gp.name}`;
    return gp.options.find((o) => o.id === key)?.name ?? key;
  };

  if (!groups.length && !hiddenKeys.length) {
    return (
      <EmptyState icon={<Columns3 size={28} />} title="Nothing to show">
        <span>There are no columns yet.</span>
      </EmptyState>
    );
  }

  const tpl = groups.map((c) => (g.collapsed.includes(c.key) ? "2.75rem" : `${width}px`)).join(" ");
  const renderCell = (cell: Cell) => {
    const closed = g.collapsed.includes(cell.col.key);
    if (closed) return <div key={cell.id} className="rounded-xl border border-dashed border-line/60" aria-hidden />;
    return (
      <BoardCell
        key={cell.id}
        cell={cell}
        ids={items[cell.id] ?? []}
        rowById={rowById}
        width={width}
        cover={cover}
        fit={view.cardFit}
        props={shown}
        titles={titles}
        gp={gp}
        onOpen={openCard}
        canDrag={view.sorts.length === 0 || (canMove && canMoveSub)}
        adding={adding === cell.id}
        onAdd={() => setAdding(cell.id)}
        onCancelAdd={() => setAdding((a) => (a === cell.id ? null : a))}
        onCreate={(title) => void createRow(cellsFor(cell), title)}
      />
    );
  };

  return (
    <div className="h-full overflow-auto px-4 pb-6 pt-2" role="region" aria-label="Board">
      <DndContext
        sensors={sensors}
        collisionDetection={closestCorners}
        onDragStart={onDragStart}
        onDragOver={onDragOver}
        onDragEnd={onDragEnd}
        onDragCancel={() => {
          setActive(null);
          setDragItems(null);
        }}
      >
        <div className="flex items-start gap-3">
          <div className="grid items-start gap-x-3 gap-y-2" style={{ gridTemplateColumns: tpl }}>
            {groups.map((col) => {
              const closed = g.collapsed.includes(col.key);
              return closed ? (
                <button
                  key={col.key + "h"}
                  type="button"
                  title={`Expand ${col.label}`}
                  aria-label={`Expand ${col.label}`}
                  onClick={() => {
                    const p = setCollapsed(view as View, col.key, false);
                    if (p) updateView(p);
                  }}
                  className="flex min-h-[8rem] flex-col items-center gap-2 rounded-xl bg-surface px-1 py-2 text-muted hover:bg-surface-2"
                >
                  <ChevronRight size={14} className="rtl:rotate-180" />
                  <span className="text-xs tabular-nums">{col.count}</span>
                  <span dir="auto" className="text-[0.8rem] font-medium [writing-mode:vertical-rl]">{col.label}</span>
                </button>
              ) : (
                <div key={col.key + "h"} className="group/h flex h-7 min-w-0 items-center gap-1.5 px-1" data-column={col.key}>
                  <GroupChip g={col} />
                  <span className="text-xs tabular-nums text-faint" data-count>{col.count}</span>
                  <span className="ms-auto flex items-center opacity-0 transition-opacity group-hover/h:opacity-100 focus-within:opacity-100">
                    <button
                      type="button"
                      aria-label={`Collapse ${col.label}`}
                      title="Collapse"
                      className="grid h-6 w-6 place-items-center rounded-md text-faint hover:bg-surface-2 hover:text-fg"
                      onClick={() => {
                        const p = setCollapsed(view, col.key, true);
                        if (p) updateView(p);
                      }}
                    >
                      <ChevronDown size={14} />
                    </button>
                    <button
                      type="button"
                      aria-label={`Hide ${col.label}`}
                      title="Hide column"
                      className="grid h-6 w-6 place-items-center rounded-md text-faint hover:bg-surface-2 hover:text-fg"
                      onClick={() => patchGroup({ hidden: [...g.hidden, col.key] })}
                    >
                      <EyeOff size={13} />
                    </button>
                  </span>
                </div>
              );
            })}

            {lanes
              ? lanes.map((lane) => {
                  const closed = laneClosed.has(lane.key);
                  const count = groups.reduce((n, c) => n + (c.subs.find((s) => s.key === lane.key)?.count ?? 0), 0);
                  return (
                    <LaneRows key={lane.key} span={groups.length}>
                      <button
                        type="button"
                        aria-expanded={!closed}
                        onClick={() =>
                          setLaneClosed((s) => {
                            const n = new Set(s);
                            if (n.has(lane.key)) n.delete(lane.key);
                            else n.add(lane.key);
                            return n;
                          })
                        }
                        className="sticky start-0 mt-2 flex w-fit items-center gap-1.5 rounded-md px-1 py-0.5 text-[0.82rem] font-medium hover:bg-surface-2"
                        style={{ gridColumn: "1 / -1" }}
                      >
                        {closed ? <ChevronRight size={14} className="rtl:rotate-180" /> : <ChevronDown size={14} />}
                        <span dir="auto">{lane.label}</span>
                        <span className="text-xs font-normal tabular-nums text-faint">{count}</span>
                      </button>
                      {closed ? null : cells.filter((c) => c.lane?.key === lane.key).map(renderCell)}
                    </LaneRows>
                  );
                })
              : cells.map(renderCell)}
          </div>

          {hiddenKeys.length ? (
            <div className="flex w-44 shrink-0 flex-col gap-1 pt-0.5" aria-label="Hidden columns">
              <div className="px-1 text-xs text-faint">Hidden columns</div>
              {hiddenKeys.map((k) => (
                <button
                  key={k}
                  type="button"
                  onClick={() => patchGroup({ hidden: g.hidden.filter((x) => x !== k) })}
                  className="flex items-center gap-1.5 rounded-lg px-2 py-1 text-start text-[0.8rem] text-muted hover:bg-surface-2 hover:text-fg"
                  title="Show this column"
                >
                  <EyeOff size={12} className="shrink-0" />
                  <span dir="auto" className="truncate">{hiddenLabel(k)}</span>
                </button>
              ))}
            </div>
          ) : null}
        </div>

        <DragOverlay dropAnimation={null}>
          {activeRow ? (
            <div style={{ width }} className="cursor-grabbing">
              <CardBody row={activeRow} cover={cover} fit={view.cardFit} props={shown} titles={titles} overlay />
            </div>
          ) : null}
        </DragOverlay>
      </DndContext>
    </div>
  );
}

/** Lets a swimlane's header and cells join the grid of the board. */
function LaneRows({ children }: { children: React.ReactNode; span: number }) {
  return <>{children}</>;
}

/* ---------- One cell (a column, or a column inside a swimlane) ---------- */

function BoardCell({
  cell,
  ids,
  rowById,
  width,
  cover,
  fit,
  props,
  titles,
  gp,
  canDrag,
  onOpen,
  adding,
  onAdd,
  onCancelAdd,
  onCreate,
}: {
  cell: Cell;
  ids: string[];
  rowById: Map<string, DbRow>;
  width: number;
  cover: string;
  fit: boolean;
  props: Property[];
  titles: Map<string, string>;
  gp: Property;
  canDrag: boolean;
  onOpen: (id: string) => void;
  adding: boolean;
  onAdd: () => void;
  onCancelAdd: () => void;
  onCreate: (title: string) => void;
}) {
  void width;
  void gp;
  const { setNodeRef, isOver } = useDroppable({ id: cell.id });
  return (
    <div
      ref={setNodeRef}
      data-cell={cell.id}
      className={cn("flex min-h-[4.5rem] min-w-0 flex-col gap-2 rounded-xl bg-surface p-1.5 transition-colors", isOver && "bg-surface-2")}
    >
      <SortableContext items={ids} strategy={verticalListSortingStrategy}>
        {ids.map((id) => {
          const row = rowById.get(id.split("@")[0]);
          return row ? (canDrag
            ? <SortableCard key={id} id={id} cell={cell.id} row={row} cover={cover} fit={fit} props={props} titles={titles} onOpen={onOpen} />
            : <CardFrame key={id} row={row} cover={cover} fit={fit} props={props} titles={titles} onOpen={onOpen} />) : null;
        })}
      </SortableContext>
      {adding ? (
        <NewTitleInput keepOpen onSubmit={onCreate} onCancel={onCancelAdd} />
      ) : (
        <NewButton onClick={onAdd} className="w-full" />
      )}
    </div>
  );
}

interface CardProps {
  row: DbRow; cover: string; fit: boolean; props: Property[]; titles: Map<string, string>;
  onOpen: (rowId: string) => void;
}

const SortableCard = memo(function SortableCard({ id, cell, ...props }: CardProps & { id: string; cell: string }) {
  const sortable = useSortable({ id, data: { cell } });
  return <CardFrame {...props} sortable={sortable} />;
});

const CardFrame = memo(function CardFrame({ row, cover, fit, props, titles, onOpen, sortable }: CardProps & { sortable?: ReturnType<typeof useSortable> }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = sortable ?? {};
  return (
    <div
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform ?? null), transition }}
      {...attributes}
      {...listeners}
      role="button"
      tabIndex={0}
      data-card={row.id}
      aria-label={row.title || "Untitled"}
      onClick={() => onOpen(row.id)}
      onKeyDown={(e) => {
        if (e.key === "Enter" && e.target === e.currentTarget) {
          e.preventDefault();
          onOpen(row.id);
          return;
        }
        listeners?.onKeyDown?.(e);
      }}
      className={cn("touch-none rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-accent", isDragging && "opacity-40")}
    >
      <CardBody row={row} cover={cover} fit={fit} props={props} titles={titles} />
    </div>
  );
});

const CardBody = memo(function CardBody({ row, cover, fit, props, titles, overlay }: { row: DbRow; cover: string; fit: boolean; props: Property[]; titles: Map<string, string>; overlay?: boolean }) {
  const img = cover ? firstImage(row, cover) : null;
  return (
    <div className={cn("overflow-hidden rounded-lg border border-line bg-elevated text-[0.85rem] transition-shadow hover:shadow-float", overlay && "shadow-float")}>
      {img ? <img src={imageUrl(img.id)} alt="" draggable={false} loading="lazy" className={cn("h-28 w-full", fit ? "bg-surface object-contain" : "object-cover")} /> : null}
      <div className="flex flex-col gap-1.5 px-2.5 py-2">
        <RowTitle row={row} className="font-medium" />
        <PropLines row={row} props={props} titles={titles} />
      </div>
    </div>
  );
});
