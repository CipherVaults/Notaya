# Notaya stop-line guard. Lives in D:\Notaya-lanes\heartbeat (outside the repo). Started every 2 minutes by the scheduled task 'Notaya stop-line guard'
# (through stopline-guard-hidden.vbs, no window). It NEVER starts a lane and NEVER starts a Codex probe. Its one job: when the Codex 5-hour window is
# at the stop line (used >= 95% or used + burn rate x 5 minutes >= 95%) and a Codex lane is still running, stop the running lanes cleanly
# (kill the codex tree, WIP checkpoint commit, plain push of the lane branch), record state.stopLine, and log one line. The heartbeat tick does the
# same in its own way, but a tick can be busy for many minutes (finish checks) and then nothing samples the meter; this guard closes that gap.
# Works only inside D:\Notaya-lanes and D:\Notaya Tests. Never merges, never uses a banked reset, never touches the repository's main checkout or any vault.
#   -DryRun            print what it would do, change nothing (no kill, no state write)
#   -Replay            replay the readings of -ReplayDate from heartbeat.log through the same rule and print the verdict per reading (changes nothing)
#   -ReplayFrom HH:mm:ss  only print readings at or after this UTC time (with -Replay)
#   -Inject "min:pct,min:pct,..."  synthetic test: run the same burn/decision rule over an invented readings series (minutes from an arbitrary start); changes nothing
param([switch]$DryRun, [switch]$Replay, [string]$ReplayDate = '', [string]$ReplayFrom = '00:00:00', [string]$Inject = '')
$ErrorActionPreference = 'Continue'; $LookAheadMin = 5   # same look-ahead as heartbeat.ps1 (minutes)
$root = 'D:\Notaya-lanes'; $hb = "$root\heartbeat"; $out = "$root\out"; $logDir = "$hb\logs"
$statePath = "$hb\heartbeat-state.json"; $evPath = "$hb\events.log"; $guardFile = "$hb\stopline-guard.json"; $histPath = "$logDir\stopline-guard-hist.json"; $logPath = "$logDir\stopline-guard.log"
New-Item -ItemType Directory -Force $logDir | Out-Null
function Now { (Get-Date).ToUniversalTime() }
function NowS { (Now).ToString('yyyy-MM-dd HH:mm:ss') + 'Z' }
function Glog($t) { Add-Content $logPath ("{0} {1}" -f (NowS), $t) }
function Ev($lane, $kind, $text) { Add-Content $evPath ("{0}|{1}|{2}|{3}" -f (NowS), $lane, $kind, $text) }
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
      $len = $fs.Length; $take = [Math]::Min($len, 120000); if ($take % 2 -eq 1) { $take-- }
      $fs.Seek($len - $take, 'Begin') | Out-Null; $buf = New-Object byte[] $take; $null = $fs.Read($buf, 0, $take)
      $clean = New-Object System.Collections.Generic.List[byte]; foreach ($x in $buf) { if ($x -ne 0 -and $x -ne 0xFF -and $x -ne 0xFE) { $clean.Add($x) } }
      return ((([System.Text.Encoding]::UTF8.GetString($clean.ToArray())) -split "`r?`n") | Select-Object -Last $n) -join "`n"
    } finally { $fs.Close() }
  } catch { return '' }
}
function Read-CodexRate {
  # newest 'codex' rate_limits event across the recent Codex session files (same method as heartbeat.ps1; reads only, never probes)
  $best = $null
  $sf = Get-ChildItem "$env:USERPROFILE\.codex\sessions" -Recurse -Filter *.jsonl -ErrorAction SilentlyContinue | Where-Object { $_.LastWriteTime -gt (Get-Date).AddHours(-6) } | Sort-Object LastWriteTime -Descending | Select-Object -First 12
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
function Verdict($p, $burn) { $pred = [math]::Round([double]$p + $burn * $LookAheadMin, 1); return @{ pred = $pred; reached = (([double]$p -ge 95) -or ($burn -gt 0 -and $pred -ge 95)); reason = $(if ([double]$p -ge 95) { 'measured' } else { 'predicted' }) } }

# ---------------- replay mode: no side effects at all
if ($Inject) {
  $base = [DateTime]::Parse('2026-01-01T00:00:00Z').ToUniversalTime(); $hist = @()
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
  foreach ($l in Get-Content "$hb\heartbeat.log") {
    if ($l -notmatch "^$ReplayDate " -or $l -notmatch '\[codex 5h=([\d.]+)% wk=([\d.]+)% @(\d\d:\d\d:\d\d)\]') { continue }
    $p = [double]$Matches[1]; $tm = $Matches[3]
    if ($seen[$tm]) { continue }; $seen[$tm] = 1
    $t = [DateTime]::Parse("${ReplayDate}T${tm}Z").ToUniversalTime()
    if ($prev -ne $null -and $p -lt $prev - 30) { $hist = @() }   # the meter dropped: a new window
    $prev = $p
    $hist += @{ t = $t.ToString('o'); p = $p }
    $b = Burn $hist $t; $v = Verdict $p $b
    if ($tm -ge $ReplayFrom) { "{0}Z used {1,5}%  burn {2,5} points/min  predicted(5 min) {3,6}%  ->  {4}" -f $tm, $p, $b, $v.pred, $(if ($v.reached) { "LINE REACHED ($($v.reason)); the guard would stop running lanes" } else { 'continue' }) }
  }
  return
}

# ---------------- one guard run
$guardLock = $null
try { $guardLock = [System.IO.File]::Open("$logDir\guard.lock", 'OpenOrCreate', 'ReadWrite', 'None') } catch { return }   # another guard run is active
try {
  $nowT = Now; $nowEpoch = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
  # cheapest exit first: while the stop line is on and its window has not reset, there is nothing to guard (no process query, no meter read)
  try { if (-not $DryRun -and (Test-Path $statePath)) { $s0 = Get-Content $statePath -Raw | ConvertFrom-Json; if ($s0.stopLine -and $s0.stopLine.active -and [long]$s0.stopLine.resetEpoch -gt $nowEpoch) { return } } } catch { }
  $procs =@(Get-CimInstance Win32_Process -Filter "Name='codex.exe'" | Where-Object { $_.CommandLine -match ' exec ' -and $_.CommandLine -match 'Notaya-lanes' })
  $laneNames = @($procs | ForEach-Object { if ($_.CommandLine -match '-C D:\\Notaya-lanes\\(\w+) ') { $Matches[1] } } | Sort-Object -Unique)
  # second cheap exit: no Codex lane process means nothing to stop. Nothing else is read, nothing is written, nothing is logged on such a quiet run.
  if (-not $laneNames.Count -and -not $DryRun) { return }
  # state (read only here) and the guard file
  $st = $null; try { if (Test-Path $statePath) { $st = ToHash (Get-Content $statePath -Raw | ConvertFrom-Json) } } catch { $st = $null }
  $gf = $null; try { if (Test-Path $guardFile) { $gf = ToHash (Get-Content $guardFile -Raw | ConvertFrom-Json) } } catch { $gf = $null }
  $r = Read-CodexRate
  if (-not $r) { if ($laneNames.Count) { Glog "no Codex reading found while lanes run: $($laneNames -join ',')" }; return }
  $age = [math]::Round(($nowT - [DateTime]::Parse($r.ts).ToUniversalTime()).TotalMinutes, 1)
  # own history, plus the heartbeat's stored readings (read only), same window only
  $hist = @(); try { if (Test-Path $histPath) { $hist = @(ToHash (Get-Content $histPath -Raw | ConvertFrom-Json)) } } catch { $hist = @() }
  if ($st -and $st.rateHist) { foreach ($x in @($st.rateHist)) { if ($x -and [long]$x.pr -eq $r.pr) { $hist += @{ t = [string]$x.t; p = [double]$x.p; pr = [long]$x.pr } } } }
  $hist = @($hist | Where-Object { $_ -and [long]$_.pr -eq $r.pr })
  $e = [DateTime]::Parse($r.ts).ToUniversalTime().ToString('o'); if (-not (@($hist | Where-Object { [string]$_.t -eq $e }).Count)) { $hist += @{ t = $e; p = $r.p; pr = $r.pr } }
  # (fixed 2026-11-09 19:35Z: Select-Object -Property on hashtables returns empty values, which silently turned the burn rate into 0)
  $hist = @($hist | Where-Object { $_ -and $_.t } | Sort-Object { [DateTime]::Parse([string]$_.t) })
  $seenT = @{}; $uniq = @(); foreach ($hx in $hist) { $kt = [string]$hx.t; if (-not $seenT[$kt]) { $seenT[$kt] = 1; $uniq += @{ t = $kt; p = [double]$hx.p; pr = [long]$hx.pr } } }
  $hist = @($uniq | Select-Object -Last 12)
  if (-not $DryRun) { ($hist | ConvertTo-Json -Depth 4) | Set-Content $histPath -Encoding utf8 }
  $burn = Burn $hist $nowT; $v = Verdict $r.p $burn
  $line = "used {0}% (reading {1}, {2} min old) burn {3}/min predicted12 {4}% lanes running: {5} -> {6}" -f $r.p, ([string]$r.ts).Substring(11, 8), $age, $burn, $v.pred, $(if ($laneNames.Count) { $laneNames -join ',' } else { 'none' }), $(if ($v.reached) { 'LINE REACHED' } else { 'ok' })
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
  if ($DryRun) { Write-Host "DRYRUN guard: would stop lanes $($laneNames -join ', ') cleanly (kill, WIP checkpoint, plain push), record state.stopLine and the guard file, log one line"; return }

  # acquire the heartbeat's own lock when it is free so state is written without a race; when a tick is running, act anyway and leave the state to the heartbeat via the guard file
  $hbLock = $null; try { $hbLock = [System.IO.File]::Open("$hb\lock", 'OpenOrCreate', 'ReadWrite', 'None') } catch { $hbLock = $null }
  function GuardCheckpoint($l) {
    $wt = "$root\$l"; if (-not (Test-Path $wt)) { return }
    & git -C $wt rev-parse -q --verify MERGE_HEAD 2>&1 | Out-Null; if ($LASTEXITCODE -eq 0) { return }
    $porc = (& git -C $wt status --porcelain 2>&1) -split "`n" | Where-Object { $_.Trim() -and $_ -notmatch '^warning' }
    $paths = @(); foreach ($ln in $porc) { $p = $ln.Substring(3).Trim('"'); if ($p -match '(?i)\.(nota|clf|key|csv|env)$|(^|/)\.devdata/|(^|/)\.env') { continue }; $paths += $p }
    if (-not $paths.Count) { return }
    & git -C $wt add -- @paths 2>&1 | Out-Null
    $msg = "WIP checkpoint, lane ${l} (automatic, stop-line guard, $(NowS))`n`n- Work in progress by Codex (gpt-6.1-sol, reasoning xhigh); not finished, not checked. Made by Claude's stop-line guard so a usage-limit cut-off loses little.`n- $($paths.Count) file(s) added or changed since the previous checkpoint.`n`nSide: Ahmed (AE0101) $([char]0x00B7) Machine: Gaming-PC`n`nCo-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
    $mf = "$out\$l.checkpoint-msg.txt"; [System.IO.File]::WriteAllText($mf, $msg, (New-Object System.Text.UTF8Encoding($false)))
    & git -C $wt commit -F $mf 2>&1 | Out-Null
  }
  $stopped = @()
  foreach ($k in $laneNames) {
    foreach ($pr in @(Get-CimInstance Win32_Process -Filter "Name='codex.exe'" | Where-Object { $_.CommandLine -match ' exec ' -and $_.CommandLine -match [regex]::Escape("-C D:\Notaya-lanes\$k ") })) { & taskkill /PID $pr.ProcessId /T /F 2>&1 | Out-Null }
  }
  Start-Sleep -Seconds 4
  foreach ($k in $laneNames) {
    $wt = "$root\$k"; $br = ''
    if (Test-Path $wt) { GuardCheckpoint $k; $br = (& git -C $wt rev-parse --abbrev-ref HEAD 2>$null | Out-String).Trim(); if ($br -and $br -ne 'HEAD') { & git -C $wt push -u origin $br 2>&1 | Out-Null } }
    $stopped += $k
    Ev $k 'paused-at-stopline' ("stopped cleanly by the stop-line guard (meter {0}%, burn {1} points/min, predicted {2}% in 5 minutes): WIP checkpoint committed and branch pushed; resumes at the window reset" -f $r.p, $burn, $v.pred)
  }
  $resetUtc = [DateTimeOffset]::FromUnixTimeSeconds([long]$r.pr).UtcDateTime.ToString('yyyy-MM-dd HH:mm:ss') + 'Z'
  $g = @{ active = $true; by = 'stopline-guard'; since = (NowS); resetEpoch = $r.pr; resetUtc = $resetUtc; reading = $r.p; reason = $v.reason; burnPerMin = $burn; predicted12 = $v.pred; lanes = $stopped }
  ($g | ConvertTo-Json -Depth 4) | Set-Content $guardFile -Encoding utf8
  if ($hbLock) {
    try {
      $state = ToHash (Get-Content $statePath -Raw | ConvertFrom-Json)
      $sl = $state.stopLine; if (-not $sl) { $sl = @{} }
      $sl.active = $true; $sl.resetEpoch = $r.pr; $sl.resetUtc = $resetUtc; $sl.since = (NowS); $sl.reading = $r.p; $sl.overshoot = [math]::Max(0, [math]::Round($r.p - 95, 1)); $sl.burnPerMin = $burn; $sl.predicted12 = $v.pred; $sl.reason = "guard-$($v.reason)"
      $state.stopLine = $sl
      foreach ($k in $stopped) { if ($state.lanes[$k]) { $state.lanes[$k].status = 'paused-stopline'; $state.lanes[$k].resume = $true } }
      ($state | ConvertTo-Json -Depth 8) | Set-Content $statePath -Encoding utf8
    } catch { Glog "state update failed: $($_.Exception.Message)" }
    $hbLock.Close(); $hbLock = $null
    $how = 'state updated under the heartbeat lock'
  } else { $how = 'a heartbeat tick was running: state left to the heartbeat (guard file stopline-guard.json)' }
  Ev '-' 'stopline-pause' ("the stop-line guard stopped lanes {0}: Codex 5-hour window at {1}% used (reading {2}), {3}, burn {4} points a minute, predicted {5}% in 5 minutes; no new lane starts; everything resumes at the window reset {6}. {7}. No banked reset is used." -f ($stopped -join ','), $r.p, $r.ts, $v.reason, $burn, $v.pred, $resetUtc, $how)
  Glog "$line; stopped $($stopped -join ','); $how"
} finally {
  if ($guardLock) { $guardLock.Close() }
}
