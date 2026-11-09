// Development-only host until lane A's TitleBar integration is available.
// It renders the real SettingsView, marker and dialogs with mock IPC only.
import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { Toaster } from "sonner";
import { ConfirmProvider, TooltipProvider } from "@/components/ui";
import AiMarker from "@/features/ai/AiMarker";
import SettingsView from "@/features/settings/SettingsView";
import { useApp } from "@/store/app";
import { applyAppearance } from "@/lib/theme";
import "@fontsource-variable/inter";
import "@/styles/app.css";

function Harness() {
  const [ready, setReady] = useState(false);
  const [settings, setSettings] = useState(true);
  useEffect(() => {
    void (async () => {
      await useApp.getState().refreshStatus();
      await useApp.getState().loadVault();
      const value = useApp.getState().settings!;
      const theme = new URLSearchParams(location.search).get("theme");
      if (theme && ["sunrise", "sunrise-dark", "black", "graphite", "white", "paper"].includes(theme)) {
        value.theme = theme as typeof value.theme;
        useApp.getState().setSettings({ ...value });
      }
      await applyAppearance(value, true);
      setReady(true);
    })();
  }, []);
  return <TooltipProvider><ConfirmProvider><div className="flex h-screen flex-col bg-bg text-fg">
    <header className="flex h-10 shrink-0 items-center justify-between gap-3 bg-sidebar px-3">
      <span className="text-xs">AI screens · mock data</span>{ready && <AiMarker />}
    </header>
    <nav className="flex gap-4 border-b border-line px-6 py-2 text-sm">
      <button onClick={() => setSettings(true)}>Settings</button><button onClick={() => setSettings(false)}>Notes</button>
    </nav>
    <main className="min-h-0 flex-1">{ready && (settings ? <SettingsView /> : <p className="p-8">Mock notes view</p>)}</main>
    <Toaster />
  </div></ConfirmProvider></TooltipProvider>;
}

createRoot(document.getElementById("root")!).render(<Harness />);
