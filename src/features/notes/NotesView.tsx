import { useEffect, useMemo, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import {
  DndContext,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import { SortableContext, arrayMove, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import {
  BookPlus,
  ChevronRight,
  FilePlus2,
  FileText,
  MoreHorizontal,
  NotebookPen,
  Pencil,
  Plus,
  RotateCcw,
  Star,
  Trash2,
} from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { NotesTree, PageMeta } from "@/lib/types";
import { cn, timeAgo } from "@/lib/utils";
import { useApp } from "@/store/app";
import { Button, EmptyState, IconButton, Input, Menu, MenuContent, MenuItem, MenuSeparator, MenuTrigger, useConfirm } from "@/components/ui";
import NotebookDialog, { type NotebookDraft } from "./NotebookDialog";
import PageEditor from "./PageEditor";

type Scope = "section" | "favorites" | "trash";

export default function NotesView() {
  const tree = useApp((s) => s.tree);
  const setTree = useApp((s) => s.setTree);
  const activeSectionId = useApp((s) => s.activeSectionId);
  const activePageId = useApp((s) => s.activePageId);
  const openPage = useApp((s) => s.openPage);
  const intent = useApp((s) => s.intent);
  const setIntent = useApp((s) => s.setIntent);
  const [scope, setScope] = useState<Scope>("section");
  const [newNotebookOpen, setNewNotebookOpen] = useState(false);

  // Navigating to a section (e.g. from search) always shows that section.
  useEffect(() => setScope("section"), [activeSectionId]);

  const run = async (fn: () => Promise<NotesTree>) => {
    try {
      setTree(await fn());
    } catch (e) {
      toast.error(errorText(e));
    }
  };

  const newPage = async () => {
    const sectionId = useApp.getState().activeSectionId;
    if (!sectionId) {
      setNewNotebookOpen(true);
      return;
    }
    try {
      const page = await api.createPage(sectionId, "");
      setTree(await api.listNotes());
      setScope("section");
      openPage(page.id);
    } catch (e) {
      toast.error(errorText(e));
    }
  };

  useEffect(() => {
    if (intent === "new-page") {
      setIntent(null);
      newPage();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [intent]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey && !e.shiftKey && e.key.toLowerCase() === "n") {
        e.preventDefault();
        newPage();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (tree.notebooks.length === 0) {
    return (
      <>
        <EmptyState
          icon={<NotebookPen size={28} />}
          title="Your first notebook"
          body="Notebooks hold sections, and sections hold pages, just like OneNote. Everything is encrypted the moment you type it."
          action={
            <Button variant="primary" onClick={() => setNewNotebookOpen(true)}>
              <BookPlus size={16} /> Create notebook
            </Button>
          }
        />
        <NotebookDialog
          open={newNotebookOpen}
          onOpenChange={setNewNotebookOpen}
          title="New notebook"
          onSubmit={async (d) => {
            setNewNotebookOpen(false);
            const t = await api.createNotebook(d.name, d.color, d.icon);
            setTree(t);
            const nb = t.notebooks[t.notebooks.length - 1];
            useApp.getState().openNotebook(nb.id);
          }}
        />
      </>
    );
  }

  return (
    <div className="flex h-full">
      <Sidebar scope={scope} setScope={setScope} run={run} newNotebookOpen={newNotebookOpen} setNewNotebookOpen={setNewNotebookOpen} />
      <PagesColumn scope={scope} run={run} onNewPage={newPage} />
      <section className="min-w-0 flex-1">
        <AnimatePresence mode="wait">
          {activePageId ? (
            <motion.div
              key={activePageId}
              className="h-full"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.12 }}
            >
              <PageEditor pageId={activePageId} />
            </motion.div>
          ) : (
            <motion.div key="empty" className="h-full" initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
              <EmptyState
                icon={<FileText size={28} />}
                title="Pick a page or start a new one"
                body="Tip: press Ctrl+N for a new page, or Ctrl+K to search everything."
                action={
                  scope === "section" && (
                    <Button variant="primary" onClick={newPage}>
                      <FilePlus2 size={16} /> New page
                    </Button>
                  )
                }
              />
            </motion.div>
          )}
        </AnimatePresence>
      </section>
    </div>
  );
}

/* ---------------- Notebooks & sections ---------------- */

function Sidebar({
  scope,
  setScope,
  run,
  newNotebookOpen,
  setNewNotebookOpen,
}: {
  scope: Scope;
  setScope: (s: Scope) => void;
  run: (fn: () => Promise<NotesTree>) => Promise<void>;
  newNotebookOpen: boolean;
  setNewNotebookOpen: (v: boolean) => void;
}) {
  const tree = useApp((s) => s.tree);
  const activeNotebookId = useApp((s) => s.activeNotebookId);
  const activeSectionId = useApp((s) => s.activeSectionId);
  const openNotebook = useApp((s) => s.openNotebook);
  const openSection = useApp((s) => s.openSection);
  const confirm = useConfirm();
  const [editing, setEditing] = useState<{ id: string; draft: NotebookDraft } | null>(null);
  const [addingSection, setAddingSection] = useState(false);
  const [sectionName, setSectionName] = useState("");
  const [renamingSection, setRenamingSection] = useState<string | null>(null);

  const sections = tree.sections.filter((s) => s.notebookId === activeNotebookId);
  const favorites = tree.pages.filter((p) => p.favorite && p.trashedAt === null).length;
  const trashed = tree.pages.filter((p) => p.trashedAt !== null).length;
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }));

  const onSectionDragEnd = (e: DragEndEvent) => {
    if (!e.over || e.active.id === e.over.id) return;
    const ids = sections.map((s) => s.id);
    const next = arrayMove(ids, ids.indexOf(String(e.active.id)), ids.indexOf(String(e.over.id)));
    run(() => api.reorder(next));
  };

  return (
    <aside className="flex w-[236px] shrink-0 flex-col border-r border-line">
      <div className="flex h-12 items-center justify-between px-4">
        <span className="text-xs font-semibold uppercase tracking-wider text-faint">Notebooks</span>
        <IconButton size="sm" label="New notebook" onClick={() => setNewNotebookOpen(true)}>
          <Plus size={15} />
        </IconButton>
      </div>

      <div className="min-h-0 flex-1 space-y-0.5 overflow-y-auto px-2 pb-3">
        {tree.notebooks.map((nb) => {
          const active = nb.id === activeNotebookId;
          return (
            <div key={nb.id}>
              <div
                className={cn(
                  "group flex cursor-pointer items-center gap-2 rounded-xl px-2.5 transition-colors",
                  active ? "bg-surface-2" : "hover:bg-surface",
                )}
                style={{ paddingTop: "var(--row-pad)", paddingBottom: "var(--row-pad)" }}
                onClick={() => {
                  openNotebook(nb.id);
                  setScope("section");
                }}
              >
                <ChevronRight size={14} className={cn("text-faint transition-transform", active && "rotate-90")} />
                <span className="text-base leading-none">{nb.icon}</span>
                <span className="flex-1 truncate text-sm font-medium">{nb.name}</span>
                <span className="h-2 w-2 rounded-full" style={{ background: nb.color }} />
                <Menu>
                  <MenuTrigger asChild>
                    <button
                      onClick={(e) => e.stopPropagation()}
                      className="rounded-md p-0.5 text-faint opacity-0 hover:text-fg group-hover:opacity-100"
                      aria-label="Notebook options"
                    >
                      <MoreHorizontal size={15} />
                    </button>
                  </MenuTrigger>
                  <MenuContent>
                    <MenuItem
                      icon={<Pencil size={14} />}
                      onSelect={() => setEditing({ id: nb.id, draft: { name: nb.name, icon: nb.icon, color: nb.color } })}
                    >
                      Edit notebook
                    </MenuItem>
                    <MenuSeparator />
                    <MenuItem
                      danger
                      icon={<Trash2 size={14} />}
                      onSelect={async () => {
                        const ok = await confirm({
                          title: `Delete "${nb.name}"?`,
                          body: "All of its sections and pages will be permanently deleted. This can't be undone.",
                          confirm: "Delete notebook",
                          danger: true,
                        });
                        if (ok) {
                          await run(() => api.deleteNotebook(nb.id));
                          const first = useApp.getState().tree.notebooks[0]?.id ?? null;
                          openNotebook(first);
                        }
                      }}
                    >
                      Delete notebook
                    </MenuItem>
                  </MenuContent>
                </Menu>
              </div>

              <AnimatePresence initial={false}>
                {active && (
                  <motion.div
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: "auto", opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    transition={{ duration: 0.18 }}
                    className="overflow-hidden"
                  >
                    <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={onSectionDragEnd}>
                      <SortableContext items={sections.map((s) => s.id)} strategy={verticalListSortingStrategy}>
                        <div className="ml-5 space-y-0.5 border-l border-line py-1 pl-2">
                          {sections.map((s) => (
                            <SortableRow key={s.id} id={s.id}>
                              {renamingSection === s.id ? (
                                <form
                                  onSubmit={(e) => {
                                    e.preventDefault();
                                    const name = (new FormData(e.currentTarget).get("name") as string).trim();
                                    setRenamingSection(null);
                                    if (name) run(() => api.updateSection(s.id, name, s.color));
                                  }}
                                >
                                  <Input name="name" defaultValue={s.name} autoFocus className="h-8 text-sm" onBlur={() => setRenamingSection(null)} />
                                </form>
                              ) : (
                                <div
                                  onClick={() => {
                                    openSection(s.id);
                                    setScope("section");
                                  }}
                                  onDoubleClick={() => setRenamingSection(s.id)}
                                  className={cn(
                                    "group/section flex cursor-pointer items-center gap-2 rounded-lg px-2.5 text-sm transition-colors",
                                    scope === "section" && s.id === activeSectionId
                                      ? "bg-accent-soft font-medium text-fg"
                                      : "text-muted hover:bg-surface hover:text-fg",
                                  )}
                                  style={{ paddingTop: "calc(var(--row-pad) - 2px)", paddingBottom: "calc(var(--row-pad) - 2px)" }}
                                >
                                  <span className="h-3.5 w-1 rounded-full" style={{ background: s.color || nb.color }} />
                                  <span className="flex-1 truncate">{s.name}</span>
                                  <Menu>
                                    <MenuTrigger asChild>
                                      <button
                                        onClick={(e) => e.stopPropagation()}
                                        className="rounded p-0.5 text-faint opacity-0 hover:text-fg group-hover/section:opacity-100"
                                        aria-label="Section options"
                                      >
                                        <MoreHorizontal size={14} />
                                      </button>
                                    </MenuTrigger>
                                    <MenuContent>
                                      <MenuItem icon={<Pencil size={14} />} onSelect={() => setRenamingSection(s.id)}>
                                        Rename
                                      </MenuItem>
                                      <MenuSeparator />
                                      <MenuItem
                                        danger
                                        icon={<Trash2 size={14} />}
                                        onSelect={async () => {
                                          const ok = await confirm({
                                            title: `Delete section "${s.name}"?`,
                                            body: "All pages in this section will be permanently deleted.",
                                            confirm: "Delete section",
                                            danger: true,
                                          });
                                          if (ok) run(() => api.deleteSection(s.id));
                                        }}
                                      >
                                        Delete section
                                      </MenuItem>
                                    </MenuContent>
                                  </Menu>
                                </div>
                              )}
                            </SortableRow>
                          ))}
                          {addingSection ? (
                            <form
                              onSubmit={(e) => {
                                e.preventDefault();
                                const name = sectionName.trim();
                                setAddingSection(false);
                                setSectionName("");
                                if (name) run(() => api.createSection(nb.id, name, ""));
                              }}
                            >
                              <Input
                                value={sectionName}
                                onChange={(e) => setSectionName(e.target.value)}
                                onBlur={() => setAddingSection(false)}
                                autoFocus
                                placeholder="Section name"
                                className="h-8 text-sm"
                              />
                            </form>
                          ) : (
                            <button
                              onClick={() => setAddingSection(true)}
                              className="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-sm text-faint hover:bg-surface hover:text-fg"
                            >
                              <Plus size={13} /> Add section
                            </button>
                          )}
                        </div>
                      </SortableContext>
                    </DndContext>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          );
        })}
      </div>

      <div className="space-y-0.5 border-t border-line p-2">
        <ScopeButton icon={<Star size={15} />} label="Favorites" count={favorites} active={scope === "favorites"} onClick={() => { setScope("favorites"); useApp.getState().openPage(null); }} />
        <ScopeButton icon={<Trash2 size={15} />} label="Trash" count={trashed} active={scope === "trash"} onClick={() => { setScope("trash"); useApp.getState().openPage(null); }} />
      </div>

      <NotebookDialog
        open={newNotebookOpen}
        onOpenChange={setNewNotebookOpen}
        title="New notebook"
        onSubmit={async (d) => {
          setNewNotebookOpen(false);
          await run(() => api.createNotebook(d.name, d.color, d.icon));
          const t = useApp.getState().tree;
          openNotebook(t.notebooks[t.notebooks.length - 1]?.id ?? null);
          setScope("section");
        }}
      />
      <NotebookDialog
        open={!!editing}
        onOpenChange={(o) => !o && setEditing(null)}
        title="Edit notebook"
        initial={editing?.draft}
        onSubmit={(d) => {
          if (editing) run(() => api.updateNotebook(editing.id, d.name, d.color, d.icon));
          setEditing(null);
        }}
      />
    </aside>
  );
}

function ScopeButton({ icon, label, count, active, onClick }: { icon: React.ReactNode; label: string; count: number; active: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-sm transition-colors",
        active ? "bg-accent-soft text-fg" : "text-muted hover:bg-surface hover:text-fg",
      )}
    >
      {icon}
      <span className="flex-1 text-left">{label}</span>
      {count > 0 && <span className="text-xs text-faint">{count}</span>}
    </button>
  );
}

function SortableRow({ id, children }: { id: string; children: React.ReactNode }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id });
  return (
    <div
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition, zIndex: isDragging ? 10 : undefined, opacity: isDragging ? 0.8 : 1 }}
      {...attributes}
      {...listeners}
    >
      {children}
    </div>
  );
}

/* ---------------- Pages list ---------------- */

function PagesColumn({ scope, run, onNewPage }: { scope: Scope; run: (fn: () => Promise<NotesTree>) => Promise<void>; onNewPage: () => void }) {
  const tree = useApp((s) => s.tree);
  const activeSectionId = useApp((s) => s.activeSectionId);
  const activePageId = useApp((s) => s.activePageId);
  const openPage = useApp((s) => s.openPage);
  const confirm = useConfirm();
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }));

  const pages = useMemo(() => {
    if (scope === "favorites") return tree.pages.filter((p) => p.favorite && p.trashedAt === null);
    if (scope === "trash") return [...tree.pages.filter((p) => p.trashedAt !== null)].sort((a, b) => (b.trashedAt ?? 0) - (a.trashedAt ?? 0));
    return tree.pages.filter((p) => p.sectionId === activeSectionId && p.trashedAt === null);
  }, [tree.pages, scope, activeSectionId]);

  const section = tree.sections.find((s) => s.id === activeSectionId);
  const heading = scope === "favorites" ? "Favorites" : scope === "trash" ? "Trash" : section?.name ?? "Pages";

  const onDragEnd = (e: DragEndEvent) => {
    if (scope !== "section" || !e.over || e.active.id === e.over.id) return;
    const ids = pages.map((p) => p.id);
    run(() => api.reorder(arrayMove(ids, ids.indexOf(String(e.active.id)), ids.indexOf(String(e.over!.id)))));
  };

  return (
    <div className="flex w-[280px] shrink-0 flex-col border-r border-line">
      <div className="flex h-12 items-center justify-between px-4">
        <span className="truncate text-sm font-semibold">{heading}</span>
        {scope === "section" && (
          <IconButton size="sm" label="New page (Ctrl+N)" onClick={onNewPage}>
            <FilePlus2 size={15} />
          </IconButton>
        )}
        {scope === "trash" && pages.length > 0 && (
          <button
            className="text-xs font-medium text-danger hover:underline"
            onClick={async () => {
              const ok = await confirm({
                title: "Empty the trash?",
                body: `${pages.length} page(s) will be permanently deleted.`,
                confirm: "Empty trash",
                danger: true,
              });
              if (ok) {
                await run(() => api.emptyTrash());
                openPage(null);
              }
            }}
          >
            Empty trash
          </button>
        )}
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
        {pages.length === 0 ? (
          <p className="px-3 py-10 text-center text-sm text-faint">
            {scope === "trash" ? "Trash is empty." : scope === "favorites" ? "Star a page to see it here." : "No pages yet."}
          </p>
        ) : (
          <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={onDragEnd}>
            <SortableContext items={pages.map((p) => p.id)} strategy={verticalListSortingStrategy}>
              <div className="space-y-1">
                {pages.map((p) => (
                  <SortableRow key={p.id} id={p.id}>
                    <PageRow
                      page={p}
                      active={p.id === activePageId}
                      onClick={() => openPage(p.id)}
                      trash={scope === "trash"}
                      onRestore={() => run(() => api.restorePage(p.id))}
                      onDelete={async () => {
                        const ok = await confirm({
                          title: "Delete forever?",
                          body: "This page will be permanently deleted.",
                          confirm: "Delete",
                          danger: true,
                        });
                        if (ok) {
                          await run(() => api.deletePage(p.id));
                          if (activePageId === p.id) openPage(null);
                        }
                      }}
                    />
                  </SortableRow>
                ))}
              </div>
            </SortableContext>
          </DndContext>
        )}
      </div>
    </div>
  );
}

function PageRow({
  page,
  active,
  onClick,
  trash,
  onRestore,
  onDelete,
}: {
  page: PageMeta;
  active: boolean;
  onClick: () => void;
  trash: boolean;
  onRestore: () => void;
  onDelete: () => void;
}) {
  return (
    <div
      onClick={onClick}
      className={cn(
        "group relative cursor-pointer rounded-xl px-3 py-2.5 transition-colors",
        active ? "bg-accent-soft" : "hover:bg-surface-2",
      )}
    >
      {active && <motion.span layoutId="page-active" className="absolute left-0 top-3 bottom-3 w-[3px] rounded-full bg-accent" />}
      <div className="flex items-center gap-1.5">
        <span className={cn("flex-1 truncate text-sm font-medium", !page.title && "text-faint")}>
          {page.title || "Untitled page"}
        </span>
        {page.favorite && <Star size={12} className="shrink-0 text-warn" fill="currentColor" />}
      </div>
      <p className="mt-0.5 line-clamp-2 text-xs leading-relaxed text-muted">{page.preview || "Empty page"}</p>
      <div className="mt-1 flex items-center justify-between text-[0.7rem] text-faint">
        <span>{timeAgo(page.updated)}</span>
        {trash && (
          <span className="flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
            <button onClick={(e) => { e.stopPropagation(); onRestore(); }} className="rounded p-1 hover:bg-surface hover:text-fg" title="Restore">
              <RotateCcw size={13} />
            </button>
            <button onClick={(e) => { e.stopPropagation(); onDelete(); }} className="rounded p-1 hover:bg-surface hover:text-danger" title="Delete forever">
              <Trash2 size={13} />
            </button>
          </span>
        )}
      </div>
    </div>
  );
}
