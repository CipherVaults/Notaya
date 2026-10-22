import { AnimatePresence, motion } from "motion/react";
import { BookPlus, FilePlus2, FileText, NotebookPen, RotateCcw, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { NotesTree } from "@/lib/types";
import { timeAgo } from "@/lib/utils";
import { useApp } from "@/store/app";
import { Button, EmptyState, useConfirm } from "@/components/ui";
import PageEditor from "./PageEditor";

/** The main area of the Notes view. The notebook tree lives in the sidebar. */
export default function NotesView() {
  const activePageId = useApp((s) => s.activePageId);
  const showTrash = useApp((s) => s.showTrash);

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
        {showTrash ? <TrashView /> : activePageId ? <PageEditor pageId={activePageId} /> : <Home />}
      </motion.div>
    </AnimatePresence>
  );
}

function Home() {
  const tree = useApp((s) => s.tree);
  const setIntent = useApp((s) => s.setIntent);
  const openPage = useApp((s) => s.openPage);

  if (tree.notebooks.length === 0) {
    return (
      <EmptyState
        icon={<NotebookPen size={28} />}
        title="Your first notebook"
        body="Notebooks hold sections, and sections hold pages. Everything is encrypted the moment you type it."
        action={
          <Button variant="primary" onClick={() => setIntent("new-notebook")}>
            <BookPlus size={16} /> Create notebook
          </Button>
        }
      />
    );
  }

  const recent = [...tree.pages]
    .filter((p) => p.trashedAt === null)
    .sort((a, b) => b.updated - a.updated)
    .slice(0, 6);
  const hour = new Date().getHours();
  const greeting = hour < 5 ? "Working late" : hour < 12 ? "Good morning" : hour < 18 ? "Good afternoon" : "Good evening";

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-3xl px-10 pb-16 pt-16">
        <h1 className="text-[2rem] font-bold tracking-tight">{greeting}</h1>
        <p className="mt-1 text-muted">Everything here is encrypted. Pick a page from the sidebar, or start a new one.</p>

        <div className="mt-8 flex gap-2">
          <Button variant="primary" onClick={() => setIntent("new-page")}>
            <FilePlus2 size={16} /> New page
          </Button>
          <Button
            variant="outline"
            onClick={() => api.createSticky(true).then(async () => useApp.getState().setStickies(await api.listStickies()))}
          >
            New sticky note
          </Button>
        </div>

        {recent.length > 0 && (
          <>
            <h2 className="mb-3 mt-12 text-xs font-semibold uppercase tracking-wider text-faint">Recently edited</h2>
            <div className="grid grid-cols-2 gap-3">
              {recent.map((p) => {
                const section = tree.sections.find((s) => s.id === p.sectionId);
                return (
                  <button
                    key={p.id}
                    onClick={() => openPage(p.id)}
                    className="group rounded-xl border border-line bg-surface p-4 text-left transition-colors hover:bg-surface-2"
                  >
                    <div className="mb-2 text-xl leading-none">
                      {p.icon ? <span className="emoji-mono">{p.icon}</span> : <FileText size={20} className="text-faint" />}
                    </div>
                    <div className="truncate font-medium">{p.title || "Untitled"}</div>
                    <div className="mt-0.5 line-clamp-2 min-h-[2.4em] text-xs leading-relaxed text-muted">{p.preview || "Empty page"}</div>
                    <div className="mt-2 text-[0.7rem] text-faint">
                      {section?.name} · {timeAgo(p.updated)}
                    </div>
                  </button>
                );
              })}
            </div>
          </>
        )}
      </div>
    </div>
  );
}

function TrashView() {
  const tree = useApp((s) => s.tree);
  const setTree = useApp((s) => s.setTree);
  const openPage = useApp((s) => s.openPage);
  const confirm = useConfirm();
  const pages = [...tree.pages.filter((p) => p.trashedAt !== null)].sort((a, b) => (b.trashedAt ?? 0) - (a.trashedAt ?? 0));

  const run = async (fn: () => Promise<NotesTree>) => {
    try {
      setTree(await fn());
    } catch (e) {
      toast.error(errorText(e));
    }
  };

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-3xl px-10 pb-16 pt-14">
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
                  {p.icon ? <span className="emoji-mono">{p.icon}</span> : <FileText size={17} className="mx-auto text-faint" />}
                </span>
                <button onClick={() => openPage(p.id)} className="min-w-0 flex-1 text-left">
                  <div className="truncate text-sm font-medium">{p.title || "Untitled"}</div>
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
