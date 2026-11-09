// Loopback-only CDP client for the disposable WebView created by these runners.
// No browser process is spawned here. All RPCs and target discovery are bounded.
import assert from 'node:assert/strict';
export const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
export async function connect(port) {
  assert.match(String(port), /^\d+$/);
  const origin = `http://127.0.0.1:${port}`;
  for (let attempt = 0; attempt < 100; attempt++) {
    let targets;
    try { targets = await (await fetch(`${origin}/json`, { signal: AbortSignal.timeout(1000) })).json(); }
    catch { await sleep(250); continue; }
    for (const target of targets.filter(t => t.type === 'page')) {
      const url = new URL(target.webSocketDebuggerUrl);
      assert.equal(url.protocol, 'ws:');
      assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(url.hostname));
      assert.equal(url.port, String(port));
      const socket = new WebSocket(url);
      try {
        await new Promise((resolve, reject) => {
          const timer = setTimeout(() => reject(Error('WebSocket open timeout')), 3000);
          socket.onopen = () => { clearTimeout(timer); resolve(); };
          socket.onerror = () => { clearTimeout(timer); reject(Error('WebSocket open error')); };
        });
      } catch { socket.close(); continue; }
      let id = 0;
      const pending = new Map();
      socket.onmessage = event => {
        const data = JSON.parse(event.data);
        const waiter = pending.get(data.id);
        if (waiter) { pending.delete(data.id); clearTimeout(waiter.timer); data.error ? waiter.reject(Error(data.error.message)) : waiter.resolve(data.result); }
      };
      socket.onclose = () => { for (const waiter of pending.values()) { clearTimeout(waiter.timer); waiter.reject(Error('CDP connection closed')); } pending.clear(); };
      const send = (method, params = {}) => new Promise((resolve, reject) => {
        const key = ++id;
        const timer = setTimeout(() => { pending.delete(key); reject(Error(`CDP timeout: ${method}`)); }, 30000);
        pending.set(key, { resolve, reject, timer });
        socket.send(JSON.stringify({ id: key, method, params }));
      });
      const evaluate = async expression => {
        const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
        if (r.exceptionDetails) throw Error(r.exceptionDetails.exception?.description || r.exceptionDetails.text);
        return r.result?.value;
      };
      try {
        const label = await evaluate('window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label');
        if (label !== 'main') { socket.close(); continue; }
        return { send, evaluate, close: () => socket.close(),
          invoke: (command, args = {}) => evaluate(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)},${JSON.stringify(args)})`) };
      } catch { socket.close(); }
    }
    await sleep(250);
  }
  throw Error('No disposable main WebView ready within discovery timeout');
}
export async function waitFor(client, expression, timeout = 30000) {
  const start = performance.now();
  do { if (await client.evaluate(expression)) return performance.now() - start; await sleep(50); } while (performance.now() - start < timeout);
  throw Error(`WebView condition not met: ${expression}`);
}
