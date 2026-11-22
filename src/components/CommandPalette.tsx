import { useEffect, useState } from "react";
import { Command } from "cmdk";
import { Dialog } from "radix-ui";
import {
  BookOpen,
  Download,
  FilePlus2,
  FileText,
  Lock,
  NotebookPen,
  Palette,
  Settings,
  StickyNote,
  Table2,
  Upload,
} from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { SearchHit } from "@/lib/types";
import { useApp } from "@/store/app";
import { PageIcon } from "@/lib/pageLook";
import { openExport } from "@/features/notes/ExportDialog";
import { RecentSearches, useRecentSearches } from "@/features/search/RecentSearches";

type PaletteAction = {
  id: string;
  label: string;
  icon: React.ReactNode;
  hint?: string;
  requiresSecurity?: boolean;
  execute: () => void | Promise<void>;
};

function commandText(text: string) {
  return text.trim().toLowerCase().replace(/(?:…|\.\.\.)$/, "").trimEnd();
}

export default function CommandPalette() {
  const open = useApp((s) => s.paletteOpen);
  const securityOn = useApp((s) => s.status?.securityOn ?? false);
  const setOpen = useApp((s) => s.setPaletteOpen);
  const tree = useApp((s) => s.tree);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<{ query: string; hits: SearchHit[]; failed?: boolean } | null>(null);
  const currentResults = results?.query === query ? results : null;
  const hits = currentResults?.hits ?? [];
  const hasQuery = query.trim().length > 0;
  const searches = useRecentSearches(open);

  useEffect(() => {
    if (!open) {
      setQuery("");
      setResults(null);
    }
  }, [open]);

  useEffect(() => {
    if (!open || !hasQuery) return;
    let cancelled = false;
    const t = setTimeout(() => {
      api.search(query).then(
        (hits) => {
          if (!cancelled) setResults({ query, hits });
        },
        () => {
          if (!cancelled) setResults({ query, hits: [], failed: true });
        },
      );
    }, 90);
    return () => {
      cancelled = true;
      clearTimeout(t);
    };
  }, [open, hasQuery, query]);

  const run = (fn: () => void | Promise<void>) => {
    setOpen(false);
    Promise.resolve(fn()).catch((e) => toast.error(errorText(e)));
  };

  const goToHit = (hit: SearchHit) => {
    searches.remember(query);
    return goToHitNow(hit);
  };
  const goToHitNow = (hit: SearchHit) =>
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
      } else if (hit.kind === "database") {
        const db = s.tree.databases.find((d) => d.id === hit.id);
        const section = s.tree.sections.find((x) => x.id === db?.sectionId);
        if (section) useApp.setState({ activeNotebookId: section.notebookId, activeSectionId: section.id });
        s.openDatabase(hit.id);
      } else if (hit.kind === "row") {
        s.openRow(hit.id);
      }
    });

  const icon = (h: SearchHit) => {
    // A page (or database) with an icon of its own shows it here.
    const own = h.kind === "page" ? tree.pages.find((x) => x.id === h.id)?.icon : h.kind === "database" ? tree.databases.find((x) => x.id === h.id)?.icon : "";
    if (own) return <PageIcon icon={own} size={16} />;
    return {
      page: <FileText size={16} />,
      sticky: <StickyNote size={16} />,
      notebook: <BookOpen size={16} />,
      database: <Table2 size={16} />,
      row: <FileText size={16} />,
    }[h.kind];
  };

  const actions: PaletteAction[] = [
    {
      id: "new-page",
      label: "New page",
      icon: <FilePlus2 size={16} />,
      execute: () => {
        const s = useApp.getState();
        s.setView("notes");
        s.setIntent("new-page");
      },
    },
    {
      id: "new-sticky",
      label: "New sticky note",
      icon: <StickyNote size={16} />,
      execute: async () => {
        await api.createSticky(true);
        useApp.getState().setStickies(await api.listStickies());
      },
    },
    {
      id: "notes",
      label: "Go to Notes",
      icon: <NotebookPen size={16} />,
      execute: () => useApp.getState().setView("notes"),
    },
    {
      id: "settings",
      label: "Appearance & settings",
      icon: <Palette size={16} />,
      execute: () => useApp.getState().setView("settings"),
    },
    {
      id: "export",
      label: "Export…",
      icon: <Download size={16} />,
      execute: () => {
        const s = useApp.getState();
        if (s.activePageId) openExport("page", s.activePageId);
        else if (s.activeNotebookId) openExport("notebook", s.activeNotebookId);
        else toast("Open a page or a notebook to export it.");
      },
    },
    {
      id: "import",
      label: "Import pages…",
      icon: <Upload size={16} />,
      execute: () => {
        const s = useApp.getState();
        s.setView("notes");
        s.setImportOpen(true);
      },
    },
    {
      id: "backup",
      label: "Back up vault now",
      icon: <Settings size={16} />,
      execute: async () => {
        if (await api.backupNow()) toast.success("Backup saved.");
      },
    },
    {
      id: "lock",
      label: "Lock vault",
      icon: <Lock size={16} />,
      hint: "Ctrl L",
      requiresSecurity: true,
      execute: () => api.lockVault(),
    },
  ];
  const commandQuery = commandText(query);
  const matchingActions = actions.filter((action) => {
    if (action.requiresSecurity && !securityOn) return false;
    if (!hasQuery) return true;
    return commandQuery.length > 0 && commandText(action.label).includes(commandQuery);
  });
  const exactCommand = matchingActions.some((action) => commandText(action.label) === commandQuery);

  const itemClass =
    "flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm text-fg data-[selected=true]:bg-surface-2 data-[selected=true]:text-fg";

  const actionGroup = matchingActions.length > 0 && (
    <Command.Group heading="Actions" className="[&_[cmdk-group-heading]]:px-3 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:text-faint">
      {matchingActions.map((action) => (
        <Command.Item key={action.id} value={"action-" + action.id} onSelect={() => run(action.execute)} className={itemClass}>
          <span className="text-muted">{action.icon}</span>
          <span className="flex-1">{action.label}</span>
          {action.hint && <span className="text-xs text-faint">{action.hint}</span>}
        </Command.Item>
      ))}
    </Command.Group>
  );

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
              dir="auto"
              autoFocus
              placeholder="Search notes and sticky notes, or type a command"
              className="h-14 w-full border-b border-line bg-transparent px-5 text-[0.95rem] text-fg outline-none placeholder:text-faint"
            />
            <Command.List className="max-h-[52vh] overflow-y-auto p-2">
              {(!hasQuery || currentResults) && (
                <Command.Empty className="px-4 py-8 text-center text-sm text-muted">
                  {currentResults?.failed ? "Could not search notes." : "No matches."}
                </Command.Empty>
              )}

              {!hasQuery && (
                <RecentSearches
                  recent={searches.recent}
                  onPick={setQuery}
                  onForget={searches.forget}
                  onClear={searches.clear}
                  itemClass={itemClass}
                  headingClass="mb-1 [&_[cmdk-group-heading]]:px-3 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:text-faint"
                />
              )}

              {exactCommand && actionGroup}

              {hits.length > 0 && (
                <Command.Group heading="Results" className="mb-1 [&_[cmdk-group-heading]]:px-3 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:text-faint">
                  {hits.map((h) => (
                    <Command.Item key={`${h.kind}-${h.id}`} value={`${h.kind}-${h.id}`} onSelect={() => goToHit(h)} className={itemClass}>
                      <span className="text-muted">{icon(h)}</span>
                      <div className="min-w-0 flex-1">
                        <div dir="auto" className="truncate font-medium">{h.title || "Untitled"}</div>
                        {(h.subtitle || h.snippet) && (
                          <div dir="auto" className="truncate text-xs text-muted">{h.subtitle || h.snippet}</div>
                        )}
                      </div>
                      <span className="text-[0.7rem] uppercase tracking-wider text-faint">{h.kind}</span>
                    </Command.Item>
                  ))}
                </Command.Group>
              )}

              {!exactCommand && (!hasQuery || currentResults) && actionGroup}
            </Command.List>
          </Command>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
