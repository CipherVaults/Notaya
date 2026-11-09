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
