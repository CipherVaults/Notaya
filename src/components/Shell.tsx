import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { Loader2 } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { applyAppearance } from "@/lib/theme";
import { useApp, type View } from "@/store/app";
import CommandPalette from "./CommandPalette";
import Sidebar from "./Sidebar";
import NotesView from "@/features/notes/NotesView";
import ExportDialog from "@/features/notes/ExportDialog";
import StickyBoard from "@/features/sticky/StickyBoard";
import SettingsView from "@/features/settings/SettingsView";

const SHORTCUTS: Record<string, View> = { "1": "notes", "3": "stickies" };

export default function Shell() {
  const view = useApp((s) => s.view);
  const setView = useApp((s) => s.setView);
  const settings = useApp((s) => s.settings);
  const securityOn = useApp((s) => s.status?.securityOn ?? false);
  const loadVault = useApp((s) => s.loadVault);
  const setPaletteOpen = useApp((s) => s.setPaletteOpen);
  const sidebarOpen = useApp((s) => s.sidebarOpen);
  const setSidebarOpen = useApp((s) => s.setSidebarOpen);
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
        if (securityOn) api.lockVault();
      } else if (k === ",") {
        e.preventDefault();
        setView("settings");
      } else if (k === "\\") {
        e.preventDefault();
        setSidebarOpen(!useApp.getState().sidebarOpen);
      } else if (SHORTCUTS[k]) {
        e.preventDefault();
        setView(SHORTCUTS[k]);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [setPaletteOpen, setView, setSidebarOpen, securityOn]);

  if (!ready) {
    return (
      <div className="grid h-full place-items-center text-muted">
        <Loader2 className="animate-spin" />
      </div>
    );
  }

  return (
    <div className="flex h-full">
      {sidebarOpen && <Sidebar />}

      <main className="relative min-w-0 flex-1 overflow-hidden bg-bg">
        <AnimatePresence mode="wait">
          <motion.div
            key={view}
            className="absolute inset-0"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.14, ease: "easeOut" }}
          >
            {view === "notes" && <NotesView />}
            {view === "stickies" && <StickyBoard />}
            {view === "settings" && <SettingsView />}
          </motion.div>
        </AnimatePresence>
      </main>

      <CommandPalette />
      <ExportDialog />
    </div>
  );
}
