import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { EditorContent, useEditor, type Editor } from "@tiptap/react";
import { BubbleMenu } from "@tiptap/react/menus";
import { Bold, Check, Code, Highlighter, Italic, Loader2, MoreHorizontal, Star, Strikethrough, Trash2, Underline } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { Page } from "@/lib/types";
import { cn, debounce, formatDate } from "@/lib/utils";
import { useApp } from "@/store/app";
import { Menu, MenuContent, MenuItem, MenuSeparator, MenuTrigger } from "@/components/ui";
import Toolbar from "./editor/Toolbar";
import { editorProps, pageExtensions, parseContent } from "./editor/extensions";

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
  const [saveState, setSaveState] = useState<SaveState>("saved");
  const [updated, setUpdated] = useState(page.updated);
  const [words, setWords] = useState(page.text.split(/\s+/).filter(Boolean).length);
  const titleRef = useRef(page.title);
  const editorRef = useRef<Editor | null>(null);
  const dirtyRef = useRef(false);
  const tree = useApp((s) => s.tree);
  const setTree = useApp((s) => s.setTree);
  const readOnly = page.trashedAt !== null;

  const save = useMemo(
    () =>
      debounce(async () => {
        const editor = editorRef.current;
        if (!editor) return;
        dirtyRef.current = false;
        try {
          const meta = await api.savePage(page.id, titleRef.current, JSON.stringify(editor.getJSON()), editor.getText());
          const t = useApp.getState().tree;
          setTree({ ...t, pages: t.pages.map((p) => (p.id === meta.id ? meta : p)) });
          setUpdated(meta.updated);
          setWords(meta.words);
          setSaveState("saved");
        } catch (e) {
          setSaveState("error");
          toast.error(`Couldn't save: ${errorText(e)}`);
        }
      }, 500),
    [page.id, setTree],
  );

  const markDirty = useCallback(() => {
    dirtyRef.current = true;
    setSaveState("saving");
    save();
  }, [save]);

  const editor = useEditor({
    extensions: pageExtensions("Start writing… use the toolbar, Markdown shortcuts like # or -, or paste an image."),
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
      if (editorRef.current && dirtyRef.current) {
        const e = editorRef.current;
        api.savePage(page.id, titleRef.current, JSON.stringify(e.getJSON()), e.getText()).catch(() => {});
      }
    },
    [],
  );

  const sectionName = tree.sections.find((s) => s.id === page.sectionId)?.name ?? "";
  const meta = tree.pages.find((p) => p.id === page.id);

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
      {!readOnly && (
        <div className="flex justify-center px-6 pt-4">
          <Toolbar editor={editor} />
        </div>
      )}

      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto px-10 pb-32 pt-10" style={{ maxWidth: "calc(var(--editor-width) + 80px)" }}>
          {readOnly && (
            <div className="mb-6 flex items-center justify-between rounded-xl border border-warn/30 bg-warn/10 px-4 py-3 text-sm">
              <span>This page is in the trash.</span>
              <button
                className="font-medium text-accent hover:underline"
                onClick={async () => setTree(await api.restorePage(page.id))}
              >
                Restore
              </button>
            </div>
          )}
          <div className="mb-2 flex items-center gap-2 text-xs text-faint">
            <span>{sectionName}</span>
            <span>·</span>
            <span>Edited {formatDate(updated)}</span>
          </div>
          <div className="flex items-start gap-3">
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
              placeholder="Untitled page"
              className="min-w-0 flex-1 bg-transparent text-[2.2rem] font-bold leading-tight tracking-tight text-fg outline-none placeholder:text-faint"
            />
            {!readOnly && (
              <div className="flex items-center gap-1 pt-3">
                <button
                  onClick={toggleFavorite}
                  className={cn(
                    "rounded-lg p-2 transition-colors hover:bg-surface-2",
                    meta?.favorite ? "text-warn" : "text-faint hover:text-fg",
                  )}
                  aria-label="Favorite"
                  title="Favorite"
                >
                  <Star size={17} fill={meta?.favorite ? "currentColor" : "none"} />
                </button>
                <Menu>
                  <MenuTrigger asChild>
                    <button className="rounded-lg p-2 text-faint hover:bg-surface-2 hover:text-fg" aria-label="Page options">
                      <MoreHorizontal size={17} />
                    </button>
                  </MenuTrigger>
                  <MenuContent>
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

          <div className="mt-6">
            <BubbleMenu
              editor={editor}
              className="flex items-center gap-0.5 rounded-xl border border-line bg-elevated p-1 shadow-float"
            >
              <BubbleButton active={editor.isActive("bold")} onClick={() => editor.chain().focus().toggleBold().run()}>
                <Bold size={14} />
              </BubbleButton>
              <BubbleButton active={editor.isActive("italic")} onClick={() => editor.chain().focus().toggleItalic().run()}>
                <Italic size={14} />
              </BubbleButton>
              <BubbleButton active={editor.isActive("underline")} onClick={() => editor.chain().focus().toggleUnderline().run()}>
                <Underline size={14} />
              </BubbleButton>
              <BubbleButton active={editor.isActive("strike")} onClick={() => editor.chain().focus().toggleStrike().run()}>
                <Strikethrough size={14} />
              </BubbleButton>
              <BubbleButton active={editor.isActive("code")} onClick={() => editor.chain().focus().toggleCode().run()}>
                <Code size={14} />
              </BubbleButton>
              <BubbleButton
                active={editor.isActive("highlight")}
                onClick={() => editor.chain().focus().toggleHighlight({ color: "#fde68a" }).run()}
              >
                <Highlighter size={14} />
              </BubbleButton>
            </BubbleMenu>
            <EditorContent editor={editor} />
          </div>
        </div>
      </div>

      <footer className="flex h-9 shrink-0 items-center justify-between border-t border-line px-5 text-xs text-faint">
        <span>
          {words} {words === 1 ? "word" : "words"}
        </span>
        <span className="flex items-center gap-1.5">
          {saveState === "saving" && (
            <>
              <Loader2 size={12} className="animate-spin" /> Encrypting…
            </>
          )}
          {saveState === "saved" && (
            <>
              <Check size={12} className="text-ok" /> Saved & encrypted
            </>
          )}
          {saveState === "error" && <span className="text-danger">Not saved</span>}
        </span>
      </footer>
    </div>
  );
}

function BubbleButton({ active, onClick, children }: { active: boolean; onClick: () => void; children: React.ReactNode }) {
  return (
    <button
      onMouseDown={(e) => e.preventDefault()}
      onClick={onClick}
      className={cn(
        "grid h-7 w-7 place-items-center rounded-lg transition-colors",
        active ? "bg-accent-soft text-accent" : "text-muted hover:bg-surface-2 hover:text-fg",
      )}
    >
      {children}
    </button>
  );
}
