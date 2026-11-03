// What the pieces of a database screen (table, panels) share with the shell that owns the data.

import { createContext, useContext } from "react";
import type { Database, DatabaseMeta, DbRow, View } from "@/lib/database-types";

export interface DbShell {
  db: Database;
  view: View;
  /** All databases of the vault (for relations). */
  databases: DatabaseMeta[];
  /** Locked: rows can change, properties and views cannot. */
  locked: boolean;
  /** Shown inside a page (small) rather than as a page of its own. */
  compact: boolean;
  /** Changes the database definition (properties, templates, ...) and saves it soon. `now` saves at once. */
  editDb: (fn: (d: Database) => Database, now?: boolean) => Promise<void>;
  /** Changes the current view. Unsaved while the database is locked. */
  editView: (fn: (v: View) => View) => void;
  /** Moves rows to the trash (with an undo in a toast). */
  trashRows: (ids: string[]) => Promise<void>;
  duplicateRow: (id: string) => Promise<void>;
  /** Runs the view again. */
  requery: () => void;
  rowById: Map<string, DbRow>;
  /** A row that was just added and wants its title edited. */
  focusRowId: string | null;
  clearFocusRow: () => void;
  /** Asks the table to edit the title of a row that was just added. */
  focusRow: (rowId: string) => void;
  /** Shows a value on screen that the backend already stored (files). */
  patchRowLocal: (rowId: string, propId: string, value: unknown) => void;
  /** Opens the settings of a property. */
  openProp: (propId: string) => void;
  /** The first row's computed cell of a property, for the formula preview. */
  sampleOf: (propId: string) => { t: string; text: string } | null;
}

export const DbShellContext = createContext<DbShell | null>(null);

export function useDbShell(): DbShell {
  const c = useContext(DbShellContext);
  if (!c) throw new Error("useDbShell outside a database");
  return c;
}
