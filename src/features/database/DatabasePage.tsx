// A database opened as the page of the Notes view: breadcrumb on top, the database below.

import { ChevronRight, Star } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { cn } from "@/lib/utils";
import { useApp } from "@/store/app";
import DatabaseView from "./DatabaseView";
import { setDatabase, peekDatabase } from "./dbStore";
import { PageIcon } from "@/lib/pageLook";

export default function DatabasePage({ databaseId }: { databaseId: string }) {
  const tree = useApp((s) => s.tree);
  const setTree = useApp((s) => s.setTree);
  const meta = tree.databases.find((d) => d.id === databaseId);
  const section = tree.sections.find((s) => s.id === meta?.sectionId);
  const notebook = tree.notebooks.find((n) => n.id === section?.notebookId);
  const parentPage = meta?.parent ? tree.pages.find((p) => p.id === meta.parent) : undefined;

  const toggleFavorite = async () => {
    if (!meta) return;
    try {
      const m = await api.setDatabaseLook(meta.id, meta.title, meta.icon, !meta.favorite, meta.locked);
      const t = useApp.getState().tree;
      setTree({ ...t, databases: t.databases.map((d) => (d.id === m.id ? m : d)) });
      const cur = peekDatabase(meta.id);
      if (cur) setDatabase({ ...cur, favorite: m.favorite });
    } catch (e) {
      toast.error(errorText(e));
    }
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex h-11 shrink-0 items-center gap-3 px-4">
        <nav className="flex min-w-0 items-center gap-1 text-[0.8rem] text-muted" aria-label="Breadcrumb">
          {notebook && (
            <>
              <span className="emoji-mono shrink-0">{notebook.icon}</span>
              <span className="max-w-[10rem] truncate">{notebook.name}</span>
              <ChevronRight size={12} className="shrink-0 text-faint rtl:rotate-180" />
            </>
          )}
          {section && (
            <>
              <span className="max-w-[10rem] truncate">{section.name}</span>
              <ChevronRight size={12} className="shrink-0 text-faint rtl:rotate-180" />
            </>
          )}
          {parentPage && (
            <>
              <button onClick={() => useApp.getState().openPage(parentPage.id)} className="max-w-[10rem] truncate hover:text-fg hover:underline">
                {parentPage.icon && <PageIcon icon={parentPage.icon} size={14} className="me-1" />}
                {parentPage.title || "Untitled"}
              </button>
              <ChevronRight size={12} className="shrink-0 text-faint rtl:rotate-180" />
            </>
          )}
          <span dir="auto" className="max-w-[16rem] truncate text-fg">
            {meta?.icon && <span className="emoji-mono me-1">{meta.icon}</span>}
            {meta?.title || "Untitled database"}
          </span>
        </nav>
        <div className="flex-1" />
        {meta && (
          <button
            onClick={toggleFavorite}
            className={cn("rounded-lg p-1.5 transition-colors hover:bg-surface-2", meta.favorite ? "text-fg" : "text-faint hover:text-fg")}
            aria-label={meta.favorite ? "Remove from favorites" : "Add to favorites"}
            aria-pressed={meta.favorite}
          >
            <Star size={16} fill={meta.favorite ? "currentColor" : "none"} />
          </button>
        )}
      </div>
      {meta?.trashedAt != null && (
        <div className="mx-6 mb-2 flex items-center justify-between rounded-lg border border-warn/30 bg-warn/10 px-4 py-2 text-sm sm:mx-10">
          <span>This database is in the trash.</span>
          <button className="font-medium underline-offset-2 hover:underline" onClick={async () => setTree(await api.restorePage(databaseId))}>
            Restore
          </button>
        </div>
      )}
      <div className="min-h-0 flex-1">
        <DatabaseView key={databaseId} databaseId={databaseId} />
      </div>
    </div>
  );
}
