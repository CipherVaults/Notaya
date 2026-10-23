# Cipherleaf: handoff checkpoint (2026-10-23)

This file freezes where the project stands so a new conversation can pick it up without guessing. It is a snapshot, not the live state: [STATUS.md](../STATUS.md) stays the owner of the live state, and this file does not copy it. For the story of how the project got here, see the sibling record [2026-10-23-project-history.md](2026-10-23-project-history.md) (written at the same time by another agent; this file only links to it).

## 1. Handoff status

| Item | Value |
|---|---|
| Status | **PREPARED** only |
| Delivered | No. Nothing was sent to any thread or person. |
| Acknowledged | No. No recipient has confirmed reading it. |
| Executed | No. No step in section 6 has been started by a recipient. |
| Accepted | No. The owner has accepted no result and has not accepted this handoff. |
| Prepared by | Claude (Claude Code, Sonnet 5.5), through a helper agent, in the owner's working session |
| Prepared at | 2026-10-23, system clock 16:38 UTC (read with `date -u` when the work began; the request said "about 16:30") |
| Stopping condition | The work is committed and pushed through `7c40246`; the owner is waiting to try the rebuilt app (A10 to A19) and choose the next task. Development is not paused by the owner, but nothing new is authorized beyond what section 3 lists. |
| Repository head | `7c402469a9c5f5ee4e1114834b2a69f092c07d17` (short `7c40246`, "README refresh for GitHub; record the push/pull confirmation", 2026-10-23 19:13:29 +0300) |

When a recipient confirms they read it, or runs a step, update this table with the evidence. Do not mark it ahead of the evidence.

## 2. Intended recipient and access limits

**Intended recipient:** the owner's "CipherLeaf" project thread in the Claude desktop app. A successor Claude Code session on the owner's PC may also use it, with the extra access noted below.

**Do not assume another conversation sees these files.** The project thread works from the GitHub copy only.

| Thing | Project thread | Successor session on the owner's PC |
|---|---|---|
| Private repository <https://github.com/AbyssWatcher2/CipherLeaf> (branches `main` and `feat/a9-keepassxc`, both at `7c40246` when this was written) | Can read | Can read, push and pull |
| Files in the repository (`docs/`, `CLAUDE.md`, `src/`, `src-tauri/`) | Can read | Can read and edit (with the permissions in section 3) |
| This file and the history file | Only after they are committed and pushed (they are not yet) | Can read them on disk now |
| Claude's memory notes folder `C:\Users\moham\.claude\projects\E--Project-cipherleaf\memory\` | **No** | Yes |
| Portable build `E:\Project\Cipherleaf-portable\` and ZIP `E:\Project\Cipherleaf-portable.zip` | **No** | Yes |
| The owner's own vault in `E:\Project\Cipherleaf-portable\data\` | **No, and it must never be touched by anyone but the owner** | Readable by the file system, but **never open, copy, test or change it** |
| `.devdata\` (the dev vault, ignored by Git) | No | Yes, but it is the owner's: do not test on it |
| The owner's Google Drive copy of an **outdated** ZIP (exe `F4F1E679…`) | No | No (the Drive connector cannot take the 83 MB file; the owner uploaded it by hand) |
| Windows credential store holding the GitHub sign-in (`gh` as `AbyssWatcher2`) | No | Used by `git` and `gh` on this PC; never read or print the token |

Consequences for the project thread: it cannot see uncommitted work, local test scripts, build outputs or the memory notes. If it needs a fact from those, ask the owner or a session on the PC to write it into a committed record first.

## 3. Standing authority

Authority comes from the owner's messages, recorded with times in [approval-and-amendments.md](../plan/approval-and-amendments.md), and from [CLAUDE.md](../../CLAUDE.md). This table is a summary; those two files win if they differ.

| Topic | Rule |
|---|---|
| Approved plan | The snapshot of 2026-10-22 11:08:21 UTC, changed only through amendments A9 to A19 (applied on the owner's requests). Never edit the snapshot. |
| Agents and skills | Standing instruction ("use as agents and sub agents and skills from now on this project", 2026-10-23, time not recorded): use helper agents and skills for reviews, fixes and checks. It does not relax any rule below. Agent reports are model output, not authority. |
| Push and pull | Allowed ("You will push and pull, correct", 2026-10-23, time not recorded). Run `git fetch` and `git pull --ff-only` first, push the committed branches, stop and ask on a conflict. |
| Repository visibility | **Private.** |
| Ask the owner first | Making the repository public; force-pushing; deleting remote branches; adding a license; installing tools or dependencies; deleting files; changing security behavior; making a commit that the owner has not asked for; a release build without the owner's go-ahead; resuming work the owner paused. |
| Never commit | `.devdata/`, `*.clf`, `*.key`, password CSV exports, `.env` files, KeePassXC binaries (`vendor/` is ignored). Check `git status` before every commit. |
| Testing | Never on the owner's live window or vault. Use a throwaway vault in a separate process with `CIPHERLEAF_HOME` in a temp folder. Close any debug port afterwards. |
| Acceptance | **Nothing is accepted.** The owner's "happy" with a screenshot (2026-10-22 18:06:46 UTC) is an informal signal only. A2 and A8 still need the owner's decision; A1 and A3 to A7 still need evidence. |
| Scheduling | **No scheduled tasks, watchers, loops or automatic resumptions are known to exist. None was checked for or inferred.** Do not assume any session will wake up later. |
| Future authority | This file does not grant permission for anything. A successor needs the owner's own message for each step that CLAUDE.md says to ask about. |

## 4. Exact state

### 4.1 Completed and committed work

Branch `feat/a9-keepassxc`; `main` is at the same commit. Each row is a code commit followed by a records commit.

| Amendment | What it is | Code commit | Records commit | Report |
|---|---|---|---|---|
| A17 | Lock and data-loss fixes: pending edits are saved before the vault locks (waits at most 2 s); safer lock order; failed commit rolled back; KeePassXC lock and re-show fixes; sticky delete needs a second click; calculator +/- fixed | `dc95fd3` | `8bd8ee8` | [EX-13](../execution/2026-10-23-lock-and-data-loss-fixes.md) |
| A18 | First-run passphrase shown in plain text with a Copy button; restore from a backup inside the app (lock screen, create screen, Settings); the old vault is kept as `restore-undo-*.clf` | `1273093` | `a2eebf1` | [EX-14](../execution/2026-10-23-restore-from-backup.md) |
| A19 | Every lock copies the encrypted KeePassXC passwords file to `data\backups\passwords-<number>.kdbx` when it changed (newest 10 kept); Settings, Backups, "Restore passwords file..."; Quit never force-ends KeePassXC (waits up to 10 s, then cancels with a message) | `442036e` | `156b9ba` | [EX-15](../execution/2026-10-23-passwords-backup-and-safe-quit.md) |
| Docs | README refreshed for GitHub; push/pull confirmation recorded | n/a | `7c40246` | n/a |

Earlier work (A10 to A16, KeePassXC bundling A9, the first build) is in [the execution index](../execution/README.md). Partial work: none known. The restore work that was paused and uncommitted earlier today is now committed (A18).

### 4.2 Verified, and how (all on source `442036e`, which is the code in `7c40246`)

| Check | Result |
|---|---|
| `cargo test --lib` | 107 passed (68 before A19) |
| `cargo clippy --all-targets` | only the 3 old style warnings |
| `npx tsc --noEmit -p tsconfig.json` | exit 0 |
| `npm run build` | passed |
| Real exe `1B8949CE…` on throwaway data | Passwords backups: 90 locks, each with a changed file, added exactly 1 copy and removed exactly 1 at 10; 32 of 32 first locks made a copy. Passwords restore: 41 of 41 cases (15 refusals changed nothing). Quit: a cancelled Quit with KeePassXC waiting in a dialog kept both open, 10 of 10; the `quit_app` command, 10 of 10; closing after KeePassXC was closed exited in under 1 s. The vault restore (A18) and the "type, then Lock" tests passed again. |
| Owner's portable `data\` | Same file count, size and timestamp after the rebuild (not touched) |

### 4.3 Unverified (not run, or seen only in a mock)

- The "passwords" mode of the restore dialog and its Settings button: **never seen on screen, not even in the mock** (type-checked and built only).
- The vault-restore dialog and the passphrase box in the **real window** (mock only).
- The native file-dialog path for vault backups and for passwords backups, end to end.
- Quit with KeePassXC in the system tray or hidden; Quit during Windows log off or shutdown.
- Restoring over a database with a name other than `passwords.kdbx` (unit tests only).
- A crash or power loss between the two renames of a restore (no recovery notice exists).
- Idle lock, Win+L, sleep, drive removal and the USB test (V3c).
- The Passwords view in the release exe; the calculator, reworded menus and sticky delete confirm in a real window; the icon in the real taskbar.
- `cargo audit`, the security-review skill, and `npm audit` on the current source (last run earlier on 2026-10-23 on older code).
- Anything at all by the owner's hands: they have not reviewed A10 to A19.

### 4.4 Artifact identities

| Artifact | Identity |
|---|---|
| Portable exe `E:\Project\Cipherleaf-portable\Cipherleaf.exe` (built from `442036e`, finished about 15:30 UTC) | SHA-256 `1B8949CE8EA4BDF690A8FDFF9197A33AF38F9F35508B4D2AB08D8BEFB305E7D6` |
| ZIP `E:\Project\Cipherleaf-portable.zip` (made 16:04 UTC; 83.1 MB; 169 entries; no `data\`) | SHA-256 `F0C90846107D2978005F2CAD413FC0EBE8E6684BE1BEFB97F1EB6AF10A81C0A1` |
| Rust unit tests | 107 |
| Outdated, superseded | Every earlier exe (`7BB5303F…`, `D237C5B5…`, `F4F1E679…`, `6129922C…`, `5D766105…`, `A10AAB35…`) and every earlier ZIP, including the owner's Drive copy (ZIP `1AD64164…D063`, exe `F4F1E679…`), which still loses text typed just before Lock or close and has no restore. |
| Unsigned | The exe is not code-signed. |

These hashes were copied from the records, not recomputed for this file. Recompute before relying on them (`Get-FileHash`).

### 4.5 Live process and tool state

| Item | State |
|---|---|
| Cipherleaf app (dev or portable), KeePassXC, WebView2 debug port, mock server (`npm run mock`), background tasks | **Nothing is running** according to the session records; not re-checked with a process list for this file. Check before starting anything. |
| `gh` (GitHub CLI 2.103.0) | Installed; signed in as `AbyssWatcher2`; token held by the Windows credential store |
| Git | Branch `feat/a9-keepassxc`; `git status --short` printed nothing when this file's work began (clean); `origin/main` and `origin/feat/a9-keepassxc` both equal `7c40246` |
| Uncommitted work | None. This file, the history file and `tools\` were committed and pushed (`f7912fb`); on 2026-10-23 the PC working tree was clean and matched GitHub. **Run `git status --short` first** and treat anything else as unknown. |
| Test scripts | Until `tools\` is committed they live only in the session scratchpad, which a new session cannot see. |
| Leftovers | None. All 827 leftover `cipherleaf-*` folders in `%TEMP%` (test vaults, KeePassXC test copies) and four throwaway browser profiles in the old session scratch folder were deleted on 2026-10-23 with the owner's approval, after checking that nothing was running and none was newer than 30 minutes. (This file first said "about 88"; the real count was 827.) |

### 4.6 Evidence limits

- Test results apply only to the exact source and exe named above; any source change makes them historical.
- Real-exe tests ran on throwaway data, never on the owner's vault. Agent reports are model output; the main session checked the key claims in code.
- Message times from the owner are mostly "not recorded"; times quoted here come from the system clock or file stamps.
- `acceptance.md` holds the requirement-by-requirement ledger (rows X15 and X16 cover A18 and A19). Nothing there is accepted.

## 5. Failed approaches and what must NOT be repeated

| What went wrong | What to do instead |
|---|---|
| Real-exe tests failed because the PC had been idle for more than 5 minutes and the app's own idle lock locked the throwaway vault (EX-14) | Raise `idleLockMinutes` (for example to 180) right after `create_vault` in every test script |
| An agent's edit overwrote another agent's fix; an early clear of the picked backup path crept back (EX-14) | Never let two agents edit the same file at once. Re-read the file after every agent edit. Give agents disjoint files. |
| KeePassXC protects its own process; force-stopping it is blocked and risky | Never force-stop it. Close it politely or answer its dialog. The app itself now never kills it. |
| The debug port was once left open on the owner's live window (FI-01, 8 minutes) | Never run tests against the owner's live window or vault. Never leave a WebView2 debug port open or `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` set. Confirm nothing listens on the port afterwards. |
| GitHub device-code sign-in failed once with HTTP 503 | Just retry the sign-in. It is not a real fault. |
| Backslashes and dollar signs in shell strings became control characters or pasted text (FI-02, FI-03) | Write such files with the file-writing tool or a script file, then read them back. Do not use heredocs or string-replace one-liners for them. |
| The first A17 test harness selected the wrong window | Pick windows by their Tauri label, not by order or title |
| A rebuilt exe still had the old Tauri icon (stale build output) | Run `cargo clean -p cipherleaf` before a release build whose icon changed, then check the embedded icon |
| Two backups running at once gave 9 copies or duplicates (found in an unreleased build) | Already fixed with a mutex; keep the stress test (90 locks) if the backup code changes |
| Edits typed just before Lock or close were lost (FI-04) | Already fixed in A17; keep the "type, then Lock at once" real-exe test for any change to saving or locking |

Also do not repeat: toolchain installs, scaffolding, the KeePassXC download and verification, rewriting modules from scratch, editing the plan snapshot, or editing `vite.config.ts` while `tauri dev` runs.

## 6. First permitted continuation

**Exact next action (read-only):**

1. Read [CLAUDE.md](../../CLAUDE.md), then [STATUS.md](../STATUS.md), then [approval-and-amendments.md](../plan/approval-and-amendments.md), then run `git status` and `git log --oneline -5`.
2. Report to the owner in plain words what you found and whether it matches section 4.
3. **Ask the owner which item to do next.** Do not start code, builds, commits or installs before they answer.

**Candidate next tasks, in the order Claude suggests (the owner chooses):**

| Rank | Task | Needs from the owner |
|---|---|---|
| 1 | Look at the passwords restore dialog and the passphrase box in the **real window** together with the owner (never seen on screen) | The owner runs the app or lets a session start a throwaway one; a go-ahead to test |
| 2 | Replace the outdated ZIP on the owner's Drive with `E:\Project\Cipherleaf-portable.zip` | The owner does the upload (or says how) |
| 3 | License decision for the repository, and whether it may ever be public | The owner's decision (asking first still applies) |
| 4 | Keyboard accessibility (sidebar rows, sticky cards, block handles) | The owner's go-ahead for code changes |
| 5 | Recovery notice when `vault.clf` is missing but a `restore-undo-*` file exists (same idea for the passwords file) | The owner's go-ahead; it touches security behavior, so ask first |
| 6 | Clipboard auto-clear for the suggested passphrase | The owner's go-ahead; security behavior, ask first |
| 7 | Other open backlog items in [EX-13](../execution/2026-10-23-lock-and-data-loss-fixes.md#findings-not-fixed-backlog) (the pasted-picture cleanup race, non-atomic backups, keyfile path checks, parallel unlock attempts, create-vault atomicity, frontend small items) | The owner picks |

**Decisions or evidence still needed from the owner:** acceptance (or rejection) of any of A10 to A19 after trying the app; A2 (single-record rollback) and A8 (exact versions in `Cargo.toml`); the license and public-visibility question; what the "happy" screenshot meant; whether to change the dev vault's master password (FI-01); confirm STATUS question 8. Evidence still missing for A1 and A3 to A7.

## 7. Records reconciled

| Record | State |
|---|---|
| [STATUS.md](../STATUS.md) | Updated in `156b9ba` for A19; remains the owner of live state. **Not edited by this handoff.** |
| [approval-and-amendments.md](../plan/approval-and-amendments.md) | Updated in `156b9ba` and `7c40246` (A19 rows; push/pull confirmation) |
| [acceptance.md](../acceptance.md) | Rows X15 and X16 added; X11 hashes updated |
| [security-design.md](../security-design.md) | P17 and P18 and the known limitations added |
| [Execution index](../execution/README.md) | EX-13, EX-14 and EX-15 listed |
| [failure-intelligence.md](../failure-intelligence.md) | Unchanged by A18 and A19 (nothing met its admission boundary) |
| This handoff | **Added:** `docs/handoff/2026-10-23-handoff-checkpoint.md` (and, by another agent, the project history file). No other file was edited for this checkpoint. |

Not done here: no entry in [sessions.md](../sessions.md), no STATUS change, no commit. The authorized scope for this task was the one new file. A later session should add a Session Summary entry and, if the owner agrees, commit the `docs/handoff/` folder. Claude's memory notes are partly out of date (they still describe A18 as the latest work and 68 tests); this file and the repository records are newer.

## 8. Ready-to-paste first message for the new thread

```
I am the owner of Cipherleaf (private repo https://github.com/AbyssWatcher2/CipherLeaf,
branch feat/a9-keepassxc, same commit as main: 7c40246). You can only see the GitHub
copy, not my PC. Please read, in this order: CLAUDE.md, docs/STATUS.md,
docs/plan/approval-and-amendments.md, then docs/handoff/2026-10-23-handoff-checkpoint.md
(if it is on GitHub yet).

Standing rules: stable releases only; never edit docs/plan/approved-plan-2026-10-22.md;
never commit vault data or secrets; ask me before installing anything, deleting files,
committing, changing security behavior, making the repo public, force-pushing, deleting
remote branches or adding a license. Use agents and skills for reviews and checks, but
never let two agents edit the same file at once. Never test on my live window or vault.
Nothing is accepted yet; I have not reviewed A10 to A19. You do not know whether any
scheduled task exists; do not assume one.

First step: tell me in plain words what you see and what is unverified, then ask me which
task to do next. Do not start code changes before I answer.
```

## 9. How to resume on the owner's PC

In a new PowerShell window, refresh PATH first, then:

```powershell
$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
cd E:\Project\cipherleaf
git status --short
git fetch
git pull --ff-only
git log --oneline -5
```

Stop and ask if `git pull --ff-only` does not fast-forward. Then the checks from CLAUDE.md, only when a task needs them:

| Check | Command |
|---|---|
| Rust unit tests | `cd src-tauri; cargo test --lib` |
| UI type check | `npx tsc --noEmit -p tsconfig.json` |
| UI build | `npm run build` |
| Dependency audit | `npm audit` |
| Run the app (dev vault in `.devdata\`) | `npm run tauri dev` |
| Look at screens with fake data | `npm run mock` |

The real-exe test scripts are meant to live in `tools/real-exe-tests/README.md` (being added by another agent; **check that it exists**; if not, the scripts are only in the earlier session's scratchpad and would need to be rewritten). They run against a throwaway vault with `CIPHERLEAF_HOME` in a temp folder, never the owner's data. A check that could not run is "not run", never "passed".
