// Links inside Notaya: text that jumps to another page, or to a heading on the same or another page.
//
// An internal link is the ordinary link mark whose address is `notaya:page/<pageId>` or
// `notaya:page/<pageId>#<anchor>`. Headings carry a short random `anchor` (stored in the page, never shown).
// A page that has not been edited since anchors existed has none yet; its headings are then addressed by their
// text instead: `#t:<heading text, percent-encoded>`.
//
// A `notaya:` address is never passed to the operating system or the browser: it is only ever handled here.

import { Extension, type Editor } from "@tiptap/react";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";
import type { Node as PMNode } from "@tiptap/pm/model";
import { toast } from "sonner";
import { useApp } from "@/store/app";

const PREFIX = "notaya:page/";
const HREF = /^notaya:page\/([A-Za-z0-9_-]+)(?:#(.*))?$/;

export interface InternalTarget {
  pageId: string;
  /** The heading's anchor, when the link points at a heading. */
  anchor?: string;
  /** The heading's text, used when the heading has no anchor yet. */
  text?: string;
}

export function isInternalHref(href: unknown): href is string {
  return typeof href === "string" && HREF.test(href.trim());
}

/** True for any address in our own scheme, valid or not: these must never be opened outside the app. */
export function isOwnScheme(href: unknown): boolean {
  return typeof href === "string" && /^\s*notaya:/i.test(href);
}

export function parseInternalHref(href: string): InternalTarget | null {
  const m = HREF.exec(href.trim());
  if (!m) return null;
  const target: InternalTarget = { pageId: m[1] };
  const frag = m[2];
  if (frag) {
    if (frag.startsWith("t:")) {
      try {
        target.text = decodeURIComponent(frag.slice(2));
      } catch {
        /* an unreadable fragment just means "the page" */
      }
    } else target.anchor = frag;
  }
  return target;
}

export function makeInternalHref(pageId: string, heading?: { anchor?: string | null; text: string } | null): string {
  if (!heading) return PREFIX + pageId;
  if (heading.anchor) return `${PREFIX}${pageId}#${heading.anchor}`;
  return `${PREFIX}${pageId}#t:${encodeURIComponent(heading.text.trim())}`;
}

export function newAnchor(): string {
  const bytes = new Uint8Array(5);
  crypto.getRandomValues(bytes);
  return "h" + Array.from(bytes, (b) => b.toString(36).padStart(2, "0")).join("").slice(0, 8);
}

/** The page a link points to, or undefined when it is gone, in the trash, or was never imported. */
export function livePage(pageId: string) {
  return useApp.getState().tree.pages.find((p) => p.id === pageId && p.trashedAt === null);
}

/** Goes to a page, and to its notebook and section in the sidebar (it may be elsewhere). */
export function goToPage(pageId: string) {
  const page = livePage(pageId);
  if (!page) return;
  const s = useApp.getState();
  const section = s.tree.sections.find((x) => x.id === page.sectionId);
  if (section) useApp.setState({ activeNotebookId: section.notebookId, activeSectionId: section.id });
  s.openPage(pageId);
}

/* ---------- finding a heading ---------- */

const flashKey = new PluginKey<number | null>("anchorFlash");

function headingPos(doc: PMNode, target: InternalTarget): number | null {
  let byAnchor: number | null = null;
  let byText: number | null = null;
  const want = target.text?.trim().toLowerCase();
  doc.descendants((node, pos) => {
    if (node.type.name !== "heading") return;
    if (target.anchor && byAnchor === null && node.attrs.anchor === target.anchor) byAnchor = pos;
    if (want && byText === null && node.textContent.trim().toLowerCase() === want) byText = pos;
  });
  return byAnchor ?? byText;
}

/** Scrolls to the heading a link points at and flashes it. Returns false when nothing matched. */
export function jumpInEditor(editor: Editor, target: InternalTarget): boolean {
  if (!target.anchor && !target.text) return false;
  let pos = headingPos(editor.state.doc, target);
  // A target without an anchor in a text link may still be found by its text.
  if (pos === null && target.anchor && target.text) pos = headingPos(editor.state.doc, { pageId: target.pageId, text: target.text });
  if (pos === null) return false;

  // Show a heading that sits inside a collapsed section.
  if (editor.isEditable) {
    const $pos = editor.state.doc.resolve(pos);
    const tr = editor.state.tr;
    let changed = false;
    for (let d = $pos.depth; d >= 1; d--) {
      const anc = $pos.node(d);
      if (anc.type.name === "toggle" && anc.attrs.open === false && $pos.index(d) !== 0) {
        tr.setNodeMarkup($pos.before(d), undefined, { ...anc.attrs, open: true });
        changed = true;
      }
    }
    if (changed) editor.view.dispatch(tr);
  }

  const dom = editor.view.nodeDOM(pos) as HTMLElement | null;
  dom?.scrollIntoView?.({ block: "center", behavior: "smooth" });
  editor.view.dispatch(editor.state.tr.setMeta(flashKey, pos));
  window.setTimeout(() => {
    if (!editor.isDestroyed) editor.view.dispatch(editor.state.tr.setMeta(flashKey, null));
  }, 1700);
  return true;
}

/* ---------- following a link ---------- */

let pending: { pageId: string; target: InternalTarget } | null = null;

/** Called by the editor of a freshly opened page: carries out a jump asked for before the page loaded. */
export function consumePendingJump(editor: Editor, pageId: string) {
  if (!pending || pending.pageId !== pageId) return;
  const { target } = pending;
  pending = null;
  window.setTimeout(() => {
    if (!editor.isDestroyed) jumpInEditor(editor, target);
  }, 120);
}

/** Opens the page an internal link points at (and its heading). Shows "Page not found" when it is gone. */
export function followInternalLink(href: string, editor: Editor | null) {
  const target = parseInternalHref(href);
  if (!target || !livePage(target.pageId)) {
    toast.error("Page not found");
    return;
  }
  const active = useApp.getState().activePageId;
  if (target.pageId === active && editor) {
    jumpInEditor(editor, target);
    return;
  }
  pending = { pageId: target.pageId, target };
  goToPage(target.pageId);
}

/* ---------- asking the editor to open the picker ---------- */

type Listener = (editor: Editor) => void;
const listeners = new Set<Listener>();

export function onLinkPickerRequest(fn: Listener) {
  listeners.add(fn);
  return () => void listeners.delete(fn);
}

export function requestInternalLinkPicker(editor: Editor) {
  listeners.forEach((fn) => fn(editor));
}

/* ---------- the extension: heading anchors, hover titles, the flash ---------- */

export const InternalLinks = Extension.create({
  name: "internalLinks",

  addGlobalAttributes() {
    return [
      {
        types: ["heading"],
        attributes: {
          anchor: {
            default: null,
            keepOnSplit: false,
            parseHTML: (el: HTMLElement) => el.getAttribute("data-anchor"),
            renderHTML: (attrs: Record<string, unknown>) => (attrs.anchor ? { "data-anchor": String(attrs.anchor) } : {}),
          },
        },
      },
    ];
  },

  addProseMirrorPlugins() {
    const editor = this.editor;
    let cachedDoc: PMNode | null = null;
    let cachedSet = DecorationSet.empty;

    return [
      // Every heading gets an anchor of its own the next time the page is edited.
      new Plugin({
        key: new PluginKey("headingAnchors"),
        appendTransaction(transactions, _old, state) {
          if (!editor.isEditable || !transactions.some((t) => t.docChanged)) return null;
          const seen = new Set<string>();
          const fix: number[] = [];
          state.doc.descendants((node, pos) => {
            if (node.type.name !== "heading") return false;
            const a = node.attrs.anchor as string | null;
            if (!a || seen.has(a)) fix.push(pos);
            else seen.add(a);
            return false;
          });
          if (!fix.length) return null;
          const tr = state.tr;
          for (const pos of fix) {
            let id = newAnchor();
            while (seen.has(id)) id = newAnchor();
            seen.add(id);
            tr.setNodeMarkup(pos, undefined, { ...state.doc.nodeAt(pos)!.attrs, anchor: id });
          }
          return tr.setMeta("addToHistory", false);
        },
      }),

      // Internal links say where they go when you hover them.
      new Plugin({
        key: new PluginKey("internalLinkTitles"),
        props: {
          decorations(state) {
            if (state.doc === cachedDoc) return cachedSet;
            const decos: Decoration[] = [];
            state.doc.descendants((node, pos) => {
              if (!node.isText) return;
              const mark = node.marks.find((m) => m.type.name === "link" && isInternalHref(m.attrs.href));
              if (!mark) return;
              const target = parseInternalHref(String(mark.attrs.href));
              if (!target) return;
              const page = livePage(target.pageId);
              const title = page ? `Open ${page.title || "Untitled"}${target.anchor || target.text ? " (section)" : ""}` : "Page not found";
              decos.push(Decoration.inline(pos, pos + node.nodeSize, { class: "internal-link", title }));
            });
            cachedDoc = state.doc;
            cachedSet = DecorationSet.create(state.doc, decos);
            return cachedSet;
          },
        },
      }),

      // A heading you jumped to is lit up for a moment.
      new Plugin<number | null>({
        key: flashKey,
        state: {
          init: () => null,
          apply(tr, value) {
            const meta = tr.getMeta(flashKey) as number | null | undefined;
            if (meta !== undefined) return meta;
            return value === null ? null : tr.mapping.map(value);
          },
        },
        props: {
          decorations(state) {
            const pos = flashKey.getState(state);
            const node = pos === null || pos === undefined ? null : state.doc.nodeAt(pos);
            return node ? DecorationSet.create(state.doc, [Decoration.node(pos!, pos! + node.nodeSize, { class: "anchor-flash" })]) : null;
          },
        },
      }),
    ];
  },
});
