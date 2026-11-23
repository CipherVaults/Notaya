// Opt-in, content-free diagnostics for the throwaway real-exe harness. No probe
// is created, persisted or logged by the app during an ordinary start.
type StartupProbe = { startedAtMs: number; [key: string]: unknown };

export function startupMark(stage: string, count = false): void {
  const probe = (window as Window & { __notayaStartupProbe?: StartupProbe }).__notayaStartupProbe;
  if (!probe) return;
  probe[stage] = count ? Number(probe[stage] ?? 0) + 1 : performance.now() - probe.startedAtMs;
}

// Run after the status screen commits, without a paint or timer dependency.
// Deferral permits StrictMode cleanup to cancel the first effect's callback.
export function scheduleWindowReady(ready: () => void): () => void {
  let done = false;
  queueMicrotask(() => {
    if (done) return;
    done = true;
    ready();
  });
  return () => { done = true; };
}

// The watchdog reveals the window if initialization stalls. Successful startup
// still signals after the screen commits, without waiting for the watchdog.
export const STARTUP_READY_TIMEOUT_MS = 5000;

export function watchStartupInitialization(
  refreshStatus: () => Promise<unknown>,
  lockListeners: Promise<unknown>[],
  listenersReady: () => void,
  revealFallback: () => void,
  statusFailed: () => void,
): () => void {
  let active = true;
  let fallbackSent = false;
  let initialRejected = false;
  let statusPending = true;
  let statusFailureReported = false;
  const reportStatusFailure = () => {
    if (!active || statusFailureReported) return;
    statusFailureReported = true;
    statusFailed();
  };
  const reveal = () => {
    if (!active) return;
    // A listener failure can reveal the window before a pending retry expires.
    if (!initialRejected || !statusPending) clearTimeout(timer);
    if (fallbackSent) return;
    fallbackSent = true;
    revealFallback();
  };
  // One deadline covers both status attempts and listener registration.
  // A slow first request can still finish while the loading screen is visible.
  const timer = setTimeout(() => {
    if (initialRejected && statusPending) reportStatusFailure();
    reveal();
  }, STARTUP_READY_TIMEOUT_MS);
  const loadStatus = async () => {
    try {
      await refreshStatus();
    } catch (error) {
      initialRejected = true;
      // Keep the automatic retry within the original readiness window.
      if (!active || fallbackSent) throw error;
      await refreshStatus();
    }
  };
  const statusLoaded = loadStatus().then(
    () => {
      statusPending = false;
      if (fallbackSent) clearTimeout(timer);
    },
    () => {
      statusPending = false;
      reportStatusFailure();
      reveal();
    },
  );
  const registered = Promise.all(lockListeners).then(() => {
    if (active) listenersReady();
  });
  void Promise.all([statusLoaded, registered]).then(() => clearTimeout(timer), reveal);
  return () => {
    active = false;
    clearTimeout(timer);
  };
}
