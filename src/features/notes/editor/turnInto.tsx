// "Change type" for blocks: the list shown in the selection toolbar, the block menu and the "/" menu.

import { Code2, File, Heading1, Heading2, Heading3, Lightbulb, List, ListChecks, ListCollapse, Library, ListOrdered, Pilcrow, Quote, Sigma, Workflow } from "lucide-react";
import type { Editor } from "@tiptap/react";
import { TextSelection } from "@tiptap/pm/state";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { useApp } from "@/store/app";
import { MERMAID_MAX_CHARS, MERMAID_TEMPLATES } from "./mermaid";

export type BlockKey =
  | "p"
  | "h1"
  | "h2"
  | "h3"
  | "bullet"
  | "ordered"
  | "task"
  | "toggle"
  | "shelf"
  | "code"
  | "quote"
  | "callout"
  | "equation"
  | "mermaid"
  | "page";

const s = 17;

export const BLOCKS: { key: BlockKey; label: string; keywords: string; icon: React.ReactNode }[] = [
  { key: "p", label: "Paragraph", keywords: "text paragraph plain body", icon: <Pilcrow size={s} /> },
  { key: "h1", label: "Large heading", keywords: "heading title h1 big", icon: <Heading1 size={s} /> },
  { key: "h2", label: "Medium heading", keywords: "heading h2", icon: <Heading2 size={s} /> },
  { key: "h3", label: "Small heading", keywords: "heading h3 small", icon: <Heading3 size={s} /> },
  { key: "bullet", label: "Bullet list", keywords: "bullet list dots ul unordered", icon: <List size={s} /> },
  { key: "ordered", label: "Numbered list", keywords: "number list ol ordered", icon: <ListOrdered size={s} /> },
  { key: "task", label: "Checklist", keywords: "checklist todo task checkbox check", icon: <ListChecks size={s} /> },
  { key: "toggle", label: "Collapsible section", keywords: "collapsible toggle fold hide details", icon: <ListCollapse size={s} /> },
  { key: "shelf", label: "Shelf", keywords: "shelf collapse collapsible box group", icon: <Library size={s} /> },
  { key: "quote", label: "Quote", keywords: "quote citation", icon: <Quote size={s} /> },
  { key: "callout", label: "Highlight box", keywords: "highlight box callout note tip warning info", icon: <Lightbulb size={s} /> },
  { key: "code", label: "Code block", keywords: "code snippet pre", icon: <Code2 size={s} /> },
  { key: "equation", label: "Equation", keywords: "equation math formula tex latex", icon: <Sigma size={s} /> },
  { key: "mermaid", label: "Diagram", keywords: "diagram mermaid flowchart graph chart sequence uml gantt pie mindmap", icon: <Workflow size={s} /> },
  { key: "page", label: "Sub-page", keywords: "page subpage sub-page child inside", icon: <File size={s} /> },
];

/** The collapsible section the cursor is in, when the cursor's block is the section's first line. */
function toggleAround(editor: Editor): { pos: number; size: number; shelf: boolean } | null {
  const { $from } = editor.state.selection;
  for (let d = $from.depth; d >= 1; d--) {
    if ($from.node(d).type.name === "toggle") {
      return $from.index(d) === 0 ? { pos: $from.before(d), size: $from.node(d).nodeSize, shelf: !!$from.node(d).attrs.shelf } : null;
    }
  }
  return null;
}

/** Which entry of the list describes the block the cursor is in. */
export function currentBlock(editor: Editor): BlockKey {
  const { $from } = editor.state.selection;
  const parent = $from.parent;
  const tog = toggleAround(editor);
  if (tog) return tog.shelf ? "shelf" : "toggle";
  // Pages written with older, now removed, heading sizes show as the smallest heading.
  if (parent.type.name === "heading") return `h${Math.min(3, Number(parent.attrs.level))}` as BlockKey;
  if (editor.isActive("taskList")) return "task";
  if (editor.isActive("bulletList")) return "bullet";
  if (editor.isActive("orderedList")) return "ordered";
  if (editor.isActive("codeBlock")) return "code";
  if (editor.isActive("blockquote")) return "quote";
  if (editor.isActive("callout")) return "callout";
  return "p";
}

function unwrapToggle(editor: Editor, tog: { pos: number; size: number }) {
  const { state } = editor;
  const node = state.doc.nodeAt(tog.pos);
  if (!node) return;
  const tr = state.tr.replaceWith(tog.pos, tog.pos + tog.size, node.content);
  tr.setSelection(TextSelection.near(tr.doc.resolve(tog.pos + 1)));
  editor.view.dispatch(tr);
}

/** Wraps the block the cursor is in as the first line of a new collapsible section, with an empty line inside. */
function wrapInToggle(editor: Editor, shelf = false) {
  editor.chain().focus().wrapIn("toggle", { open: true, shelf }).run();
  const { $from } = editor.state.selection;
  for (let d = $from.depth; d >= 1; d--) {
    if ($from.node(d).type.name === "toggle") {
      const start = $from.start(d);
      editor
        .chain()
        .insertContentAt($from.end(d), { type: "paragraph" })
        .setTextSelection(start + 1)
        .run();
      return;
    }
  }
}

/** Replaces the cursor's block with an equation; the block's text becomes the TeX. */
function toEquation(editor: Editor) {
  const { $from } = editor.state.selection;
  const latex = $from.parent.textContent;
  const depth = $from.depth;
  (editor.storage as unknown as { mathBlock: { justInserted: { current: boolean } } }).mathBlock.justInserted.current = latex === "";
  editor
    .chain()
    .focus()
    .insertContentAt({ from: $from.before(depth), to: $from.after(depth) }, [{ type: "mathBlock", attrs: { latex } }, { type: "paragraph" }])
    .run();
}

/** Replaces the cursor's block with a Mermaid diagram; the block's text (a code block's code, for one) becomes its code. */
function toMermaid(editor: Editor) {
  const { $from } = editor.state.selection;
  const code = $from.parent.textContent.slice(0, MERMAID_MAX_CHARS) || MERMAID_TEMPLATES[0].code;
  const depth = $from.depth;
  (editor.storage as unknown as { mermaidBlock: { justInserted: { current: boolean } } }).mermaidBlock.justInserted.current = true;
  editor
    .chain()
    .focus()
    .insertContentAt({ from: $from.before(depth), to: $from.after(depth) }, [{ type: "mermaidBlock", attrs: { code, showCode: true } }, { type: "paragraph" }])
    .run();
}

/** Makes a page inside the current page and puts a link to it where the block was. */
async function toSubpage(editor: Editor) {
  const pageId = useApp.getState().activePageId;
  if (!pageId) return;
  const { $from } = editor.state.selection;
  const title = $from.parent.textContent.trim().slice(0, 120);
  try {
    const created = await api.createSubpage(pageId, title);
    useApp.getState().setTree(await api.listNotes());
    const now = editor.state.selection.$from;
    const depth = now.depth;
    editor
      .chain()
      .focus()
      .insertContentAt({ from: now.before(depth), to: now.after(depth) }, [{ type: "pageLink", attrs: { pageId: created.id } }, { type: "paragraph" }])
      .run();
  } catch (e) {
    toast.error(errorText(e));
  }
}

export async function applyTurnInto(editor: Editor, key: BlockKey) {
  const tog = toggleAround(editor);
  // Already a collapsible section: "Collapsible section" and "Shelf" only switch its look.
  if (tog && (key === "toggle" || key === "shelf")) {
    const shelf = key === "shelf";
    if (tog.shelf !== shelf) {
      const node = editor.state.doc.nodeAt(tog.pos);
      if (node) editor.view.dispatch(editor.state.tr.setNodeMarkup(tog.pos, undefined, { ...node.attrs, shelf }));
    }
    editor.view.focus();
    return;
  }
  if (tog) unwrapToggle(editor, tog);

  const c = () => editor.chain().focus().clearNodes();
  switch (key) {
    case "p":
      c().run();
      break;
    case "h1":
    case "h2":
    case "h3":
      c().setHeading({ level: Number(key[1]) as 1 | 2 | 3 }).run();
      break;
    case "bullet":
      c().toggleBulletList().run();
      break;
    case "ordered":
      c().toggleOrderedList().run();
      break;
    case "task":
      c().toggleTaskList().run();
      break;
    case "code":
      c().setCodeBlock().run();
      break;
    case "quote":
      c().setBlockquote().run();
      break;
    case "callout":
      c().wrapIn("callout").run();
      break;
    case "equation":
      c().run();
      toEquation(editor);
      break;
    case "mermaid":
      c().run();
      toMermaid(editor);
      break;
    case "toggle":
      c().run();
      wrapInToggle(editor);
      break;
    case "shelf":
      c().run();
      wrapInToggle(editor, true);
      break;
    case "page":
      c().run();
      await toSubpage(editor);
      break;
  }
}

