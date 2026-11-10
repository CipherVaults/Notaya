# PC automation: the heartbeat, the guard, the lanes, and what changes without Claude

**Who and where:** Ahmed's side (`AE0101`). Written by Claude, cloud, 2026-11-10 about 04:30 UTC (system clock). Source: a read-only look at `D:\Notaya-lanes\heartbeat` on the Gaming-PC (file list, `README.txt`, the headers and the Claude-meter part of `heartbeat.ps1`, `events.log` and `heartbeat.log` tails at about 03:31 UTC). **Reference copies of the scripts are in `tools/heartbeat/`** (no secrets found by a pattern search; they contain Ahmed's Windows user name in a few paths). The live copies on the PC are what runs; the repository copy is a snapshot of 2026-11-10 03:3x UTC. Result state: *produced*; nothing was run.

## 1. What runs on the PC

Two per-user scheduled tasks (no admin rights, hidden, one instance at a time). **Ahmed registers them himself** (Claude Code's permission system refuses to register persistence; keep it so: nobody installs persistence for him).

| Task | Every | Starts | Script |
|---|---|---|---|
| **Notaya heartbeat** | 10 minutes | `heartbeat-hidden.vbs` (window style 0) → `heartbeat.ps1` | registered by `register-task-hidden.ps1` (the older `register-task.ps1` is the undo) |
| **Notaya stop-line guard** | 2 minutes | `stopline-guard-hidden.vbs` → `stopline-guard.ps1` | registered by `register-guard-task.ps1` |

Remove: `Unregister-ScheduledTask -TaskName 'Notaya heartbeat' -Confirm:$false` (and `'Notaya stop-line guard'`). Backups of earlier task XML are in `task-backup\`.

**Folders (all outside the repository):** `D:\Notaya-lanes\heartbeat` (scripts, logs, state), `D:\Notaya-lanes\<lane>` (one git worktree per lane, branch `AE0101/<topic>`), `D:\Notaya-lanes\out` (lane logs and hand-back notes), `D:\Notaya-lanes\_target` (shared Rust build cache), `D:\Notaya Tests\v1-6-<lane>` (each lane's throwaway `NOTAYA_HOME`). The heartbeat works **only** inside `D:\Notaya-lanes` and `D:\Notaya Tests`, never touches `D:\Notaya`'s checkout, any real vault, or drive `E:`, and **never merges**.

## 2. What one heartbeat tick does (`heartbeat.ps1`, lines in the file's own comments)

1. Takes a lock (`lock`) so only one tick runs at a time.
2. Reads the **Codex meters** (5-hour and weekly percent used) from Codex's own session files under `%USERPROFILE%\.codex\sessions\` (newest `token_count` event with rate limits). If the newest event is older than 15 minutes and lanes are running, takes one tiny non-ephemeral `codex exec` probe (about 29,000 input tokens, low effort).
3. Reads the **Claude meter** by a tiny `claude -p` probe (Haiku, no tools, about 900 tokens) at most every 5 minutes, stored in `claude-rate.json` (the status-line script `claude-statusline.ps1` can also feed it). Claude weekly pause line: 70% was the rule, **100% since A50**; 60% is information only.
4. Applies the **stop line**: Codex's 5-hour window used ≥ 95%, or used + burn rate × 5 minutes ≥ 95% (predictive, look-ahead `$LookAheadMin = 5`), stops the running lanes cleanly (kills the Codex process tree, makes a WIP checkpoint commit, plain-pushes the lane branch) and pauses until the reset time. A line held only by the *prediction* lifts again when nothing runs, used < 90% and used + burn × look-ahead < 95. At the **weekly 100%** it pauses and reports; it **never uses or assumes a banked reset** (Ahmed decides and applies it himself).
5. **Starts queued lanes** while fewer than `$maxAlive` (3) Codex lane processes run and quota is available; relaunches stopped lanes (a stop with exit 0 and no file changes is an *idle-stop*, retried up to 2 times; a launch that dies at once is retried up to 3 times; lanes are started through WMI from the real Codex path).
6. After a lane finishes: runs the checks on the pushed commit (`cargo test --lib`, `npx tsc --noEmit -p tsconfig.json`, `npm run build`, `npm audit`, plus the lane's browser/real-exe scripts), makes the commit, pushes the branch, logs `finished ... checks: ...`.
7. Appends one row per tick to `docs/ledger/quota-readings.csv` and pushes the new rows about hourly to the branch `AE0101/ledger-readings`.
8. Logs: `heartbeat.log` (one line per tick), `events.log` (`started`, `finished`, `idle-check`, `limit-pause`, `limit-resume`, `weekly-limit`, `stopline-resume`, `claude-meter`, `ledger-push`), `heartbeat-state.json` (lane status, stalls, stop-line state).

**The stop-line guard** (`stopline-guard.ps1`) never starts a lane or a probe. Its one job: when the Codex 5-hour window is at the stop line and a lane still runs, stop the lanes cleanly as above. It exists because a heartbeat tick can be busy for many minutes (finish checks) and nothing samples the meter meanwhile (Q65). Options: `-DryRun`, `-Replay -ReplayDate <date> [-ReplayFrom HH:mm:ss]`, `-Inject "min:pct,..."` (synthetic rule test; changes nothing).

**Rules the automation enforces** (Ahmed's words, see `03-standing-rules.md`): work in the 5-hour window until about 5% is left, then stop cleanly (2026-11-08 11:48:49 UTC); at most three Codex lanes at once (two once the window is above 60%); banked resets never automatic; Codex lane gets its own worktree and throwaway data; never merge.

## 3. Lanes

`$lanes` in `heartbeat.ps1` is a table: lane id → `br` (branch), `title`, `deps`, `base`, `merge` (branches merged in first), `range` (line range of `docs/plan/2026-11-08-v1-6-codex-briefs.md`), optional `promptFile` (`D:\Notaya-lanes\heartbeat\prompt-<id>.md`), optional `model` and `effort` (A54: add `model='gpt-6.1-terra'; effort='high'` for light lanes; no field = Sol xhigh; the model id is checked by a one-line trial before first use, see `terra-trial.ps1`), `review`/`reviewRef` for read-only review lanes. **To add a lane:** write `prompt-<id>.md` (use the template at the end of `02-start-prompts.md`), add its table entry, commit nothing in the repo for it (the lane branch is created from `base`), and let the next tick start it. A lane's hand-back goes to `D:\Notaya-lanes\out\<tag>.md`.

Lane state at about 03:31 UTC on 2026-11-10 [Observed, `heartbeat.log`]: S has "attention" (tip `3a2581f`, one changed file, security area 3, blocked by the cyber filter); N2 pushed `e11b115`; P6b pushed `31c2ce8`; P6 pushed `d1d5ee6`; all other lanes pushed or done. Codex 5-hour 62%, weekly 56% used; Claude weekly 91%, 5-hour 16%. The prompt files `prompt-*.md` and `briefs.md` stay on the PC (about 400 KB, they duplicate the briefs in `docs/plan/2026-11-08-v1-6-codex-briefs.md` plus follow-ups); the coordinator may copy any it needs into `docs/plan/` when a lane becomes a record.

## 4. What must change now that Claude is not the coordinator

| Item | Where | What to do |
|---|---|---|
| **The Codex launcher path** | `run-lane.ps1`: `C:\Users\Ahmed\AppData\Local\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\npm\codex.ps1 exec ...` | That install is visible only to the packaged Claude app (root cause found 2026-11-08 12:04 UTC, EX-2026-11-08-55 section 5). Point it at the real Codex CLI the Codex desktop app uses (find it with `where.exe codex` or in `%APPDATA%\npm`) and run one lane as a test before relying on it. [Check] |
| **The Claude meter probe** | `heartbeat.ps1` (the `claude -p` probe, `claude-rate.json`, `claude_quota_state`) and `claude-statusline.ps1` / `apply-claude-statusline.ps1` | Not needed once nobody leads from Claude. Either leave it (it costs about 900 tokens every 5+ minutes from Ahmed's remaining Claude quota, so **switch it off**: Codex should change the `$needs` rule to `$false` or guard the block with a setting) or keep it only if Mohamed's account were used on that PC (it is not). The script handles a missing `claude` by falling back to a fixed path, inside a try block; run `heartbeat.ps1 -DryRun` after the change. [Check] |
| **Commit message texts** | `heartbeat.ps1` (WIP checkpoint, lane commits) and `stopline-guard.ps1`: "run and committed by Claude on Ahmed's PC", "Made by Claude's heartbeat" | Reword to the new actor ("ChatGPT coordinator" or "the Notaya heartbeat") so records stay truthful (author and machine on every record). |
| **Claude-only variables and paths** | `$legacy = 'C:\Users\Ahmed\AppData\Local\Temp\claude\...'` (old output folder of the first run), `claude.exe` fallback path, `loop.ps1` | Harmless history; leave or remove. |
| **Quota to read** | New | The Codex meters keep working if the Codex desktop app writes the same `~\.codex\sessions` files as the CLI [Check]. Add ChatGPT's own meter by hand (Ahmed reads its usage page) into `quota-readings.csv` until a safer automatic reading exists. |
| **Who merges** | Records | The heartbeat never merges. The ChatGPT coordinator merges (merge commit) after reading the lane in full and seeing the PC's checks on the exact commit. |
| **Persistence rule** | Policy | Ahmed registers or changes scheduled tasks himself, with `-DryRun` first. |

Test any change with `heartbeat.ps1 -DryRun` and `stopline-guard.ps1 -DryRun`, on throwaway data, and read the log lines before the next real tick. Never leave `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` set; close debug ports and confirm nothing listens (FI-01).

## 5. If the PC automation stops

The records still say where each lane is (STATUS "Continuation", claims). Worst case: Codex lanes are run by hand in the Codex desktop app with `AGENTS.md`, one lane per worktree, and the checks are run by hand from the table in `03-standing-rules.md`. The three-lane limit and the stop line then become the coordinator's own discipline.
