// Pure transport fixtures: no application, listener, registry, filesystem writes or real HTTP.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';
const readerURL = process.env.NOTAYA_VISIBILITY_READER_TEST_FILE
  ? pathToFileURL(resolve(process.env.NOTAYA_VISIBILITY_READER_TEST_FILE))
  : new URL('./main-window-visible.mjs', import.meta.url);
const { readMainWindowVisible, strictVisibility } = await import(readerURL.href);
function fixture(options = {}) {
  const { label = 'main', rpcError = false, exception = false, targets, httpError = false, noServer = false } = options;
  const value = Object.hasOwn(options, 'value') ? options.value : false;
  const requests = []; let sockets = 0, fetches = 0;
  class Socket {
    constructor() { sockets++; queueMicrotask(() => this.onopen?.()); }
    send(raw) {
      const request = JSON.parse(raw); requests.push(request);
      assert.equal(request.method, 'Runtime.evaluate');
      const metadata = request.params.expression.includes('metadata?.currentWindow?.label');
      if (!metadata) assert.equal(request.params.expression, "window.__TAURI_INTERNALS__.invoke('plugin:window|is_visible', { label: 'main' })");
      const result = !metadata && exception ? { exceptionDetails: {} } : { result: { value: metadata ? label : value } };
      const reply = !metadata && rpcError ? { id: request.id, error: { code: -1 } } : { id: request.id, result };
      queueMicrotask(() => this.onmessage?.({ data: JSON.stringify(reply) }));
    }
    close() { this.onclose?.(); }
  }
  const fetchImpl = async (url, options) => {
    fetches++; assert.equal(url, 'http://127.0.0.1:9337/json'); assert.equal(options.redirect, 'error');
    if (noServer) throw new Error('synthetic empty port');
    return new Response(JSON.stringify(targets ?? [{ type: 'page', webSocketDebuggerUrl: 'ws://127.0.0.1:9337/devtools/page/synthetic' }]), { status: httpError ? 500 : 200 });
  };
  return { io: { fetchImpl, Socket }, requests, sockets: () => sockets, fetches: () => fetches };
}
for (const value of [true, false]) {
  test(`labelled main visibility ${value} is preserved`, async () => {
    const f = fixture({ value });
    assert.equal(await readMainWindowVisible('9337', f.io), value);
    assert.equal(f.requests.length, 2); assert.equal(f.sockets(), 1);
  });
}
for (const [name, options] of [
  ['bad label', { label: 'sticky' }], ['unavailable value', { value: undefined }],
  ['RPC error', { rpcError: true }], ['getter exception', { exception: true }],
  ['null', { value: null }], ['string false', { value: 'false' }],
  ['numeric zero', { value: 0 }], ['numeric one', { value: 1 }],
  ['empty selection', { targets: [] }], ['HTTP error', { httpError: true }],
  ['valid but empty port', { noServer: true }],
]) {
  test(`${name} fails instead of becoming hidden`, async () => {
    const f = fixture(options);
    await assert.rejects(readMainWindowVisible('9337', f.io));
  });
}
for (const value of [undefined, null, NaN, Infinity, -Infinity, 0, 1, '', 'false', {}, []]) {
  test(`strict observation rejects ${String(value)}`, () => assert.throws(() => strictVisibility(value)));
}
for (const port of [undefined, '', 'NaN', 'Infinity', '0', '65536', '9337junk']) {
  test(`invalid port ${String(port)} is rejected before access`, async () => {
    const f = fixture(); await assert.rejects(readMainWindowVisible(port, f.io)); assert.equal(f.fetches(), 0); assert.equal(f.sockets(), 0);
  });
}
test('off-port or foreign websocket targets are refused before connection', async () => {
  for (const address of ['ws://127.0.0.1:9338/devtools/page/x', 'ws://example.com:9337/devtools/page/x', 'ws://127.0.0.1:9337/devtools/page/x?secret=omitted']) {
    const f = fixture({ targets: [{ type: 'page', webSocketDebuggerUrl: address }] });
    await assert.rejects(readMainWindowVisible('9337', f.io)); assert.equal(f.sockets(), 0);
  }
});