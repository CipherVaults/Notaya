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
  /** An installed Notaya with no notes yet: offer starting new notes or importing a portable copy. */
  offerImport: boolean;
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

/** What "Import notes from a portable copy" did. */
export interface ImportSummary {
  backupsCopied: number;
  backupsSkipped: number;
}

export interface RestoreResult {
  /** File name of the old vault that was kept in the data folder, if there was one. */
  undoFile: string | null;
}

export type ThemeMode = "sunrise" | "sunrise-dark" | "black" | "graphite" | "white" | "paper" | "system";
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

/** The Windows section of Settings (kept next to the vault, readable while it is locked). */
export interface ShellPrefs {
  /** The close button hides Notaya to the tray instead of quitting. */
  closeToTray: boolean;
  /** Windows starts this copy of Notaya when you sign in. */
  startWithWindows: boolean;
  /** When Windows starts it, Notaya stays hidden in the tray. */
  startMinimized: boolean;
  /** False for a portable copy (its start-up entry names the folder it is in now). */
  installed: boolean;
  /** False when no tray icon could be made; the close button then quits. */
  trayAvailable: boolean;
}

/** One line of the shared calculator history. */
export interface CalcEntry {
  expr: string;
  result: string;
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

export type ExportFormat = "markdown" | "pdf" | "docx";

/** What to export. The backend opens the save dialog and writes the file itself. */
export interface ExportRequest {
  scope: "pages" | "section" | "notebook";
  ids: string[];
  format: ExportFormat;
  includeSubpages: boolean;
}

/** What an export wrote (mirrors `ExportSummary` in src-tauri/src/export/mod.rs). */
export interface ExportSummary {
  path: string;
  pages: number;
  images: number;
  /** Pictures that could not be read from the vault. */
  missingImages: number;
  /** Pictures the PDF could not hold (GIF, WebP). */
  imagesLeftOut: number;
  /** Characters the PDF fonts do not have. */
  charsReplaced: number;
  /** Markdown: the folder the pictures went to. */
  assetsFolder: string | null;
}

/** Where an import comes from. The backend opens the dialog itself. */
export type ImportSource = "files" | "folder";

/** One thing an import left out, and why. */
export interface ImportSkipped {
  item: string;
  reason: string;
}

/** What an import created (mirrors `ImportSummary` in src-tauri/src/importers/plan.rs). */
export interface NotesImportSummary {
  notebookId: string;
  notebookName: string;
  sections: number;
  pages: number;
  images: number;
  skipped: ImportSkipped[];
  notes: string[];
}

export interface SearchHit {
  kind: "page" | "sticky" | "notebook";
  id: string;
  title: string;
  subtitle: string;
  snippet: string;
  score: number;
}

