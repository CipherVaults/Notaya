# Guard, tasks and model trial

This part of the kit consumes the shared `config.ps1`, which must define `$Cfg` as a
hashtable. Copy the complete `config.example.ps1` supplied with the assembled kit,
then edit that single configuration. Scripts accept `-ConfigPath` for an isolated
copy; the launchers pass that same file to PowerShell. Relative launcher paths
come from the VBS file location. Install all scripts together in `Cfg.KitDir`.

The shared configuration, heartbeat and runner are supplied by the other kit part.
They are not duplicated here. This part never needs a second settings file.

## Configuration comments to include in the shared example

The configuration owner should include these two optional keys and comments in
`config.example.ps1`. Neither changes any existing contract key:

```powershell
# Read-only JSONL source for the guard. An absent key falls back to
# Join-Path $Cfg.TestsDir 'codex-sessions', which can hold synthetic readings.
# To use your signed-in CLI history, set the source directory explicitly.
CodexSessionsDir = 'C:\devkit\lanes\tests\codex-sessions'
# Old root-folder task names to retire when registering the heartbeat.
# Each existing task is backed up before removal. Empty means no migration.
LegacyTaskNames = @()
```

`Root`, `KitDir`, `OutDir`, `TestsDir`, `StateFile`, `EventsLog`, `HeartbeatLog`,
`Remote`, `BranchPrefix`, `CodexCmd`, task names, cadence, `LookAheadMin`,
`StopAtPct`, defaults and the two action flags use the shared contract unchanged.
`EnableRepoActions = $false` and `EnableProcessStop = $false` must stay the example
defaults. The task scripts have their own deliberate invocation boundary: these
flags do not disable task registration or unregistration.

## Start with dry runs

From the installed kit folder, in a normal PowerShell 5.1 console:

```powershell
.\stopline-guard.ps1 -DryRun
.\stopline-guard.ps1 -Inject '0:80,2:86,4:94,6:96'
.\register-heartbeat-task.ps1 -DryRun
.\register-guard-task.ps1 -DryRun
.\unregister-tasks.ps1 -DryRun
.\terra-trial.ps1 -Model example-model -Effort high -WaitSeconds 0 -DryRun
```

Registration dry runs intentionally do not query Scheduler or running processes;
they print both the planned backup and replacement. They also validate the script
and VBS dependencies exist. No dry run creates log directories or lock files.
The guard's dry run reads only the configured state, history and session source,
and inspects process command lines to identify lane children of `Root`.

After inspecting the dry-run output, the user can run each registration script
without `-DryRun`. It backs up an existing root-folder task XML into
`KitDir/task-backup` before replacing it, registers under the current interactive
user with limited privileges, hidden, one instance at a time, and starts it.
Cadences come from `TickMinutes` and `GuardMinutes`. Runtime limits remain three
hours and ten minutes respectively; the triggers repeat for ten years.

Unregistering lists both configured tasks and requires the exact typed answer
`yes`. All selected XML backups must succeed before any task is unregistered.
It does not stop an already running task process. Keep the backup folder if you
want to restore an earlier definition with Task Scheduler's Import Task action.

The VBS launchers use `wscript.exe`, window style **0**, and **wait=True** so task
instance and timeout rules remain effective. An optional first argument is the
configuration path. A second argument `--dry-run` passes `-DryRun` to PowerShell.
The scripts and configuration are trusted executable code; keep them in a folder
only the intended user can write.

## Stop-line behavior and action flags

The guard does not launch lanes, probe a service, merge branches, or spend banked
resets. It reads recent Codex JSONL events, retains readings for the same reset
window, calculates burn from an older reading 0.5 to 20 minutes earlier, ignores
burn when the latest reading is more than four minutes old, and stops at either
`StopAtPct` measured or measured + burn x `LookAheadMin`. The main heartbeat owns
lifting a predicted pause and resuming at reset. State/event field names and pipe
separators match the original, including the historical `predicted12` key (its
value uses the configured look-ahead, despite that old name).

| EnableProcessStop | EnableRepoActions | At the stop line |
| --- | --- | --- |
| false | false | Log what would happen; no process stop, Git call or active pause |
| false | true | Same: a running writer is never checkpointed by this guard |
| true | false | Stop matched process trees; log that checkpoint/push is disabled; record pause |
| true | true | Stop trees; checkpoint eligible files and push branch; record pause |

When enabled, the guard retains `taskkill /T /F`, a four-second wait, `git add`,
WIP `git commit`, and plain `git push -u` to `Cfg.Remote`. It never force-pushes.
A failed stop skips repository work for that lane. A merge in progress, detached
HEAD, or branch outside `BranchPrefix` skips checkpoint and push. A failed Git
operation is logged without claiming the branch was saved.

## Differences from the source beyond paths

- Side effects now require the flags above; observation modes truly write nothing.
- Lane command-line matching supports quoted paths and hyphenated lane names;
  only a complete `-C Root/child` argument is eligible, not substring matches.
- Short odd-length JSONL files retain their opening byte, fixing missed readings.
- Configured look-ahead and threshold appear in replay/output; state keys remain compatible.
- Process termination is checked again before checkpointing. Git failures are
  checked. Status uses NUL-separated paths and literal pathspecs, handling spaces,
  Unicode and rename pairs. Already staged protected paths fail closed.
- Checkpoint text has no personal identity or unverifiable model attribution.
- Registration backs up both task types before replacement with `-Force`;
  legacy names are explicit opt-in migration settings, rather than embedded names.
  XML declarations are normalized to the actual UTF-8 backup encoding, and a
  Scheduler query failure aborts rather than being treated as an absent task.
  Only a legacy `KitDir/loop.ps1` process is eligible to stop, behind the stop flag.
- Unregistration with typed confirmation and backup is new.
- The model trial waits a relative parameterized duration instead of a historical
  deadline, uses configured model/effort by default, invokes the CLI directly,
  and adds a dry run. Its response, errors, model observations, tokens and meter
  reporting are retained. A real trial consumes quota and may contact the provider.
- The ledger changes and inherited forecasting assumptions are in `ledger/README.md`.

## Risks and verification limits

Enabling the flags enables real actions, not a sandbox. Forced termination can
lose unsaved work and kills descendants too. Checkpoints may run Git hooks and
include any eligible file, including a secret stored under an unrecognized name.
The protected-extension filter is a guardrail, not secret detection. Git metadata,
hooks, filesystem junctions and remotes can reach outside the configured folders;
configure only trusted throwaway/work repositories. Push uses the user's existing
Git credentials and network connection. Do not enable actions on an unreviewed
checkout. This code adds no credentials or provider tokens.

The heartbeat and guard use different locks. As in the source, if a tick holds
the heartbeat lock the guard still stops lanes and writes the guard file for
adoption on the next tick. Repository operations can therefore race a busy tick;
review enabled operation before production use. File state writes retain the
source's non-transactional behavior. Scheduler backup/replacement is not a group
transaction: failure may leave a partially migrated set, with XML backups retained.
Registration may be denied by Windows; the user must run it themselves. Persistence
is created only by the explicit non-dry-run registration invocation.

Tests use synthetic data and stubs. No live task registration, termination, push,
provider call, browser or application run is needed to validate the package.
The assembled heartbeat/config interaction and actual Windows scheduling still
need coordinator verification. A self-check does not replace independent review.
