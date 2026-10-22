// Equations written in TeX: a block equation and an inline one.
// KaTeX draws them. It runs only on this device, with no network, and with "trust" off
// so equations can't contain links or other active content.

import katex from "katex";
import { InputRule, Node, mergeAttributes, type Editor } from "@tiptap/react";
import { NodeSelection } from "@tiptap/pm/state";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    math: {
      insertMathBlock: (latex?: string) => ReturnType;
      /** Turns the selected text into an inline equation (or inserts an empty one). */
      insertMathInline: () => ReturnType;
    };
  }
}

function draw(el: HTMLElement, latex: string, display: boolean) {
  el.classList.toggle("empty", !latex.trim());
  if (!latex.trim()) {
    el.textContent = display ? "Add a TeX equation" : "equation";
    return;
  }
  try {
    katex.render(latex, el, { displayMode: display, throwOnError: false, strict: "ignore", trust: false, output: "htmlAndMathml" });
  } catch {
    el.textContent = latex;
  }
}

/** A small box under the equation where its TeX is typed, with a live preview. */
function openEditor(anchor: HTMLElement, initial: string, display: boolean, onInput: (latex: string) => void, onClose: () => void) {
  const box = document.createElement("div");
  box.className = "math-editor";
  const area = document.createElement("textarea");
  area.value = initial;
  area.rows = display ? 3 : 2;
  area.spellcheck = false;
  area.placeholder = String.raw`E = mc^2   ·   \frac{a}{b}   ·   \sqrt{x}`;
  const preview = document.createElement("div");
  preview.className = "math-preview";
  const hint = document.createElement("div");
  hint.className = "math-hint";
  hint.textContent = "TeX. Press Enter to finish; Shift+Enter for a new line.";
  box.append(area, preview, hint);
  document.body.append(box);

  const place = () => {
    const r = anchor.getBoundingClientRect();
    const w = box.offsetWidth || 420;
    const h = box.offsetHeight || 160;
    const below = window.innerHeight - r.bottom;
    box.style.left = `${Math.max(8, Math.min(r.left + r.width / 2 - w / 2, window.innerWidth - w - 8))}px`;
    box.style.top = `${below < h + 16 ? Math.max(8, r.top - h - 8) : r.bottom + 8}px`;
  };
  const refresh = () => {
    draw(preview, area.value, true);
    if (!area.value.trim()) preview.textContent = "";
    place();
  };
  refresh();
  area.focus();
  area.setSelectionRange(area.value.length, area.value.length);

  let closed = false;
  const close = () => {
    if (closed) return;
    closed = true;
    window.removeEventListener("mousedown", outside, true);
    box.remove();
    onClose();
  };
  const outside = (e: MouseEvent) => {
    if (!box.contains(e.target as globalThis.Node)) close();
  };
  window.addEventListener("mousedown", outside, true);
  area.addEventListener("input", () => {
    onInput(area.value);
    refresh();
  });
  area.addEventListener("keydown", (e) => {
    e.stopPropagation();
    if (e.key === "Escape" || (e.key === "Enter" && !e.shiftKey)) {
      e.preventDefault();
      close();
    }
  });
  return close;
}

function equationView(display: boolean, justInserted: { current: boolean }) {
  return ({ node, getPos, editor }: { node: import("@tiptap/pm/model").Node; getPos: () => number | undefined; editor: Editor }) => {
    let current = node;
    const dom = document.createElement(display ? "div" : "span");
    dom.className = display ? "math-block" : "math-inline";
    dom.contentEditable = "false";
    const body = document.createElement("span");
    body.className = "math-body";
    dom.append(body);
    draw(body, current.attrs.latex, display);

    let closeEditor: (() => void) | null = null;
    const edit = () => {
      if (!editor.isEditable || closeEditor) return;
      dom.dataset.editing = "true";
      closeEditor = openEditor(
        dom,
        current.attrs.latex,
        display,
        (latex) => {
          const pos = getPos();
          if (pos === undefined) return;
          editor.view.dispatch(editor.state.tr.setNodeMarkup(pos, undefined, { latex }));
        },
        () => {
          closeEditor = null;
          delete dom.dataset.editing;
          // Put the cursor back after the equation, so typing carries on.
          const pos = getPos();
          if (pos !== undefined) editor.chain().focus(pos + current.nodeSize).run();
        },
      );
    };
    dom.addEventListener("click", edit);
    if (justInserted.current && editor.isEditable) {
      justInserted.current = false;
      setTimeout(edit, 0);
    }

    return {
      dom,
      update(updated: import("@tiptap/pm/model").Node) {
        if (updated.type !== current.type) return false;
        current = updated;
        draw(body, current.attrs.latex, display);
        return true;
      },
      selectNode() {
        dom.dataset.selected = "true";
      },
      deselectNode() {
        delete dom.dataset.selected;
      },
      stopEvent: (e: Event) => !!closeEditor && e.target instanceof HTMLElement && e.target.closest(".math-editor") !== null,
      ignoreMutation: () => true,
      destroy() {
        closeEditor?.();
      },
    };
  };
}

const latexAttr = {
  latex: {
    default: "",
    parseHTML: (el: HTMLElement) => el.getAttribute("data-latex") ?? "",
    renderHTML: (attrs: Record<string, unknown>) => ({ "data-latex": String(attrs.latex ?? "") }),
  },
};

/** An equation on its own line. */
export const MathBlock = Node.create({
  name: "mathBlock",
  group: "block",
  atom: true,
  selectable: true,
  draggable: true,
  addStorage() {
    return { justInserted: { current: false } };
  },
  addAttributes() {
    return latexAttr;
  },
  parseHTML() {
    return [{ tag: "div[data-math-block]" }];
  },
  renderHTML({ HTMLAttributes }) {
    return ["div", mergeAttributes(HTMLAttributes, { "data-math-block": "", class: "math-block" })];
  },
  renderText: ({ node }) => String(node.attrs.latex ?? ""),
  addNodeView() {
    return equationView(true, this.storage.justInserted);
  },
  addCommands() {
    return {
      insertMathBlock:
        (latex = "") =>
        ({ commands }) => {
          this.storage.justInserted.current = latex === "";
          return commands.insertContent([{ type: this.name, attrs: { latex } }, { type: "paragraph" }]);
        },
    };
  },
});

/** An equation inside a line of text. Type $x^2$ to make one. */
export const MathInline = Node.create({
  name: "mathInline",
  group: "inline",
  inline: true,
  atom: true,
  selectable: true,
  addStorage() {
    return { justInserted: { current: false } };
  },
  addAttributes() {
    return latexAttr;
  },
  parseHTML() {
    return [{ tag: "span[data-math-inline]" }];
  },
  renderHTML({ HTMLAttributes }) {
    return ["span", mergeAttributes(HTMLAttributes, { "data-math-inline": "", class: "math-inline" })];
  },
  renderText: ({ node }) => String(node.attrs.latex ?? ""),
  addNodeView() {
    return equationView(false, this.storage.justInserted);
  },
  addCommands() {
    return {
      insertMathInline:
        () =>
        ({ state, tr, dispatch }) => {
          const { from, to, empty } = state.selection;
          // Only text inside one line can become an equation; a selection across blocks is left alone.
          if (!empty && !state.selection.$from.sameParent(state.selection.$to)) return false;
          const raw = empty ? "" : state.doc.textBetween(from, to, " ");
          // Spaces picked up by a double-click stay outside the equation.
          const lead = raw.length - raw.trimStart().length;
          const trail = raw.length - raw.trimEnd().length;
          const latex = raw.trim();
          if (dispatch) {
            this.storage.justInserted.current = latex === "";
            const start = empty ? from : from + lead;
            const end = empty ? to : to - trail;
            tr.replaceWith(start, Math.max(start, end), this.type.create({ latex }));
            // Select the new equation so it shows as selected.
            try {
              tr.setSelection(NodeSelection.create(tr.doc, start));
            } catch {
              // Keep the default selection.
            }
          }
          return true;
        },
    };
  },
  addInputRules() {
    const type = this.type;
    return [
      new InputRule({
        find: /(?:^|[^$\\])\$([^$\n]+)\$$/,
        handler: ({ state, range, match }) => {
          const start = range.from + match[0].indexOf("$");
          state.tr.replaceWith(start, range.to, type.create({ latex: match[1] }));
        },
      }),
    ];
  },
});
