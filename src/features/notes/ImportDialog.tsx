// "Import pages…": Markdown, HTML, Word and PDF files, a folder of them, or ZIP files (for example a
// Notion export) become one new notebook. The dialog only chooses the kind of source; the backend
// opens the file dialog, reads and converts the files, and saves everything in one step.

import { useEffect, useRef, useState } from "react";
import { CheckCircle2, Loader2, ShieldCheck } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { ImportSource, NotesImportSummary } from "@/lib/types";
import { useApp } from "@/store/app";
import { Button, Modal, Segmented } from "@/components/ui";

const SOURCES: { value: ImportSource; label: string }[] = [
  { value: "files", label: "Files" },
  { value: "folder", label: "A folder" },
];

const SOURCE_HINT: Record<ImportSource, string> = {
  files:
    "Choose one or more files in one go: Markdown (.md), HTML, Word (.docx), PDF, or ZIP files such as a Notion export. Different kinds can be mixed.",
  folder: "Choose a folder. Every supported file inside it is imported, and its subfolders become sections.",
};

const CARRIES = [
  "Headings, bold, italic, underline, strikethrough, links and lists, including to-do lists",
  "Quotes, code blocks, tables, equations and dividers",
  "Pictures that sit next to the pages or inside the ZIP file (they are stored encrypted in your notes)",
  "From Notion: sub-pages, links between pages, callouts, toggles, columns, page icons and covers",
];

const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;

export default function ImportDialog() {
  const open = useApp((s) => s.importOpen);
  const setOpen = useApp((s) => s.setImportOpen);
  const [source, setSource] = useState<ImportSource>("files");
  const [busy, setBusy] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  const [result, setResult] = useState<NotesImportSummary | null>(null);
  const cancelRequested = useRef(false);

  useEffect(() => {
    if (open) {
      setResult(null);
      setBusy(false);
      setCancelling(false);
    }
  }, [open]);

  const run = async () => {
    if (busy) return;
    setBusy(true);
    try {
      const done = await api.importNotes(source);
      if (!done) {
        // The file dialog was cancelled, or the Cancel button stopped the import (nothing was added).
        if (cancelling || cancelRequested.current) toast.message("Import cancelled. Nothing was added.");
        return;
      }
      const app = useApp.getState();
      app.setTree(await api.listNotes());
      app.setView("notes");
      app.openNotebook(done.notebookId);
      setResult(done);
    } catch (e) {
      toast.error(errorText(e));
    } finally {
      setBusy(false);
      setCancelling(false);
      cancelRequested.current = false;
    }
  };

  // Asks the backend to stop. It finishes what it is doing within moments, takes back anything it
  // stored, and answers the import call with "nothing".
  const cancel = async () => {
    if (!busy || cancelRequested.current) return;
    cancelRequested.current = true;
    setCancelling(true);
    try {
      await api.importCancel();
    } catch (e) {
      toast.error(errorText(e));
    }
  };

  return (
    <Modal open={open} onOpenChange={(o) => !o && !busy && setOpen(false)} title={result ? "Import finished" : "Import pages"} width={520}>
      {result ? (
        <div className="space-y-4">
          <div className="flex items-start gap-3">
            <CheckCircle2 size={20} className="mt-0.5 shrink-0 text-accent" aria-hidden />
            <div className="text-sm">
              <div className="font-medium">
                {result.notebooks.length > 1
                  ? `${result.notebooks.length} new notebooks were added: ${result.notebooks.map((n) => `"${n.name}"`).join(", ")}.`
                  : `A new notebook "${result.notebookName}" was added.`}
              </div>
              <div className="mt-0.5 text-muted">
                {plural(result.pages, "page")} in {plural(result.sections, "section")}
                {result.images > 0 && `, ${plural(result.images, "picture")}`}. Your other notes were not changed.
              </div>
            </div>
          </div>
          {result.notes.length > 0 && (
            <ul className="list-disc space-y-1 pl-5 text-sm text-muted">
              {result.notes.map((n) => (
                <li key={n}>{n}</li>
              ))}
            </ul>
          )}
          {result.skipped.length > 0 && (
            <div>
              <div className="mb-1.5 text-xs font-medium uppercase tracking-wider text-faint">Left out ({result.skipped.length})</div>
              <ul className="max-h-48 space-y-1.5 overflow-y-auto rounded-lg border border-line p-2.5 text-sm" aria-label="Items that were not imported">
                {result.skipped.map((s, i) => (
                  <li key={i}>
                    <div className="break-all font-medium">{s.item}</div>
                    <div className="text-muted">{s.reason}</div>
                  </li>
                ))}
              </ul>
            </div>
          )}
          <div className="flex justify-end pt-1">
            <Button type="button" variant="primary" onClick={() => setOpen(false)}>
              Done
            </Button>
          </div>
        </div>
      ) : (
        <div className="space-y-5">
          <div className="space-y-1.5">
            <div id="import-source" className="text-xs font-medium uppercase tracking-wider text-faint">
              Import from
            </div>
            <div role="group" aria-labelledby="import-source">
              <Segmented value={source} options={SOURCES} onChange={setSource} />
            </div>
            <p className="text-xs text-faint">{SOURCE_HINT[source]}</p>
          </div>

          <div>
            <div className="mb-1.5 text-xs font-medium uppercase tracking-wider text-faint">What carries over</div>
            <ul className="list-disc space-y-1 pl-5 text-sm">
              {CARRIES.map((c) => (
                <li key={c}>{c}</li>
              ))}
            </ul>
          </div>

          <div className="space-y-2 rounded-lg border border-line bg-surface-2/50 px-3 py-2.5 text-sm text-muted">
            <p>
              <span className="font-medium text-fg">Notion:</span> export your pages as <span className="font-medium text-fg">HTML</span> (Export, then Format: HTML)
              to keep colors, highlights and the page font. The Markdown export drops them.
            </p>
            <p>
              <span className="font-medium text-fg">PDF:</span> only the text is imported, not scanned pages, pictures or the layout.
            </p>
            <p>Databases (tables), comments and page history are not imported.</p>
          </div>

          <div role="note" className="flex items-start gap-2.5 text-sm text-muted">
            <ShieldCheck size={16} className="mt-0.5 shrink-0" aria-hidden />
            <span>Everything is read on this computer. Nothing is sent over the internet, and pictures on the web stay links. The pages go into one new notebook; your existing notes are not changed.</span>
          </div>

          <div className="flex justify-end gap-2 pt-1">
            {busy ? (
              <Button type="button" variant="ghost" onClick={cancel} disabled={cancelling}>
                {cancelling ? "Cancelling…" : "Cancel import"}
              </Button>
            ) : (
              <Button type="button" variant="ghost" onClick={() => setOpen(false)}>
                Cancel
              </Button>
            )}
            <Button type="button" variant="primary" onClick={run} disabled={busy}>
              {busy && <Loader2 size={15} className="animate-spin" />}
              {busy ? "Importing…" : source === "folder" ? "Choose folder…" : "Choose files…"}
            </Button>
          </div>
        </div>
      )}
    </Modal>
  );
}
