// Titles of rows that relation cells point to. They live in another database, so the table does not have them;
// a small cache asks the backend once per id.

import { useEffect, useState } from "react";
import { api } from "@/lib/ipc";
import type { RowHit } from "@/lib/database-types";

const cache = new Map<string, RowHit>();
const pending = new Set<string>();
const listeners = new Set<() => void>();

function notify() {
  listeners.forEach((l) => l());
}

/** Remember a title we already know (after the person typed it, or a search hit). */
export function rememberRow(hit: RowHit) {
  cache.set(hit.id, hit);
  notify();
}
export function forgetRows() {
  cache.clear();
  notify();
}

async function fetchMissing(ids: string[]) {
  const need = ids.filter((id) => !cache.has(id) && !pending.has(id));
  if (!need.length) return;
  need.forEach((id) => pending.add(id));
  try {
    const hits = await api.rowTitles(need);
    for (const h of hits) cache.set(h.id, h);
    // Rows that no longer exist stay unknown; remember that so we do not ask again and again.
    for (const id of need) if (!cache.has(id)) cache.set(id, { id, title: "", icon: "" });
  } catch {
    /* a later render asks again */
  } finally {
    need.forEach((id) => pending.delete(id));
    notify();
  }
}

/** Title and icon for each id (empty title while loading or when the row is gone). */
export function useRowTitles(ids: string[]): Map<string, RowHit> {
  const [, setTick] = useState(0);
  useEffect(() => {
    const l = () => setTick((n) => n + 1);
    listeners.add(l);
    return () => {
      listeners.delete(l);
    };
  }, []);
  const key = ids.join(",");
  useEffect(() => {
    void fetchMissing(ids);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key]);
  const out = new Map<string, RowHit>();
  for (const id of ids) out.set(id, cache.get(id) ?? { id, title: "", icon: "" });
  return out;
}
