// Board view: one column per group of the view's "group by" property; drag a card to another column to change
// that property, drag it up or down to change the manual order. Optional swimlanes from the sub-group.

import { memo, useCallback, useContext, useEffect, useLayoutEffect, useMemo, useRef, useState, type CSSProperties, type RefObject } from "react";
import {
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  closestCorners,
  useDroppable,
  useSensor,
  useSensors,
  type CollisionDetection,
  type DragEndEvent,
  type DragOverEvent,
  type DragStartEvent,
} from "@dnd-kit/core";
import { SortableContext, sortableKeyboardCoordinates, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { defaultRangeExtractor, useVirtualizer, type Range, type VirtualItem } from "@tanstack/react-virtual";
import { ChevronDown, ChevronRight, Columns3, EyeOff } from "lucide-react";
import type { DbRow, GroupBy, GroupOut, Property, View } from "@/lib/database-types";
import { DbShellContext } from "../context";
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
  useViewRows,
  visibleProps,
} from "./shared";

const SEP = "\u0001";
const WIDTHS: Record<string, number> = { small: 220, medium: 272, large: 344 };
const COLUMN_GAP = 12;
const ROW_GAP = 8;
const COLLAPSED_WIDTH = 44;
const SMALL_GROUP_COUNT = 5;
const noIds: string[] = [];
const GROUPABLE = ["select", "status", "multiSelect", "person", "checkbox"];
const cardId = (cell: string, row: string) => `${cell}@${row}`;
const estimateCardHeight = () => 104;

interface Lane {
  key: string;
  label: string;
  count: number;
}

interface Cell {
  id: string;
  col: GroupOut;
  lane: Lane | null;
  /** Card ids (a row can be in several cells when the property is multi-select). */
  ids: string[];
}

interface BoardDrag {
  id: string;
  row: DbRow;
  from: Cell;
  current: Cell;
  items: Record<string, string[]>;
}

function boardRange(range: Range, pinned: (number | undefined)[]) {
  if (range.count <= SMALL_GROUP_COUNT) return Array.from({ length: range.count }, (_, index) => index);
  const indices = new Set(defaultRangeExtractor(range));
  for (const index of pinned) if (index !== undefined) indices.add(index);
  return [...indices].sort((a, b) => a - b);
}

function columnPosition(item: VirtualItem): CSSProperties {
  // Sharing a grid track lets the tallest mounted cell define the lane height.
  return { gridArea: "1 / 1", marginInlineStart: item.start, width: item.size };
}

const visibleCorners: CollisionDetection = (args) => closestCorners({
  ...args,
  droppableContainers: args.droppableContainers.filter((container) => {
    const node = container.node.current;
    if (!node) return false;
    const board = node.closest("[data-board-viewport]")?.getBoundingClientRect();
    const rect = node.getBoundingClientRect();
    if (board && (rect.right <= board.left || rect.left >= board.right || rect.bottom <= board.top || rect.top >= board.bottom)) return false;
    const scroll = node.closest("[data-board-scroll]");
    if (!scroll) return true;
    const viewport = scroll.getBoundingClientRect();
    return rect.bottom > viewport.top && rect.top < viewport.bottom;
  }),
});

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

function BoardInner({ db, view, result, rowById, rowTitles, tableRowIds, snapshotRowIds, loadTableRows, openRow, setCells, createRow, updateView, reorderRows, g, gp, groups }: ViewProps & { g: GroupBy; gp: Property; groups: GroupOut[] }) {
  const compact = useContext(DbShellContext)?.compact;
  const sub = view.subGroup && groupProp(db, view.subGroup) ? view.subGroup : null;
  const subProp = sub ? groupProp(db, sub) : undefined;
  const width = WIDTHS[view.cardSize] ?? WIDTHS.medium;
  const cover = propOf(db, view.cardCover)?.kind === "files" ? view.cardCover : "";
  const shown = useMemo(() => visibleProps(db, view, [g.prop, sub?.prop]), [db, view, g.prop, sub?.prop]);
  const titles = useMemo(() => rowTitles ?? titleMap(result.rows), [rowTitles, result.rows]);
  const guard = useDragClickGuard();
  const openCard = useCallback((id: string) => {
    if (!guard.recent()) openRow(id);
  }, [guard, openRow]);
  const [adding, setAdding] = useState<Cell | null>(null);
  const [laneClosed, setLaneClosed] = useState<Set<string>>(new Set());
  const [focused, setFocused] = useState<{ column?: string; lane?: string }>({});
  const [rtl, setRtl] = useState(false);
  const [laneStart, setLaneStart] = useState(44);
  const host = useRef<HTMLDivElement>(null);
  const header = useRef<HTMLDivElement>(null);

  const canMove = canGroupEdit(gp, g);
  const canMoveSub = !sub || canGroupEdit(subProp, sub);

  /* ----- the cells ----- */
  const lanes: Lane[] | null = useMemo(() => {
    if (!sub) return null;
    const seen = new Map<string, Lane>();
    for (const col of groups) for (const group of col.subs) {
      const lane = seen.get(group.key);
      if (lane) lane.count += group.count;
      else seen.set(group.key, { key: group.key, label: group.label, count: group.count });
    }
    return [...seen.values()];
  }, [groups, sub]);

  const laneMembers = useMemo(() => lanes && groups.map((col) => new Map(col.subs.map((s) => [s.key, s.rows]))), [groups, lanes]);
  const columnIndices = useMemo(() => new Map(groups.map((col, index) => [col.key, index])), [groups]);
  const laneIndices = useMemo(() => new Map(lanes?.map((lane, index) => [lane.key, index])), [lanes]);
  const cellAt = (index: number, lane: Lane | null): Cell => {
    const col = groups[index];
    const ids = lane ? laneMembers?.[index].get(lane.key) ?? noIds : col.rows;
    return { id: col.key + SEP + (lane?.key ?? ""), col, lane, ids };
  };
  const [drag, setDrag] = useState<BoardDrag | null>(null);
  const dragRef = useRef<BoardDrag | null>(null);
  const updateDrag = (next: BoardDrag | null) => {
    dragRef.current = next;
    setDrag(next);
  };

  const hasColumns = groups.length > 0;
  useLayoutEffect(() => {
    const el = host.current;
    const head = header.current;
    if (!el || !head) return;
    const readDirection = () => setRtl(getComputedStyle(el).direction === "rtl");
    const direction = new MutationObserver(readDirection);
    for (let ancestor: HTMLElement | null = el; ancestor; ancestor = ancestor.parentElement) {
      direction.observe(ancestor, { attributes: true, attributeFilter: ["dir"] });
    }
    const measure = () => setLaneStart(head.offsetTop + head.offsetHeight + ROW_GAP);
    const resize = new ResizeObserver(measure);
    resize.observe(head);
    readDirection();
    measure();
    return () => {
      direction.disconnect();
      resize.disconnect();
    };
  }, [hasColumns]);

  const collapsed = useMemo(() => new Set(g.collapsed), [g.collapsed]);
  const { from, current } = drag ?? {};
  const columnKey = useCallback((index: number) => groups[index].key, [groups]);
  const laneKey = useCallback((index: number) => lanes![index].key, [lanes]);
  const estimateColumn = useCallback((index: number) => collapsed.has(groups[index].key) ? COLLAPSED_WIDTH : width, [collapsed, groups, width]);
  const columnRange = useCallback((range: Range) => boardRange(range, [focused.column, adding?.col.key, from?.col.key, current?.col.key]
    .map((key) => key === undefined ? undefined : columnIndices.get(key))), [focused.column, adding, from, current, columnIndices]);
  const laneRange = useCallback((range: Range) => boardRange(range, [focused.lane, adding?.lane?.key, from?.lane?.key, current?.lane?.key]
    .map((key) => key === undefined ? undefined : laneIndices.get(key))), [focused.lane, adding, from, current, laneIndices]);
  const columns = useVirtualizer({
    horizontal: true,
    isRtl: rtl,
    count: groups.length,
    getScrollElement: () => host.current,
    getItemKey: columnKey,
    estimateSize: estimateColumn,
    gap: COLUMN_GAP,
    overscan: 2,
    rangeExtractor: columnRange,
    useFlushSync: false,
  });
  const laneRows = useVirtualizer({
    enabled: !!lanes,
    count: lanes?.length ?? 0,
    getScrollElement: () => host.current,
    getItemKey: laneKey,
    estimateSize: (index) => laneClosed.has(lanes![index].key) ? 32 : 120,
    scrollMargin: laneStart,
    anchorTo: (lanes?.length ?? 0) > SMALL_GROUP_COUNT ? "end" : "start",
    gap: ROW_GAP,
    overscan: 2,
    rangeExtractor: laneRange,
    useFlushSync: false,
  });
  const virtualColumns = columns.getVirtualItems();
  useLayoutEffect(() => columns.measure(), [columns, collapsed, width]);
  const [ranges, setRanges] = useState<Record<string, readonly string[]>>({});
  const reportRange = useCallback((id: string, ids: readonly string[]) => {
    setRanges((prior) => {
      const previous = prior[id];
      if (!ids.length && !previous) return prior;
      if (previous?.length === ids.length && previous.every((row, index) => row === ids[index])) return prior;
      const next = { ...prior };
      if (ids.length) next[id] = ids;
      else delete next[id];
      return next;
    });
  }, []);
  const wanted = useMemo(() => {
    const ids = new Set(Object.values(ranges).flat());
    if (drag) ids.add(drag.row.id);
    return [...ids];
  }, [ranges, drag?.row.id]);
  useViewRows(loadTableRows, wanted);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 6 } }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
      keyboardCodes: { start: ["Space"], cancel: ["Escape"], end: ["Space", "Enter"] },
    }),
  );

  const onDragStart = (e: DragStartEvent) => {
    const data = e.active.data.current;
    if (!data?.boardCell) return;
    const row = rowById.get(data.rowId);
    if (row) updateDrag({ id: String(e.active.id), row, from: data.boardCell, current: data.boardCell, items: {} });
  };

  const onDragOver = (e: DragOverEvent) => {
    const cur = dragRef.current;
    const over = e.over;
    const target: Cell | undefined = over?.data.current?.boardCell;
    if (!cur || !over || !target || cur.current.id === target.id || !canMove || !canMoveSub) return;
    const source = cur.from;
    const to = target.id;
    const id = cur.row.id;
    if (source.id !== to && target.ids.includes(id)) return;
    const dst = source.id === to ? target.ids.filter((r) => r !== id) : [...target.ids];
    const at = dst.indexOf(over.data.current?.rowId);
    const rect = e.active.rect.current.translated;
    const below = !!rect && rect.top > over.rect.top + over.rect.height / 2;
    dst.splice(at >= 0 ? at + (below ? 1 : 0) : dst.length, 0, id);
    updateDrag({ ...cur, current: target, items: { [source.id]: source.ids.filter((r) => r !== id), [to]: dst } });
  };

  const onDragEnd = (e: DragEndEvent) => {
    guard.mark();
    const cur = dragRef.current;
    updateDrag(null);
    if (!cur || !e.over) return;
    const targetData = e.over.data.current;
    const to: Cell | undefined = targetData?.boardCell;
    const from = cur.from;
    if (!to) return;
    const row = rowById.get(cur.row.id) ?? cur.row;
    if (to.id !== from.id) {
      if (!canMove || !canMoveSub || to.ids.includes(row.id)) return;
      const colPatch = to.col.key === from.col.key ? null
        : groupPatch(gp, g, to.col.key, to.col.label, row, from.col);
      const lanePatch = to.lane && from.lane && to.lane.key !== from.lane.key
        ? groupPatch(subProp, sub, to.lane.key, to.lane.label, row, from.lane) : null;
      const patch = { ...colPatch, ...lanePatch };
      if (Object.keys(patch).length) void setCells(row.id, patch);
      return;
    }
    if (view.sorts.length) return;
    const oldIndex = from.ids.indexOf(row.id);
    const newIndex = from.ids.indexOf(targetData?.rowId);
    if (oldIndex < 0 || newIndex < 0 || oldIndex === newIndex) return;
    const reordered = [...from.ids];
    reordered.splice(oldIndex, 1);
    reordered.splice(newIndex, 0, row.id);
    // Change this cell's positions inside the full snapshot. Hidden rows and
    // cards in other cells keep their relative order, even during Find.
    const ordered = snapshotRowIds ?? tableRowIds ?? result.rows.map((r) => r.id);
    const members = new Set(from.ids);
    let index = 0;
    void reorderRows(ordered.map((id) => members.has(id) ? reordered[index++] : id));
  };

  /* ----- new cards ----- */
  const cellsFor = (cell: Cell): Record<string, unknown> => {
    const a = groupPatch(gp, g, cell.col.key, cell.col.label) ?? {};
    const b = cell.lane ? groupPatch(subProp, sub, cell.lane.key, cell.lane.label) ?? {} : {};
    return Object.fromEntries(Object.entries({ ...a, ...b }).filter(([, v]) => v !== null));
  };

  const patchGroup = (p: Partial<GroupBy>) => updateView({ group: { ...g, ...p } });

  const activeRow = drag ? rowById.get(drag.row.id) ?? drag.row : undefined;
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

  const renderCell = (item: VirtualItem, lane: Lane | null) => {
    const cell = cellAt(item.index, lane);
    return (
      <div key={item.key} style={columnPosition(item)} data-board-column={cell.col.key}>
        {collapsed.has(cell.col.key) ? (
          <div className="rounded-xl border border-dashed border-line/60" aria-hidden />
        ) : (
            <BoardCell
              cell={cell}
              ids={drag?.items[cell.id] ?? cell.ids}
              rowById={rowById}
              host={host}
              compact={compact}
              active={drag?.current.id === cell.id ? drag : null}
              onRange={loadTableRows ? reportRange : undefined}
              cover={cover}
              fit={view.cardFit}
              props={shown}
              titles={titles}
              onOpen={openCard}
              canDrag={view.sorts.length === 0 || (canMove && canMoveSub)}
              adding={adding?.id === cell.id}
              onAdd={() => setAdding(cell)}
              onCancelAdd={() => setAdding((a) => (a?.id === cell.id ? null : a))}
              onCreate={(title) => void createRow(cellsFor(cell), title)}
            />
        )}
      </div>
    );
  };

  return (
    <div
      ref={host}
      className={cn("relative h-full overflow-auto px-4 pb-6 pt-2 [overflow-anchor:none]", compact && "max-h-[520px]")}
      role="region"
      aria-label="Board"
      data-board-viewport
      onFocusCapture={({ target }) => setFocused({
        column: target.closest<HTMLElement>("[data-board-column]")?.dataset.boardColumn,
        lane: target.closest<HTMLElement>("[data-board-lane]")?.dataset.boardLane,
      })}
      onBlurCapture={({ currentTarget, relatedTarget }) => {
        if (!currentTarget.contains(relatedTarget)) setFocused({});
      }}
    >
      <DndContext
        sensors={sensors}
        collisionDetection={visibleCorners}
        onDragStart={onDragStart}
        onDragOver={onDragOver}
        onDragEnd={onDragEnd}
        onDragCancel={() => updateDrag(null)}
      >
        <div className="flex items-start gap-3">
          <div className="shrink-0" style={{ width: columns.getTotalSize() }}>
            <div ref={header} className={cn("grid grid-cols-1 items-start", collapsed.size ? "min-h-[8rem]" : "min-h-7")}>
              {virtualColumns.map((item) => {
                const col = groups[item.index];
                const closed = collapsed.has(col.key);
                return closed ? (
                  <button
                    key={item.key}
                    style={columnPosition(item)}
                    data-board-column={col.key}
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
                  <div key={item.key} style={columnPosition(item)} data-board-column={col.key} className="group/h flex h-7 min-w-0 items-center gap-1.5 px-1" data-column={col.key}>
                    <GroupChip g={col} />
                    <span className="text-xs tabular-nums text-faint" data-count>{col.count}</span>
                    {Object.entries(col.calcs).map(([propId, value]) => (
                      <span key={propId} title={propOf(db, propId)?.name} className="truncate text-xs tabular-nums text-faint" data-calc={propId}>{value}</span>
                    ))}
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
            </div>

            {lanes
              ? <div className="relative mt-2" style={{ height: laneRows.getTotalSize() }}>
                {laneRows.getVirtualItems().map((item) => {
                  const lane = lanes[item.index];
                  const closed = laneClosed.has(lane.key);
                  return (
                    <div key={item.key} ref={laneRows.measureElement} data-index={item.index} data-board-lane={lane.key} className="absolute inset-x-0" style={{ top: item.start - laneStart }}>
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
                        className="sticky start-0 flex w-fit items-center gap-1.5 rounded-md px-1 py-0.5 text-[0.82rem] font-medium hover:bg-surface-2"
                      >
                        {closed ? <ChevronRight size={14} className="rtl:rotate-180" /> : <ChevronDown size={14} />}
                        <span dir="auto">{lane.label}</span>
                        <span className="text-xs font-normal tabular-nums text-faint">{lane.count}</span>
                      </button>
                      {closed ? null : <div className="mt-2 grid grid-cols-1 items-start">{virtualColumns.map((col) => renderCell(col, lane))}</div>}
                    </div>
                  );
                })}
              </div>
              : <div className="mt-2 grid grid-cols-1 items-start">{virtualColumns.map((col) => renderCell(col, null))}</div>}
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
            <div style={{ width }} className="cursor-grabbing" data-board-overlay>
              <CardBody row={activeRow} cover={cover} fit={view.cardFit} props={shown} titles={titles} overlay />
            </div>
          ) : null}
        </DragOverlay>
      </DndContext>
    </div>
  );
}

/* ---------- One cell (a column, or a column inside a swimlane) ---------- */

function BoardCell({
  cell,
  ids,
  rowById,
  host,
  compact,
  active,
  onRange,
  cover,
  fit,
  props,
  titles,
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
  host: RefObject<HTMLDivElement | null>;
  compact?: boolean;
  active: BoardDrag | null;
  onRange?: (cell: string, ids: readonly string[]) => void;
  cover: string;
  fit: boolean;
  props: Property[];
  titles: Map<string, string>;
  canDrag: boolean;
  onOpen: (id: string) => void;
  adding: boolean;
  onAdd: () => void;
  onCancelAdd: () => void;
  onCreate: (title: string) => void;
}) {
  const scroller = useRef<HTMLDivElement>(null);
  const hasCards = ids.length > 0;
  const [inView, setInView] = useState(!onRange);
  useEffect(() => {
    const el = scroller.current;
    if (!onRange || !hasCards || !el) return;
    const observer = new IntersectionObserver(([entry]) => setInView(entry.isIntersecting), { root: host.current, rootMargin: "64px" });
    observer.observe(el);
    return () => observer.disconnect();
  }, [host, onRange, hasCards]);

  const getItemKey = useCallback((index: number) => ids[index], [ids]);
  const virtualizer = useVirtualizer({
    count: ids.length,
    getScrollElement: () => scroller.current,
    getItemKey,
    estimateSize: estimateCardHeight,
    gap: 8,
    overscan: 5,
    useFlushSync: false,
  });
  const range = virtualizer.getVirtualItems();
  const virtualRows = useMemo(() => inView ? range.map((item) => ids[item.index]) : [], [inView, range, ids]);
  useEffect(() => onRange?.(cell.id, virtualRows), [cell.id, onRange, virtualRows]);
  useEffect(() => () => onRange?.(cell.id, []), [cell.id, onRange]);

  const { setNodeRef, isOver } = useDroppable({ id: cell.id, data: { boardCell: cell } });
  const cards = inView ? range.map((item) => {
    const rowId = ids[item.index];
    const id = active?.row.id === rowId ? active.id : cardId(cell.id, rowId);
    const row = rowById.get(rowId);
    return { item, id, row };
  }) : [];

  return (
    <div
      ref={setNodeRef}
      data-cell={cell.id}
      className={cn("flex min-h-[4.5rem] min-w-0 flex-col gap-2 rounded-xl bg-surface p-1.5 transition-colors", isOver && "bg-surface-2")}
    >
      {hasCards ? (
        <div ref={scroller} data-board-scroll className="overflow-y-auto" style={{ maxHeight: compact ? "min(80vh, 26rem)" : "min(80vh, 42rem)" }}>
          <div className="relative" style={{ height: virtualizer.getTotalSize() }}>
            <SortableContext items={cards.flatMap(({ id, row }) => row ? [id] : [])} strategy={verticalListSortingStrategy}>
              {cards.map(({ item, id, row }) => (
                <div key={item.key} data-index={item.index} ref={row ? virtualizer.measureElement : undefined} className="absolute inset-x-0" style={{ top: item.start }}>
                  {row ? (canDrag
                    ? <SortableCard id={id} cell={cell} row={row} cover={cover} fit={fit} props={props} titles={titles} onOpen={onOpen} />
                    : <CardFrame row={row} cover={cover} fit={fit} props={props} titles={titles} onOpen={onOpen} />)
                    : <div aria-hidden className="rounded-lg border border-line bg-elevated" style={{ height: item.size }} />}
                </div>
              ))}
            </SortableContext>
          </div>
        </div>
      ) : null}
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

const SortableCard = memo(function SortableCard({ id, cell, ...props }: CardProps & { id: string; cell: Cell }) {
  const sortable = useSortable({ id, data: { boardCell: cell, rowId: props.row.id } });
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
