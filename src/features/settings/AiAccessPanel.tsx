import { useRef, useState } from "react";
import { Bot } from "lucide-react";
import { Button, Field, Input, Modal } from "@/components/ui";
import { api, type AiLevel } from "@/lib/ipc";
import { aiErrorText } from "@/features/ai/contract";
import { useApp } from "@/store/app";
import { useAiState } from "@/features/ai/AiMarker";

const LEVELS: { level: AiLevel; label: string; sentence: string }[] = [
  { level: "look", label: "Look", sentence: "Read the notebooks you have not hidden, without changing them." },
  { level: "edit", label: "Edit", sentence: "Read, create and edit notes, and move items to the bin. Removing items still needs your confirmation." },
  { level: "full", label: "Full", sentence: "Also permanently delete items from the bin, with your confirmation." },
];

export default function AiAccessPanel() {
  const { state, failed, refresh } = useAiState();
  const notebooks = useApp((s) => s.tree.notebooks);
  const [busy, setBusy] = useState(false);
  const inFlight = useRef(false);
  const [error, setError] = useState<string | null>(null);
  const [enableOpen, setEnableOpen] = useState(false);
  const [word, setWord] = useState("");
  const change = async (action: () => Promise<void>) => {
    if (inFlight.current) return;
    inFlight.current = true;
    setBusy(true);
    setError(null);
    try {
      await action();
      setEnableOpen(false);
      setWord("");
      await refresh();
    } catch (error) { setError(aiErrorText(error)); }
    finally { inFlight.current = false; setBusy(false); }
  };
  return <section aria-labelledby="ai-access-title" className="mb-10">
    <h2 id="ai-access-title" className="mb-5 flex items-center gap-2 text-sm font-semibold"><Bot size={16} />AI access</h2>
    <p className="mb-5 text-sm text-muted">Let a paired AI work with your notes. You control its access here, and can stop it at any time.</p>
    {!state ? <div role="status" className="text-sm text-muted">
      {failed ? <>AI access could not be loaded. <Button size="sm" onClick={() => void refresh()}>Try again</Button></> : "Loading AI access…"}
    </div> : <fieldset disabled={busy} className="min-w-0 space-y-6 disabled:opacity-60">
      <div className="flex items-start justify-between gap-4">
        <div><p className="text-sm font-medium">AI access</p><p className="mt-1 text-xs text-muted">{state.enabled ? "On — only paired AIs can connect." : "Off — no AI can connect."}</p></div>
        <Toggle label="AI access" checked={state.enabled} onChange={() => {
          if (state.enabled) void change(() => api.aiSetEnabled(false));
          else { setWord(""); setError(null); setEnableOpen(true); }
        }} />
      </div>
      {state.paused && state.enabled && <div className="flex items-center justify-between gap-4 rounded-lg border border-line p-3">
        <span className="text-sm">AI access is stopped.</span><Button size="sm" onClick={() => void change(api.aiResume)}>Resume AI</Button>
      </div>}
      <div>
        <h3 className="mb-3 text-sm font-medium">Connected AIs</h3>
        {state.pairings.length === 0 && <p className="text-sm text-muted">No AIs paired yet. Each new pairing starts at Look.</p>}
        <div className="space-y-3">{state.pairings.map((pairing) => <div key={pairing.id} className="rounded-lg border border-line p-4" aria-label={`Access for ${pairing.name}`}>
          <div className="mb-3 flex items-center justify-between gap-4">
            <span className="break-words text-sm font-semibold"><bdi>{pairing.name}</bdi></span>
            <Button size="sm" variant="outline" aria-label={`Remove ${pairing.name}`} onClick={() => void change(() => api.aiRemovePairing(pairing.id))}>Remove</Button>
          </div>
          <fieldset className="space-y-2"><legend className="sr-only">Level for {pairing.name}</legend>
            {LEVELS.map((option) => <label key={option.level} className="flex cursor-pointer items-start gap-3 text-sm">
              <input type="radio" name={`ai-level-${pairing.id}`} value={option.level} checked={pairing.level === option.level}
                onChange={() => void change(() => api.aiSetLevel(pairing.id, option.level))} className="mt-1 accent-[var(--accent)]" />
              <span><span className="font-medium">{option.label}</span><span className="mt-0.5 block text-xs text-muted">{option.sentence}</span></span>
            </label>)}
          </fieldset>
        </div>)}</div>
      </div>
      <fieldset className="space-y-2"><legend className="mb-2 text-sm font-medium">Hide notebooks from the AI</legend>
        <p className="pb-1 text-xs text-muted">Checked notebooks are invisible to every AI, at every level.</p>
        {notebooks.length === 0 && <p className="text-sm text-muted">No notebooks to hide.</p>}
        {notebooks.map((notebook) => <label key={notebook.id} className="flex items-center gap-3 text-sm">
          <input type="checkbox" checked={state.hiddenNotebooks.includes(notebook.id)}
            onChange={(e) => {
              const next = e.target.checked ? [...state.hiddenNotebooks, notebook.id] : state.hiddenNotebooks.filter((id) => id !== notebook.id);
              void change(() => api.aiSetHidden(next));
            }} className="accent-[var(--accent)]" /><bdi>{notebook.name}</bdi>
        </label>)}
      </fieldset>
      <div className="flex items-start justify-between gap-4">
        <div><p className="text-sm font-medium">Ask before big changes</p>
          <p className="mt-1 text-xs text-muted">Ask when more than {state.bigChangeItems} items change, or for another large change. Deletes still ask when this is off.</p></div>
        <Toggle label="Ask before big changes" checked={state.bigChangeConfirm} onChange={() => void change(() => api.aiSetBigChange(!state.bigChangeConfirm))} />
      </div>
      <details className="rounded-lg border border-line p-4">
        <summary className="cursor-pointer text-sm font-medium">How to connect an AI</summary>
        <div className="mt-3 space-y-3 text-xs text-muted">
          <p>Turn on AI access, set up a local client with the bridge next to the app, then approve its connection in the Notaya window.</p>
          <p>Replace the folder in these examples with the folder containing Notaya and its companion program.</p>
          <p>Bridge command for your client's local server setup:</p>
          <pre className="selectable overflow-x-auto rounded-lg bg-surface p-3 text-fg" tabIndex={0}>{'"C:\\Your Notaya folder\\notaya-bridge.exe"'}</pre>
          <p>For a client that uses a local server configuration, adapt this entry:</p>
          <pre className="selectable overflow-x-auto rounded-lg bg-surface p-3 text-fg" tabIndex={0}>{JSON.stringify({ mcpServers: { notaya: { command: "C:\\Your Notaya folder\\notaya-bridge.exe", args: [] } } }, null, 2)}</pre>
          <p>Keep Notaya open and unlocked. The connection works only on this computer.</p>
        </div>
      </details>
    </fieldset>}
    {error && !enableOpen && <p role="alert" className="mt-3 text-sm text-danger">{error}</p>}
    <Modal open={enableOpen} onOpenChange={(open) => { if (!busy) { setEnableOpen(open); setWord(""); } }}
      title="Turn on AI access?" description="Paired AIs can read the notebooks you have not hidden. You choose whether each AI can edit or delete.">
      <Field label="Type ALLOW to turn on AI access"><Input aria-label="Type ALLOW" value={word} disabled={busy} onChange={(e) => setWord(e.target.value)} /></Field>
      {error && <p role="alert" className="mt-3 text-sm text-danger">{error}</p>}
      <div className="mt-6 flex justify-end gap-2"><Button disabled={busy} onClick={() => { setEnableOpen(false); setWord(""); }}>Cancel</Button>
        <Button variant="primary" disabled={busy || word !== "ALLOW"} onClick={() => void change(() => api.aiSetEnabled(true, word))}>Turn on AI access</Button></div>
    </Modal>
  </section>;
}

function Toggle({ label, checked, onChange }: { label: string; checked: boolean; onChange: () => void }) {
  return <button type="button" role="switch" aria-label={label} aria-checked={checked} onClick={onChange}
    className={`relative h-6 w-11 shrink-0 rounded-full border border-line transition-colors ${checked ? "bg-accent" : "bg-surface-2"}`}>
    <span className={`absolute top-[2px] h-[18px] w-[18px] rounded-full transition-transform ${checked ? "left-[2px] translate-x-[18px] bg-accent-fg" : "left-[2px] bg-muted"}`} />
  </button>;
}
