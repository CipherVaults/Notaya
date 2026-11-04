// A diagram written as Mermaid code: the code is kept in the page, and the picture is drawn from it on this device.
// See mermaid.ts for how the code is handled (it may come from an imported file, so it is never trusted).

import { InputRule, Node, mergeAttributes, type Editor } from "@tiptap/react";
import { NodeSelection } from "@tiptap/pm/state";
import { drawMermaid, lookKey, MERMAID_MAX_CHARS, MERMAID_TEMPLATES, onLookChange } from "./mermaid";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    mermaid: {
      /** Puts a diagram on its own line; with no code, a small flowchart as a start. */
      insertMermaid: (code?: string) => ReturnType;
    };
  }
}

const DEBOUNCE_MS = 400;

type PmNode = import("@tiptap/pm/model").Node;

function mermaidView(justInserted: { current: boolean }) {
  return ({ node, getPos, editor }: { node: PmNode; getPos: () => number | undefined; editor: Editor }) => {
    let current = node;
    const dom = document.createElement("div");
    dom.className = "mermaid-block";
    dom.contentEditable = "false";
    dom.setAttribute("data-mermaid-block", "");

    const bar = document.createElement("div");
    bar.className = "mermaid-bar";
    const label = document.createElement("span");
    label.className = "mermaid-label";
    label.textContent = "Diagram";
    const picker = document.createElement("select");
    picker.className = "mermaid-templates";
    picker.title = "Start from a sample";
    picker.setAttribute("aria-label", "Insert a sample diagram");
    picker.add(new Option("Sample…", ""));
    for (const t of MERMAID_TEMPLATES) picker.add(new Option(t.label, t.key));
    const toggle = document.createElement("button");
    toggle.type = "button";
    toggle.className = "mermaid-toggle";
    bar.append(label, picker, toggle);

    const area = document.createElement("textarea");
    area.className = "mermaid-code";
    area.spellcheck = false;
    area.setAttribute("dir", "ltr");
    area.setAttribute("autocomplete", "off");
    area.setAttribute("autocapitalize", "off");
    area.setAttribute("aria-label", "Mermaid diagram code");
    area.maxLength = MERMAID_MAX_CHARS;
    area.rows = 6;
    area.placeholder = "flowchart TD\n    A[Start] --> B[Finish]";

    const out = document.createElement("div");
    out.className = "mermaid-out";
    const note = document.createElement("div");
    note.className = "mermaid-note";
    const err = document.createElement("pre");
    err.className = "mermaid-error";
    err.setAttribute("role", "alert");
    dom.append(bar, area, out, note, err);

    let token = 0;
    let timer = 0;
    let pending = "";
    let dirty = false;

    const showCode = () => current.attrs.showCode === true;
    const writable = () => editor.isEditable;

    const syncChrome = () => {
      const open = showCode();
      area.hidden = !open;
      area.readOnly = !writable();
      picker.hidden = !open || !writable();
      toggle.textContent = open ? "Hide code" : "Show code";
      toggle.setAttribute("aria-expanded", String(open));
      dom.dataset.code = open ? "open" : "closed";
      dom.dataset.empty = String(current.attrs.code.trim() === "");
    };

    const paint = async (code: string) => {
      const mine = ++token;
      const result = await drawMermaid(code);
      if (mine !== token) return;
      if (result.ok) {
        out.replaceChildren(result.svg);
        out.hidden = false;
        err.textContent = "";
        err.hidden = true;
        note.textContent = result.dropped ? "Settings inside the code (init lines and front matter) are ignored, so the diagram follows your look." : "";
        note.hidden = !result.dropped;
        dom.dataset.state = "ok";
      } else {
        note.hidden = true;
        if (result.error === "") {
          out.replaceChildren();
          out.hidden = true;
          err.hidden = true;
          dom.dataset.state = "empty";
        } else {
          // Keep the last good picture (dimmed) while the code is being fixed.
          out.dataset.stale = out.firstChild ? "true" : "false";
          out.hidden = !out.firstChild;
          err.textContent = result.error;
          err.hidden = false;
          dom.dataset.state = "error";
        }
      }
      if (result.ok) delete out.dataset.stale;
    };

    const schedule = (code: string, delay: number) => {
      pending = code;
      window.clearTimeout(timer);
      timer = window.setTimeout(() => void paint(code), delay);
    };

    const commit = () => {
      if (!dirty) return;
      dirty = false;
      const pos = getPos();
      if (pos === undefined || area.value === current.attrs.code) return;
      editor.view.dispatch(editor.state.tr.setNodeMarkup(pos, undefined, { ...current.attrs, code: area.value }));
    };
    let commitTimer = 0;

    const setShowCode = (open: boolean) => {
      commit();
      const pos = getPos();
      if (pos === undefined) return;
      editor.view.dispatch(editor.state.tr.setNodeMarkup(pos, undefined, { ...current.attrs, showCode: open }));
      if (open) setTimeout(() => area.focus(), 0);
    };

    toggle.addEventListener("mousedown", (e) => e.preventDefault());
    toggle.addEventListener("click", () => setShowCode(!showCode()));

    picker.addEventListener("change", () => {
      const t = MERMAID_TEMPLATES.find((x) => x.key === picker.value);
      picker.value = "";
      if (!t || !writable()) return;
      area.focus();
      area.select();
      // Typing the sample in keeps the text box's own undo working.
      if (!document.execCommand("insertText", false, t.code)) area.value = t.code;
      area.dispatchEvent(new Event("input", { bubbles: true }));
    });

    area.addEventListener("input", () => {
      dirty = true;
      schedule(area.value, DEBOUNCE_MS);
      window.clearTimeout(commitTimer);
      commitTimer = window.setTimeout(commit, DEBOUNCE_MS);
    });
    area.addEventListener("blur", () => {
      window.clearTimeout(commitTimer);
      commit();
    });
    area.addEventListener("keydown", (e) => {
      // The page editor and the app's own shortcuts must not see keys typed into the code.
      e.stopPropagation();
      if (e.key === "Tab" && !e.ctrlKey && !e.altKey && !e.metaKey) {
        e.preventDefault();
        if (!document.execCommand("insertText", false, "  ")) {
          const { selectionStart: a, selectionEnd: b } = area;
          area.setRangeText("  ", a, b, "end");
          area.dispatchEvent(new Event("input", { bubbles: true }));
        }
      } else if (e.key === "Escape" || (e.key === "Enter" && (e.ctrlKey || e.metaKey))) {
        e.preventDefault();
        commit();
        area.blur();
        const pos = getPos();
        if (pos !== undefined) editor.chain().focus(pos + current.nodeSize).run();
      }
    });

    dom.addEventListener("click", (e) => {
      if (e.target instanceof Element && e.target.closest(".mermaid-bar, .mermaid-code")) return;
      if (!writable()) return;
      const pos = getPos();
      if (pos === undefined) return;
      try {
        editor.view.dispatch(editor.state.tr.setSelection(NodeSelection.create(editor.state.doc, pos)));
      } catch {
        // Leave the selection as it is.
      }
    });

    area.value = current.attrs.code;
    syncChrome();
    void paint(current.attrs.code);

    // A change of look redraws the diagram in the new colours.
    let lastLook = lookKey();
    const stopLook = onLookChange(() => {
      const k = lookKey();
      if (k === lastLook) return;
      lastLook = k;
      pending = "";
      schedule(dirty ? area.value : current.attrs.code, 0);
    });

    if (justInserted.current && writable()) {
      justInserted.current = false;
      setTimeout(() => area.focus(), 0);
    }

    return {
      dom,
      update(updated: PmNode) {
        if (updated.type !== current.type) return false;
        const codeChanged = updated.attrs.code !== current.attrs.code;
        current = updated;
        syncChrome();
        // Typing here already put this code in the page; other changes (undo, redo) must reach the text box.
        if (codeChanged && area.value !== current.attrs.code && !dirty) {
          area.value = current.attrs.code;
          schedule(current.attrs.code, 0);
        } else if (codeChanged && pending !== current.attrs.code) {
          schedule(current.attrs.code, 0);
        }
        return true;
      },
      selectNode() {
        dom.dataset.selected = "true";
      },
      deselectNode() {
        delete dom.dataset.selected;
      },
      stopEvent: (e: Event) => e.target instanceof Element && e.target.closest(".mermaid-bar, .mermaid-code") !== null,
      ignoreMutation: () => true,
      destroy() {
        window.clearTimeout(timer);
        window.clearTimeout(commitTimer);
        token++;
        stopLook();
        commit();
      },
    };
  };
}

const codeAttr = {
  code: {
    default: "",
    parseHTML: (el: HTMLElement) => (el.getAttribute("data-mermaid") ?? "").slice(0, MERMAID_MAX_CHARS),
    renderHTML: (attrs: Record<string, unknown>) => ({ "data-mermaid": String(attrs.code ?? "").slice(0, MERMAID_MAX_CHARS) }),
  },
  // Whether the code is shown above the picture (set when a diagram is added; imported diagrams start with it hidden).
  showCode: {
    default: false,
    parseHTML: (el: HTMLElement) => el.getAttribute("data-show-code") === "true",
    renderHTML: (attrs: Record<string, unknown>) => (attrs.showCode ? { "data-show-code": "true" } : {}),
  },
};

export const MermaidBlock = Node.create({
  name: "mermaidBlock",
  group: "block",
  atom: true,
  selectable: true,
  draggable: true,
  // Beats the code block's own "```mermaid" rule.
  priority: 1000,
  addStorage() {
    return { justInserted: { current: false } };
  },
  addAttributes() {
    return codeAttr;
  },
  parseHTML() {
    return [{ tag: "div[data-mermaid-block]" }];
  },
  renderHTML({ HTMLAttributes }) {
    return ["div", mergeAttributes(HTMLAttributes, { "data-mermaid-block": "", class: "mermaid-block" })];
  },
  // What the plain-text copy of the page (search, word count) holds for the diagram.
  renderText: ({ node }) => String(node.attrs.code ?? ""),
  addNodeView() {
    return mermaidView(this.storage.justInserted);
  },
  addCommands() {
    return {
      insertMermaid:
        (code) =>
        ({ commands }) => {
          const text = (code ?? "").slice(0, MERMAID_MAX_CHARS);
          this.storage.justInserted.current = true;
          return commands.insertContent([{ type: this.name, attrs: { code: text || MERMAID_TEMPLATES[0].code, showCode: true } }, { type: "paragraph" }]);
        },
    };
  },
  addInputRules() {
    const type = this.type;
    return [
      // "```mermaid" and a space or Enter at the start of a line makes a diagram instead of a code block.
      new InputRule({
        find: /^```mermaid[\s\n]$/,
        handler: ({ state, range }) => {
          const $from = state.doc.resolve(range.from);
          const start = $from.before();
          const end = $from.after();
          (this.storage as { justInserted: { current: boolean } }).justInserted.current = true;
          state.tr.replaceWith(start, end, [type.create({ code: "", showCode: true }), state.schema.nodes.paragraph.create()]);
        },
      }),
    ];
  },
});
