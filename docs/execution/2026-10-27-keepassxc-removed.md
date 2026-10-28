# EX-2026-10-27-23: KeePassXC removed (A24)

Executor: Claude (Claude Code), in the cloud thread session of the owner's project, with one writing agent for the documentation and one independent read-only review agent (model output). Short (4-section) form.

## 1. Task

The owner wrote in the project chat at 2026-10-27 06:45:11 UTC (message timestamp), as item 1 of four: "remove KeePass entirely". The coordinator gave this item to this thread; the other three items (calculator as a pop-out, multi-select delete of sticky notes, a ribbon editor) are worked on in other threads. Recorded as A24 in [approval-and-amendments.md](../plan/approval-and-amendments.md). Branch `claude/project-thread-4cul4u`, from `main` at `b64f04d`.

## 2. What changed

- **Rust core.** Deleted `keepassxc.rs` (starting KeePassXC after the hash check, the preset config, `--lock`, polite close), `embed.rs` (the in-window overlay), `pwbackup.rs` (copies of the passwords file on lock and their restore) and `commands/keepassxc.rs`. Removed the commands `set_passwords_enabled`, `list_password_backups`, `pick_passwords_backup_file`, `restore_passwords` and the seven `keepassxc_*` commands. `Paths` no longer has `keepassxc_config` or `keepassxc_dir` (`Paths::at` takes only the root), `CIPHERLEAF_KEEPASSXC` is no longer read and `data\keepassxc\` is no longer created. `Settings.passwords_enabled` and `resolve_passwords` are gone; an old settings record that still has `passwordsEnabled` loads (the field is ignored; new unit test). `lock` no longer copies the passwords file or asks KeePassXC to lock. `quit` now locks (pending edits saved, backup made), shows the backup-failure box if the backup failed, and exits; the cancelled-quit path and `reopen_open_vault` are gone. Unused `windows` crate features (`Win32_Graphics_Gdi`, `Win32_System_Threading`, `Win32_UI_WindowsAndMessaging`) dropped; `Cargo.lock` is unchanged. The package description no longer says "password vault".
- **UI.** Deleted `KeePassXCPanel.tsx`. Removed the Passwords sidebar item, `Ctrl+2`, the command palette entry, the Settings section "Passwords with KeePassXC (optional)", the "Restore passwords file…" button and the passwords mode of `RestoreDialog`, the About row "Passwords", and the KeePassXC sentences on the lock screen, Start over and Turn off security. The licenses dialog no longer lists KeePassXC. Settings, About, "Good to know" now says that a passwords file from an earlier version (and its copies in `data\backups`) is left as it is and still opens in KeePassXC itself. `Ctrl+1` and `Ctrl+3` are unchanged. The mock backend (`dev-mock/core.ts`) dropped the removed commands.
- **Existing files are not touched.** `data\passwords.kdbx`, `data\keepassxc\`, `data\backups\passwords-*.kdbx` and `passwords-undo-*.kdbx` stay on disk. Nothing deletes them; Cipherleaf simply no longer reads or copies them.
- **Docs and packaging.** README (KeePassXC sections, keys, folder layout, VC++ runtime, troubleshooting, credits removed; new "Upgrading from 1.0 or 1.0.1" note), `THIRD-PARTY-NOTICES.md` (KeePassXC sections and its COPYING text removed; open decisions 2 to 4 marked no longer applicable), `docs/security-design.md` (new "A24: KeePassXC removed" subsection; P6, P11, P18, threat 11 and the KeePassXC boundary row marked "Removed in A24"), `packaging/Licenses/README.txt`, `CLAUDE.md` (the Rust tests row only). Deleted `packaging/Licenses/KeePassXC-LIBRARIES-SOURCE.txt`, `vendor/keepassxc-manifest.md` and `tools/real-exe-tests/passwords/` (the KeePassXC-only real-exe scripts). `optionalsecurity.mjs` lost its five Passwords-switch checks in phase p1. `.gitignore` drops the vendor block and gains `*.kdbx`. Historical records (execution, plan, handoff, audit, sessions) are not edited.

## 3. Checks

All in the cloud container, on the working tree that is committed as this change (system clock, 2026-10-27 about 06:50 to 07:05 UTC):

- `npx tsc --noEmit -p tsconfig.json`: passed. `npm run build`: passed (the notices still bundle).
- `cargo check --lib --tests --target x86_64-pc-windows-gnu`: 0 errors; the same 2 library warnings as on `main` (`Session::create`, `Header::create`/`rewrap_open` unused outside tests).
- `cargo test --lib`, cross-built for `x86_64-pc-windows-gnu` and run under Wine 9.0: **124 passed, 0 failed**. The same run on `main` (`b64f04d`) gave 167 passed; the 43 fewer are exactly the removed tests (keepassxc 5, embed 3, pwbackup 33, two in `state.rs` about the passwords pick and the quit notice text). This is the GNU target under Wine, not the owner's MSVC toolchain on Windows.
- `node --check tools/real-exe-tests/optionalsecurity.mjs`: passed (syntax only).
- Independent read-only review agent (model output): no High or Medium findings. Its Low and Nit items were applied (unused `windows` features, stale "cancelled quit" comments, `restore::list_in` made private, the package description, the backups mention in the Settings note), except removing `core:window:allow-set-focus` from the capabilities (not KeePassXC-specific, so left for the owner).

**Not run:** `cargo test --lib`, `cargo clippy` and the release build on the owner's PC (MSVC); the app itself (`npm run tauri dev` or the release exe); the real-exe scripts (`run-optionalsecurity.ps1`, restore flow, lock flow) on throwaway data; `npm audit` and `cargo audit` (no dependency was added or updated).

## 4. Open items

- The owner's review and merge of the PR. Nothing is accepted.
- A PC pass on throwaway data before a release: MSVC build and tests, release build, the optional-security, restore and lock-flow scripts, and a look at Settings (no Passwords section) and the sidebar (no Passwords item).
- Amendment number A24 may collide with numbers picked by the parallel threads; renumber at merge if needed.
- Whether `Ctrl+3` for Sticky notes should become `Ctrl+2` now that the gap exists (not changed).

## Addendum (2026-10-27, records catch-up)

Code commit `4104557` on branch `claude/project-thread-4cul4u` ([PR #20](https://github.com/AbyssWatcher2/CipherLeaf/pull/20)); sticky multi-delete (with the ribbon) was merged into this branch as `ef548eb`. The PC pass named in section 4 was done in the combined test ([EX-2026-10-27-25](2026-10-27-combined-test.md): 128/128 Rust tests, optional security 47/47, restore 13/13, lock flow as designed, no Passwords entry or switch). The owner merged PR #20 at 08:04:55 UTC (GitHub; merge commit `7f64b86`), confirmed at 08:23:08 UTC that Cipherleaf's own optional security stays, and released it in V1.1.0 (tag `v1.1.0` on `70d38a1`). The number A24 was kept; the calculator became A25. Not decided: `Ctrl+3` for Sticky notes. Still open: the owner's acceptance.

Note: this report and the calculator report both carry the short id EX-23 (this one as EX-2026-10-27-23). Ids stay stable, so neither is renumbered; the file names tell them apart.
