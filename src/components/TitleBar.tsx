import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ChevronLeft, ChevronRight, Download, Lock, Minus, PanelLeft, Search, Square, Upload, X, Copy as Restore } from "lucide-react";
import { api } from "@/lib/ipc";
import { goBack, goForward, useNav } from "@/lib/navHistory";
import { useApp } from "@/store/app";
import { Kbd } from "./ui";
import Logo from "./Logo";
import { openExport } from "@/features/notes/ExportDialog";

const win = getCurrentWindow();

export default function TitleBar() {
  const unlocked = useApp((s) => s.status?.unlocked ?? false);
  // The Lock button exists only while security is on (a vault without a password has nothing to lock).
  const securityOn = useApp((s) => s.status?.securityOn ?? false);
  const setPaletteOpen = useApp((s) => s.setPaletteOpen);
  const sidebarOpen = useApp((s) => s.sidebarOpen);
  const setSidebarOpen = useApp((s) => s.setSidebarOpen);
  const tree = useApp((s) => s.tree);
  const activeNotebookId = useApp((s) => s.activeNotebookId);
  const activePageId = useApp((s) => s.activePageId);
  const view = useApp((s) => s.view);
  const showTrash = useApp((s) => s.showTrash);
  const page = view === "notes" && !showTrash ? tree.pages.find((p) => p.id === activePageId && p.trashedAt === null) : undefined;
  const database = view === "notes" && !showTrash ? tree.databases.find((d) => d.id === activePageId && d.trashedAt === null) : undefined;
  const section = tree.sections.find((s) => s.id === (page ?? database)?.sectionId);
  const exportNotebookId = section?.notebookId ?? (tree.notebooks.some((n) => n.id === activeNotebookId) ? activeNotebookId : tree.notebooks[0]?.id);
  const canBack = useNav((s) => s.canBack);
  const canForward = useNav((s) => s.canForward);
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    win.isMaximized().then(setMaximized);
    const un = win.onResized(() => win.isMaximized().then(setMaximized));
    return () => {
      un.then((f) => f());
    };
  }, []);

  return (
    <header
      data-tauri-drag-region
      className="title-bar relative z-40 flex h-10 shrink-0 items-center gap-2 bg-sidebar pl-3 select-none"
      onDoubleClick={(e) => {
        if ((e.target as HTMLElement).dataset.tauriDragRegion !== undefined) win.toggleMaximize();
      }}
    >
      <div data-tauri-drag-region className="flex items-center gap-2">
        <Logo size={18} />
        <span data-tauri-drag-region className="text-[0.8rem] font-semibold tracking-tight">
          Notaya
        </span>
      </div>

      {unlocked && (
        <button
          onClick={() => setSidebarOpen(!sidebarOpen)}
          aria-label={sidebarOpen ? "Hide sidebar" : "Show sidebar"}
          title={`${sidebarOpen ? "Hide" : "Show"} sidebar  ·  Ctrl+\\`}
          className="ml-1 grid h-7 w-7 place-items-center rounded-md text-muted transition-colors hover:bg-surface-2 hover:text-fg"
        >
          <PanelLeft size={16} />
        </button>
      )}

      {unlocked && (
        <div className="flex items-center">
          <button
            onClick={goBack}
            disabled={!canBack}
            aria-label="Back"
            title="Back  ·  Backspace, Alt+Left or the mouse's back button"
            className="grid h-7 w-7 place-items-center rounded-md text-muted transition-colors hover:bg-surface-2 hover:text-fg disabled:pointer-events-none disabled:opacity-35"
          >
            <ChevronLeft size={17} />
          </button>
          <button
            onClick={goForward}
            disabled={!canForward}
            aria-label="Forward"
            title="Forward  ·  Shift+Backspace, Alt+Right or the mouse's forward button"
            className="grid h-7 w-7 place-items-center rounded-md text-muted transition-colors hover:bg-surface-2 hover:text-fg disabled:pointer-events-none disabled:opacity-35"
          >
            <ChevronRight size={17} />
          </button>
        </div>
      )}

      <div data-tauri-drag-region className="flex min-w-0 flex-1 justify-center self-stretch items-center">
        {unlocked && (
          <button
            onClick={() => setPaletteOpen(true)}
            aria-label="Search"
            className="title-search flex h-7 w-[min(380px,28vw)] min-w-0 items-center gap-2 rounded-md border border-line bg-surface px-2.5 text-xs text-muted transition-colors hover:bg-surface-2 hover:text-fg"
          >
            <Search size={13} />
            <span className="flex-1 text-left">Search</span>
            <span className="title-search-hint"><Kbd>Ctrl K</Kbd></span>
          </button>
        )}
      </div>

      {unlocked && (
        <div className="flex shrink-0 items-center gap-0.5" aria-label="Import and export">
          <button
            onClick={() => useApp.getState().setImportOpen(true)}
            className="flex h-7 items-center gap-1.5 rounded-md px-2 text-xs font-medium text-muted transition-colors hover:bg-surface-2 hover:text-fg"
          >
            <Upload size={14} /> Import
          </button>
          <button
            onClick={() => {
              if (page) openExport("page", page.id);
              else if (exportNotebookId) openExport("notebook", exportNotebookId);
            }}
            disabled={!page && !exportNotebookId}
            className="flex h-7 items-center gap-1.5 rounded-md px-2 text-xs font-medium text-muted transition-colors hover:bg-surface-2 hover:text-fg disabled:opacity-40"
          >
            <Download size={14} /> Export
          </button>
        </div>
      )}

      <div className="flex shrink-0 items-center">
        {unlocked && securityOn && (
          <button
            onClick={() => api.lockVault()}
            className="mr-1 flex h-7 items-center gap-1.5 rounded-md px-2.5 text-xs font-medium text-muted transition-colors hover:bg-surface-2 hover:text-fg"
            title="Lock now (Ctrl+L)"
          >
            <Lock size={13} />
            Lock
          </button>
        )}
        <WinButton label="Minimize" onClick={() => win.minimize()}>
          <Minus size={15} />
        </WinButton>
        <WinButton label={maximized ? "Restore" : "Maximize"} onClick={() => win.toggleMaximize()}>
          {maximized ? <Restore size={12} className="rotate-180" /> : <Square size={12} />}
        </WinButton>
        <WinButton label="Close" danger onClick={() => win.close()}>
          <X size={16} />
        </WinButton>
      </div>
    </header>
  );
}

function WinButton({
  children,
  label,
  onClick,
  danger,
}: {
  children: React.ReactNode;
  label: string;
  onClick: () => void;
  danger?: boolean;
}) {
  return (
    <button
      aria-label={label}
      title={label}
      onClick={onClick}
      className={
        "grid h-10 w-11 place-items-center text-muted transition-colors " +
        (danger ? "hover:bg-[#e81123] hover:text-white" : "hover:bg-surface-2 hover:text-fg")
      }
    >
      {children}
    </button>
  );
}
