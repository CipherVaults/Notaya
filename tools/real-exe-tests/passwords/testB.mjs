// testB.mjs - restore of the passwords file from a backup (B0-B6): list_password_backups, restore_passwords while
//          locked (refused), 15 refused inputs (bad paths, garbage, directory, undo files...), good restores with
//          undo files, restore with no live database.
// Usage:   .\run.ps1 -Script testB.mjs -WithKP -KeePassXCDir <folder>     (see run.ps1 and ..\README.md)
// Needs:   Node 24; a test copy of Cipherleaf.exe started with the debug port by run.ps1 (throwaway vault in a temp
//          folder, passphrase generated per run). Never point it at a real vault.
import { readFileSync, writeFileSync, mkdirSync, rmSync, readdirSync, existsSync } from "node:fs";
import { createHash } from "node:crypto";
import { join } from "node:path";
import { connectMain, sleep, check, fake, ls, setup, summary } from "./lib.mjs";
const [port, pwFile, dir] = process.argv.slice(2);
const password = readFileSync(pwFile, "utf8").trim();
const m = await connectMain(port); await sleep(1500);
const data = join(dir, "data"), bk = join(data, "backups"), live = join(data, "passwords.kdbx");
await setup(m, password);
const H = (b) => createHash("sha256").update(b).digest("hex").slice(0, 12);
const tree = (d) => readdirSync(d, { withFileTypes: true }).map((e) => e.isFile() ? `${e.name}:${H(readFileSync(join(d, e.name)))}` : `${e.name}/`).sort().join("|");
const world = () => tree(data) + "##" + tree(bk);
const lock = async () => { await m.inv("lock_vault"); await sleep(1800); };
const unlock = async () => { await m.inv("unlock_vault", { password, keyfile: null }); await sleep(1000); };
const undos = () => ls(data).filter((n) => /^passwords-undo-\d+\.kdbx$/.test(n)).sort();

// three versions -> three backups
const V = [fake(), fake(), fake()];
const pwb = () => ls(bk).filter((n) => /^passwords-\d+\.kdbx$/.test(n)).sort((x, y) => Number(x.match(/\d+/)[0]) - Number(y.match(/\d+/)[0]));
const byContent = (c) => pwb().filter((n) => readFileSync(join(bk, n)).equals(c));
for (let i = 0; i < 3; i++) {
  writeFileSync(live, V[i]); await sleep(300); await lock();
  if (!byContent(V[i]).length) { console.log(`NOTE: lock ${i} made no copy (flaky, see testA5); locking once more`); await unlock(); await lock(); }
  await unlock();
}
const real = pwb();
const oldest = byContent(V[0])[0], middle = byContent(V[1])[0], newest = byContent(V[2]).slice(-1)[0];
check("B0 backups for the three versions exist", !!(oldest && middle && newest), JSON.stringify(real));
let list = (await m.inv("list_password_backups")).ok;
console.log("list:", JSON.stringify(list));
check("B1 list_password_backups -> [{name,modifiedMs,size}] newest first", Array.isArray(list) && list.length === real.length && list[0].name === real[real.length-1] && list[list.length-1].name === real[0] && list.every((x) => x.name && typeof x.modifiedMs === "number" && typeof x.size === "number") && list[0].modifiedMs >= list[1].modifiedMs);

// locked restore
await lock();
let w0 = world();
let r = await m.inv("restore_passwords", { source: oldest });
check("B2 restore while LOCKED fails with 'The vault is locked.'", !!r.err && /vault is locked/i.test(r.err), JSON.stringify(r));
check("B2b locked restore changed nothing", world() === w0);
list = (await m.inv("list_password_backups")).ok;
console.log("list while locked:", list ? list.length + " entries" : JSON.stringify(list));
check("B2c list_password_backups works while locked (informational)", Array.isArray(list) && list.length === real.length);
await unlock();

// garbage inputs in backups
writeFileSync(join(bk, "passwords-1.kdbx"), "garbage garbage garbage garbage");
writeFileSync(join(bk, "passwords-2.kdbx"), Buffer.alloc(0));
mkdirSync(join(bk, "passwords-3.kdbx"), { recursive: true });
writeFileSync(join(bk, "passwords-undo-111.kdbx"), fake());
writeFileSync(join(data, "passwords-undo-222.kdbx"), fake());
writeFileSync(join(bk, "passwords-4.txt"), fake());
writeFileSync(join(bk, "passwords-5.kdbx.kdbx"), fake());
w0 = world();
const refusals = [
  ["absolute C:\\Windows\\win.ini", "C:\\Windows\\win.ini"],
  ["absolute path of a real backup (not picked)", join(bk, oldest)],
  ["UNC double-backslash 127.0.0.1 share", "\\\\127.0.0.1\\c$\\x.kdbx"],
  ["dotdot-backslash passwords.kdbx", "..\\passwords.kdbx"],
  ["../passwords.kdbx", "../passwords.kdbx"],
  ["passwords-1.kdbx garbage content", "passwords-1.kdbx"],
  ["passwords-2.kdbx empty", "passwords-2.kdbx"],
  ["passwords-3.kdbx is a directory", "passwords-3.kdbx"],
  ["passwords-undo-111.kdbx", "passwords-undo-111.kdbx"],
  ["passwords-undo-222.kdbx (name only)", "passwords-undo-222.kdbx"],
  ["passwords-4.txt wrong extension", "passwords-4.txt"],
  ["passwords-5.kdbx.kdbx", "passwords-5.kdbx.kdbx"],
  ["missing bare name passwords-999.kdbx", "passwords-999.kdbx"],
  ["empty string", ""],
  ["live name passwords.kdbx", "passwords.kdbx"],
];
for (const [label, src] of refusals) {
  const x = await m.inv("restore_passwords", { source: src });
  check(`B3 refused: ${label}`, !!x.err, JSON.stringify(x));
}
check("B3z all refusals changed nothing (data+backups content hashes identical)", world() === w0);
check("B3y no restore-check temp left after refusals", !ls(data).some((n) => n.endsWith(".tmp")));

// good restore: oldest
const liveBefore = readFileSync(live);
check("B4 pre-state: live == newest version", liveBefore.equals(V[2]));
r = await m.inv("restore_passwords", { source: oldest });
console.log("restore #1:", JSON.stringify(r));
check("B4a restore of oldest returns {undoFile}", !!r.ok?.undoFile && /^passwords-undo-\d+\.kdbx$/.test(r.ok.undoFile), JSON.stringify(r));
check("B4b live == oldest backup content (V1)", readFileSync(live).equals(V[0]));
const u1 = r.ok?.undoFile; const u1path = join(data, u1 ?? "x");
check("B4c undo file exists in data and == previous live", existsSync(u1path) && readFileSync(u1path).equals(liveBefore));
check("B4d backup file used is unchanged", readFileSync(join(bk, oldest)).equals(V[0]));
check("B4e restore-check/temp files gone", !ls(data).concat(ls(bk)).some((n) => n.endsWith(".tmp")));
let st = (await m.inv("keepassxc_status")).ok; console.log("status:", JSON.stringify(st));
check("B4f keepassxc_status.database ends with passwords.kdbx", typeof st?.database === "string" && st.database.toLowerCase().endsWith("\\passwords.kdbx"), JSON.stringify(st));
await sleep(5);
const mid = readFileSync(live);
r = await m.inv("restore_passwords", { source: middle });
console.log("restore #2:", JSON.stringify(r));
const u2 = r.ok?.undoFile;
check("B5a second restore creates a SECOND undo file (different name)", !!u2 && u2 !== u1 && undos().length >= 3 - 1 && undos().includes(u1) && undos().includes(u2), JSON.stringify({ u1, u2, undos: undos() }));
check("B5b first undo file still exists and unchanged", existsSync(u1path) && readFileSync(u1path).equals(liveBefore));
check("B5c second undo file == live before second restore (V1)", readFileSync(join(data, u2 ?? "x")).equals(mid));
check("B5d live == middle backup (V2)", readFileSync(live).equals(V[1]));
check("B5e planted passwords-undo-222.kdbx untouched", existsSync(join(data, "passwords-undo-222.kdbx")));
st = (await m.inv("keepassxc_status")).ok;
check("B5f keepassxc_status.database still passwords.kdbx", st?.database?.toLowerCase().endsWith("\\passwords.kdbx"), JSON.stringify(st));
check("B5g vault still unlocked", (await m.inv("vault_status")).ok?.unlocked === true);

// No live database
rmSync(live);
// other undo files remain in data; make sure they are not treated as the database
st = (await m.inv("keepassxc_status")).ok;
console.log("status with no live db:", JSON.stringify(st));
const undoCountBefore = undos().length;
r = await m.inv("restore_passwords", { source: newest });
console.log("restore #3 (no live db):", JSON.stringify(r));
check("B6a restore with NO live database succeeds, undoFile null", r.ok && r.ok.undoFile === null, JSON.stringify(r));
check("B6b data\\passwords.kdbx created == newest backup (V3)", existsSync(live) && readFileSync(live).equals(V[2]));
check("B6c no extra undo file was created", undos().length === undoCountBefore);
st = (await m.inv("keepassxc_status")).ok;
check("B6d keepassxc_status.database -> passwords.kdbx", st?.database?.toLowerCase().endsWith("\\passwords.kdbx"), JSON.stringify(st));
check("B6e no temp files left", !ls(data).concat(ls(bk)).some((n) => n.endsWith(".tmp")));
summary(); process.exit(0);
