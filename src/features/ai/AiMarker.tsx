import { useEffect, useState } from "react";
import { Pause, Square } from "lucide-react";
import { aiErrorText, api } from "@/lib/ipc";
import { dismissAiRequest, useAiState } from "@/store/ai";
import { useApp } from "@/store/app";
export { useAiState } from "@/store/ai";
import PairingDialog from "./PairingDialog";
import ConfirmDialog from "./ConfirmDialog";

// Mount once in the unlocked TitleBar. This owns the
// window-wide request dialogs as well as the marker, including outside Settings.
export default function AiMarker() {
  const { state, refresh } = useAiState();
  const tree = useApp((s) => s.tree);
  const stickies = useApp((s) => s.stickies);
  const itemName = (id: string) => tree.notebooks.find((n) => n.id === id)?.name
    ?? tree.sections.find((n) => n.id === id)?.name
    ?? tree.pages.find((p) => p.id === id)?.title
    ?? tree.databases?.find((d) => d.id === id)?.title
    ?? (stickies.some((s) => s.id === id) ? "Sticky note" : id);
  const [now, setNow] = useState(Date.now);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, []);
  if (!state?.enabled) return null;
  const active = state.activity.filter((a) => now - a.lastActiveAt < 30_000);
  const changePause = async () => {
    if (busy) return;
    setBusy(true);
    setError(null);
    try {
      if (state.paused) await api.aiResume(); else await api.aiPause();
      await refresh();
    } catch (error) { setError(aiErrorText(error, "Could not change AI access. Try again.")); }
    finally { setBusy(false); }
  };
  const pairing = state.pairingRequests[0];
  const confirm = state.confirmRequests[0];
  return <>
    {(state.paused || active.length > 0) && <div className="flex min-w-0 items-center gap-2 text-xs" aria-label="AI activity">
      <span tabIndex={0} title={active.map((a) => `${a.name}: ${a.claims.length ? a.claims.map(itemName).join(", ") : "No items listed"}`).join("\n")}
        className="max-w-56 truncate text-muted">{state.paused ? "AI stopped" : `with ${active.map((a) => a.name).join(", ")}`}</span>
      <button disabled={busy} onClick={() => void changePause()}
        className="flex h-7 shrink-0 items-center gap-1 rounded-md px-2 text-muted hover:bg-surface-2 hover:text-fg disabled:opacity-45">
        {state.paused ? <Pause size={12} /> : <Square size={12} />}{state.paused ? "Resume AI" : "Stop"}
      </button>
      {error && <span role="alert" className="text-danger">{error}</span>}
    </div>}
    {!state.paused && (confirm ? <ConfirmDialog key={confirm.id} request={confirm} onAnswered={() => dismissAiRequest(confirm.id)} />
      : pairing ? <PairingDialog key={pairing.id} request={pairing} alreadyPaired={state.pairings.some((p) => p.name === pairing.name)} onAnswered={() => dismissAiRequest(pairing.id)} /> : null)}
  </>;
}
