import { createContext, useContext, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
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
  ArrowDown,
  ArrowUp,
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

/** Runs a tree-changing command and reports whether it worked (a failure is shown as a toast). */
type Run = (fn: () => Promise<NotesTree>) => Promise<boolean>;

const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;

/** What deleting the given sections would remove, counted from the loaded tree. */
function contents(tree: NotesTree, sectionIds: string[], withSections: boolean): string {
  const ids = new Set(sectionIds);
  const pages = tree.pages.filter((p) => ids.has(p.sectionId));
  const trashed = pages.filter((p) => p.trashedAt !== null).length;
  const pageText = pages.length === 0 ? "no pages" : plural(pages.length, "page");
  const trashText = trashed > 0 ? ` (${trashed} already in the trash)` : "";
  return withSections ? `It has ${plural(ids.size, "section")} and ${pageText}${trashText}.` : `It has ${pageText}${trashText}.`;
}

const DELETE_NOTE =
  "This can't be undone from inside the app, but Cipherleaf saves a backup copy first, so you can use Restore from a backup in Settings.";

const rowByKey = (key: string) => document.querySelector<HTMLElement>(`[data-rowkey="${window.CSS.escape(key)}"]`);

/** The keys of the rows next to a row at the same level, nearest first: the next sibling, then the previous one. */
function siblingKeys(key: string): string[] {
  const row = rowByKey(key);
  const items = row ? Array.from(row.closest('[role="tree"]')?.querySelectorAll<HTMLElement>('[role="treeitem"]') ?? []) : [];
  const at = row ? items.indexOf(row) : -1;
  if (at < 0) return [];
  const level = Number(items[at].getAttribute("aria-level"));
  const find = (dir: 1 | -1) => {
    for (let i = at + dir; i >= 0 && i < items.length; i += dir) {
      const l = Number(items[i].getAttribute("aria-level"));
      if (l < level) return null;
      if (l === level) return items[i].dataset.rowkey ?? null;
    }
    return null;
  };
  return [find(1), find(-1)].filter((k): k is string => !!k);
}

/**
 * Deletes a notebook or section after a confirm that counts what goes. Shows the error if the backup or the delete fails.
 * `rowKey` is the row being deleted: once the dialog closes, the keyboard focus goes back to that row,
 * or, after a delete, to the next row at the same level, the previous one, or the row of the tree that is in the Tab order.
 */
function useDeleteWithConfirm() {
  const confirm = useConfirm();
  const setTree = useApp((s) => s.setTree);
  return async (o: { rowKey: string; title: string; body: string; confirm: string; name: string; del: () => Promise<NotesTree> }) => {
    const near = siblingKeys(o.rowKey);
    const tree = rowByKey(o.rowKey)?.closest<HTMLElement>('[role="tree"]') ?? null;
    // Wait a moment so the dialog has closed and the tree has redrawn before looking for the row.
    const refocus = (keys: string[]) =>
      setTimeout(() => {
        const row =
          keys.map(rowByKey).find((r) => r) ??
          (tree?.isConnected ? tree : document).querySelector<HTMLElement>('[role="treeitem"][tabindex="0"], [role="treeitem"]');
        row?.focus();
      }, 120);
    if (!(await confirm({ title: o.title, body: o.body, confirm: o.confirm, danger: true }))) {
      refocus([o.rowKey]);
      return false;
    }
    try {
      setTree(await o.del());
      refocus([o.rowKey, ...near]);
      return true;
    } catch (e) {
      toast.error(`"${o.name}" was not deleted. ${errorText(e)}`, { duration: 12000 });
      refocus([o.rowKey]);
      return false;
    }
  };
}

/** Moves an item one place up or down among its siblings. */
const moveBy = (run: Run, ids: string[], id: string, delta: -1 | 1) => {
  const from = ids.indexOf(id);
  const to = from + delta;
  if (from < 0 || to < 0 || to >= ids.length) return Promise.resolve(false);
  return run(() => api.reorder(arrayMove(ids, from, to)));
};

/** Keeps the keyboard focus on the same row after the row's menu closes. */
const focusRow = (key: string) =>
  requestAnimationFrame(() => document.querySelector<HTMLElement>(`[data-rowkey="${window.CSS.escape(key)}"]`)?.focus());

/** Tells screen readers the truth about reordering: the sortable rows are dragged by pointer only. */
const dndA11y = {
  // Keep dnd-kit's hidden text and live region out of the tree element.
  container: typeof document === "undefined" ? undefined : document.body,
  screenReaderInstructions: {
    draggable: "Drag with the mouse. To reorder with the keyboard, open the row's menu and choose Move up or Move down.",
  },
};

export default function Sidebar() {
  const view = useApp((s) => s.view);
  const passwordsOn = useApp((s) => s.settings?.passwordsEnabled === true);
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
  const deleteWithConfirm = useDeleteWithConfirm();

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
      return true;
    } catch (e) {
      toast.error(errorText(e));
      return false;
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
  const notebookIds = tree.notebooks.map((n) => n.id);

  return (
    <aside className="relative flex shrink-0 flex-col bg-sidebar" style={{ width }}>
      <nav aria-label="Primary" className="space-y-0.5 px-2 pb-2 pt-1">
        <NavRow icon={<Home size={17} />} label="Home" active={homeActive} onClick={() => { useApp.setState({ activePageId: null, showTrash: false }); setView("notes"); }} />
        {passwordsOn && (
          <NavRow icon={<KeyRound size={17} />} label="Passwords" hint="Ctrl 2" active={view === "passwords"} onClick={() => setView("passwords")} />
        )}
        <NavRow icon={<StickyNote size={17} />} label="Sticky notes" hint="Ctrl 3" active={view === "stickies"} onClick={() => setView("stickies")} />
        <NavRow icon={<SettingsIcon size={17} />} label="Settings" hint="Ctrl ," active={view === "settings"} onClick={() => setView("settings")} />
      </nav>

      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
        {favorites.length > 0 && (
          <div className="mb-2">
            <GroupHeader label="Favorites" open={favOpen} onToggle={() => setFavOpen(!favOpen)} />
            {favOpen && (
              <Tree label="Favorites">
                {favorites.map((p, i) => (
                  <PageItem
                    key={p.id}
                    rowKey={`fav:${p.id}`}
                    page={p}
                    depth={0}
                    pos={{ i, n: favorites.length }}
                    active={p.id === activePageId && view === "notes"}
                    run={run}
                    sections={tree}
                    onOpen={() => useApp.getState().openPage(p.id)}
                  />
                ))}
              </Tree>
            )}
          </div>
        )}

        <GroupHeader label="Notebooks" action={{ label: "New notebook", onClick: () => setNewNotebook(true) }} />
        {tree.notebooks.length === 0 && (
          <button onClick={() => setNewNotebook(true)} className="mt-1 flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-sm text-muted hover:bg-surface-2 hover:text-fg">
            <Plus size={15} /> Create your first notebook
          </button>
        )}
        {tree.notebooks.length > 0 && (
          <Tree label="Notebooks">
            {tree.notebooks.map((nb, nbIndex) => {
              const open = openIds.has(nb.id);
              const sections = tree.sections.filter((s) => s.notebookId === nb.id);
              const addSection = () => run(() => api.createSection(nb.id, "New section", "")).then(() => toggle(nb.id, true));
              return (
                <div key={nb.id}>
                  <Row
                    rowKey={`nb:${nb.id}`}
                    depth={0}
                    pos={{ i: nbIndex, n: tree.notebooks.length }}
                    open={open}
                    onToggle={() => toggle(nb.id)}
                    onClick={() => {
                      toggle(nb.id);
                      useApp.setState({ activeNotebookId: nb.id });
                    }}
                    icon={<span className="emoji-mono text-[0.95rem] leading-none">{nb.icon}</span>}
                    label={nb.name}
                    active={false}
                    menuLabel="Notebook options"
                    quick={{ label: "Add section", onClick: addSection }}
                    menu={
                      <>
                        <MenuItem icon={<Plus size={14} />} onSelect={addSection}>
                          Add section
                        </MenuItem>
                        <MenuItem icon={<Pencil size={14} />} onSelect={() => setEditing({ id: nb.id, draft: { name: nb.name, icon: nb.icon, color: nb.color } })}>
                          Edit notebook
                        </MenuItem>
                        {nbIndex > 0 && (
                          <MenuItem icon={<ArrowUp size={14} />} onSelect={() => moveBy(run, notebookIds, nb.id, -1)}>
                            Move up
                          </MenuItem>
                        )}
                        {nbIndex < notebookIds.length - 1 && (
                          <MenuItem icon={<ArrowDown size={14} />} onSelect={() => moveBy(run, notebookIds, nb.id, 1)}>
                            Move down
                          </MenuItem>
                        )}
                        <MenuSeparator />
                        <MenuItem
                          danger
                          icon={<Trash2 size={14} />}
                          onSelect={async () => {
                            const ok = await deleteWithConfirm({
                              rowKey: `nb:${nb.id}`,
                              title: `Delete notebook "${nb.name}"?`,
                              body: `${contents(tree, sections.map((s) => s.id), true)} ${DELETE_NOTE}`,
                              confirm: "Delete notebook",
                              name: nb.name,
                              del: () => api.deleteNotebook(nb.id),
                            });
                            if (ok) useApp.getState().openNotebook(useApp.getState().tree.notebooks[0]?.id ?? null);
                          }}
                        >
                          Delete notebook
                        </MenuItem>
                      </>
                    }
                  />
                  <AnimatePresence initial={false}>
                    {open && (
                      <motion.div initial={{ height: 0, opacity: 0 }} animate={{ height: "auto", opacity: 1 }} exit={{ height: 0, opacity: 0 }} transition={{ duration: 0.15 }} className="overflow-hidden">
                        <DndContext
                          sensors={sensors}
                          accessibility={dndA11y}
                          collisionDetection={closestCenter}
                          onDragEnd={(e: DragEndEvent) => {
                            if (!e.over || e.active.id === e.over.id) return;
                            const ids = sections.map((s) => s.id);
                            if (!ids.includes(String(e.active.id))) return;
                            run(() => api.reorder(arrayMove(ids, ids.indexOf(String(e.active.id)), ids.indexOf(String(e.over!.id)))));
                          }}
                        >
                          <SortableContext items={sections.map((s) => s.id)} strategy={verticalListSortingStrategy}>
                            {sections.map((s, i) => (
                              <SectionBlock
                                key={s.id}
                                section={s}
                                pos={{ i, n: sections.length }}
                                siblings={sections.map((x) => x.id)}
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
          </Tree>
        )}
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

      <div
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize sidebar"
        aria-valuemin={200}
        aria-valuemax={420}
        aria-valuenow={width}
        tabIndex={0}
        onPointerDown={startResize}
        onKeyDown={(e) => {
          if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
            e.preventDefault();
            setWidth(Math.min(420, Math.max(200, width + (e.key === "ArrowRight" ? 20 : -20))));
          }
        }}
        className="absolute right-0 top-0 z-10 h-full w-1.5 cursor-col-resize outline-none transition-colors hover:bg-line focus-visible:bg-accent"
      />

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

/** Where a row sits among its siblings (zero-based index and count), for screen readers. */
type Pos = { i: number; n: number };

const TreeContext = createContext<{ tabKey: string | null; setTabKey: (k: string) => void }>({ tabKey: null, setTabKey: () => {} });

/**
 * One tree widget. Only one row at a time is in the Tab order (roving tabindex): the row you
 * last used, or the open page, or the first row. Arrow keys move between the rows.
 */
function Tree({ label, children }: { label: string; children: React.ReactNode }) {
  const ref = useRef<HTMLDivElement>(null);
  const [tabKey, setTabKey] = useState<string | null>(null);
  // If the remembered row is gone (collapsed, deleted), fall back to the open page or the first row.
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el || el.querySelector('[role="treeitem"][tabindex="0"]')) return;
    const items = el.querySelectorAll<HTMLElement>('[role="treeitem"]');
    const pick = el.querySelector<HTMLElement>('[role="treeitem"][aria-selected="true"]') ?? items[0];
    if (pick?.dataset.rowkey) setTabKey(pick.dataset.rowkey);
  });
  return (
    <TreeContext.Provider value={{ tabKey, setTabKey }}>
      <div ref={ref} role="tree" aria-label={label}>
        {children}
      </div>
    </TreeContext.Provider>
  );
}

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
      aria-current={active ? "page" : undefined}
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
      <button
        onClick={onToggle}
        disabled={!onToggle}
        aria-expanded={onToggle ? !!open : undefined}
        className="flex items-center gap-1 text-[0.72rem] font-semibold uppercase tracking-wider text-faint hover:text-muted"
      >
        {onToggle && <ChevronRight size={12} className={cn("transition-transform", open && "rotate-90")} />}
        {label}
      </button>
      {action && (
        <button
          onClick={action.onClick}
          title={action.label}
          aria-label={action.label}
          className="grid h-5 w-5 place-items-center rounded text-faint opacity-0 hover:bg-surface-2 hover:text-fg focus-visible:opacity-100 group-hover/gh:opacity-100"
        >
          <Plus size={14} />
        </button>
      )}
    </div>
  );
}

/** A small icon button inside a row. Mouse only: the keyboard reaches the same actions through the row's menu. */
const RowButton = ({ label, children, ...props }: { label: string; children: React.ReactNode } & React.ButtonHTMLAttributes<HTMLButtonElement>) => (
  <button
    aria-label={label}
    title={label}
    tabIndex={-1}
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

/**
 * A row of the tree (a notebook, section or page). It is the focusable `treeitem`.
 * Keys: Up/Down move, Right/Left open/close or step into/out of a branch, Home/End jump,
 * Enter/Space open or toggle, Shift+F10 or the Menu key opens the row's menu.
 */
function Row({
  rowKey,
  depth,
  pos,
  open,
  onToggle,
  onClick,
  icon,
  label,
  active,
  menu,
  menuLabel,
  quick,
  muted,
}: {
  rowKey: string;
  depth: number;
  pos: Pos;
  open?: boolean;
  onToggle?: () => void;
  onClick: () => void;
  icon: React.ReactNode;
  label: string;
  active: boolean;
  menu?: React.ReactNode;
  menuLabel?: string;
  quick?: { label: string; onClick: () => void };
  muted?: boolean;
}) {
  const { tabKey, setTabKey } = useContext(TreeContext);
  const ref = useRef<HTMLDivElement>(null);
  const [menuOpen, setMenuOpen] = useState(false);
  const expandable = onToggle !== undefined;

  const onKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.target !== e.currentTarget) return;
    if ((e.key === "ContextMenu" || (e.key === "F10" && e.shiftKey)) && menu) {
      e.preventDefault();
      setMenuOpen(true);
      return;
    }
    if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
    const el = e.currentTarget;
    const items = Array.from(el.closest('[role="tree"]')?.querySelectorAll<HTMLElement>('[role="treeitem"]') ?? []);
    const at = items.indexOf(el);
    const level = Number(el.getAttribute("aria-level"));
    const go = (i: number) => {
      e.preventDefault();
      items[Math.max(0, Math.min(items.length - 1, i))]?.focus();
    };
    switch (e.key) {
      case "ArrowDown":
        return go(at + 1);
      case "ArrowUp":
        return go(at - 1);
      case "Home":
        return go(0);
      case "End":
        return go(items.length - 1);
      case "ArrowRight":
        if (expandable && !open) {
          e.preventDefault();
          onToggle();
        } else if (expandable && Number(items[at + 1]?.getAttribute("aria-level")) > level) go(at + 1);
        return;
      case "ArrowLeft":
        if (expandable && open) {
          e.preventDefault();
          onToggle();
        } else {
          for (let i = at - 1; i >= 0; i--) {
            if (Number(items[i].getAttribute("aria-level")) < level) return go(i);
          }
        }
        return;
      case "Enter":
      case " ":
        e.preventDefault();
        onClick();
    }
  };

  return (
    <div
      ref={ref}
      role="treeitem"
      aria-label={label}
      aria-level={depth + 1}
      aria-setsize={pos.n}
      aria-posinset={pos.i + 1}
      aria-expanded={expandable ? !!open : undefined}
      aria-selected={active}
      aria-current={active ? "page" : undefined}
      data-rowkey={rowKey}
      tabIndex={tabKey === rowKey ? 0 : -1}
      onFocus={(e) => {
        if (e.target === e.currentTarget) setTabKey(rowKey);
      }}
      onKeyDown={onKeyDown}
      onClick={(e) => {
        // Clicks inside the row's menu (a portal) also bubble here in React; ignore them.
        if (e.currentTarget.contains(e.target as Node)) onClick();
      }}
      className={cn(
        "group/row flex cursor-pointer items-center gap-1.5 rounded-md pr-1.5 text-sm outline-none transition-colors focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-accent",
        active ? "bg-surface-2 text-fg" : muted ? "text-muted hover:bg-surface-2 hover:text-fg" : "text-fg/90 hover:bg-surface-2",
      )}
      style={{ paddingLeft: 6 + depth * 14, ...rowPad }}
    >
      {expandable ? (
        <span
          aria-hidden="true"
          onClick={(e) => {
            e.stopPropagation();
            onToggle();
          }}
          className="grid h-5 w-5 shrink-0 place-items-center rounded text-faint hover:bg-surface-2 hover:text-fg"
        >
          <ChevronRight size={13} className={cn("transition-transform", open && "rotate-90")} />
        </span>
      ) : (
        <span className="w-5 shrink-0" />
      )}
      <span className="grid shrink-0 place-items-center">{icon}</span>
      <span className="min-w-0 flex-1 truncate">{label}</span>
      <span className="flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity focus-within:opacity-100 group-focus-visible/row:opacity-100 group-hover/row:opacity-100">
        {menu && (
          <Menu
            open={menuOpen}
            onOpenChange={(o) => {
              setMenuOpen(o);
              if (!o) {
                // Radix hands focus back to the (hidden) button a tick later; put it on the row instead,
                // unless a dialog or an input has taken it in the meantime.
                setTimeout(() => {
                  const row = ref.current;
                  const a = document.activeElement;
                  if (row && (a === document.body || row.contains(a))) row.focus();
                }, 50);
              }
            }}
          >
            <MenuTrigger asChild>
              <RowButton label={menuLabel ?? "Options"}>
                <MoreHorizontal size={14} />
              </RowButton>
            </MenuTrigger>
            <MenuContent align="start">{menu}</MenuContent>
          </Menu>
        )}
        {quick && (
          <RowButton label={quick.label} onClick={quick.onClick}>
            <Plus size={14} />
          </RowButton>
        )}
      </span>
    </div>
  );
}

/** Drag by pointer anywhere on the row. No dnd-kit `attributes`: they would turn the wrapper into a focusable role="button" around the row. */
function SortableRow({ id, children }: { id: string; children: React.ReactNode }) {
  const { listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id });
  return (
    <div
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition, zIndex: isDragging ? 10 : undefined, opacity: isDragging ? 0.7 : 1 }}
      {...listeners}
    >
      {children}
    </div>
  );
}

/* ---------------- sections and pages ---------------- */

function SectionBlock({
  section,
  pos,
  siblings,
  tree,
  open,
  onToggle,
  run,
  onNewPage,
  sensors,
  activePageId,
}: {
  section: NotesTree["sections"][number];
  pos: Pos;
  siblings: string[];
  tree: NotesTree;
  open: boolean;
  onToggle: (force?: boolean) => void;
  run: Run;
  onNewPage: () => void;
  sensors: ReturnType<typeof useSensors>;
  activePageId: string | null;
}) {
  const deleteWithConfirm = useDeleteWithConfirm();
  const [renaming, setRenaming] = useState(false);
  const pages = tree.pages.filter((p) => p.sectionId === section.id && p.trashedAt === null);
  const known = new Set(pages.map((p) => p.id));
  const roots = pages.filter((p) => !p.parent || !known.has(p.parent));
  const rootIds = roots.map((p) => p.id);
  const rowKey = `sec:${section.id}`;
  // Only the section's own row starts a drag, so dragging its pages doesn't move the section.
  const { listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: section.id });

  return (
    <div
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition, zIndex: isDragging ? 10 : undefined, opacity: isDragging ? 0.7 : 1 }}
    >
      <div {...listeners}>
      {renaming ? (
        <form
          className="py-0.5 pl-6 pr-1"
          onSubmit={(e) => {
            e.preventDefault();
            const name = (new FormData(e.currentTarget).get("name") as string).trim();
            setRenaming(false);
            focusRow(rowKey);
            if (name) run(() => api.updateSection(section.id, name, section.color));
          }}
        >
          <Input
            name="name"
            aria-label="Section name"
            defaultValue={section.name}
            autoFocus
            className="h-8 text-sm"
            onBlur={() => setRenaming(false)}
            onKeyDown={(e) => {
              if (e.key === "Escape") {
                setRenaming(false);
                focusRow(rowKey);
              }
            }}
            onFocus={(e) => e.currentTarget.select()}
          />
        </form>
      ) : (
        <Row
          rowKey={rowKey}
          depth={1}
          pos={pos}
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
          menuLabel="Section options"
          quick={{ label: "New page in this section", onClick: onNewPage }}
          menu={
            <>
              <MenuItem icon={<Plus size={14} />} onSelect={onNewPage}>
                New page
              </MenuItem>
              <MenuItem icon={<Pencil size={14} />} onSelect={() => setRenaming(true)}>
                Rename
              </MenuItem>
              {pos.i > 0 && (
                <MenuItem icon={<ArrowUp size={14} />} onSelect={() => moveBy(run, siblings, section.id, -1)}>
                  Move up
                </MenuItem>
              )}
              {pos.i < siblings.length - 1 && (
                <MenuItem icon={<ArrowDown size={14} />} onSelect={() => moveBy(run, siblings, section.id, 1)}>
                  Move down
                </MenuItem>
              )}
              <MenuSeparator />
              <MenuItem
                danger
                icon={<Trash2 size={14} />}
                onSelect={async () => {
                  const ok = await deleteWithConfirm({
                    rowKey,
                    title: `Delete section "${section.name}"?`,
                    body: `${contents(tree, [section.id], false)} ${DELETE_NOTE}`,
                    confirm: "Delete section",
                    name: section.name,
                    del: () => api.deleteSection(section.id),
                  });
                  const st = useApp.getState();
                  if (ok && st.activePageId && !st.tree.pages.some((p) => p.id === st.activePageId)) st.openPage(null);
                }}
              >
                Delete section
              </MenuItem>
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
              accessibility={dndA11y}
              collisionDetection={closestCenter}
              onDragEnd={(e: DragEndEvent) => {
                if (!e.over || e.active.id === e.over.id) return;
                if (!rootIds.includes(String(e.active.id))) return;
                run(() => api.reorder(arrayMove(rootIds, rootIds.indexOf(String(e.active.id)), rootIds.indexOf(String(e.over!.id)))));
              }}
            >
              <SortableContext items={rootIds} strategy={verticalListSortingStrategy}>
                {roots.map((p, i) => (
                  <SortableRow key={p.id} id={p.id}>
                    <PageBranch page={p} depth={2} pos={{ i, n: roots.length }} siblings={rootIds} pages={pages} tree={tree} activePageId={activePageId} run={run} />
                  </SortableRow>
                ))}
              </SortableContext>
            </DndContext>
            {pages.length === 0 && (
              <Row rowKey={`add:${section.id}`} depth={2} pos={{ i: 0, n: 1 }} onClick={onNewPage} icon={<Plus size={14} className="text-faint" />} label="Add a page" active={false} muted />
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
  pos,
  siblings,
  pages,
  tree,
  activePageId,
  run,
}: {
  page: PageMeta;
  depth: number;
  pos: Pos;
  siblings: string[];
  pages: PageMeta[];
  tree: NotesTree;
  activePageId: string | null;
  run: Run;
}) {
  const kids = pages.filter((p) => p.parent === page.id);
  const kidIds = kids.map((k) => k.id);
  const containsActive = (id: string): boolean => pages.some((p) => p.parent === id && (p.id === activePageId || containsActive(p.id)));
  const [open, setOpen] = useState(() => containsActive(page.id));
  useEffect(() => {
    if (activePageId && containsActive(page.id)) setOpen(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activePageId]);
  return (
    <>
      <PageItem
        rowKey={`p:${page.id}`}
        page={page}
        depth={depth}
        pos={pos}
        siblings={siblings}
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
      {open &&
        kids.map((k, i) => (
          <PageBranch key={k.id} page={k} depth={depth + 1} pos={{ i, n: kids.length }} siblings={kidIds} pages={pages} tree={tree} activePageId={activePageId} run={run} />
        ))}
    </>
  );
}

function PageItem({
  rowKey,
  page,
  depth,
  pos,
  siblings,
  active,
  run,
  sections,
  onOpen,
  childCount = 0,
  open,
  onToggle,
  onAddSub,
}: {
  rowKey: string;
  page: PageMeta;
  depth: number;
  pos: Pos;
  /** Ids of the pages next to this one, in order. Leave out where moving isn't offered (Favorites). */
  siblings?: string[];
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
      rowKey={rowKey}
      depth={depth}
      pos={pos}
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
      menuLabel="Page options"
      quick={onAddSub ? { label: "Add a page inside", onClick: onAddSub } : undefined}
      menu={
        <>
          {onAddSub && (
            <MenuItem icon={<Plus size={14} />} onSelect={onAddSub}>
              Add a page inside
            </MenuItem>
          )}
          <MenuItem icon={<Star size={14} />} onSelect={() => run(() => api.setPageFavorite(page.id, !page.favorite))}>
            {page.favorite ? "Remove from favorites" : "Add to favorites"}
          </MenuItem>
          {siblings && siblings.length > 1 && pos.i > 0 && (
            <MenuItem icon={<ArrowUp size={14} />} onSelect={() => moveBy(run, siblings, page.id, -1)}>
              Move up
            </MenuItem>
          )}
          {siblings && pos.i < siblings.length - 1 && (
            <MenuItem icon={<ArrowDown size={14} />} onSelect={() => moveBy(run, siblings, page.id, 1)}>
              Move down
            </MenuItem>
          )}
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
        </>
      }
    />
  );
}
