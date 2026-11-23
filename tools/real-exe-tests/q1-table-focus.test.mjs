// Headed React DOM regression for compact-table focus. Run with:
//   node tools/real-exe-tests/q1-table-focus.test.mjs
// Requires a graphical session and compatible headed Chromium and Playwright
// with connectOverCDP({ noDefaults: true }) and routeWebSocket support.
// Set PW_PATH to a Playwright entry point or install playwright/playwright-core.
// Set CHROME_PATH to the Chromium executable or use Playwright's installed browser.
// The Git object for cbf0193f must be available for the preimage comparison.
// Q1_TMP_DIR and Q1_RECEIPT_DIR override the OS temp-based scratch and receipt roots.
import assert from "node:assert/strict";
import { execFileSync, spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { createConnection } from "node:net";
import os from "node:os";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath } from "node:url";
import { build } from "vite";
import react from "@vitejs/plugin-react";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const require = createRequire(import.meta.url);
const tablePath = "src/features/database/views/TableView.tsx";
const preimageCommit = "cbf0193fabe77d04cdd84f40e2560f8dd643c333";
const pinnedPaths = [tablePath, "src/features/database/context.tsx", "src/features/database/undo.ts"];
const wallTimeMs = 120000;
const rendererCleanupMs = 1000;
const windows = process.platform === "win32";
const inputPosition = { x: 80, y: 12 };
const backgroundCase = "background-document-declines-repair";

// V5 fixture, using the production TableView, context and undo modules.
const fixture = String.raw`
import React, { StrictMode, useCallback, useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { flushSync } from "react-dom";
import TableView from "@/features/database/views/TableView";
import { DbShellContext } from "@/features/database/context";
import { handleUndoKey, pushUndo, retainUndo, undoDepth } from "@/features/database/undo";
const seedRows = Array.from({ length: 80 }, (_, i) => ({
  id: "r" + (i + 1), title: "Row " + (i + 1), icon: "", number: i + 1,
  created: 1, updated: 1, favorite: false, values: {},
}));
const prop = {
  id: "title", name: "Name", kind: "title", options: [], numberFormat: "", numberDisplay: "",
  numberMax: 0, dateFormat: "", formula: "", relationDb: "", relationBack: "", relationLimitOne: false,
  rollupRelation: "", rollupProp: "", rollupCalc: "", idPrefix: "", description: "", buttonLabel: "", actions: [],
};
const groupBy = { prop: "title", sort: "manual", dateBy: "", numberStep: 0, hideEmpty: false, hidden: [], collapsed: [] };
function makeView(cfg: any, id: string) {
  return {
    id, name: "Table", kind: "table", filter: { op: "and", conds: [], groups: [] },
    sorts: cfg.sort ? [{ prop: "title", descending: false }] : [],
    group: cfg.group ? { ...groupBy } : null, subGroup: cfg.group === "subgroup" ? { ...groupBy } : null,
    columns: [{ prop: "title", visible: true, width: 280 }], calcs: [], freeze: 0, wrap: !!cfg.wrap,
    rowNumbers: false, openIn: "full", cardSize: "", cardCover: "", cardFit: false, dateProp: "",
    endProp: "", calendarMode: "", timelineZoom: "", showDeps: false, showSubItems: false, chart: {},
  };
}
function groupsFor(kind: string | null, rows: any[]) {
  const ids = rows.map(r => r.id);
  const group = (key: string, rowIds: string[], subs: any[] = []) => ({ key, label: key, color: "", count: rowIds.length, rows: rowIds, subs, calcs: {} });
  if (kind === "subgroup")
    return [group("g", ids, [group("s", ids)])];
  if (kind === "duplicate")
    return [group("g1", ids), group("g2", ["r2"])];
  return kind ? [group("g", ids)] : null;
}
const audit = { writes: [] as string[], unexpectedShell: [] as string[] };
(window as any).q1FixtureAudit = audit;
let serial = 0;
const root = createRoot(document.getElementById("mount")!);
function App({ cfg, token }: { cfg: any; token: number }) {
  const [state, setState] = useState<any>(() => ({
    rows: seedRows, view: makeView(cfg, "v1"), dbId: "db-" + token,
    mounted: true, hidden: false, dialog: null, autofocus: false, tick: 0,
  }));
  const stats = useRef({
    writes: [] as any[], unexpected: [] as string[], undo: 0, redo: 0, rootKeys: 0, probe: { attach: 0, detach: 0 },
  });
  const probe = useCallback((node: HTMLElement | null) => { stats.current.probe[node ? "attach" : "detach"]++; }, []);
  const db: any = {
    id: state.dbId, sectionId: "section", parent: "", inline: true, title: "Fixture", icon: "", description: "",
    order: 0, favorite: false, locked: false, props: [prop], views: [state.view], templates: [], defaultTemplate: "",
    nextId: 81, subParentProp: "", subChildProp: "", blockedByProp: "", blockingProp: "", automations: [],
    trashedAt: null, created: 1, updated: 1,
  };
  const result: any = { rows: state.rows, total: 80, filtered: state.rows.length, truncated: false, groups: groupsFor(cfg.group, state.rows), calcs: {} };
  const unexpected = (name: string) => {
    stats.current.unexpected.push(name);
    audit.unexpectedShell.push(name);
    throw new Error("Unexpected shell operation: " + name);
  };
  useEffect(() => {
    const release = retainUndo(state.dbId);
    pushUndo(state.dbId, {
      label: "fixture add row",
      undo: () => {
        stats.current.undo++;
        setState((s: any) => ({ ...s, rows: s.rows.filter((r: any) => r.id !== "r2") }));
      },
      redo: () => {
        stats.current.redo++;
        setState((s: any) => ({ ...s, rows: seedRows }));
      },
    });
    return release;
  }, [state.dbId]);
  const shell: any = { db, view: state.view, databases: [], locked: false, compact: true,
    editDb: async () => unexpected("editDb"), editView: (f: any) => setState((s: any) => ({ ...s, view: f(s.view) })),
    trashRows: async () => unexpected("trashRows"), duplicateRow: async () => unexpected("duplicateRow"), requery: () => unexpected("requery"),
    rowById: new Map(state.rows.map((r: any) => [r.id, r])), focusRowId: null, clearFocusRow: () => { }, focusRow: () => unexpected("focusRow"),
    patchRowLocal: () => unexpected("patchRowLocal"), openProp: () => unexpected("openProp"), sampleOf: () => null };
  const props: any = { db, view: state.view, result, rowById: shell.rowById, locked: false, openRow: () => unexpected("openRow"),
    setTitle: async (id: string, title: string) => { audit.writes.push("title"); stats.current.writes.push({ kind: "title", id, title }); },
    setCells: async (id: string, patch: any) => { audit.writes.push("cells"); stats.current.writes.push({ kind: "cells", id, patch }); },
    createRow: async () => unexpected("createRow"),
    updateView: (patch: any) => setState((s: any) => ({ ...s, view: { ...s.view, ...patch } })),
    reorderRows: async () => unexpected("reorderRows") };
  (window as any).fixture = {
    ready: true,
    mutate: (kind: string) => flushSync(() => setState((s: any) => {
      if (kind === "remove" || kind === "filter")
        return { ...s, rows: s.rows.filter((r: any) => r.id !== "r2") };
      if (kind === "other-remove")
        return { ...s, rows: s.rows.filter((r: any) => r.id !== "r3") };
      if (kind === "collapse")
        return { ...s, view: { ...s.view, group: { ...s.view.group, collapsed: [cfg.group === "duplicate" ? "g2" : "g"] } } };
      if (kind === "subgroup")
        return { ...s, view: { ...s.view, group: { ...s.view.group, collapsed: ["g/s"] } } };
      if (kind === "scope-db")
        return { ...s, rows: s.rows.filter((r: any) => r.id !== "r2"), dbId: s.dbId + "-new" };
      if (kind === "scope-view")
        return { ...s, rows: s.rows.filter((r: any) => r.id !== "r2"), view: { ...s.view, id: "v2" } };
      if (kind === "unmount" || kind === "navigate")
        return { ...s, mounted: false };
      if (kind === "hidden")
        return { ...s, rows: s.rows.filter((r: any) => r.id !== "r2"), hidden: true };
      if (kind === "wrap")
        return { ...s, view: { ...s.view, wrap: !s.view.wrap } };
      if (kind === "sort")
        return { ...s, view: { ...s.view, sorts: s.view.sorts.length ? [] : [{ prop: "title", descending: false }] } };
      if (kind === "modal-remove")
        return { ...s, rows: s.rows.filter((r: any) => r.id !== "r2"), dialog: "unfocused" };
      if (kind === "autofocus-remove")
        return { ...s, rows: s.rows.filter((r: any) => r.id !== "r2"), autofocus: true };
      if (kind === "dismiss-modal")
        return { ...s, dialog: null, tick: s.tick + 1 };
      return { ...s, tick: s.tick + 1 };
    })),
    snapshot: () => ({ focusTag: document.activeElement?.tagName, focusId: (document.activeElement as HTMLElement)?.id,
      inPrimary: !!document.querySelector('[data-test="primary"] .db-scroll')?.contains(document.activeElement),
      wrapper: document.activeElement === document.querySelector('[data-test="primary"] .db-scroll'),
      connected: !!document.activeElement?.isConnected, previousConnected: !!(window as any).previousFocus?.isConnected, sourceRowCount: state.rows.length, modelHasRow2: state.rows.some((r: any) => r.id === "r2"), collapsed: state.view.group?.collapsed ?? [], rows: Array.from(document.querySelectorAll('[data-test="primary"] tr[data-row-id]')).map(x => x.getAttribute("data-row-id")),
      depth: undoDepth(state.dbId), stats: stats.current, dbId: state.dbId, viewId: state.view.id }),
  };
  const otherView = makeView({}, "other-v");
  const otherDb = { ...db, id: "other-" + token, views: [otherView] };
  const otherShell = { ...shell, db: otherDb, view: otherView, rowById: new Map(seedRows.map(r => [r.id, r])) };
  const otherProps = { ...props, db: otherDb, view: otherView, result: { ...result, rows: seedRows, groups: null }, rowById: otherShell.rowById };
  return <>
    <span ref={probe} aria-hidden="true" />
    <div data-test="primary" data-database={db.id} hidden={state.hidden} onKeyDown={e => {
      if (handleUndoKey(db.id, e)) stats.current.rootKeys++;
    }}>
      {state.mounted && <DbShellContext.Provider value={shell}><TableView {...props} /></DbShellContext.Provider>}
    </div>
    <div data-test="other" data-database={otherDb.id} onKeyDown={e => handleUndoKey(otherDb.id, e)}>
      <DbShellContext.Provider value={otherShell}><TableView {...otherProps} /></DbShellContext.Provider>
    </div>
    {state.autofocus && <input id="commit-focus-target" autoFocus />}
    {!state.mounted && <input id="navigation-target" aria-label="New page" autoFocus={cfg.navigate} />}
    {state.dialog && <div role="dialog" id="unfocused-modal"><input aria-label="Modal input" /></div>}
  </>;
}
(window as any).setup = async (cfg: any) => {
  flushSync(() => root.render(<StrictMode><App key={++serial} token={serial} cfg={cfg}/></StrictMode>));
  await new Promise<void>(r => requestAnimationFrame(() => requestAnimationFrame(() => r())));
};
(window as any).settle = () => new Promise<void>(r => requestAnimationFrame(() => requestAnimationFrame(() => r())));
(window as any).dispose = () => flushSync(() => root.unmount());
(window as any).fixtureBooted = true;
`;

const variants = [
  { name: "sortable-fixed", sort: false, wrap: false }, { name: "plain-fixed", sort: true, wrap: false },
  { name: "sortable-wrapped", sort: false, wrap: true }, { name: "plain-wrapped", sort: true, wrap: true }
];
const CASES = [
  ...variants.flatMap(v => ["Control+y", "Control+Shift+z"].map(key => ({ name: "owned-remove-natural-redo/" + v.name + "/" + key, kind: "redo", cfg: v, key }))),
  { name: "filtered-focused-row", kind: "repair", action: "filter", cfg: {} },
  { name: "collapsed-group-focused-row", kind: "repair", action: "collapse", cfg: { group: "group" } },
  { name: "collapsed-subgroup-focused-row", kind: "repair", action: "subgroup", cfg: { group: "subgroup" } },
  { name: "collapsed-duplicate-occurrence-keeps-source-row", kind: "duplicate", cfg: { group: "duplicate" } },
  ...["outside-input", "outside-textarea", "outside-select", "outside-editor", "dialog-input", "other-table"].map(target => ({ name: "preserve-focus/" + target, kind: "external", target, cfg: {} })),
  { name: "preserve-focus/other-row-title-editor", kind: "editor", cfg: {} },
  { name: "owned-input-removal-no-duplicate-commit", kind: "input-remove", cfg: {} },
  { name: "same-commit-external-autofocus-wins", kind: "autofocus", cfg: {} },
  { name: "background-document-declines-repair", kind: "background", cfg: {} },
  { name: "navigation-autofocus", kind: "scope", action: "navigate", cfg: { navigate: true }, expected: "navigation-target" },
  { name: "complete-table-unmount", kind: "scope", action: "unmount", cfg: {} },
  { name: "database-identity-change", kind: "scope", action: "scope-db", cfg: {} },
  { name: "view-identity-change", kind: "scope", action: "scope-view", cfg: {} },
  { name: "hidden-navigation-container", kind: "scope", action: "hidden", cfg: {} },
  { name: "same-props-rerender", kind: "keep", action: "rerender", cfg: {} },
  { name: "measurement-ref-churn-wrap", kind: "keep", action: "wrap", cfg: {} },
  { name: "present-key-sort-branch-change", kind: "no-repair", action: "sort", cfg: {} },
  { name: "virtual-scroll-keeps-owned-cell", kind: "scroll", cfg: {} },
  { name: "unfocused-open-dialog-cancels-pending-repair", kind: "modal", cfg: {} },
  { name: "removing-another-row-keeps-focus", kind: "keep", action: "other-remove", cfg: {} }
];

const expectedPreimageFailures = [
  ...CASES.filter(({ kind }) => kind === "redo").map(({ name }) => name),
  "filtered-focused-row",
  "collapsed-group-focused-row",
  "collapsed-subgroup-focused-row",
  "collapsed-duplicate-occurrence-keeps-source-row",
  "owned-input-removal-no-duplicate-commit",
];

const html = `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <link rel="icon" href="data:,">
  <title>Notaya table focus regression</title>
  <style>
    body { font: 14px sans-serif; }
    table { border-collapse: collapse; }
    td { padding: 3px; }
    .db-scroll { height: 240px; max-height: 240px !important; width: 680px; overflow: auto; }
    tr[data-row-id] { height: 35px; }
    thead { height: 33px; }
    [data-test=other] { margin-top: 12px; }
    #external-dialog { border: 1px solid; display: none; }
  </style>
</head>
<body>
  <input id="outside-input"><textarea id="outside-textarea"></textarea>
  <select id="outside-select"><option>Fixture</option></select>
  <div id="outside-editor" contenteditable="true" tabindex="0">External editor</div>
  <div id="external-dialog" role="dialog"><input id="dialog-input"></div>
  <div class="ProseMirror" contenteditable="true">
    <span id="outer-note-marker">Original note marker</span><div id="mount" contenteditable="false"></div>
  </div>
  <script type="module" src="/entry.js"></script>
</body>
</html>`;

const csp = [
  "default-src 'none'", "script-src 'self'", "style-src 'self' 'unsafe-inline'",
  "img-src data:", "connect-src 'none'", "base-uri 'none'", "form-action 'none'",
  "frame-ancestors 'none'", "worker-src 'none'",
].join("; ");

function resolvePlaywright() {
  if (process.env.PW_PATH) return require.resolve(path.resolve(process.env.PW_PATH));
  for (const name of ["playwright", "playwright-core"]) {
    try {
      return require.resolve(name);
    } catch (error) {
      if (error.code !== "MODULE_NOT_FOUND") throw error;
    }
  }
  throw new Error("Playwright unavailable. Set PW_PATH to a local Playwright entry point.");
}

async function bundleTable(source, scratch) {
  const entryId = "virtual:q1-table-focus.tsx";
  const tableModule = path.resolve(repo, tablePath);
  let tableLoads = 0;
  const result = await build({
    root: repo,
    configFile: false,
    envDir: false,
    publicDir: false,
    cacheDir: path.join(scratch, "vite-cache"),
    mode: "development",
    logLevel: "warn",
    define: { "process.env.NODE_ENV": JSON.stringify("development") },
    resolve: { alias: { "@": path.resolve(repo, "src") }, preserveSymlinks: true },
    plugins: [{
      name: "q1-table-focus-fixture",
      enforce: "pre",
      resolveId(id) {
        if (id === entryId) return `\0${id}`;
      },
      load(id) {
        if (id === `\0${entryId}`) return fixture;
        if (path.isAbsolute(id) && path.normalize(id) === tableModule) {
          tableLoads++;
          return source.toString("utf8");
        }
      },
    }, react()],
    build: {
      write: false,
      minify: false,
      sourcemap: false,
      target: "chrome120",
      rollupOptions: { input: entryId, output: { entryFileNames: "entry.js" } },
    },
  });
  assert.equal(tableLoads, 1, "The selected TableView must load at its production module ID");
  const assets = new Map([["/", html]]);
  for (const bundle of Array.isArray(result) ? result : [result]) {
    for (const item of bundle.output) {
      assets.set(`/${item.fileName}`, item.type === "chunk" ? item.code : item.source);
    }
  }
  assert.ok(assets.has("/entry.js"), "Missing fixture entry output");
  return assets;
}

function observeRequest(receipt, request) {
  receipt.unexpectedNetwork.push({ method: request.method, url: request.url });
}

function fixtureServer(assets, receipt) {
  return createServer((request, response) => {
    const asset = assets.get(request.url);
    if (request.method !== "GET" || asset === undefined) {
      observeRequest(receipt, request);
      response.writeHead(404).end();
      return;
    }
    const contentType = request.url === "/" ? "text/html" :
      request.url.endsWith(".css") ? "text/css" : "text/javascript";
    response.writeHead(200, {
      "Content-Type": `${contentType}; charset=utf-8`,
      "Content-Security-Policy": csp,
      "Cache-Control": "no-store",
      "X-Content-Type-Options": "nosniff",
    }).end(asset);
  });
}

function installTauriShim() {
  let nextCallbackId = 0;
  const audit = { editing: [], subscriptions: [], unexpectedIPC: [], cspViolations: [] };
  window.q1BrowserAudit = audit;
  window.__TAURI_INTERNALS__ = {
    transformCallback(callback, once = false) {
      const id = ++nextCallbackId;
      window[`_${id}`] = (value) => {
        if (once) delete window[`_${id}`];
        callback?.(value);
      };
      return id;
    },
    unregisterCallback(id) {
      delete window[`_${id}`];
    },
    invoke(command, args) {
      // dbStore subscribes to this one read-only notification on module load.
      if (command === "plugin:event|listen" && args?.event === "items-changed" &&
          args.target?.kind === "Any" && Object.keys(args.target).length === 1 &&
          Number.isInteger(args.handler) && typeof window[`_${args.handler}`] === "function" &&
          Object.keys(args).length === 3 && audit.subscriptions.length === 0) {
        audit.subscriptions.push(args.event);
        return Promise.resolve(args.handler);
      }
      if (command === "editing" && typeof args?.id === "string" &&
          /^(db|other)-\d+(-new)?$/.test(args.id) && typeof args.active === "boolean" &&
          Object.keys(args).length === 2) {
        audit.editing.push({ id: args.id, active: args.active });
        return Promise.resolve();
      }
      audit.unexpectedIPC.push(String(command));
      throw new Error(`Unexpected fixture Tauri IPC: ${command}`);
    },
  };
  document.addEventListener("securitypolicyviolation", (event) => {
    audit.cspViolations.push({ directive: event.effectiveDirective, url: event.blockedURI });
  });
}

function blockServiceWorkerRegistration() {
  // The CDP default context preserves native tab focus. Apply the same
  // registration block as a Playwright-managed context before page scripts run.
  if (window.ServiceWorkerContainer) {
    Object.defineProperty(ServiceWorkerContainer.prototype, "register", {
      value: async () => { throw new DOMException("Fixture blocks service workers", "SecurityError"); },
    });
  }
}

async function bypassServiceWorkers(context, page) {
  const cdp = await context.newCDPSession(page);
  await cdp.send("Network.setBypassServiceWorker", { bypass: true });
}

function startHeadedChrome(chrome, profile) {
  // Chrome is launched directly without a headless flag. Connecting with
  // noDefaults avoids Playwright's focus emulation on every newly created tab.
  const child = spawn(chrome, [
    `--user-data-dir=${profile}`, "--remote-debugging-address=127.0.0.1", "--remote-debugging-port=0",
    "--no-first-run", "--no-default-browser-check", "--disable-background-networking",
    "--disable-component-update", "--disable-sync", "--disable-extensions",
    "--disable-component-extensions-with-background-pages", "--disable-crashpad-for-testing",
    "--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1", "about:blank",
  ], { detached: !windows, stdio: ["ignore", "ignore", "pipe"] });
  const closed = new Promise((resolve) => child.once("close", resolve));
  let port;
  const endpoint = new Promise((resolve, reject) => {
    let stderr = "";
    const timeout = setTimeout(() => reject(new Error(`Chrome endpoint timeout: ${stderr}`)), 15000);
    child.once("error", (error) => { clearTimeout(timeout); reject(error); });
    child.once("close", (code) => {
      clearTimeout(timeout);
      reject(new Error(`Chrome exited before connection (${code}): ${stderr}`));
    });
    child.stderr.on("data", (chunk) => {
      stderr = (stderr + chunk).slice(-8192);
      const match = /DevTools listening on (ws:\/\/127\.0\.0\.1:\d+\/devtools\/browser\/[^\s]+)\r?\n/.exec(stderr);
      if (match) {
        clearTimeout(timeout);
        port = Number(new URL(match[1]).port);
        resolve(match[1]);
      }
    });
  });
  return { child, closed, profile, endpoint, get port() { return port; } };
}

async function within(work, ms, label) {
  let timer;
  try {
    const deadline = new Promise((_, reject) => {
      timer = setTimeout(() => reject(new Error(`${label} exceeded ${ms} ms`)), ms);
    });
    return await Promise.race([work, deadline]);
  } finally {
    clearTimeout(timer);
  }
}

function processSnapshot() {
  const options = { encoding: "utf8", timeout: 5000, maxBuffer: 8 * 1024 * 1024, windowsHide: true };
  if (windows) {
    const script = "[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)\n" +
      "Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,CreationDate,CommandLine | ConvertTo-Json -Compress";
    const output = execFileSync("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", script], options);
    return [JSON.parse(output)].flat().map((item) => ({
      pid: item.ProcessId, parent: item.ParentProcessId, started: item.CreationDate, args: item.CommandLine,
    }));
  }
  const output = execFileSync("ps", ["-A", "-o", "pid=,ppid=,pgid=,stat=,args="], options);
  return output.trim().split("\n").flatMap((line) => {
    const match = /^\s*(\d+)\s+(\d+)\s+(\d+)\s+(\S+)\s*(.*)$/.exec(line);
    if (!match || match[4].startsWith("Z")) return [];
    return [{ pid: Number(match[1]), parent: Number(match[2]), group: Number(match[3]), args: match[5] }];
  });
}

function ownedChromeProcesses(chrome, seen) {
  const root = chrome.child.pid;
  const profile = windows ? chrome.profile.toLowerCase() : chrome.profile;
  const processes = processSnapshot();
  const owned = new Map();
  for (const item of processes) {
    const args = windows ? item.args?.toLowerCase() : item.args;
    if ((item.pid === root && item.parent === process.pid) ||
        (seen.has(item.pid) && seen.get(item.pid) === item.started) ||
        (!windows && root && item.group === root) ||
        args?.includes(`--user-data-dir=${profile}`) || args?.includes(`--user-data-dir="${profile}"`)) {
      owned.set(item.pid, item);
    }
  }
  let previousSize;
  do {
    previousSize = owned.size;
    for (const item of processes) {
      if (owned.has(item.parent)) owned.set(item.pid, item);
    }
  } while (owned.size !== previousSize);
  for (const item of owned.values()) seen.set(item.pid, item.started);
  return owned;
}

function forceChromeTree(chrome, owned) {
  if (windows) {
    const roots = [...owned.values()].filter(({ parent }) => !owned.has(parent));
    if (!roots.length) return;
    // A process may exit between enumeration and taskkill. Verify by readback.
    const args = roots.flatMap(({ pid }) => ["/PID", String(pid)]);
    const result = spawnSync("taskkill.exe", [...args, "/T", "/F"], { timeout: 3000, windowsHide: true });
    if (result.error) throw result.error;
    return;
  }
  const group = chrome.child.pid ? [-chrome.child.pid] : [];
  for (const pid of new Set([...group, ...owned.keys()])) {
    try {
      process.kill(pid, "SIGKILL");
    } catch (error) {
      if (error.code !== "ESRCH") throw error;
    }
  }
}

async function stopChrome(chrome, browser, timedOut, receipt) {
  const seen = new Map();
  try {
    receipt.chromePids = [...ownedChromeProcesses(chrome, seen).keys()];
    if (!timedOut && browser?.isConnected()) {
      await within((async () => {
        const cdp = await browser.newBrowserCDPSession();
        await cdp.send("Browser.close");
        await chrome.closed;
      })(), rendererCleanupMs, "Graceful Chrome shutdown");
    }
  } catch (error) {
    receipt.gracefulShutdownError = errorText(error);
  } finally {
    try {
      let owned = ownedChromeProcesses(chrome, seen);
      receipt.forcedChromePids = [...owned.keys()];
      for (let attempt = 0; owned.size && attempt < 2; attempt++) {
        forceChromeTree(chrome, owned);
        await delay(150);
        owned = ownedChromeProcesses(chrome, seen);
      }
      receipt.remainingChromePids = [...owned.keys()];
      assert.deepEqual(receipt.remainingChromePids, [], "Chrome process tree must be gone");
      await within(chrome.closed, rendererCleanupMs, "Chrome process exit");
    } finally {
      try {
        if (chrome.child.pid && chrome.child.exitCode === null && chrome.child.signalCode === null) {
          forceChromeTree(chrome, new Map([[chrome.child.pid, { pid: chrome.child.pid, parent: process.pid }]]));
        }
      } finally {
        chrome.child.stderr.destroy();
        chrome.child.unref();
      }
    }
  }
}

function assertPortClosed(port) {
  return new Promise((resolve, reject) => {
    const socket = createConnection({ host: "127.0.0.1", port });
    socket.setTimeout(1500, () => socket.destroy(new Error(`Could not verify closure of port ${port}`)));
    socket.once("connect", () => {
      socket.destroy();
      reject(new Error(`Still listening on 127.0.0.1:${port}`));
    });
    socket.once("error", (error) => error.code === "ECONNREFUSED" ? resolve() : reject(error));
  });
}

const errorText = (error) => String(error.stack || error);
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

async function stopFixturePage(page, receipt, label) {
  const audit = await within(page.evaluate(() => {
    window.dispose?.();
    return { ...window.q1BrowserAudit, ...window.q1FixtureAudit };
  }), rendererCleanupMs, `${label} disposal and audit`);
  for (const key of ["editing", "subscriptions", "unexpectedIPC", "cspViolations", "writes", "unexpectedShell"]) {
    assert.ok(Array.isArray(audit[key]), `Missing ${label} ${key} audit`);
    receipt[key].push(...audit[key].map((value) => ({ page: label, value })));
  }
}

async function verifyServiceWorkerBlock(page, receipt, label) {
  const observed = await page.evaluate(async () => {
    const container = navigator.serviceWorker;
    if (!container) return { registration: "unavailable", registered: 0, controlled: false };
    // An existing fixture asset ensures a missing probe URL cannot pass this check.
    const registration = await container.register("/entry.js").then(
      () => "allowed",
      (error) => error.name,
    );
    return {
      registration, registered: (await container.getRegistrations()).length, controlled: Boolean(container.controller),
    };
  });
  receipt.serviceWorkers[label] = observed;
  assert.ok(["SecurityError", "unavailable"].includes(observed.registration), `${label} must deny worker registration`);
  assert.equal(observed.registered, 0, `${label} must have no registered workers`);
  assert.equal(observed.controlled, false, `${label} must have no controlling worker`);
}

function verifyIPC(receipt) {
  const expectedEditing = ["db-18", "db-19"].flatMap((id) =>
    [true, false].map((active) => ({ page: "primary", value: { id, active } })));
  assert.deepEqual(receipt.editing, expectedEditing, "Expected exactly two same-ID editing on/off pairs");
  assert.deepEqual(receipt.subscriptions, [
    { page: "background-control", value: "items-changed" },
    { page: "primary", value: "items-changed" },
  ], "Expected exactly one items-changed subscription per page");
  assert.deepEqual(receipt.unexpectedIPC, [], "Unexpected Tauri IPC");
}

async function runCases(page, context, origin, receipt, signal) {
  const primary = '[data-test="primary"]';
  const row2 = `${primary} tr[data-row-id="r2"]`;
  const scroller = page.locator(`${primary} .db-scroll`);
  const snap = () => page.evaluate(() => window.fixture.snapshot());
  const settle = () => page.evaluate(() => window.settle());
  const remember = () => page.evaluate(() => { window.previousFocus = document.activeElement; });
  const scrollToEnd = () => scroller.evaluate((element) => { element.scrollTop = 1e6; });
  const cell = page.locator(`${row2} [role="gridcell"]`).first();

  async function focusRow2() {
    await cell.click({ position: inputPosition });
    await page.waitForFunction((selector) => document.activeElement?.closest(selector), row2);
    await remember();
  }

  async function mutate(kind) {
    await page.evaluate((action) => window.fixture.mutate(action), kind);
    await settle();
  }

  function common(snapshot) {
    const { probe, writes, unexpected } = snapshot.stats;
    assert.ok(probe.attach >= 2 && probe.detach >= 1, "Actual development StrictMode ref lifecycle must be observed");
    assert.equal(writes.length, 0, "Focus repair must not submit a cell/title write");
    assert.equal(unexpected.length, 0, "Unexpected shell operation");
  }

  for (const c of CASES) {
    signal.throwIfAborted();
    const started = Date.now();
    let before = null;
    let after = null;
    let removed = null;
    let focus;
    let failure;
    try {
      await page.evaluate(async (cfg) => {
        document.getElementById("external-dialog").style.display = "none";
        await window.setup(cfg);
      }, c.cfg);
      if (c.kind === "duplicate") {
        await scrollToEnd();
        await settle();
        await page.locator(`${row2} [role="gridcell"]`).last().click({ position: inputPosition });
        await remember();
        before = await snap();
        await mutate("collapse");
        after = await snap();
        assert.equal(after.previousConnected, false);
        assert.equal(after.wrapper, true);
        assert.equal(after.modelHasRow2, true);
        assert.deepEqual(after.collapsed, ["g2"]);
      } else {
        await focusRow2();
        before = await snap();
        if (c.kind === "redo") {
          await page.keyboard.press("Control+z");
          await page.waitForFunction((selector) => !document.querySelector(selector), row2);
          await settle();
          removed = await snap();
          // Both versions receive the natural redo key without supplying focus after removal.
          await page.keyboard.press(c.key);
          await settle();
          after = await snap();
          assert.equal(removed.wrapper, true, "Removed owned row must naturally leave focus on its same table wrapper");
          assert.deepEqual(removed.depth, { undo: 0, redo: 1 });
          assert.equal(removed.stats.undo, 1);
          common(removed);
          assert.equal(after.modelHasRow2, true);
          assert.equal(after.stats.redo, 1);
          assert.equal(after.stats.rootKeys, 2);
          assert.deepEqual(after.depth, { undo: 1, redo: 0 });
        } else if (c.kind === "repair") {
          await mutate(c.action);
          after = await snap();
          assert.equal(after.previousConnected, false);
          assert.equal(after.wrapper, true);
        } else if (c.kind === "external") {
          if (c.target === "dialog-input") {
            await page.locator("#external-dialog").evaluate((element) => { element.style.display = "block"; });
          }
          const target = c.target === "other-table" ?
            page.locator('[data-test="other"] tr[data-row-id="r1"] [role="gridcell"]').first() :
            page.locator(`#${c.target}`);
          if (c.target === "other-table") await target.click({ position: inputPosition });
          else await target.focus();
          await remember();
          await mutate("remove");
          after = await snap();
          assert.equal(after.previousConnected, true);
          assert.equal(after.wrapper, false);
          assert.equal(await target.evaluate((element) => element === document.activeElement), true);
        } else if (c.kind === "editor") {
          await page.locator(`${primary} tr[data-row-id="r1"] [role="gridcell"]`).first().click({ position: inputPosition });
          await page.keyboard.press("F2");
          const input = page.locator(`${primary} tr[data-row-id="r1"] input`).first();
          await input.waitFor();
          await page.waitForFunction(() => document.activeElement?.closest('[data-row-id="r1"]')?.querySelector("input") === document.activeElement);
          await remember();
          await mutate("remove");
          after = await snap();
          assert.equal(await input.evaluate((element) => element === document.activeElement), true);
          assert.equal(after.previousConnected, true);
          assert.equal(after.wrapper, false);
        } else if (c.kind === "input-remove") {
          await page.keyboard.press("F2");
          await page.waitForFunction(() => document.activeElement?.closest('[data-row-id="r2"]')?.querySelector("input") === document.activeElement);
          await remember();
          await mutate("remove");
          after = await snap();
          assert.equal(after.previousConnected, false);
          assert.equal(after.wrapper, true);
        } else if (c.kind === "autofocus") {
          await mutate("autofocus-remove");
          after = await snap();
          assert.equal(after.focusId, "commit-focus-target");
          assert.equal(after.wrapper, false);
        } else if (c.kind === "background") {
          focus = { primaryBefore: await page.evaluate(() => document.hasFocus()) };
          const otherPage = await context.newPage();
          try {
            await bypassServiceWorkers(context, otherPage);
            await otherPage.goto(`${origin}/`, { waitUntil: "load", timeout: 15000 });
            await verifyServiceWorkerBlock(otherPage, receipt, "background-control");
            await otherPage.bringToFront();
            focus.primary = await page.evaluate(() => document.hasFocus());
            focus.other = await otherPage.evaluate(() => document.hasFocus());
            assert.equal(focus.primary, false, "Background precondition must be observed");
            assert.equal(focus.primaryBefore, true, "The primary tab started in the foreground");
            assert.equal(focus.other, true, "The other tab really took focus");
            // A genuinely hidden tab pauses requestAnimationFrame. Commit the
            // removal there, then wait two frames on the foreground tab.
            await page.evaluate(() => window.fixture.mutate("remove"));
            await otherPage.evaluate(() => window.settle());
            focus.primaryAfter = await page.evaluate(() => document.hasFocus());
            assert.equal(focus.primaryAfter, false, "Removal must complete while the primary tab is still in the background");
            after = await snap();
            assert.equal(after.modelHasRow2, false);
            assert.equal(after.rows.includes("r2"), false);
            assert.equal(after.previousConnected, false);
            assert.equal(after.wrapper, false);
          } finally {
            if (!signal.aborted) {
              try {
                if (otherPage.url() === `${origin}/`) await stopFixturePage(otherPage, receipt, "background-control");
              } finally {
                await within(otherPage.close(), rendererCleanupMs, "Background tab close");
                await within(page.bringToFront(), rendererCleanupMs, "Restore foreground tab");
              }
            }
          }
        } else if (c.kind === "scope") {
          await mutate(c.action);
          after = await snap();
          assert.equal(after.wrapper, false);
          if (c.expected) assert.equal(after.focusId, c.expected);
        } else if (c.kind === "keep") {
          await mutate(c.action);
          after = await snap();
          assert.equal(after.previousConnected, true);
          assert.equal(await page.evaluate(() => document.activeElement === window.previousFocus), true);
        } else if (c.kind === "no-repair") {
          await mutate(c.action);
          after = await snap();
          assert.equal(after.modelHasRow2, true);
          assert.equal(after.wrapper, false, "A still-present item key must not claim removal ownership");
        } else if (c.kind === "scroll") {
          await scrollToEnd();
          await settle();
          after = await snap();
          assert.equal(after.previousConnected, true);
          assert.equal(await page.evaluate(() => document.activeElement === window.previousFocus), true);
          assert.ok(after.rows.length < 80, "Actual virtualization remains bounded");
        } else if (c.kind === "modal") {
          await mutate("modal-remove");
          after = await snap();
          assert.equal(after.focusTag, "BODY");
          assert.equal(after.wrapper, false);
          await mutate("dismiss-modal");
          const later = await snap();
          assert.equal(later.focusTag, "BODY");
          assert.equal(later.wrapper, false, "Discarded pending focus must not replay");
          after = later;
        } else throw new Error("Undeclared case kind");
      }
      common(after);
      assert.equal(await page.locator("#outer-note-marker").textContent(), "Original note marker");
    } catch (error) {
      signal.throwIfAborted();
      failure = {
        error: errorText(error), errorCode: error.code, actual: error.actual, expected: error.expected,
      };
      after ??= await snap().catch(() => null);
    }
    receipt.cases.push({ name: c.name, passed: !failure, wallMs: Date.now() - started, before, removed, after, focus, ...failure });
  }
}

function verifyCases(receipt, expectedFailures) {
  assert.deepEqual(receipt.cases.map(({ name }) => name), CASES.map(({ name }) => name));
  const failed = receipt.cases.filter(({ passed }) => !passed);
  assert.deepEqual(failed.map(({ name }) => name), expectedFailures);
  for (const item of failed) {
    assert.equal(item.errorCode, "ERR_ASSERTION", item.name);
    assert.equal(item.actual, false, item.name);
    assert.equal(item.expected, true, item.name);
    if (item.name.startsWith("owned-remove-natural-redo/")) {
      assert.equal(item.removed.wrapper, false, item.name);
      assert.equal(item.after.modelHasRow2, false, item.name);
      assert.equal(item.after.stats.redo, 0, item.name);
    } else {
      assert.equal(item.after.previousConnected, false, item.name);
      assert.equal(item.after.wrapper, false, item.name);
    }
  }
  const background = receipt.cases[21];
  assert.equal(background.name, backgroundCase);
  assert.equal(background.passed, true);
  assert.deepEqual(background.focus, { primaryBefore: true, primary: false, other: true, primaryAfter: false });
}

function sourceHashes() {
  return Object.fromEntries(pinnedPaths.map((file) => [file, sha256(readFileSync(path.resolve(repo, file)))]));
}

function verifySources(receipt, before) {
  receipt.sourceStability = { before, after: sourceHashes() };
  receipt.sourceStability.passed = pinnedPaths.every((file) => before[file] === receipt.sourceStability.after[file]);
  assert.deepEqual(receipt.sourceStability.after, before, "Pinned source file bytes changed during the run");
}

function removeScratchDirectory(directory) {
  if (!existsSync(directory)) return;
  assert.equal(realpathSync(directory), directory, "Refusing to remove a redirected scratch directory");
  rmSync(directory, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
  assert.equal(existsSync(directory), false, `Scratch directory still exists: ${directory}`);
}

function saveReceipt(out, name, receipt) {
  const receiptPath = path.join(out, `${name}.json`);
  writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`, { flag: "wx" });
  return receiptPath;
}

async function runSource({ source, mode, expectedFailures }, { chromium, chrome, playwright, scratch, out, before, signal }) {
  const profile = path.join(scratch, `${mode}-profile`);
  const receipt = {
    mode, preimageCommit, sourcePath: tablePath, sourceSha256: sha256(source),
    harnessSha256: sha256(readFileSync(fileURLToPath(import.meta.url))),
    startedUtc: new Date().toISOString(), headless: false, chrome, playwright, profile,
    expectedFailures, cases: [], editing: [], subscriptions: [], unexpectedIPC: [], unexpectedNetwork: [],
    cspViolations: [], writes: [], unexpectedShell: [], pageErrors: [], consoleErrors: [], serviceWorkers: {},
    serviceWorkerTargets: [],
    cleanupErrors: [], passed: false,
  };
  let server;
  let chromeProcess;
  let browser;
  let page;
  let fixturePort;
  const cleanup = async (work) => {
    try {
      await work();
    } catch (error) {
      receipt.cleanupErrors.push(errorText(error));
    }
  };

  async function exerciseFixture() {
    const assets = await bundleTable(source, scratch);
    signal.throwIfAborted();
    server = fixtureServer(assets, receipt);
    await new Promise((resolve, reject) => {
      server.once("error", reject);
      server.listen({ port: 0, host: "127.0.0.1", signal }, () => {
        fixturePort = server.address().port;
        resolve();
      });
    });
    signal.throwIfAborted();
    const origin = `http://127.0.0.1:${fixturePort}`;
    receipt.loopbackOrigin = origin;
    chromeProcess = startHeadedChrome(chrome, profile);
    const endpoint = await chromeProcess.endpoint;
    signal.throwIfAborted();
    browser = await chromium.connectOverCDP(endpoint, { noDefaults: true, timeout: 15000 });
    signal.throwIfAborted();
    const [context] = browser.contexts();
    const cdp = await browser.newBrowserCDPSession();
    receipt.chromeVersion = await cdp.send("Browser.getVersion");
    await cdp.detach();
    const observePage = (current) => {
      current.on("pageerror", (error) => receipt.pageErrors.push(errorText(error)));
      current.on("console", (message) => {
        if (message.type() === "error") receipt.consoleErrors.push(message.text());
      });
    };
    for (const current of context.pages()) observePage(current);
    context.on("page", observePage);
    const observeWorker = (worker) => receipt.serviceWorkerTargets.push(worker.url());
    for (const worker of context.serviceWorkers()) observeWorker(worker);
    context.on("serviceworker", observeWorker);
    await context.route("**/*", async (route) => {
      const request = route.request();
      const url = new URL(request.url());
      if (request.method() === "GET" && url.origin === origin && !url.search && assets.has(url.pathname)) {
        return route.continue();
      }
      observeRequest(receipt, { method: request.method(), url: request.url() });
      await route.abort();
    });
    await context.routeWebSocket("**/*", (socket) => {
      observeRequest(receipt, { method: "WebSocket", url: socket.url() });
      return socket.close();
    });
    await context.addInitScript(installTauriShim);
    await context.addInitScript(blockServiceWorkerRegistration);
    [page] = context.pages();
    page.setDefaultTimeout(2500);
    await bypassServiceWorkers(context, page);
    await page.bringToFront();
    await page.goto(`${origin}/`, { waitUntil: "load", timeout: 15000 });
    await page.waitForFunction(() => window.fixtureBooted);
    await verifyServiceWorkerBlock(page, receipt, "primary");
    await runCases(page, context, origin, receipt, signal);
    verifyCases(receipt, expectedFailures);
  }

  let onTimeout;
  try {
    signal.throwIfAborted();
    const deadline = new Promise((_, reject) => {
      onTimeout = () => reject(signal.reason);
      signal.addEventListener("abort", onTimeout, { once: true });
    });
    await Promise.race([exerciseFixture(), deadline]);
  } catch (error) {
    receipt.failure = errorText(error);
  } finally {
    if (onTimeout) signal.removeEventListener("abort", onTimeout);
    await cleanup(async () => {
      if (page?.url() === `${receipt.loopbackOrigin}/`) await stopFixturePage(page, receipt, "primary");
    });
    await cleanup(async () => {
      if (chromeProcess) {
        await stopChrome(chromeProcess, browser, signal.aborted, receipt);
        receipt.chromeClosed = true;
      }
    });
    await cleanup(async () => {
      if (server?.listening) {
        await within(new Promise((resolve, reject) => {
          server.close((error) => error ? reject(error) : resolve());
          server.closeAllConnections();
        }), rendererCleanupMs, "Fixture server close");
      }
      receipt.serverClosed = !server?.listening;
    });
    await cleanup(() => {
      receipt.cdpPort = chromeProcess?.port;
      const portFile = path.join(profile, "DevToolsActivePort");
      if (!receipt.cdpPort && existsSync(portFile)) {
        const port = Number(readFileSync(portFile, "utf8").split(/\r?\n/, 1)[0]);
        assert.ok(Number.isInteger(port) && port > 0 && port <= 65535, "Invalid Chrome port file");
        receipt.cdpPort = port;
      }
    });
    receipt.portsClosed = {};
    await Promise.all(Object.entries({ fixture: fixturePort, cdp: receipt.cdpPort }).map(([name, port]) =>
      cleanup(async () => {
        if (port) {
          await assertPortClosed(port);
          receipt.portsClosed[name] = true;
        }
      })));
    await cleanup(() => {
      removeScratchDirectory(profile);
      receipt.profileRemoved = true;
    });
  }
  try {
    verifySources(receipt, before);
    verifyIPC(receipt);
    const webWorkers = receipt.serviceWorkerTargets.filter((url) => !/^chrome(-extension)?:\/\//.test(url));
    assert.deepEqual(webWorkers, [], "No web service workers may start");
    signal.throwIfAborted();
  } catch (error) {
    receipt.failure ??= errorText(error);
  }
  const zeroCounters = ["writes", "unexpectedShell", "unexpectedIPC", "unexpectedNetwork",
    "cspViolations", "pageErrors", "consoleErrors", "cleanupErrors"];
  receipt.passed = !receipt.failure && receipt.serverClosed && receipt.chromeClosed && receipt.profileRemoved &&
    receipt.portsClosed.fixture && receipt.portsClosed.cdp &&
    zeroCounters.every((key) => receipt[key].length === 0);
  receipt.finishedUtc = new Date().toISOString();
  receipt.passCount = receipt.cases.filter(({ passed }) => passed).length;
  receipt.failCount = receipt.cases.length - receipt.passCount;
  const receiptPath = saveReceipt(out, mode, receipt);
  console.log(`${mode}: ${receipt.passCount} passed / ${receipt.failCount} failed (${receipt.cases.length}/33 ran)`,
    `${receipt.passed ? "expected result" : "unexpected result"}; ${receiptPath}`);
  if (!receipt.passed) {
    for (const key of zeroCounters) {
      if (receipt[key].length) console.error(`${mode} ${key}:`, receipt[key]);
    }
    if (receipt.failure) console.error(receipt.failure);
  }
  return receipt.passed;
}

async function main() {
  assert.equal(CASES.length, 33);
  assert.equal(new Set(CASES.map(({ name }) => name)).size, 33);
  const before = sourceHashes();
  const candidate = readFileSync(path.resolve(repo, tablePath));
  assert.equal(sha256(candidate), before[tablePath]);
  const preimage = execFileSync("git", ["show", `${preimageCommit}:${tablePath}`], { cwd: repo, timeout: 10000 });
  const playwright = resolvePlaywright();
  const { chromium } = require(playwright);
  const chromePath = process.env.CHROME_PATH || chromium?.executablePath?.();
  assert.ok(chromePath, "Chromium unavailable. Set CHROME_PATH.");
  const chrome = path.resolve(chromePath);
  assert.ok(existsSync(chrome), `Chrome unavailable at ${chrome}. Set CHROME_PATH.`);
  const outputRoot = path.resolve(process.env.Q1_RECEIPT_DIR || path.join(os.tmpdir(), "notaya-q1-focus-receipts"));
  const tempRoot = path.resolve(process.env.Q1_TMP_DIR || os.tmpdir());
  mkdirSync(outputRoot, { recursive: true });
  const out = mkdtempSync(path.join(outputRoot, "run-"));
  mkdirSync(tempRoot, { recursive: true });
  const scratch = mkdtempSync(path.join(realpathSync(tempRoot), "notaya-q1-focus-"));
  const controller = new AbortController();
  const { signal } = controller;
  const timer = setTimeout(() => controller.abort(new Error(`Whole-run wall-time limit exceeded (${wallTimeMs} ms)`)), wallTimeMs);
  const receipt = { scratch, wallTimeMs, results: {} };
  try {
    const options = { chromium, chrome, playwright, scratch, out, before, signal };
    receipt.results.preimage = await runSource({ source: preimage, mode: "preimage", expectedFailures: expectedPreimageFailures }, options);
    signal.throwIfAborted();
    receipt.results.candidate = await runSource({ source: candidate, mode: "candidate", expectedFailures: [] }, options);
    signal.throwIfAborted();
  } catch (error) {
    receipt.failure = errorText(error);
  } finally {
    clearTimeout(timer);
    receipt.timedOut = signal.aborted;
    try {
      removeScratchDirectory(scratch);
      receipt.scratchRemoved = true;
    } catch (error) {
      receipt.cleanupError = errorText(error);
    }
    try {
      verifySources(receipt, before);
    } catch (error) {
      receipt.sourceStabilityError = errorText(error);
    }
    receipt.passed = !receipt.failure && receipt.scratchRemoved && receipt.sourceStability?.passed === true &&
      receipt.results.preimage === true && receipt.results.candidate === true;
    console.log(`Whole run: ${receipt.passed ? "passed" : "failed"}; ${saveReceipt(out, "run", receipt)}`);
  }
  if (!receipt.passed) process.exitCode = 1;
  return receipt.timedOut;
}

main().then((timedOut) => {
  if (timedOut) process.exit(1);
}).catch((error) => {
  console.error(errorText(error));
  process.exitCode = 1;
});
