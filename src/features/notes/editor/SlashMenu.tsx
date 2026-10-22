import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { Editor } from "@tiptap/react";
import { CalendarDays, ImagePlus, Minus, Radical, Table2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { pickImages } from "./extensions";
import { applyTurnInto, BLOCKS, type BlockKey } from "./turnInto";
import type { SlashState } from "./slash";

interface Item {
  title: string;
  hint: string;
  group: string;
  keywords: string;
  icon: React.ReactNode;
  run: (editor: Editor) => void;
}

const s = 17;
const icon = (key: BlockKey) => BLOCKS.find((b) => b.key === key)!.icon;
const turn = (key: BlockKey) => (e: Editor) => void applyTurnInto(e, key);

const ITEMS: Item[] = [
  { group: "Text", title: "Paragraph", hint: "Ordinary text", keywords: "text paragraph plain body", icon: icon("p"), run: turn("p") },
  { group: "Text", title: "Large heading", hint: "A big title for a section", keywords: "heading title h1 big", icon: icon("h1"), run: turn("h1") },
  { group: "Text", title: "Medium heading", hint: "A section title", keywords: "heading h2", icon: icon("h2"), run: turn("h2") },
  { group: "Text", title: "Small heading", hint: "A minor title", keywords: "heading h3 small", icon: icon("h3"), run: turn("h3") },
  { group: "Lists", title: "Bullet list", hint: "Items with dots", keywords: "bullet list dots ul unordered", icon: icon("bullet"), run: turn("bullet") },
  { group: "Lists", title: "Numbered list", hint: "Items that count up", keywords: "number list ol ordered", icon: icon("ordered"), run: turn("ordered") },
  { group: "Lists", title: "Checklist", hint: "Tick things off", keywords: "checklist todo task checkbox check", icon: icon("task"), run: turn("task") },
  { group: "Lists", title: "Collapsible section", hint: "A line that hides or shows what's under it", keywords: "collapsible toggle fold hide details", icon: icon("toggle"), run: turn("toggle") },
  { group: "Insert", title: "Quote", hint: "Set a passage apart", keywords: "quote citation", icon: icon("quote"), run: turn("quote") },
  { group: "Insert", title: "Highlight box", hint: "A shaded box for a tip or warning", keywords: "highlight box callout note tip warning info", icon: icon("callout"), run: turn("callout") },
  { group: "Insert", title: "Code block", hint: "Monospaced code with colors", keywords: "code snippet pre", icon: icon("code"), run: turn("code") },
  { group: "Insert", title: "Divider line", hint: "A thin line across the page", keywords: "divider line rule separator hr", icon: <Minus size={s} />, run: (e) => e.chain().focus().setHorizontalRule().run() },
  { group: "Insert", title: "Table", hint: "Rows and columns", keywords: "table grid rows columns", icon: <Table2 size={s} />, run: (e) => e.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run() },
  { group: "Insert", title: "Picture", hint: "Add an image from your PC (it is encrypted)", keywords: "picture image photo upload png jpg", icon: <ImagePlus size={s} />, run: (e) => pickImages(e) },
  { group: "Insert", title: "Equation", hint: "Maths written in TeX", keywords: "equation math formula tex latex block", icon: icon("equation"), run: turn("equation") },
  { group: "Insert", title: "Inline equation", hint: "Maths inside a line (or type $x^2$)", keywords: "equation math formula tex latex inline", icon: <Radical size={s} />, run: (e) => e.chain().focus().insertMathInline().run() },
  { group: "Insert", title: "Sub-page", hint: "A new page inside this one", keywords: "page subpage sub-page child inside", icon: icon("page"), run: turn("page") },
  {
    group: "Quick",
    title: "Today's date",
    hint: new Date().toLocaleDateString(undefined, { dateStyle: "long" }),
    keywords: "date today time now",
    icon: <CalendarDays size={s} />,
    run: (e) => e.chain().focus().insertContent(new Date().toLocaleDateString(undefined, { dateStyle: "long" })).run(),
  },
];

/** Shared with the editor: lets the slash plugin ask the menu to handle keys. */
export interface SlashController {
  state: SlashState | null;
  onKey: (event: KeyboardEvent) => boolean;
}

export default function SlashMenu({
  editor,
  slash,
  controller,
}: {
  editor: Editor;
  slash: SlashState | null;
  controller: React.MutableRefObject<SlashController>;
}) {
  const [index, setIndex] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);
  const [placement, setPlacement] = useState<{ left: number; top: number } | null>(null);

  const items = useMemo(() => {
    const q = (slash?.query ?? "").trim().toLowerCase();
    if (!q) return ITEMS;
    const starts = ITEMS.filter((i) => i.title.toLowerCase().startsWith(q));
    const rest = ITEMS.filter((i) => !starts.includes(i) && (i.title.toLowerCase().includes(q) || i.keywords.includes(q)));
    return [...starts, ...rest];
  }, [slash?.query]);

  useEffect(() => setIndex(0), [slash?.query, slash?.from]);

  const choose = (item: Item) => {
    const st = controller.current.state;
    if (!st) return;
    editor.chain().focus().deleteRange({ from: st.from, to: st.to }).run();
    item.run(editor);
  };

  // Let the slash plugin drive the list with the keyboard.
  controller.current.state = slash;
  controller.current.onKey = (event) => {
    if (items.length === 0) return false;
    if (event.key === "ArrowDown") setIndex((i) => (i + 1) % items.length);
    else if (event.key === "ArrowUp") setIndex((i) => (i - 1 + items.length) % items.length);
    else choose(items[Math.min(index, items.length - 1)]);
    event.preventDefault();
    return true;
  };

  // Place the menu under the caret, or above it when there isn't room below.
  useLayoutEffect(() => {
    if (!slash) return;
    const height = Math.min(listRef.current?.offsetHeight ?? 360, 380);
    const below = window.innerHeight - slash.bottom;
    const top = below < height + 16 ? Math.max(8, slash.top - height - 8) : slash.bottom + 6;
    const left = Math.min(slash.left, window.innerWidth - 300);
    setPlacement({ left, top });
  }, [slash, items.length]);

  useEffect(() => {
    listRef.current?.querySelector<HTMLElement>('[data-active="true"]')?.scrollIntoView({ block: "nearest" });
  }, [index]);

  if (!slash || items.length === 0) return null;

  let lastGroup = "";
  return createPortal(
    <div
      ref={listRef}
      style={{ left: placement?.left ?? slash.left, top: placement?.top ?? slash.bottom + 6 }}
      className="fixed z-[80] max-h-[380px] w-[290px] overflow-y-auto rounded-xl border border-line bg-elevated p-1.5 shadow-float"
      onMouseDown={(e) => e.preventDefault()}
    >
      {items.map((item, i) => {
        const header = item.group !== lastGroup && !slash.query ? item.group : null;
        lastGroup = item.group;
        return (
          <div key={item.title}>
            {header && <div className="px-2.5 pb-1 pt-2 text-[0.7rem] font-medium uppercase tracking-wider text-faint">{header}</div>}
            <button
              data-active={i === index}
              onMouseMove={() => setIndex(i)}
              onClick={() => choose(item)}
              className={cn("flex w-full items-center gap-3 rounded-lg px-2.5 py-1.5 text-left", i === index ? "bg-surface-2" : "")}
            >
              <span className="grid h-9 w-9 shrink-0 place-items-center rounded-lg border border-line bg-bg text-muted">{item.icon}</span>
              <span className="min-w-0">
                <span className="block text-sm text-fg">{item.title}</span>
                <span className="block truncate text-xs text-faint">{item.hint}</span>
              </span>
            </button>
          </div>
        );
      })}
    </div>,
    document.body,
  );
}
