# Predictive stop-line guard. Never launches lanes, probes, merges or spends resets.
# -DryRun / -Replay / -Inject never write files or stop processes.
param(
  [switch]$DryRun, [switch]$Replay, [string]$ReplayDate = '',
  [string]$ReplayFrom = '00:00:00', [string]$Inject = '',
  [string]$ConfigPath = (Join-Path $PSScriptRoot 'config.ps1')
)
$ErrorActionPreference = 'Stop'
. $ConfigPath
if ($Cfg -isnot [hashtable]) { throw 'Config must define $Cfg as a hashtable.' }
$LookAheadMin = [double]$Cfg.LookAheadMin
$StopAtPct = [double]$Cfg.StopAtPct
if ($LookAheadMin -lt 0 -or $StopAtPct -le 0 -or $StopAtPct -gt 100) { throw 'Invalid stop-line settings.' }
$root = [IO.Path]::GetFullPath($Cfg.Root).TrimEnd('\')
$hb = $Cfg.KitDir; $out = $Cfg.OutDir; $logDir = Join-Path $hb 'logs'
$statePath = $Cfg.StateFile; $evPath = $Cfg.EventsLog
$guardFile = Join-Path $hb 'stopline-guard.json'
$histPath = Join-Path $logDir 'stopline-guard-hist.json'
$logPath = Join-Path $logDir 'stopline-guard.log'
# Optional extra contract key: CodexSessionsDir is the read-only Codex JSONL source.
# Until configured, use an empty/synthetic sessions folder beneath TestsDir.
$sessionsDir = if ($Cfg.CodexSessionsDir) { $Cfg.CodexSessionsDir } else { Join-Path $Cfg.TestsDir 'codex-sessions' }
$observeOnly = $DryRun -or $Replay -or [bool]$Inject
function Log-Line($path, $line) {
  if ($observeOnly) { Write-Host $line; return }
  New-Item -ItemType Directory -Force (Split-Path -Parent $path) | Out-Null
  Add-Content -LiteralPath $path -Value $line
}
function Lane-Processes {
  $prefix = [regex]::Escape($root + '\')
  # Match the complete -C argument, quoted or unquoted; only direct lane children.
  $rx = '(?:^|\s)-C\s+(?:"' + $prefix + '(?<quoted>[^"\\]+)"|' + $prefix + '(?<bare>[^\s"\\]+))(?=\s|$)'
  foreach ($proc in @(Get-CimInstance Win32_Process -Filter "Name='codex.exe'")) {
    if ($proc.CommandLine -notmatch '(?:^|\s)exec(?:\s|$)') { continue }
    $match = [regex]::Match([string]$proc.CommandLine, $rx, 'IgnoreCase')
    if (-not $match.Success) { continue }
    $lane = $match.Groups['quoted'].Value
    if (-not $lane) { $lane = $match.Groups['bare'].Value }
    if ($lane -notmatch '^[A-Za-z0-9][A-Za-z0-9_-]*$') { continue }
    [pscustomobject]@{ Lane = $lane; ProcessId = $proc.ProcessId }
  }
}
function Now { (Get-Date).ToUniversalTime() }
function NowS { (Now).ToString('yyyy-MM-dd HH:mm:ss') + 'Z' }
function Glog($t) { Log-Line $logPath ("{0} {1}" -f (NowS), $t) }
function Ev($lane, $kind, $text) { Log-Line $evPath ("{0}|{1}|{2}|{3}" -f (NowS), $lane, $kind, $text) }
function ToHash($o) {
  if ($o -is [System.Management.Automation.PSCustomObject]) { $h = @{}; foreach ($p in $o.PSObject.Properties) { $h[$p.Name] = ToHash $p.Value }; return $h }
  if ($o -is [System.Array]) { return @($o | ForEach-Object { ToHash $_ }) }
  return $o
}
function Tail-Text($path, $n) {
  if (-not (Test-Path $path)) { return '' }
  try {
    $fs = [System.IO.File]::Open($path, 'Open', 'Read', 'ReadWrite')
    try {
      $len = $fs.Length; $take = [Math]::Min($len, 120000)
      $fs.Seek($len - $take, 'Begin') | Out-Null; $buf = New-Object byte[] $take; $null = $fs.Read($buf, 0, $take)
      $clean = New-Object System.Collections.Generic.List[byte]; foreach ($x in $buf) { if ($x -ne 0 -and $x -ne 0xFF -and $x -ne 0xFE) { $clean.Add($x) } }
      return ((([System.Text.Encoding]::UTF8.GetString($clean.ToArray())) -split "`r?`n") | Select-Object -Last $n) -join "`n"
    } finally { $fs.Close() }
  } catch { return '' }
}
function Read-CodexRate {
  # newest 'codex' rate_limits event across the recent Codex session files (same method as heartbeat.ps1; reads only, never probes)
  $best = $null
  $sf = Get-ChildItem $sessionsDir -Recurse -Filter *.jsonl -ErrorAction SilentlyContinue | Where-Object { $_.LastWriteTime -gt (Get-Date).AddHours(-6) } | Sort-Object LastWriteTime -Descending | Select-Object -First 12
  foreach ($s in $sf) {
    try {
      $tt = Tail-Text $s.FullName 4000
      $ln = @($tt -split "`n" | Where-Object { $_ -match '"limit_id":"codex"' -and $_ -match '"primary":\{' }) | Select-Object -Last 1
      if ($ln) { $o = $ln | ConvertFrom-Json; $c = @{ f = $s.Name; ts = [string]$o.timestamp; p = [double]$o.payload.rate_limits.primary.used_percent; w = $o.payload.rate_limits.secondary.used_percent; pr = [long]$o.payload.rate_limits.primary.resets_at }; if (-not $best -or $c.ts -gt $best.ts) { $best = $c } }
    } catch { }
  }
  return $best
}
# same rule as heartbeat.ps1 (BurnPerMin): newest reading vs the oldest reading of the same window that is 0.5 to 20 minutes older; 0 when unknown or older than 4 minutes
function Burn($hist, $nowT) {
  if (@($hist).Count -lt 2) { return 0.0 }
  $last = $hist[-1]; $lastT = [DateTime]::Parse([string]$last.t).ToUniversalTime()
  if (($nowT - $lastT).TotalMinutes -gt 4) { return 0.0 }
  $base = $null
  for ($i = 0; $i -lt $hist.Count - 1; $i++) { $dm0 = ($lastT - [DateTime]::Parse([string]$hist[$i].t).ToUniversalTime()).TotalMinutes; if ($dm0 -ge 0.5 -and $dm0 -le 20) { $base = $hist[$i]; break } }
  if (-not $base) { return 0.0 }
  $dm = ($lastT - [DateTime]::Parse([string]$base.t).ToUniversalTime()).TotalMinutes
  $b = ([double]$last.p - [double]$base.p) / $dm
  if ($b -lt 0) { return 0.0 } else { return [math]::Round($b, 2) }
}
function Verdict($p, $burn) { $pred = [math]::Round([double]$p + $burn * $LookAheadMin, 1); return @{ pred = $pred; reached = (([double]$p -ge $StopAtPct) -or ($burn -gt 0 -and $pred -ge $StopAtPct)); reason = $(if ([double]$p -ge $StopAtPct) { 'measured' } else { 'predicted' }) } }

# ---------------- replay mode: no side effects at all
if ($Inject) {
  $base = [DateTime]::Parse('2000-01-01T00:00:00Z').ToUniversalTime(); $hist = @()
  foreach ($pair in ($Inject -split ',')) {
    $mp = $pair.Trim() -split ':'; if ($mp.Count -ne 2) { continue }
    $t = $base.AddMinutes([double]$mp[0]); $p = [double]$mp[1]
    $hist += @{ t = $t.ToString('o'); p = $p }
    $b = Burn $hist $t; $v = Verdict $p $b
    "t+{0,5} min used {1,5}%  burn {2,5} points/min  predicted({3} min) {4,6}%  ->  {5}" -f $mp[0], $p, $b, $LookAheadMin, $v.pred, $(if ($v.reached) { "REACHED ($($v.reason)): the guard would stop running lanes" } else { 'continue' })
  }
  return
}
if ($Replay) {
  if (-not $ReplayDate) { $ReplayDate = (Now).ToString('yyyy-MM-dd') }
  $seen = @{}; $hist = @(); $prev = $null
  foreach ($l in Get-Content -LiteralPath $Cfg.HeartbeatLog) {
    if ($l -notmatch "^$ReplayDate " -or $l -notmatch '\[codex 5h=([\d.]+)% wk=([\d.]+)% @(\d\d:\d\d:\d\d)\]') { continue }
    $p = [double]$Matches[1]; $tm = $Matches[3]
    if ($seen[$tm]) { continue }; $seen[$tm] = 1
    $t = [DateTime]::Parse("${ReplayDate}T${tm}Z").ToUniversalTime()
    if ($prev -ne $null -and $p -lt $prev - 30) { $hist = @() }   # the meter dropped: a new window
    $prev = $p
    $hist += @{ t = $t.ToString('o'); p = $p }
    $b = Burn $hist $t; $v = Verdict $p $b
    if ($tm -ge $ReplayFrom) { "{0}Z used {1,5}%  burn {2,5} points/min  predicted({5} min) {3,6}%  ->  {4}" -f $tm, $p, $b, $v.pred, $(if ($v.reached) { "LINE REACHED ($($v.reason)); the guard would stop running lanes" } else { 'continue' }), $LookAheadMin }
  }
  return
}

# ---------------- one guard run
$guardLock = $null; $hbLock = $null
if (-not $DryRun) {
  New-Item -ItemType Directory -Force $logDir | Out-Null
  try { $guardLock = [System.IO.File]::Open("$logDir\guard.lock", 'OpenOrCreate', 'ReadWrite', 'None') } catch { return }
}   # another guard run is active
try {
  $nowT = Now; $nowEpoch = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
  # cheapest exit first: while the stop line is on and its window has not reset, there is nothing to guard (no process query, no meter read)
  try { if (-not $DryRun -and (Test-Path $statePath)) { $s0 = Get-Content $statePath -Raw | ConvertFrom-Json; if ($s0.stopLine -and $s0.stopLine.active -and [long]$s0.stopLine.resetEpoch -gt $nowEpoch) { return } } } catch { }
  $procs = @(Lane-Processes)
  $laneNames = @($procs | ForEach-Object { $_.Lane } | Sort-Object -Unique)
  # second cheap exit: no Codex lane process means nothing to stop. Nothing else is read, nothing is written, nothing is logged on such a quiet run.
  if (-not $laneNames.Count -and -not $DryRun) { return }
  # state (read only here) and the guard file
  $st = $null; try { if (Test-Path $statePath) { $st = ToHash (Get-Content $statePath -Raw | ConvertFrom-Json) } } catch { $st = $null }
  $gf = $null; try { if (Test-Path $guardFile) { $gf = ToHash (Get-Content $guardFile -Raw | ConvertFrom-Json) } } catch { $gf = $null }
  $r = Read-CodexRate
  if (-not $r) { if ($DryRun) { Write-Host "DRYRUN guard: no reading in configured sessions folder; no action" }; if ($laneNames.Count) { Glog "no Codex reading found while lanes run: $($laneNames -join ',')" }; return }
  $age = [math]::Round(($nowT - [DateTime]::Parse($r.ts).ToUniversalTime()).TotalMinutes, 1)
  # own history, plus the heartbeat's stored readings (read only), same window only
  $hist = @(); try { if (Test-Path $histPath) { $hist = @(ToHash (Get-Content $histPath -Raw | ConvertFrom-Json)) } } catch { $hist = @() }
  if ($st -and $st.rateHist) { foreach ($x in @($st.rateHist)) { if ($x -and [long]$x.pr -eq $r.pr) { $hist += @{ t = [string]$x.t; p = [double]$x.p; pr = [long]$x.pr } } } }
  $hist = @($hist | Where-Object { $_ -and [long]$_.pr -eq $r.pr })
  $e = [DateTime]::Parse($r.ts).ToUniversalTime().ToString('o'); if (-not (@($hist | Where-Object { [string]$_.t -eq $e }).Count)) { $hist += @{ t = $e; p = $r.p; pr = $r.pr } }
  # Sort hashtable values explicitly; Select-Object -Property would silently erase them.
  $hist = @($hist | Where-Object { $_ -and $_.t } | Sort-Object { [DateTime]::Parse([string]$_.t) })
  $seenT = @{}; $uniq = @(); foreach ($hx in $hist) { $kt = [string]$hx.t; if (-not $seenT[$kt]) { $seenT[$kt] = 1; $uniq += @{ t = $kt; p = [double]$hx.p; pr = [long]$hx.pr } } }
  $hist = @($uniq | Select-Object -Last 12)
  if (-not $DryRun) { ($hist | ConvertTo-Json -Depth 4) | Set-Content $histPath -Encoding utf8 }
  $burn = Burn $hist $nowT; $v = Verdict $r.p $burn
  $line = "used {0}% (reading {1}, {2} min old) burn {3}/min predicted {4}% lanes running: {5} -> {6}" -f $r.p, ([string]$r.ts).Substring(11, 8), $age, $burn, $v.pred, $(if ($laneNames.Count) { $laneNames -join ',' } else { 'none' }), $(if ($v.reached) { 'LINE REACHED' } else { 'ok' })
  if ($DryRun) { Write-Host "DRYRUN guard: $line" }
  if ($r.pr -le $nowEpoch) { if ($DryRun) { Write-Host 'DRYRUN guard: the reading belongs to a window that has already reset; nothing to do' }; return }
  if (-not $v.reached) { if ($laneNames.Count -and ((Get-Date).Minute -lt 2)) { Glog $line }; return }   # lanes running, line not reached: one log line per hour at most
  # the line is reached
  $paused = ($st -and $st.stopLine -and $st.stopLine.active -and [long]$st.stopLine.resetEpoch -eq $r.pr) -or ($gf -and $gf.active -and [long]$gf.resetEpoch -eq $r.pr)
  if ($paused) { if ($DryRun) { Write-Host 'DRYRUN guard: state already says paused-stopline for this window; exit' }; return }
  if (-not $laneNames.Count) {
    if ($DryRun) { Write-Host 'DRYRUN guard: line reached but no Codex lane process is running; nothing to stop' } else { Glog "$line (no lane to stop)" }
    return
  }
  if ($DryRun -or $Cfg.EnableProcessStop -ne $true) {
    $plan = "would stop lanes $($laneNames -join ', ') (process tree), then checkpoint/push if EnableRepoActions is true; no pause state written"
    if ($DryRun) { Write-Host "DRYRUN guard: $plan" } else { Glog "EnableProcessStop=false: $plan"; Ev '-' 'would-stop-at-stopline' $plan }
    return
  }

  # acquire the heartbeat's own lock when it is free so state is written without a race; when a tick is running, act anyway and leave the state to the heartbeat via the guard file
  $hbLock = $null; try { $hbLock = [System.IO.File]::Open("$hb\lock", 'OpenOrCreate', 'ReadWrite', 'None') } catch { $hbLock = $null }
  function GuardCheckpoint($l) {
    if ($Cfg.EnableRepoActions -ne $true) { Glog "EnableRepoActions=false: would checkpoint and push lane $l"; return 'repository actions disabled' }
    $wt = Join-Path $root $l
    if (-not (Test-Path -LiteralPath $wt)) { return 'worktree absent' }
    & git -C $wt rev-parse -q --verify MERGE_HEAD 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) { return 'merge in progress; checkpoint/push skipped' }
    $branch = (& git -C $wt rev-parse --abbrev-ref HEAD 2>$null | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $branch -or $branch -eq 'HEAD' -or -not $branch.StartsWith([string]$Cfg.BranchPrefix)) { return 'branch outside BranchPrefix; checkpoint/push skipped' }
    # -z preserves spaces, Unicode and rename pairs without shell interpolation.
    $raw = & git -C $wt -c core.quotepath=false status --porcelain=v1 -z --untracked-files=all
    if ($LASTEXITCODE -ne 0) { throw "git status failed for $l" }
    $parts = (($raw -join "`n") -split "`0"); $paths = @()
    for ($i = 0; $i -lt $parts.Count; $i++) {
      $entry = $parts[$i]; if ($entry.Length -lt 4) { continue }
      $status = $entry.Substring(0, 2); $p = $entry.Substring(3)
      $pair = @($p)
      if ($status -match '[RC]') { $i++; if ($i -lt $parts.Count) { $pair += $parts[$i] } }
      if (@($pair | Where-Object { $_ -match '(?i)\.(nota|clf|key|csv|env)$|(^|/)\.devdata/|(^|/)\.env' }).Count) {
        # Already-staged protected paths would otherwise enter a commit as well.
        if ($status -notmatch '^( |\?\?)') { throw 'Protected path already staged; refusing checkpoint/push.' }
        continue
      }
      $paths += $pair
    }
    if ($paths.Count) {
      & git -C $wt --literal-pathspecs add -- @paths 2>&1 | Out-Null
      if ($LASTEXITCODE -ne 0) { throw "git add failed for $l" }
      $msg = "WIP checkpoint, lane ${l} (automatic, stop-line guard, $(NowS))`n`nWork in progress; not finished, not checked. $($paths.Count) changed path(s)."
      New-Item -ItemType Directory -Force $out | Out-Null
      $mf = Join-Path $out "$l.checkpoint-msg.txt"
      [IO.File]::WriteAllText($mf, $msg, (New-Object System.Text.UTF8Encoding($false)))
      & git -C $wt commit -F $mf 2>&1 | Out-Null
      if ($LASTEXITCODE -ne 0) { throw "git commit failed for $l" }
    }
    & git -C $wt push -u $Cfg.Remote $branch 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "git push failed for $l" }
    return 'WIP checkpoint (when changes exist) and branch push completed'
  }
  $stopped = @()
  foreach ($k in $laneNames) {
    foreach ($pr in @(Lane-Processes | Where-Object { $_.Lane -eq $k })) {
      & taskkill /PID $pr.ProcessId /T /F 2>&1 | Out-Null
      if ($LASTEXITCODE -ne 0) { Glog "process stop failed for lane $k" }
    }
  }
  Start-Sleep -Seconds 4
  foreach ($k in $laneNames) {
    if (@(Lane-Processes | Where-Object { $_.Lane -eq $k }).Count) { Glog "lane $k still running; checkpoint/push skipped"; continue }
    $repoResult = 'checkpoint/push failed; inspect guard log'
    try { $repoResult = GuardCheckpoint $k } catch { Glog $_.Exception.Message }
    $stopped += $k
    Ev $k 'paused-at-stopline' ("stopped by the stop-line guard (meter {0}%, burn {1} points/min, predicted {2}% in {3} minutes): {4}; resumes at the window reset" -f $r.p, $burn, $v.pred, $LookAheadMin, $repoResult)
  }
  if (-not $stopped.Count) { if ($hbLock) { $hbLock.Close(); $hbLock = $null }; return }
  $resetUtc = [DateTimeOffset]::FromUnixTimeSeconds([long]$r.pr).UtcDateTime.ToString('yyyy-MM-dd HH:mm:ss') + 'Z'
  $g = @{ active = $true; by = 'stopline-guard'; since = (NowS); resetEpoch = $r.pr; resetUtc = $resetUtc; reading = $r.p; reason = $v.reason; burnPerMin = $burn; predicted12 = $v.pred; lanes = $stopped }
  ($g | ConvertTo-Json -Depth 4) | Set-Content $guardFile -Encoding utf8
  if ($hbLock) {
    try {
      $state = ToHash (Get-Content $statePath -Raw | ConvertFrom-Json)
      $sl = $state.stopLine; if (-not $sl) { $sl = @{} }
      $sl.active = $true; $sl.resetEpoch = $r.pr; $sl.resetUtc = $resetUtc; $sl.since = (NowS); $sl.reading = $r.p; $sl.overshoot = [math]::Max(0, [math]::Round($r.p - $StopAtPct, 1)); $sl.burnPerMin = $burn; $sl.predicted12 = $v.pred; $sl.reason = "guard-$($v.reason)"
      $state.stopLine = $sl
      foreach ($k in $stopped) { if ($state.lanes[$k]) { $state.lanes[$k].status = 'paused-stopline'; $state.lanes[$k].resume = $true } }
      ($state | ConvertTo-Json -Depth 8) | Set-Content $statePath -Encoding utf8
    } catch { Glog "state update failed: $($_.Exception.Message)" }
    $hbLock.Close(); $hbLock = $null
    $how = 'state updated under the heartbeat lock'
  } else { $how = 'a heartbeat tick was running: state left to the heartbeat (guard file stopline-guard.json)' }
  Ev '-' 'stopline-pause' ("the stop-line guard stopped lanes {0}: Codex 5-hour window at {1}% used (reading {2}), {3}, burn {4} points a minute, predicted {5}% in {8} minutes; no new lane starts; everything resumes at the window reset {6}. {7}. No banked reset is used." -f ($stopped -join ','), $r.p, $r.ts, $v.reason, $burn, $v.pred, $resetUtc, $how, $LookAheadMin)
  Glog "$line; stopped $($stopped -join ','); $how"
} finally {
  if ($hbLock) { $hbLock.Close() }
  if ($guardLock) { $guardLock.Close() }
}
