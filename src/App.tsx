import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { AnimatePresence, motion } from "motion/react";
import { toast, Toaster } from "sonner";
import { flushAll } from "@/lib/flush";
import { api } from "@/lib/ipc";
import { startLastPlacePersistence, startNavHistory, startNavKeys } from "@/lib/navHistory";
import { applyAppearance, cachedAppearance, isDark } from "@/lib/theme";
import { useApp } from "@/store/app";
import { Button, ConfirmProvider, TooltipProvider } from "@/components/ui";
import TitleBar from "@/components/TitleBar";
import UnlockScreen from "@/features/unlock/UnlockScreen";
import Shell from "@/components/Shell";
import { pollNotices } from "@/features/database/notices";
import { scheduleWindowReady, startupMark, watchStartupInitialization } from "@/lib/startup";

export default function App() {
  const status = useApp((s) => s.status);
  const refreshStatus = useApp((s) => s.refreshStatus);
  const theme = useApp((s) => s.settings?.theme ?? cachedAppearance().theme);
  const hasStatus = status !== null;
  const [lockListenerReady, setLockListenerReady] = useState(false);
  const [readinessFallback, setReadinessFallback] = useState(false);
  const [startupStatusFailed, setStartupStatusFailed] = useState(false);
  const readySent = useRef(false);

  // Reveal after React commits the status screen or a startup fallback.
  // Rust still decides visibility: --minimized stays hidden with a tray.
  useLayoutEffect(() => {
    startupMark("effectRan", true);
    if (readySent.current || (!readinessFallback && (!hasStatus || !lockListenerReady))) return;
    startupMark("screenCommittedAtMs");
    startupMark("microtaskScheduled", true);
    const cancel = scheduleWindowReady(() => {
      readySent.current = true;
      startupMark("microtaskFired", true);
      void api.windowReady();
    });
    return () => { startupMark("readyCleanup", true); cancel(); };
  }, [hasStatus, lockListenerReady, readinessFallback]);

  const unlocked = status?.unlocked ?? false;
  // Back and forward (Backspace, Alt+Left/Right, the mouse buttons): only while unlocked, in memory only.
  useEffect(() => {
    if (!unlocked) return;
    const stopHistory = startNavHistory();
    const stopKeys = startNavKeys();
    const stopLastPlace = startLastPlacePersistence();
    return () => {
      stopHistory();
      stopKeys();
      stopLastPlace();
    };
  }, [unlocked]);
  // Database reminders: shown inside Notaya, title only, once per run. Only while unlocked.
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
    startupMark("startupEffectRan", true);
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

    const dismissedNoticeKey = "notaya.dismissedAppNotice";
    let active = true;
    let submittedId: string | undefined;
    let dismissedId: string | null = null;
    let checking = Promise.resolve();
    try {
      dismissedId = sessionStorage.getItem(dismissedNoticeKey);
    } catch {
      // The backend still keeps undismissed notices pending across reloads.
    }
    const checkNotice = () => {
      checking = checking.then(async () => {
        if (!active || document.hidden || submittedId) return;
        const notice = await api.pendingNotice();
        if (!active || document.hidden || !notice) return;
        if (dismissedId === notice.id) {
          await api.acknowledgeNotice(notice.id);
          checkNotice();
          return;
        }
        submittedId = notice.id;
        const show = notice.text.startsWith("Notaya could not") ? toast.error : toast.info;
        show(notice.text, {
          id: notice.id,
          duration: Infinity,
          dismissible: false,
          action: {
            label: "Dismiss",
            onClick: (event) => {
              // A keyboard user can reach buttons behind the collapsed Sonner stack.
              const frontToast = event.currentTarget.closest(
                '[data-sonner-toast][data-front="true"][data-mounted="true"][data-removed="false"]',
              );
              if (!active || document.hidden || submittedId !== notice.id || !frontToast) {
                event.preventDefault();
                return;
              }
              submittedId = undefined;
              dismissedId = notice.id;
              try {
                // Replay the acknowledgement if the WebView reloads during the IPC call.
                sessionStorage.setItem(dismissedNoticeKey, notice.id);
              } catch {
                // The receipt still lives in this window until it reloads.
              }
              checkNotice();
            },
          },
        });
      }).catch(() => {});
    };
    // Register at startup, independently of window readiness and vault status.
    const unlistenNotice = listen("app-notice", checkNotice);
    void unlistenNotice.then(checkNotice, checkNotice);
    document.addEventListener("visibilitychange", checkNotice);

    // CSS is applied synchronously. Native backdrop IPC must not hold status
    // initialization hostage while the window is hidden.
    startupMark("appearanceStartedAtMs");
    void applyAppearance(cachedAppearance(), true).finally(() => startupMark("appearanceCompletedAtMs"));

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
    const stopStartup = watchStartupInitialization(
      refreshStatus,
      [unlisten, unlistenLocking],
      () => {
        startupMark("lockListenerReadyAtMs");
        setLockListenerReady(true);
      },
      () => setReadinessFallback(true),
      () => setStartupStatusFailed(true),
    );
    return () => {
      stopStartup();
      active = false;
      document.removeEventListener("visibilitychange", checkNotice);
      if (submittedId) toast.dismiss(submittedId);
      // Rejected registrations have no listener to remove. A registration
      // completing after cleanup still removes its listener when it arrives.
      void unlistenNotice.then((f) => f(), () => {});
      void unlisten.then((f) => f(), () => {});
      void unlistenLocking.then((f) => f(), () => {});
    };
  }, [refreshStatus]);

  async function retryStartupStatus() {
    if (hasStatus || !startupStatusFailed) return;
    setStartupStatusFailed(false);
    try {
      await refreshStatus();
    } catch {
      setStartupStatusFailed(true);
    }
  }

  return (
    <TooltipProvider>
      <ConfirmProvider>
        <div className="flex h-full flex-col">
          <TitleBar />
          <div className="relative min-h-0 flex-1">
            {status === null && (
              <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-bg p-4 text-center text-fg">
                {startupStatusFailed ? (
                  <>
                    <p role="alert">Could not read the vault status.</p>
                    <Button type="button" onClick={retryStartupStatus}>Try again</Button>
                  </>
                ) : (
                  <div role="status" className="flex flex-col items-center gap-3">
                    <span
                      aria-hidden="true"
                      className="h-7 w-7 rounded-full border-2 border-line border-t-accent motion-safe:animate-spin"
                    />
                    <span>Opening your vault…</span>
                  </div>
                )}
              </div>
            )}
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
