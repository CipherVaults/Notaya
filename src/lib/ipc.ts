// Typed wrappers around the Rust commands. Every call goes through Tauri's IPC;
// the UI has no other way to touch files, the network or the clipboard.

import { invoke } from "@tauri-apps/api/core";
import { flushAll } from "./flush";
import type {
  BackupInfo,
  CalcEntry,
  EnableSecurityResult,
  NotesTree,
  Page,
  PageMeta,
  RestoreResult,
  SearchHit,
  Settings,
  Sticky,
  Strength,
  VaultStatus,
} from "./types";

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
