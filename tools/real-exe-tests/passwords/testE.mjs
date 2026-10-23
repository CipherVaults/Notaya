// testE.mjs - a changed passwords file is backed up when the window is closed with the vault unlocked (E).
//          KeePassXC is started (locked window, no dialog), the vault is locked and unlocked again, the file is
//          changed, the window is closed: Cipherleaf and KeePassXC exit and a final backup holds the new content.
// Usage:   .\run.ps1 -Script testE.mjs -WithKP -KeePassXCDir <folder>     (see run.ps1 and ..\README.md)
// Needs:   Node 24; a test copy of Cipherleaf.exe started with the debug port by run.ps1 (throwaway vault in a temp
//          folder, passphrase generated per run). Never point it at a real vault.
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { connectMain, sleep, check, fake, ls, setup, summary, kpw, procs, hasProc } from "./lib.mjs";
const [port, pwFile, dir] = process.argv.slice(2);
const password = readFileSync(pwFile, "utf8").trim();
const data = join(dir, "data"), bk = join(data, "backups"), live = join(data, "passwords.kdbx");
const m = await connectMain(port); await sleep(1500);
await setup(m, password);
const V1 = fake(300); writeFileSync(live, V1);
await m.inv("keepassxc_open");
let wins = ""; for (let i = 0; i < 40; i++) { wins = kpw("list", dir); if (/visible=True/.test(wins)) break; await sleep(1000); }
check("E KeePassXC started with its locked window (no dialog)", /visible=True/.test(wins), wins.split(String.fromCharCode(10)).filter((x) => /visible=True/.test(x)).join(" ; "));
await sleep(2000);
await m.inv("lock_vault"); await sleep(2500);                       // last lock: backup of V1
const pwb = () => ls(bk).filter((n) => /^passwords-\d+\.kdbx$/.test(n));
check("E last lock made a copy of V1", pwb().length === 1 && readFileSync(join(bk, pwb()[0])).equals(V1), String(pwb().length));
const m2 = await connectMain(port, 20);
await m2.inv("unlock_vault", { password, keyfile: null }); await sleep(1500);
const V2 = fake(300); writeFileSync(live, V2);                       // changed after the last lock, vault unlocked
console.log("KeePassXC still running before close:", hasProc(dir, "KeePassXC"));
const t0 = Date.now();
m2.ev(`window.__TAURI_INTERNALS__.invoke('plugin:window|close', { label: 'main' }).catch(()=>{})`).catch(() => {});
let ex = -1; for (let i = 0; i < 40; i++) { if (!hasProc(dir, "Cipherleaf")) { ex = Date.now() - t0; break; } await sleep(500); }
check("E Cipherleaf exited after window close", ex >= 0, `after ${ex} ms`);
check("E KeePassXC closed (no process left)", !hasProc(dir, "KeePassXC"), procs(dir).join(","));
const copies = pwb();
const hit = copies.filter((n) => readFileSync(join(bk, n)).equals(V2));
check("E data\\backups has a copy equal to the changed file (final backup after close)", hit.length >= 1, `copies=${copies.length} matching=${hit.length}`);
summary(); process.exit(0);
