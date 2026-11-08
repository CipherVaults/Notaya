import type { Editor } from "@tiptap/react";

/** Save/reload metadata is not an edit. TipTap otherwise emits `update` even
 * when editability is unchanged, feeding that notification back into autosave.
 */
export function setPageEditable(editor: Pick<Editor, "setEditable">, editable: boolean) {
  editor.setEditable(editable, false);
}
