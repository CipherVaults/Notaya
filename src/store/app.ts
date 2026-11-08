import { create } from "zustand";
import { api } from "@/lib/ipc";
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
  openNotebook: (id: string | null) => void;
  openSection: (id: string | null) => void;
  /** Opens a page, or a database when the id is one (looked up in the tree). */
  openPage: (id: string | null) => void;
  openDatabase: (id: string) => void;
  openRow: (id: string) => void;
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

const EMPTY_TREE: NotesTree = { notebooks: [], sections: [], pages: [], databases: [] };

export const useApp = create<AppState>((set, get) => ({
  status: null,
  settings: null,
  view: "notes",
  tree: EMPTY_TREE,
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
    const status = await api.vaultStatus();
    set({ status });
    return status;
  },

  loadVault: async () => {
    const [settings, tree, stickies] = await Promise.all([
      api.getSettings(),
      api.listNotes(),
      api.listStickies(),
    ]);
    const restored = await restoreLastPlace((settings as VaultSettings).lastPlace, tree);
    set({
      settings,
      tree,
      stickies,
      ...restored,
    });
  },

  setSettings: (settings) => set({ settings }),
  setTree: (tree) => set({ tree }),
  setStickies: (stickies) => set({ stickies }),

  openNotebook: (id) => {
    const first = get().tree.sections.find((s) => s.notebookId === id)?.id ?? null;
    set({ activeNotebookId: id, activeSectionId: first, activePageId: null, activeKind: "page", activeDatabaseViewId: null, showTrash: false });
  },
  openSection: (id) => set({ activeSectionId: id, activePageId: null, activeKind: "page", activeDatabaseViewId: null, showTrash: false }),
  openPage: (id) =>
    set(
      id
        ? { ...locationParents(get().tree, id), activePageId: id, activeKind: get().tree.databases.some((d) => d.id === id) ? "database" : "page", activeDatabaseViewId: get().activePageId === id ? get().activeDatabaseViewId : null, showTrash: false, view: "notes" }
        : { activePageId: null, activeKind: "page", activeDatabaseViewId: null },
    ),
  openDatabase: (id) => set({ ...locationParents(get().tree, id), activePageId: id, activeKind: "database", activeDatabaseViewId: get().activePageId === id ? get().activeDatabaseViewId : null, showTrash: false, view: "notes" }),
  openRow: (id) => set({ activePageId: id, activeKind: "row", activeDatabaseViewId: null, showTrash: false, view: "notes" }),
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

function locationParents(tree: NotesTree, id: string) {
  return sectionParents(tree, (tree.pages.find((p) => p.id === id) ?? tree.databases.find((d) => d.id === id))?.sectionId);
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
      : place.kind === "database" ? tree.databases.find((d) => d.id === place.id) : tree.pages.find((p) => p.id === place.id) ?? await api.getPage(place.id);
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
      const ancestor = tree.pages.find((p) => p.id === parent)
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
