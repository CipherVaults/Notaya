# Execution Summary: EX-2026-10-23-13 — Code review, lock and data-loss fixes (A17), rebuild

## 1. Task Overview

**Task requested** (the owner, 2026-10-23; message times not recorded, so the system clock is used): first "Upload the zip in my drive" (after 23:50 UTC on 2026-10-22), then, about 08:28 UTC, "the app seems very good, any suggestions how to improve it", "re review the codes and apps" and "did you deleted notion words in its codes". After two "stop" messages and "re do ... use all skills and agents and sub agents", the owner chose: "Yes, fix the lock and data-loss problems first (unsaved edits on lock, the failed commit, lock timing, KeePassXC lock and re-show, sticky delete confirm, calculator +/-), then rebuild and commit."

**Execution mode:** Direct project, with helper agents (section 3). **Executor:** Claude (Sonnet 5.5).

**Date / time:** 2026-10-23 (UTC). The system clock read 08:28:27 when the review started and 09:05:27 after the portable ZIP was made. The owner's messages in this stretch have no recorded timestamps; the times here are the clock's, approximate.

**Authority:** the owner's messages above, in [approval-and-amendments.md](../plan/approval-and-amendments.md); recorded as [A17](../plan/A17-lock-and-data-loss-fixes.md). The last message authorizes the fixes, the rebuild and the commits. **Stopping condition:** stop with the fixes committed, the portable build and ZIP rebuilt, and the records updated. No push (there is no remote).

## 2. Context Used

`CLAUDE.md`, `docs/STATUS.md`, the amendments record, the security design, and the source (Rust backend, editor, sticky notes, KeePassXC embedding). The owner's question about "Notion" was answered from a search of the code and Git history.

## 3. Work Performed

**Answer to "did you delete the Notion words?":** no source or app file contains "Notion" now, and Git history shows it was never committed there. It was removed from the code comments in the earlier reword-and-trim pass ([A13](../plan/A13-calculator-and-wording.md)). The records still name it as history.

**Review.** Three read-only review agents (Rust backend, frontend and editor, product and UX) plus the code-review skill. Claude checked the top findings in the code itself. The security-review skill could not run: it needs a Git remote, and there is none (there is no `main` branch either). Agent reports are model output; only the findings Claude confirmed in the code were acted on.

**Fixes** (two fix agents, then a fourth independent review agent that reviewed the first set and found 11 more problems; two follow-up fix agents fixed 9 of them in a second round; 1 was deliberately not done: an automatic timed retry of a failed save). All code is in commit `dc95fd3` (18 files):

1. **Pending edits are saved before locking.** Rust sends the event `vault-locking` with a round number to every window that is ready. Each window writes its pending saves (registered in `src/lib/flush.ts`) and answers with the command `flush_done(round)`. Rust waits for each window by name, at most 2 seconds, then locks. The UI also limits its own wait to 2 seconds, so a hung save cannot block the Lock button. Used by the Lock button, Ctrl+L, the palette, idle lock, Win+L, sleep, minimize and closing the window. No waiting when the drive was removed. `lock_vault` and `quit_app` now run on a worker thread so the waiting cannot freeze the window.
2. **New lock order:** wait for saves, take and hold the session, hide the KeePassXC overlay, close sticky notes, tell the window to reset (`vault-locked`), then make the backup, wipe the session from memory, and ask KeePassXC to lock.
3. **A failed SQLite `COMMIT` is rolled back** (before, the connection could stay inside an open transaction). 2 tests added.
4. **KeePassXC** is asked to lock even if Cipherleaf did not start it. `keepassxc_place` and the embed step refuse while the vault is locked (and check again after the wait). The Passwords panel no longer shows KeePassXC if you leave the view while it starts.
5. **Failed page save** is kept as unsaved and retried by the next edit, leaving the page, or the next lock; saves run one after another; closing a sticky note stays open and shows "Couldn't save this note. Try again." if its save fails.
6. **Sticky note delete** from its own window needs a second click (red "Delete?" for 3 seconds).
7. **Calculator +/-** negates the whole expression (`-2+3` becomes `-(-2+3)` = -1; it gave 5).

**Agents used** (the project's standing instruction from the owner, see section 9): 3 review agents (Rust backend, frontend and editor, product and UX), 2 fix agents, 2 follow-up fix agents, 1 independent review agent, and 1 records agent (this file and the other records). The instruction listed a separate UX agent; whether it is the same as the product and UX review agent is not recorded. Their reports are model output; Claude checked the top findings against the code.

**Release build.** `npm run tauri build` finished about 08:56 UTC (file time 11:56 local, local = UTC+3). New exe SHA-256 `D237C5B5D9346323EA2C7C5B2B0E1585FF7BCB226F551C90233AF91CD39B4183`; the previous one (`F4F1E679…4800`) is superseded. It was copied into `E:\Project\Cipherleaf-portable\` and the README's hash line updated.

**The owner's vault in the portable folder.** That folder has a `data\` folder created by the owner's own run (305 files, 76,099,122 bytes, `vault.clf` last written 2026-10-23 02:54:14 local). It was not touched: file count, size and stamp were the same afterwards.

**ZIP.** `E:\Project\Cipherleaf-portable.zip` was remade **without** `data\`: 83.1 MB, 169 entries, 0 `data` entries, SHA-256 `A6BD40E62621760F40FFE7BA0D968D3D2D9BF556A16BACFD44B6426254E2C382`. The extracted exe equals the folder's exe and the extracted KeePassXC equals the folder's. The ZIP the owner uploaded to their Drive themselves (SHA-256 `1AD64164…D063`, exe `F4F1E679…`) still has the old problem; the owner should replace it.

**Drive upload.** Claude could not upload the ZIP: the Drive connector cannot take an 83 MB file, and Claude in Chrome was not connected. The owner uploaded it themselves ("Nothing I did it manually").

## 4. Commands / Tools Used

`cargo test --lib`, `cargo clippy --all-targets`, `npx tsc --noEmit`, `npm run build`, `node dev-mock/calc-check.cjs`, `npm audit` (earlier the same day), `npm run tauri build`, PowerShell hash and ZIP commands, and a real-exe test script (see section 5). Skills: code-review (security-review could not run). Helper agents as listed above.

## 5. Verification / Testing

All on the exact committed source `dc95fd3`, unless stated.

| Check | Outcome | Applicability and limits |
|---|---|---|
| `cargo test --lib` | pass, 39 tests (33 before the A16 step plus 6 new: 2 store tests for the rolled-back commit, 4 flush-round tests) | unit level only |
| `cargo clippy --all-targets` | only the 3 old style warnings (`notes.rs`, `stickies.rs`, `crypto.rs`) | no new warnings |
| `npx tsc --noEmit -p tsconfig.json` | exit 0 | |
| `npm run build` | passed | |
| `dev-mock/calc-check.cjs` | ALL OK, including the new negate cases | calculator engine only |
| `npm audit` | 0 vulnerabilities | run earlier on 2026-10-23, not repeated after the last source change |
| **Real-exe test** on a throwaway vault (temp folder, WebView2 debug port 9335 for the test process only): type text, click Lock within milliseconds, unlock, look for the text | **pass** for a page in the main window; **pass** for a sticky-note window typed into while Lock was clicked in the main window; **pass** for closing the main window right after typing | the exe `src-tauri\target\release\Cipherleaf.exe` (SHA-256 `D237C5B5…4183`, checked just before the test; the same file was then copied into the portable folder). The port was closed afterwards, nothing listens on it, the environment variable was not left set, the temp folder was deleted. The test script (`lockflow.mjs` and `run-lockflow.ps1`) is in the session scratchpad, not in the repository. |
| Same real-exe test on the **previous** build (commit `3a22c56`, the exe from before the fixes) | **fail**: the text was lost in both the page and the sticky-note cases | shows the problem was real and the fix changes the result |
| Portable folder and ZIP checks (hashes, entry counts, no `data`, owner's `data\` untouched) | pass | see section 3 |

**Not tested in the real exe:** idle lock, Win+L, sleep, drive removal, the KeePassXC lock and re-show paths, the `quit_app` command path, the sticky delete confirm and the calculator +/- in a real window. The new exe was not run by the owner. The security-review skill and `cargo audit` were not run. Everything is unsigned.

## 6. Problems Encountered

- The first set of fixes still had holes. The independent review agent found 11 further problems; 9 were fixed in a second round, one was fixed only in part (the lock after sleep or screen lock can still wait up to 2 seconds; only a lock because the drive was removed skips the waiting), and one (automatic timed retry of a failed save) was left out on purpose.
- Claude could not upload the ZIP to Drive (see section 3).
- The portable folder is not "fresh" any more: it holds the owner's vault. Care was taken not to touch it.

## 7. Documentation Impact

Added A17 and this file. Updated the execution index, STATUS, the amendments record (authority rows and the A17 row), the acceptance ledger (new row X14, X11 hash text), the security design (P8, P10, new P16, threat 6, known limitations), failure intelligence (new FI-04) and the session history. Counts in the older records (33 tests) describe the older source and were left as written.

## 8. Git / Version Control Impact

Code and tests: `dc95fd3` ("Lock and data-loss fixes: save pending edits before locking, safer lock order (A17)"). The records are in the commit after `dc95fd3`. No push (no remote).

## 9. Decisions Made During Execution

- **The owner's standing instruction** (message "And use as agents and sub agents and skills from now on this project"): use agents, sub-agents and skills on this project from now on. Saved to Claude's memory. It does not change `CLAUDE.md` or the owner's authority: the "ask first" rules still apply.
- Fix order and scope: the owner picked the lock and data-loss problems first. Other findings wait (section 10).
- No automatic timed retry of a failed save (deliberately left out: it would show the same error message again and again while the problem lasts). A failed save is retried by the next edit, leaving the page, or the next lock.
- If a window does not answer within 2 seconds, lock anyway, so that a hung save (for example a removed USB drive) cannot block the Lock button. The cost: that last edit is lost.

## 10. Items That Need Human / Coordinator Review

- The owner should try the new exe: type and lock at once, close the window right after typing, sticky delete, calculator +/-.
- **Replace the older ZIP** (on the Drive and anywhere else it was shared) with `E:\Project\Cipherleaf-portable.zip`.
- The security change in the lock order (A17 item 2) is unreviewed by the owner.
- Open decisions that remain: A2 (single-record rollback), A8 (exact versions in `Cargo.toml`), STATUS question 8, and evidence for A1 and A3–A7. The owner's earlier check of the Paper theme and other A10–A16 items is still pending.
- Which backlog items to do next (below).

### Findings not fixed (backlog)

Severity words are a rough guess added when these records were written (the review reports were not kept in the repository); treat them as a starting order, not a measurement. None of these was reproduced in the real app.

**Backend**

| Finding | Severity |
|---|---|
| A pasted image can be removed by the cleanup if another page is deleted within the save delay | Low-medium (plausible, low frequency) |
| Quit force-kills KeePassXC after 3 seconds, which can interrupt a save | Medium |
| Backups: the file is replaced non-atomically; partial backups count toward the limit of 10; every lock (even only moving a sticky note) makes a backup, so old states age out | Medium |
| Keyfile paths from the UI are not checked (for example network UNC paths) | Low-medium |
| `open_url` only checks the start of the link | Low-medium |
| Several unlock attempts in parallel can skip the delay and use a lot of memory | Medium |
| Settings text fields are not validated | Low |
| Creating a vault is not atomic; one damaged record blocks the whole vault | Medium |

**Frontend**

| Finding | Severity |
|---|---|
| "New page" and Ctrl+N do nothing while the sidebar is hidden | Low |
| "Restore" on a trashed page stays read-only until the page is reopened | Low |
| The sidebar re-expands and re-renders on every autosave | Low |
| Keyboard users cannot open sidebar rows, sticky cards or block handles (accessibility) | Medium |
| Several buttons fail with no message when an error happens | Low-medium |
| The JavaScript bundle is 1.6 MB and every sticky window loads all of it | Low |
| KaTeX size limits are not set; "$5 and $" can turn into an equation | Low |
| Links saved without `https://` are stored as typed | Low |

**Product ideas (from the UX review, ranked by Claude)**

1. Show the suggested passphrase in plain text with a copy button (today it fills the field with dots only).
2. Restore from a backup inside the app.
3. A "recovery and backup" card. The backups cover only `vault.clf`, **not** the KeePassXC passwords file `passwords.kdbx`.
4. Decide whether backups should include the passwords file.
5. (Done in A17: the lock and data-loss fixes.)
6. A calculator entry in the sidebar with a shortcut.
7. Ctrl+F inside a page, and export to Markdown or PDF (an export writes readable text to disk, so it needs a warning and a note in the security design).
8. A first-run welcome page and a list of shortcuts.
9. Keyboard polish.
10. Settings split into tabs.

## 11. Current Result

The lock and data-loss fixes are committed (`dc95fd3`) and checked as listed in section 5, including a real-exe test on a throwaway vault where the old build lost text and the new one did not. The portable folder and ZIP are rebuilt; the owner's vault in the portable folder was not touched. **Not accepted;** only the owner accepts results. Known weakness: a save that truly hangs is dropped after 2 seconds; lock after sleep or screen lock may wait up to 2 seconds.

## 12. Recommended Next Step

The owner tries `E:\Project\Cipherleaf-portable\Cipherleaf.exe`, replaces the older ZIP copies, and says which backlog items (section 10) to do next. Anything that changes security behavior, adds a dependency, or pushes waits for the owner.

**Do not repeat:** the review, the 2-second flush design, the build.
