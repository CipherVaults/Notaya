// Development-only event bus with the same callback envelope as Tauri.
type Handler = (event: { event: string; id: number; payload: unknown }) => void;
const handlers = new Map<string, Map<number, Handler>>();
let nextId = 0;

export async function listen<T = unknown>(event: string, handler: (event: { event: string; id: number; payload: T }) => void): Promise<() => void> {
  const id = ++nextId;
  const entries = handlers.get(event) ?? new Map<number, Handler>();
  handlers.set(event, entries);
  entries.set(id, handler as Handler);
  return () => { entries.delete(id); if (!entries.size) handlers.delete(event); };
}

export async function emit<T>(event: string, payload?: T): Promise<void> {
  for (const [id, handler] of [...(handlers.get(event)?.entries() ?? [])]) handler({ event, id, payload });
}
