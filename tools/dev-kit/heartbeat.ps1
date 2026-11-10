# Configurable lane heartbeat. Runtime files belong to configured folders only.
# Repo mutations/network Git operations and process stopping are separate opt-ins.
param([switch]$DryRun, [string]$ConfigPath = (Join-Path $PSScriptRoot 'config.ps1'))
$ErrorActionPreference = 'Stop'
$ConfigPath = (Resolve-Path -LiteralPath $ConfigPath).Path
. $ConfigPath
if ($Cfg -isnot [hashtable]) { throw 'Config must define $Cfg as a hashtable.' }
foreach ($key in 'EnableRepoActions','EnableProcessStop','EnableClaudeProbe','EnableCodexProbe') {
  if ($Cfg[$key] -isnot [bool]) { throw "$key must be a Boolean." }
}
foreach ($key in 'MaxAlive','MaxAliveHigh','LookAheadMin','WatchSampleSec','TickMinutes','GuardMinutes') {
  if ([double]$Cfg[$key] -le 0) { throw "$key must be positive." }
}
foreach ($key in 'StopAtPct','HighMeterPct','WatchStartPct','LiftBelowPct','WatchPredictPct') {
  if ([double]$Cfg[$key] -le 0 -or [double]$Cfg[$key] -gt 100) { throw "$key must be in (0,100]." }
}
function Assert-NoReparse([string]$path) {
  $walk = [IO.Path]::GetFullPath($path)
  while ($walk) {
    if ((Test-Path -LiteralPath $walk) -and ((Get-Item -LiteralPath $walk -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw "Reparse point not allowed: $walk" }
    $walk = Split-Path -Parent $walk
  }
}
function Assert-Child([string]$path, [string]$parent) {
  $full = [IO.Path]::GetFullPath($path)
  $prefix = [IO.Path]::GetFullPath($parent).TrimEnd('\') + '\'
  if (-not $full.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { throw "Path must stay below configured folder: $path" }
  Assert-NoReparse $full
  return $full
}
foreach ($key in 'Root','KitDir','MainCheckout','TestsDir','CodexSessionsDir','CargoTargetDir') {
  if (-not [IO.Path]::IsPathRooted([string]$Cfg[$key])) { throw "$key must be absolute." }
  Assert-NoReparse $Cfg[$key]
}
foreach ($key in 'OutDir','LedgerDir','CargoTargetDir') { $Cfg[$key] = Assert-Child $Cfg[$key] $Cfg.Root }
foreach ($key in 'StateFile','EventsLog','HeartbeatLog','LanesFile') { $Cfg[$key] = Assert-Child $Cfg[$key] $Cfg.KitDir }
if ($Cfg.Remote -notmatch '^[A-Za-z0-9][A-Za-z0-9_.-]*$') { throw 'Invalid remote name.' }
function Assert-Ref([string]$ref) {
  if ($ref -notmatch '^[A-Za-z0-9][A-Za-z0-9_./-]*$' -or $ref -match '\.\.|//|/\.|\.$|/$|\.lock(?:/|$)') { throw "Invalid Git ref: $ref" }
}
Assert-Ref $Cfg.MainBranch
Assert-Ref ($Cfg.BranchPrefix + 'example')
if ($Cfg.LedgerRepoPath -notmatch '^[A-Za-z0-9_./-]+$' -or $Cfg.LedgerRepoPath -match '(^|/)\.\.(/|$)' -or [IO.Path]::IsPathRooted($Cfg.LedgerRepoPath)) { throw 'Invalid relative ledger repository path.' }
$root = $Cfg.Root; $hb = $Cfg.KitDir; $out = $Cfg.OutDir
$statePath = $Cfg.StateFile; $logPath = $Cfg.HeartbeatLog; $evPath = $Cfg.EventsLog
$LookAheadMin = [double]$Cfg.LookAheadMin; $maxAlive = [int]$Cfg.MaxAlive
$mainRef = "$($Cfg.Remote)/$($Cfg.MainBranch)"
# Read-only Git queries remain available with actions off; ALL Git is skipped in DryRun.
function Invoke-KitGit {
  $ErrorActionPreference = 'Continue' # Native Git diagnostics do not imply a failed command.
  $gitArgs = @($args)
  if ($gitArgs.Count -lt 3 -or $gitArgs[0] -ne '-C') { throw 'Git requires an explicit configured checkout.' }
  $cwd = [IO.Path]::GetFullPath([string]$gitArgs[1])
  if ($cwd -ne [IO.Path]::GetFullPath($Cfg.MainCheckout)) { $null = Assert-Child $cwd $root }
  $readOnly = $gitArgs[2] -in 'status','log','diff','rev-parse','rev-list'
  if ($DryRun -or (-not $Cfg.EnableRepoActions -and -not $readOnly)) {
    Write-Host ("{0} git {1}" -f $(if ($DryRun) { 'DRYRUN' } else { 'DISABLED' }), ($gitArgs -join ' '))
    $global:LASTEXITCODE = 0; return
  }
  & git.exe @gitArgs
  $global:LASTEXITCODE = $LASTEXITCODE
}
function Get-LaneProcesses([string]$lane = '') {
  if ($DryRun) { return @() }
  $names = @($lanes.Keys); if ($lane) { $names = @($lane) }
  foreach ($proc in @(Get-CimInstance Win32_Process -Filter "Name='codex.exe'")) {
    if ($proc.CommandLine -notmatch '\s+exec\s+') { continue }
    foreach ($name in $names) {
      $wt = [regex]::Escape((Join-Path $root $name))
      if ($proc.CommandLine -match ('(?i)(?:^|\s)-C\s+(?:"' + $wt + '"|' + $wt + ')(?=\s|$)')) { $proc; break }
    }
  }
}
function Now { (Get-Date).ToUniversalTime() }
function NowS { (Now).ToString('yyyy-MM-dd HH:mm:ss') + 'Z' }
function Write-HbLog($text) {
  if ($DryRun) { Write-Host "DRYRUN diagnostic $text" }
  else { Add-Content -LiteralPath $logPath $text }
}
function Ev($lane, $kind, $text) {
  $line = "{0}|{1}|{2}|{3}" -f (NowS), $lane, $kind, $text
  if ($DryRun) { Write-Host "DRYRUN event $line" } else { Add-Content -LiteralPath $evPath $line }
}
function ToHash($o) {
  if ($o -is [System.Management.Automation.PSCustomObject]) { $h = @{}; foreach ($p in $o.PSObject.Properties) { $h[$p.Name] = ToHash $p.Value }; return $h }
  if ($o -is [System.Array]) { return @($o | ForEach-Object { ToHash $_ }) }
  return $o
}
function Save-State {
  if ($DryRun) { return }
  $tmp = "$statePath.tmp"; ($script:state | ConvertTo-Json -Depth 12) | Set-Content -LiteralPath $tmp -Encoding utf8
  Move-Item -LiteralPath $tmp -Destination $statePath -Force
}

# JSON array order is launch priority; no project-specific lane table is shipped.
$lanes = [ordered]@{}
$laneEntries = Get-Content -LiteralPath $Cfg.LanesFile -Raw | ConvertFrom-Json
foreach ($entry in $laneEntries) {
  if (-not $entry) { continue }
  $ld = ToHash $entry; $name = [string]$ld.name
  if ($name -notmatch '^[A-Za-z0-9][A-Za-z0-9_-]*$' -or $name -match '^(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])$') { throw "Invalid lane name: $name" }
  if ($lanes.Contains($name)) { throw "Duplicate lane: $name" }
  $lanePath = Assert-Child (Join-Path $root $name) $root
  foreach ($reserved in @($Cfg.OutDir, $Cfg.TestsDir, $Cfg.KitDir, $Cfg.CargoTargetDir, $Cfg.LedgerDir, (Join-Path $root 'ledger-wt'), $Cfg.MainCheckout)) {
    if ($lanePath -eq [IO.Path]::GetFullPath($reserved)) { throw "Lane overlaps a configured runtime directory: $name" }
  }
  if (-not $ld.branch) { $ld.branch = $Cfg.BranchPrefix + $name }
  if (-not $ld.base) { $ld.base = $mainRef }
  Assert-Ref $ld.branch; Assert-Ref $ld.base
  foreach ($ref in @($ld.merge)) { if ($ref) { Assert-Ref $ref } }
  $ld.br = $ld.branch # Internal alias retained for state/report compatibility.
  if (-not $ld.model) { $ld.model = $Cfg.DefaultModel }; if (-not $ld.effort) { $ld.effort = $Cfg.DefaultEffort }
  if ($ld.model -notmatch '^[A-Za-z0-9_.:-]+$' -or $ld.effort -notin 'minimal','low','medium','high','xhigh','max') { throw "Invalid model/effort: $name" }
  foreach ($flag in 'review','fix') { if ($ld.ContainsKey($flag) -and $ld[$flag] -isnot [bool]) { throw "Lane $flag must be Boolean: $name" } }
  if ($ld.promptFile) { $ld.promptFile = Assert-Child (Join-Path $hb $ld.promptFile) $hb }
  $lanes[$name] = $ld
}
foreach ($name in $lanes.Keys) {
  foreach ($dep in @($lanes[$name].deps) + @($lanes[$name].gate)) { if ($dep -and -not $lanes.Contains([string]$dep)) { throw "Unknown dependency/gate '$dep' in $name" } }
}
if (-not $DryRun) {
  foreach ($dir in @($hb, $out, $Cfg.TestsDir, $Cfg.LedgerDir)) { New-Item -ItemType Directory -Force $dir | Out-Null }
}

# ---- lock (one heartbeat at a time)
$lockFs = $null
if (-not $DryRun) {
  try { $lockFs = [System.IO.File]::Open("$hb\lock", 'OpenOrCreate', 'ReadWrite', 'None') }
  catch { Write-HbLog ("{0} skipped: another heartbeat is running" -f (NowS)); return }
}

try {
  # ---- state
  if (Test-Path $statePath) { $state = ToHash (Get-Content $statePath -Raw | ConvertFrom-Json) } else { $state = @{} }
  if (-not $state.lanes) { $state.lanes = @{} }
  if (-not $state.limit) { $state.limit = @{ active=$false; resetUtc=$null; note='' } }
  foreach ($k in $lanes.Keys) { if (-not $state.lanes[$k]) { $state.lanes[$k] = @{ status='pending'; legacy=$false; followupDone=$false } } }

  # ---- process table
  $procs = @(Get-LaneProcesses)
  function Alive($l) { return (@(Get-LaneProcesses $l).Count -gt 0) }
  function PathsFor($l, $tag) { return @{ md="$out\$tag.md"; log="$out\$tag.log"; err="$out\$tag.err" } }
  function Tail-Text($path, $n) {
    # last ~120 KB of a (UTF-16) log, last $n lines; fast, no full read
    if (-not (Test-Path $path)) { return '' }
    try {
      $fs = [System.IO.File]::Open($path, 'Open', 'Read', 'ReadWrite')
      try {
        $len = $fs.Length; $take = [Math]::Min($len, 120000); if ($take % 2 -eq 1) { $take-- }
        $fs.Seek($len - $take, 'Begin') | Out-Null; $buf = New-Object byte[] $take; $null = $fs.Read($buf, 0, $take)
        # logs can be UTF-16 with plain-ASCII lines appended: drop NUL bytes, then decode as UTF-8 (apostrophes may show as junk; patterns avoid them)
        $clean = New-Object System.Collections.Generic.List[byte]; foreach ($x in $buf) { if ($x -ne 0 -and $x -ne 0xFF -and $x -ne 0xFE) { $clean.Add($x) } }
        $txt = [System.Text.Encoding]::UTF8.GetString($clean.ToArray())
        return (($txt -split "`r?`n") | Select-Object -Last $n) -join "`n"
      } finally { $fs.Close() }
    } catch { return '' }
  }

  function PausedWindow {
    # true while the stop line is on for a window whose stored reset time is known, not more than 5.5 hours away, and not yet passed by more than 60 s: nothing runs, so probes tell nothing new. Missing or implausible reset time => false (old behaviour).
    try {
      $slx = $state.stopLine
      if ($slx -and $slx.active -and $slx.resetEpoch -and ([string]$slx.reason -notmatch 'predicted' -or [long]$state.liftedForWindow -eq [long]$slx.resetEpoch -or [double]$state.rate.p -ge $Cfg.StopAtPct)) { $rem = [long]$slx.resetEpoch - [DateTimeOffset]::UtcNow.ToUnixTimeSeconds(); return ($rem -gt -60 -and $rem -le 19800) }
    } catch { }
    return $false
  }
  function ModelNote($l) {
    $ls = $state.lanes[$l]; $mdl = [string]$lanes[$l].model; if (-not $mdl) { $mdl = $Cfg.DefaultModel }; $eff = [string]$lanes[$l].effort; if (-not $eff) { $eff = $Cfg.DefaultEffort }
    $now5 = ''; $nowW = ''; $at = ''; if ($state.rate) { $now5 = $state.rate.p; $nowW = $state.rate.w; $at = ([string]$state.rate.ts); if ($at.Length -ge 19) { $at = $at.Substring(11, 8) + 'Z' } }
    return (" | model={0} effort={1} | Codex meter 5h {2}% -> {3}%, weekly {4}% -> {5}% (last reading {6})" -f $mdl, $eff, $ls.meterBefore5h, $now5, $ls.meterBeforeW, $nowW, $at)
  }
  function Read-CodexRate {
    # newest 'codex' rate_limits event across ALL recent Codex session files (not only the active lane's)
    $best = $null
    $sf = Get-ChildItem $Cfg.CodexSessionsDir -Recurse -Filter *.jsonl -ErrorAction SilentlyContinue | Where-Object { $_.LastWriteTime -gt (Get-Date).AddHours(-6) } | Sort-Object LastWriteTime -Descending | Select-Object -First 12
    foreach ($s in $sf) {
      try {
        $tt = Tail-Text $s.FullName 4000
        $ln = @($tt -split "`n" | Where-Object { $_ -match '"limit_id"\s*:\s*"codex"' -and $_ -match '"primary"\s*:\s*\{' }) | Select-Object -Last 1
        if ($ln) { $o = $ln | ConvertFrom-Json; $c = @{ f = $s.Name; ts = [string]$o.timestamp; p = $o.payload.rate_limits.primary.used_percent; w = $o.payload.rate_limits.secondary.used_percent; pr = $o.payload.rate_limits.primary.resets_at; wr = $o.payload.rate_limits.secondary.resets_at; plan = [string]$o.payload.rate_limits.plan_type }; if (-not $best -or $c.ts -gt $best.ts) { $best = $c } }
      } catch { }
    }
    return $best
  }
  function Get-CodexRateFresh {
    # A stale reading may trigger one non-ephemeral low-effort probe when enabled.
    $r = Read-CodexRate
    $stale = $true; if ($r) { $stale = (((Now) - [datetime]::Parse($r.ts).ToUniversalTime()).TotalMinutes -gt 15) }
    if (PausedWindow) { $stale = $false }
    if ($stale -and -not $DryRun -and $Cfg.EnableCodexProbe) {
      try {
        $probeDir = Join-Path $Cfg.TestsDir 'codex-probe'
        New-Item -ItemType Directory -Force $probeDir | Out-Null
        Push-Location $probeDir
        try { & $Cfg.CodexCmd exec --json -m $Cfg.DefaultModel -c 'model_reasoning_effort="low"' -s read-only --skip-git-repo-check 'Reply with ok.' *> "$out\codex-probe.jsonl" }
        finally { Pop-Location }
        $r2 = Read-CodexRate; if ($r2) { $r2.probe = $true; $r = $r2 }
      } catch { Ev '-' 'attention' ('Codex probe failed: ' + $_.Exception.Message) }
    }
    return $r
  }
  function AliveNow($k) { return (@(Get-LaneProcesses $k).Count -gt 0) }
  function StopLaneClean($k) {
    # Separate opt-ins: without process stopping there can be no clean checkpoint
    # of a stopped writer, so keep it running and report the requested action.
    if ($DryRun -or -not $Cfg.EnableProcessStop) {
      Ev $k 'stop-disabled' 'would stop the lane tree; EnableProcessStop is false (or DryRun)'
      return $false
    }
    $ld = $lanes[$k]; $wt = "$root\$k"
    foreach ($pr in @(Get-LaneProcesses $k)) {
      & taskkill /PID $pr.ProcessId /T /F 2>&1 | Out-Null
      if ($LASTEXITCODE -ne 0) { Ev $k 'attention' 'taskkill failed; checkpoint/push skipped'; return $false }
    }
    Start-Sleep -Seconds 4
    if (AliveNow $k) { Ev $k 'attention' 'lane is still alive; checkpoint/push skipped'; return $false }
    if (-not $ld.review -and (Test-Path $wt)) {
      if ($Cfg.EnableRepoActions) { Checkpoint $k; Invoke-KitGit -C $wt push -u $Cfg.Remote $ld.br 2>&1 | Out-Null }
      else { Ev $k 'repo-actions-disabled' 'stopped; checkpoint and push require EnableRepoActions' }
    }
    $state.lanes[$k].status = 'paused-stopline'; $state.lanes[$k].resume = $true
    return $true
  }
  # Predict used percentage from the same quota window; never use a banked reset.
  function RateHistAdd($r) {
    if (-not $r -or -not $r.ts) { return }
    $hist = @($state.rateHist | Where-Object { $_ })
    $e = [DateTime]::Parse([string]$r.ts).ToUniversalTime().ToString('o')
    if ($hist.Count -gt 0 -and [string]$hist[-1].t -eq $e) { return }
    $hist += @{ t = $e; p = [double]$r.p; pr = [long]$r.pr }
    if ($hist.Count -gt 8) { $hist = @($hist[($hist.Count - 8)..($hist.Count - 1)]) }
    $state.rateHist = $hist
  }
  function BurnPerMin($r) {
    # points of the 5-hour meter per minute, from the newest stored reading and the oldest one of the same window that is 0.5 to 20 minutes older; 0 when unknown or the newest reading is older than 4 minutes (nobody is burning)
    if (-not $r -or -not $r.ts) { return 0.0 }
    $hist = @($state.rateHist | Where-Object { $_ -and [long]$_.pr -eq [long]$r.pr })
    if ($hist.Count -lt 2) { return 0.0 }
    $lastT = [DateTime]::Parse([string]$hist[-1].t).ToUniversalTime()
    if (((Now) - $lastT).TotalMinutes -gt 4) { return 0.0 }
    $base = $null
    for ($i = 0; $i -lt $hist.Count - 1; $i++) { $dm0 = ($lastT - [DateTime]::Parse([string]$hist[$i].t).ToUniversalTime()).TotalMinutes; if ($dm0 -ge 0.5 -and $dm0 -le 20) { $base = $hist[$i]; break } }
    if (-not $base) { return 0.0 }
    $dm = ($lastT - [DateTime]::Parse([string]$base.t).ToUniversalTime()).TotalMinutes
    $b = ([double]$hist[-1].p - [double]$base.p) / $dm
    if ($b -lt 0) { return 0.0 } else { return [math]::Round($b, 2) }
  }
  function EnforceStopLine($r) {
    if ($DryRun -or -not $r) { return }
    $nowEpoch = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    RateHistAdd $r; $burn = BurnPerMin $r; $pred = [double]$r.p + $burn * $LookAheadMin; $reached = ($r.p -ge $Cfg.StopAtPct) -or ($burn -gt 0 -and $pred -ge $Cfg.StopAtPct)
    $sl = $state.stopLine; if (-not $sl) { $sl = @{ active = $false }; $state.stopLine = $sl }
    # the stop-line guard (separate scheduled task, stopline-guard.ps1) may have stopped lanes while this tick was busy: adopt its record
    $gfp = "$hb\stopline-guard.json"
    if (Test-Path $gfp) {
      try {
        $gg = ToHash (Get-Content $gfp -Raw | ConvertFrom-Json)
        if ($gg.active -and [long]$gg.resetEpoch -gt $nowEpoch -and -not $sl.active) {
          $sl.active = $true; $sl.resetEpoch = [long]$gg.resetEpoch; $sl.resetUtc = $gg.resetUtc; $sl.since = $gg.since; $sl.reading = $gg.reading; $sl.overshoot = [math]::Max(0, [math]::Round([double]$gg.reading - $Cfg.StopAtPct, 1)); $sl.burnPerMin = $gg.burnPerMin; $sl.predicted12 = $gg.predicted12; $sl.reason = "guard-$($gg.reason)"
          foreach ($gk in @($gg.lanes)) { if ($state.lanes[$gk]) { $state.lanes[$gk].status = 'paused-stopline'; $state.lanes[$gk].resume = $true } }
          Ev '-' 'stopline-pause' ("adopted the stop-line guard's record: lanes {0} were stopped by the guard at {1}% ({2}); everything resumes at the window reset {3}" -f (@($gg.lanes) -join ','), $gg.reading, $gg.reason, $gg.resetUtc)
        }
      } catch { }
    }
    if ($reached -and $r.pr -gt $nowEpoch) {
      if (-not $sl.active) {
        $sl.active = $true; $sl.resetEpoch = $r.pr; $sl.resetUtc = [DateTimeOffset]::FromUnixTimeSeconds([long]$r.pr).UtcDateTime.ToString('yyyy-MM-dd HH:mm:ss') + 'Z'; $sl.since = NowS; $sl.reading = $r.p; $sl.overshoot = [math]::Max(0, [math]::Round([double]$r.p - $Cfg.StopAtPct, 1)); $sl.burnPerMin = $burn; $sl.predicted12 = [math]::Round($pred, 1); $sl.reason = $(if ($r.p -ge $Cfg.StopAtPct) { 'measured' } else { 'predicted' })
        Ev '-' 'stopline-pause' ("Codex 5-hour window at {0}% used (reading {1}); stop line {7}% reached ({4}: burn {5} points/min, predicted {6}% in {8} minutes; overshoot {2}): no new starts; process stop enabled={9}, repo actions enabled={10}; reset {3}. No banked reset is used." -f $r.p, $r.ts, $sl.overshoot, $sl.resetUtc, $sl.reason, $burn, $sl.predicted12, $Cfg.StopAtPct, $LookAheadMin, $Cfg.EnableProcessStop, $Cfg.EnableRepoActions)
      }
      foreach ($k in @($lanes.Keys)) { $st = $state.lanes[$k].status; if (($st -eq 'running' -or $st -eq 'followup-running') -and (AliveNow $k)) { if (StopLaneClean $k) { Ev $k 'paused-at-stopline' (("lane stopped at meter {0}%; repo actions enabled={1}; resumes at the window reset" -f $r.p, $Cfg.EnableRepoActions) + (ModelNote $k)) } } }
    } elseif ($sl.active -and ($nowEpoch -ge [long]$sl.resetEpoch -or [long]$r.pr -ne [long]$sl.resetEpoch)) {
      $sl.active = $false
      if (Test-Path "$hb\stopline-guard.json") { try { Remove-Item "$hb\stopline-guard.json" -Force } catch { } }
      foreach ($k in @($lanes.Keys)) { if ($state.lanes[$k].status -eq 'paused-stopline') { $state.lanes[$k].status = 'pending'; $state.lanes[$k].resume = $true } }
      Ev '-' 'stopline-resume' ("Codex 5-hour window is at {0}% used again / its reset time has passed; lanes paused at the stop line are resumed" -f $r.p)
    } elseif ($sl.active -and [string]$sl.reason -match 'predicted' -and [long]$r.pr -eq [long]$sl.resetEpoch -and [long]$state.liftedForWindow -ne [long]$r.pr -and [double]$r.p -lt $Cfg.LiftBelowPct -and ([double]$r.p + $burn * $LookAheadMin) -lt $Cfg.StopAtPct -and -not (@(Get-LaneProcesses).Count)) {
      $state.liftedForWindow = [long]$r.pr   # at most ONE predicted lift per window: the first lift (19:33Z) gave S two minutes of work and 5 points of meter before the line fired again at 89%, i.e. mostly relaunch cost
      $sl.active = $false
      if (Test-Path "$hb\stopline-guard.json") { try { Remove-Item "$hb\stopline-guard.json" -Force } catch { } }
      foreach ($k in @($lanes.Keys)) { if ($state.lanes[$k].status -eq 'paused-stopline') { $state.lanes[$k].status = 'pending'; $state.lanes[$k].resume = $true } }
      Ev '-' 'stopline-resume' ("predicted stop line lifted once this window: no lane runs, used {0}% < {3}%, burn {1}/min, used + {2} x burn < {4}%; high-meter max lanes {5} from {6}%. A measured hold stays until reset." -f $r.p, $burn, $LookAheadMin, $Cfg.LiftBelowPct, $Cfg.StopAtPct, $Cfg.MaxAliveHigh, $Cfg.HighMeterPct)
    }
  }  # ---- (b) limit detection from newest logs (only lines near the end of recently written logs)
  $limRx = 'hit your usage limit|usage limit (has been )?reached|reached your .{0,30}limit|rate limit exceeded|rate_limit_exceeded|insufficient_quota|429 Too Many Requests'
  $found = $null
  $cut = (Get-Date).AddHours(-2)
  if ($state.limitIgnoreBefore) { $ib = [datetime]::Parse([string]$state.limitIgnoreBefore).ToLocalTime(); if ($ib -gt $cut) { $cut = $ib } }
  foreach ($d in @($out)) {
    if (-not (Test-Path $d)) { continue }
    foreach ($f in Get-ChildItem "$d\*" -File -Include *.log,*.err -ErrorAction SilentlyContinue | Where-Object { $_.LastWriteTime -gt $cut }) {
      $t = Tail-Text $f.FullName 40
      if ($t -match $limRx) {
        $tl = @($t -split "`n"); $ix = -1; for ($k = 0; $k -lt $tl.Count; $k++) { if ($tl[$k] -match $limRx) { $ix = $k } }
        $line = $tl[$ix]; if ($ix + 1 -lt $tl.Count) { $line = $line + ' ' + $tl[$ix + 1] }
        $reset = $null
        if ($line -match '(?i)try again (?:at|after|on)\s+([^.]+?)(?:\.|$)') { $dt = [datetime]::MinValue; if ([datetime]::TryParse($Matches[1].Trim(), [ref]$dt)) { $dt = $f.LastWriteTime.Date + $dt.TimeOfDay; if ($dt -lt $f.LastWriteTime) { $dt = $dt.AddDays(1) }; $reset = $dt.ToUniversalTime() } }
        if (-not $reset -and $line -match '(?i)(?:in|after)\s+(?:(\d+)\s*h\w*)?\s*(?:(\d+)\s*m\w*)?') { $h = [int]($Matches[1]); $m = [int]($Matches[2]); if ($h + $m -gt 0) { $reset = (Now).AddHours($h).AddMinutes($m) } }
        if (-not $reset) { $reset = (Now).AddMinutes(60) }
        $found = @{ file=$f.Name; line=($line.Trim() -replace '\s+',' '); reset=$reset; seen=$f.LastWriteTime.ToUniversalTime() }
      }
    }
  }
  $wasActive = [bool]$state.limit.active
  if ($found -and $found.reset -gt (Now)) {
    $prev = $null; if ($state.limit.resetUtc) { $prev = [datetime]::Parse($state.limit.resetUtc).ToUniversalTime() }
    if (-not $wasActive -or ($found.reset -gt $prev)) {
      $state.limit = @{ active=$true; resetUtc=$found.reset.ToString('yyyy-MM-dd HH:mm:ss') + 'Z'; note=("{0}: {1}" -f $found.file, $found.line) }
      if (-not $wasActive) { Ev '-' 'limit-pause' ("reset {0}; {1}" -f $state.limit.resetUtc, $state.limit.note) }
    }
  }
  if ($state.limit.active -and $state.limit.resetUtc -and ([datetime]::Parse($state.limit.resetUtc).ToUniversalTime() -le (Now))) {
    $state.limit.active = $false; Ev '-' 'limit-resume' ("reset time {0} has passed" -f $state.limit.resetUtc)
  }
  $limitActive = [bool]$state.limit.active
  # ---- Codex's own rate readings (newest rate_limits event in the session files): 5-hour and weekly percent.
  $rate = Get-CodexRateFresh
  try {
    $sf = Get-ChildItem $Cfg.CodexSessionsDir -Recurse -Filter *.jsonl -ErrorAction SilentlyContinue | Where-Object { $_.LastWriteTime -gt (Get-Date).AddHours(-6) } | Sort-Object LastWriteTime -Descending | Select-Object -First 6
    foreach ($s in $sf) {
      $tt = Tail-Text $s.FullName 60
      $ln = @($tt -split "`n" | Where-Object { $_ -match '"limit_id"\s*:\s*"codex"' -and $_ -match '"primary"\s*:\s*\{' }) | Select-Object -Last 1
      if ($ln) { $o = $ln | ConvertFrom-Json; $c = @{ f = $s.Name; ts = [string]$o.timestamp; p = $o.payload.rate_limits.primary.used_percent; w = $o.payload.rate_limits.secondary.used_percent; pr = $o.payload.rate_limits.primary.resets_at; wr = $o.payload.rate_limits.secondary.resets_at }; if (-not $rate -or $c.ts -gt $rate.ts) { $rate = $c } }
    }
  } catch { }
  if ($rate) {
    $state.rate = $rate
    $nowEpoch = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    if ($rate.w -ge 100 -and $rate.wr -gt $nowEpoch) {
      if (-not $state.weeklyLimit) { $state.weeklyLimit = $true; Ev '-' 'weekly-limit' ("Codex WEEKLY limit is at {0}% (reading {1}); heartbeat PAUSED, no banked reset is used or assumed; the operator decides" -f $rate.w, $rate.ts) }
      $limitActive = $true
    } elseif ($state.weeklyLimit) { $state.weeklyLimit = $false; Ev '-' 'weekly-resume' ("weekly reading is {0}% again (reading {1}); heartbeat resumes" -f $rate.w, $rate.ts) }
    if ($rate.p -ge 99.9 -and $rate.pr -gt $nowEpoch -and -not $state.limit.active) {
      $rs = [DateTimeOffset]::FromUnixTimeSeconds([long]$rate.pr).UtcDateTime.ToString('yyyy-MM-dd HH:mm:ss') + 'Z'
      $state.limit = @{ active = $true; resetUtc = $rs; note = ("rate_limits reading {0}: 5-hour window {1}% used" -f $rate.ts, $rate.p) }
      Ev '-' 'limit-pause' ("5-hour window at {0}% by Codex's own reading; reset {1}" -f $rate.p, $rs); $limitActive = $true
    }
  }

  # Optional noninteractive Claude probe returns account-wide utilization.
  # Refresh after five minutes; while paused, allow a cached reading for two hours.
  # utilization is returned as a fraction (0.28 = 28%); we store percent = utilization x 100.
  $probeNote = ''
  try {
    $crf = "$hb\claude-rate.json"; $needs = $true
    if (Test-Path $crf) { try { $needs = (((Now) - ([datetime]::Parse((Get-Content $crf -Raw | ConvertFrom-Json).ts)).ToUniversalTime()).TotalMinutes -gt 5) } catch { $needs = $true } }
    # while everything is paused at the stop line the Claude meter is not needed either: skip the probe unless the stored reading is older than 2 hours; at the reset (+1 minute) PausedWindow ends and the normal 5-minute rule resumes
    if ($needs -and (PausedWindow) -and (Test-Path $crf)) { try { if ((((Now) - ([datetime]::Parse((Get-Content $crf -Raw | ConvertFrom-Json).ts)).ToUniversalTime()).TotalMinutes -le 120)) { $needs = $false } } catch { } }
    if ($needs -and -not $DryRun -and $Cfg.EnableClaudeProbe -and $Cfg.ClaudeCmd) {
      $cl = $Cfg.ClaudeCmd
      $pd = (Join-Path $Cfg.TestsDir 'claude-probe'); New-Item -ItemType Directory -Force $pd | Out-Null
      $po = "$out\claude-probe-stream.jsonl"
      Push-Location $pd
      try { & $cl -p ok --model haiku --tools '' --strict-mcp-config --disable-slash-commands --no-session-persistence --system-prompt 'Reply with ok.' --output-format stream-json --verbose *> $po }
      finally { Pop-Location }
      $evl = Get-Content $po -ErrorAction SilentlyContinue | Where-Object { $_ -match 'unifiedWindows' } | Select-Object -Last 1
      if ($evl) {
        $ei = ($evl | ConvertFrom-Json).rate_limit_info.unifiedWindows
        $raw = Get-Content $po -Raw
        function LastNum($rx) { $m = [regex]::Matches($raw, $rx); if ($m.Count) { return [int]$m[$m.Count - 1].Groups[1].Value } else { return 0 } }
        $tin = LastNum '"input_tokens":(\d+)'; $tcc = LastNum '"cache_creation_input_tokens":(\d+)'; $tcr = LastNum '"cache_read_input_tokens":(\d+)'; $tout = LastNum '"output_tokens":(\d+)'
        $probeNote = ('one tiny claude -p probe (haiku, no tools): input {0}, cache-creation {1}, cache-read {2}, output {3} tokens' -f $tin, $tcc, $tcr, $tout)
        $obj = [ordered]@{ ts = (Now).ToString('o'); source = 'claude -p probe, rate_limit_event (utilization x 100)'; five_hour = [ordered]@{ used_percentage = [math]::Round([double]$ei.five_hour.utilization * 100, 1); raw_utilization = [double]$ei.five_hour.utilization; resets_at = $ei.five_hour.resetsAt }; seven_day = [ordered]@{ used_percentage = [math]::Round([double]$ei.seven_day.utilization * 100, 1); raw_utilization = [double]$ei.seven_day.utilization; resets_at = $ei.seven_day.resetsAt }; probe_usage = $probeNote }
        $tmp = "$crf.tmp"; ($obj | ConvertTo-Json -Depth 4) | Set-Content $tmp -Encoding UTF8; Move-Item $tmp $crf -Force
      } else { Write-HbLog ("{0} claude probe returned no rate_limit_event (see {1})" -f (NowS), $po) }
    }
  } catch { Write-HbLog ("{0} claude probe failed: {1}" -f (NowS), ($_.Exception.Message -replace '\s+', ' ')) }
  # ---- Claude quota control: 100% weekly => pause annotation; 60% => warning.
  try {
    if (Test-Path "$hb\claude-rate.json") {
      $cj0 = Get-Content "$hb\claude-rate.json" -Raw | ConvertFrom-Json
      $age0 = (Now) - [datetime]::Parse($cj0.ts).ToUniversalTime(); $fresh0 = ($age0.TotalMinutes -le 30)
      $pw = [double]$cj0.seven_day.used_percentage; $p5 = [double]$cj0.five_hour.used_percentage
      $wr = [DateTimeOffset]::FromUnixTimeSeconds([long]$cj0.seven_day.resets_at).UtcDateTime.ToString('yyyy-MM-dd HH:mm:ss') + 'Z'; $r5s = [DateTimeOffset]::FromUnixTimeSeconds([long]$cj0.five_hour.resets_at).UtcDateTime.ToString('yyyy-MM-dd HH:mm:ss') + 'Z'
      $cq = $state.claude_quota_state; if (-not $cq) { $cq = @{ claude_pause = $false; claude_warn = $false } }
      if ($fresh0) {
        $np = ($pw -ge 100); $nw = ($pw -ge 60) # Informational warning; pause only when exhausted.
        if ($np -ne [bool]$cq.claude_pause) { Ev '-' $(if ($np) { 'claude-pause' } else { 'claude-resume' }) ("Claude WEEKLY plan meter {0}% (pause line 100% when fully used; 60% is information only); weekly resets {1}; 5-hour {2}% resets {3}. {4}" -f $pw, $wr, $p5, $r5s, $(if ($np) { 'claude_pause=true: stop Claude-side work until the weekly reset.' } else { 'claude_pause=false.' })) }
        elseif (-not $np -and ($nw -ne [bool]$cq.claude_warn)) { Ev '-' $(if ($nw) { 'claude-warn' } else { 'claude-warn-clear' }) ("Claude WEEKLY plan meter {0}% (warning at 60%, pause at 100%); weekly resets {1}" -f $pw, $wr) }
        $cq.claude_pause = $np; $cq.claude_warn = $nw
      }
      $cq.weekly_used_pct = $pw; $cq.five_hour_used_pct = $p5; $cq.weekly_resets_utc = $wr; $cq.five_hour_resets_utc = $r5s; $cq.reading_ts = [string]$cj0.ts; $cq.fresh = $fresh0; $cq.source = [string]$cj0.source
      $state.claude_quota_state = $cq
    }
  } catch { Write-HbLog ("{0} claude quota control failed: {1}" -f (NowS), ($_.Exception.Message -replace '\s+', ' ')) }  # ---- ledger rows: one row per tick for each meter we can read, appended to a LOCAL csv outside the repo.
  # Columns match docs/ledger/quota-readings.csv: time_utc,tool,account,limit,reading,resets_utc,read_by,source,notes
  if (-not $DryRun) { try {
    $ledgerDir = $Cfg.LedgerDir; New-Item -ItemType Directory -Force $ledgerDir | Out-Null
    $csv = "$ledgerDir\quota-readings-pc.csv"
    if (-not (Test-Path $csv)) { Set-Content $csv 'time_utc,tool,account,limit,reading,resets_utc,read_by,source,notes' -Encoding UTF8 }
    function Q($v) { '"' + ([string]$v).Replace('"', '""') + '"' }
    $nowT = (Now).ToString('yyyy-MM-ddTHH:mm') + 'Z'
    if ($rate) {
      $age = (Now) - [datetime]::Parse($rate.ts).ToUniversalTime()
      $stale = ''; if ($age.TotalMinutes -gt 15) { $stale = 'stale (newest Codex event is {0} min old)' -f [int]$age.TotalMinutes }
      $r5 = [DateTimeOffset]::FromUnixTimeSeconds([long]$rate.pr).UtcDateTime.ToString('yyyy-MM-ddTHH:mm:ss') + 'Z'; $rw = [DateTimeOffset]::FromUnixTimeSeconds([long]$rate.wr).UtcDateTime.ToString('yyyy-MM-ddTHH:mm:ss') + 'Z'
      $cells = @($nowT, 'Codex', $Cfg.LedgerAccount, 'five-hour window and weekly limit', ('5-hour {0}% used, weekly {1}% used' -f $rate.p, $rate.w), ('5-hour {0}; weekly {1}' -f $r5, $rw), $Cfg.LedgerReader, ('heartbeat tick, Codex rate_limits event {0} at {1}{2}' -f $rate.f, $rate.ts, $(if ($rate.probe) { ' (read via a tiny codex exec probe because no session event was newer than 15 minutes)' } else { '' })), $stale)
    } else {
      $cells = @($nowT, 'Codex', $Cfg.LedgerAccount, 'five-hour window and weekly limit', 'no Codex rate_limits event found in the last 6 hours', 'not recorded', $Cfg.LedgerReader, ([string]$Cfg.CodexSessionsDir), 'stale')
    }
    Add-Content $csv (($cells | ForEach-Object { Q $_ }) -join ',') -Encoding UTF8
    if ($rate -and $rate.probe) { $cx = @($nowT, 'Codex', $Cfg.LedgerAccount, 'usage by the heartbeat probe', 'Codex stale-meter probe; see its rollout for actual token usage', 'not applicable', $Cfg.LedgerReader, 'heartbeat tick, codex exec probe', ''); Add-Content $csv (($cx | ForEach-Object { Q $_ }) -join ',') -Encoding UTF8 }
    $cr = "$hb\claude-rate.json"
    if (Test-Path $cr) {
      $cj = Get-Content $cr -Raw | ConvertFrom-Json; $cage = (Now) - [datetime]::Parse($cj.ts).ToUniversalTime()
      $cs = ''; if ($cage.TotalMinutes -gt 15) { $cs = 'stale (newest Claude status-line reading is {0} min old)' -f [int]$cage.TotalMinutes }
      $c5 = [DateTimeOffset]::FromUnixTimeSeconds([long]$cj.five_hour.resets_at).UtcDateTime.ToString('yyyy-MM-ddTHH:mm:ss') + 'Z'; $cw = [DateTimeOffset]::FromUnixTimeSeconds([long]$cj.seven_day.resets_at).UtcDateTime.ToString('yyyy-MM-ddTHH:mm:ss') + 'Z'
      $cc = @($nowT, 'Claude', $Cfg.LedgerAccount, 'five-hour window and weekly limit', ('5-hour {0}% used, weekly {1}% used' -f $cj.five_hour.used_percentage, $cj.seven_day.used_percentage), ('5-hour {0}; weekly {1}' -f $c5, $cw), $Cfg.LedgerReader, $(if ($cj.source) { 'heartbeat tick, Claude ' + $cj.source + ' at ' + $cj.ts } else { 'heartbeat tick, Claude Code status-line rate_limits written at ' + $cj.ts }), $cs)
      Add-Content $csv (($cc | ForEach-Object { Q $_ }) -join ',') -Encoding UTF8
      if ($probeNote) { $cp = @($nowT, 'Claude', $Cfg.LedgerAccount, 'usage by the heartbeat probe', $probeNote, 'not applicable', $Cfg.LedgerReader, 'heartbeat tick, claude -p probe stream-json', ''); Add-Content $csv (($cp | ForEach-Object { Q $_ }) -join ',') -Encoding UTF8 }
    }
  } catch { Write-HbLog ("{0} ledger row failed: {1}" -f (NowS), $_.Exception.Message) } }
  try {
    if (-not $DryRun -and $Cfg.EnableRepoActions -and (Test-Path (Join-Path $Cfg.LedgerDir 'quota-readings-pc.csv'))) {
      $lg = $state.ledger; if (-not $lg) { $lg = @{ lastLine = 1; lastPushUtc = $null }; $state.ledger = $lg }
      $due = $true; if ($lg.lastPushUtc) { $due = (((Now) - [datetime]::Parse($lg.lastPushUtc).ToUniversalTime()).TotalMinutes -ge 60) }
      $all = @(Get-Content (Join-Path $Cfg.LedgerDir 'quota-readings-pc.csv') -Encoding UTF8)
      $newRows = @(); if ($all.Count -gt [int]$lg.lastLine) { $newRows = @($all[[int]$lg.lastLine..($all.Count - 1)]) }
      if ($due -and $newRows.Count -gt 0) {
        $lw = "$root\ledger-wt"; $lb = $Cfg.BranchPrefix + 'ledger-readings'
        Invoke-KitGit -C $Cfg.MainCheckout fetch $Cfg.Remote 2>&1 | Out-Null
        if (-not (Test-Path $lw)) {
          $remoteHas = [bool](Invoke-KitGit -C $Cfg.MainCheckout ls-remote --heads $Cfg.Remote $lb 2>$null)
          $base = $mainRef; if ($remoteHas) { $base = "$($Cfg.Remote)/$lb" }
          Invoke-KitGit -C $Cfg.MainCheckout worktree add -B $lb $lw $base 2>&1 | Out-Null
        }
        $mg = Invoke-KitGit -C $lw merge --no-edit $mainRef 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) {
          # keep both sides: only docs/ledger/quota-readings.csv may conflict (everyone appends rows at its end)
          $cf = @(Invoke-KitGit -C $lw diff --name-only --diff-filter=U 2>$null | Where-Object { $_ })
          if ($cf.Count -eq 1 -and $cf[0] -eq $Cfg.LedgerRepoPath) {
            $cp = (Join-Path $lw $Cfg.LedgerRepoPath); $seen = New-Object System.Collections.Generic.HashSet[string]; $keep = New-Object System.Collections.Generic.List[string]
            foreach ($ln in [System.IO.File]::ReadAllLines($cp)) { if ($ln -match '^(<<<<<<<|=======|>>>>>>>)') { continue }; if ($seen.Add($ln)) { $keep.Add($ln) } }
            [System.IO.File]::WriteAllText($cp, (($keep -join "`n") + "`n"), (New-Object System.Text.UTF8Encoding($false)))
            Invoke-KitGit -C $lw add -- $Cfg.LedgerRepoPath 2>&1 | Out-Null
            Invoke-KitGit -C $lw commit --no-edit 2>&1 | Out-Null
            Ev '-' 'ledger-merge' 'merge conflict in quota-readings.csv resolved by keeping both sides (duplicate lines removed)'
          } else { Invoke-KitGit -C $lw merge --abort 2>&1 | Out-Null; throw "ledger merge of $mainRef failed: $mg" }
        }
        $target = (Join-Path $lw $Cfg.LedgerRepoPath)
        if (-not (Test-Path $target)) { throw "Ledger target $($Cfg.LedgerRepoPath) not found on $mainRef" }
        $cur = [System.IO.File]::ReadAllText($target); $eol = "`n"; if ($cur.Contains("`r`n")) { $eol = "`r`n" }
        $prefix = ''; if (-not ($cur.EndsWith("`n"))) { $prefix = $eol }
        [System.IO.File]::AppendAllText($target, $prefix + (($newRows) -join $eol) + $eol, (New-Object System.Text.UTF8Encoding($false)))
        Invoke-KitGit -C $lw add -- $Cfg.LedgerRepoPath 2>&1 | Out-Null
        $lm = "Quota readings from the heartbeat ($($newRows.Count) rows)`n`nAccount: $($Cfg.LedgerAccount)`nReader: $($Cfg.LedgerReader)`nStale readings are marked; keep both sides on a CSV append conflict."
        $lmf = "$out\ledger.commitmsg.txt"; [System.IO.File]::WriteAllText($lmf, $lm, (New-Object System.Text.UTF8Encoding($false)))
        Invoke-KitGit -C $lw commit -F $lmf 2>&1 | Out-Null
        $okp = $false; foreach ($i in 0..3) { Invoke-KitGit -C $lw push -u $Cfg.Remote $lb 2>&1 | Out-Null; if ($LASTEXITCODE -eq 0) { $okp = $true; break }; Start-Sleep -Seconds ([math]::Pow(2, $i + 1)) }
        if ($okp) { $lg.lastLine = $all.Count; $lg.lastPushUtc = (NowS); Ev '-' 'ledger-push' ("pushed {0} quota rows to {1} ({2})" -f $newRows.Count, $lb, (Invoke-KitGit -C $lw rev-parse --short HEAD).Trim()) } else { Ev '-' 'attention' 'ledger push failed (will retry next tick)' }
      }
    }
  } catch { Write-HbLog ("{0} ledger push failed: {1}" -f (NowS), ($_.Exception.Message -replace '\s+', ' ')); Ev '-' 'attention' ("ledger push error: " + ($_.Exception.Message -replace '\s+', ' ').Substring(0, [Math]::Min(180, $_.Exception.Message.Length))) }
  # ---- prompt files: operator-authored text only; no embedded private briefs.
  function Make-Prompt($l, $resume) {
    $ld = $lanes[$l]
    $source = $ld.promptFile
    if (-not $source) { $source = Join-Path $hb "prompt-$l.md" }
    if (Test-Path -LiteralPath $source -PathType Leaf) { $p = Get-Content -LiteralPath $source -Raw }
    else {
      $briefFile = Join-Path $hb 'briefs.md'
      if (-not $ld.range -or -not (Test-Path -LiteralPath $briefFile)) { throw "Supply prompt-$l.md or promptFile (optional briefs.md plus range is also supported)." }
      if ($ld.range -notmatch '^(\d+),(\d+)$') { throw 'range must be first,last (1-based inclusive).' }
      $first = [int]$Matches[1]; $last = [int]$Matches[2]; $b = @(Get-Content -LiteralPath $briefFile)
      if ($first -lt 1 -or $last -lt $first -or $last -gt $b.Count) { throw "Invalid brief range for $l" }
      $p = "Read AGENTS.md first.`n" + (($b[($first-1)..($last-1)]) -join "`n")
    }
    if ($p -notmatch '^Read AGENTS\.md\b[^\r\n]*first') { throw "Prompt must start 'Read AGENTS.md ... first': $source" }
    if ($resume) {
      $p += "`n`nRESUME: inspect git status and git diff, keep prior work, finish only what is missing and rerun the required checks."
      $note = Join-Path $hb "done-note-$l.md"
      if (Test-Path -LiteralPath $note) { $p += "`n" + (Get-Content -LiteralPath $note -Raw) }
    }
    if ($state.lanes[$l].mergeInfo -and $state.lanes[$l].mergeInfo -ne 'no conflicts') { $p += "`nMerge in progress; resolve: $($state.lanes[$l].mergeInfo)" }
    $followup = Join-Path $hb "followup-$l.md"
    if (Test-Path -LiteralPath $followup) { $p += "`n" + (Get-Content -LiteralPath $followup -Raw); $state.lanes[$l].followupDone = $true }
    $pf = Join-Path $out "prompt-$l.generated.md"
    if (-not $DryRun) { Set-Content -LiteralPath $pf $p -Encoding utf8 }
    return $pf
  }
  function ProbeNow($why) {
    # one tiny claude -p probe right now; writes claude-rate.json and the Claude ledger rows (notes = why). Returns the reading or $null.
    if ($DryRun -or -not $Cfg.EnableClaudeProbe -or -not $Cfg.ClaudeCmd -or (PausedWindow)) { return $null }
    try {
      $cl = $Cfg.ClaudeCmd
      $pd = (Join-Path $Cfg.TestsDir 'claude-probe'); New-Item -ItemType Directory -Force $pd | Out-Null; $po = "$out\claude-probe-stream.jsonl"
      Push-Location $pd
      try { & $cl -p ok --model haiku --tools '' --strict-mcp-config --disable-slash-commands --no-session-persistence --system-prompt 'Reply with ok.' --output-format stream-json --verbose *> $po }
      finally { Pop-Location }
      $evl = Get-Content $po -ErrorAction SilentlyContinue | Where-Object { $_ -match 'unifiedWindows' } | Select-Object -Last 1
      if (-not $evl) { Write-HbLog ("{0} claude probe ({1}) returned no rate_limit_event" -f (NowS), $why); return $null }
      $ei = ($evl | ConvertFrom-Json).rate_limit_info.unifiedWindows; $raw = Get-Content $po -Raw
      $pn = 'one tiny claude -p probe (haiku, no tools)'
      $obj = [ordered]@{ ts = (Now).ToString('o'); source = 'claude -p probe, rate_limit_event (utilization x 100)'; five_hour = [ordered]@{ used_percentage = [math]::Round([double]$ei.five_hour.utilization * 100, 1); raw_utilization = [double]$ei.five_hour.utilization; resets_at = $ei.five_hour.resetsAt }; seven_day = [ordered]@{ used_percentage = [math]::Round([double]$ei.seven_day.utilization * 100, 1); raw_utilization = [double]$ei.seven_day.utilization; resets_at = $ei.seven_day.resetsAt }; probe_usage = $pn }
      $crf = "$hb\claude-rate.json"; $tmp = "$crf.tmp"; ($obj | ConvertTo-Json -Depth 4) | Set-Content $tmp -Encoding UTF8; Move-Item $tmp $crf -Force
      $q = { param($v) '"' + ([string]$v).Replace('"', '""') + '"' }
      $nowT = (Now).ToString('yyyy-MM-ddTHH:mm') + 'Z'
      $r5 = [DateTimeOffset]::FromUnixTimeSeconds([long]$ei.five_hour.resetsAt).UtcDateTime.ToString('yyyy-MM-ddTHH:mm:ss') + 'Z'; $rw = [DateTimeOffset]::FromUnixTimeSeconds([long]$ei.seven_day.resetsAt).UtcDateTime.ToString('yyyy-MM-ddTHH:mm:ss') + 'Z'
      $row = @($nowT, 'Claude', $Cfg.LedgerAccount, 'five-hour window and weekly limit', ('5-hour {0}% used, weekly {1}% used (raw {2} and {3})' -f $obj.five_hour.used_percentage, $obj.seven_day.used_percentage, $ei.five_hour.utilization, $ei.seven_day.utilization), ('5-hour {0}; weekly {1}' -f $r5, $rw), $Cfg.LedgerReader, ('heartbeat tick, Claude claude -p probe rate_limit_event at ' + $obj.ts), $why)
      Add-Content (Join-Path $Cfg.LedgerDir 'quota-readings-pc.csv') (($row | ForEach-Object { & $q $_ }) -join ',') -Encoding UTF8
      return $obj
    } catch { Write-HbLog ("{0} claude probe ({1}) failed: {2}" -f (NowS), $why, ($_.Exception.Message -replace '\s+', ' ')); return $null }
  }
  function ClaudeDeltaRow($l, $before, $after) {
    try {
      $q = { param($v) '"' + ([string]$v).Replace('"', '""') + '"' }
      $reading = ('weekly {0}% -> {1}% (+{2}); 5-hour {3}% -> {4}% (+{5}) between {6} and {7}' -f $before.pw, $after.seven_day.used_percentage, ([math]::Round($after.seven_day.used_percentage - $before.pw, 1)), $before.p5, $after.five_hour.used_percentage, ([math]::Round($after.five_hour.used_percentage - $before.p5, 1)), $before.ts, $after.ts)
      $row = @(((Now).ToString('yyyy-MM-ddTHH:mm') + 'Z'), 'Claude', $Cfg.LedgerAccount, ("meter change while lane $l ran"), $reading, 'not applicable', $Cfg.LedgerReader, "heartbeat before/after probes around lane $l", 'the Claude meter is account-wide: the change includes every other Claude session during the same time, not only this lane')
      Add-Content (Join-Path $Cfg.LedgerDir 'quota-readings-pc.csv') (($row | ForEach-Object { & $q $_ }) -join ',') -Encoding UTF8
      Ev $l 'claude-meter' $reading
    } catch { }
  }  function Launch($l, $promptFile, $tag) {
    if ($DryRun) { Write-Host "DRYRUN launch $l model=$($lanes[$l].model) effort=$($lanes[$l].effort)"; return }
    $stamp = (Get-Date).ToString('HHmmss'); foreach ($ext in 'md','log') { $o = "$out\$tag.$ext"; if (Test-Path $o) { Move-Item $o "$out\$tag.prev-$stamp.$ext" -Force } }
    # Use the physical CodexCmd via run-lane.ps1: an MSIX-packaged app hides the npm folder.
    # Launch through WMI so the lane's process tree is NOT a child of this tick's job object: a tick started by Task Scheduler (or a background job) kills its children when it ends.
    $mdl = [string]$lanes[$l].model; if (-not $mdl) { $mdl = $Cfg.DefaultModel }; $eff = [string]$lanes[$l].effort; if (-not $eff) { $eff = $Cfg.DefaultEffort }
    $cmdline = "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$hb\run-lane.ps1`" -L $l -Prompt `"$promptFile`" -Tag $tag -Model $mdl -Effort $eff -ConfigPath `"$ConfigPath`""
    try {
      $si = New-CimInstance -ClassName Win32_ProcessStartup -ClientOnly -Property @{ ShowWindow = [uint16]0 }
      $cr = Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{ CommandLine = $cmdline; CurrentDirectory = $hb; ProcessStartupInformation = $si }
      if ($cr.ReturnValue -ne 0) { throw "Win32_Process.Create returned $($cr.ReturnValue)" }
    } catch {
      Write-HbLog ("{0} WMI launch failed for {1} ({2}); falling back to Start-Process" -f (NowS), $l, $_.Exception.Message)
      $fallbackArgs = @('-NoProfile','-ExecutionPolicy','Bypass','-File',('"' + "$hb\run-lane.ps1" + '"'),'-L',$l,'-Prompt',('"' + $promptFile + '"'),'-Tag',$tag,'-Model',$mdl,'-Effort',$eff,'-ConfigPath',('"' + $ConfigPath + '"'))
      Start-Process powershell -ArgumentList $fallbackArgs -WindowStyle Hidden
    }
  }
  function GitC($wt) { return (Invoke-KitGit -C $wt @args 2>&1 | Out-String).Trim() }
  function Ensure-Worktree($l) {
    $ld = $lanes[$l]; $wt = "$root\$l"
    if (Test-Path $wt) { return $true }
    if ($DryRun) { Write-Host "DRYRUN would prepare worktree $l (repo actions enabled=$($Cfg.EnableRepoActions))"; return $false }
    if (-not $Cfg.EnableRepoActions) { Ev $l 'repo-actions-disabled' 'worktree creation/fetch requires EnableRepoActions'; return $false }
    if ($ld.review) {
      Invoke-KitGit -C $Cfg.MainCheckout fetch --all --prune 2>&1 | Out-Null
      Invoke-KitGit -C $Cfg.MainCheckout worktree add --detach $wt $ld.base 2>&1 | Out-Null
      if ($LASTEXITCODE -ne 0) { return $false }
      return (Test-Path -LiteralPath $wt)
    }
    Invoke-KitGit -C $Cfg.MainCheckout fetch --all --prune 2>&1 | Out-Null
    $o = Invoke-KitGit -C $Cfg.MainCheckout worktree add -b $ld.br $wt $ld.base 2>&1 | Out-String
    if (-not (Test-Path $wt)) { Ev $l 'attention' "worktree add failed: $o"; return $false }
    foreach ($m in $ld.merge) {
      $mo = Invoke-KitGit -C $wt merge --no-edit $m 2>&1 | Out-String
      if ($LASTEXITCODE -ne 0) { Invoke-KitGit -C $wt merge --abort 2>&1 | Out-Null; Ev $l 'attention' "merge of $m conflicted; merge aborted, lane NOT started (needs a decision)"; return $false }
    }
    $state.lanes[$l].workBase = (Invoke-KitGit -C $wt rev-parse HEAD).Trim()
    return $true
  }
  function Run-Check($name, $cmd, $dir, $lane) {
    $lf = "$out\$lane.check-$name.log"
    if ($DryRun) { return @{ name=$name; exit=-1; log=$lf; skipped=$true } }
    $env:CARGO_TARGET_DIR = $Cfg.CargoTargetDir; $env:CARGO_NET_OFFLINE = 'true'
    foreach ($key in $Cfg.LaneEnvironment.Keys) { [Environment]::SetEnvironmentVariable($key, ([string]$Cfg.LaneEnvironment[$key]).Replace('{TestsDir}', $Cfg.TestsDir).Replace('{lane}', $lane).Replace('{Root}', $root), 'Process') }
    Push-Location $dir; cmd /c "$cmd > `"$lf`" 2>&1"; $ec = $LASTEXITCODE; Pop-Location
    return @{ name=$name; exit=$ec; log=$lf; tail=((Get-Content $lf -Tail 6 -ErrorAction SilentlyContinue) -join ' | ') }
  }
  function Checkpoint($l) {
    # local WIP commit of a lane's work so a cut-off loses little; Codex cannot write the git dir itself. Never pushes.
    if ($DryRun -or -not $Cfg.EnableRepoActions) { return }
    $wt = "$root\$l"; if (-not (Test-Path $wt)) { return }
    Invoke-KitGit -C $wt rev-parse -q --verify MERGE_HEAD 2>&1 | Out-Null; if ($LASTEXITCODE -eq 0) { return }   # merge in progress: never commit conflict markers
    $porc = (Invoke-KitGit -C $wt status --porcelain 2>&1) -split "`n" | Where-Object { $_.Trim() -and $_ -notmatch '^warning' }
    $paths = @(); foreach ($line in $porc) { $p = $line.Substring(3).Trim('"'); if ($p -match $Cfg.ExcludedCommitPathPattern ) { continue }; $paths += $p }
    if (-not $paths.Count) { return }
    Invoke-KitGit -C $wt add -- @paths 2>&1 | Out-Null
    $msg = "WIP checkpoint, lane ${l} (automatic, $(NowS))`n`nWork in progress; not finished or checked. Model=$($lanes[$l].model) effort=$($lanes[$l].effort).`n$($paths.Count) changed files."
    $mf = "$out\$l.checkpoint-msg.txt"; [System.IO.File]::WriteAllText($mf, $msg, (New-Object System.Text.UTF8Encoding($false)))
    Invoke-KitGit -C $wt commit -F $mf 2>&1 | Out-Null
  }
  function MergeLatestMain($l) {
    if ($DryRun -or -not $Cfg.EnableRepoActions) { Ev $l 'repo-actions-disabled' 'merge before finish requires EnableRepoActions'; return $false }
    # Merge the configured remote main branch before finishing when requested.
    $ld = $lanes[$l]; $wt = "$root\$l"; $ls = $state.lanes[$l]
    Invoke-KitGit -C $Cfg.MainCheckout fetch $Cfg.Remote $Cfg.MainBranch 2>&1 | Out-Null
    Invoke-KitGit -C $wt rev-parse -q --verify MERGE_HEAD 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
      # an earlier merge is still open: refuse to commit conflict markers, then complete it
      foreach ($cf in @(([string]$ls.mergeInfo) -split ',\s*' | Where-Object { $_ -and $_ -ne 'no conflicts' })) { $cp = Join-Path $wt $cf; if ((Test-Path $cp) -and (Select-String -Path $cp -Pattern '^(<<<<<<<|>>>>>>>) ' -Quiet)) { $ls.status = 'attention'; Ev $l 'attention' "conflict markers left in $cf; needs a fix"; return $false } }
      $porc = (Invoke-KitGit -C $wt status --porcelain 2>&1) -split "`n" | Where-Object { $_.Trim() -and $_ -notmatch '^warning' }
      $ps = @(); foreach ($line in $porc) { $p = $line.Substring(3).Trim('"'); if ($p -match $Cfg.ExcludedCommitPathPattern) { continue }; $ps += $p }
      if ($ps.Count) { Invoke-KitGit -C $wt add -- @ps 2>&1 | Out-Null }
      Invoke-KitGit -C $wt commit -m "WIP: complete the open merge of $mainRef (heartbeat, $(NowS))" 2>&1 | Out-Null
    } else { Checkpoint $l }
    $mo = Invoke-KitGit -C $wt merge --no-commit --no-ff $mainRef 2>&1 | Out-String
    $conf = @(Invoke-KitGit -C $wt diff --name-only --diff-filter=U 2>$null | Where-Object { $_ })
    if ($conf.Count) {
      $ls.mergeInfo = $conf -join ', '; $ls.status = 'pending'; $ls.resume = $true
      Ev $l 'merge-conflict' ("merging the newest $mainRef into $($ld.br) conflicts in: $($ls.mergeInfo); lane relaunched to resolve them (mechanical only)"); return $false
    }
    $ls.mergeInfo = 'no conflicts'; Ev $l 'merged-main' "merged the newest $mainRef into $($ld.br) before finishing"; return $true
  }  function Finish-Lane($l) {
    $ld = $lanes[$l]; $wt = "$root\$l"; $ls = $state.lanes[$l]
    $pa = ProbeNow "after lane $l finished"; if ($pa -and $ls.claudeBefore) { ClaudeDeltaRow $l $ls.claudeBefore $pa; $ls.claudeBefore = $null }
    if ($ld.review) { $ls.status = 'done'; Ev $l 'finished' ("review run ended; hand-back: $out\$l.md" + (ModelNote $l)); return }
    if ($DryRun -or -not $Cfg.EnableRepoActions) { Ev $l 'repo-actions-disabled' 'finish checks/commit/push deferred; enable repo actions or finish manually'; $ls.status = 'attention'; return }
    $porc = (Invoke-KitGit -C $wt status --porcelain 2>&1) -split "`n" | Where-Object { $_.Trim() -and $_ -notmatch '^warning' }
    $paths = @(); foreach ($line in $porc) { $p = $line.Substring(3).Trim('"'); if ($p -match $Cfg.ExcludedCommitPathPattern ) { continue }; $paths += $p }
    # own work = commits after the worktree was prepared (merges of earlier lanes do not count as this lane's work)
    $wb = $ld.base; if ($ls.workBase) { $wb = $ls.workBase }
    $ahead = [int](Invoke-KitGit -C $wt rev-list --first-parent --no-merges --count "$wb..HEAD" 2>$null); $cnames = @(Invoke-KitGit -C $wt diff --name-only "$wb..HEAD" 2>$null); $allnames = @($paths) + $cnames
    $changedRust = [bool]($allnames | Where-Object { $_ -like 'src-tauri/*' }); $changedBridge = [bool]($allnames | Where-Object { $_ -like 'bridge/*' }); $changedTs = [bool]($allnames | Where-Object { $_ -like 'src/*' -or $_ -like 'dev-mock/*' })
    $res = @()
    if ($changedTs) { $res += Run-Check 'tsc' 'npx tsc --noEmit -p tsconfig.json' $wt $l; $res += Run-Check 'build' 'npm run build' $wt $l }
    if ($changedRust) { $res += Run-Check 'cargo-test' 'cargo test --lib --offline' "$wt\src-tauri" $l }
    if ($changedBridge -and (Test-Path "$wt\bridge")) { $res += Run-Check 'bridge-test' 'cargo test --offline' "$wt\bridge" $l }
    $res += Run-Check 'audit' 'npm audit' $wt $l
    $failed = @($res | Where-Object { $_.exit -ne 0 -and $_.name -ne 'audit' })
    $status = 'pushed'
    if ($paths.Count -eq 0 -and $ahead -eq 0) { $ls.stalls = [int]$ls.stalls + 1; if ($ls.stalls -le 2) { $ls.status = 'pending'; $ls.resume = $true; Ev $l 'idle-stop' ("run ended with exit 0 and NO file changes (boundary stop, not finished); relaunch #{0} queued; see hand-back {1}" -f $ls.stalls, $out) } else { $ls.status = 'attention'; Ev $l 'attention' 'three runs in a row ended with no file changes; needs a decision' }; return }
    if (-not $DryRun) {
      if ($paths.Count) { Invoke-KitGit -C $wt add -- @paths 2>&1 | Out-Null }
      $stat = GitC $wt diff --cached --stat $wb
      $hand = ''; $mdp = (PathsFor $l ($(if ($ls.followupDone -and (Test-Path "$out\$l.fu.md")) { "$l.fu" } else { $l }))).md; if (Test-Path $mdp) { $hand = (Get-Content $mdp -TotalCount 25) -join "`n" }
      $msg = "Lane ${l}: $($ld.title)`n`nWritten by Codex, model=$($ld.model) effort=$($ld.effort). Automated heartbeat commit.`nFiles:`n$stat`nChecks: " + (($res | ForEach-Object { "$($_.name)=$($_.exit)" }) -join ', ') + "`nNot merged or released."
      $mf = "$out\$l.commitmsg.txt"; [System.IO.File]::WriteAllText($mf, $msg, (New-Object System.Text.UTF8Encoding($false)))
      if ($paths.Count) { Invoke-KitGit -C $wt commit -F $mf 2>&1 | Out-Null }
      $hash =(Invoke-KitGit -C $wt rev-parse --short HEAD).Trim()
      $pushed = $false; foreach ($i in 0..3) { Invoke-KitGit -C $wt push -u $Cfg.Remote $ld.br 2>&1 | Out-Null; if ($LASTEXITCODE -eq 0) { $pushed = $true; break }; Start-Sleep -Seconds ([math]::Pow(2, $i + 1)) }
      $rep = "# Lane $l report ($(NowS))`n`nBranch: $($ld.br)  Commit: $hash  Pushed: $pushed`n`n## Checks`n" + (($res | ForEach-Object { "- $($_.name): exit $($_.exit) ($($_.log)) :: $($_.tail)" }) -join "`n") + "`n`n## Files`n$stat`n`n## Codex hand-back (first lines)`n$hand`n"
      Set-Content "$out\$l-report.md" $rep -Encoding utf8
      $ls.commit = $hash; $ls.pushed = $pushed; $ls.mergeInfo = $null; if ($pushed) { $ls.reviewReady = $true }
      if ($pushed -and $failed.Count -eq 0) { $ls.status = 'pushed'; $ls.checksFailed = $false; Ev $l 'finished' ("branch $($ld.br) commit $hash pushed; checks: " + (($res | ForEach-Object { "$($_.name)=$($_.exit)" }) -join ' ') + (ModelNote $l)) }
      elseif ($pushed) { $ls.status = 'pushed'; $ls.checksFailed = $true; Ev $l 'finished-checks-failed' ("branch $($ld.br) commit $hash pushed; FAILED: " + (($failed | ForEach-Object { $_.name }) -join ',') + " ; see $out\$l-report.md" + (ModelNote $l)) }
      else { $ls.status = 'attention'; Ev $l 'attention' "commit $hash made but push failed" }
    }
  }

  # ---- (a)(d)(e)(f) per-lane pass
  EnforceStopLine $rate
  $effMax = $maxAlive; if ($rate -and [double]$rate.p -ge $Cfg.HighMeterPct) { $effMax = [Math]::Min($maxAlive, $Cfg.MaxAliveHigh) }
  $planBurn = if ($rate) { BurnPerMin $rate } else { 0.0 }
  if ($DryRun) { Write-Host ("DRYRUN lane models: " + (($lanes.Keys | Where-Object { $state.lanes[$_].status -in 'pending','paused-stopline' } | ForEach-Object { $mm = [string]$lanes[$_].model; if (-not $mm) { $mm = $Cfg.DefaultModel }; $ee = [string]$lanes[$_].effort; if (-not $ee) { $ee = $Cfg.DefaultEffort }; "$_=$mm/$ee" }) -join ', ')) }
  if ($DryRun -and $rate -and $state.stopLine -and $state.stopLine.active) { $anyLane = @(Get-LaneProcesses).Count; $liftOk = ([string]$state.stopLine.reason -match 'predicted' -and [long]$rate.pr -eq [long]$state.stopLine.resetEpoch -and [double]$rate.p -lt $Cfg.LiftBelowPct -and ([double]$rate.p + $planBurn * $LookAheadMin) -lt $Cfg.StopAtPct -and -not $anyLane); Write-Host ("DRYRUN stop-line lift check: held because '{0}' since {1}, reset {2}; used {3}% burn {4}/min lanes running {5} -> {6}" -f $state.stopLine.reason, $state.stopLine.since, $state.stopLine.resetUtc, $rate.p, $planBurn, $anyLane, $(if ($liftOk) { 'WOULD LIFT (predicted line, nothing runs, used < 90, used + look-ahead x burn < 95)' } else { 'stays held' })) }
  if ($DryRun -and $rate) { Write-Host ("DRYRUN stop-line plan: used {0}% burn {1} points/min predicted(5 min) {2}% -> line {3}; effective max lanes {4}; watch loop {5}" -f $rate.p, $planBurn, [math]::Round([double]$rate.p + $planBurn * $LookAheadMin, 1), $(if ([double]$rate.p -ge $Cfg.StopAtPct -or ($planBurn -gt 0 -and [double]$rate.p + $planBurn * $LookAheadMin -ge $Cfg.StopAtPct)) { 'REACHED' } else { 'not reached' }), $effMax, $(if ([double]$rate.p -ge $Cfg.WatchStartPct -or ($planBurn -gt 0 -and [double]$rate.p + $planBurn * $Cfg.WatchPredictMin -ge $Cfg.WatchPredictPct)) { ('on (' + $Cfg.WatchSampleSec + ' s samples)') } else { 'off' })) }
  $script:launched = 0; $script:launchedFix = 0; $lineParts = @()
  foreach ($l in @($lanes.Keys)) {
    $ls = $state.lanes[$l]; $wt = "$root\$l"
    $alive = Alive $l
    $fuTag = "$l.fu"
    if ($ls.status -eq 'running' -and -not $alive -and -not (AliveNow $l)) {
      $mdp = (PathsFor $l $l).md; $lgp = (PathsFor $l $l).log
      $logT = Tail-Text $lgp 30
      if ((Test-Path $mdp) -and ($logT -match 'EXIT 0|EXIT')) {
        $fu = "$hb\followup-$l.md"
        if ((Test-Path $fu) -and -not $ls.followupDone -and -not $limitActive -and -not $state.stopLine.active -and -not $DryRun) {
          $ls.followupDone = $true; $ls.status = 'followup-running'; Launch $l $fu $fuTag; Ev $l 'followup-started' "first pass finished; follow-up launched"
        } else { $ls.status = 'checking' }
      } elseif ($limitActive) { $ls.status = 'pending'; $ls.resume = $true; Ev $l 'paused-by-limit' 'run ended without a hand-back while a usage limit is active; will resume after reset' }
      else { $lsz = 0; if (Test-Path $lgp) { $lsz = (Get-Item $lgp).Length }; $ls.stalls = [int]$ls.stalls + 1; if ($lsz -lt 2000 -and $ls.stalls -le 3) { $ls.status = 'pending'; $ls.resume = $true; Ev $l 'launch-failed' ('the lane process died at once (log {0} bytes); relaunch #{1} queued' -f $lsz, $ls.stalls) } else { $ls.status = 'attention'; Ev $l 'attention' 'codex process gone without a hand-back file; see log' } }
    }
    if ($ls.status -eq 'followup-running' -and -not $alive -and -not (AliveNow $l)) {
      $mdp = (PathsFor $l $fuTag).md; $lgp = (PathsFor $l $fuTag).log
      if ((Test-Path $mdp) -and ((Tail-Text $lgp 30) -match 'EXIT')) { $ls.status = 'checking' }
      elseif ((Test-Path $lgp) -and -not $limitActive -and -not (Test-Path (PathsFor $l $fuTag).md)) { $ls.status = 'attention'; Ev $l 'attention' 'follow-up ended without hand-back' }
    }
    if ($ls.status -eq 'checking') { $go = $true; if ($ls.mergeMainBeforeFinish -and -not $DryRun) { $go = MergeLatestMain $l }; if ($go) { $ls.mergeMainBeforeFinish = $false; Finish-Lane $l } }
    if ($ls.status -eq 'pending') {
      $depsOk = $true; foreach ($d in $lanes[$l].deps) { if ($state.lanes[$d].status -ne 'pushed' -or $state.lanes[$d].checksFailed) { $depsOk = $false } }; if ($lanes[$l].gate -and -not $state.lanes[$lanes[$l].gate].reviewReady) { $depsOk = $false }
      $aliveCount = @($lanes.Keys | Where-Object { Alive $_ }).Count + $script:launched
      $quotaOk = $true; if ($rate -and $rate.p -ge $Cfg.StopAtPct -and $rate.pr -gt [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()) { $quotaOk = $false }; if ($state.stopLine -and $state.stopLine.active) { $quotaOk = $false }
      if ($rate -and $planBurn -gt 0 -and ([double]$rate.p + $planBurn * $LookAheadMin -ge $Cfg.StopAtPct)) { $quotaOk = $false }


      if ($depsOk -and $quotaOk -and -not $limitActive -and $aliveCount -lt $effMax) {
        if (Ensure-Worktree $l) {
          if ($ls.premerge -and -not $DryRun -and $Cfg.EnableRepoActions) {
            Invoke-KitGit -C "$root\$l" fetch $Cfg.Remote $Cfg.MainBranch 2>&1 | Out-Null
            if ($ls.premergeExtra) { $xo = Invoke-KitGit -C "$root\$l" merge --no-edit $ls.premergeExtra 2>&1 | Out-String; if ($LASTEXITCODE -ne 0) { Invoke-KitGit -C "$root\$l" merge --abort 2>&1 | Out-Null; $ls.status = 'attention'; Ev $l 'attention' ("merge of " + $ls.premergeExtra + " failed: " + (($xo -replace '\s+', ' ').Substring(0, [Math]::Min(160, $xo.Length)))); continue }; $ls.premergeExtra = $null }
            $ls.workBase = (Invoke-KitGit -C "$root\$l" rev-parse HEAD).Trim()
            $mo = Invoke-KitGit -C "$root\$l" merge --no-commit --no-ff $mainRef 2>&1 | Out-String
            $conf = @(Invoke-KitGit -C "$root\$l" diff --name-only --diff-filter=U 2>$null | Where-Object { $_ })
            $ls.mergeInfo = if ($conf.Count) { $conf -join ', ' } else { 'no conflicts' }
            $ls.premerge = $false; Ev $l 'premerge' "merged $mainRef into $($lanes[$l].br) without committing; conflicts: $($ls.mergeInfo)"
          }
          $pb = ProbeNow "before lane $l starts"; if ($pb) { $ls.claudeBefore = @{ ts = $pb.ts; p5 = $pb.five_hour.used_percentage; pw = $pb.seven_day.used_percentage } }
          try { $pf = @(Make-Prompt $l ([bool]$ls.resume))[-1] }
          catch { $ls.status = 'attention'; Ev $l 'attention' $_.Exception.Message; continue }; $ls.legacy = $false; $ls.status = 'running'; $ls.startedUtc = NowS; $ls.resume = $false
          Launch $l $pf $l; $script:launched++; if ($lanes[$l].fix) { $script:launchedFix++ }; if ($rate) { $ls.meterBefore5h = $rate.p; $ls.meterBeforeW = $rate.w }; Ev $l 'started' ("launched on $($lanes[$l].br)" + (ModelNote $l))
        } else { $ls.status = 'attention' }
      }
    }
    if ($ls.status -in 'running','pending','followup-running') { Checkpoint $l }
    $changed = 0; if (Test-Path $wt) { $changed = @((Invoke-KitGit -C $wt status --porcelain 2>&1) | Where-Object { $_ }).Count; $last = (Invoke-KitGit -C $wt log -1 --format='%h' 2>&1) } else { $last = '-' }
    $lineParts += ("{0}:{1}{2} chg={3} {4}" -f $l, $ls.status, $(if ($alive) { '(alive)' } else { '' }), $changed, $last)
  }
  $aliveNow = $script:launched + @($lanes.Keys | Where-Object { Alive $_ }).Count
  if ($aliveNow -lt 2 -and -not $DryRun) { $blocked = @($lanes.Keys | Where-Object { $state.lanes[$_].status -in "pending","hold","attention" }) -join ","; Ev "-" "idle-check" ("fewer than 2 Codex lanes running at tick end (alive={0}); limit={1}; lanes not running: {2}" -f $aliveNow, $state.limit.active, $blocked) }
  $lim = if ($state.limit.active) { "LIMIT until $($state.limit.resetUtc)" } else { 'no-limit' }; if ($state.weeklyLimit) { $lim += " WEEKLY-LIMIT" }; if ($state.claude_quota_state) { $cqs = $state.claude_quota_state; $lim += (" [claude 5h={0}% wk={1}%{2}{3}]" -f $cqs.five_hour_used_pct, $cqs.weekly_used_pct, $(if ($cqs.claude_pause) { " PAUSE" } elseif ($cqs.claude_warn) { " WARN" } else { "" }), $(if (-not $cqs.fresh) { " stale" } else { "" })) }; if ($state.rate) { $lim += (" [codex 5h={0}% wk={1}% @{2}]" -f $state.rate.p, $state.rate.w, ([string]$state.rate.ts).PadRight(19).Substring(11,8)) }
  $summary = "{0} {1} | {2}" -f (NowS), $lim, ($lineParts -join ' | ')
  if ($DryRun) { Write-Host "DRYRUN summary $summary" } else { Write-HbLog $summary }
  if (-not $DryRun -and $rate -and ([double]$rate.p -ge $Cfg.WatchStartPct -or ($planBurn -gt 0 -and [double]$rate.p + $planBurn * $Cfg.WatchPredictMin -ge $Cfg.WatchPredictPct))) {
    $wEnd = (Get-Date).AddMinutes([Math]::Max(0, $Cfg.TickMinutes - $Cfg.GuardMinutes))
    while ((Get-Date) -lt $wEnd) {
      $anyAlive = @(Get-LaneProcesses).Count
      if ($anyAlive -eq 0) { break }
      Start-Sleep -Seconds $Cfg.WatchSampleSec
      $rr = Read-CodexRate
      if ($rr) { $state.rate = $rr; EnforceStopLine $rr; if ($state.stopLine -and $state.stopLine.active) { break } }
    }
  }
  Save-State
} finally {
  if ($lockFs) { $lockFs.Close() }
}
