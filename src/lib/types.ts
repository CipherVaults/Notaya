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
  clipboardClearSeconds: number;
  screenCaptureProtection: boolean;
  breachCheckEnabled: boolean;
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

export interface CustomField {
  name: string;
  value: string;
  secret: boolean;
}

export interface PasswordHistory {
  password: string;
  changed: number;
}

export interface Login {
  id: string;
  title: string;
  username: string;
  password: string;
  urls: string[];
  notes: string;
  totp: string;
  fields: CustomField[];
  tags: string[];
  folder: string;
  favorite: boolean;
  color: string;
  created: number;
  updated: number;
  passwordChanged: number;
  history: PasswordHistory[];
}

export interface LoginSummary {
  id: string;
  title: string;
  username: string;
  url: string;
  folder: string;
  tags: string[];
  favorite: boolean;
  color: string;
  hasTotp: boolean;
  updated: number;
  passwordChanged: number;
}

export interface LoginInput {
  id: string | null;
  title: string;
  username: string;
  password: string;
  urls: string[];
  notes: string;
  totp: string;
  fields: CustomField[];
  tags: string[];
  folder: string;
  favorite: boolean;
  color: string;
}

export interface TotpCode {
  code: string;
  remaining: number;
  period: number;
}

export interface HealthIssue {
  id: string;
  title: string;
  username: string;
  detail: string;
}

export interface Health {
  total: number;
  score: number;
  weak: HealthIssue[];
  reused: HealthIssue[];
  old: HealthIssue[];
}

export interface ImportResult {
  imported: number;
  skipped: number;
  path: string;
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

export interface PasswordOptions {
  length: number;
  lower: boolean;
  upper: boolean;
  digits: boolean;
  symbols: boolean;
  avoidAmbiguous: boolean;
}

export interface PassphraseOptions {
  words: number;
  separator: string;
  capitalize: boolean;
  includeNumber: boolean;
}

export interface SearchHit {
  kind: "page" | "login" | "sticky" | "notebook";
  id: string;
  title: string;
  subtitle: string;
  snippet: string;
  score: number;
}
