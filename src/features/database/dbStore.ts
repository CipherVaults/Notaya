// One shared copy of each open database definition (properties, views, templates).
// The table, a row peek and the property list of a row page all edit this copy, so none of them can save an older
// version over another's changes. Edits are saved soon after (debounced); `now` saves at once.

import { create } from "zustand";
import { toast } from "sonner";
import { onItemsChanged, watchItemChanges } from "@/lib/changes";
import { api, errorText } from "@/lib/ipc";
import { registerFlush, trackSave } from "@/lib/flush";
import type { Database, Property } from "@/lib/database-types";

interface DbState {
  dbs: Record<string, Database>;
  /** Counts the saves that reached the backend, per database: what a view shows can change when one lands. */
  saves: Record<string, number>;
}
const useStore = create<DbState>(() => ({ dbs: {}, saves: {} }));

/** Something other than an edit in a view changed rows (a button or an automation ran): views run their query again. */
export function rowsChanged() {
  useStore.setState((s) => ({ saves: Object.fromEntries(Object.keys(s.dbs).map((id) => [id, (s.saves[id] ?? 0) + 1])) }));
  window.dispatchEvent(new CustomEvent("notaya:rows-changed"));
}

/** Changes whenever a save of this database has reached the backend (the query of a view is then run again). */
export function useSaveCount(id: string): number {
  return useStore((s) => s.saves[id] ?? 0);
}

const timers = new Map<string, ReturnType<typeof setTimeout>>();
const dirty = new Set<string>();
const chains = new Map<string, Promise<void>>();
const revs = new Map<string, number>();
const watchers = new Map<string, ReturnType<typeof watchItemChanges>>();
const editingFields = new Set<string>();
let rowsPending = false;

function watchDatabase(id: string) {
  if (watchers.has(id)) return;
  watchers.set(id, watchItemChanges(id, () => revs.get(id) ?? 0,
    () => dirty.has(id) || chains.has(id) || editingFields.has(id), async (alive) => {
      const db = await api.getDatabase(id);
      if (alive() && !hasUnsaved(id) && !editingFields.has(id) && db.rev >= (revs.get(id) ?? 0)) {
        put(db); revs.set(id, db.rev);
      }
    }));
}

function refreshRowsWhenClean() {
  if (rowsPending && !editingFields.size) { rowsPending = false; rowsChanged(); }
}

const get = (id: string) => useStore.getState().dbs[id];
const put = (db: Database) => useStore.setState((s) => ({ dbs: { ...s.dbs, [db.id]: db } }));

/** The shared copy of a database, or undefined until it is loaded. */
export function useDatabase(id: string): Database | undefined {
  return useStore((s) => s.dbs[id]);
}
export function peekDatabase(id: string): Database | undefined {
  return get(id);
}

/** Loads (or reloads) a database from the backend into the shared copy. Unsaved edits are saved first. */
export async function loadDatabase(id: string): Promise<Database> {
  watchDatabase(id);
  if (dirty.has(id)) await saveNow(id);
  await chains.get(id);
  const db = await api.getDatabase(id);
  if (!dirty.has(id) && !chains.has(id)) { put(db); revs.set(id, db.rev); }
  return db;
}

/** Replaces the shared copy with a database the backend returned (for example after enabling sub-items). */
export function setDatabase(db: Database) {
  watchDatabase(db.id);
  put(db);
}

export function dropDatabase(id: string) {
  watchers.get(id)?.dispose(); watchers.delete(id);
  revs.delete(id);
  useStore.setState((s) => {
    const dbs = { ...s.dbs };
    delete dbs[id];
    return { dbs };
  });
}

/** Changes a database. Properties and views are saved soon; a locked database only changes on screen. */
export function editDatabase(id: string, fn: (d: Database) => Database, now = false): Promise<void> {
  const cur = get(id);
  if (!cur) return Promise.resolve();
  const next = fn(cur);
  if (next === cur) return Promise.resolve();
  put(next);
  if (next.locked) return Promise.resolve();
  dirty.add(id);
  void api.editing(id, true).catch(() => {});
  const t = timers.get(id);
  if (t) clearTimeout(t);
  if (now) {
    timers.delete(id);
    return saveNow(id);
  }
  timers.set(
    id,
    setTimeout(() => {
      timers.delete(id);
      void saveNow(id);
    }, 700),
  );
  return Promise.resolve();
}

function mergeSaved(current: Database, sent: Database, saved: Database): Database {
  if (current === sent) return saved;
  // Edits were made while the save ran. Keep them, and take over what only the backend could decide:
  // the other side of new relations, and the counters.
  const props: Property[] = current.props.map((p) => {
    if (p.kind === "relation" && p.relationBack === "+") {
      const s = saved.props.find((x) => x.id === p.id);
      if (s) return { ...p, relationBack: s.relationBack };
    }
    return p;
  });
  const added = saved.props.filter((p) => !sent.props.some((x) => x.id === p.id) && !current.props.some((x) => x.id === p.id));
  const views = current.views.map((v) => ({ ...v, columns: [...v.columns, ...added.filter((p) => !v.columns.some((c) => c.prop === p.id)).map((p) => ({ prop: p.id, visible: true, width: 0 }))] }));
  return { ...current, props: [...props, ...added], views, nextId: saved.nextId, updated: saved.updated };
}

/** Writes the pending edits of a database now. Saves run one after another. */
export function saveNow(id: string): Promise<void> {
  const t = timers.get(id);
  if (t) {
    clearTimeout(t);
    timers.delete(id);
  }
  const run = async () => {
    if (!dirty.has(id)) return;
    dirty.delete(id);
    const sent = get(id);
    if (!sent || sent.locked) return;
    try {
      const saved = await api.saveDatabase(sent, revs.get(id));
      revs.set(id, saved.rev);
      if (!dirty.has(id) && !editingFields.has(id)) void api.editing(id, false).catch(() => {});
      put(mergeSaved(get(id) ?? sent, sent, saved));
      useStore.setState((s) => ({ saves: { ...s.saves, [id]: (s.saves[id] ?? 0) + 1 } }));
      // New relations create a property in the other database too: refresh it if it is open.
      for (const p of sent.props) {
        if (p.kind === "relation" && p.relationBack === "+" && p.relationDb !== id && get(p.relationDb)) void api.getDatabase(p.relationDb).then(put).catch(() => undefined);
      }
    } catch (e) {
      dirty.add(id); // keep the edit so the next save retries it
      toast.error(`Couldn't save the database: ${errorText(e)}`);
    }
  };
  const p = (chains.get(id) ?? Promise.resolve()).then(run);
  chains.set(id, p);
  p.then(() => {
    if (chains.get(id) === p) {
      chains.delete(id);
      watchers.get(id)?.recheck();
    }
  });
  return trackSave(p);
}

export function hasUnsaved(id: string): boolean {
  return dirty.has(id) || chains.has(id);
}

registerFlush(async () => {
  await Promise.allSettled([...dirty].map((id) => saveNow(id)));
});


onItemsChanged((change) => {
  if (change.kinds.some((kind) => kind === "row" || kind === "database")) {
    rowsPending = true; refreshRowsWhenClean();
  }
});


// Database cells keep local input until blur/Enter. Their existing root attribute
// lets the store protect that interval without changing the table or its undo flow.
if (typeof document !== "undefined") {
  const databaseOf = (target: EventTarget | null) => target instanceof HTMLElement
    && target.matches("input, textarea, [contenteditable=true]")
    ? target.closest<HTMLElement>("[data-database]")?.dataset.database : undefined;
  document.addEventListener("focusin", (event) => {
    const id = databaseOf(event.target);
    if (id) { editingFields.add(id); void api.editing(id, true).catch(() => {}); }
  });
  document.addEventListener("focusout", (event) => {
    const id = databaseOf(event.target);
    if (id) queueMicrotask(() => {
      if (databaseOf(document.activeElement) !== id) {
        editingFields.delete(id);
        if (!hasUnsaved(id)) void api.editing(id, false).catch(() => {});
        watchers.get(id)?.recheck();
        refreshRowsWhenClean();
      }
    });
  });
}
