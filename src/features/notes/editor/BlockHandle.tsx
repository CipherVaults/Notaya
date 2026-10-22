import { useEffect, useRef, useState } from "react";
import type { Editor } from "@tiptap/react";
import { NodeSelection, Selection } from "@tiptap/pm/state";
import type { EditorView } from "@tiptap/pm/view";
import { ArrowDown, ArrowUp, ChevronRight, Copy, GripVertical, Plus, RefreshCw, Trash2 } from "lucide-react";
import { TypeMenu } from "./BubbleToolbar";
import { currentBlock } from "./turnInto";

interface Target {
  pos: number;
  top: number;
}

const NESTED_PARENTS = new Set(["callout", "toggle", "blockquote", "column"]);
const ITEMS = new Set(["listItem", "taskItem"]);

/** Finds the block under a document position: a top-level block, a list item, or a block inside a callout/toggle/quote. */
function blockAt(view: EditorView, hitPos: number): { pos: number; size: number } | null {
  const doc = view.state.doc;
  const $pos = doc.resolve(Math.min(Math.max(hitPos, 0), doc.content.size));
  for (let d = $pos.depth; d >= 1; d--) {
    const node = $pos.node(d);
    const parent = $pos.node(d - 1);
    if (d === 1 || ITEMS.has(node.type.name) || NESTED_PARENTS.has(parent.type.name)) {
      return { pos: $pos.before(d), size: node.nodeSize };
    }
  }
  // Between two top-level blocks: take the one that starts here.
  const after = doc.nodeAt($pos.pos);
  return after ? { pos: $pos.pos, size: after.nodeSize } : null;
}

/**
 * The "⋮⋮" grip and "+" button that appear beside a block when you hover it:
 * drag the grip to move the block, click it for a small menu, click "+" to add a block below.
 */
export default function BlockHandle({ editor, container }: { editor: Editor; container: React.RefObject<HTMLDivElement | null> }) {
  const [target, setTarget] = useState<Target | null>(null);
  const [menu, setMenu] = useState(false);
  const menuRef = useRef(false);
  menuRef.current = menu;
  const targetRef = useRef<Target | null>(null);
  targetRef.current = target;

  useEffect(() => {
    const el = container.current;
    if (!el) return;
    const onMove = (e: MouseEvent) => {
      if (!editor.isEditable || menuRef.current) return;
      const view = editor.view;
      const rect = view.dom.getBoundingClientRect();
      // Over the handle (left of the text) use a point inside the block on the same line.
      const x = e.clientX < rect.left + 4 ? rect.left + 30 : Math.min(e.clientX, rect.right - 6);
      const hit = view.posAtCoords({ left: x, top: e.clientY });
      if (!hit) return;
      const block = blockAt(view, hit.inside >= 0 ? hit.inside + 1 : hit.pos);
      if (!block) return;
      const dom = view.nodeDOM(block.pos) as HTMLElement | null;
      if (!dom || !dom.getBoundingClientRect) return;
      const top = dom.getBoundingClientRect().top - el.getBoundingClientRect().top;
      if (targetRef.current?.pos === block.pos && targetRef.current.top === top) return;
      setTarget({ pos: block.pos, top });
    };
    const onLeave = () => {
      if (!menuRef.current) setTarget(null);
    };
    const onKey = () => setTarget(null);
    el.addEventListener("mousemove", onMove);
    el.addEventListener("mouseleave", onLeave);
    editor.view.dom.addEventListener("keydown", onKey);
    return () => {
      el.removeEventListener("mousemove", onMove);
      el.removeEventListener("mouseleave", onLeave);
      editor.view.dom.removeEventListener("keydown", onKey);
    };
  }, [editor, container]);

  // Close the menu on any click elsewhere or Escape.
  useEffect(() => {
    if (!menu) return;
    const close = (e?: Event) => {
      // Clicks inside the block-type list (drawn outside this box) are not "elsewhere".
      if ((e?.target as HTMLElement | undefined)?.closest?.("[data-radix-popper-content-wrapper]")) return;
      setMenu(false);
      setTarget(null);
    };
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && close();
    const t = setTimeout(() => window.addEventListener("mousedown", close), 0);
    window.addEventListener("keydown", onKey);
    return () => {
      clearTimeout(t);
      window.removeEventListener("mousedown", close);
      window.removeEventListener("keydown", onKey);
    };
  }, [menu]);

  if (!target || !editor.isEditable) return null;
  const node = editor.state.doc.nodeAt(target.pos);
  if (!node) return null;

  const addBelow = () => {
    const { pos } = target;
    if (node.isTextblock && node.content.size === 0) {
      editor.chain().focus().insertContentAt(pos + 1, "/").run();
    } else {
      const end = pos + node.nodeSize;
      editor.chain().insertContentAt(end, { type: "paragraph", content: [{ type: "text", text: "/" }] }).focus().run();
    }
    setTarget(null);
  };

  const onDragStart = (e: React.DragEvent) => {
    const view = editor.view;
    const sel = NodeSelection.create(view.state.doc, target.pos);
    view.dispatch(view.state.tr.setSelection(sel));
    view.dragging = { slice: sel.content(), move: !e.altKey };
    e.dataTransfer.effectAllowed = "copyMove";
    e.dataTransfer.setData("text/plain", "");
    const dom = view.nodeDOM(target.pos) as HTMLElement | null;
    if (dom) e.dataTransfer.setDragImage(dom, 0, 0);
  };

  const run = (fn: () => void) => () => {
    fn();
    setMenu(false);
    setTarget(null);
  };
  const remove = () => editor.view.dispatch(editor.state.tr.delete(target.pos, target.pos + node.nodeSize));
  const duplicate = () => editor.view.dispatch(editor.state.tr.insert(target.pos + node.nodeSize, node.copy(node.content)));
  const move = (dir: -1 | 1) => () => {
    const { doc } = editor.state;
    const $p = doc.resolve(target.pos);
    const parent = $p.parent;
    const index = $p.index();
    const sibling = index + dir;
    if (sibling < 0 || sibling >= parent.childCount) return;
    const tr = editor.state.tr;
    if (dir === -1) {
      const prev = parent.child(sibling);
      tr.delete(target.pos, target.pos + node.nodeSize).insert(target.pos - prev.nodeSize, node);
    } else {
      const next = parent.child(sibling);
      tr.insert(target.pos + node.nodeSize + next.nodeSize, node).delete(target.pos, target.pos + node.nodeSize);
    }
    editor.view.dispatch(tr);
  };

  return (
    <div className="block-handle" data-visible="true" style={{ left: 10, top: target.top + 1 }} onMouseDown={(e) => e.stopPropagation()}>
      <button aria-label="Add a block below" title="Add a block below" onClick={addBelow}>
        <Plus size={16} />
      </button>
      <button
        className="grip"
        aria-label="Drag to move, click for options"
        title="Drag to move · click for options"
        draggable
        onDragStart={onDragStart}
        onDragEnd={() => {
          editor.view.dragging = null;
        }}
        onClick={() => {
          if (!menu) {
            // Put the cursor inside this block so "Change type" knows which block it is about.
            const tr = editor.state.tr;
            tr.setSelection(Selection.near(tr.doc.resolve(Math.min(target.pos + 1, tr.doc.content.size))));
            editor.view.dispatch(tr);
          }
          setMenu((m) => !m);
        }}
      >
        <GripVertical size={16} />
      </button>
      {menu && (
        <div
          className="absolute left-full top-7 z-30 w-48 rounded-xl border border-line bg-elevated p-1.5 shadow-float"
          style={{ marginLeft: -22 }}
          onMouseDown={(e) => e.stopPropagation()}
        >
          {node.isTextblock || ["bulletList", "orderedList", "taskList", "blockquote", "callout", "toggle"].includes(node.type.name) ? (
            <TypeMenu
              editor={editor}
              side="right"
              current={currentBlock(editor)}
              track={(open) => {
                if (!open) {
                  setMenu(false);
                  setTarget(null);
                }
              }}
            >
              <button className="flex w-full items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-left text-sm text-fg hover:bg-surface-2">
                <span className="opacity-70">
                  <RefreshCw size={15} />
                </span>
                <span className="flex-1">Change type</span>
                <ChevronRight size={14} className="text-faint" />
              </button>
            </TypeMenu>
          ) : null}
          <MenuRow icon={<Trash2 size={15} />} danger onClick={run(remove)}>
            Delete
          </MenuRow>
          <MenuRow icon={<Copy size={15} />} onClick={run(duplicate)}>
            Duplicate
          </MenuRow>
          <MenuRow icon={<ArrowUp size={15} />} onClick={run(move(-1))}>
            Move up
          </MenuRow>
          <MenuRow icon={<ArrowDown size={15} />} onClick={run(move(1))}>
            Move down
          </MenuRow>
        </div>
      )}
    </div>
  );
}

function MenuRow({ icon, danger, onClick, children }: { icon: React.ReactNode; danger?: boolean; onClick: () => void; children: React.ReactNode }) {
  return (
    <button
      onClick={onClick}
      className={
        "flex w-full items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-left text-sm hover:bg-surface-2 " +
        (danger ? "text-danger" : "text-fg")
      }
    >
      <span className="opacity-70">{icon}</span>
      {children}
    </button>
  );
}
