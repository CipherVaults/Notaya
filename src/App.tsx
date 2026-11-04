import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { AnimatePresence, motion } from "motion/react";
import { toast, Toaster } from "sonner";
import { flushAll } from "@/lib/flush";
import { api } from "@/lib/ipc";
import { applyAppearance, cachedAppearance, isDark } from "@/lib/theme";
import { useApp } from "@/store/app";
import { ConfirmProvider, TooltipProvider } from "@/components/ui";
import TitleBar from "@/components/TitleBar";
import UnlockScreen from "@/features/unlock/UnlockScreen";
import Shell from "@/components/Shell";
import { pollNotices } from "@/features/database/notices";

export default function App() {
  const status = useApp((s) => s.status);
  const refreshStatus = useApp((s) => s.refreshStatus);
  const theme = useApp((s) => s.settings?.theme ?? cachedAppearance().theme);

  // Database reminders: shown inside Notaya, title only, once per run. Only while unlocked.
  const unlocked = status?.unlocked ?? false;
  useEffect(() => {
    if (!unlocked) return;
    let stopped = false;
    const check = () => {
      if (!stopped) void pollNotices();
    };
    check();
    const t = setInterval(check, 60000);
    return () => {
      stopped = true;
      clearInterval(t);
    };
  }, [unlocked]);

  useEffect(() => {
    // A message saved just before a reload (for example a restore that failed): show it once.
    try {
      const notice = sessionStorage.getItem("notaya.notice");
      if (notice) {
        sessionStorage.removeItem("notaya.notice");
        toast.error(notice, { duration: 12000 });
      }
    } catch {
      // Storage may be unavailable; nothing to show then.
    }

    // A message the backend wants shown once (for example after a backup failed).
    // A failed backup is shown as an error; the other messages are information.
    const showNotice = () =>
      api
        .takeNotice()
        .then((text) => {
          if (!text) return;
          if (text.startsWith("Notaya could not")) toast.error(text, { duration: 15000 });
          else toast.info(text, { duration: 15000 });
        })
        .catch(() => {});
    showNotice();

    applyAppearance(cachedAppearance(), true).finally(() => {
      refreshStatus().finally(() => {
        // Show the window only after the first paint, so there is no white flash.
        requestAnimationFrame(() => requestAnimationFrame(() => api.windowReady()));
      });
    });

    // Rust tells us when the vault locks (idle, Win+L, sleep, USB removed...).
    // Reloading discards every decrypted value held by the UI.
    // If a restore failed after the vault was locked, the payload is "restore-failed: <message>".
    // Keep the message across the reload so it can be shown once on the next start.
    const unlisten = listen<string>("vault-locked", (event) => {
      const payload = typeof event.payload === "string" ? event.payload : "";
      if (payload.startsWith("restore-failed")) {
        const text = payload.replace(/^restore-failed:?\s*/, "");
        try {
          sessionStorage.setItem("notaya.notice", text || "The restore did not finish.");
        } catch {
          // Storage may be unavailable; the reload still happens.
        }
      }
      window.location.reload();
    });
    // Rust waits for every window to write its pending saves before it locks.
    const unlistenLocking = listen<number>("vault-locking", async (event) => {
      try {
        await flushAll();
      } finally {
        await api.flushDone(event.payload).catch(() => {});
      }
    });
    // The backend can also send a notice while the app is running.
    // The payload is ignored on purpose: taking the stored copy shows the message once and clears it.
    const unlistenNotice = listen<string>("app-notice", () => {
      showNotice();
    });
    // A notice set before the listener existed (while the window was reloading) would be missed: look once more now.
    unlistenNotice.then(() => showNotice());
    return () => {
      unlistenNotice.then((f) => f());
      unlisten.then((f) => f());
      unlistenLocking.then((f) => f());
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
