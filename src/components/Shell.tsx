import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { KeyRound, Loader2, NotebookPen, Settings as SettingsIcon, StickyNote } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { applyAppearance } from "@/lib/theme";
import { cn } from "@/lib/utils";
import { useApp, type View } from "@/store/app";
import { Tip } from "./ui";
import CommandPalette from "./CommandPalette";
import NotesView from "@/features/notes/NotesView";
import KeePassXCPanel from "@/features/passwords/KeePassXCPanel";
import StickyBoard from "@/features/sticky/StickyBoard";
import SettingsView from "@/features/settings/SettingsView";

const NAV: { view: View; label: string; icon: React.ReactNode; shortcut: string }[] = [
  { view: "notes", label: "Notes", icon: <NotebookPen size={19} />, shortcut: "1" },
  { view: "passwords", label: "Passwords", icon: <KeyRound size={19} />, shortcut: "2" },
  { view: "stickies", label: "Sticky notes", icon: <StickyNote size={19} />, shortcut: "3" },
];

export default function Shell() {
  const view = useApp((s) => s.view);
  const setView = useApp((s) => s.setView);
  const settings = useApp((s) => s.settings);
  const loadVault = useApp((s) => s.loadVault);
  const setPaletteOpen = useApp((s) => s.setPaletteOpen);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    loadVault()
      .then(() => setReady(true))
      .catch((e) => toast.error(errorText(e)));
  }, [loadVault]);

  useEffect(() => {
    if (settings) applyAppearance(settings, true);
  }, [settings]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!e.ctrlKey || e.altKey) return;
      const k = e.key.toLowerCase();
      if (k === "k") {
        e.preventDefault();
        setPaletteOpen(true);
      } else if (k === "l") {
        e.preventDefault();
        api.lockVault();
      } else if (k === ",") {
        e.preventDefault();
        setView("settings");
      } else {
        const nav = NAV.find((n) => n.shortcut === k);
        if (nav) {
          e.preventDefault();
          setView(nav.view);
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [setPaletteOpen, setView]);

  if (!ready) {
    return (
      <div className="grid h-full place-items-center text-muted">
        <Loader2 className="animate-spin" />
      </div>
    );
  }

  return (
    <div className="flex h-full">
      <nav className="flex w-[68px] shrink-0 flex-col items-center gap-1.5 pb-4 pt-2">
        {NAV.map((n) => (
          <NavButton key={n.view} label={n.label} icon={n.icon} keyHint={n.shortcut} active={view === n.view} onClick={() => setView(n.view)} />
        ))}
        <div className="flex-1" />
        <NavButton
          label="Settings"
          icon={<SettingsIcon size={19} />}
          keyHint=","
          active={view === "settings"}
          onClick={() => setView("settings")}
        />
      </nav>

      <main className="relative mb-2 mr-2 min-w-0 flex-1 overflow-hidden rounded-2xl border border-line bg-surface">
        <AnimatePresence mode="wait">
          <motion.div
            key={view}
            className="absolute inset-0"
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            transition={{ duration: 0.18, ease: "easeOut" }}
          >
            {view === "notes" && <NotesView />}
            {view === "passwords" && <KeePassXCPanel />}
            {view === "stickies" && <StickyBoard />}
            {view === "settings" && <SettingsView />}
          </motion.div>
        </AnimatePresence>
      </main>

      <CommandPalette />
    </div>
  );
}

function NavButton({
  label,
  icon,
  active,
  onClick,
  keyHint: hint,
}: {
  label: string;
  icon: React.ReactNode;
  active: boolean;
  onClick: () => void;
  keyHint?: string;
}) {
  return (
    <Tip label={`${label}${hint ? `  ·  Ctrl+${hint}` : ""}`} side="right">
      <button
        onClick={onClick}
        aria-label={label}
        className={cn(
          "relative grid h-11 w-11 place-items-center rounded-xl transition-colors",
          active ? "text-accent" : "text-muted hover:bg-surface-2 hover:text-fg",
        )}
      >
        {active && (
          <motion.span
            layoutId="nav-active"
            className="absolute inset-0 rounded-xl bg-accent-soft"
            transition={{ type: "spring", stiffness: 500, damping: 36 }}
          />
        )}
        {active && (
          <motion.span
            layoutId="nav-bar"
            className="absolute -left-[12px] h-5 w-[3px] rounded-full bg-accent"
            transition={{ type: "spring", stiffness: 500, damping: 36 }}
          />
        )}
        <span className="relative">{icon}</span>
      </button>
    </Tip>
  );
}
