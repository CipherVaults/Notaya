import { useEffect, type Ref } from "react";
import { Extension, useEditorState, type Editor } from "@tiptap/react";
import type { Node as ProseMirrorNode } from "@tiptap/pm/model";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";
import { ChevronDown, ChevronUp, X } from "lucide-react";

type Match = { from: number; to: number };
type FindState = { query: string; matches: Match[]; index: number };
type FindAction = { query: string } | { step: number };

const findKey = new PluginKey<FindState>("noteFind");

function findMatches(doc: ProseMirrorNode, query: string): Match[] {
  if (!query) return [];
  const pattern = new RegExp(query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "giu");
  const matches: Match[] = [];
  doc.descendants((node, pos) => {
    if (!node.isTextblock) return;
    // Join adjacent text nodes so matches may cross formatting marks. Inline
    // objects and line breaks end the run and preserve ProseMirror offsets.
    let text = "";
    let start = 0;
    const collect = () => {
      for (const match of text.matchAll(pattern)) {
        const from = pos + 1 + start + match.index!;
        matches.push({ from, to: from + match[0].length });
      }
      text = "";
    };
    node.forEach((child, offset) => {
      if (child.text === undefined) {
        collect();
      } else {
        if (!text) start = offset;
        text += child.text;
      }
    });
    collect();
    return false;
  });
  return matches;
}

export const NoteFindExtension = Extension.create({
  name: "noteFind",
  addProseMirrorPlugins() {
    return [new Plugin<FindState>({
      key: findKey,
      state: {
        init: () => ({ query: "", matches: [], index: 0 }),
        apply(tr, state) {
          const action = tr.getMeta(findKey) as FindAction | undefined;
          if (!tr.docChanged && !action) return state;
          const query = action && "query" in action ? action.query : state.query;
          const matches = tr.docChanged || query !== state.query ? findMatches(tr.doc, query) : state.matches;
          const last = Math.max(0, matches.length - 1);
          let index = query === state.query ? Math.min(state.index, last) : 0;
          if (action && "step" in action && matches.length) {
            index = (index + action.step + matches.length) % matches.length;
          }
          return { query, matches, index };
        },
      },
      props: {
        decorations(state) {
          const find = findKey.getState(state);
          const active = find?.matches[find.index];
          return active ? DecorationSet.create(state.doc, [Decoration.inline(active.from, active.to, {
            "data-note-find-match": "active",
            class: "bg-accent/30 outline outline-1 outline-accent",
          })]) : null;
        },
      },
    })];
  },
});

function changeFind(editor: Editor, action: FindAction) {
  editor.view.dispatch(editor.state.tr.setMeta(findKey, action));
}

export function clearNoteFind(editor: Editor) {
  changeFind(editor, { query: "" });
}

const button = "rounded p-1 text-muted hover:bg-surface-2 hover:text-fg disabled:opacity-40";

export default function NoteFind({ editor, inputRef, onClose }: {
  editor: Editor;
  inputRef: Ref<HTMLInputElement>;
  onClose: () => void;
}) {
  const { query, count, index } = useEditorState({ editor, selector: ({ editor }) => {
    const find = findKey.getState(editor.state);
    return { query: find?.query ?? "", count: find?.matches.length ?? 0, index: find?.index ?? 0 };
  } });

  useEffect(() => {
    editor.view.dom.querySelector('[data-note-find-match="active"]')?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, [editor, query, count, index]);

  return (
    <div role="search" aria-label="Note find" className="flex shrink-0 items-center justify-end gap-1 border-b border-line px-3 py-1.5"
      onKeyDown={(e) => {
        if (e.key !== "Escape") return;
        e.preventDefault();
        onClose();
      }}>
      <input
        ref={inputRef}
        autoFocus
        type="search"
        aria-label="Find in note"
        placeholder="Find in note…"
        dir="auto"
        value={query}
        onChange={(e) => changeFind(editor, { query: e.target.value })}
        onKeyDown={(e) => {
          if (e.key !== "Enter" || e.nativeEvent.isComposing) return;
          e.preventDefault();
          changeFind(editor, { step: e.shiftKey ? -1 : 1 });
        }}
        className="h-7 w-44 min-w-0 rounded-md border border-line bg-surface px-2 text-xs outline-none focus:border-accent"
      />
      <span role="status" className="shrink-0 px-1 text-xs tabular-nums text-faint">
        {query && (count ? `${index + 1} of ${count}` : "No matches")}
      </span>
      <button type="button" className={button} aria-label="Previous match" title="Previous match" disabled={!count}
        onClick={() => changeFind(editor, { step: -1 })}><ChevronUp size={15} /></button>
      <button type="button" className={button} aria-label="Next match" title="Next match" disabled={!count}
        onClick={() => changeFind(editor, { step: 1 })}><ChevronDown size={15} /></button>
      <button type="button" className={button} aria-label="Close find" title="Close find" onClick={onClose}><X size={15} /></button>
    </div>
  );
}
