// Mirrors the Rust structs in src-tauri/src/model.rs and the command results.

export interface VaultStatus {
  exists: boolean;
  unlocked: boolean;
  keyfileRequired: boolean;
  dataDir: string;
  version: string;
}

export type ThemeMode = "dark" | "light" | "system";
export type Backdrop = "mica" | "acrylic" | "none";

export interface Settings {
  theme: ThemeMode;
  accent: string;
  backdrop: Backdrop;
  font: "inter" | "system" | "serif" | "mono";
  fontSize: number;
  editorWidth: "narrow" | "normal" | "wide";
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

export interface Generated {
  value: string;
  entropy: number;
}

export interface Strength {
  score: number;
  guessesLog10: number;
  crackTime: string;
  warning: string | null;
  suggestions: string[];
}

export interface PassphraseOptions {
  words: number;
  separator: string;
  capitalize: boolean;
  includeNumber: boolean;
}

export interface SearchHit {
  kind: "page" | "sticky" | "notebook";
  id: string;
  title: string;
  subtitle: string;
  snippet: string;
  score: number;
}

/** The bundled KeePassXC, which holds the passwords (amendment A9). */
export interface KeePassXCStatus {
  /** KeePassXC is present next to Cipherleaf. */
  installed: boolean;
  /** Its executable matches the verified 2.8.0 release. */
  verified: boolean;
  version: string;
  /** The password database that will be opened, if one exists. */
  database: string | null;
  dataDir: string;
  /** KeePassXC was started by Cipherleaf and is still running. */
  running: boolean;
}
