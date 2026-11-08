import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { Button, Modal } from "@/components/ui";
import { api, errorText, type HistoryMeta, type HistoryBody } from "@/lib/ipc";
import { flushAll } from "@/lib/flush";
import { useApp } from "@/store/app";

export default function HistoryDialog({ pageId, open, onOpenChange }: { pageId: string; open: boolean; onOpenChange: (open: boolean) => void }) {
  const [versions, setVersions] = useState<HistoryMeta[]>([]);
  const [selected, setSelected] = useState("");
  const [body, setBody] = useState<HistoryBody | null>(null);
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const generation = useRef(0);
  useEffect(() => {
    if (!open) return;
    let alive = true;
    setVersions([]); setSelected(""); setBody(null); setLoading(true);
    (async () => {
      await flushAll();
      await api.historySnapshot(pageId);
      const list = await api.historyList(pageId);
      if (alive) { setVersions(list); setSelected(list[0]?.versionId ?? ""); }
    })().catch((e) => { if (alive) toast.error(errorText(e)); }).finally(() => { if (alive) setLoading(false); });
    return () => { alive = false; };
  }, [open, pageId]);
  useEffect(() => {
    const current = ++generation.current;
    setBody(null);
    if (!open || !selected) return;
    api.historyGet(pageId, selected).then((body) => { if (generation.current === current) setBody(body); }, (e) => { if (generation.current === current) toast.error(errorText(e)); });
    return () => { generation.current++; };
  }, [open, pageId, selected]);
  const restore = async () => {
    if (busy || !body) return;
    setBusy(true);
    try {
      await flushAll();
      await api.historyRestore(pageId, selected);
      useApp.getState().setTree(await api.listNotes());
      toast.success("Version restored. The previous version is kept in history.");
      onOpenChange(false);
    } catch (e) { toast.error(errorText(e)); }
    finally { setBusy(false); }
  };
  return <Modal open={open} onOpenChange={(value) => { if (!busy) onOpenChange(value); }} title="Version history" description="Earlier versions stay in your vault until thinned or the page is deleted for good." width={820}>
    {loading ? <p className="text-sm text-muted" role="status">Loading versions…</p> : versions.length === 0 ? <p className="text-sm text-muted">No earlier versions yet.</p> : <div className="grid gap-5 sm:grid-cols-[220px_1fr]">
      <div className="max-h-[50vh] space-y-1 overflow-y-auto" aria-label="Versions">
        {versions.map((version) => <button key={version.versionId} onClick={() => setSelected(version.versionId)} aria-pressed={selected === version.versionId} className={`block w-full rounded-lg px-3 py-2 text-left text-xs hover:bg-surface-2 ${selected === version.versionId ? "bg-surface-2" : ""}`}>
          <span className="block">{new Date(version.at).toLocaleString()}</span>
          <span className="text-muted">{version.actor === "person" ? "you" : version.actor.startsWith("ai:") ? `with ${version.actor.slice(3)}` : "automation"} · {version.words} words{version.beforeAi ? " · before AI edit" : ""}</span>
        </button>)}
      </div>
      <div className="min-w-0">
        {body ? <><h3 className="mb-3 break-words font-semibold" dir="auto">{body.title || "Untitled"}</h3><div className="max-h-[40vh] overflow-y-auto whitespace-pre-wrap break-words text-sm" dir="auto" aria-label="Version preview">{body.text || "Empty page"}</div></> : <p role="status" className="text-sm text-muted">Loading version…</p>}
        <Button className="mt-5" disabled={!body || busy} onClick={() => void restore()}>Restore this version</Button>
      </div>
    </div>}
  </Modal>;
}
