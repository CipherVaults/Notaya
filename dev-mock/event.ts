// Dev-only event bus. The production build resolves Tauri's event implementation.
type Handler = (event: { payload: unknown; event: string; id: number }) => void;
const handlers = new Map<string, Set<Handler>>();
export async function listen<T>(event: string, fn: (event: { payload: T; event: string; id: number }) => void): Promise<() => void> {
  const set = handlers.get(event) ?? new Set<Handler>();
  set.add(fn as Handler); handlers.set(event, set);
  return () => { set.delete(fn as Handler); };
}
export async function emit(event: string, payload: unknown): Promise<void> {
  for (const fn of handlers.get(event) ?? []) fn({ payload, event, id: 0 });
}
