# Configuration and runtime formats

The scripts target Windows PowerShell 5.1. Copy `config.example.ps1` to
`config.ps1`, edit that one file, and copy `lanes.example.json` to the configured
`LanesFile`. The examples are invented. No runtime state, account meter readings,
session identifiers, credentials or original lane prompts are included.

Each PowerShell entry point dot-sources `config.ps1` next to itself unless
`-ConfigPath` supplies another file. It must define `$Cfg` as a hashtable. All
contract keys and additional settings are commented in `config.example.ps1`.
Derived settings are computed when that file is loaded; when changing a root in
another config, update its derived paths too. Copying the kit does not rewrite
absolute paths automatically. Use the same config for heartbeat, guard and task
registration. Each independent installation needs its own roots, task names,
logs and state.

## Switches and operation boundaries

| Setting | Default | Effect |
| --- | --- | --- |
| `EnableRepoActions` | false | Enables Git fetch, worktree creation, merge, staging, checkpoint/final commits, plain pushes, and hourly ledger publication. Read-only Git status/log/diff can run while disabled. |
| `EnableProcessStop` | false | Enables `taskkill /PID ... /T /F` for declared lane Codex process trees. With it disabled, the stop line logs the action and blocks new starts but leaves existing writers running. |
| `EnableClaudeProbe` | false | Enables optional paid Claude meter probes if `ClaudeCmd` is supplied. Passive reading of an existing meter file remains possible. |
| `EnableCodexProbe` | true | Preserves the stale-meter Codex probe from the original loop; costs tokens and may access the network. Set false for passive readings only. |
| `-DryRun` | switch | Overrides all opt-ins: no process enumeration, tool execution, Git commands, probes, checks, sleeps, locks, file writes, or state saves. It reads configured fixture inputs and prints a plan. |

A normal heartbeat can launch Codex into existing worktrees even with repo
actions disabled. That flag controls the heartbeat's Git operations, not the
contents of operator-authored prompts or what an agent can do. With repo actions
disabled, missing worktrees and developer lanes awaiting finalization get an
`attention` state and an event instead of being reported as pushed. Review lanes
can finish without a push. Stopping is independent: a successful process stop
can produce `paused-stopline` without a checkpoint or push when repo actions are
disabled. A failed kill leaves the lane unpaused and logs `attention`.

No scheduled task is registered by heartbeat or run-lane. The separate register
scripts own task setup. Launch uses WMI `Win32_Process.Create` with a hidden
startup window, then a hidden `Start-Process` fallback. This separates the lane
from the tick's job object, which otherwise can end its children. `run-lane.ps1`
invokes the physical `CodexCmd` wrapper: an MSIX-packaged app can hide its npm
folder from scheduler-created processes. Both paths carry the same config,
model and effort. The fallback may still be subject to its caller's job object.

## Lane table

`$Cfg.LanesFile` contains a JSON array. Array order is launch priority. An empty
array (`[]`) is valid and useful for a first dry run.

| Field | Meaning |
| --- | --- |
| `name` | Unique, case-insensitive directory/event identifier: letters, numbers, `_` or `-`, starting with a letter or number. Windows device names and runtime-directory collisions are rejected. Worktree is `Root\name`; test home is `TestsDir\name`. |
| `branch` | Git branch; empty means `BranchPrefix + name`. |
| `base` | Starting ref; empty means `Remote/MainBranch`. For a review, this is the ref used for its detached worktree. |
| `merge` | Array of refs merged after creating a developer worktree. A conflict aborts that preparation and blocks launch. |
| `deps` | Array of lane names that must be `pushed` with `checksFailed=false`. Names must exist. |
| `title` | Plain description for commit/report text. |
| `promptFile` | Optional path relative to `KitDir`; it must remain below that folder. Default is `prompt-<lane>.md`. |
| `model` | CLI model identifier; empty uses `DefaultModel`. No model availability is implied. |
| `effort` | CLI effort; empty uses `DefaultEffort`. Allowed: minimal, low, medium, high, xhigh, max. The installed model must support it. |
| `review` | Boolean: create a detached worktree and finish with a hand-back, without a developer commit/push. The prompt and CLI sandbox grants must enforce the intended review scope. |
| `gate` | Optional lane name whose `reviewReady` state must be true. |
| `fix` | Boolean: retains the original fix-launch counter; normal concurrency limits still apply. |
| `range` | Optional `first,last` inclusive, 1-based line range into an operator-supplied `KitDir\briefs.md`, used only when no prompt file exists. No private briefs table is shipped. |

Ref/model fields are validated before interpolation into launch arguments.
Dependency cycles remain blocked; the loader does not resolve cycles. Changing a
lane definition does not erase its previous state. Use a new name for new work,
or review and explicitly edit the state with both tasks stopped.

## Prompt and output conventions

Prompt files start with `Read AGENTS.md ... first` on the first line, for example:

```text
Read AGENTS.md first.
Complete the invented widget task within its declared write area.
```

Supply the actual bounded task, write area, checks and stop point. Prompts are
trusted operator instructions, not a permission grant from this kit. There are
no embedded project rules or project-specific follow-up messages.

Optional `KitDir` files:

- `briefs.md`: source for the lane's `range` when its prompt is absent.
- `done-note-<lane>.md`: prior state text appended on a resumed run.
- `followup-<lane>.md`: extra requirements appended to a generated prompt; if a
  lane finishes before this file has been consumed, it can launch once as a
  follow-up, subject to quota holds. Standalone follow-ups follow the same
  `Read AGENTS.md ... first` convention.

Generated prompts go to `OutDir\prompt-<lane>.generated.md`; source prompts stay
unchanged. `run-lane.ps1` reads the prompt on stdin, uses `workspace-write`,
grants the worktree plus configured Cargo/test/output directories and
`AdditionalWriteDirs`, and writes `OutDir\<lane>.md` (hand-back) and
`OutDir\<lane>.log` (all CLI streams). Follow-up output uses `<lane>.fu`.
Logs end in `EXIT <code>`. Prior `.md` and `.log` output is moved to a timestamped
`.prev-HHmmss` name before launch. `LaneEnvironment` values expand `{Root}`,
`{TestsDir}` and `{lane}` without expression evaluation; use them to configure
the target project's throwaway data. Cargo is kept offline. A normal completed
lane uses `CargoTargetDir`; heartbeat finish checks use `CheckCargoTargetDir`
to preserve the original separate cache for checks. A completed
developer lane can run tsc/build for changed source, Rust/bridge tests for those
paths, and npm audit, then write `<lane>-report.md` and commit-message files.
These retained check conventions may need adaptation for another project.

## State file

`$Cfg.StateFile` is UTF-8 JSON, serialized to a sibling `.tmp` and replaced after
a tick. The heartbeat holds `KitDir\lock` exclusively for the tick. The guard
uses its own coordination; the heartbeat adopts `KitDir\stopline-guard.json`
when present. No populated state file is distributed. New configured lanes
start `pending`; there is no implicit import of historical lanes or logs.

Top-level keys (optional until first needed):

| Key | Shape / purpose |
| --- | --- |
| `lanes` | Object keyed by lane name; per-lane fields below. |
| `limit` | `{active, resetUtc, note}` from recent usage-limit log text or exhausted primary meter. |
| `limitIgnoreBefore` | UTC timestamp; older textual limit evidence is ignored. |
| `rate` | Latest Codex reading `{f, ts, p, w, pr, wr, plan?, probe?}`: rollout filename, UTC timestamp, used five-hour/weekly percentages, primary/weekly reset Unix seconds, optional plan/probe marker. |
| `rateHist` | Up to eight `{t,p,pr}` readings, deduplicated by timestamp; `pr` identifies a quota window. |
| `stopLine` | `{active, resetEpoch, resetUtc, since, reading, overshoot, burnPerMin, predicted12, reason}`. `reason` is measured/predicted or a guard-prefixed version. `predicted12` is a legacy field name: its horizon is actually `LookAheadMin`. |
| `liftedForWindow` | Reset Unix seconds identifying the one window whose predicted hold already lifted. |
| `weeklyLimit` | Boolean annotation; no banked-reset operation exists. |
| `claude_quota_state` | `{claude_pause, claude_warn, weekly_used_pct, five_hour_used_pct, weekly_resets_utc, five_hour_resets_utc, reading_ts, fresh, source}`. This annotates Claude work; it does not stop the Codex lanes. |
| `ledger` | `{lastLine,lastPushUtc}`: local CSV publication cursor and last successful hourly push. |

Per-lane keys:

- `status`: pending, running, followup-running, checking, pushed, done,
  paused-stopline, attention, or an operator-supplied hold.
- `resume`, `followupDone`, `startedUtc`, `stalls`: continuation and relaunch
  bookkeeping. `legacy` is retained for compatibility but new runs use false
  and never read a historical scratch folder.
- `workBase`, `mergeMainBeforeFinish`, `mergeInfo`, `premerge`, `premergeExtra`:
  baseline and optional integration requests/conflict filenames. Merges into a
  lane are enabled operations; this kit never merges a PR or makes a release.
- `meterBefore5h`, `meterBeforeW`: Codex readings at launch, for the event suffix.
- `claudeBefore`: optional `{ts,p5,pw}` snapshot for the before/after probe delta.
- `commit`, `pushed`, `reviewReady`, `checksFailed`: finalization facts. A pushed
  branch with a failing non-audit check has `checksFailed=true`; npm audit is
  recorded but retains the original advisory-only treatment.

## Meter and stop-line rules

Codex readings come only from `CodexSessionsDir\**\*.jsonl`: the newest matching
`rate_limits` event among up to twelve recently modified rollout files (six-hour
mtime horizon, bounded tail reads). The event has `timestamp` and
`payload.rate_limits` with `limit_id="codex"`, `primary` and `secondary` objects
containing `used_percent` and `resets_at`. Whitespace around JSON keys is allowed.
Meters are account-wide; a lane event's delta also includes other sessions.

Burn is the percentage-point increase per minute from the oldest same-window
reading 0.5 to 20 minutes before the newest reading. Negative burn, insufficient
history, or a newest history reading over four minutes old gives zero. Launches
hold when measured usage reaches `StopAtPct`, or positive burn predicts that
threshold within `LookAheadMin`. Current lanes are stopped only with the process
flag on; WIP checkpoint/plain push also need the repo flag. Overshoot is measured
usage above the configured line. No banked reset is used.

A reset or changed window releases a hold. A predicted hold can also lift once
per window when no declared lane runs, usage is below `LiftBelowPct`, and its
prediction is below `StopAtPct`. A measured hold waits for reset. Normal parallel
capacity is `MaxAlive`; at `HighMeterPct` it becomes the smaller of that value and
`MaxAliveHigh`. The tick-start process snapshot plus launches in this tick avoids
counting a fresh launch twice.

`PausedWindow` recognizes an active measured hold (or a predicted hold after its
one lift, or measured usage at the stop line) with a plausible stored reset from
one minute ago through 5.5 hours ahead. It suppresses Codex stale-meter probes
and Claude before/after probes. The periodic optional Claude refresh accepts
cached data for two hours while paused, then can refresh once; this preserves
the original periodic refresh policy. Outside a hold, a stale Codex reading over
15 minutes can cause one enabled probe; Claude refreshes after five minutes.

The watch loop runs at `WatchStartPct`, or when burn predicts `WatchPredictPct`
within `WatchPredictMin`. It samples every `WatchSampleSec`, for at most
`TickMinutes - GuardMinutes`, while lanes remain alive. It enforces the same stop
line and exits on a hold. The independent guard covers time spent in long checks.

## Log and CSV lines

`EventsLog` is append-only text, one event per line:

```text
<UTC yyyy-MM-dd HH:mm:ssZ>|<lane or ->|<kind>|<message>
```

Lane start, finish, failed-check finish, and stop events append:

```text
 | model=<model> effort=<effort> | Codex meter 5h <before>% -> <now>%, weekly <before>% -> <now>% (last reading <HH:mm:ssZ>)
```

The suffix itself contains pipes: split the first three separators for the
event header and retain the entire rest as its message. A missing meter is an
empty value, not a zero reading. `claude-meter` events contain before/after
account-wide percentages and timestamps. Lifecycle kinds include started,
followup-started, finished, finished-checks-failed, paused-at-stopline,
stopline-pause, stopline-resume, stop-disabled, repo-actions-disabled,
limit-pause/resume, weekly-limit/resume, attention and idle-check.

`HeartbeatLog` has UTC tick summaries and free-text diagnostics:

```text
<UTC> <no-limit or LIMIT until reset> [optional WEEKLY-LIMIT] [claude 5h=<p>% wk=<p>% WARN/PAUSE/stale] [codex 5h=<p>% wk=<p>% @HH:mm:ss] | <lane>:<status>[(alive)] chg=<count> <last-short-commit> | ...
```

No-lane summaries end after the pipe. Diagnostic lines start with UTC and a
description. DryRun prefixes console output with `DRYRUN` and never appends logs.
`LedgerDir\quota-readings-pc.csv` uses quoted cells in this exact column order:

```csv
time_utc,tool,account,limit,reading,resets_utc,read_by,source,notes
```

The account/observer labels are configured. Each tick appends available meter
readings; stale evidence is marked in notes. Hourly publication uses
`Root\ledger-wt`, branch `BranchPrefix + ledger-readings`, and `LedgerRepoPath`.
A conflict limited to that CSV keeps both sides and deduplicates identical lines;
other conflicts abort publication and log attention. Push retries are plain,
never forced. `claude-rate.json`, if present, has `ts`, `source`, `five_hour` and
`seven_day` objects (`used_percentage`, `raw_utilization`, `resets_at`), plus
optional `probe_usage`.

## Integration notes for README and USAGE

The documentation lane should carry these concrete limits into the README and
usage guide. The executable merge/push/kill behavior is deliberately included.

- Opting into repo actions grants real commit, fetch, merge and push access,
  including Git worktree metadata at `MainCheckout`. Opting into process stop
  forcibly kills matched process trees and can interrupt an in-progress write.
  Use disposable worktrees and review recovery behavior before enabling either.
- Config is executable PowerShell. Config, lane definitions, prompts, state,
  extra sandbox directories, environment settings, Git hooks and repository
  build commands must be trusted. These switches are not a security sandbox.
  External tools can also use their own profiles, caches and temporary files;
  configure their environment separately when stronger isolation is required.
  Existing path traversal and reparse points in configured write paths/lane
  roots are rejected; concurrent filesystem replacement remains an OS boundary.
- File filters are heuristics, not secret detection. Existing staged content
  can be committed, and pushes publish existing commits too. Inspect the index,
  extend `ExcludedCommitPathPattern`, and never put secrets in these worktrees.
- `EnableRepoActions=false` and `EnableProcessStop=false` do not make a normal
  tick offline or read-only: agent launches, logs/state/CSV writes, and enabled
  Codex probes remain possible. `-DryRun` is the no-execution/no-write preview.
  Enabled agent/probe calls and the retained npm audit check may use the network.
- Missing/stale meter evidence can delay a stop. Missing evidence does not add
  a new fail-closed launch rule; that original behavior is retained. Predictions
  reduce overshoot but cannot guarantee an exact quota ceiling. Probes and
  relaunches themselves consume quota. A refusal/error log is evidence for the
  operator to review; this kit does not bypass provider restrictions.
- A second tick shares the heartbeat lock, but the separate guard and external
  coordinators can still race a commit, push or merge. Atomic heartbeat state
  replacement prevents a partially written state file, not a cross-process Git
  transaction. Avoid another writer and review conflict/attention events.
- Parsed scripts, synthetic meters and mocked enabled actions do not establish
  scheduler permissions, real account/model availability, real process-tree
  termination, Git hook safety or successful production pushes. Those require
  a separate operator-controlled disposable integration run.

The README owner must state the pending joint licence decision and the
redistribution restriction. No licence file is added here.
