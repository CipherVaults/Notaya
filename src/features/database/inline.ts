// Putting a database into a page: a new inline database, or a linked view of one that exists.

import type { Editor } from "@tiptap/react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { useApp } from "@/store/app";
import { editDatabase, loadDatabase } from "./dbStore";
import { newId } from "./model";

/** Inserts the block at the cursor: in place of an empty paragraph, else below the current block. */
function insertBlock(editor: Editor, databaseId: string, viewId: string) {
  const { $from } = editor.state.selection;
  const nodes = [{ type: "databaseBlock", attrs: { databaseId, viewId } }, { type: "paragraph" }];
  let at: number | { from: number; to: number } = editor.state.selection.to;
  for (let d = $from.depth; d >= 1; d--) {
    const container = $from.node(d - 1);
    const idx = $from.index(d - 1);
    if (!container.canReplaceWith(idx, idx, editor.schema.nodes.databaseBlock)) continue;
    const node = $from.node(d);
    const empty = d === $from.depth && node.type.name === "paragraph" && node.content.size === 0;
    at = empty ? { from: $from.before(d), to: $from.after(d) } : $from.after(d);
    break;
  }
  editor.chain().focus().insertContentAt(at, nodes).run();
}

/** A new database that lives inside the page and shows as a table there. */
export async function insertInlineDatabase(editor: Editor, page: { id: string; sectionId: string }) {
  try {
    const { database } = await api.createDatabase(page.sectionId, page.id, true, "Untitled database", "blank");
    useApp.getState().setTree(await api.listNotes());
    insertBlock(editor, database.id, database.views[0]?.id ?? "");
  } catch (e) {
    toast.error(errorText(e));
  }
}

/**
 * A view of an existing database inside the page. It gets a view of its own (a copy of the database's first view),
 * so its filter, sort and layout can differ from the other views. A locked database cannot get a new view,
 * so the block shows its first view then.
 */
export async function insertLinkedView(editor: Editor, databaseId: string) {
  try {
    const db = await loadDatabase(databaseId);
    let viewId = db.views[0]?.id ?? "";
    if (!db.locked && db.views[0]) {
      const copy = { ...structuredClone(db.views[0]), id: newId(), name: `${db.views[0].name || "Table"} (linked)` };
      await editDatabase(databaseId, (d) => ({ ...d, views: [...d.views, copy] }), true);
      viewId = copy.id;
    } else if (db.locked) {
      toast.info("This database is locked, so the page shows its first view.");
    }
    insertBlock(editor, databaseId, viewId);
  } catch (e) {
    toast.error(errorText(e));
  }
}
