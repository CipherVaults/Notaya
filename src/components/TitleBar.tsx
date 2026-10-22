import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Lock, Minus, Search, Square, X, Copy as Restore } from "lucide-react";
import { api } from "@/lib/ipc";
import { useApp } from "@/store/app";
import { Kbd } from "./ui";
import Logo from "./Logo";

const win = getCurrentWindow();

export default function TitleBar() {
  const unlocked = useApp((s) => s.status?.unlocked ?? false);
  const setPaletteOpen = useApp((s) => s.setPaletteOpen);
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
      className="relative z-40 flex h-11 shrink-0 items-center gap-3 pl-4 select-none"
      onDoubleClick={(e) => {
        if ((e.target as HTMLElement).dataset.tauriDragRegion !== undefined) win.toggleMaximize();
      }}
    >
      <div data-tauri-drag-region className="flex items-center gap-2.5">
        <Logo size={20} />
        <span data-tauri-drag-region className="text-[0.82rem] font-semibold tracking-tight">
          Cipherleaf
        </span>
      </div>

      <div data-tauri-drag-region className="flex flex-1 justify-center">
        {unlocked && (
          <button
            onClick={() => setPaletteOpen(true)}
            className="flex h-7 w-[min(420px,40vw)] items-center gap-2 rounded-lg border border-line bg-surface px-3 text-xs text-faint transition-colors hover:bg-surface-2 hover:text-muted"
          >
            <Search size={13} />
            <span className="flex-1 text-left">Search notes, passwords, actions…</span>
            <Kbd>Ctrl K</Kbd>
          </button>
        )}
      </div>

      <div className="flex items-center">
        {unlocked && (
          <button
            onClick={() => api.lockVault()}
            className="mr-1 flex h-7 items-center gap-1.5 rounded-lg px-2.5 text-xs font-medium text-muted transition-colors hover:bg-surface-2 hover:text-fg"
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
        "grid h-11 w-12 place-items-center text-muted transition-colors " +
        (danger ? "hover:bg-[#e81123] hover:text-white" : "hover:bg-surface-2 hover:text-fg")
      }
    >
      {children}
    </button>
  );
}
