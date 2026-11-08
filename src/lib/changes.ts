import { listen } from "@tauri-apps/api/event";

export interface ItemsChanged {
  ids: string[];
  kinds: string[];
  tree: boolean;
  by: string;
  revs: Record<string, number>;
}
export type Revisioned<T> = T & { rev: number };
export interface BinEntry {
  id: string;
  kind: "notebook" | "section" | "page" | "database" | "row" | "sticky";
  title: string;
  icon: string;
  trashedAt: number;
  parentInBin: boolean;
}

const subscribers = new Set<(change: ItemsChanged) => void>();
let listening: Promise<() => void> | undefined;

/** One Tauri listener per window, shared by the stores and open editors. */
export function onItemsChanged(fn: (change: ItemsChanged) => void): () => void {
  subscribers.add(fn);
  listening ??= listen<ItemsChanged>("items-changed", ({ payload }) => {
    for (const subscriber of subscribers) subscriber(payload);
  });
  return () => { subscribers.delete(fn); };
}

/** Also used by the mock checks to exercise dirty/clean reload decisions. */
export function canReload(id: string, rev: number, dirty: boolean, change: ItemsChanged): boolean {
  return !dirty && change.ids.includes(id) && (change.revs[id] ?? 0) > rev;
}

/** Keep signals received during typing/saving until the caller becomes clean.
 * Reading the revision again after a save also suppresses that save's own signal.
 */
export function watchItemChanges(id: string, revision: () => number, busy: () => boolean,
  reload: (alive: () => boolean) => Promise<void>) {
  let wanted = 0;
  let loading = false;
  let alive = true;
  const recheck = () => {
    if (!alive || loading || busy() || wanted <= revision()) return;
    loading = true;
    const requested = wanted;
    void reload(() => alive).catch(() => {}).finally(() => {
      loading = false;
      if (wanted > requested) recheck();
    });
  };
  const off = onItemsChanged((change) => {
    if (!change.ids.includes(id)) return;
    wanted = Math.max(wanted, change.revs[id] ?? 0);
    recheck();
  });
  return { recheck, dispose: () => { alive = false; off(); } };
}
