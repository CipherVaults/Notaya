import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import { Button, Input, Modal } from "@/components/ui";
import { api, errorText, type ActivityEntry } from "@/lib/ipc";
import { flushAll } from "@/lib/flush";
import { useApp } from "@/store/app";

export default function ActivityPanel() {
  const [entries, setEntries] = useState<ActivityEntry[]>([]);
  const [who, setWho] = useState("");
  const [busy, setBusy] = useState(false);
  const [clearOpen, setClearOpen] = useState(false);
  const [confirmation, setConfirmation] = useState("");
  const [checked, setChecked] = useState("");
  const [hasMore, setHasMore] = useState(false);
  const [undo, setUndo] = useState<ActivityEntry | null>(null);
  const securityOn = useApp((s) => s.status?.securityOn ?? false);
  const load = useCallback(async () => {
    const list = await api.activityList();
    setChecked(""); setEntries(list); setHasMore(list.length === 100);
  }, []);
  useEffect(() => { let alive = true; api.activityList().then((list) => { if (alive) { setEntries(list); setHasMore(list.length === 100); } }, (e) => { if (alive) toast.error(errorText(e)); }); return () => { alive = false; }; }, []);
  const run = async (work: () => Promise<void>) => {
    if (busy) return;
    setBusy(true);
    try { await work(); } catch (e) { toast.error(errorText(e)); } finally { setBusy(false); }
  };
  const people = [...new Set(entries.map((entry) => entry.actor))];
  return <div className="space-y-4">
    <p className="text-xs text-muted">Who changed what and when. Stored in your vault, separate from the error log.</p>
    {!securityOn && <p className="text-xs text-muted">Security is off. Someone with the vault file can rewrite this log consistently.</p>}
    <div className="flex flex-wrap items-center gap-2">
      <label className="text-xs">Who <select aria-label="Filter activity by who" value={who} onChange={(e) => setWho(e.target.value)} className="ml-2 rounded-lg border border-line bg-surface px-2 py-1.5 text-sm"><option value="">Everyone</option>{people.map((actor) => <option key={actor} value={actor}>{actor === "person" ? "You" : actor}</option>)}</select></label>
      <Button size="sm" disabled={busy} onClick={() => void run(async () => { setChecked(""); await api.activityVerify(); setChecked("The log checks out."); })}>Check the log</Button>
      <Button size="sm" disabled={busy} onClick={() => void run(load)}>Refresh</Button>
      <Button size="sm" variant="ghost" disabled={busy} onClick={() => { setConfirmation(""); setClearOpen(true); }}>Clear activity log</Button>
    </div>
    {checked && <p role="status" className="text-xs text-muted">{checked}</p>}
    <div className="max-h-96 space-y-2 overflow-y-auto" aria-label="Activity entries">
      {entries.filter((entry) => !who || entry.actor === who).map((entry) => <div key={entry.seq} className="rounded-lg border border-line px-3 py-2 text-xs">
        <div className="flex items-start justify-between gap-3"><span>{entry.actor === "person" ? "You" : entry.actor} · {new Date(entry.at).toLocaleString()}</span>{entry.undoable && <Button size="sm" disabled={busy} onClick={() => setUndo(entry)}>Undo this change</Button>}</div>
        <p className="mt-1 text-muted">{entry.action === "log.cleared" ? "Log cleared" : entry.action === "log.pruned" ? "Older activity thinned" : entry.action === "ai.reads" ? `${entry.counts.reads ?? 0} reads` : entry.items.map((item) => item.titleAtTheTime || item.kind).join(", ")}</p>
        <p className="mt-1 text-faint">{Object.entries(entry.counts).map(([kind, count]) => `${count} ${kind}`).join(" · ")}</p>
      </div>)}
      {entries.length === 0 && <p className="text-xs text-muted">No activity yet.</p>}
    </div>
    {hasMore && <Button size="sm" disabled={busy} onClick={() => void run(async () => { const next = await api.activityList(undefined, entries[entries.length - 1]?.seq); setEntries((current) => [...current, ...next]); setHasMore(next.length === 100); })}>Load older activity</Button>}
    <Modal open={clearOpen} onOpenChange={setClearOpen} title="Clear activity log" description="This removes activity and its undo records. Page version history stays. Type CLEAR to continue.">
      <Input aria-label="Type CLEAR" value={confirmation} onChange={(e) => setConfirmation(e.target.value)} />
      <Button className="mt-4" variant="danger" disabled={confirmation !== "CLEAR" || busy} onClick={() => void run(async () => { await api.activityClear(confirmation); setWho(""); setChecked(""); setClearOpen(false); await load(); })}>Clear log</Button>
    </Modal>
    <Modal open={undo !== null} onOpenChange={(open) => { if (!open) setUndo(null); }} title="Undo this change" description="Items you changed afterwards are skipped. New items go to the bin.">
      <Button disabled={busy} onClick={() => void run(async () => { if (!undo) return; await flushAll(); const result = await api.activityUndo(undo.changeSet); setUndo(null); setChecked(""); toast.success(`${result.restored} restored, ${result.skipped} skipped.`); useApp.getState().setTree(await api.listNotes()); await load(); })}>Undo change set</Button>
    </Modal>
  </div>;
}
