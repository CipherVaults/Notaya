// testC.mjs - Quit / window close with KeePassXC running (C1, c2, c2q). Needs a KeePassXC folder.
//          mode c1  : close the window with KeePassXC open on a locked database: Cipherleaf and KeePassXC both exit.
//          mode c2  : KeePassXC has a modal dialog open: closing Cipherleaf must NOT kill KeePassXC; the vault is
//                     locked, a notice is shown once, and after closing KeePassXC politely a second close exits.
//          mode c2q : same as c2 but using the quit_app command.
// Usage:   .\run.ps1 -Script testC.mjs -WithKP -KeePassXCDir <folder> -Extra c1|c2|c2q     (see run.ps1 and ..\README.md)
//          KeePassXC is only ever closed politely (WM_CLOSE / answering its dialog); it is never force-stopped.
//          If it will not close, the test stops, leaves it running and run.ps1 keeps the temp folder.
// Needs:   Node 24; a test copy of Cipherleaf.exe started with the debug port by run.ps1 (throwaway vault in a temp
//          folder, passphrase generated per run). Never point it at a real vault.
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { execFileSync } from "node:child_process";
import { connectMain, sleep, check, fake, ls, setup, summary, kpw, uia, procs, hasProc } from "./lib.mjs";
const [port, pwFile, dir, clPid, mode] = process.argv.slice(2);   // mode: c1 | c2 | c2q
const password = readFileSync(pwFile, "utf8").trim();
const NL = String.fromCharCode(10);
const data = join(dir, "data"), live = join(data, "passwords.kdbx");
const m = await connectMain(port); await sleep(1500);
await setup(m, password);
console.log("mode:", mode, "processes:", procs(dir).join(","));
const fakeDb = mode !== "c1";
if (fakeDb) writeFileSync(live, fake(300));
// Best-effort capture of the app-notice event across the reload (sessionStorage survives reload)
const LISTEN_SRC = `(() => {
  const KEY = '__TAURI_INTERNALS__'; let val = window[KEY];
  const log = (o) => { try { const l = JSON.parse(sessionStorage.getItem('tnLog') || '[]'); l.push(o); sessionStorage.setItem('tnLog', JSON.stringify(l)); } catch (e) {} };
  const wrap = (v) => { if (v && !v.__w) { v.__w = true; const o = v.invoke; v.invoke = function (cmd, ...r) { if (cmd === 'take_notice' && !window.__allow) { log({ blockedPageCall: Date.now() }); return Promise.resolve(null); } return o.apply(this, [cmd, ...r]); }; } return v; };
  const hook = () => { try { if (window[KEY] && window[KEY].invoke && !window[KEY].__w) { wrap(window[KEY]); log({ hooked: Date.now() }); } } catch (e) { log({ hookError: String(e) }); } };
  const t0 = Date.now(); const iv = setInterval(() => { hook(); if (window[KEY] && window[KEY].transformCallback) { clearInterval(iv); window[KEY].invoke('plugin:event|listen', { event: 'app-notice', target: { kind: 'Any' }, handler: window[KEY].transformCallback((p) => { sessionStorage.setItem('noticeEvt', JSON.stringify(p)); }) }).catch(() => {}); } else if (Date.now() - t0 > 15000) clearInterval(iv); }, 30);
})();`;
await m.send("Page.enable");
await m.send("Page.addScriptToEvaluateOnNewDocument", { source: LISTEN_SRC });
await m.ev(LISTEN_SRC); console.log("app-notice listener installed (now and for every new document)");
const o = await m.inv("keepassxc_open");
console.log("keepassxc_open:", JSON.stringify(o));
let wins = "";
for (let i = 0; i < 40; i++) { wins = kpw("list", dir); if (/visible=True/.test(wins)) break; await sleep(1000); }
console.log("KeePassXC visible windows:" + NL + wins.split(NL).filter((x) => /visible=True/.test(x)).join(NL));
check(`${mode} KeePassXC started with a visible window`, /visible=True/.test(wins));
await sleep(3000);
if (mode !== "c1") { console.log("UIA: opening a modal file dialog in KeePassXC ->", uia("invoke", "Open Database")); await sleep(2500); }
console.log("KeePassXC visible windows after settle:" + NL + kpw("list", dir).split(NL).filter((x) => /visible=True/.test(x)).join(NL));

const t0 = Date.now();
if (mode === "c2q") m.ev(`window.__TAURI_INTERNALS__.invoke('quit_app').catch(()=>{})`).catch(() => {});
else m.ev(`window.__TAURI_INTERNALS__.invoke('plugin:window|close', { label: 'main' }).catch(()=>{})`).catch(() => {});
console.log("close/quit requested");

if (mode === "c1") {
  let exited = -1;
  for (let i = 0; i < 40; i++) { if (!hasProc(dir, "Cipherleaf")) { exited = Date.now() - t0; break; } await sleep(500); }
  check("C1 Cipherleaf exited after window close request", exited >= 0, `after ${exited} ms`);
  check("C1 no KeePassXC process from the temp folder remains", !hasProc(dir, "KeePassXC"), procs(dir).join(","));
  summary(); process.exit(0);
}

// c2 / c2q
await sleep(8000);
let after8 = procs(dir);
console.log(`after ${Date.now() - t0} ms processes:`, after8.join(","));
check(`${mode} Cipherleaf STILL RUNNING after 8 s`, after8.some((p) => p.startsWith("Cipherleaf:")));
check(`${mode} KeePassXC STILL RUNNING (not killed)`, after8.some((p) => p.startsWith("KeePassXC:")));
console.log("KeePassXC visible windows now:" + NL + kpw("list", dir).split(NL).filter((x) => /visible=True/.test(x)).join(NL));
await sleep(5000);
const after13 = procs(dir);
check(`${mode} both still running after ~13 s (covers a 10 s wait too)`, after13.some((p) => p.startsWith("Cipherleaf:")) && after13.some((p) => p.startsWith("KeePassXC:")), after13.join(","));

const m2 = await connectMain(port, 20);
const vs = (await m2.inv("vault_status")).ok;
check(`${mode} vault is locked`, vs?.unlocked === false, JSON.stringify(vs));
const body = await m2.ev(`document.body.innerText.slice(0, 600)`);
const evt = await m2.ev(`sessionStorage.getItem('noticeEvt')`);
console.log("app-notice event payload captured in sessionStorage:", evt);
console.log("page text contains notice:", /KeePassXC is still open/.test(String(body)));
const EXPECT = "KeePassXC is still open (it may be asking to save). Finish there, close it, then close Cipherleaf again.";
const pageLog = await m2.ev(`sessionStorage.getItem('tnLog')`); await m2.ev('window.__allow = true');
const n1 = await m2.inv("take_notice"), n2 = await m2.inv("take_notice");
console.log("take_notice calls made by the page itself (sessionStorage log):", await m2.ev(`sessionStorage.getItem('tnLog')`));
console.log("take_notice #1:", JSON.stringify(n1), "#2:", JSON.stringify(n2));
const uiShows = String(body).includes(EXPECT);
check(`${mode} notice text delivered exactly once: the UI already took it (toast shows the exact text) and my take_notice calls return null; or I got it myself`, (n1.ok === EXPECT && !uiShows) || (n1.ok === null && uiShows), `uiShows=${uiShows} mine=${JSON.stringify(n1)}`);
check(`${mode} take_notice second call returns null`, n2.ok === null, JSON.stringify(n2));
check(`${mode} app-notice event was emitted (best effort capture)`, !!evt && JSON.stringify(evt).includes("still open"), String(evt));

// polite cleanup
let gone = false;
for (let round = 0; round < 10 && !gone; round++) {
  console.log(`cleanup round ${round}: WM_CLOSE ->`, kpw("close", dir).split(NL).join("; "));
  for (let i = 0; i < 2; i++) { await sleep(1000); if (!hasProc(dir, "KeePassXC")) { gone = true; break; } }
}
if (!gone) {
  console.log("still running; visible windows:" + NL + kpw("list", dir).split(NL).filter((l) => /visible=True/.test(l)).join(NL));
  for (const k of ["{ESC}", "{ENTER}", "{ESC}", "{ENTER}"]) {
    console.log("keys", k, "->", kpwKeys(k));
    for (let i = 0; i < 6; i++) { await sleep(1000); if (!hasProc(dir, "KeePassXC")) { gone = true; break; } }
    if (gone) break;
  }
}
check(`${mode} KeePassXC closed politely`, gone, procs(dir).join(","));
if (!gone) { console.log("STOP: KeePassXC would not close politely; leaving it (not killing)."); summary(); process.exit(2); }
await sleep(1500);
// close again -> must exit
const t1 = Date.now();
const m3 = await connectMain(port, 10);
if (mode === "c2q") m3.ev(`window.__TAURI_INTERNALS__.invoke('quit_app').catch(()=>{})`).catch(() => {});
else m3.ev(`window.__TAURI_INTERNALS__.invoke('plugin:window|close', { label: 'main' }).catch(()=>{})`).catch(() => {});
let ex = -1;
for (let i = 0; i < 40; i++) { if (!hasProc(dir, "Cipherleaf")) { ex = Date.now() - t1; break; } await sleep(500); }
check(`${mode} closing again after KeePassXC is closed exits Cipherleaf`, ex >= 0, `after ${ex} ms`);
summary(); process.exit(0);

function kpwKeys(k) { return kpw("keys", dir, k); }
