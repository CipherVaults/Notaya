// Exercise the actual injected probe, with the read-only descriptors Tauri uses.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import vm from "node:vm";
import ts from "typescript";

const source = readFileSync(new URL("./startup-minimized.mjs", import.meta.url), "utf8");
const tree = ts.createSourceFile("probe.js", source, ts.ScriptTarget.Latest, true);
const seam = tree.statements.find((s) => ts.isFunctionDeclaration(s) && s.name?.text === "frameProbe");
assert.ok(seam);

function harness({ reject = false, fallback = false, lockedFetch = false } = {}) {
  const callbacks = new Map(); let now = 10, next = 0, forwarded = 0;
  const internals = { callbacks };
  const deliver = (message) => {
    forwarded++;
    callbacks.get(reject ? message.error : message.callback)?.();
  };
  const window = {
    fetch(input, init) {
      const headers = new Headers(init.headers);
      const message = { cmd: input.endsWith("window_ready") ? "window_ready" : "vault_status",
        callback: Number(headers.get("Tauri-Callback")), error: Number(headers.get("Tauri-Error")) };
      if (fallback) window.ipc.postMessage(JSON.stringify(message)); else deliver(message);
      return Promise.resolve();
    },
    ipc: { postMessage(data) { deliver(JSON.parse(data)); } },
    requestAnimationFrame: () => ++next, cancelAnimationFrame() {},
    __TAURI_INTERNALS__: internals,
  };
  const invoke = (cmd) => new Promise((resolve, rejectCall) => {
    const ok = ++next, error = ++next;
    callbacks.set(ok, resolve); callbacks.set(error, rejectCall);
    void window.fetch(`http://ipc.localhost/${cmd}`, { method: "POST", headers: new Headers({ "Tauri-Callback": String(ok), "Tauri-Error": String(error) }) });
  });
  Object.defineProperty(internals, "invoke", { value: invoke });
  if (lockedFetch) Object.defineProperty(window, "fetch", { writable: false, configurable: false });
  const timers = [];
  vm.runInNewContext(seam.getText(tree) + '; frameProbe(8000, "reload-1");', {
    window, Headers, performance: { now: () => now }, setTimeout: (fn) => timers.push(fn),
  });
  return { window, probe: window.__notayaStartupProbe, invoke, forwarded: () => forwarded,
    advance: (ms) => { now += ms; }, timers };
}

test("read-only Tauri invoke: real IPC is observed and successful callback is counted", async () => {
  const h = harness();
  assert.equal(h.window.__TAURI_INTERNALS__.invoke, h.invoke);
  assert.equal(Object.getOwnPropertyDescriptor(h.window.__TAURI_INTERNALS__, "invoke").writable, false);
  await h.invoke("vault_status");
  assert.equal(h.probe.readyCalls, 0);
  h.advance(30); await h.invoke("window_ready");
  assert.equal(h.probe.hookInstalled, true);
  assert.equal(h.probe.readyCalls, 1); assert.equal(h.probe.readyCompleted, 1);
  assert.equal(h.probe.readyAtMs, 30); assert.equal(h.probe.documentId, "reload-1");
  assert.equal(h.forwarded(), 2);
});

test("IPC fallback is forwarded and counted once, rather than once per transport", async () => {
  const h = harness({ fallback: true });
  await h.invoke("window_ready");
  assert.equal(h.probe.nativeHookInstalled, true);
  assert.equal(h.probe.readyCalls, 1); assert.equal(h.probe.readyCompleted, 1);
  assert.equal(h.forwarded(), 1);
});

test("a read-only transport is reported as unhooked, never a false installed pass", async () => {
  const h = harness({ lockedFetch: true });
  await h.invoke("window_ready");
  assert.equal(h.probe.fetchHookInstalled, false); assert.equal(h.probe.hookInstalled, false);
  assert.equal(h.probe.readyCalls, 0); assert.equal(h.forwarded(), 1);
});

test("rejected ready call is distinguished from a successful hidden-tray early return", async () => {
  const h = harness({ reject: true });
  await assert.rejects(h.invoke("window_ready"));
  assert.equal(h.probe.readyCalls, 1); assert.equal(h.probe.readyCompleted, 0);
  assert.equal(h.probe.readyRejected, 1); assert.equal(h.probe.readyAtMs, null);
});

test("held frames stay held until explicit release even if the page timer never fires", () => {
  const h = harness();
  h.window.requestAnimationFrame(() => {});
  assert.equal(h.probe.heldFrames, 1); assert.equal(h.probe.framesReleased, false);
  h.advance(8000); h.probe.releaseFrames(); h.probe.releaseFrames();
  assert.equal(h.probe.framesReleasedAtMs, 8000); assert.equal(h.probe.framesReleased, true);
});
