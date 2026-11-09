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

// Only a failure fallback: successful startup still signals after the screen
// commits, without waiting for a timer or an animation frame.
export const STARTUP_READY_TIMEOUT_MS = 5000;

export function watchStartupInitialization(
  refreshStatus: () => Promise<unknown>,
  lockListeners: Promise<unknown>[],
  listenersReady: () => void,
  failed: () => void,
): () => void {
  let active = true;
  let finished = false;
  const fail = () => {
    if (!active || finished) return;
    finished = true;
    clearTimeout(timer);
    failed();
  };
  // One bound covers both status attempts and listener registration. A hung
  // request cannot leave the native window hidden indefinitely.
  const timer = setTimeout(fail, STARTUP_READY_TIMEOUT_MS);
  const loadStatus = async () => {
    try {
      await refreshStatus();
    } catch {
      if (active && !finished) await refreshStatus();
    }
  };
  const registered = Promise.all(lockListeners).then(() => {
    if (active) listenersReady();
  });
  void Promise.all([loadStatus(), registered]).then(() => {
    finished = true;
    clearTimeout(timer);
  }, fail);
  return () => {
    active = false;
    clearTimeout(timer);
  };
}
