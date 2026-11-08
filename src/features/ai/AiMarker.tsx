import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Pause, Square } from "lucide-react";
import { api, type AiState } from "@/lib/ipc";
import PairingDialog from "./PairingDialog";
import ConfirmDialog from "./ConfirmDialog";

// Subscribe before the first snapshot so a request arriving during mount is not lost.
// Events are invalidations, never permission grants; the window re-reads core state.
export function useAiState() {
  const [state, setState] = useState<AiState | null>(null);
  const [failed, setFailed] = useState(false);
  const refreshRef = useRef<() => Promise<void>>(async () => {});
  const refresh = useCallback(() => refreshRef.current(), []);
  useEffect(() => {
    let alive = true;
    let sequence = 0;
    const unlisteners: (() => void)[] = [];
    refreshRef.current = async () => {
      const request = ++sequence;
      try {
        const next = await api.aiGetState();
        if (alive && request === sequence) { setState(next); setFailed(false); }
      } catch {
        if (alive && request === sequence) { setState(null); setFailed(true); }
      }
    };
    void (async () => {
      try {
        for (const event of ["ai-state-changed", "ai-activity", "ai-pairing-request", "ai-confirm-request"]) {
          const unlisten = await listen(event, () => void refresh());
          if (!alive) { unlisten(); return; }
          unlisteners.push(unlisten);
        }
        await refresh();
      } catch {
        if (alive) setFailed(true);
      }
    })();
    return () => { alive = false; unlisteners.forEach((unlisten) => unlisten()); };
  }, [refresh]);
  return { state, failed, refresh };
}

// Mount once in the unlocked TitleBar after lane A is integrated. This owns the
// window-wide request dialogs as well as the marker, including outside Settings.
export default function AiMarker() {
  const { state, refresh } = useAiState();
  const [now, setNow] = useState(Date.now);
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, []);
  if (!state?.enabled) return null;
  const active = state.activity.filter((a) => now - a.lastActiveAt < 30_000);
  const changePause = async () => {
    if (busy) return;
    setBusy(true);
    setFailed(false);
    try {
      if (state.paused) await api.aiResume(); else await api.aiPause();
      await refresh();
    } catch { setFailed(true); }
    finally { setBusy(false); }
  };
  const pairing = state.pairingRequests[0];
  const confirm = state.confirmRequests[0];
  return <>
    {(state.paused || active.length > 0) && <div className="flex min-w-0 items-center gap-2 text-xs" aria-label="AI activity">
      <span tabIndex={0} title={active.map((a) => `${a.name}: ${a.claims.length ? a.claims.join(", ") : "No items claimed"}`).join("\n")}
        className="max-w-56 truncate text-muted">{state.paused ? "AI stopped" : `with ${active.map((a) => a.name).join(", ")}`}</span>
      <button disabled={busy} onClick={() => void changePause()}
        className="flex h-7 shrink-0 items-center gap-1 rounded-md px-2 text-muted hover:bg-surface-2 hover:text-fg disabled:opacity-45">
        {state.paused ? <Pause size={12} /> : <Square size={12} />}{state.paused ? "Resume AI" : "Stop"}
      </button>
      {failed && <span role="alert" className="text-danger">Could not change AI access. Try again.</span>}
    </div>}
    {!state.paused && (confirm ? <ConfirmDialog key={confirm.id} request={confirm} onAnswered={() => void refresh()} />
      : pairing ? <PairingDialog key={pairing.id} request={pairing} onAnswered={() => void refresh()} /> : null)}
  </>;
}
