// One shared copy of each open database definition (properties, views, templates).
// The table, a row peek and the property list of a row page all edit this copy, so none of them can save an older
// version over another's changes. Edits are saved soon after (debounced); `now` saves at once.

import { create } from "zustand";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { registerFlush, trackSave } from "@/lib/flush";
import type { Database, Property } from "@/lib/database-types";

interface DbState {
  dbs: Record<string, Database>;
  /** Counts the saves that reached the backend, per database: what a view shows can change when one lands. */
  saves: Record<string, number>;
}
const useStore = create<DbState>(() => ({ dbs: {}, saves: {} }));

/** Changes whenever a save of this database has reached the backend (the query of a view is then run again). */
export function useSaveCount(id: string): number {
  return useStore((s) => s.saves[id] ?? 0);
}

const timers = new Map<string, ReturnType<typeof setTimeout>>();
const dirty = new Set<string>();
const chains = new Map<string, Promise<void>>();

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
  if (dirty.has(id)) await saveNow(id);
  const db = await api.getDatabase(id);
  put(db);
  return db;
}

/** Replaces the shared copy with a database the backend returned (for example after enabling sub-items). */
export function setDatabase(db: Database) {
  put(db);
}

export function dropDatabase(id: string) {
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
      const saved = await api.saveDatabase(sent);
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
    if (chains.get(id) === p) chains.delete(id);
  });
  return trackSave(p);
}

export function hasUnsaved(id: string): boolean {
  return dirty.has(id);
}

registerFlush(async () => {
  await Promise.allSettled([...dirty].map((id) => saveNow(id)));
});
