// Read only the labelled main window's native visibility. Unknown is an error, never hidden.
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
export function strictVisibility(value) {
  if (typeof value !== 'boolean') throw new Error('Main window visibility is not boolean');
  return value;
}
export async function readMainWindowVisible(port, { fetchImpl = globalThis.fetch, Socket = globalThis.WebSocket } = {}) {
  const number = Number(port);
  if (!/^[0-9]+$/.test(String(port)) || !Number.isInteger(number) || number < 1 || number > 65535) throw new Error('An explicit valid loopback port is required');
  const sockets = new Set(), abort = new AbortController();
  let stopped = false;
  let budgetTimer;
  const budget = new Promise((_, reject) => {
    var timer = setTimeout(() => { stopped = true; abort.abort(); for (const c of sockets) c.close(); reject(new Error('Visibility deadline')); }, 6000);
    budgetTimer = timer;
  });
  function connection(address) {
    const url = new URL(address);
    if (url.protocol !== 'ws:' || !['127.0.0.1', 'localhost'].includes(url.hostname) || Number(url.port) !== number || url.username || url.password || url.search || url.hash || !url.pathname.startsWith('/devtools/page/')) throw new Error('Non-loopback CDP endpoint refused');
    const ws = new Socket(url.href), pending = new Map(); let id = 0;
    const rejectPending = () => { for (const p of pending.values()) { clearTimeout(p.timer); p.reject(new Error('CDP unavailable')); } pending.clear(); };
    const close = () => { rejectPending(); try { ws.close(); } catch {} };
    sockets.add({ close });
    const opened = new Promise((resolveOpen, reject) => {
      const timer = setTimeout(() => { close(); reject(new Error('CDP connect deadline')); }, 1000);
      ws.onopen = () => { clearTimeout(timer); resolveOpen(); };
      ws.onerror = () => { clearTimeout(timer); rejectPending(); reject(new Error('CDP unavailable')); };
    });
    ws.onclose = rejectPending;
    ws.onmessage = ({ data }) => {
      let message; try { message = JSON.parse(data); } catch { rejectPending(); return; }
      const p = pending.get(message.id); if (!p) return;
      clearTimeout(p.timer); pending.delete(message.id);
      if (message.error) p.reject(new Error('CDP error')); else p.resolve(message.result);
    };
    return { close, async evaluate(expression) {
      await opened; if (stopped) throw new Error('Visibility deadline');
      const result = await new Promise((resolveReply, reject) => {
        const key = ++id, timer = setTimeout(() => { pending.delete(key); reject(new Error('CDP reply deadline')); }, 1500);
        pending.set(key, { timer, reject, resolve: resolveReply });
        ws.send(JSON.stringify({ id: key, method: 'Runtime.evaluate', params: { expression, awaitPromise: true, returnByValue: true } }));
      });
      if (result?.exceptionDetails) throw new Error('Visibility unavailable');
      return result?.result?.value;
    } };
  }
  async function probe() {
    const response = await fetchImpl(`http://127.0.0.1:${number}/json`, { signal: abort.signal, redirect: 'error' });
    if (!response.ok || !response.body) throw new Error('CDP unavailable');
    const reader = response.body.getReader(), decoder = new TextDecoder(); let bytes = 0, text = '';
    while (true) {
      const part = await reader.read(); if (part.done) break;
      bytes += part.value.byteLength; if (bytes > 65536) { await reader.cancel(); throw new Error('CDP response limit'); }
      text += decoder.decode(part.value, { stream: true });
    }
    const targets = JSON.parse(text + decoder.decode());
    if (!Array.isArray(targets) || targets.length > 16) throw new Error('CDP target limit');
    for (const target of targets.filter((t) => t?.type === 'page')) {
      if (stopped) throw new Error('Visibility deadline');
      let c, main = false;
      try {
        c = connection(target.webSocketDebuggerUrl);
        main = await c.evaluate("window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label ?? null") === 'main';
        if (main) return strictVisibility(await c.evaluate("window.__TAURI_INTERNALS__.invoke('plugin:window|is_visible', { label: 'main' })"));
      } catch { if (main) throw new Error('Main visibility unavailable'); }
      finally { c?.close(); }
    }
    throw new Error('Labelled main window unavailable');
  }
  try { return await Promise.race([probe(), budget]); }
  finally { stopped = true; clearTimeout(budgetTimer); abort.abort(); for (const c of sockets) c.close(); }
}
if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  try {
    if (process.argv.length !== 3) throw new Error('One port is required');
    process.stdout.write(String(await readMainWindowVisible(process.argv[2])) + '\n');
  } catch { process.stderr.write('Main window visibility unavailable\n'); process.exitCode = 1; }
}