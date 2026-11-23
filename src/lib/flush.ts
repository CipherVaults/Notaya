// Pending saves register here so they can be written before the vault locks or the app closes.

type FlushFn = () => Promise<void> | void;

/** The longest the UI waits for pending saves, so a hung write can never block locking. */
const FLUSH_TIMEOUT_MS = 2000;

const fns = new Set<FlushFn>();
const tracked = new Set<Promise<unknown>>();

/** Registers a function that writes pending edits. Returns the function that unregisters it. */
export function registerFlush(fn: FlushFn): () => void {
  fns.add(fn);
  return () => {
    fns.delete(fn);
  };
}

/**
 * Marks a save as outstanding until it settles, so flushAll also waits for saves whose editor
 * is already gone (for example a page that was just switched away from). Returns the same promise.
 */
export function trackSave<T>(p: Promise<T>): Promise<T> {
  tracked.add(p);
  const done = () => {
    tracked.delete(p);
  };
  p.then(done, done);
  return p;
}

/**
 * Runs every registered flush and waits for them and for every tracked save, even if some fail.
 * Always settles within about two seconds.
 */
export async function flushAll(): Promise<void> {
  if (fns.size === 0 && tracked.size === 0) return;
  const work = (async () => {
    await Promise.allSettled([...fns].map((fn) => Promise.resolve().then(fn)));
    await Promise.allSettled([...tracked]);
  })();
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<void>((resolve) => {
    timer = setTimeout(resolve, FLUSH_TIMEOUT_MS);
  });
  try {
    await Promise.race([work, timeout]);
  } finally {
    clearTimeout(timer);
  }
}
