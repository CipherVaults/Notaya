import { create } from "zustand";
import { api } from "@/lib/ipc";
import { getKey, setKey } from "@/lib/storage";
import type { NotesTree, Settings, Sticky, VaultStatus } from "@/lib/types";

export type View = "notes" | "stickies" | "settings";

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
  settings: Settings | null;
  view: View;
  tree: NotesTree;
  stickies: Sticky[];
  activeNotebookId: string | null;
  activeSectionId: string | null;
  activePageId: string | null;
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
  openPage: (id: string | null) => void;
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

const EMPTY_TREE: NotesTree = { notebooks: [], sections: [], pages: [] };

export const useApp = create<AppState>((set, get) => ({
  status: null,
  settings: null,
  view: "notes",
  tree: EMPTY_TREE,
  stickies: [],
  activeNotebookId: null,
  activeSectionId: null,
  activePageId: null,
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
    const firstNotebook = tree.notebooks[0]?.id ?? null;
    const firstSection = tree.sections.find((s) => s.notebookId === firstNotebook)?.id ?? null;
    set({
      settings,
      tree,
      stickies,
      activeNotebookId: get().activeNotebookId ?? firstNotebook,
      activeSectionId: get().activeSectionId ?? firstSection,
    });
  },

  setSettings: (settings) => set({ settings }),
  setTree: (tree) => set({ tree }),
  setStickies: (stickies) => set({ stickies }),

  openNotebook: (id) => {
    const first = get().tree.sections.find((s) => s.notebookId === id)?.id ?? null;
    set({ activeNotebookId: id, activeSectionId: first, activePageId: null, showTrash: false });
  },
  openSection: (id) => set({ activeSectionId: id, activePageId: null, showTrash: false }),
  openPage: (id) => set(id ? { activePageId: id, showTrash: false, view: "notes" } : { activePageId: null }),
  setShowTrash: (showTrash) => set({ showTrash, activePageId: null }),
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
