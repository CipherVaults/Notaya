// Blocks that StarterKit doesn't have: highlight boxes (callouts) and collapsible sections (toggles).

import { Node, mergeAttributes } from "@tiptap/react";

export const CALLOUT_ICONS = ["💡", "ℹ️", "⚠️", "❗", "✅", "📌", "⭐", "🔥"];

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    callout: { insertCallout: () => ReturnType };
    toggle: { insertToggle: () => ReturnType };
  }
}

/** A highlighted box with an emoji. Click the emoji to cycle through a few. */
export const Callout = Node.create({
  name: "callout",
  group: "block",
  content: "block+",
  defining: true,

  addAttributes() {
    return {
      icon: {
        default: CALLOUT_ICONS[0],
        parseHTML: (el) => el.getAttribute("data-icon") ?? CALLOUT_ICONS[0],
        renderHTML: (attrs) => ({ "data-icon": attrs.icon }),
      },
    };
  },

  parseHTML() {
    return [{ tag: "div[data-callout]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return ["div", mergeAttributes(HTMLAttributes, { "data-callout": "", class: "callout" }), 0];
  },

  addNodeView() {
    return ({ node, getPos, editor }) => {
      let current = node;
      const dom = document.createElement("div");
      dom.className = "callout";
      const icon = document.createElement("span");
      icon.className = "callout-icon";
      icon.contentEditable = "false";
      icon.title = "Change icon";
      icon.textContent = node.attrs.icon;
      const body = document.createElement("div");
      body.className = "callout-body";
      dom.append(icon, body);

      icon.addEventListener("mousedown", (e) => e.preventDefault());
      icon.addEventListener("click", () => {
        if (!editor.isEditable) return;
        const pos = getPos();
        if (pos === undefined) return;
        const i = CALLOUT_ICONS.indexOf(current.attrs.icon);
        const next = CALLOUT_ICONS[(i + 1) % CALLOUT_ICONS.length];
        editor.view.dispatch(editor.state.tr.setNodeMarkup(pos, undefined, { ...current.attrs, icon: next }));
      });

      return {
        dom,
        contentDOM: body,
        update(updated) {
          if (updated.type !== current.type) return false;
          current = updated;
          icon.textContent = updated.attrs.icon;
          return true;
        },
      };
    };
  },

  addCommands() {
    return {
      insertCallout:
        () =>
        ({ commands }) =>
          commands.insertContent({ type: this.name, content: [{ type: "paragraph" }] }),
    };
  },
});

const CHEVRON =
  '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="m9 18 6-6-6-6"/></svg>';

/** A block whose first line is a heading you can click to show or hide the rest. */
export const Toggle = Node.create({
  name: "toggle",
  group: "block",
  content: "block+",
  defining: true,

  addAttributes() {
    return {
      open: {
        default: true,
        parseHTML: (el) => el.getAttribute("data-open") !== "false",
        renderHTML: (attrs) => ({ "data-open": String(attrs.open) }),
      },
      // "Shelf" look: a filled, rounded box. Pages without this attribute keep the plain look.
      shelf: {
        default: false,
        parseHTML: (el) => el.hasAttribute("data-shelf") && el.getAttribute("data-shelf") !== "false",
        renderHTML: (attrs) => (attrs.shelf ? { "data-shelf": "true" } : {}),
      },
    };
  },

  parseHTML() {
    return [{ tag: "div[data-toggle]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return ["div", mergeAttributes(HTMLAttributes, { "data-toggle": "", class: "toggle" }), 0];
  },

  addNodeView() {
    return ({ node, getPos, editor }) => {
      let current = node;
      const dom = document.createElement("div");
      dom.className = "toggle";
      dom.dataset.open = String(node.attrs.open);
      dom.dataset.shelf = String(!!node.attrs.shelf);
      const row = document.createElement("div");
      row.className = "toggle-row";
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "toggle-btn";
      btn.contentEditable = "false";
      btn.setAttribute("aria-label", "Show or hide");
      btn.innerHTML = CHEVRON;
      const body = document.createElement("div");
      body.className = "toggle-body";
      row.append(btn, body);
      dom.append(row);

      btn.addEventListener("mousedown", (e) => e.preventDefault());
      btn.addEventListener("click", () => {
        const pos = getPos();
        if (pos === undefined) return;
        editor.view.dispatch(editor.state.tr.setNodeMarkup(pos, undefined, { ...current.attrs, open: !current.attrs.open }));
      });

      return {
        dom,
        contentDOM: body,
        update(updated) {
          if (updated.type !== current.type) return false;
          current = updated;
          dom.dataset.open = String(updated.attrs.open);
          dom.dataset.shelf = String(!!updated.attrs.shelf);
          return true;
        },
      };
    };
  },

  addCommands() {
    return {
      insertToggle:
        () =>
        ({ commands }) =>
          commands.insertContent({ type: this.name, attrs: { open: true }, content: [{ type: "paragraph" }, { type: "paragraph" }] }),
    };
  },
});
