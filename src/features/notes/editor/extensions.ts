import StarterKit from "@tiptap/starter-kit";
import { Color, FontFamily, FontSize, TextStyle } from "@tiptap/extension-text-style";
import Highlight from "@tiptap/extension-highlight";
import { TaskItem, TaskList } from "@tiptap/extension-list";
import { TableKit } from "@tiptap/extension-table";
import Image from "@tiptap/extension-image";
import { Callout, Toggle } from "./blocks";
import { Column, Columns, Comment, PageLink } from "./more-blocks";
import { MathBlock, MathInline } from "./math";
import { MermaidBlock } from "./MermaidBlock";
import { DatabaseBlock } from "./DatabaseBlock";
import { ImageBlock } from "./ImageBlock";
import { SlashCommand, type SlashOptions } from "./slash";
import { Direction } from "./direction";
import { followInternalLink, InternalLinks, isInternalHref, isOwnScheme } from "./internalLinks";
import { Placeholder } from "@tiptap/extensions";
import CodeBlockLowlight from "@tiptap/extension-code-block-lowlight";
import TextAlign from "@tiptap/extension-text-align";
import { common, createLowlight } from "lowlight";
import { Extension, type Editor } from "@tiptap/react";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";
import { convertFileSrc } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";

const lowlight = createLowlight(common);

const BLOCK_TYPES = [
  "paragraph", "heading", "blockquote", "bulletList", "orderedList", "taskList",
  "listItem", "taskItem", "codeBlock", "horizontalRule", "image", "table",
  "tableRow", "tableCell", "tableHeader", "callout", "toggle", "columns", "column",
  "mathBlock", "mermaidBlock", "pageLink", "databaseBlock",
];
const blockIdsKey = new PluginKey("notayaBlockIds");
const validBid = (id: unknown): id is string => typeof id === "string" && /^[0-9a-fA-F]{16}$/.test(id);

/** Native block identities survive JSON, HTML, splits and pasted duplicates. */
export const BlockIds = Extension.create({
  name: "notayaBlockIds",
  addGlobalAttributes() {
    return [{
      types: BLOCK_TYPES,
      attributes: {
        bid: {
          default: null,
          parseHTML: (el: HTMLElement) => {
            const id = el.getAttribute("data-bid");
            return validBid(id) ? id : null;
          },
          renderHTML: (attrs: Record<string, unknown>) => validBid(attrs.bid) ? { "data-bid": attrs.bid } : {},
        },
      },
    }];
  },
  onCreate() {
    // Appended id assignments must not mark a legacy page dirty merely on open.
    this.editor.view.dispatch(this.editor.state.tr.setMeta(blockIdsKey, true).setMeta("preventUpdate", true));
  },
  addProseMirrorPlugins() {
    return [new Plugin({
      key: blockIdsKey,
      appendTransaction(transactions, old, state) {
        if (!transactions.some((tr) => tr.docChanged || tr.getMeta(blockIdsKey))) return null;
        // A pasted copy can precede its source. Keep the identity at the mapped
        // original position, rather than letting the first duplicate steal it.
        const originals = new Map<string, number>();
        old.doc.descendants((node, pos) => {
          if (!BLOCK_TYPES.includes(node.type.name) || !validBid(node.attrs.bid)) return;
          let mapped = pos;
          for (const tr of transactions) mapped = tr.mapping.map(mapped, 1);
          if (state.doc.nodeAt(mapped)?.attrs.bid === node.attrs.bid) originals.set(node.attrs.bid, mapped);
        });
        const reserved = new Set<string>();
        state.doc.descendants((node) => {
          if (BLOCK_TYPES.includes(node.type.name) && validBid(node.attrs.bid)) reserved.add(node.attrs.bid);
        });
        const seen = new Set<string>();
        const tr = state.tr;
        state.doc.descendants((node, pos) => {
          if (!BLOCK_TYPES.includes(node.type.name)) return;
          let id: unknown = node.attrs.bid;
          if (!validBid(id) || seen.has(id) || (originals.has(id) && originals.get(id) !== pos)) {
            do {
              id = Array.from(crypto.getRandomValues(new Uint8Array(8)), (b) => b.toString(16).padStart(2, "0")).join("");
            } while (reserved.has(id as string));
            reserved.add(id as string);
            tr.setNodeMarkup(pos, undefined, { ...node.attrs, bid: id });
          }
          seen.add(id as string);
        });
        return tr.docChanged ? tr.setMeta("addToHistory", false) : null;
      },
    })];
  },
});

const link = {
  openOnClick: false,
  autolink: true,
  defaultProtocol: "https",
  HTMLAttributes: { rel: "noopener noreferrer nofollow", target: null },
} as const;

// Notebook pages may also hold links to other pages and headings (`notaya:page/...`).
const pageLinkConfig = {
  ...link,
  isAllowedUri: (url: string, ctx: { defaultValidate: (url: string) => boolean }) => isInternalHref(url) || ctx.defaultValidate(url),
};

// Resolve the caret against the state being drawn. TipTap's incremental
// descendant placeholder cache can otherwise retain hints from prior selections.
const CaretPlaceholder = Placeholder.extend({
  addProseMirrorPlugins() {
    const { editor, options } = this;
    return [new Plugin({
      props: {
        decorations({ doc, selection }) {
          if (!editor.isEditable) return null;
          const first = doc.firstChild;
          const emptyDocument = doc.childCount === 1 && first?.type.name === "paragraph" && first.content.size === 0;
          const { $head, empty } = selection;
          if (!emptyDocument && (!empty || $head.depth === 0)) return null;
          const node = emptyDocument ? first : $head.parent;
          if (!node?.isTextblock || node.content.size > 0) return null;
          const pos = emptyDocument ? 0 : $head.before();
          const text = typeof options.placeholder === "function"
            ? options.placeholder({ editor, node, pos, hasAnchor: true })
            : options.placeholder;
          if (!text) return null;
          return DecorationSet.create(doc, [Decoration.node(pos, pos + node.nodeSize, {
            class: `is-empty${emptyDocument ? " is-editor-empty" : ""}`,
            "data-placeholder": text,
          })]);
        },
      },
    })];
  },
});

function editorPlaceholder(text: string, headings = false) {
  return CaretPlaceholder.configure({
    placeholder: ({ node }) => {
      if (node.type.name === "paragraph") return text;
      if (headings && node.type.name === "heading") return `Heading ${node.attrs.level}`;
      return "";
    },
  });
}

/** Full editor for notebook pages: block types, the "/" menu and pictures. */
export function pageExtensions(slash: SlashOptions) {
  return [
    BlockIds,
    StarterKit.configure({ codeBlock: false, heading: { levels: [1, 2, 3, 4] }, link: pageLinkConfig }),
    InternalLinks,
    TextStyle,
    Color,
    FontSize,
    FontFamily,
    Highlight.configure({ multicolor: true }),
    TaskList,
    TaskItem.configure({ nested: true }),
    TableKit.configure({ table: { resizable: true } }),
    // Only images stored in the vault (or pasted files) are allowed; remote
    // images are also blocked by the Content Security Policy.
    ImageBlock.configure({ allowBase64: false }),
    Callout,
    Toggle,
    Columns,
    Column,
    PageLink,
    Comment,
    MathBlock,
    MathInline,
    MermaidBlock,
    DatabaseBlock,
    editorPlaceholder("Press / to insert something, or just start typing", true),
    CodeBlockLowlight.configure({ lowlight }),
    TextAlign.configure({ types: ["heading", "paragraph"] }),
    Direction,
    SlashCommand.configure(slash),
  ];
}

/** Lighter editor for sticky notes. */
export function stickyExtensions() {
  return [
    StarterKit.configure({ codeBlock: false, heading: { levels: [2, 3] }, link }),
    Highlight,
    TaskList,
    TaskItem.configure({ nested: true }),
    Image.configure({ allowBase64: false }),
    Direction,
    editorPlaceholder("Write something…"),
  ];
}

const IMAGE_TYPES = ["image/png", "image/jpeg", "image/gif", "image/webp"];

/** Encrypts an image into the vault and returns the URL the editor can show. */
export async function uploadImage(file: File): Promise<string | null> {
  if (!IMAGE_TYPES.includes(file.type)) {
    toast.error("Only PNG, JPEG, GIF and WebP images can be added.");
    return null;
  }
  try {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const id = await api.addAttachment(bytes);
    return convertFileSrc(id, "clatt");
  } catch (e) {
    toast.error(errorText(e));
    return null;
  }
}

/** Opens the file chooser and inserts the chosen pictures at the cursor. */
export function pickImages(editor: Editor) {
  const input = document.createElement("input");
  input.type = "file";
  input.accept = IMAGE_TYPES.join(",");
  input.multiple = true;
  input.onchange = () => {
    const files = Array.from(input.files ?? []);
    if (files.length) insertImages(editor, files);
  };
  input.click();
}

export async function insertImages(editor: Editor, files: File[], pos?: number) {
  for (const file of files) {
    const src = await uploadImage(file);
    if (!src) continue;
    const chain = editor.chain().focus();
    if (pos !== undefined) chain.insertContentAt(pos, { type: "image", attrs: { src } }).run();
    else chain.setImage({ src }).run();
  }
}

/** Shared paste/drop/click handling: images become encrypted attachments, Ctrl+click opens links. */
export function editorProps(getEditor: () => Editor | null) {
  return {
    handlePaste: (_view: unknown, event: ClipboardEvent) => {
      const files = Array.from(event.clipboardData?.files ?? []).filter((f) => f.type.startsWith("image/"));
      const editor = getEditor();
      if (files.length && editor) {
        event.preventDefault();
        insertImages(editor, files);
        return true;
      }
      return false;
    },
    handleDrop: (view: { posAtCoords: (c: { left: number; top: number }) => { pos: number } | null }, event: DragEvent) => {
      const files = Array.from(event.dataTransfer?.files ?? []).filter((f) => f.type.startsWith("image/"));
      const editor = getEditor();
      if (files.length && editor) {
        event.preventDefault();
        const pos = view.posAtCoords({ left: event.clientX, top: event.clientY })?.pos;
        insertImages(editor, files, pos);
        return true;
      }
      return false;
    },
    // An internal link must never take the window (or the operating system) anywhere.
    handleDOMEvents: {
      click: (_view: unknown, event: Event) => {
        const a = (event.target as HTMLElement | null)?.closest?.("a");
        if (a && isOwnScheme(a.getAttribute("href"))) event.preventDefault();
        return false;
      },
    },
    handleClick: (_view: unknown, _pos: number, event: MouseEvent) => {
      const a = (event.target as HTMLElement).closest("a");
      const href = a?.getAttribute("href");
      if (!a || !href) return false;
      if (isOwnScheme(href)) {
        // A click goes to the page or heading. Shift/Alt+click or dragging a selection keeps editing the text instead.
        if (!isInternalHref(href) || event.shiftKey || event.altKey || event.button !== 0 || window.getSelection()?.isCollapsed === false) return false;
        followInternalLink(href, getEditor());
        return true;
      }
      if (event.ctrlKey) {
        api.openUrl(href).catch((e) => toast.error(errorText(e)));
        return true;
      }
      return false;
    },
  };
}

export function parseContent(json: string) {
  if (!json) return "";
  try {
    return JSON.parse(json);
  } catch {
    return "";
  }
}
