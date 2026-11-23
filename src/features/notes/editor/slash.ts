// "/" menu: typing a slash at the start of a line or after a space opens a block picker.
// This file only detects the slash and forwards keys; SlashMenu.tsx draws the menu.

import { Extension } from "@tiptap/react";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { openBlockMenu, openRadixPopup } from "./openPopup";

export interface SlashState {
  /** Document range covering the "/" and the text typed after it. */
  from: number;
  to: number;
  query: string;
  /** Screen position just under the caret. */
  left: number;
  top: number;
  bottom: number;
}

export interface SlashOptions {
  onState: (state: SlashState | null) => void;
  /** Called for arrow/enter/tab while the menu is open; return true if handled. */
  onKey: (event: KeyboardEvent) => boolean;
  getMenu?: () => HTMLElement | null;
}

const key = new PluginKey("slashMenu");

export const SlashCommand = Extension.create<SlashOptions>({
  name: "slashMenu",

  addOptions() {
    return { onState: () => {}, onKey: () => false };
  },

  addProseMirrorPlugins() {
    const options = this.options;
    // A slash the user dismissed with Escape stays closed until it is typed again.
    let dismissedAt = -1;
    // React re-renders make the editor re-check its plugins, so only report real changes.
    let last: SlashState | null = null;
    const report = (next: SlashState | null) => {
      const same =
        last === next ||
        (last !== null &&
          next !== null &&
          last.from === next.from &&
          last.to === next.to &&
          last.query === next.query &&
          last.left === next.left &&
          last.top === next.top);
      if (same) return;
      last = next;
      options.onState(next);
    };

    const find = (state: import("@tiptap/pm/state").EditorState) => {
      const { selection } = state;
      if (!selection.empty) return null;
      const { $from } = selection;
      if (!$from.parent.isTextblock || $from.parent.type.spec.code) return null;
      const before = $from.parent.textBetween(0, $from.parentOffset, undefined, "￼");
      const m = /(?:^|\s)\/([\p{L}\p{N}-]*(?: [\p{L}\p{N}-]*)?)$/u.exec(before);
      if (!m) return null;
      const from = $from.pos - m[1].length - 1;
      return { from, to: $from.pos, query: m[1] };
    };

    const dismiss = (from: number) => {
      dismissedAt = from;
      report(null);
    };

    return [
      new Plugin({
        key,
        props: {
          handleKeyDown(view, event) {
            const found = find(view.state);
            if (!found || found.from === dismissedAt) return false;
            if (event.key === "Escape") {
              dismiss(found.from);
              return true;
            }
            if (["ArrowDown", "ArrowUp", "Enter", "Tab"].includes(event.key)) return options.onKey(event);
            return false;
          },
        },
        view(view) {
          const doc = view.dom.ownerDocument;
          const onEscape = (event: KeyboardEvent) => {
            const menu = options.getMenu?.();
            if (event.key !== "Escape" || event.defaultPrevented || event.isComposing || !menu || !(event.target instanceof Element)) return;
            const owner = view.dom.closest("[data-note-editor]");
            if (event.target.closest('[data-note-editor], [data-database-view], [role="dialog"]') !== owner && !menu.contains(event.target)) return;
            if (doc.querySelector(`${openRadixPopup}, ${openBlockMenu}`)) return;
            const found = find(view.state);
            if (!found || found.from === dismissedAt) return;
            // The centered row dialog also handles Escape in capture.
            event.preventDefault();
            event.stopPropagation();
            if (menu.contains(event.target)) view.focus();
            dismiss(found.from);
          };
          doc.defaultView?.addEventListener("keydown", onEscape, true);
          return {
            update(view) {
              const found = find(view.state);
              if (!found || found.from === dismissedAt) {
                if (!found) dismissedAt = -1;
                report(null);
                return;
              }
              const c = view.coordsAtPos(found.to);
              report({ ...found, left: c.left, top: c.top, bottom: c.bottom });
            },
            destroy() {
              doc.defaultView?.removeEventListener("keydown", onEscape, true);
              report(null);
            },
          };
        },
      }),
    ];
  },
});
