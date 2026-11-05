// More pieces: links to sub-pages, and columns and comments. Columns and comments can no longer be created,
// but pages that already contain them still show them (nothing is dropped from older pages).

import { Mark, Node, mergeAttributes } from "@tiptap/react";
import { NodeSelection, Plugin } from "@tiptap/pm/state";
import { useApp } from "@/store/app";
import { goToPage, livePage } from "./internalLinks";
import { iconText } from "@/lib/pageLook";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    comment: {
      setComment: (text: string) => ReturnType;
      unsetComment: () => ReturnType;
    };
  }
}

/** Two to five blocks side by side. */
export const Columns = Node.create({
  name: "columns",
  group: "block",
  content: "column{2,5}",
  isolating: true,
  defining: true,
  parseHTML() {
    return [{ tag: "div[data-columns]" }];
  },
  renderHTML({ HTMLAttributes }) {
    return ["div", mergeAttributes(HTMLAttributes, { "data-columns": "", class: "columns" }), 0];
  },
});

export const Column = Node.create({
  name: "column",
  content: "block+",
  isolating: true,
  defining: true,
  parseHTML() {
    return [{ tag: "div[data-column]" }];
  },
  renderHTML({ HTMLAttributes }) {
    return ["div", mergeAttributes(HTMLAttributes, { "data-column": "", class: "column" }), 0];
  },
});

/** A line that links to a page inside this one. Click it to open the page. */
export const PageLink = Node.create({
  name: "pageLink",
  group: "block",
  atom: true,
  selectable: true,
  draggable: true,

  addAttributes() {
    return {
      pageId: {
        default: "",
        parseHTML: (el) => el.getAttribute("data-page-id") ?? "",
        renderHTML: (attrs) => ({ "data-page-id": attrs.pageId }),
      },
    };
  },

  parseHTML() {
    return [{ tag: "div[data-page-link]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return ["div", mergeAttributes(HTMLAttributes, { "data-page-link": "", class: "page-link" })];
  },

  // With a page link selected (arrow keys or a click on its edge), Enter or Space opens the page.
  addProseMirrorPlugins() {
    return [
      new Plugin({
        props: {
          handleKeyDown(view, event) {
            if (event.key !== "Enter" && event.key !== " ") return false;
            if (event.ctrlKey || event.metaKey || event.altKey || event.shiftKey) return false;
            const sel = view.state.selection;
            if (!(sel instanceof NodeSelection) || sel.node.type.name !== "pageLink") return false;
            event.preventDefault();
            goToPage(sel.node.attrs.pageId as string);
            return true;
          },
        },
      }),
    ];
  },

  addNodeView() {
    return ({ node }) => {
      let pageId = node.attrs.pageId as string;
      const dom = document.createElement("div");
      dom.className = "page-link";
      dom.contentEditable = "false";
      dom.setAttribute("role", "link");
      const icon = document.createElement("span");
      icon.className = "page-link-icon";
      const title = document.createElement("span");
      title.className = "page-link-title";
      title.dir = "auto";
      dom.append(icon, title);

      const paint = () => {
        const page = livePage(pageId);
        icon.textContent = iconText(page?.icon) || "📄";
        title.textContent = page ? page.title || "Untitled" : "Page not found";
        dom.dataset.missing = page ? "false" : "true";
      };
      paint();
      const unsubscribe = useApp.subscribe(paint);

      dom.addEventListener("click", () => goToPage(pageId));

      return {
        dom,
        update(updated) {
          if (updated.type.name !== "pageLink") return false;
          pageId = updated.attrs.pageId;
          paint();
          return true;
        },
        destroy: unsubscribe,
        ignoreMutation: () => true,
      };
    };
  },
});

/** A note attached to some text. The note itself is stored (encrypted) inside the page. */
export const Comment = Mark.create({
  name: "comment",
  inclusive: false,

  addAttributes() {
    return {
      text: {
        default: "",
        parseHTML: (el) => el.getAttribute("data-comment") ?? "",
        renderHTML: (attrs) => ({ "data-comment": attrs.text, title: attrs.text || "Comment" }),
      },
    };
  },

  parseHTML() {
    return [{ tag: "span[data-comment]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return ["span", mergeAttributes(HTMLAttributes, { class: "comment-mark" }), 0];
  },

  addCommands() {
    return {
      setComment:
        (text) =>
        ({ commands }) =>
          commands.setMark(this.name, { text }),
      unsetComment:
        () =>
        ({ chain }) =>
          chain().extendMarkRange(this.name).unsetMark(this.name).run(),
    };
  },
});
