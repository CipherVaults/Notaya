// Mirrors the Rust structs in src-tauri/src/model.rs and the command results.

export interface VaultStatus {
  exists: boolean;
  unlocked: boolean;
  keyfileRequired: boolean;
  /** The vault has a master password. False: notes open straight away (security off). */
  securityOn: boolean;
  /** No vault file, but earlier copies exist: ask whether to restore or start new notes. */
  earlierCopies: boolean;
  /** A vault with security off could not be opened (for example a damaged file). */
  openError: string | null;
  /** The vault header could be read, so keyfileRequired is known. */
  headerOk: boolean;
  dataDir: string;
  version: string;
}

export interface EnableSecurityResult {
  /** How many earlier copies made while security was off were deleted. */
  removedCopies: number;
  /** Deleting was asked for, but the fresh backup failed, so the earlier copies were kept. */
  backupFailed: boolean;
}

/** One automatic backup file in the data folder (newest first from the backend). */
export interface BackupInfo {
  name: string;
  modifiedMs: number;
  size: number;
}

export interface RestoreResult {
  /** File name of the old vault that was kept in the data folder, if there was one. */
  undoFile: string | null;
}

export type ThemeMode = "black" | "graphite" | "white" | "paper" | "system";
export type Backdrop = "mica" | "acrylic" | "none";

export interface Settings {
  theme: ThemeMode;
  accent: string;
  backdrop: Backdrop;
  font: "inter" | "system" | "serif" | "mono";
  fontSize: number;
  editorWidth: "narrow" | "normal" | "wide" | "full";
  density: "comfortable" | "compact";
  unlockBackground: "aurora" | "mesh" | "midnight" | "plain";
  idleLockMinutes: number;
  lockOnMinimize: boolean;
  screenCaptureProtection: boolean;
  stickyDefaultColor: string;
}

export interface Notebook {
  id: string;
  name: string;
  color: string;
  icon: string;
  order: number;
  created: number;
  updated: number;
}

export interface Section {
  id: string;
  notebookId: string;
  name: string;
  color: string;
  order: number;
  created: number;
  updated: number;
}

export interface PageMeta {
  id: string;
  sectionId: string;
  title: string;
  preview: string;
  words: number;
  order: number;
  favorite: boolean;
  icon: string;
  /** Id of the page this one sits inside, or empty. */
  parent: string;
  trashedAt: number | null;
  created: number;
  updated: number;
}

export interface Page {
  id: string;
  sectionId: string;
  title: string;
  content: string;
  text: string;
  order: number;
  favorite: boolean;
  icon: string;
  /** "gradient:<name>", an image URL, or empty. */
  cover: string;
  parent: string;
  trashedAt: number | null;
  created: number;
  updated: number;
}

export interface NotesTree {
  notebooks: Notebook[];
  sections: Section[];
  pages: PageMeta[];
}

export interface Sticky {
  id: string;
  content: string;
  text: string;
  color: string;
  x: number;
  y: number;
  w: number;
  h: number;
  onTop: boolean;
  open: boolean;
  created: number;
  updated: number;
}

export interface Strength {
  score: number;
  guessesLog10: number;
  crackTime: string;
  warning: string | null;
  suggestions: string[];
}

export interface SearchHit {
  kind: "page" | "sticky" | "notebook";
  id: string;
  title: string;
  subtitle: string;
  snippet: string;
  score: number;
}

