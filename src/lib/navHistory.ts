// Back and forward through the places the person has been in the Notes view (page, database, section,
// notebook, trash, and the Notes / Sticky notes / Settings views). Backspace, Alt+Left, the mouse's back
// button and the title bar buttons all use it. Kept in memory only: nothing about where you have been is
// written anywhere, and it is gone when the window reloads (for example after a lock).
// Only the last open note location is saved separately, inside the vault's settings record.

import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { registerFlush, trackSave } from "@/lib/flush";
import { findPageMeta, useApp, type LastPlace } from "@/store/app";

type Loc = {
  view: ReturnType<typeof useApp.getState>["view"];
  activeNotebookId: string | null;
  activeSectionId: string | null;
  activePageId: string | null;
  activeKind: ReturnType<typeof useApp.getState>["activeKind"];
  activeDatabaseViewId: string | null;
  showTrash: boolean;
};

const MAX_STEPS = 100;

const pick = (s: ReturnType<typeof useApp.getState>): Loc => ({
  view: s.view,
  activeNotebookId: s.activeNotebookId,
  activeSectionId: s.activeSectionId,
  activePageId: s.activePageId,
  activeKind: s.activeKind,
  activeDatabaseViewId: s.activeDatabaseViewId,
  showTrash: s.showTrash,
});
const same = (a: Loc, b: Loc) =>
  a.view === b.view &&
  a.activeNotebookId === b.activeNotebookId &&
  a.activeSectionId === b.activeSectionId &&
  a.activePageId === b.activePageId &&
  a.activeKind === b.activeKind &&
  a.activeDatabaseViewId === b.activeDatabaseViewId &&
  a.showTrash === b.showTrash;

/** What the title bar buttons show. */
export const useNav = create<{ canBack: boolean; canForward: boolean }>(() => ({ canBack: false, canForward: false }));

let back: Loc[] = [];
let forward: Loc[] = [];
let current: Loc | null = null;
let applying = false;

const publish = () => useNav.setState({ canBack: back.length > 0, canForward: forward.length > 0 });

/** A place is still worth going to when the notebook, section and page (or database) it names still exist. */
function stillThere(l: Loc): boolean {
  const t = useApp.getState().tree;
  if (l.activeNotebookId && !t.notebooks.some((n) => n.id === l.activeNotebookId)) return false;
  if (l.activeSectionId && !t.sections.some((n) => n.id === l.activeSectionId)) return false;
  if (l.activePageId && l.activeKind === "page" && findPageMeta(t, l.activePageId)?.trashedAt !== null) return false;
  if (l.activePageId && l.activeKind === "database" && !t.databases.some((d) => d.id === l.activePageId && d.trashedAt === null)) return false;
  return true;
}

function go(from: Loc[], to: Loc[]) {
  while (from.length > 0) {
    const next = from.pop() as Loc;
    if (!stillThere(next) || (current && same(next, current))) continue;
    if (current) to.push(current);
    if (to.length > MAX_STEPS) to.shift();
    current = next;
    applying = true;
    try {
      useApp.setState(next);
    } finally {
      applying = false;
    }
    publish();
    return;
  }
  publish();
}

export const goBack = () => go(back, forward);
export const goForward = () => go(forward, back);

/** Starts recording. Returns the function that stops it. */
export function startNavHistory(): () => void {
  back = [];
  forward = [];
  current = pick(useApp.getState());
  publish();
  return useApp.subscribe((s) => {
    if (applying) return;
    const next = pick(s);
    if (current && same(next, current)) return;
    // The first notebook chosen when the vault loads is not a step the person took.
    if (current && current.activeNotebookId !== null) {
      back.push(current);
      if (back.length > MAX_STEPS) back.shift();
      forward = [];
    }
    current = next;
    publish();
  });
}

/** Debounced, serialized saves, included in the existing pre-lock / pre-quit flush. */
export function startLastPlacePersistence(): () => void {
  let initialized = false;
  let latest: LastPlace | null = null;
  let key = "";
  let dirty = false;
  let warned = false;
  let stopped = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let chain = Promise.resolve();
  const flush = () => {
    clearTimeout(timer);
    chain = trackSave(chain.then(async () => {
      if (!dirty || stopped || !useApp.getState().status?.unlocked) return;
      const sent = latest;
      dirty = false;
      try {
        await invoke<void>("set_last_place", { lastPlace: sent });
        warned = false;
      } catch {
        dirty = true; // A later navigation or the lock flush retries the current location.
        if (!warned) toast.error("Couldn't remember the last open place.");
        warned = true;
      }
    }));
    return chain;
  };
  const observe = (s: ReturnType<typeof useApp.getState>) => {
    // Loading the vault restores the location and settings together; don't save Home over it.
    if (!s.settings || !s.status?.unlocked) return;
    if (!initialized) {
      initialized = true;
      latest = s.settings.lastPlace ?? null;
      key = JSON.stringify(latest);
    }
    // Settings and Sticky notes do not replace the last open note.
    if (s.view !== "notes") return;
    const next: LastPlace | null = s.activePageId && !s.showTrash ? {
      kind: s.activeKind,
      id: s.activePageId,
      ...(s.activeKind === "database" && s.activeDatabaseViewId ? { viewId: s.activeDatabaseViewId } : {}),
    } : null;
    const nextKey = JSON.stringify(next);
    if (key === nextKey) return;
    key = nextKey;
    latest = next;
    dirty = true;
    clearTimeout(timer);
    timer = setTimeout(() => void flush(), 400);
  };
  const unsubscribe = useApp.subscribe(observe);
  const unregister = registerFlush(flush);
  observe(useApp.getState());
  return () => {
    stopped = true;
    clearTimeout(timer);
    unsubscribe();
    unregister();
  };
}

/** True when the key or click happened where Backspace and the mouse buttons must keep their own meaning. */
function busy(target: EventTarget | null): boolean {
  const el = target instanceof HTMLElement ? target : null;
  if (!el) return false;
  if (el.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName)) return true;
  if (el.closest("[role=dialog], [role=menu], [role=listbox], .db-scroll, [role=grid]")) return true;
  return false;
}

/** Backspace, Shift+Backspace, Alt+Left, Alt+Right and the mouse's back and forward buttons. */
export function startNavKeys(): () => void {
  const onKey = (e: KeyboardEvent) => {
    if (e.defaultPrevented || e.ctrlKey || e.metaKey) return;
    const isBack = (e.key === "Backspace" && !e.shiftKey && !e.altKey) || (e.key === "ArrowLeft" && e.altKey && !e.shiftKey);
    const isForward = (e.key === "Backspace" && e.shiftKey && !e.altKey) || (e.key === "ArrowRight" && e.altKey && !e.shiftKey);
    if (!isBack && !isForward) return;
    if (busy(e.target)) return;
    // With a text box open or a menu on screen, the key belongs to it.
    if (document.querySelector("[role=dialog]")) return;
    e.preventDefault();
    if (isBack) goBack();
    else goForward();
  };
  // Buttons 3 and 4 are the mouse's back and forward. The web view would otherwise try its own history.
  const onMouse = (e: MouseEvent) => {
    if (e.button !== 3 && e.button !== 4) return;
    e.preventDefault();
    if (e.type !== "mouseup") return;
    if (document.querySelector("[role=dialog]")) return;
    if (e.button === 3) goBack();
    else goForward();
  };
  window.addEventListener("keydown", onKey);
  window.addEventListener("mousedown", onMouse);
  window.addEventListener("mouseup", onMouse);
  window.addEventListener("auxclick", onMouse);
  return () => {
    window.removeEventListener("keydown", onKey);
    window.removeEventListener("mousedown", onMouse);
    window.removeEventListener("mouseup", onMouse);
    window.removeEventListener("auxclick", onMouse);
  };
}
