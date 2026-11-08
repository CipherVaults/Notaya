import { closeHistory } from "@tiptap/pm/history";
import type { EditorState } from "@tiptap/pm/state";

// Test setup must not join the equation edit's undo event, even in the same tick.
// Shared by the browser check and the browser-less node-view regression.
export function equationFixtureTransaction(state: EditorState, bid: string) {
  const content = [
    state.schema.nodes.mathBlock.create({ bid, latex: "x^2" }),
    state.schema.nodes.paragraph.create(),
  ];
  return closeHistory(
    state.tr.replaceWith(0, state.doc.content.size, content).setMeta("addToHistory", false),
  );
}
