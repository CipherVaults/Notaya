// Typed wrappers around the Rust commands. Every call goes through Tauri's IPC;
// the UI has no other way to touch files, the network or the clipboard.

import { invoke } from "@tauri-apps/api/core";
import type { BinEntry, Revisioned } from "./changes";
import { flushAll } from "./flush";
import { startupMark } from "./startup";
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
  Reminder,
  DbRow,
  FileCell,
  QueryRequest,
  RowRange,
  RowHit,
  ViewResult,
  ViewWindow,
} from "./database-types";

/** What a CSV import did (the backend asks for the file). */
export interface CsvImportResult {
  databaseId: string;
  /** True when a new database was made from the CSV. */
  created: boolean;
  rows: number;
  title: string;
  columns: number;
  /** Columns that could not be imported, with the reason. */
  skipped: string[];
  /** Values that did not fit their property and were left empty. */
  badCells: number;
  /** Shown once: the safety backup made before the first database. */
  notice: string | null;
}

// C5 window contract. These DTOs deliberately contain no pairing token or hash.
// UI answers are window-only commands; none belongs in the bridge action registry.
export type AiLevel = "look" | "edit" | "full";
export interface AiPairing {
  id: string;
  name: string;
  level: AiLevel;
  created: number;
  lastUsed: number | null;
}
export interface AiStatus {
  enabled: boolean;
  paused: boolean;
  pairings: AiPairing[];
  hiddenNotebooks: string[];
  bigChangeConfirm: boolean;
  bigChangeItems: number;
}
export interface AiPairingEvent {
  id: string;
  kind: "pairing";
  details: { client: string };
  word: null;
}
export interface AiConfirmEvent {
  id: string;
  kind: "confirm";
  details: { client: string; action: string; items: { id: string; title?: string }[] };
  word: string | null;
}
export interface AiActivityEvent {
  pairingId: string;
  name: string;
  at: number;
  items: string[];
}
// Window-local presentation state, assembled from status plus C5 events.
// C5 enforces its own five-minute timeout; the local deadline only disables Allow.
export interface AiPairingRequest {
  id: string;
  name: string;
  expiresAt: number;
}
export interface AiConfirmRequest {
  id: string;
  pairingName: string;
  summary: string;
  items: string[];
  kind: "ask" | "strong";
  word: string | null; // random challenge supplied and checked by the core
  expiresAt: number;
}
export interface AiActivity {
  pairingId: string;
  name: string;
  lastActiveAt: number;
  claims: string[];
}
export interface AiState extends AiStatus {
  pairingRequests: AiPairingRequest[];
  confirmRequests: AiConfirmRequest[];
  activity: AiActivity[];
}

export interface HistoryMeta {
  pageId: string; versionId: string; at: number; actor: string; changeSet: string;
  title: string; words: number; pictureIds: string[]; beforeAi: boolean;
}
export interface HistoryBody { title: string; content: string; text: string; icon: string; cover: string }
export interface ActivityEntry {
  seq: number; at: number; actor: string; changeSet: string; action: string;
  items: { id: string; kind: string; titleAtTheTime: string }[];
  counts: Record<string, number>; undoable: boolean;
}

export const api = {
  aiGetStatus: () => invoke<AiStatus>("ai_get_status"),
  aiSetEnabled: (enabled: boolean, confirmation: string = "") =>
    invoke<void>("ai_set_enabled", { enabled, confirmation }),
  aiSetLevel: (pairingId: string, level: AiLevel) => invoke<void>("ai_set_level", { pairingId, level }),
  aiRemovePairing: (pairingId: string) => invoke<void>("ai_remove_pairing", { pairingId }),
  aiSetHidden: (notebookIds: string[]) => invoke<void>("ai_set_hidden", { notebookIds }),
  aiSetBigChange: (enabled: boolean, items?: number) => invoke<void>("ai_set_big_change", { enabled, items }),
  aiPairingAnswer: (id: string, allow: boolean) => invoke<void>("ai_pair_answer", { id, allow }),
  aiConfirmAnswer: (id: string, allow: boolean, word: string | null = null) =>
    invoke<void>("ai_confirm_answer", { id, allow, word }),
  aiPause: () => invoke<void>("ai_pause"),
  aiResume: () => invoke<void>("ai_resume"),
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
  pendingNotice: () => invoke<{ id: string; text: string } | null>("pending_notice"),
  acknowledgeNotice: (id: string) => invoke<void>("acknowledge_notice", { id }),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  setBackdrop: (kind: string, dark: boolean) => invoke<boolean>("set_backdrop", { kind, dark }),
  windowReady: async () => {
    startupMark("invokeCalledAtMs");
    startupMark("appReadyCalls", true);
    try {
      await invoke<void>("window_ready");
      startupMark("invokeCompletedAtMs");
    } catch (error) {
      startupMark("appReadyRejected", true);
      throw error;
    }
  },
  quitApp: () => invoke<void>("quit_app"),
  openDataFolder: () => invoke<void>("open_data_folder"),
  // The error log (a text file in the logs folder): where it is, and open that folder in Explorer
  logInfo: () => invoke<LogInfo>("log_info"),
  openLogFolder: () => invoke<void>("open_log_folder"),
  // Close to the tray and start with Windows (saved next to the vault, not in the encrypted settings)
  getShellPrefs: () => invoke<ShellPrefs>("get_shell_prefs"),
  setShellPrefs: (prefs: Pick<ShellPrefs, "closeToTray" | "startWithWindows" | "startMinimized">) =>
    invoke<ShellPrefs>("set_shell_prefs", { prefs }),

  // Encrypted history and activity; only display metadata crosses IPC for lists.
  historyList: (pageId: string) => invoke<HistoryMeta[]>("history_list", { pageId }),
  historyGet: (pageId: string, versionId: string) => invoke<HistoryBody>("history_get", { pageId, versionId }),
  historySnapshot: (pageId: string) => invoke<void>("history_snapshot", { pageId }),
  historyRestore: (pageId: string, versionId: string) => invoke<void>("history_restore", { pageId, versionId }),
  activityList: (who?: string, before?: number) => invoke<ActivityEntry[]>("activity_list", { who, before, limit: 100 }),
  activityVerify: () => invoke<void>("activity_verify"),
  activityUndo: (changeSet: string) => invoke<{ restored: number; skipped: number }>("activity_undo", { changeSet }),
  activityClear: (confirmation: string) => invoke<void>("activity_clear", { confirmation }),

  // Notes
  listNotes: () => invoke<NotesTree>("list_notes"),
  listBin: () => invoke<BinEntry[]>("list_bin"),
  restoreBinItem: (id: string, restoreContainers: boolean) => invoke<NotesTree>("restore_bin_item", { id, restoreContainers }),
  editing: (id: string, active: boolean) => invoke<void>("editing", { id, active }),
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
  getPage: (id: string) => invoke<Revisioned<Page>>("get_page", { id }),
  getPageLinkTarget: (id: string) => invoke<{ sectionId: string; title: string; icon: string; isRow: boolean } | null>("get_page_link_target", { id }),
  savePage: (id: string, title: string, content: string, text: string, baseRev?: number) =>
    invoke<Revisioned<PageMeta>>("save_page", { id, title, content, text, baseRev }),
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
  getDatabase: (id: string) => invoke<Revisioned<Database>>("get_database", { id }),
  /** The whole database definition as JSON text (properties, views, templates). */
  saveDatabase: (database: Database, baseRev?: number) => invoke<Revisioned<Database>>("save_database", { databaseJson: JSON.stringify(database), baseRev }),
  queryView: (request: QueryRequest) => invoke<ViewResult>("query_view", { request }),
  queryViewWindow: (request: QueryRequest, find: string) => invoke<ViewWindow>("query_view", { request, windowed: true, find }),
  viewRows: (databaseId: string, viewId: string, generation: string, ranges: RowRange[]) =>
    invoke<DbRow[] | null>("view_rows", { request: { databaseId, viewId, generation, ranges } }),
  createRow: (databaseId: string, title: string, cells: Record<string, unknown>, templateId = "") =>
    invoke<{ row: DbRow }>("create_row", { databaseId, title, cells: JSON.stringify(cells), templateId }),
  setCells: (rowId: string, title: string | null, patch: Record<string, unknown>) =>
    invoke<{ row: DbRow }>("set_cells", { rowId, title, patch: JSON.stringify(patch) }),
  /** Runs the actions of a button property on a row. */
  pressButton: (rowId: string, propId: string) => invoke<{ row: DbRow }>("press_button", { rowId, propId }),
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
  /** The database (or one view) as a document with one table. format: html | markdown | pdf | docx. */
  exportDatabase: (databaseId: string, viewId: string | null, format: "html" | "markdown" | "pdf" | "docx") =>
    invoke<ExportSummary | null>("export_database", { databaseId, viewId, format }),
  /** Reminders that came due since the last call (title only). */
  dueReminders: () => invoke<Reminder[]>("due_reminders_now"),
  // Trash, restore and delete of a row or a database use trashPage / restorePage / deletePage with its id.
  /** One row with its cell values (Rust command `get_row`, same result as setCells with an empty patch but without touching the row). */
  getRow: (rowId: string) => invoke<{ row: DbRow }>("get_row", { rowId }),
  /** The backend opens the file dialog; null databaseId makes a new database from the CSV in the section sectionId. Resolves to null if cancelled. */
  importCsv: (databaseId: string | null, sectionId: string | null = null) => invoke<CsvImportResult | null>("import_csv", { databaseId, sectionId }),
  /** The backend asks where to save (null if cancelled) and writes the rows of the view as an unencrypted CSV. */
  exportCsv: (databaseId: string, viewId: string | null) => invoke<{ path: string; rows: number } | null>("export_csv", { databaseId, viewId }),
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
  getSticky: (id: string) => invoke<Revisioned<Sticky>>("get_sticky", { id }),
  saveSticky: (id: string, content: string, text: string, baseRev?: number) =>
    invoke<number>("save_sticky", { id, content, text, baseRev }),
  setStickyStyle: (id: string, color: string, onTop: boolean) =>
    invoke<Sticky>("set_sticky_style", { id, color, onTop }),
  setStickyGeometry: (id: string, x: number, y: number, w: number, h: number) =>
    invoke<void>("set_sticky_geometry", { id, x, y, w, h }),
  openSticky: (id: string) => invoke<void>("open_sticky", { id }),
  closeSticky: (id: string) => invoke<void>("close_sticky", { id }),
  hideStickies: () => invoke<void>("hide_stickies"),
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
  // The last five searches, kept in the encrypted vault (cleared with Clear; wiped from memory on lock).
  searchHistory: () => invoke<string[]>("search_history"),
  addSearchTerm: (term: string) => invoke<string[]>("add_search_term", { term }),
  removeSearchTerm: (term: string) => invoke<string[]>("remove_search_term", { term }),
  clearSearchHistory: () => invoke<void>("clear_search_history"),
};

/** Turns any thrown IPC error into a readable message. */
export function errorText(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return "Something went wrong.";
}
