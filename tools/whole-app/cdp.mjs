// Loopback-only CDP client for the disposable WebView created by these runners.
// No browser process is spawned here. All RPCs and target discovery are bounded.
import assert from 'node:assert/strict';
export const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
export const isNavigationError = error => /context (was destroyed|not found)|Cannot find context|Inspected target navigated|Cannot find default execution context/i.test(error.message);
export async function connect(port, match = label => label === 'main') {
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
        try { socket.send(JSON.stringify({ id: key, method, params })); }
        catch (error) { pending.delete(key); clearTimeout(timer); reject(error); }
      });
      const evaluate = async expression => {
        const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
        if (r.exceptionDetails) throw Error(r.exceptionDetails.exception?.description || r.exceptionDetails.text);
        if (!r.result || r.result.subtype === 'error') throw Error('CDP evaluation returned no usable result');
        return r.result.value;
      };
      try {
        const label = await evaluate('window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label');
        if (!match(label)) { socket.close(); continue; }
        const client = { send, evaluate, label,
          close: () => new Promise((resolve, reject) => {
            if (socket.readyState === WebSocket.CLOSED) return resolve();
            const timer = setTimeout(() => reject(Error('CDP socket did not close within 3 seconds')), 3000);
            socket.addEventListener('close', () => { clearTimeout(timer); resolve(); }, { once: true });
            socket.close();
          }),
          invoke: (command, args = {}) => evaluate(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)},${JSON.stringify(args)})`) };
        client.ready = () => waitFor(client, `document.readyState === 'complete' && typeof window.__TAURI_INTERNALS__?.invoke === 'function'`);
        client.reload = async () => {
          const before = await evaluate('performance.timeOrigin');
          await send('Page.reload');
          await waitFor(client, `performance.timeOrigin !== ${before} && document.readyState === 'complete' && typeof window.__TAURI_INTERNALS__?.invoke === 'function'`);
        };
        await client.ready();
        return client;
      } catch { socket.close(); }
    }
    await sleep(250);
  }
  throw Error('No disposable main WebView ready within discovery timeout');
}
export async function waitFor(client, expression, timeout = 30000) {
  const start = performance.now();
  do {
    try { if (await client.evaluate(expression)) return performance.now() - start; }
    catch (error) { if (!isNavigationError(error)) throw error; }
    await sleep(50);
  } while (performance.now() - start < timeout);
  throw Error(`WebView condition not met: ${expression}`);
}
