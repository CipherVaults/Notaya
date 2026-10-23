// lib.mjs - shared helpers for the passwords/Quit real-exe tests (testA, testB, testC, testD1, testE).
// Purpose: connect to the main window of a test copy of Cipherleaf.exe over the WebView2 debug port, call Tauri
//          commands (inv), print PASS / FAIL (check) and a summary, make fake .kdbx files, and find / close
//          the KeePassXC windows that the test itself started (never the owner's own KeePassXC, and never
//          force-stopping anything: closing is a polite WM_CLOSE, see kpwin.ps1).
// Usage:   imported by the test scripts, which run.ps1 starts. Not run on its own.
// Needs:   Node 24 (global fetch and WebSocket), Windows PowerShell 5.1 for kpwin.ps1 / uia.ps1.
import { readFileSync, writeFileSync, readdirSync, statSync, existsSync, mkdirSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
export async function connectMain(port, tries = 60) {
  for (let i = 0; i < tries; i++) {
    try {
      const t = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).filter((x) => x.type === "page");
      for (const hit of t) {
        const ws = new WebSocket(hit.webSocketDebuggerUrl);
        await new Promise((r, j) => { ws.onopen = r; ws.onerror = j; });
        let id = 0; const pend = new Map();
        ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.id) pend.get(d.id)?.(d); };
        const send = (method, params = {}) => new Promise((res) => { const i = ++id; pend.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); });
        const ev = (expression) => send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true }).then((r) => r.result?.result?.value ?? r.result?.exceptionDetails?.exception?.description);
        const inv = (cmd, args = {}) => ev(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args)}).then(v => ({ok: v ?? null}), e => ({err: String(e)}))`);
        await send("Runtime.enable");
        const label = await ev("window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label ?? ''");
        if (label === "main") return { inv, ev, ws, send };
        ws.close();
      }
    } catch {}
    await sleep(500);
  }
  throw new Error("no main window");
}
export const results = [];
export const check = (name, ok, extra = "") => { results.push([name, ok]); console.log(`${ok ? "PASS" : "FAIL"}  ${name} ${extra}`); };
export const SIG = Buffer.from([0x03, 0xD9, 0xA2, 0x9A, 0x67, 0xFB, 0x4B, 0xB5]);
export const fake = (n = 200) => Buffer.concat([SIG, randomBytes(n)]);
export const ls = (d) => existsSync(d) ? readdirSync(d) : [];
export function setup(m, password) { return (async () => {
  console.log("create vault:", JSON.stringify(await m.inv("create_vault", { password, keyfile: null })));
  const s = (await m.inv("get_settings")).ok;
  console.log("idle lock raised:", s ? JSON.stringify((await m.inv("save_settings", { settings: { ...s, idleLockMinutes: 180 } })).ok?.idleLockMinutes) : "settings unavailable");
})(); }
export const ps = (cmd) => execFileSync("powershell", ["-NoProfile", "-Command", cmd + "; exit 0"], { encoding: "utf8" }).trim();
export const summary = () => { const f = results.filter((r) => !r[1]); console.log(`\nSUMMARY: ${results.length - f.length}/${results.length} passed` + (f.length ? "; FAILED: " + f.map((x) => x[0]).join("; ") : "")); };
const NL = String.fromCharCode(10);
const list = (o) => (o ? o.split(NL).map((x) => x.trim()).filter(Boolean) : []);
export const kpIdsNow = () => list(ps("Get-Process KeePassXC -ErrorAction Ignore | ForEach-Object { $_.Id }"));
export const kpState = { baseline: kpIdsNow() };   // KeePassXC processes that existed before the test (e.g. the owner's own)
export const mineKp = () => kpIdsNow().filter((id) => !kpState.baseline.includes(id));
const here = (name) => fileURLToPath(new URL(name, import.meta.url));
const PSFILE = here("./kpwin.ps1");
export const kpw = (action, dir, keys) => { const ids = mineKp(); if (!ids.length) return "no KeePassXC process"; try { return execFileSync("powershell", ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", PSFILE, "-Action", action, "-Pids", ids.join(","), ...(keys ? [keys] : [])], { encoding: "utf8" }).trim(); } catch (e) { return "ERR " + e.message; } };
export const procs = (dir) => { const cl = list(ps(`Get-Process Cipherleaf -ErrorAction Ignore | Where-Object { $_.Path -and $_.Path.StartsWith('${dir}') } | ForEach-Object { 'Cipherleaf:' + $_.Id }`)); return cl.concat(mineKp().map((i) => "KeePassXC:" + i)); };
export const hasProc = (dir, name) => procs(dir).some((p) => p.startsWith(name + ":"));
export const uia = (action, name = "") => { const ids = mineKp(); if (!ids.length) return "no KeePassXC process"; try { return execFileSync("powershell", ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", here("./uia.ps1"), "-Action", action, "-Pids", ids.join(","), "-Name", name], { encoding: "utf8" }).trim(); } catch (e) { return "ERR " + e.message; } };
