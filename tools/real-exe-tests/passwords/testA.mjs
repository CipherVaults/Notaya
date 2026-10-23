// testA.mjs - passwords-file backup on lock (A0-A9): a copy of data\passwords.kdbx is made at each lock when it
//          changed, only the newest 10 are kept, other files are left alone, bad files are skipped.
// Usage:   .\run.ps1 -Script testA.mjs -WithKP -KeePassXCDir <folder>     (see run.ps1 and ..\README.md)
// Needs:   Node 24; a test copy of Cipherleaf.exe started with the debug port by run.ps1 (throwaway vault in a temp
//          folder, passphrase generated per run). Never point it at a real vault.
import { readFileSync, writeFileSync, rmSync } from "node:fs";
import { join } from "node:path";
import { connectMain, sleep, check, fake, ls, setup, summary } from "./lib.mjs";
const [port, pwFile, dir] = process.argv.slice(2);
const password = readFileSync(pwFile, "utf8").trim();
const m = await connectMain(port); await sleep(1500);
const data = join(dir, "data"), bk = join(data, "backups"), live = join(data, "passwords.kdbx");
await setup(m, password);
const pwb = () => ls(bk).filter((n) => /^passwords-\d+\.kdbx$/.test(n)).sort((a, b) => Number(a.match(/\d+/)[0]) - Number(b.match(/\d+/)[0]));
const clf = () => ls(bk).filter((n) => n.startsWith("vault-"));
const lock = async () => { const r = (await m.inv("lock_vault")) ?? { ok: "(page reloaded)" }; await sleep(1500); return r; };
const unlock = async () => { const r = (await m.inv("unlock_vault", { password, keyfile: null })) ?? { ok: "(page reloaded)" }; await sleep(800); return r; };

// A0: no passwords file yet -> nothing
let r = await lock();
check("A0 lock with no passwords file succeeds, no passwords copy", !r.err && pwb().length === 0, JSON.stringify(r));
console.log("unlock", JSON.stringify(await unlock()));

// A1
let c1 = fake(); writeFileSync(live, c1);
r = await lock();
let b = pwb();
check("A1 first lock -> exactly 1 copy equal to live", !r.err && b.length === 1 && readFileSync(join(bk, b[0])).equals(c1), `${JSON.stringify(b)}`);
const clf0 = clf().slice();
// A2 unchanged
await unlock(); r = await lock();
check("A2 lock again unchanged -> still 1", pwb().length === 1, JSON.stringify(pwb()));
// A3 changes up to 13 total versions
const versions = [c1]; const names = [pwb()[0]];
for (let i = 2; i <= 13; i++) {
  await unlock();
  const c = fake(); writeFileSync(live, c); versions.push(c);
  await lock();
  const nb = pwb(); console.log(`  iter ${i}: count=${nb.length} first=${nb[0]?.slice(10,23)} last=${nb[nb.length-1]?.slice(10,23)} lsAfter+1s=${(await sleep(1000), pwb().length)}`);
  const added = nb.filter((n) => !names.includes(n) && !nb.slice(0, 0).includes(n));
  // newest = highest number
  const newest = nb[nb.length - 1];
  if (!names.includes(newest)) names.push(newest);
  if (i === 2) check("A3 changed file + lock -> 2 copies", nb.length === 2 && readFileSync(join(bk, newest)).equals(c), JSON.stringify(nb));
  if (i === 10) check("A3b 10 changes -> 10 copies", nb.length === 10, `count=${nb.length}`);
  if (i === 11) check("A3c 11th change -> still 10 (oldest dropped)", nb.length === 10 && !nb.includes(names[0]), `count=${nb.length}`);
}
const final = pwb();
check("A4 after 13 versions exactly 10 remain", final.length === 10, `count=${final.length}`);
const expectNames = names.slice(3);
const sameNewest = JSON.stringify(final) === JSON.stringify(expectNames);
const contents = final.map((n) => readFileSync(join(bk, n)));
const expectContents = versions.slice(3);
check("A4b remaining are the newest 10 (versions 4..13, contents match in order)", contents.length === 10 && contents.every((c, i) => c.equals(expectContents[i])) && sameNewest, `names=${final.length}`);
check("A5 vault-*.clf backups (existing after A1) all still present, none deleted by passwords rotation", clf0.length >= 1 && clf0.every((n) => clf().includes(n)), `before=${clf0.length} after=${clf().length}`);

// A6 untouched names
writeFileSync(join(bk, "passwords-undo-1.kdbx"), "undo-keep"); writeFileSync(join(data, "other.kdbx"), Buffer.concat([Buffer.from([3,0xD9,0xA2,0x9A,0x67,0xFB,0x4B,0xB5]), Buffer.from("other")]));
writeFileSync(join(bk, "other.kdbx"), "bk-other");
const before = final.slice();
await unlock(); writeFileSync(live, fake()); await lock();
check("A6 rotation leaves passwords-undo-1.kdbx, backups\\other.kdbx, data\\other.kdbx untouched",
  readFileSync(join(bk, "passwords-undo-1.kdbx"), "utf8") === "undo-keep" && readFileSync(join(bk, "other.kdbx"), "utf8") === "bk-other" && ls(data).includes("other.kdbx") && pwb().length === 10 && !pwb().includes(before[0]));

// A7 invalid signature
let cnt = pwb().slice();
await unlock(); writeFileSync(live, Buffer.concat([Buffer.from("NOTAKDBX"), Buffer.from("x".repeat(100))]));
r = await lock();
check("A7 invalid-signature live file: lock succeeds, no new copy", !r.err && JSON.stringify(pwb()) === JSON.stringify(cnt), JSON.stringify(r));
// A8 empty
await unlock(); writeFileSync(live, Buffer.alloc(0));
r = await lock();
check("A8 empty live file: lock succeeds, no new copy", !r.err && JSON.stringify(pwb()) === JSON.stringify(cnt), JSON.stringify(r));
const st = (await m.inv("vault_status")).ok;
check("A9 vault is locked after those locks", st && st.unlocked === false, JSON.stringify(st));
console.log("leftover tmp files:", ls(bk).concat(ls(data)).filter((n) => n.endsWith(".tmp")).join(",") || "none");
summary();
process.exit(0);
