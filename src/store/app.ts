import { create } from "zustand";
import { api } from "@/lib/ipc";
import type { NotesTree, Settings, Sticky, VaultStatus } from "@/lib/types";

// Passwords are managed in the bundled KeePassXC (amendment A9), so there is no
// password or generator view here, only a "passwords" panel that opens it.
export type View = "notes" | "passwords" | "stickies" | "settings";

/** A one-shot request for a view to do something once it is shown (e.g. from the command palette). */
export type Intent = "new-page" | null;

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

  setIntent: (i: Intent) => void;
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

  setIntent: (intent) => set({ intent }),
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
  openPage: (id) => set({ activePageId: id }),
  setShowTrash: (showTrash) => set({ showTrash, activePageId: null }),
  setPaletteOpen: (paletteOpen) => set({ paletteOpen }),
}));
