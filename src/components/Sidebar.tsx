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
  ChevronRight,
  FilePlus2,
  FileText,
  Home,
  KeyRound,
  MoreHorizontal,
  Pencil,
  Plus,
  Settings as SettingsIcon,
  Star,
  StickyNote,
  Trash2,
} from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { NotesTree, PageMeta } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useApp } from "@/store/app";
import { Input, Menu, MenuContent, MenuItem, MenuSeparator, MenuTrigger, useConfirm } from "./ui";
import NotebookDialog, { type NotebookDraft } from "@/features/notes/NotebookDialog";

type Run = (fn: () => Promise<NotesTree>) => Promise<void>;

export default function Sidebar() {
  const view = useApp((s) => s.view);
  const setView = useApp((s) => s.setView);
  const tree = useApp((s) => s.tree);
  const setTree = useApp((s) => s.setTree);
  const activeNotebookId = useApp((s) => s.activeNotebookId);
  const activeSectionId = useApp((s) => s.activeSectionId);
  const activePageId = useApp((s) => s.activePageId);
  const showTrash = useApp((s) => s.showTrash);
  const width = useApp((s) => s.sidebarWidth);
  const setWidth = useApp((s) => s.setSidebarWidth);
  const intent = useApp((s) => s.intent);
  const setIntent = useApp((s) => s.setIntent);
  const confirm = useConfirm();

  const [openIds, setOpenIds] = useState<Set<string>>(() => new Set([activeNotebookId, activeSectionId].filter(Boolean) as string[]));
  const [newNotebook, setNewNotebook] = useState(false);
  const [editing, setEditing] = useState<{ id: string; draft: NotebookDraft } | null>(null);
  const [favOpen, setFavOpen] = useState(true);

  const toggle = (id: string, force?: boolean) =>
    setOpenIds((prev) => {
      const next = new Set(prev);
      const on = force ?? !next.has(id);
      if (on) next.add(id);
      else next.delete(id);
      return next;
    });

  // Opening a page (for example from search) reveals it in the tree.
  useEffect(() => {
    const page = tree.pages.find((p) => p.id === activePageId);
    const section = tree.sections.find((s) => s.id === page?.sectionId);
    if (section) {
      setOpenIds((prev) => new Set(prev).add(section.id).add(section.notebookId));
    }
  }, [activePageId, tree.pages, tree.sections]);

  const run: Run = async (fn) => {
    try {
      setTree(await fn());
    } catch (e) {
      toast.error(errorText(e));
    }
  };

  const newPage = async (sectionId?: string | null) => {
    const sid = sectionId ?? useApp.getState().activeSectionId;
    if (!sid) {
      setNewNotebook(true);
      return;
    }
    try {
      const page = await api.createPage(sid, "");
      setTree(await api.listNotes());
      const section = useApp.getState().tree.sections.find((s) => s.id === sid);
      useApp.setState({ activeSectionId: sid, activeNotebookId: section?.notebookId ?? useApp.getState().activeNotebookId });
      toggle(sid, true);
      if (section) toggle(section.notebookId, true);
      useApp.getState().openPage(page.id);
    } catch (e) {
      toast.error(errorText(e));
    }
  };

  useEffect(() => {
    if (!intent) return;
    setIntent(null);
    if (intent === "new-page") newPage();
    else if (intent === "new-notebook") setNewNotebook(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [intent]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key.toLowerCase() === "n") {
        e.preventDefault();
        newPage();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const favorites = useMemo(() => tree.pages.filter((p) => p.favorite && p.trashedAt === null), [tree.pages]);
  const trashed = tree.pages.filter((p) => p.trashedAt !== null).length;
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }));

  const startResize = (e: React.PointerEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const start = width;
    const move = (ev: PointerEvent) => setWidth(Math.min(420, Math.max(200, start + ev.clientX - startX)));
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  const homeActive = view === "notes" && !activePageId && !showTrash;

  return (
    <aside className="relative flex shrink-0 flex-col bg-sidebar" style={{ width }}>
      <nav className="space-y-0.5 px-2 pb-2 pt-1">
        <NavRow icon={<Home size={17} />} label="Home" active={homeActive} onClick={() => { useApp.setState({ activePageId: null, showTrash: false }); setView("notes"); }} />
        <NavRow icon={<KeyRound size={17} />} label="Passwords" hint="Ctrl 2" active={view === "passwords"} onClick={() => setView("passwords")} />
        <NavRow icon={<StickyNote size={17} />} label="Sticky notes" hint="Ctrl 3" active={view === "stickies"} onClick={() => setView("stickies")} />
        <NavRow icon={<SettingsIcon size={17} />} label="Settings" hint="Ctrl ," active={view === "settings"} onClick={() => setView("settings")} />
      </nav>

      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
        {favorites.length > 0 && (
          <div className="mb-2">
            <GroupHeader label="Favorites" open={favOpen} onToggle={() => setFavOpen(!favOpen)} />
            {favOpen &&
              favorites.map((p) => (
                <PageItem key={p.id} page={p} depth={0} active={p.id === activePageId && view === "notes"} run={run} sections={tree} onOpen={() => useApp.getState().openPage(p.id)} />
              ))}
          </div>
        )}

        <GroupHeader label="Notebooks" action={{ label: "New notebook", onClick: () => setNewNotebook(true) }} />
        {tree.notebooks.length === 0 && (
          <button onClick={() => setNewNotebook(true)} className="mt-1 flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-sm text-muted hover:bg-surface-2 hover:text-fg">
            <Plus size={15} /> Create your first notebook
          </button>
        )}
        {tree.notebooks.map((nb) => {
          const open = openIds.has(nb.id);
          const sections = tree.sections.filter((s) => s.notebookId === nb.id);
          return (
            <div key={nb.id}>
              <Row
                depth={0}
                open={open}
                onToggle={() => toggle(nb.id)}
                onClick={() => {
                  toggle(nb.id);
                  useApp.setState({ activeNotebookId: nb.id });
                }}
                icon={<span className="emoji-mono text-[0.95rem] leading-none">{nb.icon}</span>}
                label={nb.name}
                active={false}
                actions={
                  <>
                    <Menu>
                      <MenuTrigger asChild>
                        <RowButton label="Notebook options">
                          <MoreHorizontal size={14} />
                        </RowButton>
                      </MenuTrigger>
                      <MenuContent align="start">
                        <MenuItem icon={<Plus size={14} />} onSelect={() => run(() => api.createSection(nb.id, "New section", "")).then(() => toggle(nb.id, true))}>
                          Add section
                        </MenuItem>
                        <MenuItem icon={<Pencil size={14} />} onSelect={() => setEditing({ id: nb.id, draft: { name: nb.name, icon: nb.icon, color: nb.color } })}>
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
                              useApp.getState().openNotebook(first);
                            }
                          }}
                        >
                          Delete notebook
                        </MenuItem>
                      </MenuContent>
                    </Menu>
                    <RowButton label="Add section" onClick={() => run(() => api.createSection(nb.id, "New section", "")).then(() => toggle(nb.id, true))}>
                      <Plus size={14} />
                    </RowButton>
                  </>
                }
              />
              <AnimatePresence initial={false}>
                {open && (
                  <motion.div initial={{ height: 0, opacity: 0 }} animate={{ height: "auto", opacity: 1 }} exit={{ height: 0, opacity: 0 }} transition={{ duration: 0.15 }} className="overflow-hidden">
                    <DndContext
                      sensors={sensors}
                      collisionDetection={closestCenter}
                      onDragEnd={(e: DragEndEvent) => {
                        if (!e.over || e.active.id === e.over.id) return;
                        const ids = sections.map((s) => s.id);
                        if (!ids.includes(String(e.active.id))) return;
                        run(() => api.reorder(arrayMove(ids, ids.indexOf(String(e.active.id)), ids.indexOf(String(e.over!.id)))));
                      }}
                    >
                      <SortableContext items={sections.map((s) => s.id)} strategy={verticalListSortingStrategy}>
                        {sections.map((s) => (
                          <SectionBlock
                            key={s.id}
                            section={s}
                            tree={tree}
                            open={openIds.has(s.id)}
                            onToggle={(force) => toggle(s.id, force)}
                            run={run}
                            onNewPage={() => newPage(s.id)}
                            sensors={sensors}
                            activePageId={view === "notes" ? activePageId : null}
                          />
                        ))}
                      </SortableContext>
                    </DndContext>
                    {sections.length === 0 && <div className="py-1 pl-9 text-xs text-faint">No sections yet</div>}
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          );
        })}
      </div>

      <div className="space-y-0.5 border-t border-line px-2 py-2">
        <button
          onClick={() => newPage()}
          className="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-sm text-muted transition-colors hover:bg-surface-2 hover:text-fg"
        >
          <FilePlus2 size={16} /> New page
          <span className="ml-auto text-[0.7rem] text-faint">Ctrl N</span>
        </button>
        <NavRow
          icon={<Trash2 size={16} />}
          label="Trash"
          count={trashed}
          small
          active={view === "notes" && showTrash}
          onClick={() => {
            useApp.getState().setShowTrash(true);
            setView("notes");
          }}
        />
      </div>

      <div onPointerDown={startResize} className="absolute right-0 top-0 z-10 h-full w-1.5 cursor-col-resize transition-colors hover:bg-line" />

      <NotebookDialog
        open={newNotebook}
        onOpenChange={setNewNotebook}
        title="New notebook"
        onSubmit={async (d) => {
          setNewNotebook(false);
          await run(() => api.createNotebook(d.name, d.color, d.icon));
          const t = useApp.getState().tree;
          const nb = t.notebooks[t.notebooks.length - 1];
          useApp.getState().openNotebook(nb?.id ?? null);
          if (nb) toggle(nb.id, true);
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

/* ---------------- rows ---------------- */

const rowPad = { paddingTop: "var(--row-pad)", paddingBottom: "var(--row-pad)" };

function NavRow({
  icon,
  label,
  active,
  onClick,
  hint,
  count,
  small,
}: {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
  hint?: string;
  count?: number;
  small?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "group flex w-full items-center gap-2.5 rounded-md px-2.5 text-left transition-colors",
        small ? "h-8 text-sm" : "h-9 text-[0.92rem]",
        active ? "bg-surface-2 font-medium text-fg" : "text-muted hover:bg-surface-2 hover:text-fg",
      )}
    >
      <span className={active ? "text-fg" : ""}>{icon}</span>
      <span className="flex-1 truncate">{label}</span>
      {count ? <span className="text-xs text-faint">{count}</span> : null}
      {hint && <span className="hidden text-[0.7rem] text-faint group-hover:inline">{hint}</span>}
    </button>
  );
}

function GroupHeader({
  label,
  open,
  onToggle,
  action,
}: {
  label: string;
  open?: boolean;
  onToggle?: () => void;
  action?: { label: string; onClick: () => void };
}) {
  return (
    <div className="group/gh flex h-7 items-center justify-between px-2.5 pt-1">
      <button onClick={onToggle} disabled={!onToggle} className="flex items-center gap-1 text-[0.72rem] font-semibold uppercase tracking-wider text-faint hover:text-muted">
        {onToggle && <ChevronRight size={12} className={cn("transition-transform", open && "rotate-90")} />}
        {label}
      </button>
      {action && (
        <button
          onClick={action.onClick}
          title={action.label}
          aria-label={action.label}
          className="grid h-5 w-5 place-items-center rounded text-faint opacity-0 hover:bg-surface-2 hover:text-fg group-hover/gh:opacity-100"
        >
          <Plus size={14} />
        </button>
      )}
    </div>
  );
}

const RowButton = ({ label, children, ...props }: { label: string; children: React.ReactNode } & React.ButtonHTMLAttributes<HTMLButtonElement>) => (
  <button
    aria-label={label}
    title={label}
    {...props}
    onClick={(e) => {
      e.stopPropagation();
      props.onClick?.(e);
    }}
    className="grid h-5 w-5 shrink-0 place-items-center rounded text-faint hover:bg-surface-2 hover:text-fg"
  >
    {children}
  </button>
);

function Row({
  depth,
  open,
  onToggle,
  onClick,
  icon,
  label,
  active,
  actions,
  muted,
}: {
  depth: number;
  open?: boolean;
  onToggle?: () => void;
  onClick: () => void;
  icon: React.ReactNode;
  label: string;
  active: boolean;
  actions?: React.ReactNode;
  muted?: boolean;
}) {
  return (
    <div
      onClick={onClick}
      className={cn(
        "group/row flex cursor-pointer items-center gap-1.5 rounded-md pr-1.5 text-sm transition-colors",
        active ? "bg-surface-2 text-fg" : muted ? "text-muted hover:bg-surface-2 hover:text-fg" : "text-fg/90 hover:bg-surface-2",
      )}
      style={{ paddingLeft: 6 + depth * 14, ...rowPad }}
    >
      {onToggle ? (
        <button
          aria-label={open ? "Collapse" : "Expand"}
          onClick={(e) => {
            e.stopPropagation();
            onToggle();
          }}
          className="grid h-5 w-5 shrink-0 place-items-center rounded text-faint hover:bg-surface-2 hover:text-fg"
        >
          <ChevronRight size={13} className={cn("transition-transform", open && "rotate-90")} />
        </button>
      ) : (
        <span className="w-5 shrink-0" />
      )}
      <span className="grid shrink-0 place-items-center">{icon}</span>
      <span className="min-w-0 flex-1 truncate">{label}</span>
      <span className="flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity focus-within:opacity-100 group-hover/row:opacity-100">{actions}</span>
    </div>
  );
}

function SortableRow({ id, children }: { id: string; children: React.ReactNode }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id });
  return (
    <div
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition, zIndex: isDragging ? 10 : undefined, opacity: isDragging ? 0.7 : 1 }}
      {...attributes}
      {...listeners}
    >
      {children}
    </div>
  );
}

/* ---------------- sections and pages ---------------- */

function SectionBlock({
  section,
  tree,
  open,
  onToggle,
  run,
  onNewPage,
  sensors,
  activePageId,
}: {
  section: NotesTree["sections"][number];
  tree: NotesTree;
  open: boolean;
  onToggle: (force?: boolean) => void;
  run: Run;
  onNewPage: () => void;
  sensors: ReturnType<typeof useSensors>;
  activePageId: string | null;
}) {
  const confirm = useConfirm();
  const [renaming, setRenaming] = useState(false);
  const pages = tree.pages.filter((p) => p.sectionId === section.id && p.trashedAt === null);
  const known = new Set(pages.map((p) => p.id));
  const roots = pages.filter((p) => !p.parent || !known.has(p.parent));
  // Only the section's own row starts a drag, so dragging its pages doesn't move the section.
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: section.id });

  return (
    <div
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition, zIndex: isDragging ? 10 : undefined, opacity: isDragging ? 0.7 : 1 }}
    >
      <div {...attributes} {...listeners}>
      {renaming ? (
        <form
          className="py-0.5 pl-6 pr-1"
          onSubmit={(e) => {
            e.preventDefault();
            const name = (new FormData(e.currentTarget).get("name") as string).trim();
            setRenaming(false);
            if (name) run(() => api.updateSection(section.id, name, section.color));
          }}
        >
          <Input name="name" defaultValue={section.name} autoFocus className="h-8 text-sm" onBlur={() => setRenaming(false)} onFocus={(e) => e.currentTarget.select()} />
        </form>
      ) : (
        <Row
          depth={1}
          open={open}
          onToggle={() => onToggle()}
          onClick={() => {
            onToggle();
            useApp.setState({ activeSectionId: section.id, activeNotebookId: section.notebookId });
          }}
          icon={<span className="h-3.5 w-1 rounded-full bg-faint/60" style={section.color ? { background: section.color } : undefined} />}
          label={section.name}
          active={false}
          muted
          actions={
            <>
              <Menu>
                <MenuTrigger asChild>
                  <RowButton label="Section options">
                    <MoreHorizontal size={14} />
                  </RowButton>
                </MenuTrigger>
                <MenuContent align="start">
                  <MenuItem icon={<Pencil size={14} />} onSelect={() => setRenaming(true)}>
                    Rename
                  </MenuItem>
                  <MenuSeparator />
                  <MenuItem
                    danger
                    icon={<Trash2 size={14} />}
                    onSelect={async () => {
                      const ok = await confirm({
                        title: `Delete section "${section.name}"?`,
                        body: "All pages in this section will be permanently deleted.",
                        confirm: "Delete section",
                        danger: true,
                      });
                      if (ok) run(() => api.deleteSection(section.id));
                    }}
                  >
                    Delete section
                  </MenuItem>
                </MenuContent>
              </Menu>
              <RowButton label="New page in this section" onClick={onNewPage}>
                <Plus size={14} />
              </RowButton>
            </>
          }
        />
      )}
      </div>
      <AnimatePresence initial={false}>
        {open && (
          <motion.div initial={{ height: 0, opacity: 0 }} animate={{ height: "auto", opacity: 1 }} exit={{ height: 0, opacity: 0 }} transition={{ duration: 0.15 }} className="overflow-hidden">
            <DndContext
              sensors={sensors}
              collisionDetection={closestCenter}
              onDragEnd={(e: DragEndEvent) => {
                if (!e.over || e.active.id === e.over.id) return;
                const ids = roots.map((p) => p.id);
                if (!ids.includes(String(e.active.id))) return;
                run(() => api.reorder(arrayMove(ids, ids.indexOf(String(e.active.id)), ids.indexOf(String(e.over!.id)))));
              }}
            >
              <SortableContext items={roots.map((p) => p.id)} strategy={verticalListSortingStrategy}>
                {roots.map((p) => (
                  <SortableRow key={p.id} id={p.id}>
                    <PageBranch page={p} depth={2} pages={pages} tree={tree} activePageId={activePageId} run={run} />
                  </SortableRow>
                ))}
              </SortableContext>
            </DndContext>
            {pages.length === 0 && (
              <button onClick={onNewPage} className="flex w-full items-center gap-1.5 rounded-md py-1 pl-[58px] text-left text-xs text-faint hover:bg-surface-2 hover:text-muted">
                No pages inside. Add one
              </button>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

/** A page and the pages inside it, shown nested. */
function PageBranch({
  page,
  depth,
  pages,
  tree,
  activePageId,
  run,
}: {
  page: PageMeta;
  depth: number;
  pages: PageMeta[];
  tree: NotesTree;
  activePageId: string | null;
  run: Run;
}) {
  const kids = pages.filter((p) => p.parent === page.id);
  const containsActive = (id: string): boolean => pages.some((p) => p.parent === id && (p.id === activePageId || containsActive(p.id)));
  const [open, setOpen] = useState(() => containsActive(page.id));
  useEffect(() => {
    if (activePageId && containsActive(page.id)) setOpen(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activePageId]);
  return (
    <>
      <PageItem
        page={page}
        depth={depth}
        active={page.id === activePageId}
        run={run}
        sections={tree}
        onOpen={() => useApp.getState().openPage(page.id)}
        childCount={kids.length}
        open={open}
        onToggle={() => setOpen(!open)}
        onAddSub={async () => {
          try {
            const created = await api.createSubpage(page.id, "");
            useApp.getState().setTree(await api.listNotes());
            setOpen(true);
            useApp.getState().openPage(created.id);
          } catch (e) {
            toast.error(errorText(e));
          }
        }}
      />
      {open && kids.map((k) => <PageBranch key={k.id} page={k} depth={depth + 1} pages={pages} tree={tree} activePageId={activePageId} run={run} />)}
    </>
  );
}

function PageItem({
  page,
  depth,
  active,
  run,
  sections,
  onOpen,
  childCount = 0,
  open,
  onToggle,
  onAddSub,
}: {
  page: PageMeta;
  depth: number;
  active: boolean;
  run: Run;
  sections: NotesTree;
  onOpen: () => void;
  childCount?: number;
  open?: boolean;
  onToggle?: () => void;
  onAddSub?: () => void;
}) {
  const setView = useApp((s) => s.setView);
  return (
    <Row
      depth={depth}
      open={open}
      onToggle={childCount > 0 ? onToggle : undefined}
      onClick={() => {
        setView("notes");
        onOpen();
      }}
      icon={page.icon ? <span className="emoji-mono text-[0.9rem] leading-none">{page.icon}</span> : <FileText size={15} className="text-faint" />}
      label={page.title || "Untitled"}
      active={active}
      muted={!page.title}
      actions={
        <>
        <Menu>
          <MenuTrigger asChild>
            <RowButton label="Page options">
              <MoreHorizontal size={14} />
            </RowButton>
          </MenuTrigger>
          <MenuContent align="start">
            {onAddSub && (
              <MenuItem icon={<Plus size={14} />} onSelect={onAddSub}>
                Add a page inside
              </MenuItem>
            )}
            <MenuItem icon={<Star size={14} />} onSelect={() => run(() => api.setPageFavorite(page.id, !page.favorite))}>
              {page.favorite ? "Remove from favorites" : "Add to favorites"}
            </MenuItem>
            {sections.sections.length > 1 && <div className="px-2.5 pb-1 pt-2 text-xs text-faint">Move to</div>}
            {sections.sections
              .filter((s) => s.id !== page.sectionId)
              .slice(0, 8)
              .map((s) => {
                const nb = sections.notebooks.find((n) => n.id === s.notebookId);
                return (
                  <MenuItem key={s.id} onSelect={() => run(() => api.movePage(page.id, s.id))}>
                    <span className="truncate">
                      {nb?.name} / {s.name}
                    </span>
                  </MenuItem>
                );
              })}
            <MenuSeparator />
            <MenuItem
              danger
              icon={<Trash2 size={14} />}
              onSelect={async () => {
                await run(() => api.trashPage(page.id));
                if (useApp.getState().activePageId === page.id) useApp.getState().openPage(null);
                toast("Moved to trash", { action: { label: "Undo", onClick: () => run(() => api.restorePage(page.id)) } });
              }}
            >
              Move to trash
            </MenuItem>
          </MenuContent>
        </Menu>
        {onAddSub && (
          <RowButton label="Add a page inside" onClick={onAddSub}>
            <Plus size={14} />
          </RowButton>
        )}
        </>
      }
    />
  );
}
