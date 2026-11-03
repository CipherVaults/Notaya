// Small building blocks of the database panels.

import { type ReactNode } from "react";
import { Popover } from "radix-ui";
import { ChevronDown, GripVertical } from "lucide-react";
import {
  DndContext, KeyboardSensor, PointerSensor, closestCenter, useSensor, useSensors, type DragEndEvent,
} from "@dnd-kit/core";
import { SortableContext, sortableKeyboardCoordinates, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import type { Property } from "@/lib/database-types";
import { cn } from "@/lib/utils";
import { KindIcon } from "../icons";

export const fieldCls =
  "h-8 min-w-0 rounded-md border border-line bg-surface px-2 text-sm text-fg outline-none placeholder:text-faint focus:border-accent focus:ring-2 focus:ring-accent-soft disabled:opacity-50";

/** A popover opened from a toolbar button. */
export function PanelPopover({
  trigger, children, width = 360, align = "start", open, onOpenChange,
}: { trigger: ReactNode; children: ReactNode; width?: number; align?: "start" | "end" | "center"; open?: boolean; onOpenChange?: (o: boolean) => void }) {
  return (
    <Popover.Root open={open} onOpenChange={onOpenChange}>
      <Popover.Trigger asChild>{trigger}</Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          align={align}
          sideOffset={6}
          collisionPadding={12}
          style={{ width: `min(${width}px, calc(100vw - 24px))` }}
          className="z-[75] max-h-[min(var(--radix-popover-content-available-height),620px)] overflow-y-auto rounded-xl border border-line bg-elevated p-3 shadow-float outline-none"
        >
          {children}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

export function PSelect({
  value, onChange, children, className, label, disabled,
}: { value: string; onChange: (v: string) => void; children: ReactNode; className?: string; label: string; disabled?: boolean }) {
  return (
    <span className={cn("relative inline-flex min-w-0", className)}>
      <select
        aria-label={label}
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value)}
        className={cn(fieldCls, "w-full cursor-pointer appearance-none pe-7")}
      >
        {children}
      </select>
      <ChevronDown size={13} className="pointer-events-none absolute end-2 top-1/2 -translate-y-1/2 text-faint" />
    </span>
  );
}

/** A select listing the properties of a database. */
export function PropSelect({
  props, value, onChange, label, allowNone, noneLabel = "None", filter, className,
}: { props: Property[]; value: string; onChange: (v: string) => void; label: string; allowNone?: boolean; noneLabel?: string; filter?: (p: Property) => boolean; className?: string }) {
  return (
    <PSelect value={value} onChange={onChange} label={label} className={className}>
      {allowNone && <option value="">{noneLabel}</option>}
      {!allowNone && !props.some((p) => p.id === value) && <option value="">Choose…</option>}
      {props.filter((p) => !filter || filter(p) || p.id === value).map((p) => (
        <option key={p.id} value={p.id}>{p.name}</option>
      ))}
    </PSelect>
  );
}

export function Row({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-3 py-1">
      <span className="shrink-0 text-sm text-muted">{label}</span>
      <div className="flex min-w-0 justify-end">{children}</div>
    </div>
  );
}

export function SwitchRow({ label, checked, onChange, disabled }: { label: string; checked: boolean; onChange: (v: boolean) => void; disabled?: boolean }) {
  return (
    <label className={cn("flex cursor-pointer items-center justify-between gap-3 py-1.5 text-sm", disabled && "opacity-50")}>
      <span>{label}</span>
      <input type="checkbox" role="switch" checked={checked} disabled={disabled} onChange={(e) => onChange(e.target.checked)} className="h-4 w-4 accent-[var(--accent)]" />
    </label>
  );
}

export function PanelTitle({ children }: { children: ReactNode }) {
  return <div className="px-0.5 pb-1.5 text-[0.7rem] font-semibold uppercase tracking-wider text-faint">{children}</div>;
}

export function PropLabel({ prop }: { prop: Property }) {
  return (
    <span className="flex min-w-0 items-center gap-1.5">
      <KindIcon kind={prop.kind} className="shrink-0 text-muted" />
      <span dir="auto" className="truncate">{prop.name}</span>
    </span>
  );
}

/* ---------- a reorderable list (dnd-kit) ---------- */

export function SortableRow({ id, children, disabled }: { id: string; disabled?: boolean; children: (handle: ReactNode) => ReactNode }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id, disabled });
  const handle = (
    <button
      type="button"
      {...attributes}
      {...listeners}
      disabled={disabled}
      aria-label="Drag to reorder"
      className="grid h-6 w-5 shrink-0 cursor-grab touch-none place-items-center rounded text-faint hover:text-fg active:cursor-grabbing disabled:opacity-30"
    >
      <GripVertical size={14} />
    </button>
  );
  return (
    <div ref={setNodeRef} style={{ transform: CSS.Transform.toString(transform), transition }} className={cn(isDragging && "relative z-10 opacity-80")}>
      {children(handle)}
    </div>
  );
}

export function SortableList({ ids, onReorder, children }: { ids: string[]; onReorder: (from: number, to: number) => void; children: ReactNode }) {
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 4 } }), useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }));
  const onEnd = (e: DragEndEvent) => {
    if (!e.over || e.active.id === e.over.id) return;
    const from = ids.indexOf(String(e.active.id));
    const to = ids.indexOf(String(e.over.id));
    if (from >= 0 && to >= 0) onReorder(from, to);
  };
  return (
    <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={onEnd}>
      <SortableContext items={ids} strategy={verticalListSortingStrategy}>
        {children}
      </SortableContext>
    </DndContext>
  );
}
