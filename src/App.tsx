import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { AnimatePresence, motion } from "motion/react";
import { Toaster } from "sonner";
import { api } from "@/lib/ipc";
import { applyAppearance, cachedAppearance, isDark } from "@/lib/theme";
import { useApp } from "@/store/app";
import { ConfirmProvider, TooltipProvider } from "@/components/ui";
import TitleBar from "@/components/TitleBar";
import UnlockScreen from "@/features/unlock/UnlockScreen";
import Shell from "@/components/Shell";

export default function App() {
  const status = useApp((s) => s.status);
  const refreshStatus = useApp((s) => s.refreshStatus);
  const theme = useApp((s) => s.settings?.theme ?? cachedAppearance().theme);

  useEffect(() => {
    applyAppearance(cachedAppearance(), true).finally(() => {
      refreshStatus().finally(() => {
        // Show the window only after the first paint, so there is no white flash.
        requestAnimationFrame(() => requestAnimationFrame(() => api.windowReady()));
      });
    });

    // Rust tells us when the vault locks (idle, Win+L, sleep, USB removed...).
    // Reloading discards every decrypted value held by the UI.
    const unlisten = listen<string>("vault-locked", () => window.location.reload());
    return () => {
      unlisten.then((f) => f());
    };
  }, [refreshStatus]);

  return (
    <TooltipProvider>
      <ConfirmProvider>
        <div className="flex h-full flex-col">
          <TitleBar />
          <div className="relative min-h-0 flex-1">
            <AnimatePresence mode="wait">
              {status && (
                <motion.div
                  key={status.unlocked ? "shell" : "lock"}
                  className="absolute inset-0"
                  initial={{ opacity: 0, scale: status.unlocked ? 0.985 : 1 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 1.01 }}
                  transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }}
                >
                  {status.unlocked ? <Shell /> : <UnlockScreen />}
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        </div>
        <Toaster
          position="bottom-right"
          theme={isDark(theme) ? "dark" : "light"}
          toastOptions={{
            style: {
              background: "var(--elevated)",
              border: "1px solid var(--border)",
              color: "var(--text)",
              borderRadius: "14px",
            },
          }}
        />
      </ConfirmProvider>
    </TooltipProvider>
  );
}
