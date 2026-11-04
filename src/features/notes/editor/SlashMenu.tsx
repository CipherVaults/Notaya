import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { Editor } from "@tiptap/react";
import { CalendarDays, ImagePlus, Link2, Minus, PilcrowLeft, PilcrowRight, Radical, Table2, TableProperties } from "lucide-react";
import DatabasePicker from "@/features/database/DatabasePicker";
import { insertInlineDatabase, insertLinkedView } from "@/features/database/inline";
import { cn } from "@/lib/utils";
import { useApp } from "@/store/app";
import { pickImages } from "./extensions";
import { applyTurnInto, BLOCKS, type BlockKey } from "./turnInto";
import type { SlashState } from "./slash";
import PagePicker, { type PickerPlacement } from "./PagePicker";

interface Item {
  title: string;
  hint: string;
  group: string;
  keywords: string;
  icon: React.ReactNode;
  run: (editor: Editor, page?: PageRef) => void;
  /** Opens the page or database list instead of running right away. */
  picker?: "page" | "database";
}

/** The page being edited (a row of a database counts as a page). */
export interface PageRef {
  id: string;
  sectionId: string;
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
  { group: "Lists", title: "Shelf", hint: "A shaded box with a title that hides or shows what's in it", keywords: "shelf collapse collapsible box group", icon: icon("shelf"), run: turn("shelf") },
  { group: "Insert", title: "Quote", hint: "Set a passage apart", keywords: "quote citation", icon: icon("quote"), run: turn("quote") },
  { group: "Insert", title: "Highlight box", hint: "A shaded box for a tip or warning", keywords: "highlight box callout note tip warning info", icon: icon("callout"), run: turn("callout") },
  { group: "Insert", title: "Code block", hint: "Monospaced code with colors", keywords: "code snippet pre", icon: icon("code"), run: turn("code") },
  { group: "Insert", title: "Divider line", hint: "A thin line across the page", keywords: "divider line rule separator hr", icon: <Minus size={s} />, run: (e) => e.chain().focus().setHorizontalRule().run() },
  { group: "Insert", title: "Table", hint: "Rows and columns", keywords: "table grid rows columns", icon: <Table2 size={s} />, run: (e) => e.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run() },
  { group: "Insert", title: "Picture", hint: "Add an image from your PC", keywords: "picture image photo upload png jpg", icon: <ImagePlus size={s} />, run: (e) => pickImages(e) },
  { group: "Insert", title: "Equation", hint: "Maths written in TeX", keywords: "equation math formula tex latex block", icon: icon("equation"), run: turn("equation") },
  { group: "Insert", title: "Inline equation", hint: "Maths inside a line (or type $x^2$)", keywords: "equation math formula tex latex inline", icon: <Radical size={s} />, run: (e) => e.chain().focus().insertMathInline().run() },
  { group: "Insert", title: "Diagram", hint: "A flowchart, sequence, Gantt… written as Mermaid code", keywords: "diagram mermaid flowchart graph chart sequence uml gantt pie mindmap er class state git", icon: icon("mermaid"), run: turn("mermaid") },
  { group: "Insert", title: "Sub-page", hint: "A new page inside this one", keywords: "page subpage sub-page child inside", icon: icon("page"), run: turn("page") },
  { group: "Insert", title: "Link to page", hint: "A link to a page you already have", keywords: "link page mention existing", icon: <Link2 size={s} />, run: () => {}, picker: "page" },
  { group: "Database", title: "Database (inline)", hint: "A new table inside this page", keywords: "database table inline notion rows properties board calendar", icon: <TableProperties size={s} />, run: (e, page) => page && void insertInlineDatabase(e, page) },
  { group: "Database", title: "Linked view of database…", hint: "Show a database you already have, with its own filters", keywords: "database linked view existing embed", icon: <Table2 size={s} />, run: () => {}, picker: "database" },
  { group: "Direction", title: "Right-to-left text", hint: "Arabic, Hebrew: runs from the right", keywords: "rtl right to left direction arabic hebrew persian urdu", icon: <PilcrowLeft size={s} />, run: (e) => void e.chain().focus().setBlockDirection("rtl").run() },
  { group: "Direction", title: "Left-to-right text", hint: "English: runs from the left", keywords: "ltr left to right direction english", icon: <PilcrowRight size={s} />, run: (e) => void e.chain().focus().setBlockDirection("ltr").run() },
  { group: "Direction", title: "Automatic direction", hint: "Follow the language of the text", keywords: "auto automatic direction rtl ltr language", icon: icon("p"), run: (e) => void e.chain().focus().setBlockDirection(null).run() },
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
  page,
}: {
  editor: Editor;
  slash: SlashState | null;
  controller: React.MutableRefObject<SlashController>;
  page?: PageRef;
}) {
  const securityOn = useApp((s) => s.status?.securityOn ?? false);
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

  const [picker, setPicker] = useState<(PickerPlacement & { anchor: number; kind: "page" | "database" }) | null>(null);

  const choose = (item: Item) => {
    const st = controller.current.state;
    if (!st) return;
    editor.chain().focus().deleteRange({ from: st.from, to: st.to }).run();
    if (item.picker) setPicker({ left: st.left, top: st.top, bottom: st.bottom, anchor: editor.state.selection.from, kind: item.picker });
    else item.run(editor, page);
  };

  const closePicker = () => {
    setPicker(null);
    editor.commands.focus();
  };

  const insertLinked = (databaseId: string) => {
    setPicker(null);
    void insertLinkedView(editor, databaseId);
  };

  /** Puts a link to the chosen page where the "/" was: in place of an empty line, else below the current block. */
  const insertPageLink = (pageId: string) => {
    const anchor = picker?.anchor ?? editor.state.selection.from;
    setPicker(null);
    const { doc, schema } = editor.state;
    const linkType = schema.nodes.pageLink;
    const $p = doc.resolve(Math.min(anchor, doc.content.size));
    const nodes = [{ type: "pageLink", attrs: { pageId } }, { type: "paragraph" }];
    for (let d = $p.depth; d >= 1; d--) {
      const container = $p.node(d - 1);
      const at = $p.index(d - 1);
      if (!container.canReplaceWith(at, at, linkType)) continue;
      const node = $p.node(d);
      const empty = d === $p.depth && node.type.name === "paragraph" && node.content.size === 0;
      // An empty last list item can't hold a link: put the link after the list and drop the empty item.
      const item = $p.depth >= 2 ? $p.node($p.depth - 1) : null;
      const itemDepth = $p.depth - 1;
      if (d < $p.depth && item && ["listItem", "taskItem"].includes(item.type.name) && item.childCount === 1 && $p.parent.content.size === 0 && itemDepth > d) {
        const list = $p.node(itemDepth - 1);
        if (itemDepth - 1 === d && list.childCount > 1 && $p.index(itemDepth - 1) === list.childCount - 1) {
          const tr = editor.state.tr;
          tr.insert($p.after(d), nodes.map((n) => schema.nodeFromJSON(n)));
          tr.delete($p.before(itemDepth), $p.after(itemDepth));
          editor.view.dispatch(tr);
          editor.commands.focus($p.after(d) - ($p.after(itemDepth) - $p.before(itemDepth)) + 2);
          return;
        }
      }
      editor
        .chain()
        .focus()
        .insertContentAt(empty ? { from: $p.before(d), to: $p.after(d) } : $p.after(d), nodes)
        .run();
      return;
    }
    editor.commands.focus();
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

  const pickerView = picker ? (
    picker.kind === "database" ? <DatabasePicker placement={picker} onPick={insertLinked} onClose={closePicker} /> : <PagePicker placement={picker} onPick={insertPageLink} onClose={closePicker} />
  ) : null;
  if (!slash || items.length === 0) return pickerView;

  let lastGroup = "";
  const menu = createPortal(
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
                <span className="block truncate text-xs text-faint">{item.title === "Picture" && securityOn ? `${item.hint} (it is encrypted)` : item.hint}</span>
              </span>
            </button>
          </div>
        );
      })}
    </div>,
    document.body,
  );
  return (
    <>
      {pickerView}
      {menu}
    </>
  );
}
