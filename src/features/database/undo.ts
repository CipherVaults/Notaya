// Undo and redo for what is done in an open database (cell edits, rows added or moved to the trash, row order).
// Each entry knows its own inverse, so an undo uses the same calls the edit used. Nothing is kept on disk: the stack of a
// database is dropped when its last view closes, and it holds at most 50 steps.

import { toast } from "sonner";
import { errorText } from "@/lib/ipc";

export interface UndoEntry {
  /** Short, for the message ("Undone: edit of Status"). */
  label: string;
  undo: () => Promise<void> | void;
  redo: () => Promise<void> | void;
}

const MAX = 50;
interface Stack {
  undo: UndoEntry[];
  redo: UndoEntry[];
  views: number;
  busy: boolean;
}
const stacks = new Map<string, Stack>();
const stackOf = (id: string): Stack => {
  let s = stacks.get(id);
  if (!s) stacks.set(id, (s = { undo: [], redo: [], views: 0, busy: false }));
  return s;
};

/** A view of this database is open. Returns the call that says it closed; with the last one the stack is dropped. */
export function retainUndo(id: string): () => void {
  stackOf(id).views++;
  return () => {
    const s = stacks.get(id);
    if (s && --s.views <= 0) stacks.delete(id);
  };
}

/** Remembers a step that was just done. A new step ends the redo line. */
export function pushUndo(id: string, entry: UndoEntry) {
  const s = stackOf(id);
  s.undo.push(entry);
  if (s.undo.length > MAX) s.undo.shift();
  s.redo.length = 0;
}

/** The step was undone some other way (the toast's Undo button): it leaves the stack, and is not offered again. */
export function dropUndo(id: string, entry: UndoEntry) {
  const s = stacks.get(id);
  if (!s) return;
  s.undo = s.undo.filter((e) => e !== entry);
}

export function undoDepth(id: string): { undo: number; redo: number } {
  const s = stacks.get(id);
  return { undo: s?.undo.length ?? 0, redo: s?.redo.length ?? 0 };
}

async function step(id: string, from: "undo" | "redo") {
  const s = stackOf(id);
  if (s.busy) return;
  const e = s[from].pop();
  if (!e) {
    toast(from === "undo" ? "Nothing to undo" : "Nothing to redo");
    return;
  }
  s.busy = true;
  try {
    await (from === "undo" ? e.undo() : e.redo());
    (from === "undo" ? s.redo : s.undo).push(e);
    toast(`${from === "undo" ? "Undone" : "Redone"}: ${e.label}`);
  } catch (err) {
    toast.error(`Couldn't ${from}: ${errorText(err)}`);
  } finally {
    s.busy = false;
  }
}

export const undoLast = (id: string) => step(id, "undo");
export const redoLast = (id: string) => step(id, "redo");

/**
 * Handles Ctrl+Z, Ctrl+Shift+Z and Ctrl+Y. Returns true when the key was taken. A text field, a select or the page editor
 * keeps its own undo (the browser's or TipTap's), and so does a dialog.
 */
export function handleUndoKey(id: string, e: { key: string; ctrlKey: boolean; metaKey: boolean; altKey: boolean; shiftKey: boolean; target: EventTarget | null; preventDefault: () => void }): boolean {
  if (!(e.ctrlKey || e.metaKey) || e.altKey) return false;
  const k = e.key.toLowerCase();
  const redo = (k === "z" && e.shiftKey) || (k === "y" && !e.shiftKey);
  const undo = k === "z" && !e.shiftKey;
  if (!undo && !redo) return false;
  const t = e.target instanceof Element ? e.target : null;
  if (t?.closest('input,textarea,select,[contenteditable]:not([contenteditable="false"]),.ProseMirror,[role="dialog"]')) return false;
  e.preventDefault();
  void (redo ? redoLast(id) : undoLast(id));
  return true;
}
