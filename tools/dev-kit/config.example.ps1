# Copy to config.ps1 beside the scripts. This is the ONE configuration contract.
# Every PowerShell entry point dot-sources it; protect it like executable code.
# Edit roots here first; derived values below are computed when dot-sourced.
$Cfg = @{
    Root = 'C:\devkit\lanes' # Parent of lane worktrees; each JSON name is a child directory.
    KitDir = 'C:\devkit\kit' # Installed scripts, prompts, lock and runtime metadata.
    MainCheckout = 'C:\devkit\repo' # Existing Git checkout used to create worktrees.
    Remote = 'origin' # Existing Git remote; no remote is created by the kit.
    BranchPrefix = 'lane/' # Prefix used when a lane omits its branch, and for the ledger branch.
    CodexCmd = 'C:\devkit\bin\codex.ps1' # PHYSICAL CLI wrapper, not an MSIX virtual alias.
    ClaudeCmd = '' # Optional physical Claude executable; empty disables its probes.
    TaskNameHeartbeat = 'DevKit heartbeat' # Heartbeat scheduled-task name (registration is separate).
    TaskNameGuard = 'DevKit stop-line guard' # Separate stop-line scheduled-task name.
    MaxAlive = 3 # Normal maximum parallel lanes.
    MaxAliveHigh = 2 # Maximum at or above HighMeterPct; never exceeds MaxAlive.
    HighMeterPct = 60 # Used percentage that reduces launch concurrency.
    StopAtPct = 95 # Measured or predicted used percentage that holds new launches.
    LookAheadMin = 5 # Burn-rate prediction horizon, in minutes.
    WatchStartPct = 70 # Used percentage enabling the heartbeat's short watch loop.
    WatchSampleSec = 60 # Seconds between watch-loop readings; guard is independent.
    DefaultModel = 'gpt-6.1-sol' # Default when a lane's model is empty.
    DefaultEffort = 'xhigh' # Default when a lane's effort is empty.
    EnableRepoActions = $false # OPT IN: fetch, worktree creation, add, merge, commit and push.
    EnableProcessStop = $false # OPT IN: taskkill /T /F of matching lane Codex trees.
    EnableClaudeProbe = $false # OPT IN: paid/network Claude probes and before/after samples.
    TickMinutes = 10 # Heartbeat task interval; also bounds the watch loop.
    GuardMinutes = 2 # Independent guard task interval.

    # Additional settings needed to remove machine/account/project assumptions.
    MainBranch = 'main' # Branch fetched and merged from Remote.
    EnableCodexProbe = $true # Preserves live stale-meter refresh; costs tokens/network. Disable for offline use.
    CodexSessionsDir = 'C:\devkit\codex-sessions' # Read-only rollout JSONL source; set your actual CLI sessions folder.
    ExtraPath = @() # Optional physical tool directories appended to the machine/user PATH.
    AdditionalWriteDirs = @() # Optional explicit Codex --add-dir grants, e.g. a tool cache or review folder.
    LaneEnvironment = @{} # Optional process environment, e.g. @{ DEMO_HOME = '{TestsDir}\{lane}' }.
    LedgerAccount = 'example-account' # CSV account label; never put credentials here.
    LedgerReader = 'heartbeat tick (example-machine)' # CSV observer/machine label.
    LedgerRepoPath = 'docs/ledger/quota-readings.csv' # Relative CSV tracked on the ledger branch.
    ExcludedCommitPathPattern = '(?i)\.(nota|clf|key|csv|env)$|(^|/)\.devdata/|(^|/)\.env' # Legacy checkpoint exclusion regex; extend for your project's secrets.
    LiftBelowPct = 90 # A predicted hold can lift once per window only below this percentage.
    WatchPredictMin = 10 # Additional early-watch prediction horizon, in minutes.
    WatchPredictPct = 85 # Predicted percentage enabling the watch loop.
}
$Cfg.OutDir = Join-Path $Cfg.Root 'out' # Lane hand-backs, logs, generated prompts and reports.
$Cfg.TestsDir = Join-Path $Cfg.Root 'tests' # Throwaway per-lane data and probe working directories.
$Cfg.StateFile = Join-Path $Cfg.KitDir 'heartbeat-state.json' # Persistent state; never ship a real state file.
$Cfg.EventsLog = Join-Path $Cfg.KitDir 'events.log' # UTC pipe-delimited lifecycle events.
$Cfg.HeartbeatLog = Join-Path $Cfg.KitDir 'heartbeat.log' # UTC tick summaries and diagnostic messages.
$Cfg.LanesFile = Join-Path $Cfg.KitDir 'lanes.json' # Ordered array of lane definitions (copy lanes.example.json).
$Cfg.LedgerDir = Join-Path $Cfg.Root 'ledger' # Local quota CSV; hourly publication requires EnableRepoActions.
$Cfg.CargoTargetDir = Join-Path $Cfg.Root '_target' # Shared offline Cargo build cache granted to lanes.
