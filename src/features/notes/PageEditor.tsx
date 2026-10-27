import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { EditorContent, useEditor, type Editor } from "@tiptap/react";
import { Check, ChevronRight, Loader2, MoreHorizontal, Star, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { registerFlush, trackSave } from "@/lib/flush";
import { api, errorText } from "@/lib/ipc";
import type { Page } from "@/lib/types";
import { cn, debounce, formatDate } from "@/lib/utils";
import { useApp } from "@/store/app";
import { Menu, MenuContent, MenuItem, MenuSeparator, MenuTrigger } from "@/components/ui";
import BlockHandle from "./editor/BlockHandle";
import BubbleToolbar from "./editor/BubbleToolbar";
import Ribbon from "./editor/Ribbon";
import SlashMenu, { type SlashController } from "./editor/SlashMenu";
import type { SlashState } from "./editor/slash";
import { editorProps, pageExtensions, parseContent } from "./editor/extensions";
import { PageCover, PageIconAndActions } from "./PageHeader";

type SaveState = "saved" | "saving" | "error";

export default function PageEditor({ pageId }: { pageId: string }) {
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
  return <LoadedEditor key={page.id} page={page} />;
}

function LoadedEditor({ page }: { page: Page }) {
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
  const inflightRef = useRef<Promise<void> | null>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const slashController = useRef<SlashController>({ state: null, onKey: () => false });
  const tree = useApp((s) => s.tree);
  const setTree = useApp((s) => s.setTree);
  const securityOn = useApp((s) => s.status?.securityOn ?? false);
  const readOnly = page.trashedAt !== null;

  // Saves run one after another, so an older save can never land after a newer one.
  const doSave = useCallback(() => {
    const editor = editorRef.current;
    if (!editor) return Promise.resolve();
    // Take the content now: on unmount the editor is gone by the time a queued save runs.
    const title = titleRef.current;
    const content = JSON.stringify(editor.getJSON());
    const text = editor.getText();
    dirtyRef.current = false;
    const run = async () => {
      try {
        const meta = await api.savePage(page.id, title, content, text);
        const t = useApp.getState().tree;
        setTree({ ...t, pages: t.pages.map((p) => (p.id === meta.id ? meta : p)) });
        setUpdated(meta.updated);
        setWords(meta.words);
        if (!dirtyRef.current) setSaveState("saved");
      } catch (e) {
        dirtyRef.current = true; // keep the edit so the next save retries it
        setSaveState("error");
        toast.error(`Couldn't save: ${errorText(e)}`);
      }
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
    setSaveState("saving");
    save();
  }, [save]);

  const extensions = useMemo(
    () => pageExtensions({ onState: setSlash, onKey: (e) => slashController.current.onKey(e) }),
    [],
  );

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

  // Flush any pending save when leaving the page.
  useEffect(
    () => () => {
      save.cancel();
      if (editorRef.current && dirtyRef.current) void doSave();
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  const section = tree.sections.find((s) => s.id === page.sectionId);
  const notebook = tree.notebooks.find((n) => n.id === section?.notebookId);
  const meta = tree.pages.find((p) => p.id === page.id);
  // The pages this one sits inside, outermost first.
  const parents: typeof tree.pages = [];
  for (let id = meta?.parent; id && parents.length < 8; ) {
    const pp = tree.pages.find((x) => x.id === id);
    if (!pp) break;
    parents.unshift(pp);
    id = pp.parent;
  }

  const setLook = async (nextIcon: string, nextCover: string) => {
    setIcon(nextIcon);
    setCover(nextCover);
    try {
      const m = await api.setPageLook(page.id, nextIcon, nextCover);
      const t = useApp.getState().tree;
      setTree({ ...t, pages: t.pages.map((p) => (p.id === m.id ? m : p)) });
    } catch (e) {
      toast.error(errorText(e));
    }
  };
  const toggleFavorite = async () => {
    setTree(await api.setPageFavorite(page.id, !meta?.favorite));
  };
  const trash = async () => {
    save.cancel();
    setTree(await api.trashPage(page.id));
    useApp.getState().openPage(null);
    toast("Moved to trash", {
      action: { label: "Undo", onClick: async () => setTree(await api.restorePage(page.id)) },
    });
  };
  const moveTo = async (sectionId: string) => {
    setTree(await api.movePage(page.id, sectionId));
    toast.success("Page moved.");
  };

  if (!editor) return null;

  return (
    <div className="flex h-full flex-col">
      <div className="flex h-11 shrink-0 items-center gap-3 px-4">
        <nav className="flex min-w-0 items-center gap-1 text-[0.8rem] text-muted">
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
          {parents.map((pp) => (
            <span key={pp.id} className="flex items-center gap-1">
              <button onClick={() => useApp.getState().openPage(pp.id)} className="max-w-[10rem] truncate hover:text-fg hover:underline">
                {pp.icon && <span className="mr-1">{pp.icon}</span>}
                {pp.title || "Untitled"}
              </button>
              <ChevronRight size={12} className="shrink-0 text-faint" />
            </span>
          ))}
          <span className="max-w-[16rem] truncate text-fg">
            {icon && <span className="emoji-mono mr-1">{icon}</span>}
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
              className={cn("rounded-lg p-1.5 transition-colors hover:bg-surface-2", meta?.favorite ? "text-fg" : "text-faint hover:text-fg")}
              aria-label="Favorite"
              title="Favorite"
            >
              <Star size={16} fill={meta?.favorite ? "currentColor" : "none"} />
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
                <div className="px-2.5 py-1.5 text-xs text-faint">Move to section</div>
                {tree.sections
                  .filter((s) => s.id !== page.sectionId)
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
                <MenuItem danger icon={<Trash2 size={15} />} onSelect={trash}>
                  Move to trash
                </MenuItem>
              </MenuContent>
            </Menu>
          </div>
        )}
      </div>

      {!readOnly && <Ribbon editor={editor} />}

      <div className="min-h-0 flex-1 overflow-y-auto">
        <PageCover cover={cover} readOnly={readOnly} onChange={(c) => setLook(icon, c)} />
        <div className={cn("mx-auto px-16 pb-10", cover ? "pt-0" : "pt-10")} style={{ maxWidth: "calc(var(--editor-width) + 128px)" }}>
          {readOnly && (
            <div className="mb-6 flex items-center justify-between rounded-lg border border-warn/30 bg-warn/10 px-4 py-3 text-sm">
              <span>This page is in the trash.</span>
              <button className="font-medium underline-offset-2 hover:underline" onClick={async () => setTree(await api.restorePage(page.id))}>
                Restore
              </button>
            </div>
          )}
          <PageIconAndActions icon={icon} cover={cover} readOnly={readOnly} onChange={setLook} />
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
            className="mb-4 w-full min-w-0 bg-transparent text-[2.6rem] font-bold leading-tight tracking-tight text-fg outline-none placeholder:text-faint/70"
          />

          <div ref={wrapRef} className="relative -ml-16 pl-16">
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
      <SlashMenu editor={editor} slash={slash} controller={slashController} />
    </div>
  );
}
