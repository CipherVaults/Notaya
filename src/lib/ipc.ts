// Typed wrappers around the Rust commands. Every call goes through Tauri's IPC;
// the UI has no other way to touch files, the network or the clipboard.

import { invoke } from "@tauri-apps/api/core";
import type {
  Generated,
  Health,
  ImportResult,
  Login,
  LoginInput,
  LoginSummary,
  NotesTree,
  Page,
  PageMeta,
  PassphraseOptions,
  PasswordOptions,
  SearchHit,
  Settings,
  Sticky,
  Strength,
  TotpCode,
  VaultStatus,
} from "./types";

export const api = {
  // Vault
  vaultStatus: () => invoke<VaultStatus>("vault_status"),
  createVault: (password: string, keyfile: string | null) =>
    invoke<void>("create_vault", { password, keyfile }),
  unlockVault: (password: string, keyfile: string | null) =>
    invoke<void>("unlock_vault", { password, keyfile }),
  lockVault: () => invoke<void>("lock_vault"),
  changeMaster: (
    currentPassword: string,
    currentKeyfile: string | null,
    newPassword: string,
    newKeyfile: string | null,
  ) => invoke<void>("change_master", { currentPassword, currentKeyfile, newPassword, newKeyfile }),
  pickKeyfile: () => invoke<string | null>("pick_keyfile"),
  createKeyfile: () => invoke<string | null>("create_keyfile"),
  backupNow: () => invoke<string | null>("backup_now"),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  setBackdrop: (kind: string, dark: boolean) => invoke<boolean>("set_backdrop", { kind, dark }),
  windowReady: () => invoke<void>("window_ready"),
  quitApp: () => invoke<void>("quit_app"),
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

  // Passwords
  listLogins: () => invoke<LoginSummary[]>("list_logins"),
  getLogin: (id: string) => invoke<Login>("get_login", { id }),
  saveLogin: (input: LoginInput) => invoke<LoginSummary>("save_login", { input }),
  deleteLogin: (id: string) => invoke<void>("delete_login", { id }),
  setLoginFavorite: (id: string, favorite: boolean) =>
    invoke<LoginSummary>("set_login_favorite", { id, favorite }),
  copyLoginField: (id: string, field: string) => invoke<number>("copy_login_field", { id, field }),
  copySecretText: (text: string) => invoke<number>("copy_secret_text", { text }),
  totpCode: (id: string) => invoke<TotpCode>("totp_code", { id }),
  totpPreview: (secret: string) => invoke<TotpCode>("totp_preview", { secret }),
  passwordHealth: () => invoke<Health>("password_health"),
  importCsv: () => invoke<ImportResult | null>("import_csv"),
  checkBreach: (id: string) => invoke<number>("check_breach", { id }),
  checkBreachText: (password: string) => invoke<number>("check_breach_text", { password }),
  openUrl: (url: string) => invoke<void>("open_url", { url }),

  // Tools
  generatePassword: (options: PasswordOptions) => invoke<Generated>("generate_password", { options }),
  generatePassphrase: (options: PassphraseOptions) =>
    invoke<Generated>("generate_passphrase", { options }),
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
