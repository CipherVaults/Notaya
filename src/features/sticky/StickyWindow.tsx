import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { EditorContent, useEditor, type Editor } from "@tiptap/react";
import { Popover } from "radix-ui";
import { Bold, Calculator, Italic, ListChecks, Pin, PinOff, Trash2, X } from "lucide-react";
import { flushAll, registerFlush, trackSave } from "@/lib/flush";
import { api } from "@/lib/ipc";
import { STICKY_COLORS, stickyColor } from "@/lib/palette";
import { applyAppearance, cachedAppearance } from "@/lib/theme";
import type { Sticky } from "@/lib/types";
import { cn, debounce } from "@/lib/utils";
import { editorProps, parseContent, stickyExtensions } from "@/features/notes/editor/extensions";
import MiniCalculator from "./MiniCalculator";

const win = getCurrentWindow();

/** A sticky note floating on the desktop in its own frameless window. */
export default function StickyWindow({ id }: { id: string }) {
  const [sticky, setSticky] = useState<Sticky | null>(null);

  useEffect(() => {
    applyAppearance(cachedAppearance(), false);
    api.getSticky(id).then(setSticky).catch(() => win.close());
  }, [id]);

  // Rust waits for every window to write its pending saves before it locks the vault.
  useEffect(() => {
    const unlisten = listen<number>("vault-locking", async (event) => {
      try {
        await flushAll();
      } finally {
        await api.flushDone(event.payload).catch(() => {});
      }
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  if (!sticky) return null;
  return <StickyCard sticky={sticky} onChange={setSticky} />;
}

function StickyCard({ sticky, onChange }: { sticky: Sticky; onChange: (s: Sticky) => void }) {
  const color = stickyColor(sticky.color);
  const editorRef = useRef<Editor | null>(null);
  const [focused, setFocused] = useState(false);
  const [calcOpen, setCalcOpen] = useState(false);
  const [saveFailed, setSaveFailed] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const dirtyRef = useRef(false);
  const inflightRef = useRef<Promise<void> | null>(null);

  // Saves run one after another. The returned promise rejects if the save fails.
  const doSave = useCallback((): Promise<void> => {
    const e = editorRef.current;
    if (!e) return Promise.resolve();
    const content = JSON.stringify(e.getJSON());
    const text = e.getText();
    dirtyRef.current = false;
    const run = async () => {
      try {
        await api.saveSticky(sticky.id, content, text);
        if (!dirtyRef.current) setSaveFailed(false);
      } catch (err) {
        dirtyRef.current = true; // keep the edit so the next save retries it
        setSaveFailed(true);
        throw err;
      }
    };
    const p = (inflightRef.current ?? Promise.resolve()).then(run);
    inflightRef.current = p.catch(() => {});
    return trackSave(p);
  }, [sticky.id]);

  const save = useMemo(() => debounce(() => doSave().catch(() => {}), 400), [doSave]);

  // Writes the note now. Rejects if it couldn't be saved.
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

  useEffect(() => {
    if (!confirmDelete) return;
    const t = setTimeout(() => setConfirmDelete(false), 3000);
    return () => clearTimeout(t);
  }, [confirmDelete]);

  const editor = useEditor({
    extensions: stickyExtensions(),
    content: parseContent(sticky.content),
    editorProps: { ...editorProps(() => editorRef.current), attributes: { class: "selectable" } },
    onUpdate: () => {
      dirtyRef.current = true;
      save();
    },
    onFocus: () => setFocused(true),
    onBlur: () => setFocused(false),
  });
  editorRef.current = editor;

  // Persist position and size (in logical pixels) whenever the note is moved or resized.
  useEffect(() => {
    const persist = debounce(async () => {
      const [pos, size, scale] = await Promise.all([win.outerPosition(), win.innerSize(), win.scaleFactor()]);
      const p = pos.toLogical(scale);
      const s = size.toLogical(scale);
      await api.setStickyGeometry(sticky.id, p.x, p.y, s.width, s.height).catch(() => {});
    }, 500);
    const unregister = registerFlush(() => persist.flush());
    const a = win.onMoved(() => persist());
    const b = win.onResized(() => persist());
    requestAnimationFrame(() => requestAnimationFrame(() => api.windowReady()));
    return () => {
      unregister();
      a.then((f) => f());
      b.then((f) => f());
    };
  }, [sticky.id]);

  const style = async (patch: Partial<Pick<Sticky, "color" | "onTop">>) => {
    const next = await api.setStickyStyle(sticky.id, patch.color ?? sticky.color, patch.onTop ?? sticky.onTop);
    onChange(next);
  };

  return (
    <div className="h-full p-2.5">
      <div
        className="relative flex h-full flex-col overflow-hidden rounded-2xl shadow-[0_12px_32px_-8px_rgba(0,0,0,0.45)] ring-1 ring-black/10"
        style={{ background: color.paper, color: color.ink }}
      >
        <div
          data-tauri-drag-region
          className={cn("flex h-9 shrink-0 items-center gap-1 px-2 transition-opacity", focused ? "opacity-100" : "opacity-70 hover:opacity-100")}
          style={{ background: color.bar }}
        >
          <Popover.Root>
            <Popover.Trigger asChild>
              <button className="h-4 w-4 rounded-full ring-2 ring-black/15" style={{ background: color.paper }} aria-label="Color" />
            </Popover.Trigger>
            <Popover.Portal>
              <Popover.Content sideOffset={6} align="start" className="z-50 grid grid-cols-4 gap-1.5 rounded-xl bg-white p-2 shadow-xl">
                {Object.entries(STICKY_COLORS).map(([key, c]) => (
                  <button
                    key={key}
                    title={c.label}
                    onClick={() => style({ color: key })}
                    className={cn("h-6 w-6 rounded-full ring-1 ring-black/10 transition-transform hover:scale-110", sticky.color === key && "ring-2 ring-black/50")}
                    style={{ background: c.paper }}
                  />
                ))}
              </Popover.Content>
            </Popover.Portal>
          </Popover.Root>
          <div data-tauri-drag-region className="flex-1" />
          <BarButton label="Bold" onClick={() => editor?.chain().focus().toggleBold().run()}>
            <Bold size={13} />
          </BarButton>
          <BarButton label="Italic" onClick={() => editor?.chain().focus().toggleItalic().run()}>
            <Italic size={13} />
          </BarButton>
          <BarButton label="Checklist" onClick={() => editor?.chain().focus().toggleTaskList().run()}>
            <ListChecks size={13} />
          </BarButton>
          <BarButton
            label={calcOpen ? "Back to the note" : "Calculator"}
            active={calcOpen}
            onClick={() => {
              setCalcOpen((open) => {
                // Closing returns the cursor to the note.
                if (open) requestAnimationFrame(() => editorRef.current?.commands.focus());
                return !open;
              });
            }}
          >
            <Calculator size={13} />
          </BarButton>
          <BarButton label={sticky.onTop ? "Unpin from top" : "Keep on top"} onClick={() => style({ onTop: !sticky.onTop })}>
            {sticky.onTop ? <Pin size={13} /> : <PinOff size={13} />}
          </BarButton>
          <BarButton
            label={confirmDelete ? "Click again to delete this note" : "Delete note"}
            onClick={() => (confirmDelete ? api.deleteSticky(sticky.id) : setConfirmDelete(true))}
            onLeave={() => setConfirmDelete(false)}
            className={confirmDelete ? "w-auto bg-red-600/85 px-2 text-[0.7rem] font-semibold text-white opacity-100 hover:bg-red-600" : undefined}
          >
            {confirmDelete ? "Delete?" : <Trash2 size={13} />}
          </BarButton>
          <BarButton
            label="Close (stays in your vault)"
            onClick={async () => {
              try {
                await flushNow();
              } catch {
                return; // keep the window open so the text isn't lost
              }
              api.closeSticky(sticky.id);
            }}
          >
            <X size={14} />
          </BarButton>
        </div>
        <div
          className={cn("sticky-editor min-h-0 flex-1 cursor-text overflow-y-auto px-4 py-3", calcOpen && "hidden")}
          onClick={() => editor?.commands.focus()}
        >
          <EditorContent editor={editor} className="h-full" />
        </div>
        {calcOpen && (
          <div className="min-h-0 flex-1">
            <MiniCalculator
              ink={color.ink}
              paper={color.paper}
              onClose={() => {
                setCalcOpen(false);
                requestAnimationFrame(() => editorRef.current?.commands.focus());
              }}
            />
          </div>
        )}
        {saveFailed && (
          <div role="alert" className="shrink-0 px-4 py-1.5 text-xs font-medium text-red-700" style={{ background: color.bar }}>
            Couldn't save this note. Try again.
          </div>
        )}
        <div
          className="absolute bottom-3 right-3 h-3 w-3 cursor-se-resize opacity-40"
          onMouseDown={(e) => {
            e.preventDefault();
            win.startResizeDragging("SouthEast");
          }}
          style={{
            background: `linear-gradient(135deg, transparent 50%, ${color.ink} 50%)`,
            borderBottomRightRadius: 4,
          }}
        />
      </div>
    </div>
  );
}

function BarButton({
  label,
  onClick,
  children,
  active,
  className,
  onLeave,
}: {
  label: string;
  onClick: () => void;
  children: React.ReactNode;
  active?: boolean;
  className?: string;
  onLeave?: () => void;
}) {
  return (
    <button
      title={label}
      aria-label={label}
      onMouseDown={(e) => e.preventDefault()}
      onClick={onClick}
      onMouseLeave={onLeave}
      onBlur={onLeave}
      className={cn("grid h-6 w-6 place-items-center rounded-md transition hover:bg-black/10 hover:opacity-100", active ? "bg-black/15 opacity-100" : "opacity-70", className)}
    >
      {children}
    </button>
  );
}
