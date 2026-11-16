// optionalsecurity.mjs - real-exe test of "notes first, security optional" (amendment A21).
// Purpose: one phase of the test. The runner (run-optionalsecurity.ps1) starts a COPY of Notaya.exe in
//          a temp folder, runs this script once per phase, closes the app by its window close button between
//          phases (the real quit path) and starts it again. Each phase prints PASS / FAIL lines.
// Usage:   normally started by run-optionalsecurity.ps1:  node optionalsecurity.mjs <port> <password-file> <data-dir> <phase>
//          phase is p1 .. p5 (see below). <data-dir> is the test copy's data folder (read-only here).
// Needs:   Node 24 (global fetch and WebSocket, node:sqlite) and a Notaya window started with the debug port.
//          Throwaway data only: it creates a new vault in the test folder, so never point it at a real one.
// Phases:  p1 first run opens straight to notes (security off), nothing locks, text saved
//          p2 restart opens by itself; turn security on (weak password refused, earlier open backups deleted),
//             lock and unlock, wrong password refused, turn security off (wrong password refused)
//          p3 restart opens straight to notes; turn security on, keeping earlier backups; lock and unlock
//          p4 restart shows the lock screen; unlock, turn security off
//          p5 restart opens straight to notes again
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { DatabaseSync } from "node:sqlite";
import graderChecks from '../checks/fail-loud.cjs';
const checks = graderChecks.createChecks('optionalsecurity');
const check = checks.check;
const sockets = new Set();
const [port, pwFile, dataDir, phase] = process.argv.slice(2);
const selected = checks.selection(Number.isInteger(Number(port)) && Number(port) >= 1 && Number(port) <= 65535 && Boolean(pwFile) && Boolean(dataDir) ? [phase] : [], ["p1","p2","p3","p4","p5"]);
const password = selected.length && pwFile ? readFileSync(pwFile, 'utf8').trim() : '';
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

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
        ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.id) pend.get(d.id)?.(d); };
        const send = (method, params = {}) => new Promise((res) => { const i = ++id; pend.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); });
        const ev = (expression) => send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true }).then((r) => r.result?.result?.value ?? r.result?.exceptionDetails?.exception?.description);
        const inv = (cmd, args = {}) => ev(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args)}).then(v => ({ok: v ?? null}), e => ({err: String(e)}))`);
        await send("Runtime.enable");
        const label = await ev("window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label ?? ''");
        if (label === "main") return { inv, ev };
        ws.close();
      }
    } catch {}
    await sleep(500);
  }
  throw new Error("no main window");
}

const doc = (t) => JSON.stringify({ type: "doc", content: [{ type: "paragraph", content: [{ type: "text", text: t }] }] });
const TEXT = "OPEN-MODE-NOTE-7731";
const vaultPath = `${dataDir}\\vault.nota`;
const backupsDir = `${dataDir}\\backups`;
const backups = () => (existsSync(backupsDir) ? readdirSync(backupsDir).filter((n) => /^vault-.*\.(nota|clf)$/.test(n)) : []);
// Bit 1 of the flags byte (offset 50) in the header row marks a vault with security off.
function isOpenFile(path) {
  const db = new DatabaseSync(path, { readOnly: true });
  try { return (db.prepare("SELECT data FROM header WHERE id = 1").get().data[50] & 2) === 2; } finally { db.close(); }
}
const rawHas = (path, needle) => readFileSync(path).includes(Buffer.from(needle));

try {
if (selected.length) {
const m = await connectMain();
await sleep(2500);
const status = async () => (await m.inv("vault_status")).ok;
const pageText = async () => {
  const tree = (await m.inv("list_notes")).ok;
  const p = tree?.pages?.find((x) => x.title === "Open note");
  return p ? (await m.inv("get_page", { id: p.id })).ok?.text : "(no page)";
};
const raiseIdle = async () => { const s = (await m.inv("get_settings")).ok; if (s) await m.inv("save_settings", { settings: { ...s, idleLockMinutes: 180 } }); };

if (phase === "p1") {
  let st = await status();
  check("first run: a vault exists and is already open", st?.exists === true && st?.unlocked === true, JSON.stringify(st));
  check("first run: security is off, no lock screen", st?.securityOn === false);
  check("the vault file is flagged as having security off", existsSync(vaultPath) && isOpenFile(vaultPath));
  const s = (await m.inv("get_settings")).ok;
  check("screenshot protection is off by default", s?.screenCaptureProtection === false, JSON.stringify(s?.screenCaptureProtection));
  const lockRes = await m.inv("lock_vault");
  await sleep(1500);
  st = await status();
  check("Lock does nothing while security is off", st?.unlocked === true && !lockRes.err, JSON.stringify(lockRes));
  const nb = await m.inv("create_notebook", { name: "T", color: "#888888", icon: "N" });
  const pg = await m.inv("create_page", { sectionId: nb.ok.sections[0].id, title: "Open note" });
  await m.inv("save_page", { id: pg.ok.id, title: "Open note", content: doc(TEXT), text: TEXT });
  check("text saved", (await pageText()) === TEXT);
  const chg = await m.inv("change_master", { currentPassword: "x", currentKeyfile: null, newPassword: password, newKeyfile: null });
  check("changing a master password is refused while security is off", !!chg.err, JSON.stringify(chg));
  const dis = await m.inv("disable_security", { currentPassword: password, currentKeyfile: null });
  check("turning security off while it is off is refused", !!dis.err, JSON.stringify(dis));
}

if (phase === "p2") {
  let st = await status();
  check("restart: opens by itself with no lock screen", st?.unlocked === true && st?.securityOn === false, JSON.stringify(st));
  check("restart: the text is still there", (await pageText()) === TEXT, await pageText());
  const before = backups();
  check("closing the app made a backup, and it is an open one", before.length >= 1 && before.every((n) => isOpenFile(`${backupsDir}\\${n}`)), before.join(","));
  const weak = await m.inv("enable_security", { password: "short", keyfile: null, removeUnprotectedCopies: true });
  check("a weak master password is refused", !!weak.err, JSON.stringify(weak));
  check("a refused attempt changed nothing", isOpenFile(vaultPath) && before.length === backups().length);
  const on = await m.inv("enable_security", { password, keyfile: null, removeUnprotectedCopies: true });
  console.log("enable_security:", JSON.stringify(on));
  check("security turned on; earlier open backups were deleted", !on.err && Number.isFinite(on.ok?.removedCopies) && on.ok.removedCopies >= 1);
  await raiseIdle();
  st = await status();
  check("status: security on and still unlocked", st?.securityOn === true && st?.unlocked === true);
  check("the vault file now needs a password", !isOpenFile(vaultPath));
  check("the text is not readable in the file", !rawHas(vaultPath, TEXT));
  const after = backups();
  check("a fresh protected backup exists and no open backup is left", after.length >= 1 && after.every((n) => !isOpenFile(`${backupsDir}\\${n}`)), after.join(","));
  check("the notes are intact", (await pageText()) === TEXT);
  const s = (await m.inv("get_settings")).ok;
  check("screenshot protection is still off", s?.screenCaptureProtection === false);
  await m.inv("lock_vault"); await sleep(3500);
  st = await status();
  check("Lock works now and shows the lock screen state", st?.unlocked === false && st?.securityOn === true);
  const wrong = await m.inv("unlock_vault", { password: password.split("").reverse().join("") + "x", keyfile: null });
  check("a wrong password is refused", !!wrong.err, JSON.stringify(wrong));
  const ok = await m.inv("unlock_vault", { password, keyfile: null });
  check("the right password unlocks", !ok.err, JSON.stringify(ok));
  await raiseIdle();
  check("the notes survived lock and unlock", (await pageText()) === TEXT);
  const badOff = await m.inv("disable_security", { currentPassword: "not-the-password-0", currentKeyfile: null });
  check("turning security off with a wrong password is refused", !!badOff.err && !isOpenFile(vaultPath), JSON.stringify(badOff));
  const off = await m.inv("disable_security", { currentPassword: password, currentKeyfile: null });
  check("turning security off with the right password works", !off.err && isOpenFile(vaultPath), JSON.stringify(off));
  st = await status();
  check("status: security off, unlocked", st?.securityOn === false && st?.unlocked === true);
  check("the notes survived turning security off", (await pageText()) === TEXT);
  const lockRes = await m.inv("lock_vault"); await sleep(1500);
  check("Lock does nothing again", (await status())?.unlocked === true && !lockRes.err);
}

if (phase === "p3") {
  let st = await status();
  check("restart after turning security off: opens straight to notes", st?.unlocked === true && st?.securityOn === false, JSON.stringify(st));
  check("the text is still there", (await pageText()) === TEXT);
  const before = backups();
  const on = await m.inv("enable_security", { password, keyfile: null, removeUnprotectedCopies: false });
  check("security turned on again, earlier backups kept", !on.err && on.ok?.removedCopies === 0, JSON.stringify(on));
  await raiseIdle();
  const after = backups();
  check("no backup was deleted", before.every((n) => after.includes(n)), `${before.length} -> ${after.length}`);
  // Restoring a backup made with security off onto a vault with a master password needs a confirmation.
  const openBackup = after.find((n) => isOpenFile(`${backupsDir}\\${n}`));
  check("an open backup from the earlier session is still there", !!openBackup, openBackup ?? "none");
  if (openBackup) {
    const refused = await m.inv("restore_vault", { source: openBackup, password: "", keyfile: null });
    check("restoring an open backup onto a protected vault is refused until confirmed", !!refused.err && refused.err.startsWith("SECURITY_OFF:") && !isOpenFile(vaultPath), JSON.stringify(refused));
    const st0 = await status();
    check("the refused restore changed nothing", st0?.unlocked === true && st0?.securityOn === true && (await pageText()) === TEXT);
    const allowed = await m.inv("restore_vault", { source: openBackup, password: "", keyfile: null, allowSecurityOff: true });
    check("with the confirmation the restore works", !allowed.err && !!allowed.ok?.undoFile, JSON.stringify(allowed));
    await sleep(1500);
    const st1 = await status();
    check("the restored vault is open (security off) and has the text", st1?.unlocked === true && st1?.securityOn === false && (await pageText()) === TEXT, JSON.stringify(st1));
    const again = await m.inv("enable_security", { password, keyfile: null, removeUnprotectedCopies: false });
    check("security turned on again after the restore", !again.err, JSON.stringify(again));
    await raiseIdle();
  }
  await m.inv("lock_vault"); await sleep(3500);
  const ok = await m.inv("unlock_vault", { password, keyfile: null });
  check("lock and unlock work", !ok.err && (await pageText()) === TEXT, JSON.stringify(ok));
  await raiseIdle();
}

if (phase === "p4") {
  let st = await status();
  check("restart with security on: the lock screen, not the notes", st?.unlocked === false && st?.securityOn === true, JSON.stringify(st));
  const tree = await m.inv("list_notes");
  check("notes are not readable while locked", !!tree.err, JSON.stringify(tree).slice(0, 80));
  const ok = await m.inv("unlock_vault", { password, keyfile: null });
  check("the right password unlocks", !ok.err, JSON.stringify(ok));
  await raiseIdle();
  check("the text is still there", (await pageText()) === TEXT);
  const off = await m.inv("disable_security", { currentPassword: password, currentKeyfile: null });
  check("security turned off", !off.err && isOpenFile(vaultPath), JSON.stringify(off));
}

if (phase === "p5") {
  const st = await status();
  check("final restart: opens straight to notes", st?.unlocked === true && st?.securityOn === false, JSON.stringify(st));
  check("the text is still there", (await pageText()) === TEXT);
}
}
} finally {
  for (const socket of sockets) socket.close();
  checks.finish();
}
