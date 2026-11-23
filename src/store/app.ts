import { create } from "zustand";
import { onItemsChanged, type BinEntry } from "@/lib/changes";
import { api } from "@/lib/ipc";
import { startupMark } from "@/lib/startup";
import { getKey, setKey } from "@/lib/storage";
import type { NotesTree, Settings, Sticky, VaultStatus } from "@/lib/types";

export type View = "notes" | "stickies" | "settings";

/** What the active item of the Notes view is: a page, a database (A35) or a row of a database (a page with cells). */
export type ActiveKind = "page" | "database" | "row";

/** Kept inside the vault's settings record, never in browser storage. */
export interface LastPlace {
  kind: ActiveKind;
  id: string;
  viewId?: string | null;
}
export type VaultSettings = Settings & { lastPlace?: LastPlace | null };

/** A one-shot request for a view to do something once it is shown (e.g. from the command palette). */
export type Intent = "new-page" | "new-notebook" | null;

/** Where "Export…" was chosen: the dialog offers this page, its section and its notebook. */
export interface ExportTarget {
  notebookId: string;
  sectionId?: string;
  pageId?: string;
  /** What the dialog starts with. */
  initial: "page" | "section" | "notebook";
}

interface AppState {
  status: VaultStatus | null;
  settings: VaultSettings | null;
  view: View;
  tree: NotesTree;
  bin: BinEntry[];
  binAction: string | null;
  stickies: Sticky[];
  activeNotebookId: string | null;
  activeSectionId: string | null;
  activePageId: string | null;
  /** What `activePageId` is. Pages, databases and rows share the id space. */
  activeKind: ActiveKind;
  /** Full-page database view selection. Database components must read this and call setDatabaseView. */
  activeDatabaseViewId: string | null;
  showTrash: boolean;
  paletteOpen: boolean;
  intent: Intent;
  sidebarOpen: boolean;
  sidebarWidth: number;
  exportTarget: ExportTarget | null;
  importOpen: boolean;

  setIntent: (i: Intent) => void;
  /** Opens the export dialog (null closes it). */
  setExportTarget: (t: ExportTarget | null) => void;
  /** Opens or closes the import dialog. */
  setImportOpen: (open: boolean) => void;
  setView: (v: View) => void;
  refreshStatus: () => Promise<VaultStatus>;
  loadVault: () => Promise<void>;
  setSettings: (s: Settings) => void;
  setTree: (t: NotesTree) => void;
  setStickies: (s: Sticky[]) => void;
  runBinAction: (key: string, run: () => Promise<void>) => Promise<boolean>;
  openNotebook: (id: string | null) => void;
  openSection: (id: string | null) => void;
  /** Opens a page, or a database when the id is one (looked up in the tree). */
  openPage: (id: string | null, sectionId?: string) => void;
  openDatabase: (id: string) => void;
  openRow: (id: string, sectionId?: string) => void;
  setDatabaseView: (databaseId: string, viewId: string) => void;
  setShowTrash: (v: boolean) => void;
  setPaletteOpen: (v: boolean) => void;
  setSidebarOpen: (v: boolean) => void;
  setSidebarWidth: (w: number) => void;
}

/** Small, non-secret layout choices kept in the browser storage. */
const LAYOUT_KEY = "layout";
function layout(): { open: boolean; width: number } {
  try {
    const v = JSON.parse(getKey(LAYOUT_KEY) ?? "{}");
    return { open: v.open !== false, width: Math.min(420, Math.max(200, Number(v.width) || 264)) };
  } catch {
    return { open: true, width: 264 };
  }
}
function saveLayout(open: boolean, width: number) {
  try {
    setKey(LAYOUT_KEY, JSON.stringify({ open, width }));
  } catch {
    // Not critical.
  }
}

const EMPTY_TREE: NotesTree = { notebooks: [], sections: [], pages: [], rowPages: [], databases: [] };

/** One startup sweep for pre-1.6 navigation metadata; no marker or other keys are written. */
export function cleanupLegacyDatabaseViews() {
  try {
    for (let i = localStorage.length - 1; i >= 0; i--) {
      const key = localStorage.key(i);
      if (key?.startsWith("dbview:")) localStorage.removeItem(key);
    }
  } catch {
    // Storage can be unavailable; that must not prevent opening Notaya.
  }
}
cleanupLegacyDatabaseViews();

export const useApp = create<AppState>((set, get) => ({
  status: null,
  settings: null,
  view: "notes",
  tree: EMPTY_TREE,
  bin: [],
  binAction: null,
  stickies: [],
  activeNotebookId: null,
  activeSectionId: null,
  activePageId: null,
  activeKind: "page",
  activeDatabaseViewId: null,
  showTrash: false,
  paletteOpen: false,
  intent: null,
  sidebarOpen: layout().open,
  sidebarWidth: layout().width,
  exportTarget: null,
  importOpen: false,

  setIntent: (intent) => set({ intent }),
  setExportTarget: (exportTarget) => set({ exportTarget }),
  setImportOpen: (importOpen) => set({ importOpen }),
  setView: (view) => set({ view }),

  refreshStatus: async () => {
    startupMark("statusRequestedAtMs");
    const status = await api.vaultStatus();
    set({ status });
    startupMark("statusSetAtMs");
    return status;
  },

  loadVault: async () => {
    const [settings, tree, stickies, bin] = await Promise.all([
      api.getSettings(),
      api.listNotes(),
      api.listStickies(),
      api.listBin?.() ?? Promise.resolve([]),
    ]);
    const restored = await restoreLastPlace((settings as VaultSettings).lastPlace, tree);
    set({
      settings,
      tree,
      stickies,
      bin,
      ...restored,
    });
  },

  setSettings: (settings) => set({ settings }),
  setTree: (tree) => set({ tree }),
  setStickies: (stickies) => set({ stickies }),
  runBinAction: async (key, run) => {
    if (get().binAction !== null) return false;
    set({ binAction: key });
    try {
      await run();
      await refreshing;
      return true;
    } finally {
      set({ binAction: null });
    }
  },

  openNotebook: (id) => {
    const first = get().tree.sections.find((s) => s.notebookId === id)?.id ?? null;
    set({ activeNotebookId: id, activeSectionId: first, activePageId: null, activeKind: "page", activeDatabaseViewId: null, showTrash: false });
  },
  openSection: (id) => set({ activeSectionId: id, activePageId: null, activeKind: "page", activeDatabaseViewId: null, showTrash: false }),
  openPage: (id, sectionId) =>
    set(
      id
        ? { ...(sectionId ? sectionParents(get().tree, sectionId) : locationParents(get().tree, id)), activePageId: id, activeKind: get().tree.databases.some((d) => d.id === id) ? "database" : "page", activeDatabaseViewId: get().activePageId === id ? get().activeDatabaseViewId : null, showTrash: false, view: "notes" }
        : { activePageId: null, activeKind: "page", activeDatabaseViewId: null },
    ),
  openDatabase: (id) => set({ ...locationParents(get().tree, id), activePageId: id, activeKind: "database", activeDatabaseViewId: get().activePageId === id ? get().activeDatabaseViewId : null, showTrash: false, view: "notes" }),
  openRow: (id, sectionId) => set({
    ...(sectionId ? sectionParents(get().tree, sectionId) : {}),
    activePageId: id, activeKind: "row", activeDatabaseViewId: null, showTrash: false, view: "notes",
  }),
  setDatabaseView: (databaseId, viewId) => {
    if (get().activeKind === "database" && get().activePageId === databaseId) set({ activeDatabaseViewId: viewId });
  },
  setShowTrash: (showTrash) => set({ showTrash, activePageId: null, activeKind: "page", activeDatabaseViewId: null }),
  setPaletteOpen: (paletteOpen) => set({ paletteOpen }),
  setSidebarOpen: (sidebarOpen) => {
    set({ sidebarOpen });
    saveLayout(sidebarOpen, get().sidebarWidth);
  },
  setSidebarWidth: (sidebarWidth) => {
    set({ sidebarWidth });
    saveLayout(get().sidebarOpen, sidebarWidth);
  },
}));

function sectionParents(tree: NotesTree, sectionId?: string) {
  const section = tree.sections.find((s) => s.id === sectionId);
  return { activeNotebookId: section?.notebookId ?? null, activeSectionId: section?.id ?? null };
}

export function findPageMeta(tree: NotesTree, id: string) {
  return tree.pages.find((p) => p.id === id) ?? tree.rowPages.find((p) => p.id === id);
}

function findNoteMeta(tree: NotesTree, id: string) {
  return findPageMeta(tree, id) ?? tree.databases.find((d) => d.id === id);
}

function locationParents(tree: NotesTree, id: string) {
  return sectionParents(tree, findNoteMeta(tree, id)?.sectionId);
}

function restoreActiveSection() {
  const app = useApp.getState();
  if (!app.activeSectionId || app.tree.sections.some((s) => s.id === app.activeSectionId)) return;
  if (!app.activePageId) {
    app.openSection(null);
    return;
  }
  // The sidebar can select a section while a page from another section is still open.
  const parents = locationParents(app.tree, app.activePageId);
  useApp.setState(parents.activeSectionId ? parents : { activeSectionId: null });
}

export function applySectionBin(tree: NotesTree) {
  const app = useApp.getState();
  const id = app.activePageId;
  // The backend omits the section's descendants from the updated tree. This also covers
  // row children whose recorded section ID still refers to a database's former section.
  const removed = id && findNoteMeta(app.tree, id) && !findNoteMeta(tree, id);
  app.setTree(tree);
  if (removed) app.openPage(null);
  restoreActiveSection();
}

/** Rows are absent from the sidebar tree. Their parent is their containing database. */
export async function rowExportNotebook(tree: NotesTree, rowId: string) {
  try {
    const row = await api.getPage(rowId);
    if (row.trashedAt !== null) return null;
    const database = tree.databases.find((d) => d.id === row.parent && d.trashedAt === null && d.sectionId === row.sectionId);
    const section = tree.sections.find((s) => s.id === database?.sectionId);
    return tree.notebooks.find((n) => n.id === section?.notebookId) ?? null;
  } catch {
    return null;
  }
}

/** Resolve against the unlocked tree and core reads for pages hidden beneath rows. */
export async function restoreLastPlace(place: LastPlace | null | undefined, tree: NotesTree) {
  const activeNotebookId = tree.notebooks[0]?.id ?? null;
  const home = {
    view: "notes" as View,
    activeNotebookId,
    activeSectionId: tree.sections.find((s) => s.notebookId === activeNotebookId)?.id ?? null,
    activePageId: null as string | null,
    activeKind: "page" as ActiveKind,
    activeDatabaseViewId: null as string | null,
    showTrash: false,
  };
  if (!place || !["page", "row", "database"].includes(place.kind) || typeof place.id !== "string") return home;
  try {
    const item = place.kind === "row"
      ? await api.getPage(place.id)
      : place.kind === "database" ? tree.databases.find((d) => d.id === place.id) : findPageMeta(tree, place.id) ?? await api.getPage(place.id);
    if (!item || item.trashedAt !== null) return home;
    const parents = sectionParents(tree, item.sectionId);
    if (!parents.activeSectionId || !tree.notebooks.some((n) => n.id === parents.activeNotebookId)) return home;
    let parent = item.parent;
    const seen = new Set([item.id]);
    if (place.kind === "row") {
      const db = tree.databases.find((d) => d.id === parent && d.trashedAt === null && d.sectionId === item.sectionId);
      if (!db || seen.has(db.id)) return home;
      seen.add(db.id);
      parent = db.parent;
    }
    while (parent) {
      if (seen.has(parent)) return home;
      seen.add(parent);
      const ancestor = findPageMeta(tree, parent)
        ?? tree.databases.find((d) => d.id === parent)
        ?? await api.getPage(parent);
      if (ancestor.trashedAt !== null || ancestor.sectionId !== item.sectionId) return home;
      parent = ancestor.parent;
    }
    if (place.kind === "database" && place.viewId) {
      const database = await api.getDatabase(place.id);
      if (!database.views.some((v) => v.id === place.viewId)) return home;
    }
    return { ...home, ...parents, activePageId: item.id, activeKind: place.kind, activeDatabaseViewId: place.kind === "database" ? place.viewId ?? null : null };
  } catch {
    // A location or ancestor that cannot be read must not prevent opening the vault.
    return home;
  }
}


// Coalesce a batch signal into one refresh; serialize overlapping signals so old
// reads cannot replace a newer tree. The editors decide independently whether to reload.
let refreshing: Promise<void> | null = null;
let refreshAgain = false;
onItemsChanged((change) => {
  if (!change.tree || !useApp.getState().status?.unlocked) return;
  refreshAgain = true;
  if (refreshing) return;
  refreshing = (async () => {
    try {
      while (refreshAgain) {
        refreshAgain = false;
        const [tree, stickies, bin] = await Promise.all([api.listNotes(), api.listStickies(), api.listBin()]);
        if (!useApp.getState().status?.unlocked) return;
        useApp.setState({ tree, stickies, bin });
        const app = useApp.getState();
        if (app.activePageId && !findNoteMeta(tree, app.activePageId)) {
          const id = app.activePageId;
          const target = app.activeKind === "database" ? null : await api.getPageLinkTarget(id).catch(() => null);
          if (useApp.getState().activePageId === id && (!target || target.isRow !== (app.activeKind === "row"))) app.openPage(null);
        }
        restoreActiveSection();
        const current = useApp.getState();
        if (current.activeNotebookId && !current.tree.notebooks.some((n) => n.id === current.activeNotebookId)) current.openNotebook(null);
      }
    } catch { /* A lock can overtake a refresh; the unlock loads a fresh tree. */ }
    finally { refreshing = null; }
  })();
});
