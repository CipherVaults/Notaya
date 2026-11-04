// The contract between the database shell (toolbar, view tabs, panels, table) and the other views
// (board, calendar, timeline, list, gallery, chart). Each view component is `(props: ViewProps) => JSX`.

import type { Database, DbRow, View, ViewResult } from "@/lib/database-types";

export interface ViewProps {
  /** The database definition (properties, views). */
  db: Database;
  /** The view being shown, with any edits made in the toolbar. */
  view: View;
  /** Rows after the view's filter and sort, plus groups and footer calculations. */
  result: ViewResult;
  /** Quick lookup of the rows in `result`. */
  rowById: Map<string, DbRow>;
  /** True when the database is locked: rows can change, the properties and views cannot. */
  locked: boolean;
  /** Opens a row the way the view says (full page, side peek or center peek). */
  openRow: (rowId: string) => void;
  /** Changes cells of a row (null clears). The shell updates the screen at once and reports errors in a toast. */
  setCells: (rowId: string, patch: Record<string, unknown>) => Promise<void>;
  setTitle: (rowId: string, title: string) => Promise<void>;
  /** Adds a row with the given starting cells (for example the group it was added in); resolves to it. */
  createRow: (cells?: Record<string, unknown>, title?: string) => Promise<DbRow | null>;
  /** Changes settings of the current view and saves them (for example the date property of a calendar). */
  updateView: (patch: Partial<View>) => void;
  /** Saves the manual order of rows. */
  reorderRows: (ids: string[]) => Promise<void>;
}
