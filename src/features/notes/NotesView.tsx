import { AnimatePresence, motion } from "motion/react";
import { FileText, RotateCcw, Table2, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { NotesTree } from "@/lib/types";
import { timeAgo } from "@/lib/utils";
import { useApp } from "@/store/app";
import { Button, useConfirm } from "@/components/ui";
import { PageIcon } from "@/lib/pageLook";
import Home from "./HomeView";
import PageEditor from "./PageEditor";
import DatabasePage from "@/features/database/DatabasePage";

/** The main area of the Notes view. The notebook tree lives in the sidebar. */
export default function NotesView() {
  const activePageId = useApp((s) => s.activePageId);
  const showTrash = useApp((s) => s.showTrash);
  const activeKind = useApp((s) => s.activeKind);

  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={showTrash ? "trash" : (activePageId ?? "home")}
        className="h-full"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.12 }}
      >
        {showTrash ? (
          <TrashView />
        ) : activePageId ? (
          activeKind === "database" ? <DatabasePage databaseId={activePageId} /> : <PageEditor pageId={activePageId} />
        ) : (
          <Home />
        )}
      </motion.div>
    </AnimatePresence>
  );
}

function TrashView() {
  const tree = useApp((s) => s.tree);
  const setTree = useApp((s) => s.setTree);
  const openPage = useApp((s) => s.openPage);
  const confirm = useConfirm();
  const trashedPages = tree.pages.filter((p) => p.trashedAt !== null).map((p) => ({ id: p.id, title: p.title, icon: p.icon, trashedAt: p.trashedAt, updated: p.updated, db: false }));
  const trashedDbs = tree.databases.filter((d) => d.trashedAt !== null).map((d) => ({ id: d.id, title: d.title, icon: d.icon, trashedAt: d.trashedAt, updated: d.updated, db: true }));
  const pages = [...trashedPages, ...trashedDbs].sort((a, b) => (b.trashedAt ?? 0) - (a.trashedAt ?? 0));

  const run = async (fn: () => Promise<NotesTree>) => {
    try {
      setTree(await fn());
    } catch (e) {
      toast.error(errorText(e));
    }
  };

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto px-10 pb-16 pt-14" style={{ maxWidth: "calc(var(--editor-width) + 80px)" }}>
        <div className="flex items-end justify-between">
          <div>
            <h1 className="text-[2rem] font-bold tracking-tight">Trash</h1>
            <p className="mt-1 text-muted">Pages you delete wait here until you empty the trash.</p>
          </div>
          {pages.length > 0 && (
            <Button
              variant="outline"
              onClick={async () => {
                const ok = await confirm({
                  title: "Empty the trash?",
                  body: `${pages.length} page(s) will be permanently deleted.`,
                  confirm: "Empty trash",
                  danger: true,
                });
                if (ok) run(() => api.emptyTrash());
              }}
            >
              <Trash2 size={15} /> Empty trash
            </Button>
          )}
        </div>

        {pages.length === 0 ? (
          <p className="py-16 text-center text-sm text-faint">Trash is empty.</p>
        ) : (
          <div className="mt-8 divide-y divide-line rounded-xl border border-line">
            {pages.map((p) => (
              <div key={p.id} className="flex items-center gap-3 px-4 py-3">
                <span className="w-6 text-center text-lg">
                  <PageIcon icon={p.icon} size={18} fallback={p.db ? <Table2 size={17} className="mx-auto text-faint" /> : <FileText size={17} className="mx-auto text-faint" />} />
                </span>
                <button onClick={() => openPage(p.id)} className="min-w-0 flex-1 text-left">
                  <div dir="auto" className="truncate text-sm font-medium">{p.title || "Untitled"}{p.db && <span className="ms-2 text-xs font-normal text-faint">Database</span>}</div>
                  <div className="truncate text-xs text-faint">Deleted {timeAgo(p.trashedAt ?? p.updated)}</div>
                </button>
                <Button variant="ghost" size="sm" onClick={() => run(() => api.restorePage(p.id))}>
                  <RotateCcw size={14} /> Restore
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="text-danger hover:text-danger"
                  onClick={async () => {
                    const ok = await confirm({ title: "Delete forever?", body: "This page will be permanently deleted.", confirm: "Delete", danger: true });
                    if (ok) run(() => api.deletePage(p.id));
                  }}
                >
                  <Trash2 size={14} /> Delete
                </Button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
