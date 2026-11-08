// Loading a single row and turning a row into a template.

import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { DbRow } from "@/lib/database-types";
import { editDatabase, loadDatabase, peekDatabase } from "./dbStore";
import { emptyFilter, isComputedKind, newId } from "./model";

/** One row with its cell values. Uses `get_row`; asks the first view for it when that command is not there. */
export async function loadRow(dbId: string, rowId: string): Promise<DbRow | null> {
  try {
    return (await api.getRow(rowId)).row;
  } catch {
    try {
      const db = peekDatabase(dbId) ?? (await loadDatabase(dbId));
      const view = { ...db.views[0], filter: emptyFilter(), sorts: [], group: null, subGroup: null, calcs: [] };
      const r = await api.queryView({ databaseId: dbId, viewId: view.id, view });
      return r.rows.find((x) => x.id === rowId) ?? null;
    } catch (e) {
      toast.error(errorText(e));
      return null;
    }
  }
}

/** Makes a template from a row: its cell values (not computed ones), icon and body. */
export async function saveRowAsTemplate(dbId: string, row: DbRow, name: string, content: string, text: string): Promise<void> {
  const db = peekDatabase(dbId);
  if (!db) return;
  const cells: Record<string, unknown> = Object.create(null);
  for (const p of db.props) {
    if (p.kind === "title" || p.kind === "files" || isComputedKind(p.kind) || p.kind === "id") continue;
    const v = row.values[p.id];
    if (v !== undefined && v !== null) cells[p.id] = v;
  }
  const t = { id: newId(), name: name || "Untitled", icon: row.icon, cells: JSON.stringify(cells), content, text, repeat: "", repeatLast: 0 };
  await editDatabase(dbId, (d) => ({ ...d, templates: [...d.templates, t] }), true);
  toast.success("Saved as a template");
}
