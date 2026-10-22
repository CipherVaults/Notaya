import { useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { EditorContent, useEditor, type Editor } from "@tiptap/react";
import { Popover } from "radix-ui";
import { Bold, Calculator, Italic, ListChecks, Pin, PinOff, Trash2, X } from "lucide-react";
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

  if (!sticky) return null;
  return <StickyCard sticky={sticky} onChange={setSticky} />;
}

function StickyCard({ sticky, onChange }: { sticky: Sticky; onChange: (s: Sticky) => void }) {
  const color = stickyColor(sticky.color);
  const editorRef = useRef<Editor | null>(null);
  const [focused, setFocused] = useState(false);
  const [calcOpen, setCalcOpen] = useState(false);

  const save = useMemo(
    () =>
      debounce(() => {
        const e = editorRef.current;
        if (e) api.saveSticky(sticky.id, JSON.stringify(e.getJSON()), e.getText()).catch(() => {});
      }, 400),
    [sticky.id],
  );

  const editor = useEditor({
    extensions: stickyExtensions(),
    content: parseContent(sticky.content),
    editorProps: { ...editorProps(() => editorRef.current), attributes: { class: "selectable" } },
    onUpdate: () => save(),
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
      api.setStickyGeometry(sticky.id, p.x, p.y, s.width, s.height).catch(() => {});
    }, 500);
    const a = win.onMoved(() => persist());
    const b = win.onResized(() => persist());
    requestAnimationFrame(() => requestAnimationFrame(() => api.windowReady()));
    return () => {
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
          <BarButton label="Delete note" onClick={() => api.deleteSticky(sticky.id)}>
            <Trash2 size={13} />
          </BarButton>
          <BarButton
            label="Close (stays in your vault)"
            onClick={() => {
              save.cancel();
              const e = editorRef.current;
              const flush = e ? api.saveSticky(sticky.id, JSON.stringify(e.getJSON()), e.getText()) : Promise.resolve();
              flush.finally(() => api.closeSticky(sticky.id));
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

function BarButton({ label, onClick, children, active }: { label: string; onClick: () => void; children: React.ReactNode; active?: boolean }) {
  return (
    <button
      title={label}
      aria-label={label}
      onMouseDown={(e) => e.preventDefault()}
      onClick={onClick}
      className={cn("grid h-6 w-6 place-items-center rounded-md transition hover:bg-black/10 hover:opacity-100", active ? "bg-black/15 opacity-100" : "opacity-70")}
    >
      {children}
    </button>
  );
}
