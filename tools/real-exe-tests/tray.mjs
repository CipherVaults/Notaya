// tray.mjs - real-exe test of the V1.4.0 tray behaviour (close hides to the tray) and the Windows switches.
// Purpose: with "close to tray" ON, closes the main window and prints what is left: for a vault with a
//          master password it must be LOCKED, for a vault with security off it stays open and the typed text
//          must be saved. Also reads and changes the three Windows switches through the app's own commands.
// Usage:   normally started by run-tray.ps1:  node tray.mjs <port> <phase> <mode> <password-file>
//          <mode> is "secure" (master password on) or "open" (security off). Throwaway vault only.
//   phase "setup":  create the vault, a notebook and a page (secure mode turns security on first)
//   phase "close":  (secure: unlock first) type text into the page, close the main window at once
//   phase "status": print vault_status of the (hidden) main window
//   phase "check":  (secure: unlock first) print whether the typed text was saved
//   phase "prefs":  <mode> is then on|off: set all three Windows switches, print what the app reports
import { readFileSync } from "node:fs";

import graderChecks from '../checks/fail-loud.cjs';
const checks = graderChecks.createChecks('tray');
const check = checks.check;
const sockets = new Set();
const [port, phase, mode = "secure", pwFile] = process.argv.slice(2);
const selected = checks.selection(Number.isInteger(Number(port)) && Number(port) >= 1 && Number(port) <= 65535 && (phase === 'prefs' ? ['on', 'off'].includes(mode) : ['secure', 'open'].includes(mode)) ? [phase] : [], ["setup","close","status","check","prefs"]);
const password = selected.length && pwFile ? readFileSync(pwFile, 'utf8').trim() : '';
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const TYPED = "TRAYTEST-" + mode;

async function connectMain() {
  for (let i = 0; i < 60; i++) {
    try {
      const t = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).filter((x) => x.type === "page");
      for (const hit of t) {
        const ws = new WebSocket(hit.webSocketDebuggerUrl);
        sockets.add(ws);
        await new Promise((r) => (ws.onopen = r));
        let id = 0;
        const pend = new Map();
        ws.onmessage = (m) => {
          const d = JSON.parse(m.data);
          if (d.id) pend.get(d.id)?.(d);
        };
        const send = (method, params = {}) => new Promise((res) => { const i = ++id; pend.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); });
        const ev = (expression) => send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true }).then((r) => r.result?.result?.value ?? r.result?.exceptionDetails?.exception?.description);
        const inv = (cmd, args = {}) => ev(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args)}).then(v => ({ok: v ?? null}), e => ({err: String(e)}))`);
        await send("Runtime.enable");
        const label = await ev("window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label ?? ''");
        if (label === "main") return { ws, send, ev, inv };
        ws.close();
      }
    } catch {}
    await sleep(500);
  }
  throw new Error("no main window");
}

try {
if (selected.length) {
const main = await connectMain();
await sleep(1500);

if (phase === "setup") {
  await main.inv("vault_status");
  if (mode === "secure") {
    console.log("turn security on:", JSON.stringify(await main.inv("enable_security", { password, keyfile: null, removeUnprotectedCopies: false })));
    const s = (await main.inv("get_settings")).ok;
    console.log("idle lock raised:", s ? JSON.stringify((await main.inv("save_settings", { settings: { ...s, idleLockMinutes: 180 } })).ok?.idleLockMinutes) : "settings unavailable");
    await main.ev("location.reload()");
    await sleep(4000);
  }
  const nb = await main.inv("create_notebook", { name: "T", color: "#888888", icon: "N" });
  if (!nb.ok) throw new Error('create_notebook failed');
  const pg = await main.inv("create_page", { sectionId: nb.ok.sections[0].id, title: "Tray page" });
  check('page created', typeof pg.ok?.id === 'string');
} else if (phase === "close") {
  if (mode === "secure") {
    await main.ev("location.reload()");
    await sleep(2500);
    console.log("unlock:", JSON.stringify(await main.inv("unlock_vault", { password, keyfile: null })));
  }
  await main.ev("location.reload()");
  await sleep(3000);
  check('page opened', (await main.ev(`(() => { const s=[...document.querySelectorAll('aside span')].find(s=>s.textContent==='Tray page'); s?.click(); return !!s; })()`)) === true);
  await sleep(1500);
  const typed = await main.ev(`(() => { const pm=document.querySelector('.ProseMirror'); if(!pm) return 'no editor'; pm.focus(); document.execCommand('insertText', false, ${JSON.stringify(TYPED)}); return pm.textContent; })()`);
  check('text reached the editor', typeof typed === 'string' && typed.includes(TYPED));
  // close at once, inside the save delay
  main.ev(`window.__TAURI_INTERNALS__.invoke('plugin:window|close', { label: 'main' }).catch(()=>{})`).catch(() => {});
  await sleep(4000);
} else if (phase === "status") {
  const v = await main.inv("vault_status");
  check('close keeps the expected protection state', v.ok?.unlocked === (mode === 'open') && v.ok?.securityOn === (mode === 'secure'));
} else if (phase === "check") {
  if (mode === "secure") console.log("unlock:", JSON.stringify(await main.inv("unlock_vault", { password, keyfile: null })));
  const tree = (await main.inv("list_notes")).ok;
  const p = tree.pages.find((x) => x.title === "Tray page");
  const pg = (await main.inv("get_page", { id: p.id })).ok;
  check('page retains typed text', ((pg?.text ?? '') + (pg?.content ?? '')).includes(TYPED));
} else if (phase === "prefs") {
  const on = mode === "on";
  console.log("before:", JSON.stringify((await main.inv("get_shell_prefs")).ok));
  console.log("set:", JSON.stringify(await main.inv("set_shell_prefs", { prefs: { closeToTray: true, startWithWindows: on, startMinimized: on } })));
  const after = (await main.inv('get_shell_prefs')).ok;
  check('shell preferences have their requested effects', after?.closeToTray === true && after?.startWithWindows === on && after?.startMinimized === on);
}
}
} finally {
  for (const socket of sockets) socket.close();
  checks.finish();
}
