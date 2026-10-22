import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Lock, Minus, PanelLeft, Search, Square, X, Copy as Restore } from "lucide-react";
import { api } from "@/lib/ipc";
import { useApp } from "@/store/app";
import { Kbd } from "./ui";
import Logo from "./Logo";

const win = getCurrentWindow();

export default function TitleBar() {
  const unlocked = useApp((s) => s.status?.unlocked ?? false);
  const setPaletteOpen = useApp((s) => s.setPaletteOpen);
  const sidebarOpen = useApp((s) => s.sidebarOpen);
  const setSidebarOpen = useApp((s) => s.setSidebarOpen);
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
      className="relative z-40 flex h-10 shrink-0 items-center gap-2 bg-sidebar pl-3 select-none"
      onDoubleClick={(e) => {
        if ((e.target as HTMLElement).dataset.tauriDragRegion !== undefined) win.toggleMaximize();
      }}
    >
      <div data-tauri-drag-region className="flex items-center gap-2">
        <Logo size={18} />
        <span data-tauri-drag-region className="text-[0.8rem] font-semibold tracking-tight">
          Cipherleaf
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

      <div data-tauri-drag-region className="flex flex-1 justify-center">
        {unlocked && (
          <button
            onClick={() => setPaletteOpen(true)}
            className="flex h-7 w-[min(380px,36vw)] items-center gap-2 rounded-md border border-line bg-surface px-2.5 text-xs text-faint transition-colors hover:bg-surface-2 hover:text-muted"
          >
            <Search size={13} />
            <span className="flex-1 text-left">Search</span>
            <Kbd>Ctrl K</Kbd>
          </button>
        )}
      </div>

      <div className="flex items-center">
        {unlocked && (
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
