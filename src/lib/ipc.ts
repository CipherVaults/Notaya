// Typed wrappers around the Rust commands. Every call goes through Tauri's IPC;
// the UI has no other way to touch files, the network or the clipboard.

import { invoke } from "@tauri-apps/api/core";
import { flushAll } from "./flush";
import type {
  BackupInfo,
  CalcEntry,
  EnableSecurityResult,
  ExportRequest,
  ExportSummary,
  ImportSource,
  ImportSummary,
  LogInfo,
  NotesImportSummary,
  NotesTree,
  Page,
  PageMeta,
  RestoreResult,
  SearchHit,
  Settings,
  ShellPrefs,
  Sticky,
  Strength,
  VaultStatus,
} from "./types";
import type {
  ChartData,
  Database,
  DatabaseFull,
  DatabaseMeta,
  DbRow,
  FileCell,
  QueryRequest,
  RowHit,
  ViewResult,
} from "./database-types";

/** What a CSV import did (the backend asks for the file). */
export interface CsvImportResult {
  databaseId: string;
  /** True when a new database was made from the CSV. */
  created: boolean;
  rows: number;
  /** Columns or cells that could not be imported, with the reason. */
  skipped: string[];
}

export const api = {
  // Vault
  vaultStatus: () => invoke<VaultStatus>("vault_status"),
  // Start new notes (no master password) when there is no vault but earlier copies exist
  startNotes: () => invoke<void>("start_notes"),
  // Security is optional: turn the master password on or off at any time
  enableSecurity: (password: string, keyfile: string | null, removeUnprotectedCopies: boolean) =>
    invoke<EnableSecurityResult>("enable_security", { password, keyfile, removeUnprotectedCopies }),
  disableSecurity: (currentPassword: string, currentKeyfile: string | null) =>
    invoke<void>("disable_security", { currentPassword, currentKeyfile }),
  unlockVault: (password: string, keyfile: string | null) =>
    invoke<void>("unlock_vault", { password, keyfile }),
  lockVault: async () => {
    await flushAll();
    return invoke<void>("lock_vault");
  },
  // Answers the "vault-locking" event once this window has written its pending saves
  flushDone: (round: number) => invoke<void>("flush_done", { round }),
  changeMaster: (
    currentPassword: string,
    currentKeyfile: string | null,
    newPassword: string,
    newKeyfile: string | null,
  ) => invoke<void>("change_master", { currentPassword, currentKeyfile, newPassword, newKeyfile }),
  pickKeyfile: () => invoke<string | null>("pick_keyfile"),
  createKeyfile: () => invoke<string | null>("create_keyfile"),
  backupNow: () => invoke<string | null>("backup_now"),
  // Restore from a backup (works while locked). `source` is a backup name or a full path.
  listBackups: () => invoke<BackupInfo[]>("list_backups"),
  pickBackupFile: () => invoke<string | null>("pick_backup_file"),
  restoreVault: (source: string, password: string, keyfile: string | null, allowSecurityOff = false) =>
    invoke<RestoreResult>("restore_vault", { source, password, keyfile, allowSecurityOff }),
  // Installed Notaya, first screen: copy the notes of a portable copy (the folder is chosen in a
  // dialog opened by the backend). Resolves to null if the dialog was cancelled.
  importPortableNotes: () => invoke<ImportSummary | null>("import_portable_notes"),
  // Start over (only while locked): sets the vault aside and returns its new file name.
  startOver: () => invoke<string>("start_over"),
  // A message the backend wants shown once (for example after a backup failed)
  takeNotice: () => invoke<string | null>("take_notice"),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  setBackdrop: (kind: string, dark: boolean) => invoke<boolean>("set_backdrop", { kind, dark }),
  windowReady: () => invoke<void>("window_ready"),
  quitApp: async () => {
    await flushAll();
    return invoke<void>("quit_app");
  },
  openDataFolder: () => invoke<void>("open_data_folder"),
  // The error log (a text file in the logs folder): where it is, and open that folder in Explorer
  logInfo: () => invoke<LogInfo>("log_info"),
  openLogFolder: () => invoke<void>("open_log_folder"),
  // Close to the tray and start with Windows (saved next to the vault, not in the encrypted settings)
  getShellPrefs: () => invoke<ShellPrefs>("get_shell_prefs"),
  setShellPrefs: (prefs: Pick<ShellPrefs, "closeToTray" | "startWithWindows" | "startMinimized">) =>
    invoke<ShellPrefs>("set_shell_prefs", { prefs }),

  // Notes
  listNotes: () => invoke<NotesTree>("list_notes"),
  createNotebook: (name: string, color: string, icon: string) =>
    invoke<NotesTree>("create_notebook", { name, color, icon }),
  updateNotebook: (id: string, name: string, color: string, icon: string) =>
    invoke<NotesTree>("update_notebook", { id, name, color, icon }),
  deleteNotebook: (id: string) => invoke<NotesTree>("delete_notebook", { id }),
  createSection: (notebookId: string, name: string, color: string) =>
    invoke<NotesTree>("create_section", { notebookId, name, color }),
  updateSection: (id: string, name: string, color: string) =>
    invoke<NotesTree>("update_section", { id, name, color }),
  deleteSection: (id: string) => invoke<NotesTree>("delete_section", { id }),
  createPage: (sectionId: string, title: string) => invoke<Page>("create_page", { sectionId, title }),
  getPage: (id: string) => invoke<Page>("get_page", { id }),
  savePage: (id: string, title: string, content: string, text: string) =>
    invoke<PageMeta>("save_page", { id, title, content, text }),
  createSubpage: (parentId: string, title: string) => invoke<Page>("create_subpage", { parentId, title }),
  setPageLook: (id: string, icon: string, cover: string) => invoke<PageMeta>("set_page_look", { id, icon, cover }),
  setPageFavorite: (id: string, favorite: boolean) =>
    invoke<NotesTree>("set_page_favorite", { id, favorite }),
  movePage: (id: string, sectionId: string) => invoke<NotesTree>("move_page", { id, sectionId }),
  reorder: (ids: string[]) => invoke<NotesTree>("reorder", { ids }),
  trashPage: (id: string) => invoke<NotesTree>("trash_page", { id }),
  restorePage: (id: string) => invoke<NotesTree>("restore_page", { id }),
  deletePage: (id: string) => invoke<NotesTree>("delete_page", { id }),
  emptyTrash: () => invoke<NotesTree>("empty_trash"),
  addAttachment: (bytes: Uint8Array) => invoke<string>("add_attachment", bytes),
  // Databases (A35). Rows are pages: open one with getPage/savePage like any page.
  createDatabase: (sectionId: string, parent: string, inline: boolean, title: string, starterKind: "blank" | "tasks" | "reading" | "projects") =>
    invoke<DatabaseFull>("create_database", { sectionId, parent, inline, title, starterKind }),
  getDatabase: (id: string) => invoke<Database>("get_database", { id }),
  /** The whole database definition as JSON text (properties, views, templates). */
  saveDatabase: (database: Database) => invoke<Database>("save_database", { databaseJson: JSON.stringify(database) }),
  queryView: (request: QueryRequest) => invoke<ViewResult>("query_view", { request }),
  createRow: (databaseId: string, title: string, cells: Record<string, unknown>, templateId = "") =>
    invoke<{ row: DbRow }>("create_row", { databaseId, title, cells: JSON.stringify(cells), templateId }),
  setCells: (rowId: string, title: string | null, patch: Record<string, unknown>) =>
    invoke<{ row: DbRow }>("set_cells", { rowId, title, patch: JSON.stringify(patch) }),
  duplicateRow: (rowId: string) => invoke<{ row: DbRow }>("duplicate_row", { rowId }),
  reorderRows: (ids: string[]) => invoke<void>("reorder_rows", { ids }),
  searchRows: (databaseId: string, query: string, limitN = 50) =>
    invoke<RowHit[]>("search_rows", { databaseId, query, limitN }),
  rowTitles: (ids: string[]) => invoke<RowHit[]>("row_titles", { ids }),
  setDatabaseLook: (id: string, title: string, icon: string, favorite: boolean, locked: boolean) =>
    invoke<DatabaseMeta>("set_database_look", { id, title, icon, favorite, locked }),
  moveDatabase: (id: string, sectionId: string) => invoke<void>("move_database", { id, sectionId }),
  enableSubItems: (id: string) => invoke<Database>("enable_sub_items", { id }),
  enableDependencies: (id: string) => invoke<Database>("enable_dependencies", { id }),
  chartData: (request: QueryRequest) => invoke<ChartData>("chart_data", { request }),
  // Trash, restore and delete of a row or a database use trashPage / restorePage / deletePage with its id.
  /** One row with its cell values (Rust command `get_row`, same result as setCells with an empty patch but without touching the row). */
  getRow: (rowId: string) => invoke<{ row: DbRow }>("get_row", { rowId }),
  /** The backend opens the file dialog; null databaseId makes a new database from the CSV. Resolves to null if cancelled. */
  importCsv: (databaseId: string | null) => invoke<CsvImportResult | null>("import_csv", { databaseId }),
  /** The backend asks where to save (null if cancelled) and writes the rows of the view as an unencrypted CSV. */
  exportCsv: (databaseId: string, viewId: string) => invoke<{ path: string; rows: number } | null>("export_csv", { databaseId, viewId }),
  /** Raw body upload like addAttachment; the ids and the (URL-encoded) file name travel in headers. Resolves to the cell's files after adding. */
  attachFile: (rowId: string, propId: string, name: string, bytes: Uint8Array) =>
    invoke<FileCell[]>("attach_file", bytes, {
      headers: { "x-row-id": rowId, "x-prop-id": propId, "x-file-name": encodeURIComponent(name) },
    }),
  openFile: (rowId: string, propId: string, fileId: string) => invoke<void>("open_file", { rowId, propId, fileId }),
  saveFileAs: (rowId: string, propId: string, fileId: string) => invoke<string | null>("save_file_as", { rowId, propId, fileId }),
  removeFile: (rowId: string, propId: string, fileId: string) => invoke<FileCell[]>("remove_file", { rowId, propId, fileId }),

  // Export: the backend asks where to save (resolves to null if cancelled) and writes an unencrypted file
  exportNotes: (request: ExportRequest) => invoke<ExportSummary | null>("export_notes", { request }),
  // Import: the backend asks what to import (resolves to null if cancelled) and makes one new notebook
  importNotes: (source: ImportSource) => invoke<NotesImportSummary | null>("import_notes", { request: { source } }),
  // Asks the import in progress to stop; it then resolves to null and nothing is added
  importCancel: () => invoke<boolean>("import_cancel"),

  // Sticky notes
  listStickies: () => invoke<Sticky[]>("list_stickies"),
  createSticky: (open: boolean) => invoke<Sticky>("create_sticky", { open }),
  getSticky: (id: string) => invoke<Sticky>("get_sticky", { id }),
  saveSticky: (id: string, content: string, text: string) =>
    invoke<void>("save_sticky", { id, content, text }),
  setStickyStyle: (id: string, color: string, onTop: boolean) =>
    invoke<Sticky>("set_sticky_style", { id, color, onTop }),
  setStickyGeometry: (id: string, x: number, y: number, w: number, h: number) =>
    invoke<void>("set_sticky_geometry", { id, x, y, w, h }),
  openSticky: (id: string) => invoke<void>("open_sticky", { id }),
  closeSticky: (id: string) => invoke<void>("close_sticky", { id }),
  deleteSticky: (id: string) => invoke<void>("delete_sticky", { id }),
  deleteStickies: (ids: string[]) => invoke<void>("delete_stickies", { ids }),

  // Calculators pop out in their own windows; all of them share one saved history
  openCalculator: () => invoke<void>("open_calculator"),
  calcHistory: () => invoke<CalcEntry[]>("calc_history"),
  addCalcEntry: (expr: string, result: string) => invoke<CalcEntry[]>("add_calc_entry", { expr, result }),
  clearCalcHistory: () => invoke<void>("clear_calc_history"),

  // Tools
  openUrl: (url: string) => invoke<void>("open_url", { url }),
  passwordStrength: (password: string, hints: string[] = []) =>
    invoke<Strength>("password_strength", { password, hints }),
  search: (query: string) => invoke<SearchHit[]>("search", { query }),
};

/** Turns any thrown IPC error into a readable message. */
export function errorText(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return "Something went wrong.";
}
