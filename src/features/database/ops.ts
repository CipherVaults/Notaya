// Small operations shared by the table, the row page and the peeks.

import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { DbRow, SelectOption } from "@/lib/database-types";
import { blankOption, colorForNew } from "./model";
import { editDatabase, peekDatabase } from "./dbStore";

/** Adds an option to a select, multi-select or status property and saves it, so a cell can use it at once. */
export async function createOption(dbId: string, propId: string, name: string): Promise<SelectOption | null> {
  const db = peekDatabase(dbId);
  const prop = db?.props.find((p) => p.id === propId);
  if (!db || !prop) return null;
  const existing = prop.options.find((o) => o.name.toLowerCase() === name.trim().toLowerCase());
  if (existing) return existing;
  const opt = blankOption(name.trim(), colorForNew(prop.options.length), prop.kind === "status" ? "todo" : "");
  await editDatabase(dbId, (d) => ({ ...d, props: d.props.map((p) => (p.id === propId ? { ...p, options: [...p.options, opt] } : p)) }), true);
  const after = peekDatabase(dbId)?.props.find((p) => p.id === propId)?.options.some((o) => o.id === opt.id);
  return after ? opt : null;
}

/** Saves cells of a row; resolves to the row as the backend now has it (or null after showing an error). */
export async function saveCells(rowId: string, patch: Record<string, unknown>, title: string | null = null): Promise<DbRow | null> {
  try {
    return (await api.setCells(rowId, title, patch)).row;
  } catch (e) {
    toast.error(errorText(e));
    return null;
  }
}

/** Reads a File into the backend (the 50 MB limit is checked first) and resolves to the cell's files. */
export async function uploadFile(rowId: string, propId: string, file: File, max: number) {
  if (file.size > max) throw new Error(`"${file.name}" is larger than 50 MB.`);
  const bytes = new Uint8Array(await file.arrayBuffer());
  return api.attachFile(rowId, propId, file.name, bytes);
}
