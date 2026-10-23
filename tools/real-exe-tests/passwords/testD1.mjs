// testD1.mjs - diagnostic stress test: the first lock right after create_vault copies the passwords file, then 90
//          changed-file locks each add exactly one copy and drop exactly one once 10 exist. Takes several minutes.
// Usage:   .\run.ps1 -Script testD1.mjs -WithKP -KeePassXCDir <folder>     (see run.ps1 and ..\README.md)
// Needs:   Node 24; a test copy of Cipherleaf.exe started with the debug port by run.ps1 (throwaway vault in a temp
//          folder, passphrase generated per run). Never point it at a real vault.
// Diagnose: (1) first lock right after create_vault; (2) 90 changed-file locks with precise bookkeeping.
import { readFileSync, writeFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { connectMain, sleep, check, fake, ls, setup, summary } from "./lib.mjs";
const [port, pwFile, dir] = process.argv.slice(2);
const password = readFileSync(pwFile, "utf8").trim();
const m = await connectMain(port); await sleep(1500);
const data = join(dir, "data"), bk = join(data, "backups"), live = join(data, "passwords.kdbx");
await setup(m, password);
const pwb = () => ls(bk).filter((n) => /^passwords-\d+\.kdbx$/.test(n));
const snap = () => pwb().sort().map((n) => n.slice(10, 23)).join(" ");
// Phase 1
writeFileSync(live, fake());
let r = await m.inv("lock_vault"); await sleep(5000);
console.log("phase1 lock result:", JSON.stringify(r), "copies after 5s:", pwb().length);
check("A-create-lock first lock right after create_vault copies the passwords file", pwb().length === 1);
await m.inv("unlock_vault", { password, keyfile: null }); await sleep(800);
// Phase 2
let bad = 0; let expectCount = pwb().length;
for (let i = 1; i <= 90; i++) {
  const pre = pwb(); const c = fake(); writeFileSync(live, c); await sleep(200);
  r = await m.inv("lock_vault"); await sleep(1500);
  const post = pwb(); const added = post.filter((n) => !pre.includes(n)); const removed = pre.filter((n) => !post.includes(n));
  const expAdd = 1, expRem = pre.length >= 10 ? 1 : 0;
  if (added.length !== expAdd || removed.length !== expRem) { bad++; console.log(`iter ${i}: pre=${pre.length} post=${post.length} added=${added} removed=${removed} lockResult=${JSON.stringify(r)}`); await sleep(3000); console.log(`   4.5s later count=${pwb().length}`); }
  await m.inv("unlock_vault", { password, keyfile: null }); await sleep(800);
}
check("A-stress 90 changed-file locks: each adds exactly 1, removes exactly 1 once at 10", bad === 0, `deviations=${bad}`);
summary(); process.exit(0);
