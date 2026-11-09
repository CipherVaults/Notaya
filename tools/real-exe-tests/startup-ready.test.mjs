// No browser or exe: exercise the production scheduler with frames/timers stopped.
// Uses the project's existing TypeScript compiler; installs nothing.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import vm from "node:vm";
import ts from "typescript";

const source = readFileSync(new URL("../../src/lib/startup.ts", import.meta.url), "utf8");
const js = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS } }).outputText;

function harness() {
  const tasks = [];
  const exports = {};
  const window = {
    setTimeout: () => assert.fail("readiness must not wait for a timer"),
    requestAnimationFrame: () => assert.fail("readiness must not wait for a paint"),
  };
  vm.runInNewContext(js, { exports, window, queueMicrotask: (fn) => tasks.push(fn), performance: { now: () => 100 } });
  return { schedule: exports.scheduleWindowReady, mark: exports.startupMark, window, tasks,
    tick: () => { const pending = tasks.splice(0); pending.forEach((fn) => fn()); } };
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
  let refreshed = 0, listenerReady = false;
  const events = [];
  vm.runInNewContext(hookBody("useEffect", "applyAppearance"), {
    startupMark() {}, sessionStorage: { getItem: () => null },
    toast: { error() {}, info() {} }, api: { takeNotice: () => Promise.resolve(null) },
    applyAppearance: () => new Promise(() => {}), cachedAppearance: () => ({}),
    refreshStatus: () => { refreshed++; return Promise.resolve(); },
    listen: (event) => { events.push(event); return Promise.resolve(() => {}); },
    setLockListenerReady: (value) => { listenerReady = value; },
  });
  assert.equal(refreshed, 1);
  assert.ok(events.includes("vault-locking"));
  await new Promise(setImmediate); assert.equal(listenerReady, true);
});

test("actual readiness effect waits for status/listener, survives cleanup, and sends once across screen changes", () => {
  const h = harness(); let ready = 0;
  const context = vm.createContext({ startupMark() {}, hasStatus: false, lockListenerReady: false,
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
