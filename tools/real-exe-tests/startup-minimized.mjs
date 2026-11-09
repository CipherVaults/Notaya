// Run ONLY through run-startup-minimized.ps1, on its newly-created throwaway home.
// Node 24 built-ins only. CDP observes DOM/layout, not proof of compositor pixels.
import { writeFile } from "node:fs/promises";
import { setTimeout as sleep } from "node:timers/promises";
import { randomUUID } from "node:crypto";

function options(argv) {
  const out = {};
  for (let i = 0; i < argv.length; i += 2) {
    if (!argv[i]?.startsWith("--") || argv[i + 1] === undefined) throw new Error("Expected --name value");
    out[argv[i].slice(2)] = argv[i + 1];
  }
  return out;
}
const args = options(process.argv.slice(2));
const port = Number(args.port);
const timeout = Number(args["timeout-ms"] ?? 45000);
const delayPage = Number(args["delay-page-ms"] ?? 0);
const holdFrames = Number(args["hold-frames-ms"] ?? 0);
if (!Number.isInteger(port) || port < 1024 || port > 65535 ||
    ![timeout, delayPage, holdFrames].every((n) => Number.isInteger(n) && n >= 0 && n <= 120000)) {
  throw new Error("Invalid port or bounded timeout");
}
if (!["hidden", "visible"].includes(args.expect)) throw new Error("--expect hidden|visible is required");

async function connect() {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json`, { signal: AbortSignal.timeout(2000) });
      const targets = await response.json();
      const target = targets.find((t) => t.type === "page" && /^(https?:\/\/tauri\.localhost|tauri:\/\/localhost)/.test(t.url))
        ?? targets.find((t) => t.type === "page");
      if (target) return await connection(target.webSocketDebuggerUrl);
    } catch { /* page or WebView2 may still be starting */ }
    await sleep(250);
  }
  throw new Error("No CDP page: inspect native window/error dialog and sanitized app log; this is not a DOM pass");
}

async function connection(address) {
  const url = new URL(address);
  if (url.protocol !== "ws:" || !["127.0.0.1", "localhost"].includes(url.hostname) || Number(url.port) !== port) {
    throw new Error("CDP address must stay on the child process's loopback port");
  }
  const ws = new WebSocket(address);
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => { ws.close(); reject(new Error("CDP connection timeout")); }, 3000);
    ws.onopen = () => { clearTimeout(timer); resolve(); };
    ws.onerror = () => { clearTimeout(timer); reject(new Error("CDP connection failed")); };
  });
  let id = 0;
  const pending = new Map(), listeners = new Map();
  const rejectPending = () => {
    for (const p of pending.values()) { clearTimeout(p.timer); p.reject(new Error("CDP disconnected")); }
    pending.clear();
  };
  ws.onclose = rejectPending;
  ws.onmessage = ({ data }) => {
    const message = JSON.parse(data);
    if (message.id) {
      const p = pending.get(message.id);
      if (p) {
        clearTimeout(p.timer); pending.delete(message.id);
        if (message.error) p.reject(new Error(`CDP ${p.method} failed`)); else p.resolve(message.result);
      }
    } else {
      for (const listener of listeners.get(message.method) ?? []) listener(message.params);
    }
  };
  const send = (method, params = {}) => new Promise((resolve, reject) => {
    const key = ++id;
    const timer = setTimeout(() => { pending.delete(key); reject(new Error(`CDP ${method} timed out`)); }, 5000);
    pending.set(key, { resolve, reject, timer, method });
    ws.send(JSON.stringify({ id: key, method, params }));
  });
  return {
    send,
    on: (event, fn) => listeners.set(event, [...(listeners.get(event) ?? []), fn]),
    evaluate: async (expression) => {
      const result = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
      if (result.exceptionDetails) throw new Error("Page evaluation failed (content intentionally omitted)");
      return result.result?.value;
    },
    close: () => { rejectPending(); ws.close(); },
  };
}

// Installed before the reload's bundle. No change to the production app or files.
function frameProbe(ms, documentId) {
  const start = performance.now();
  const probe = window.__notayaStartupProbe = {
    documentId, startedAtMs: start, readyCalls: 0, readyCompleted: 0, readyRejected: 0, readyAtMs: null,
    hookInstalled: false, fetchHookInstalled: false, nativeHookInstalled: false,
    callbackHookFailed: false, heldFrames: 0, framesReleased: !ms, framesReleasedAtMs: null,
    effectRan: 0, startupEffectRan: 0, timerScheduled: 0, timerFired: 0,
    microtaskScheduled: 0, microtaskFired: 0, readyCleanup: 0, appReadyCalls: 0, appReadyRejected: 0,
    bootstrapAtMs: null, appearanceStartedAtMs: null, appearanceCompletedAtMs: null,
    statusRequestedAtMs: null, statusSetAtMs: null, lockListenerReadyAtMs: null,
    screenCommittedAtMs: null, invokeCalledAtMs: null, invokeCompletedAtMs: null,
  };
  // Tauri's invoke/ipc properties are read-only. Observe its actual transports
  // and existing callback map instead; never claim an assignment hooked invoke.
  const seen = new Set();
  const observe = (cmd, success, failure) => {
    if (cmd !== "window_ready" || seen.has(success)) return;
    seen.add(success); probe.readyCalls++;
    try {
      const callbacks = window.__TAURI_INTERNALS__?.callbacks;
      const ok = callbacks?.get(success), fail = callbacks?.get(failure);
      if (!ok || !fail) { probe.callbackHookFailed = true; return; }
      callbacks.set(success, (value) => {
        probe.readyCompleted++; probe.readyAtMs = performance.now() - start;
        return ok(value);
      });
      callbacks.set(failure, (value) => { probe.readyRejected++; return fail(value); });
    } catch { probe.callbackHookFailed = true; }
  };
  const originalFetch = window.fetch;
  const observedFetch = function (input, init) {
    // Tauri sends a URL string and a Headers object. Do not read/log bodies,
    // command arguments, invoke keys or results, and forward everything intact.
    if (typeof input === "string" && /^(?:https?:\/\/ipc\.localhost|ipc:\/\/localhost)\/window_ready(?:[?#]|$)/.test(input)) {
      const headers = new Headers(init?.headers);
      observe("window_ready", Number(headers.get("Tauri-Callback")), Number(headers.get("Tauri-Error")));
    }
    return originalFetch.apply(this, arguments);
  };
  probe.fetchHookInstalled = Reflect.set(window, "fetch", observedFetch) && window.fetch === observedFetch;
  const hookNative = () => {
    const ipc = window.ipc;
    if (!ipc?.postMessage) { if (performance.now() - start < 30000) setTimeout(hookNative, 10); return; }
    const original = ipc.postMessage;
    const observed = function (data) {
      try {
        const message = typeof data === "string" ? JSON.parse(data) : data;
        observe(message?.cmd, message?.callback, message?.error);
      } catch { /* native transport remains untouched for an unrecognized frame */ }
      return original.apply(this, arguments);
    };
    probe.nativeHookInstalled = Reflect.set(ipc, "postMessage", observed) && ipc.postMessage === observed;
  };
  hookNative();
  probe.hookInstalled = probe.fetchHookInstalled;
  if (!ms) return;
  const request = window.requestAnimationFrame.bind(window), cancel = window.cancelAnimationFrame.bind(window);
  let id = 0; const held = new Map();
  window.requestAnimationFrame = (fn) => { probe.heldFrames++; held.set(--id, fn); return id; };
  window.cancelAnimationFrame = (key) => { if (key < 0) held.delete(key); else cancel(key); };
  probe.releaseFrames = () => {
    if (probe.framesReleased) return;
    window.requestAnimationFrame = request; window.cancelAnimationFrame = cancel;
    probe.framesReleased = true; probe.framesReleasedAtMs = performance.now() - start;
    for (const fn of held.values()) request(fn);
    held.clear();
  };
  setTimeout(probe.releaseFrames, ms);
}

const snapshotExpression = `(async () => {
  const root = document.getElementById('root');
  const title = root?.querySelector('.title-bar');
  const screen = title?.nextElementSibling?.firstElementChild;
  const rect = screen?.getBoundingClientRect();
  const css = screen ? getComputedStyle(screen) : null;
  let visible = null;
  try { visible = await window.__TAURI_INTERNALS__.invoke('plugin:window|is_visible', { label: 'main' }); } catch {}
  return {
    label: window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label ?? null,
    documentReady: document.readyState,
    documentTitle: document.title.slice(0, 120),
    documentHidden: document.hidden,
    rootHasContent: !!root?.childElementCount,
    titleBarText: title?.textContent?.trim().slice(0, 200) ?? '',
    screenHasContent: !!screen?.childElementCount,
    screenHasLayout: !!rect && rect.width > 0 && rect.height > 0,
    screenOpacity: css?.opacity ?? null,
    screenVisibility: css?.visibility ?? null,
    nativeVisible: visible,
    probe: window.__notayaStartupProbe ? { ...window.__notayaStartupProbe, releaseFrames: undefined } : null
  };
})()`;

let cdp;
const report = { phase: args.phase ?? "initial", expectedVisibility: args.expect, delayPageMs: delayPage,
  holdFramesMs: holdFrames, checks: {}, elapsedMs: null, snapshot: null };
const started = Date.now();
const documentId = randomUUID();
try {
  cdp = await connect();
  if (args.phase === "quit") {
    await cdp.send("Runtime.evaluate", { expression: "void window.__TAURI_INTERNALS__.invoke('quit_app')" });
    report.checks.quitRequested = true;
    report.passed = true;
  } else {
  await cdp.send("Page.enable");
  if (delayPage || holdFrames) {
    await cdp.send("Page.addScriptToEvaluateOnNewDocument", { source: `(${frameProbe.toString()})(${holdFrames}, ${JSON.stringify(documentId)})` });
    // Hold the bundle on a hidden reload. This simulates a slow page, not an OS sign-in/GPU failure.
    // Continue all requests after the bound, including ones that arrive late.
    const requests = new Set(); let released = false;
    const continueRequest = (requestId) => cdp.send("Fetch.continueRequest", { requestId }).catch(() => {});
    if (delayPage) {
      cdp.on("Fetch.requestPaused", ({ requestId }) => {
        if (released) void continueRequest(requestId); else requests.add(requestId);
      });
      await cdp.send("Fetch.enable", { patterns: [{ urlPattern: "*", resourceType: "Script", requestStage: "Request" }] });
    }
    await cdp.send("Page.reload", { ignoreCache: true });
    if (delayPage) {
      await sleep(delayPage);
      report.delayedSnapshot = await cdp.evaluate(snapshotExpression).catch(() => null);
      report.pausedScriptRequests = requests.size;
      released = true;
      await Promise.all([...requests].map(continueRequest));
      await cdp.send("Fetch.disable");
      report.checks.delayActuallyApplied = requests.size > 0;
    }
  }
  // Poll DOM while hidden; do not take a screenshot or force window_ready (either can mask the race).
  const deadline = Date.now() + timeout;
  do {
    try { report.snapshot = await cdp.evaluate(snapshotExpression); } catch { /* reload in progress */ }
    const s = report.snapshot;
    if (s?.label === "main" && s.rootHasContent && s.titleBarText.includes("Notaya") && s.screenHasContent && s.screenHasLayout &&
        s.nativeVisible === (args.expect === "visible")) {
      if (!(delayPage || holdFrames) || (s.probe?.documentId === documentId && s.probe.readyCompleted > 0)) break;
    }
    await sleep(200);
  } while (Date.now() < deadline);
  const s = report.snapshot;
  report.checks.mainDOM = s?.label === "main" && s.rootHasContent && s.titleBarText.includes("Notaya") && s.screenHasContent && s.screenHasLayout;
  report.checks.visibility = s?.nativeVisible === (args.expect === "visible");
  if (delayPage || holdFrames) {
    report.checks.probeDocument = s?.probe?.documentId === documentId && s?.label === "main";
    report.checks.readyObserved = s?.probe?.hookInstalled === true && s.probe.readyCompleted === 1 &&
      s.probe.readyCalls === 1 && !s.probe.callbackHookFailed && s.probe.readyRejected === 0;
  }
  if (holdFrames) {
    report.checks.readyWithoutFrames = !!s?.probe?.hookInstalled && s.probe.readyCompleted > 0 &&
      s.probe.heldFrames > 0 && s.probe.readyAtMs < holdFrames &&
      (s.probe.framesReleasedAtMs === null || s.probe.readyAtMs < s.probe.framesReleasedAtMs);
    // Preserve the readiness observation before frames resume. Let the test-only
    // shim expire before the runner restores the window and inspects animation.
    const releaseDeadline = Date.now() + timeout;
    let released = s?.probe?.framesReleased;
    while (!released && Date.now() < releaseDeadline) {
      await sleep(200);
      // A hidden page may throttle timers. CDP releases only the test shim,
      // after the requested hold; it never calls the app's ready command.
      await cdp.evaluate(`(() => { const p = window.__notayaStartupProbe; if (p && performance.now() - p.startedAtMs >= ${holdFrames}) p.releaseFrames?.(); })()`);
      released = await cdp.evaluate("window.__notayaStartupProbe?.framesReleased").catch(() => false);
    }
    report.checks.frameShimReleased = released === true;
    report.afterReleaseProbe = await cdp.evaluate("(() => { const p = window.__notayaStartupProbe; return p ? { ...p, releaseFrames: undefined } : null; })()");
  }
  if (args.expect === "visible") {
    // Animations must settle after a tray restore; DOM existing alone is insufficient.
    await sleep(800);
    report.snapshot = await cdp.evaluate(snapshotExpression);
    report.checks.visibleLayout = Number(report.snapshot.screenOpacity) > 0 && report.snapshot.screenVisibility === "visible";
  }
  report.passed = Object.values(report.checks).every((v) => v === true);
  }
} catch (error) {
  report.passed = false; report.error = error.message;
} finally {
  cdp?.close();
  report.elapsedMs = Date.now() - started;
  console.log(JSON.stringify(report, null, 2));
  if (args.out) await writeFile(args.out, JSON.stringify(report, null, 2) + "\n");
}
process.exitCode = report.passed ? 0 : 1;
