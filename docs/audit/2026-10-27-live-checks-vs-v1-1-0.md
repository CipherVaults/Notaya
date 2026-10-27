# Live checks of 2026-10-23 compared with the current version (V1.1.0)

**What this is:** an item-by-item comparison of every finding in [2026-10-23-live-checks.md](2026-10-23-live-checks.md) (PR #10, merged 2026-10-24 09:23:51 UTC) and of the PC thread that produced it ("Live PC audit checks") with the code in `main` `6981442`. `src/` and `src-tauri/` in `main` are identical to the V1.1.0 tag commit `70d38a1` (`git diff` is empty), so this is also the state of the released V1.1.0.

**How it was checked:** by reading the code in a cloud checkout (2026-10-27, 17:08 to 17:14 UTC and 18:56 to about 19:05 UTC, system clock) and the records. **Nothing was run on Windows**: no exe, no UI, no clippy, no axe. A row marked "not re-measured" has no code change that would fix it, but the number or behaviour was not measured again. Only the owner accepts results; this is not an acceptance.

**Owner's request:** "check the live audit" (2026-10-27 17:07:25 UTC), "compare the current verison with this thread to see if everthign got fixed" (17:11:44), "stop now at resume point" (17:13:39), "Resume" (18:56:19).

**Status words:** FIXED (a code change removes it), PARTLY FIXED, OPEN (still true in the code), NO LONGER RELEVANT (the code it was about is gone).

## Summary

| Status | Count | Rows |
|---|---|---|
| FIXED | 5 | #4 F-02 backups, #5 F-13 silent exit, #8 F-11 licence text, #9 ctutils, #15a F-09 sidebar Enter |
| PARTLY FIXED | 3 | #1 network (wording fixed, traffic not), F-03 open vault (hourly backup added, never locks by design), clippy (not re-run) |
| NO LONGER RELEVANT | 1 | #14 SE-04 (KeePassXC removed) |
| OPEN | 15 | #2 F-04, #3 F-10, #6 SE-03, #7 PK-07, #10 F-07, #12 BE-11, #13 CR-02, #15b UX-02, #15c UX-07, #15d axe, #16 Argon2 cost, #17 size limits, BE-07, BE-13, unsigned exe |
| Passed then, nothing to fix | 2 | #11 hard kills; `cargo test --lib` (last PC result 128 of 128) |

## Item by item

| Live-check item (report number) | Now | Evidence in `main` `6981442` |
|---|---|---|
| **1** WebView2 contacts Microsoft addresses at startup (Medium wording, Low risk) | **PARTLY FIXED.** The wording is fixed (V1.0.1, `63a3ffd`, A23 item 8). The traffic is OPEN. | `README.md` line 5, `README.md` line 139, Settings About and `docs/security-design.md` P5 now say Cipherleaf's own code has no network code and "does not currently stop" WebView2. No browser-flag setting exists anywhere in `src-tauri/src`. The "keep app, no network" plan was paused by the owner 2026-10-27 07:21:42 UTC. |
| **2** F-04 unknown or unreadable page content opens empty and is overwritten | **OPEN** | `parseContent` in `src/features/notes/editor/extensions.ts:156-163` returns `""` when `JSON.parse` fails; `PageEditor.tsx:126` passes it to the editor with no guard and no read-only fallback. An unknown block type is dropped by the editor schema. Deferred to V1.1 in A23 and not done in V1.1.0. |
| **3** F-10 a plain click on a link in a Trash page navigates the window (no IPC from the foreign page) | **OPEN** (Medium) | `handleClick` in `extensions.ts:144-152` handles Ctrl+click only; a plain click on a link in a read-only page falls through to the browser. No navigation guard (`on_navigation`) in `src-tauri/src`. The capability file `src-tauri/capabilities/*.json` limits the foreign page to window and event permissions, which matches the live finding that Cipherleaf commands were refused. File drop: `disable_drag_drop_handler()` is set in `ui.rs` (lines 24, 92, 129); a real OS drag was never driven. |
| **4** F-02 rotation by name, 0-byte files, silent failed backup | **FIXED** (V1.0.1, `f7bac43`, `aa3058a`, `ff28818`, `0dbe11b`; A23 item 2) | `backup.rs`: rotation by modified time with a future-date guard (`plan_rotation`), the new file is never deleted (`rotate`), 0-byte files are removed and not counted (`backup.rs:133-138`), the copy is written to a temp file, verified and renamed (`write_checked`), a failed backup sets a notice (`report_backup_failure`, `state.rs:372-379`) and shows a message box at quit. Note: a 0-byte file already in the folder can still appear in the restore list until the next backup removes it; restore refuses 0-byte files (`restore.rs:156`). |
| **5** F-13 silent exit with code 0xC0000409 when the folder is not writable | **FIXED** (V1.0.1, `c65cfd8`, `ff28818`; A23 item 9) | `lib.rs:177-219` shows a message box "Cipherleaf could not start" and exits with code 1; `portable.rs:39-41` writes and removes a probe file so an existing read-only folder is caught. PC-tested in V1.0.1 (EX-21). |
| **6** SE-03 parallel wrong passwords are all evaluated | **OPEN** | `check_rate_limit` and `record_failure` in `state.rs:219-245` are separate calls; the check happens before the slow Argon2 step and the failure is recorded after it, so N parallel calls all pass the check. Deferred to V1.1 in A23; not done. |
| **7** PK-07 build-machine paths in the exe | **OPEN** (Low, not re-measured) | No `--remap-path-prefix` and no `.cargo/config.toml`; `[profile.release]` has `strip = true` and `panic = "abort"` only (`Cargo.toml:53-58`). |
| **8** F-11 WebView2 licence text missing from the ZIP | **FIXED** (V1.0.1, PR #7 content) | `packaging/Licenses/WebView2-LICENSE.txt` exists in the repository; `THIRD-PARTY-NOTICES.md` is there. The V1.1.0 ZIP contents were not re-listed. |
| **9** DP-01 `ctutils` 0.4.3 (RUSTSEC-2026-0385) | **FIXED** (V1.0.1, A23 item 6) | `src-tauri/Cargo.lock` has `ctutils` 0.4.5. The lock files changed since the V1.0.1 audit run (`6109aee`) only by the version line (1.0.1 to 1.1.0). `npm audit --package-lock-only` run on `main` today: 0 vulnerabilities. `cargo audit` was **not** run today (not installed here); its last run was 0 vulnerabilities on the V1.0.1 lock file. |
| **10** F-07 huge equation freezes the page | **OPEN** (Low; was not confirmed at the claimed size) | `math.ts:26` calls `katex.render` with no length cap. |
| **11** Hard kills during saves, security switches and lock backups | **PASSED then.** Nothing to fix. | The storage code was changed in V1.0.1 (backups, re-key); those changes were PC-tested in EX-21. Not re-run on V1.1.0 for kills. |
| **12** BE-11 note text and typed password stay in process memory after a lock | **OPEN** (documented best effort, P10) | No change found. |
| **13** CR-02 deleting a record row from the vault file is not detected | **OPEN** (Low) | `vault/mod.rs:37-39` binds each ciphertext to the vault id, table and row id (flips and swaps are caught) but nothing checks the set of rows. |
| **14** SE-04 KeePassXC check covers the exe only | **NO LONGER RELEVANT** | KeePassXC was removed in A24 (`4104557`, PR #20, V1.1.0). The same applies to the orphan KeePassXC window after a kill and the KeePassXC "hides on lock" test that was never run. |
| **15a** F-09 Enter on a focused sidebar row did nothing | **FIXED** (V1.0.1, `f7bac43`; A23 item 5) | `Sidebar.tsx` is a keyboard-usable tree (`role="tree"`; arrows, Enter, Shift+F10, Move up/down). PC-tested in EX-21 (keyboard walk). |
| **15b** UX-02 focus falls to `<body>` after Esc closes the Ctrl+K palette | **OPEN** (not re-measured) | `CommandPalette.tsx` uses a Radix `Dialog` opened by a shortcut with no trigger element, and no focus-return code; unchanged since A24. |
| **15c** UX-07 faint text colour fails AA in every theme | **OPEN** | `--faint` values in `src/styles/app.css` (lines 33, 50, 67, 84) are unchanged since V1.0. |
| **15d** axe-core serious issues (`nested-interactive`, `aria-input-field-name`, `color-contrast`) | **OPEN** (not re-measured) | The sidebar was rebuilt as a tree in V1.0.1 and the ribbon is new in V1.1.0; axe was not run on either. |
| **16** Argon2id cost is low on a fast PC (about 135 ms per unlock) | **OPEN by design** (A1, provisional) | `crypto.rs:52` fixed 256 MiB, 3 passes, 4 lanes. |
| **17** Size limits: 20,000,000 stated, about 19,000,000 refused; 2.3 to 2.6 GB memory with huge pages | **OPEN** (Low) | `MAX_PAGE_CHARS = 20_000_000` (`commands/notes.rs:12`) versus `MAX_ITEM_BYTES = 32 MiB` (`vault/mod.rs:19`); no change. |
| **F-03** open vault and a removed drive | **PARTLY FIXED** | An open vault (security off) is now also backed up about hourly when it changed (`state.rs:383-460`, A23). It still never locks and has no drive-removal handling (A21 by design). What happens to edits after a **real** removal and re-insert was never tested. |
| **BE-07** a protected vault's lock re-creates the data folder and writes a backup at a vanished path | **OPEN** (Low) | `lock()` makes a backup for the "drive-removed" reason and `make_in` calls `create_dir_all` (`backup.rs:226`). |
| **BE-13** a second copy exits silently | **OPEN** (Low) | The single-instance plugin only focuses the first window (`lib.rs:44-50`). |
| Exe is unsigned; Control Flow Guard table empty; empty copyright and company "cipherleaf" | **OPEN** (not re-measured) | No publisher or copyright in `tauri.conf.json`. Code signing is an owner decision (open since 2026-10-23). |
| `cargo clippy --all-targets` warnings (5 in the lib, 1 in tests) | **PARTLY** (not re-run) | V1.0.1 cleaned `backup.rs`; EX-21 recorded 6 library warnings, none in the new code. Not run on V1.1.0 here. |
| `cargo test --lib` | Not run here | Last PC result: 128 of 128 on V1.1.0 (EX-2026-10-27-24, -25). |

## Never run on a real PC (still true)

Real USB removal, Win+L, real sleep and resume, SmartScreen / Smart App Control, a screen reader, High Contrast mode, IME, a real OS file drag onto the window, a real screenshot capture of a protected window, `testD1` (90 locks). The "upgrade a real V1.0.1 data folder to V1.1.0" check is also still open (STATUS question 20).

## What the owner can decide

Nothing here is urgent and nothing starts without the owner's word. If a fix release is wanted, the smallest group with the most value is: F-04 (never overwrite a page the editor could not read), SE-03 (count attempts before the slow step), F-10 (stop the window navigating on a link click), and the contrast and focus items. The WebView2 traffic belongs to the postponed no-network plan.
