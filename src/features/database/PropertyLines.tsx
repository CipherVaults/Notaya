// The properties of one row as editable lines (name on the left, value on the right), like Notion's page properties.
// Used above the body of a row page, in peeks and in the template editor.

import { useState } from "react";
import { ChevronRight } from "lucide-react";
import type { Database, DbRow, Property } from "@/lib/database-types";
import { cn } from "@/lib/utils";
import { KindIcon } from "./icons";
import EditableCell from "./cells/CellEditor";
import { isComputedKind } from "./model";

interface Props {
  db: Database;
  row: DbRow;
  /** Properties to list; defaults to every property except the title. */
  props?: Property[];
  readOnly?: boolean;
  /** Hide the "files" lines (they need a stored row). */
  noFiles?: boolean;
  onCommit: (prop: Property, value: unknown) => void;
  onLocalChange?: (prop: Property, value: unknown) => void;
  /** Properties hidden in the view are folded under a toggle. */
  hiddenIds?: Set<string>;
}

export default function PropertyLines({ db, row, props, readOnly, noFiles, onCommit, onLocalChange, hiddenIds }: Props) {
  const [editingId, setEditingId] = useState<string | null>(null);
  const [showHidden, setShowHidden] = useState(false);
  const all = (props ?? db.props.filter((p) => p.kind !== "title")).filter((p) => !(noFiles && p.kind === "files"));
  const hidden = all.filter((p) => hiddenIds?.has(p.id));
  const shown = all.filter((p) => !hiddenIds?.has(p.id) || showHidden);
  return (
    <div className="mb-3 space-y-0.5 border-b border-line pb-3" role="list" aria-label="Properties">
      {shown.map((p) => (
        <div key={p.id} role="listitem" className="flex min-h-[32px] items-start gap-2 rounded-md hover:bg-surface/60">
          <div className="flex w-[11rem] shrink-0 items-center gap-1.5 py-1.5 ps-1 text-sm text-muted" title={p.description || p.name}>
            <KindIcon kind={p.kind} size={14} className="shrink-0" />
            <span dir="auto" className="truncate">{p.name}</span>
          </div>
          <div className={cn("min-w-0 flex-1 rounded-md px-1.5 py-1.5 text-sm", !readOnly && !isComputedKind(p.kind) && p.kind !== "checkbox" && "cursor-text hover:bg-surface-2")} onClick={() => !readOnly && !isComputedKind(p.kind) && p.kind !== "checkbox" && setEditingId(p.id)}>
            <EditableCell
              dbId={db.id}
              prop={p}
              row={row}
              value={row.values[p.id]}
              readOnly={readOnly}
              editing={editingId === p.id}
              setEditing={(b) => setEditingId(b ? p.id : null)}
              commit={(v) => onCommit(p, v)}
              onLocalChange={(v) => onLocalChange?.(p, v)}
              wrap
            />
            {row.values[p.id] == null && editingId !== p.id && !isComputedKind(p.kind) && p.kind !== "checkbox" && <span className="text-faint">Empty</span>}
          </div>
        </div>
      ))}
      {hidden.length > 0 && (
        <button type="button" onClick={() => setShowHidden((v) => !v)} aria-expanded={showHidden} className="mt-1 inline-flex items-center gap-1 rounded-md px-1 py-0.5 text-xs text-faint hover:bg-surface-2 hover:text-fg">
          <ChevronRight size={12} className={cn("transition-transform", showHidden && "rotate-90")} />
          {showHidden ? "Hide" : "Show"} {hidden.length} more {hidden.length === 1 ? "property" : "properties"}
        </button>
      )}
    </div>
  );
}
