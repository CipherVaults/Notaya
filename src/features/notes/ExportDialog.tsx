// "Export…": a page, chosen pages, one or several sections, one or several notebooks, as HTML,
// Markdown, PDF or Word, in one file or as separate files in a ZIP. The dialog only chooses; the
// backend asks where to save and writes the (unencrypted) file.

import { useEffect, useMemo, useState } from "react";
import { Loader2, ShieldAlert } from "lucide-react";
import { toast } from "sonner";
import { flushAll } from "@/lib/flush";
import { api, errorText } from "@/lib/ipc";
import type { ExportFormat, ExportRequest, PageMeta } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useApp, type ExportTarget } from "@/store/app";
import { Button, Modal, Segmented, Switch } from "@/components/ui";

type Choice = "page" | "pages" | "section" | "sections" | "notebook" | "notebooks";
type Output = "single" | "zip";

/** Opens the export dialog for a page, a section or a notebook. */
export function openExport(kind: "page" | "section" | "notebook", id: string) {
  const { tree, setExportTarget } = useApp.getState();
  let target: ExportTarget | null = null;
  if (kind === "page") {
    const page = tree.pages.find((p) => p.id === id);
    const section = tree.sections.find((s) => s.id === page?.sectionId);
    if (page && section) target = { notebookId: section.notebookId, sectionId: section.id, pageId: page.id, initial: "page" };
  } else if (kind === "section") {
    const section = tree.sections.find((s) => s.id === id);
    if (section) target = { notebookId: section.notebookId, sectionId: section.id, initial: "section" };
  } else if (tree.notebooks.some((n) => n.id === id)) {
    target = { notebookId: id, initial: "notebook" };
  }
  if (target) setExportTarget(target);
}

const FORMATS: { value: ExportFormat; label: string }[] = [
  { value: "html", label: "HTML" },
  { value: "markdown", label: "Markdown" },
  { value: "pdf", label: "PDF" },
  { value: "docx", label: "Word" },
];

const OUTPUTS: { value: Output; label: string }[] = [
  { value: "single", label: "One file" },
  { value: "zip", label: "Separate files in a ZIP" },
];

const OUTPUT_HINT: Record<Output, string> = {
  single: "Everything you choose goes into one document.",
  zip: "One file per page, in folders: notebook, then section, then page. Sub-pages go in a folder named like their page. Markdown pictures sit next to each page in a folder ending in .assets.",
};

const FORMAT_HINT: Record<ExportFormat, string> = {
  html: "An .html page that opens in any browser. Pictures are inside the file; it has no scripts and loads nothing from the internet.",
  markdown: "A .md file. Pictures are saved in a folder next to it, named after the file with .assets at the end.",
  pdf: "A .pdf file. It prints Western European, Arabic, Hebrew, Chinese, Japanese and Korean letters (the last three use fonts Windows already has). Other scripts (for example Cyrillic) and emoji are replaced. Choose Word to keep them.",
  docx: "A .docx file for Microsoft Word, LibreOffice or Google Docs. Equations are kept as their TeX text.",
};

const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;

/** The notebook's pages (not in the trash) in sidebar order, each with its depth. */
function notebookPages(pages: PageMeta[], sectionIds: string[]): { page: PageMeta; depth: number; sectionId: string }[] {
  const out: { page: PageMeta; depth: number; sectionId: string }[] = [];
  for (const sid of sectionIds) {
    const here = pages.filter((p) => p.sectionId === sid && p.trashedAt === null);
    const known = new Set(here.map((p) => p.id));
    const walk = (p: PageMeta, depth: number) => {
      out.push({ page: p, depth, sectionId: sid });
      here.filter((c) => c.parent === p.id).forEach((c) => walk(c, depth + 1));
    };
    here.filter((p) => !p.parent || !known.has(p.parent)).forEach((p) => walk(p, 0));
  }
  return out;
}

export default function ExportDialog() {
  const target = useApp((s) => s.exportTarget);
  const setTarget = useApp((s) => s.setExportTarget);
  const tree = useApp((s) => s.tree);
  const securityOn = useApp((s) => s.status?.securityOn ?? false);
  const [choice, setChoice] = useState<Choice>("page");
  const [format, setFormat] = useState<ExportFormat>("pdf");
  const [output, setOutput] = useState<Output>("single");
  const [subpages, setSubpages] = useState(true);
  const [chosen, setChosen] = useState<Set<string>>(new Set());
  const [chosenSections, setChosenSections] = useState<Set<string>>(new Set());
  const [chosenNotebooks, setChosenNotebooks] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);

  const notebook = tree.notebooks.find((n) => n.id === target?.notebookId);
  const section = tree.sections.find((s) => s.id === target?.sectionId);
  const page = tree.pages.find((p) => p.id === target?.pageId);
  const sections = useMemo(() => tree.sections.filter((s) => s.notebookId === target?.notebookId), [tree.sections, target?.notebookId]);
  const list = useMemo(() => notebookPages(tree.pages, sections.map((s) => s.id)), [tree.pages, sections]);

  useEffect(() => {
    if (!target) return;
    setChoice(target.initial);
    setChosen(new Set(target.pageId ? [target.pageId] : []));
    setChosenSections(new Set(target.sectionId ? [target.sectionId] : []));
    setChosenNotebooks(new Set([target.notebookId]));
    setBusy(false);
  }, [target]);

  const request = (): ExportRequest | null => {
    if (!target) return null;
    const base = { format, includeSubpages: subpages, output };
    switch (choice) {
      case "page":
        return page ? { ...base, scope: "pages", ids: [page.id] } : null;
      case "pages":
        return chosen.size ? { ...base, scope: "pages", ids: list.map((x) => x.page.id).filter((id) => chosen.has(id)) } : null;
      case "section":
        return section ? { ...base, scope: "section", ids: [section.id] } : null;
      case "sections":
        return chosenSections.size ? { ...base, scope: "sections", ids: sections.map((x) => x.id).filter((id) => chosenSections.has(id)) } : null;
      case "notebook":
        return notebook ? { ...base, scope: "notebook", ids: [notebook.id] } : null;
      case "notebooks":
        return chosenNotebooks.size ? { ...base, scope: "notebooks", ids: tree.notebooks.map((x) => x.id).filter((id) => chosenNotebooks.has(id)) } : null;
    }
  };

  const run = async () => {
    const req = request();
    if (!req || busy) return;
    setBusy(true);
    try {
      // Pending edits are saved first, so the file has the latest text.
      await flushAll();
      const done = await api.exportNotes(req);
      if (!done) return; // the save dialog was cancelled
      setTarget(null);
      toast.success(done.archive ? `Exported ${plural(done.pages, "page")} as ${plural(done.files, "file")} in a ZIP.` : `Exported ${plural(done.pages, "page")}.`, {
        description: done.path,
      });
      const notes: string[] = [];
      if (done.imagesLeftOut) notes.push(`${plural(done.imagesLeftOut, "picture")} (GIF or WebP) could not be put in the PDF.`);
      if (done.missingImages) notes.push(`${plural(done.missingImages, "picture")} could not be read and ${done.missingImages === 1 ? "was" : "were"} left out.`);
      if (done.charsReplaced) notes.push("Some characters are not available in PDF and were replaced. Export to Word to keep them.");
      if (notes.length) toast.warning(notes.join(" "));
    } catch (e) {
      toast.error(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  const options: { value: Choice; label: string; detail?: string }[] = [];
  if (page) options.push({ value: "page", label: "This page", detail: page.title || "Untitled" });
  if (notebook) options.push({ value: "pages", label: "Choose pages", detail: choice === "pages" ? `${chosen.size} chosen` : undefined });
  if (section) options.push({ value: "section", label: "This section", detail: section.name });
  if (notebook && sections.length > 1) options.push({ value: "sections", label: "Choose sections", detail: choice === "sections" ? `${chosenSections.size} chosen` : undefined });
  if (notebook) options.push({ value: "notebook", label: "Whole notebook", detail: notebook.name });
  if (notebook && tree.notebooks.length > 1) options.push({ value: "notebooks", label: "Choose notebooks", detail: choice === "notebooks" ? `${chosenNotebooks.size} chosen` : undefined });

  const flip = (set: Set<string>, setSet: (s: Set<string>) => void, id: string) => {
    const next = new Set(set);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setSet(next);
  };
  const toggle = (id: string) => flip(chosen, setChosen, id);
  const warning =
    output === "zip"
      ? `The ZIP and the files in it are not encrypted (the ZIP has no password)${securityOn ? " and not protected by your master password" : ""}. Anyone who gets them can read these notes.`
      : `The exported file is not encrypted${securityOn ? " and not protected by your master password" : ""}. Anyone who gets it can read these notes.`;
  const pageCount = (sectionId: string) => tree.pages.filter((p) => p.sectionId === sectionId && p.trashedAt === null).length;
  const notebookPageCount = (nbId: string) => tree.sections.filter((x) => x.notebookId === nbId).reduce((n, x) => n + pageCount(x.id), 0);

  return (
    <Modal open={!!target} onOpenChange={(o) => !o && !busy && setTarget(null)} title="Export" width={500}>
      <div className="space-y-5">
        <fieldset className="space-y-1.5">
          <legend className="mb-1.5 text-xs font-medium uppercase tracking-wider text-faint">What to export</legend>
          <div className="grid gap-1">
            {options.map((o) => (
              <label
                key={o.value}
                className={cn(
                  "flex cursor-pointer items-center gap-3 rounded-lg border px-3 py-2 text-sm transition-colors",
                  choice === o.value ? "border-accent bg-accent-soft" : "border-line hover:bg-surface-2",
                )}
              >
                <input type="radio" name="export-scope" value={o.value} checked={choice === o.value} onChange={() => setChoice(o.value)} className="accent-accent" />
                <span className="font-medium">{o.label}</span>
                {o.detail && <span className="min-w-0 flex-1 truncate text-right text-muted">{o.detail}</span>}
              </label>
            ))}
          </div>
          {choice === "pages" && (
            <div role="group" aria-label="Pages to export" className="mt-2 max-h-48 overflow-y-auto rounded-lg border border-line p-1.5">
              {list.length === 0 && <div className="px-2 py-1.5 text-sm text-muted">This notebook has no pages.</div>}
              {sections.map((s) => {
                const rows = list.filter((x) => x.sectionId === s.id);
                if (!rows.length) return null;
                return (
                  <div key={s.id}>
                    <div className="px-2 pb-0.5 pt-1.5 text-xs text-faint">{s.name}</div>
                    {rows.map(({ page: p, depth }) => (
                      <label key={p.id} className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1 text-sm hover:bg-surface-2" style={{ paddingLeft: 8 + depth * 16 }}>
                        <input type="checkbox" checked={chosen.has(p.id)} onChange={() => toggle(p.id)} className="accent-accent" />
                        {p.icon && <span className="emoji-mono">{p.icon}</span>}
                        <span dir="auto" className="truncate">{p.title || "Untitled"}</span>
                      </label>
                    ))}
                  </div>
                );
              })}
            </div>
          )}
          {choice === "sections" && (
            <div role="group" aria-label="Sections to export" className="mt-2 max-h-48 overflow-y-auto rounded-lg border border-line p-1.5">
              {sections.map((x) => (
                <label key={x.id} className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1 text-sm hover:bg-surface-2">
                  <input type="checkbox" checked={chosenSections.has(x.id)} onChange={() => flip(chosenSections, setChosenSections, x.id)} className="accent-accent" />
                  <span className="min-w-0 flex-1 truncate">{x.name || "Untitled"}</span>
                  <span className="text-xs text-faint">{plural(pageCount(x.id), "page")}</span>
                </label>
              ))}
            </div>
          )}
          {choice === "notebooks" && (
            <div role="group" aria-label="Notebooks to export" className="mt-2 max-h-48 overflow-y-auto rounded-lg border border-line p-1.5">
              {tree.notebooks.map((x) => (
                <label key={x.id} className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1 text-sm hover:bg-surface-2">
                  <input type="checkbox" checked={chosenNotebooks.has(x.id)} onChange={() => flip(chosenNotebooks, setChosenNotebooks, x.id)} className="accent-accent" />
                  {x.icon && <span className="emoji-mono">{x.icon}</span>}
                  <span className="min-w-0 flex-1 truncate">{x.name || "Untitled"}</span>
                  <span className="text-xs text-faint">{plural(notebookPageCount(x.id), "page")}</span>
                </label>
              ))}
            </div>
          )}
        </fieldset>

        <div className="space-y-1.5">
          <div id="export-format" className="text-xs font-medium uppercase tracking-wider text-faint">
            Format
          </div>
          <div role="group" aria-labelledby="export-format">
            <Segmented value={format} options={FORMATS} onChange={setFormat} />
          </div>
          <p className="text-xs text-faint">{FORMAT_HINT[format]}</p>
        </div>

        <div className="space-y-1.5">
          <div id="export-output" className="text-xs font-medium uppercase tracking-wider text-faint">
            Output
          </div>
          <div role="group" aria-labelledby="export-output">
            <Segmented value={output} options={OUTPUTS} onChange={setOutput} />
          </div>
          <p className="text-xs text-faint">{OUTPUT_HINT[output]}</p>
        </div>

        <div className="flex items-center justify-between gap-4">
          <div>
            <div className="text-sm font-medium">Include sub-pages</div>
            <div className="text-xs text-faint">Pages inside the chosen pages follow them. Pages in the trash are left out.</div>
          </div>
          <Switch checked={subpages} onChange={setSubpages} label="Include sub-pages" />
        </div>

        <div role="note" className="flex items-start gap-2.5 rounded-lg border border-warn/30 bg-warn/10 px-3 py-2.5 text-sm">
          <ShieldAlert size={16} className="mt-0.5 shrink-0 text-warn" aria-hidden />
          <span>{warning}</span>
        </div>

        <div className="flex justify-end gap-2 pt-1">
          <Button type="button" variant="ghost" onClick={() => setTarget(null)} disabled={busy}>
            Cancel
          </Button>
          <Button type="button" variant="primary" onClick={run} disabled={busy || !request()}>
            {busy && <Loader2 size={15} className="animate-spin" />}
            {busy ? "Exporting…" : "Export…"}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
