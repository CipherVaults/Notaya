// A database inside a page: either one made in the page (inline) or a linked view of another database.
// The block only stores which database and which view; the data lives in the database. Removing the block
// from the page does not delete the database.

import { lazy, Suspense } from "react";
import { mergeAttributes, Node } from "@tiptap/react";
import { NodeViewWrapper, ReactNodeViewRenderer, type NodeViewProps } from "@tiptap/react";
import { Loader2, X } from "lucide-react";

// Loaded on first use. It also keeps the editor and the database code from importing each other at start-up.
const DatabaseView = lazy(() => import("@/features/database/DatabaseView"));

function DatabaseBlockView({ node, updateAttributes, deleteNode, editor, selected }: NodeViewProps) {
  const databaseId = String(node.attrs.databaseId || "");
  const viewId = String(node.attrs.viewId || "");
  return (
    <NodeViewWrapper
      className={`database-block group/dbblock relative my-3 ${selected ? "rounded-lg ring-2 ring-accent" : ""}`}
      contentEditable={false}
      data-database-block=""
      role="group"
      aria-label="Database"
    >
      {editor.isEditable && (
        <button
          type="button"
          onClick={deleteNode}
          aria-label="Remove this database from the page"
          title="Remove from page (the database is kept)"
          className="absolute end-1 top-1 z-10 grid h-6 w-6 place-items-center rounded-md bg-bg/90 text-faint opacity-0 shadow-sm transition-opacity hover:text-fg focus-visible:opacity-100 group-hover/dbblock:opacity-100"
        >
          <X size={14} />
        </button>
      )}
      {databaseId ? (
        <Suspense
          fallback={
            <div className="grid place-items-center py-8 text-muted">
              <Loader2 className="animate-spin" size={18} />
            </div>
          }
        >
          <DatabaseView key={databaseId} databaseId={databaseId} viewId={viewId || undefined} compact onViewChange={(id) => updateAttributes({ viewId: id })} />
        </Suspense>
      ) : (
        <div className="rounded-lg border border-dashed border-line p-4 text-sm text-muted">This database block is empty.</div>
      )}
    </NodeViewWrapper>
  );
}

export const DatabaseBlock = Node.create({
  name: "databaseBlock",
  group: "block",
  atom: true,
  selectable: true,
  draggable: false,

  addAttributes() {
    return {
      databaseId: {
        default: "",
        parseHTML: (el) => el.getAttribute("data-database-id") ?? "",
        renderHTML: (attrs) => ({ "data-database-id": attrs.databaseId }),
      },
      viewId: {
        default: "",
        parseHTML: (el) => el.getAttribute("data-view-id") ?? "",
        renderHTML: (attrs) => (attrs.viewId ? { "data-view-id": attrs.viewId } : {}),
      },
    };
  },

  parseHTML() {
    return [{ tag: "div[data-database-block]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return ["div", mergeAttributes(HTMLAttributes, { "data-database-block": "" })];
  },

  addNodeView() {
    return ReactNodeViewRenderer(DatabaseBlockView, {
      // Keys and pointer events inside the database belong to the database, not to the page editor.
      stopEvent: () => true,
      ignoreMutation: () => true,
    });
  },
});
