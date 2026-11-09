// No browser or exe: exercise the actual App.tsx scheduler with deferred frames.
// Uses the project's existing TypeScript compiler; installs nothing.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import vm from "node:vm";
import ts from "typescript";

const source = readFileSync(new URL("../../src/App.tsx", import.meta.url), "utf8");
const tree = ts.createSourceFile("App.tsx", source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
const seam = tree.statements.find((s) => ts.isFunctionDeclaration(s) && s.name?.text === "scheduleWindowReady");
assert.ok(seam, "App must expose its startup scheduling seam");
const js = ts.transpileModule(seam.getText(tree), { compilerOptions: { module: ts.ModuleKind.CommonJS } }).outputText;

function harness() {
  let id = 0;
  const timers = new Map(), frames = new Map();
  const exports = {};
  vm.runInNewContext(js, { exports, window: {
    setTimeout: (fn, ms) => { assert.equal(ms, 1000); timers.set(++id, fn); return id; },
    clearTimeout: (key) => timers.delete(key),
    requestAnimationFrame: (fn) => { frames.set(++id, fn); return id; },
    cancelAnimationFrame: (key) => frames.delete(key),
  } });
  const tick = (queue) => { const pending = [...queue.values()]; queue.clear(); pending.forEach((fn) => fn()); };
  return { schedule: exports.scheduleWindowReady, timers, frames, frame: () => tick(frames), timeout: () => tick(timers) };
}

test("hidden WebView: readiness completes without a single animation frame", () => {
  const h = harness(); let ready = 0;
  h.schedule(() => ready++);
  assert.equal(ready, 0);
  h.timeout();
  assert.equal(ready, 1);
  assert.equal(h.frames.size, 0);
});

test("ordinary startup keeps the two-frame path and cancels the fallback", () => {
  const h = harness(); let ready = 0;
  h.schedule(() => ready++);
  h.frame(); assert.equal(ready, 0);
  h.frame(); assert.equal(ready, 1);
  h.timeout(); assert.equal(ready, 1);
  assert.equal(h.timers.size, 0);
});

test("fallback between frames is once-only, including a late frame callback", () => {
  const h = harness(); let ready = 0;
  h.schedule(() => ready++); h.frame();
  const lateFrame = [...h.frames.values()][0];
  h.timeout(); lateFrame();
  assert.equal(ready, 1);
});

test("effect cleanup cancels both callbacks; remount has independent readiness", () => {
  const h = harness(); let ready = 0;
  const cancel = h.schedule(() => ready++);
  const lateTimer = [...h.timers.values()][0];
  cancel(); cancel(); lateTimer(); h.frame(); h.timeout();
  assert.equal(ready, 0);
  h.schedule(() => ready++); h.timeout();
  assert.equal(ready, 1);
});
