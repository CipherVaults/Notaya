// Text direction (left-to-right / right-to-left) for every text block.
//
// Each paragraph and heading may carry a `dir` attribute: "ltr", "rtl" or null (= automatic). Automatic
// means "decide from the block's own text": the first strong letter wins, so Arabic and Hebrew blocks
// run right-to-left and are right-aligned, English blocks stay left-to-right, and a page can mix them
// block by block. The stored value travels in the page JSON (vault, backups, exports). Pages saved by
// older versions have no `dir` and so are automatic, with no migration.
//
// Containers (list items, quotes, highlight boxes, collapsible sections, tables) take the direction of
// their first text block, so bullets, the quote bar, the box icon, the arrow and the table columns flip
// with the text. This is drawn with node decorations: the browser's own dir="auto" can't be used on a
// container, because a child that has a dir attribute is skipped when it works out the container's.
// The Rust exporters follow the same rule (src-tauri/src/export/doc.rs, `first_strong`).

import { Extension } from "@tiptap/react";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";
import type { Node as PMNode } from "@tiptap/pm/model";

export type Dir = "ltr" | "rtl";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    direction: {
      /** Forces the selected text blocks to "ltr" or "rtl", or back to automatic with null. */
      setBlockDirection: (dir: Dir | null) => ReturnType;
    };
  }
}

const RTL_LETTER = /^[\p{Script=Arabic}\p{Script=Hebrew}\p{Script=Syriac}\p{Script=Thaana}\p{Script=Nko}\p{Script=Samaritan}\p{Script=Mandaic}]$/u;
const LETTER = /^\p{L}$/u;

/** The direction of the first strong character of `text` (a letter, or an explicit direction mark), or null. */
export function firstStrong(text: string): Dir | null {
  for (const ch of text) {
    if (ch === "‏") return "rtl";
    if (ch === "‎") return "ltr";
    if (LETTER.test(ch)) return RTL_LETTER.test(ch) ? "rtl" : "ltr";
  }
  return null;
}

/** Blocks that hold text and store their own direction. */
const TEXT_BLOCKS = new Set(["paragraph", "heading"]);
/** Blocks whose box (marker, bar, icon, arrow, columns) follows the direction of their first text block. */
const CONTAINERS = new Set(["bulletList", "orderedList", "taskList", "listItem", "taskItem", "blockquote", "callout", "toggle", "table"]);

const own = new WeakMap<PMNode, Dir | null>();
function ownDirection(node: PMNode): Dir | null {
  const hit = own.get(node);
  if (hit !== undefined) return hit;
  const stored = node.attrs.dir === "rtl" || node.attrs.dir === "ltr" ? (node.attrs.dir as Dir) : null;
  const d = stored ?? firstStrong(node.textContent);
  own.set(node, d);
  return d;
}

function buildDecorations(doc: PMNode): DecorationSet {
  const decos: Decoration[] = [];
  // An empty line takes the direction of the line before it, so the caret does not jump sides.
  let last: Dir = "ltr";
  // Returns the direction of the first text block inside `node` (null if there is none).
  const walk = (node: PMNode, pos: number): Dir | null => {
    if (TEXT_BLOCKS.has(node.type.name)) {
      const d = ownDirection(node) ?? (node.content.size === 0 ? last : "ltr");
      last = d;
      decos.push(Decoration.node(pos, pos + node.nodeSize, { dir: d }));
      return d;
    }
    if (node.isTextblock || node.isLeaf) return null;
    let first: Dir | null = null;
    node.forEach((child, offset) => {
      const d = walk(child, pos + 1 + offset);
      if (first === null) first = d;
    });
    if (first !== null && CONTAINERS.has(node.type.name)) {
      decos.push(Decoration.node(pos, pos + node.nodeSize, { dir: first }));
    }
    return first;
  };
  doc.forEach((child, offset) => walk(child, offset));
  return DecorationSet.create(doc, decos);
}

const key = new PluginKey("textDirection");

export const Direction = Extension.create({
  name: "direction",

  addGlobalAttributes() {
    return [
      {
        types: [...TEXT_BLOCKS],
        attributes: {
          dir: {
            default: null,
            parseHTML: (el: HTMLElement) => {
              const v = el.getAttribute("dir");
              return v === "rtl" || v === "ltr" ? v : null;
            },
            // Automatic is written as dir="auto" so copied or exported HTML still decides by text.
            renderHTML: (attrs: Record<string, unknown>) => ({ dir: attrs.dir === "rtl" || attrs.dir === "ltr" ? attrs.dir : "auto" }),
          },
        },
      },
    ];
  },

  addCommands() {
    return {
      setBlockDirection:
        (dir) =>
        ({ state, tr, dispatch }) => {
          const found = new Set<number>();
          for (const range of state.selection.ranges) {
            state.doc.nodesBetween(range.$from.pos, range.$to.pos, (node, pos) => {
              if (TEXT_BLOCKS.has(node.type.name)) found.add(pos);
              return !node.isTextblock;
            });
          }
          if (found.size === 0) return false;
          if (dispatch) {
            for (const pos of found) {
              const node = tr.doc.nodeAt(pos);
              if (node) tr.setNodeMarkup(pos, undefined, { ...node.attrs, dir });
            }
            dispatch(tr);
          }
          return true;
        },
    };
  },

  addProseMirrorPlugins() {
    let cachedDoc: PMNode | null = null;
    let cachedSet = DecorationSet.empty;
    return [
      new Plugin({
        key,
        props: {
          decorations(state) {
            if (state.doc !== cachedDoc) {
              cachedDoc = state.doc;
              cachedSet = buildDecorations(state.doc);
            }
            return cachedSet;
          },
        },
      }),
    ];
  },
});

/** The direction stored on the block the cursor is in: "ltr", "rtl" or null (automatic). */
export function storedDirection(state: { selection: { $from: { parent: PMNode } } }): Dir | null {
  const d = state.selection.$from.parent.attrs.dir;
  return d === "rtl" || d === "ltr" ? d : null;
}
