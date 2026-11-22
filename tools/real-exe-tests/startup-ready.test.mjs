// No browser or exe: exercise the production scheduler with frames/timers stopped.
// Uses the project's existing TypeScript compiler; installs nothing.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import vm from "node:vm";
import ts from "typescript";

const source = readFileSync(new URL("../../src/lib/startup.ts", import.meta.url), "utf8");
const js = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS } }).outputText;
const document = { hidden: false, addEventListener() {}, removeEventListener() {} };

function harness() {
  const tasks = [];
  const timers = new Map();
  let now = 0, timerId = 0;
  const exports = {};
  const window = {
    setTimeout: () => assert.fail("readiness must not wait for a timer"),
    requestAnimationFrame: () => assert.fail("readiness must not wait for a paint"),
  };
  vm.runInNewContext(js, { exports, window, queueMicrotask: (fn) => tasks.push(fn), performance: { now: () => 100 },
    setTimeout: (fn, ms) => { timers.set(++timerId, { fn, at: now + ms }); return timerId; },
    clearTimeout: (id) => timers.delete(id),
  });
  return { schedule: exports.scheduleWindowReady, mark: exports.startupMark, window, tasks,
    watch: exports.watchStartupInitialization, bound: exports.STARTUP_READY_TIMEOUT_MS, timers,
    tick: () => { const pending = tasks.splice(0); pending.forEach((fn) => fn()); },
    advance: (ms) => {
      now += ms;
      for (const [id, timer] of timers) {
        if (timer.at <= now) { timers.delete(id); timer.fn(); }
      }
    },
  };
}

test("hidden WebView: readiness completes without a single animation frame", () => {
  const h = harness(); let ready = 0;
  h.schedule(() => ready++);
  assert.equal(ready, 0);
  h.tick();
  assert.equal(ready, 1);
});

test("ordinary startup signals after commit without waiting for either frame or fallback", () => {
  const h = harness(); let ready = 0;
  h.schedule(() => ready++);
  assert.equal(ready, 0);
  h.tick(); assert.equal(ready, 1);
  h.tick(); assert.equal(ready, 1);
});

test("readiness is once-only, including a late duplicate callback", () => {
  const h = harness(); let ready = 0;
  h.schedule(() => ready++);
  const lateCallback = h.tasks[0];
  h.tick(); lateCallback();
  assert.equal(ready, 1);
});

test("effect cleanup cancels its callback; StrictMode remount reschedules readiness", () => {
  const h = harness(); let ready = 0;
  const cancel = h.schedule(() => ready++);
  const lateCallback = h.tasks[0];
  cancel(); cancel(); lateCallback(); h.tick();
  assert.equal(ready, 0);
  h.schedule(() => ready++); h.tick();
  assert.equal(ready, 1);
});

test("startup diagnostics are absent by default and contain only counters/times when opted in", () => {
  const h = harness();
  h.mark("statusSetAtMs");
  assert.equal(h.window.__notayaStartupProbe, undefined);
  h.window.__notayaStartupProbe = { startedAtMs: 40 };
  h.mark("statusSetAtMs"); h.mark("effectRan", true); h.mark("effectRan", true);
  assert.deepEqual(h.window.__notayaStartupProbe, { startedAtMs: 40, statusSetAtMs: 60, effectRan: 2 });
});

// Run the actual hook bodies without React/DOM, so tests reach the startup gate
// rather than testing only an isolated scheduler (the first run's blind spot).
const appSource = readFileSync(new URL("../../src/App.tsx", import.meta.url), "utf8");
const appTree = ts.createSourceFile("App.tsx", appSource, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
function hookBody(hook, contains) {
  let match;
  function visit(node) {
    if (ts.isCallExpression(node) && node.expression.getText(appTree) === hook &&
        node.arguments[0].getText(appTree).includes(contains)) match = node.arguments[0].getText(appTree);
    ts.forEachChild(node, visit);
  }
  visit(appTree); assert.ok(match, `missing ${hook} startup seam`);
  return ts.transpileModule(`const runHook = ${match}; runHook();`, {}).outputText;
}

test("pending native backdrop cannot prevent status request or lock-listener registration", async () => {
  const h = harness();
  let refreshed = 0, listenerReady = false;
  const events = [];
  const cleanup = vm.runInNewContext(hookBody("useEffect", "applyAppearance"), {
    startupMark() {}, sessionStorage: { getItem: () => null },
    document, toast: { error() {}, info() {} },
    api: { pendingNotice: () => new Promise(() => {}) },
    applyAppearance: () => new Promise(() => {}), cachedAppearance: () => ({}),
    refreshStatus: () => { refreshed++; return Promise.resolve(); },
    listen: (event) => { events.push(event); return Promise.resolve(() => {}); },
    setLockListenerReady: (value) => { listenerReady = value; },
    setStartupFailed: () => assert.fail("successful initialization must not fail"),
    watchStartupInitialization: h.watch,
  });
  assert.equal(refreshed, 1);
  assert.ok(events.includes("vault-locking"));
  assert.ok(events.includes("app-notice"));
  await new Promise(setImmediate); assert.equal(listenerReady, true);
  assert.equal(h.timers.size, 0); cleanup();
});

test("actual readiness effect waits for status/listener, survives cleanup, and sends once across screen changes", () => {
  const h = harness(); let ready = 0;
  const context = vm.createContext({ startupMark() {}, hasStatus: false, lockListenerReady: false, startupFailed: false,
    readySent: { current: false }, scheduleWindowReady: h.schedule, api: { windowReady: () => { ready++; } } });
  const body = hookBody("useLayoutEffect", "scheduleWindowReady").replace("const runHook =", "var runHook =");
  vm.runInContext(body, context); h.tick(); assert.equal(ready, 0);
  context.hasStatus = true;
  vm.runInContext(body, context); h.tick(); assert.equal(ready, 0);
  context.lockListenerReady = true;
  const cleanup = vm.runInContext(body, context); cleanup(); h.tick(); assert.equal(ready, 0);
  vm.runInContext(body, context); h.tick(); assert.equal(ready, 1);
  vm.runInContext(body, context); h.tick(); assert.equal(ready, 1);
});

function deferred() {
  let resolve, reject;
  const promise = new Promise((ok, no) => { resolve = ok; reject = no; });
  return { promise, resolve, reject };
}

// Drive both production App hooks and the production initialization watcher.
// State changes simulate a React commit; only the native IPC boundary is fake.
function appHarness({ status = () => Promise.resolve(), listeners = {}, noticeFails = false } = {}) {
  const h = harness();
  const events = [], disposed = [];
  let ready = 0, attempts = 0, mounted = true, layoutCleanup;
  const layoutBody = hookBody("useLayoutEffect", "scheduleWindowReady").replace("const runHook =", "var runHook =");
  const context = vm.createContext({
    startupMark() {}, hasStatus: false, lockListenerReady: false, startupFailed: false,
    readySent: { current: false }, scheduleWindowReady: h.schedule, watchStartupInitialization: h.watch,
    document, sessionStorage: { getItem: () => null }, toast: { error() {}, info() {} },
    api: {
      pendingNotice: () => Promise.resolve(null),
      windowReady: () => { ready++; return Promise.resolve(); },
    },
    // The backdrop never resolves in these tests: it must not gate readiness.
    applyAppearance: () => new Promise(() => {}), cachedAppearance: () => ({}),
    refreshStatus: async () => {
      await status(++attempts);
      context.hasStatus = true;
      if (mounted) render();
    },
    listen: (event) => {
      events.push(event);
      if (event === "app-notice" && noticeFails) return Promise.reject(new Error("notice unavailable"));
      return (listeners[event] ? listeners[event]() : Promise.resolve()).then(() => () => disposed.push(event));
    },
    setLockListenerReady: (value) => {
      assert.ok(mounted, "no listener state update after cleanup");
      context.lockListenerReady = value; render();
    },
    setStartupFailed: (value) => {
      assert.ok(mounted, "no failure state update after cleanup");
      context.startupFailed = value; render();
    },
  });
  function render() {
    layoutCleanup?.();
    layoutCleanup = vm.runInContext(layoutBody, context);
  }
  render();
  const cleanup = vm.runInContext(hookBody("useEffect", "applyAppearance").replace("const runHook =", "var runHook ="), context);
  return { ...h, context, events, disposed, render,
    ready: () => ready, attempts: () => attempts,
    settle: async () => { await new Promise(setImmediate); h.tick(); await new Promise(setImmediate); },
    cleanup: () => { layoutCleanup?.(); cleanup(); mounted = false; },
  };
}

test("status rejects: one retry also rejects, then window_ready is sent once", async () => {
  const pending = deferred();
  const h = appHarness({ status: () => Promise.reject(new Error("status unavailable")),
    listeners: { "vault-locked": () => pending.promise } });
  await h.settle();
  assert.equal(h.attempts(), 2); assert.equal(h.context.hasStatus, false);
  assert.equal(h.ready(), 1); assert.equal(h.timers.size, 0);
  pending.resolve(); await h.settle(); h.advance(h.bound); h.render(); await h.settle();
  assert.equal(h.ready(), 1); h.cleanup(); await h.settle();
  assert.equal(h.disposed.length, 3);
});

test("status rejects once then succeeds: readiness follows the recovered status without waiting for the bound", async () => {
  const h = appHarness({ status: (attempt) => attempt === 1 ? Promise.reject(new Error("retry")) : Promise.resolve() });
  await h.settle();
  assert.equal(h.attempts(), 2); assert.equal(h.context.hasStatus, true);
  assert.equal(h.context.lockListenerReady, true); assert.equal(h.context.startupFailed, false);
  assert.equal(h.ready(), 1); assert.equal(h.timers.size, 0);
  h.advance(h.bound); h.render(); await h.settle(); assert.equal(h.ready(), 1);
  h.cleanup(); await h.settle();
});

test("first lock listener rejects: window_ready is sent once even while status is pending", async () => {
  const h = appHarness({ status: () => new Promise(() => {}),
    listeners: { "vault-locked": () => Promise.reject(new Error("first listener")) } });
  await h.settle();
  assert.equal(h.context.lockListenerReady, false); assert.equal(h.context.hasStatus, false);
  assert.equal(h.ready(), 1); assert.equal(h.timers.size, 0);
  h.cleanup(); await h.settle();
  assert.deepEqual(h.disposed.sort(), ["app-notice", "vault-locking"]);
});

test("second lock listener rejects: window_ready is sent once and cleanup handles the rejected registration", async () => {
  const h = appHarness({ listeners: { "vault-locking": () => Promise.reject(new Error("second listener")) } });
  await h.settle();
  assert.equal(h.context.lockListenerReady, false); assert.equal(h.ready(), 1);
  assert.equal(h.timers.size, 0); h.cleanup(); await h.settle();
  assert.deepEqual(h.disposed.sort(), ["app-notice", "vault-locked"]);
});

test("both lock listeners reject: window_ready is sent once with no unhandled registration or cleanup rejection", async () => {
  const h = appHarness({ listeners: {
    "vault-locked": () => Promise.reject(new Error("first listener")),
    "vault-locking": () => Promise.reject(new Error("second listener")),
  } });
  await h.settle();
  assert.equal(h.context.lockListenerReady, false); assert.equal(h.ready(), 1);
  assert.equal(h.timers.size, 0); h.cleanup(); await h.settle();
  assert.deepEqual(h.disposed, ["app-notice"]);
});

test("status never resolves within the bound: window_ready is sent once at 5000 ms and late status cannot resend", async () => {
  const pending = deferred();
  const h = appHarness({ status: () => pending.promise });
  assert.equal(h.bound, 5000);
  await h.settle(); h.advance(4999); await h.settle(); assert.equal(h.ready(), 0);
  h.advance(1); await h.settle();
  assert.equal(h.attempts(), 1); assert.equal(h.ready(), 1); assert.equal(h.timers.size, 0);
  pending.resolve(); await h.settle(); h.render(); await h.settle(); assert.equal(h.ready(), 1);
  h.cleanup(); await h.settle();
});

test("pending lock registration is also bounded and is removed when it completes after cleanup", async () => {
  const pending = deferred();
  const h = appHarness({ listeners: { "vault-locking": () => pending.promise } });
  await h.settle(); assert.equal(h.ready(), 0);
  h.advance(h.bound); await h.settle(); assert.equal(h.ready(), 1);
  h.cleanup(); pending.resolve(); await h.settle();
  assert.equal(h.disposed.length, 3); assert.equal(h.ready(), 1);
});

test("initialization cleanup cancels the bound and ignores late failures and listener completion", async () => {
  const status = deferred(), listener = deferred();
  const h = appHarness({ status: () => status.promise, listeners: { "vault-locked": () => listener.promise } });
  h.cleanup(); assert.equal(h.timers.size, 0);
  h.advance(h.bound); status.reject(new Error("late status")); listener.resolve(); await h.settle();
  assert.equal(h.ready(), 0); assert.equal(h.attempts(), 1); assert.equal(h.disposed.length, 3);
});

test("late rejection after timeout is handled without a retry or duplicate readiness", async () => {
  const status = deferred();
  const h = appHarness({ status: () => status.promise });
  h.advance(h.bound); await h.settle(); assert.equal(h.ready(), 1);
  status.reject(new Error("late status")); await h.settle();
  assert.equal(h.attempts(), 1); assert.equal(h.ready(), 1);
  h.cleanup(); await h.settle();
});

test("notice-listener rejection does not disrupt readiness or produce an unhandled cleanup rejection", async () => {
  const h = appHarness({ noticeFails: true });
  await h.settle(); assert.equal(h.ready(), 1); assert.equal(h.context.startupFailed, false);
  assert.ok(h.events.includes("app-notice"));
  h.cleanup(); await h.settle();
  assert.deepEqual(h.disposed.sort(), ["vault-locked", "vault-locking"]);
});

test("late cached backdrop completion cannot overwrite loaded appearance settings", async () => {
  const themeSource = readFileSync(new URL("../../src/lib/theme.ts", import.meta.url), "utf8");
  const themeJs = ts.transpileModule(themeSource, { compilerOptions: { module: ts.ModuleKind.CommonJS } }).outputText;
  const pending = [], stored = [];
  const root = { dataset: {}, style: { setProperty() {} } };
  const exports = {};
  vm.runInNewContext(themeJs, { exports, document: { documentElement: root },
    require: (name) => {
      if (name === "./ipc") return { api: { setBackdrop: () => new Promise((resolve) => pending.push(resolve)) } };
      if (name === "./storage") return { getKey: () => null, setKey: (_, value) => stored.push(JSON.parse(value)) };
      throw new Error(`Unexpected import: ${name}`);
    },
  });
  const cached = exports.applyAppearance(exports.DEFAULT_APPEARANCE, true);
  const settings = exports.applyAppearance({ ...exports.DEFAULT_APPEARANCE, theme: "paper", backdrop: "mica" }, true);
  pending[1](true); await settings;
  pending[0](false); await cached;
  assert.equal(root.dataset.theme, "paper"); assert.equal(root.dataset.backdrop, "on");
  assert.equal(stored.length, 1); assert.equal(stored[0].theme, "paper");
});
