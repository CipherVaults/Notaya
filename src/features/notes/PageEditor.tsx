import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { EditorContent, Extension, useEditor, type Editor } from "@tiptap/react";
import { Plugin } from "@tiptap/pm/state";
import { Check, ChevronRight, Download, LayoutTemplate, Loader2, MoreHorizontal, Star, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { registerFlush, trackSave } from "@/lib/flush";
import { api, errorText } from "@/lib/ipc";
import { watchItemChanges, type Revisioned } from "@/lib/changes";
import type { Page, PageMeta } from "@/lib/types";
import { cn, debounce, formatDate } from "@/lib/utils";
import { findPageMeta, useApp } from "@/store/app";
import { Menu, MenuContent, MenuItem, MenuSeparator, MenuTrigger, useConfirm } from "@/components/ui";
import BlockHandle from "./editor/BlockHandle";
import BubbleToolbar from "./editor/BubbleToolbar";
import NoteFind, { clearNoteFind, NoteFindExtension } from "./editor/NoteFind";
import Ribbon from "./editor/Ribbon";
import InternalLinkHost from "./editor/InternalLinkHost";
import SlashMenu, { type SlashController } from "./editor/SlashMenu";
import type { SlashState } from "./editor/slash";
import { editorProps, pageExtensions, parseContent } from "./editor/extensions";
import { setPageEditable } from "./editor/editability";
import { restoreFromBin } from "./restore";
import { PageCover, PageIconAndActions, HistoryMenuItem } from "./PageHeader";
import { PageIcon } from "@/lib/pageLook";
import { openExport } from "./ExportDialog";
import HistoryDialog from "./HistoryDialog";
import RowProperties from "@/features/database/RowProperties";
import RowChildPages from "./RowChildPages";
import { loadRow, saveRowAsTemplate } from "@/features/database/rowOps";

type SaveState = "saved" | "saving" | "error";

function isFindShortcut(e: KeyboardEvent) {
  return !e.defaultPrevented && e.ctrlKey && !e.altKey && e.key.toLowerCase() === "f";
}

function isActiveNote(id: string) {
  const { view, showTrash, activePageId, activeKind } = useApp.getState();
  return view === "notes" && !showTrash && activePageId === id && activeKind !== "database";
}

function updatePageMeta(meta: PageMeta) {
  const { tree, setTree } = useApp.getState();
  const update = (p: PageMeta) => p.id === meta.id ? meta : p;
  setTree({ ...tree, pages: tree.pages.map(update), rowPages: tree.rowPages.map(update) });
}

/** `embedded`: shown in a peek (no breadcrumb, no ribbon). Rows of a database open here too (their `parent` is the database). */
export default function PageEditor({ pageId, embedded = false, onOpenChild, onBodyEscape }: {
  pageId: string;
  embedded?: boolean;
  onOpenChild?: (id: string) => void;
  onBodyEscape?: () => void;
}) {
  const [page, setPage] = useState<Page | null>(null);

  useEffect(() => {
    let alive = true;
    setPage(null);
    api
      .getPage(pageId)
      .then((p) => alive && setPage(p))
      .catch((e) => toast.error(errorText(e)));
    return () => {
      alive = false;
    };
  }, [pageId]);

  if (!page) {
    return (
      <div className="grid h-full place-items-center text-muted">
        <Loader2 className="animate-spin" size={20} />
      </div>
    );
  }
  // Remount per page so editor history never leaks between pages.
  return <LoadedEditor key={page.id} page={page} embedded={embedded} onOpenChild={onOpenChild} onBodyEscape={onBodyEscape} />;
}

function LoadedEditor({ page, embedded, onOpenChild, onBodyEscape }: {
  page: Page;
  embedded: boolean;
  onOpenChild?: (id: string) => void;
  onBodyEscape?: () => void;
}) {
  const confirm = useConfirm();
  const [historyOpen, setHistoryOpen] = useState(false);
  const [findOpen, setFindOpen] = useState(false);
  const findInput = useRef<HTMLInputElement>(null);
  const focusBeforeFind = useRef<HTMLElement | null>(null);
  const [title, setTitle] = useState(page.title);
  const [icon, setIcon] = useState(page.icon ?? "");
  const [cover, setCover] = useState(page.cover ?? "");
  const [saveState, setSaveState] = useState<SaveState>("saved");
  const [updated, setUpdated] = useState(page.updated);
  const [words, setWords] = useState(page.text.split(/\s+/).filter(Boolean).length);
  const [slash, setSlash] = useState<SlashState | null>(null);
  const titleRef = useRef(page.title);
  const editorRef = useRef<Editor | null>(null);
  const dirtyRef = useRef(false);
  const savingRef = useRef(0);
  const recheckRef = useRef<(() => void) | null>(null);
  const revRef = useRef((page as Revisioned<Page>).rev ?? 0);
  const inflightRef = useRef<Promise<void> | null>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const slashController = useRef<SlashController>({ state: null, onKey: () => false });
  const tree = useApp((s) => s.tree);
  const setTree = useApp((s) => s.setTree);
  const securityOn = useApp((s) => s.status?.securityOn ?? false);
  const [trashedAt, setTrashedAt] = useState(page.trashedAt);
  const readOnly = trashedAt != null;
  const [rowFavorite, setRowFavorite] = useState(page.favorite);

  // Saves run one after another, so an older save can never land after a newer one.
  const doSave = useCallback(() => {
    const editor = editorRef.current;
    if (!editor) return Promise.resolve();
    // Take the content now: on unmount the editor is gone by the time a queued save runs.
    const title = titleRef.current;
    const content = JSON.stringify(editor.getJSON());
    const text = editor.getText();
    dirtyRef.current = false;
    savingRef.current++;
    const run = async () => {
      try {
        const meta = await api.savePage(page.id, title, content, text, revRef.current);
        revRef.current = meta.rev;
        setTrashedAt(meta.trashedAt);
        if (editorRef.current && !editorRef.current.isDestroyed) setPageEditable(editorRef.current, meta.trashedAt == null);
        if (!dirtyRef.current && savingRef.current === 1) void api.editing(page.id, false).catch(() => {});
        updatePageMeta(meta);
        setUpdated(meta.updated);
        setWords(meta.words);
        if (!dirtyRef.current) setSaveState("saved");
      } catch (e) {
        dirtyRef.current = true; // keep the edit so the next save retries it
        setSaveState("error");
        toast.error(`Couldn't save: ${errorText(e)}`);
      } finally { savingRef.current--; recheckRef.current?.(); }
    };
    const p = (inflightRef.current ?? Promise.resolve()).then(run);
    inflightRef.current = p;
    return trackSave(p); // a lock right after switching pages still waits for this save
  }, [page.id, setTree]);

  const save = useMemo(() => debounce(doSave, 500), [doSave]);

  // Writes the page now, for locking the vault or closing the app.
  const flushNow = useCallback(async () => {
    // A debounced save can start while we wait, so wait until the chain stops growing.
    let last: Promise<void> | null;
    do {
      last = inflightRef.current;
      await last;
    } while (inflightRef.current !== last);
    if (dirtyRef.current) {
      save.cancel();
      await doSave();
    }
    await inflightRef.current;
  }, [save, doSave]);

  useEffect(() => registerFlush(flushNow), [flushNow]);

  const markDirty = useCallback(() => {
    dirtyRef.current = true;
    void api.editing(page.id, true).catch(() => {});
    setSaveState("saving");
    save();
  }, [save, page.id]);

  const bodyEscape = useRef(onBodyEscape);
  useEffect(() => { bodyEscape.current = onBodyEscape; }, [onBodyEscape]);
  const hasBodyEscape = !!onBodyEscape;
  const extensions = useMemo(() => {
    const extensions = [...pageExtensions({ onState: setSlash, onKey: (e) => slashController.current.onKey(e) }), NoteFindExtension];
    if (hasBodyEscape) {
      // Run after editor commands and before ProseMirror's default Escape cancellation.
      extensions.push(Extension.create({
        name: "peekBodyEscape",
        priority: 1,
        addProseMirrorPlugins() {
          return [new Plugin({
            props: {
              handleKeyDown: (_view, e) => {
                if (e.key !== "Escape" || e.defaultPrevented || e.isComposing || !bodyEscape.current) return false;
                e.stopPropagation();
                bodyEscape.current();
                return true;
              },
            },
          })];
        },
      }));
    }
    return extensions;
  }, [hasBodyEscape]);

  const editor = useEditor({
    extensions,
    content: parseContent(page.content),
    editable: !readOnly,
    editorProps: {
      ...editorProps(() => editorRef.current),
      attributes: { class: "selectable", spellcheck: "true" },
    },
    onUpdate: markDirty,
  });
  editorRef.current = editor;

  const openFind = useCallback(() => {
    if (findInput.current) {
      findInput.current.focus();
      findInput.current.select();
    } else {
      const focused = document.activeElement;
      focusBeforeFind.current = focused instanceof HTMLElement && focused !== document.body ? focused : null;
      setFindOpen(true);
    }
  }, []);

  useEffect(() => {
    if (!editor || embedded) return;
    // A click in the note's empty space can leave focus on the document body.
    const onKey = (e: KeyboardEvent) => {
      if (e.target !== document.body || !isFindShortcut(e) || !isActiveNote(page.id)) return;
      e.preventDefault();
      openFind();
    };
    document.documentElement.addEventListener("keydown", onKey);
    return () => document.documentElement.removeEventListener("keydown", onKey);
  }, [editor, embedded, openFind, page.id]);

  useEffect(() => {
    const watcher = watchItemChanges(page.id, () => revRef.current,
      () => dirtyRef.current || savingRef.current > 0, async (alive) => {
      const fresh = await api.getPage(page.id);
      if (!alive() || dirtyRef.current || savingRef.current > 0 || fresh.rev < revRef.current) return;
      const e = editorRef.current;
      if (!e || e.isDestroyed) return;
      setTrashedAt(fresh.trashedAt);
      setPageEditable(e, fresh.trashedAt == null);
      const scroller = wrapRef.current?.closest<HTMLElement>(".overflow-y-auto");
      const scroll = scroller?.scrollTop ?? 0;
      const selection = e.state.selection;
      e.chain().setContent(parseContent(fresh.content), { emitUpdate: false }).setMeta("addToHistory", false).run();
      const max = e.state.doc.content.size;
      e.commands.setTextSelection({ from: Math.min(selection.from, max), to: Math.min(selection.to, max) });
      if (scroller) scroller.scrollTop = scroll;
      titleRef.current = fresh.title;
      setTitle(fresh.title); setIcon(fresh.icon); setCover(fresh.cover);
      setUpdated(fresh.updated); setWords(fresh.text.split(/\s+/).filter(Boolean).length);
      revRef.current = fresh.rev;
    });
    recheckRef.current = watcher.recheck;
    return () => { recheckRef.current = null; watcher.dispose(); };
  }, [page.id]);

  // Flush any pending save when leaving the page.
  useEffect(
    () => () => {
      save.cancel();
      if (editorRef.current && dirtyRef.current) void doSave();
      void trackSave(Promise.resolve(inflightRef.current).then(async () => {
        if (!dirtyRef.current) {
          await api.historySnapshot(page.id);
          await api.editing(page.id, false);
        }
      }).catch(() => {}));
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  const section = tree.sections.find((s) => s.id === page.sectionId);
  const notebook = tree.notebooks.find((n) => n.id === section?.notebookId);
  const meta = findPageMeta(tree, page.id);
  // A row of a database is a page that is not in the tree; its `parent` is the database.
  const rowDb = !meta ? tree.databases.find((d) => d.id === page.parent) : undefined;
  const isRow = !!rowDb;
  const isFavorite = isRow ? rowFavorite : meta?.favorite;
  // The pages this one sits inside, outermost first.
  const parents: typeof tree.pages = [];
  for (let id = meta?.parent; id && parents.length < 8; ) {
    const pp = findPageMeta(tree, id);
    if (!pp) break;
    parents.unshift(pp);
    id = pp.parent;
  }

  const setLook = async (nextIcon: string, nextCover: string) => {
    const [prevIcon, prevCover] = [icon, cover];
    setIcon(nextIcon);
    setCover(nextCover);
    try {
      const m = await api.setPageLook(page.id, nextIcon, nextCover);
      updatePageMeta(m);
    } catch (e) {
      setIcon(prevIcon);
      setCover(prevCover);
      toast.error(errorText(e));
    }
  };
  const toggleFavorite = async () => {
    if (isRow) {
      setRowFavorite(!rowFavorite);
      try {
        setTree(await api.setPageFavorite(page.id, !rowFavorite));
      } catch (e) {
        setRowFavorite(rowFavorite);
        toast.error(errorText(e));
      }
      return;
    }
    setTree(await api.setPageFavorite(page.id, !meta?.favorite));
  };
  const asTemplate = async () => {
    if (!rowDb || !editor) return;
    const r = await loadRow(rowDb.id, page.id);
    if (r) await saveRowAsTemplate(rowDb.id, r, titleRef.current, JSON.stringify(editor.getJSON()), editor.getText());
  };
  const restore = async () => {
    try {
      const restored = await restoreFromBin(page.id, confirm);
      if (restored) setTree(restored);
    } catch (error) { toast.error(errorText(error)); }
  };
  const trash = async () => {
    save.cancel();
    await flushNow();
    if (dirtyRef.current) return;
    setTree(await api.trashPage(page.id));
    if (isRow && !embedded) useApp.getState().openDatabase(page.parent);
    else if (!embedded) useApp.getState().openPage(null);
    toast("Moved to trash", {
      action: { label: "Undo", onClick: restore },
    });
  };
  const moveTo = async (sectionId: string) => {
    setTree(await api.movePage(page.id, sectionId));
    toast.success("Page moved.");
  };

  if (!editor) return null;

  return (
    <div className="flex h-full flex-col" data-note-editor onKeyDownCapture={(e) => {
      if (!isFindShortcut(e.nativeEvent) || (!embedded && !isActiveNote(page.id))) return;
      if (!(e.target instanceof Element) || e.target.closest("[data-note-editor], [data-database-view]") !== e.currentTarget) return;
      e.preventDefault();
      openFind();
    }}>
      <div className="flex h-11 shrink-0 items-center gap-3 px-4">
        <nav aria-label="Breadcrumb" className={cn("flex min-w-0 items-center gap-1 text-[0.8rem] text-muted", embedded && "hidden")}>
          {notebook && (
            <>
              <span className="emoji-mono shrink-0">{notebook.icon}</span>
              <span className="max-w-[10rem] truncate">{notebook.name}</span>
              <ChevronRight size={12} className="shrink-0 text-faint" />
            </>
          )}
          {section && (
            <>
              <span className="max-w-[10rem] truncate">{section.name}</span>
              <ChevronRight size={12} className="shrink-0 text-faint" />
            </>
          )}
          {rowDb && (
            <span className="flex items-center gap-1">
              <button onClick={() => useApp.getState().openDatabase(rowDb.id)} className="max-w-[10rem] truncate hover:text-fg hover:underline" title="Back to the database">
                {rowDb.icon && <span className="me-1">{rowDb.icon}</span>}
                {rowDb.title || "Untitled database"}
              </button>
              <ChevronRight size={12} className="shrink-0 text-faint" />
            </span>
          )}
          {parents.map((pp) => (
            <span key={pp.id} className="flex items-center gap-1">
              <button onClick={() => useApp.getState().openPage(pp.id)} className="max-w-[10rem] truncate hover:text-fg hover:underline">
                {pp.icon && <PageIcon icon={pp.icon} size={14} className="me-1" />}
                {pp.title || "Untitled"}
              </button>
              <ChevronRight size={12} className="shrink-0 text-faint" />
            </span>
          ))}
          <span dir="auto" className="max-w-[16rem] truncate text-fg">
            {icon && <PageIcon icon={icon} size={14} className="me-1" />}
            {title || "Untitled"}
          </span>
        </nav>
        <div className="flex-1" />
        <span className="flex items-center gap-1.5 text-xs text-faint" title={`Edited ${formatDate(updated)} · ${words} words`}>
          {saveState === "saving" && (
            <>
              <Loader2 size={12} className="animate-spin" /> {securityOn ? "Encrypting…" : "Saving…"}
            </>
          )}
          {saveState === "saved" && (
            <>
              <Check size={12} /> {securityOn ? "Saved & encrypted" : "Saved on this drive"}
            </>
          )}
          {saveState === "error" && <span className="text-danger">Not saved</span>}
        </span>
        {!readOnly && (
          <div className="flex items-center gap-0.5">
            <button
              onClick={toggleFavorite}
              className={cn("rounded-lg p-1.5 transition-colors hover:bg-surface-2", isFavorite ? "text-fg" : "text-faint hover:text-fg")}
              aria-label="Favorite"
              title="Favorite"
            >
              <Star size={16} fill={isFavorite ? "currentColor" : "none"} />
            </button>
            <Menu>
              <MenuTrigger asChild>
                <button className="rounded-lg p-1.5 text-faint hover:bg-surface-2 hover:text-fg" aria-label="Page options">
                  <MoreHorizontal size={16} />
                </button>
              </MenuTrigger>
              <MenuContent>
                <div className="px-2.5 py-1.5 text-xs text-faint">
                  {words} {words === 1 ? "word" : "words"} · edited {formatDate(updated)}
                </div>
                <MenuSeparator />
                <HistoryMenuItem onSelect={() => setHistoryOpen(true)} />
                {isRow && (
                  <MenuItem icon={<LayoutTemplate size={15} />} onSelect={asTemplate}>
                    Save as template
                  </MenuItem>
                )}
                {!isRow && <div className="px-2.5 py-1.5 text-xs text-faint">Move to section</div>}
                {tree.sections
                  .filter((s) => !isRow && s.id !== page.sectionId)
                  .map((s) => {
                    const nb = tree.notebooks.find((n) => n.id === s.notebookId);
                    return (
                      <MenuItem key={s.id} onSelect={() => moveTo(s.id)}>
                        <span className="truncate">
                          {nb?.icon} {nb?.name} / {s.name}
                        </span>
                      </MenuItem>
                    );
                  })}
                <MenuSeparator />
                {!isRow && (
                  <MenuItem icon={<Download size={15} />} onSelect={() => openExport("page", page.id)}>
                    Export…
                  </MenuItem>
                )}
                <MenuItem danger icon={<Trash2 size={15} />} onSelect={trash}>
                  Move to bin
                </MenuItem>
              </MenuContent>
            </Menu>
          </div>
        )}
      </div>

      {!readOnly && !embedded && <Ribbon editor={editor} />}
      {findOpen && <NoteFind editor={editor} inputRef={findInput} onClose={(restoreFocus) => {
        clearNoteFind(editor);
        setFindOpen(false);
        const focused = focusBeforeFind.current;
        focusBeforeFind.current = null;
        if (restoreFocus) {
          (focused?.isConnected ? focused : editor.view.dom).focus({ preventScroll: true });
        }
      }} />}

      <div className="min-h-0 flex-1 overflow-y-auto">
        <PageCover cover={cover} readOnly={readOnly} onChange={(c) => setLook(icon, c)} />
        <div className={cn("mx-auto pb-10", embedded ? "px-12" : "px-16", cover ? (icon ? "pt-0" : "pt-4") : embedded ? "pt-6" : "pt-10")} style={{ maxWidth: "calc(var(--editor-width) + 128px)" }}>
          {readOnly && (
            <div className="mb-6 flex items-center justify-between rounded-lg border border-warn/30 bg-warn/10 px-4 py-3 text-sm">
              <span>This page is in the trash.</span>
              <button className="font-medium underline-offset-2 hover:underline" onClick={restore}>
                Restore
              </button>
            </div>
          )}
          <div className="group/head">
          <PageIconAndActions icon={icon} cover={cover} readOnly={readOnly} allowPicture={!isRow} onChange={setLook} />
          <input
            value={title}
            readOnly={readOnly}
            onChange={(e) => {
              setTitle(e.target.value);
              titleRef.current = e.target.value;
              markDirty();
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === "ArrowDown") {
                e.preventDefault();
                editor.commands.focus("start");
              }
            }}
            placeholder="Untitled"
            dir="auto"
            className={cn("mb-4 w-full min-w-0 bg-transparent font-bold leading-tight tracking-tight text-fg outline-none placeholder:text-faint/70", embedded ? "text-[2rem]" : "text-[2.6rem]")}
          />
          </div>
          {isRow && rowDb && <div data-database={rowDb.id}><RowProperties dbId={rowDb.id} rowId={page.id} title={title} readOnly={readOnly} /></div>}
          {!readOnly && <RowChildPages parentId={page.id} pages={tree.rowPages} onOpen={onOpenChild ?? useApp.getState().openPage} />}

          <div ref={wrapRef} className={cn("relative", embedded ? "-ml-12 pl-12" : "-ml-16 pl-16")}>
            <BlockHandle editor={editor} container={wrapRef} />
            <EditorContent editor={editor} />
          </div>
          {/* Clicking the empty space under the text puts the cursor at the end. */}
          <div
            className="h-[32vh] cursor-text"
            onClick={() => {
              if (readOnly) return;
              // Clicking below the last block starts a new line, unless the last line is already empty.
              const last = editor.state.doc.lastChild;
              if (last && !(last.type.name === "paragraph" && last.content.size === 0)) {
                editor.chain().insertContentAt(editor.state.doc.content.size, { type: "paragraph" }).focus("end").run();
              } else {
                editor.chain().focus("end").run();
              }
            }}
          />
        </div>
      </div>

      <BubbleToolbar editor={editor} />
      <SlashMenu editor={editor} slash={slash} controller={slashController} page={{ id: page.id, sectionId: page.sectionId }} />
      <InternalLinkHost editor={editor} pageId={page.id} />
      <HistoryDialog pageId={page.id} open={historyOpen} onOpenChange={setHistoryOpen} />
    </div>
  );
}
