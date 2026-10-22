import { useEffect, useState } from "react";
import { Command } from "cmdk";
import { Dialog } from "radix-ui";
import {
  BookOpen,
  FilePlus2,
  FileText,
  KeyRound,
  Lock,
  NotebookPen,
  Palette,
  Settings,
  StickyNote,
} from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { SearchHit } from "@/lib/types";
import { useApp } from "@/store/app";

export default function CommandPalette() {
  const open = useApp((s) => s.paletteOpen);
  const setOpen = useApp((s) => s.setPaletteOpen);
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[]>([]);

  useEffect(() => {
    if (!open) {
      setQuery("");
      setHits([]);
    }
  }, [open]);

  useEffect(() => {
    if (!query.trim()) {
      setHits([]);
      return;
    }
    let cancelled = false;
    const t = setTimeout(() => {
      api.search(query).then((h) => !cancelled && setHits(h));
    }, 90);
    return () => {
      cancelled = true;
      clearTimeout(t);
    };
  }, [query]);

  const run = (fn: () => void | Promise<void>) => {
    setOpen(false);
    Promise.resolve(fn()).catch((e) => toast.error(errorText(e)));
  };

  const goToHit = (hit: SearchHit) =>
    run(async () => {
      const s = useApp.getState();
      if (hit.kind === "page") {
        const page = s.tree.pages.find((p) => p.id === hit.id);
        const section = s.tree.sections.find((x) => x.id === page?.sectionId);
        s.setView("notes");
        if (section) {
          useApp.setState({ activeNotebookId: section.notebookId, activeSectionId: section.id, showTrash: false });
        }
        s.openPage(hit.id);
      } else if (hit.kind === "notebook") {
        s.setView("notes");
        s.openNotebook(hit.id);
      } else if (hit.kind === "sticky") {
        await api.openSticky(hit.id);
      }
    });

  const icon = (kind: SearchHit["kind"]) =>
    ({
      page: <FileText size={16} />,
      sticky: <StickyNote size={16} />,
      notebook: <BookOpen size={16} />,
    })[kind];

  const itemClass =
    "flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm text-fg data-[selected=true]:bg-surface-2 data-[selected=true]:text-fg";

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-[80] bg-black/40 backdrop-blur-[2px]" />
        <Dialog.Content className="fixed left-1/2 top-[14vh] z-[80] w-[640px] max-w-[92vw] -translate-x-1/2 overflow-hidden rounded-xl border border-line bg-elevated shadow-float outline-none">
          <Dialog.Title className="sr-only">Command palette</Dialog.Title>
          <Dialog.Description className="sr-only">Search everything or run an action</Dialog.Description>
          <Command shouldFilter={false} loop>
            <Command.Input
              value={query}
              onValueChange={setQuery}
              autoFocus
              placeholder="Search notes and sticky notes, or type a command"
              className="h-14 w-full border-b border-line bg-transparent px-5 text-[0.95rem] text-fg outline-none placeholder:text-faint"
            />
            <Command.List className="max-h-[52vh] overflow-y-auto p-2">
              <Command.Empty className="px-4 py-8 text-center text-sm text-muted">No matches.</Command.Empty>

              {hits.length > 0 && (
                <Command.Group heading="Results" className="mb-1 [&_[cmdk-group-heading]]:px-3 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:text-faint">
                  {hits.map((h) => (
                    <Command.Item key={`${h.kind}-${h.id}`} value={`${h.kind}-${h.id}`} onSelect={() => goToHit(h)} className={itemClass}>
                      <span className="text-muted">{icon(h.kind)}</span>
                      <div className="min-w-0 flex-1">
                        <div className="truncate font-medium">{h.title || "Untitled"}</div>
                        {(h.subtitle || h.snippet) && (
                          <div className="truncate text-xs text-muted">{h.subtitle || h.snippet}</div>
                        )}
                      </div>
                      <span className="text-[0.7rem] uppercase tracking-wider text-faint">{h.kind}</span>
                    </Command.Item>
                  ))}
                </Command.Group>
              )}

              {!query.trim() && (
                <Command.Group heading="Actions" className="[&_[cmdk-group-heading]]:px-3 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:text-faint">
                  <Action icon={<FilePlus2 size={16} />} label="New page" onSelect={() => run(() => { const s = useApp.getState(); s.setView("notes"); s.setIntent("new-page"); })} className={itemClass} />
                  <Action icon={<StickyNote size={16} />} label="New sticky note" onSelect={() => run(async () => { await api.createSticky(true); useApp.getState().setStickies(await api.listStickies()); })} className={itemClass} />
                  <Action icon={<NotebookPen size={16} />} label="Go to Notes" onSelect={() => run(() => useApp.getState().setView("notes"))} className={itemClass} />
                  <Action icon={<KeyRound size={16} />} label="Passwords (KeePassXC)" onSelect={() => run(() => useApp.getState().setView("passwords"))} className={itemClass} />
                  <Action icon={<Palette size={16} />} label="Appearance & settings" onSelect={() => run(() => useApp.getState().setView("settings"))} className={itemClass} />
                  <Action icon={<Settings size={16} />} label="Back up vault now" onSelect={() => run(async () => { const p = await api.backupNow(); if (p) toast.success("Encrypted backup saved."); })} className={itemClass} />
                  <Action icon={<Lock size={16} />} label="Lock vault" hint="Ctrl L" onSelect={() => run(() => api.lockVault())} className={itemClass} />
                </Command.Group>
              )}
            </Command.List>
          </Command>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

function Action({
  icon,
  label,
  hint,
  onSelect,
  className,
}: {
  icon: React.ReactNode;
  label: string;
  hint?: string;
  onSelect: () => void;
  className: string;
}) {
  return (
    <Command.Item value={label} onSelect={onSelect} className={className}>
      <span className="text-muted">{icon}</span>
      <span className="flex-1">{label}</span>
      {hint && <span className="text-xs text-faint">{hint}</span>}
    </Command.Item>
  );
}
