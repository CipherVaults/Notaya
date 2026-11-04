// Dev-only stand-in for @tauri-apps/api/core inside the database-views harness (see database-views-harness.tsx).
type Args = Record<string, unknown>;

declare global {
  interface Window {
    __invoke?: (cmd: string, args: Args) => Promise<unknown>;
  }
}

export async function invoke<T = unknown>(cmd: string, args: Args = {}): Promise<T> {
  if (!window.__invoke) throw new Error("harness not ready");
  return (await window.__invoke(cmd, args)) as T;
}

/** A gradient picture whose colors come from the attachment id, so each file looks different. */
export function convertFileSrc(id: string): string {
  let h = 0;
  for (const c of id) h = (h * 31 + c.charCodeAt(0)) % 360;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="320" height="200"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="hsl(${h},70%,62%)"/><stop offset="1" stop-color="hsl(${(h + 60) % 360},70%,42%)"/></linearGradient></defs><rect width="320" height="200" fill="url(#g)"/><circle cx="240" cy="60" r="26" fill="rgba(255,255,255,.55)"/></svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}
