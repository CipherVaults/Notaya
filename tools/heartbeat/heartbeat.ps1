# Notaya 1.6 overnight heartbeat. Lives in D:\Notaya-lanes\heartbeat (outside the repo).
# Works only inside D:\Notaya-lanes and D:\Notaya Tests (plus git metadata of the lane worktrees).
# Never merges. Never touches D:\Notaya's checkout, any real vault, or drive E:.
param([switch]$DryRun)
$ErrorActionPreference = 'Continue'
$root = 'D:\Notaya-lanes'; $hb = "$root\heartbeat"; $out = "$root\out"
$legacy = 'C:\Users\Ahmed\AppData\Local\Temp\claude\5e6b91f3da567dce\30e48ea9-8436-4ed3-9046-527043207f7d\scratchpad\out'
$statePath = "$hb\heartbeat-state.json"; $logPath = "$hb\heartbeat.log"; $evPath = "$hb\events.log"
$LookAheadMin = 5   # stop-line look-ahead in minutes (was 12 until 2026-11-09 19:30Z; the guard samples every 2 min, so about 2.5 min of sampling and kill latency is what needs covering)
$maxAlive = 3   # fewer parallel xhigh runs so a quota window finishes lanes instead of cutting all of them off
function Now { (Get-Date).ToUniversalTime() }
function NowS { (Now).ToString('yyyy-MM-dd HH:mm:ss') + 'Z' }
function Ev($lane, $kind, $text) { Add-Content $evPath ("{0}|{1}|{2}|{3}" -f (NowS), $lane, $kind, $text) }
function ToHash($o) {
  if ($o -is [System.Management.Automation.PSCustomObject]) { $h = @{}; foreach ($p in $o.PSObject.Properties) { $h[$p.Name] = ToHash $p.Value }; return $h }
  if ($o -is [System.Array]) { return @($o | ForEach-Object { ToHash $_ }) }
  return $o
}
function Save-State { ($script:state | ConvertTo-Json -Depth 8) | Set-Content $statePath -Encoding utf8 }

# per-lane model (A54): add  model='gpt-6.1-terra'; effort='high'  to a lane entry for small lanes (records, test-script fixes, polish); no field = Sol xhigh. Model ids are checked by a one-line trial before first use.
# lane definitions: base branch, branches merged in, lanes that must be pushed first, brief line range
$lanes = [ordered]@{
  A  = @{ br='AE0101/v1-6-a';  title='small UI fixes (WP-08 to WP-14)';                 deps=@();            base='origin/main'; merge=@();                range='28,34' }
  B  = @{ br='AE0101/v1-6-b';  title='databases inside a page appear in exports (WP-07)'; deps=@();            base='origin/main'; merge=@();                range='36,43' }
  C1 = @{ br='AE0101/v1-6-c1'; title='action core (WP-01)';                              deps=@();            base='origin/main'; merge=@();                range='45,52' }
  D  = @{ br='AE0101/v1-6-d';  title='notaya-bridge.exe, the MCP bridge program';        deps=@();            base='origin/main'; merge=@();                range='54,61' }
  E  = @{ br='AE0101/v1-6-e';  title='AI screens against mock fakes';                    deps=@();            base='origin/main'; merge=@();                range='63,68'; promptFile="$hb\prompt-E-int2.md" }
  C2 = @{ br='AE0101/v1-6-c2'; title='signals, revisions, leases and the bin (WP-02, WP-05)'; deps=@(); base='origin/main'; merge=@(); range='70,76' }
  C4b = @{ br='AE0101/v1-6-c4b'; title='block ops and small saves (WP-02b)';             deps=@();            base='origin/main'; merge=@();                range='78,84' }
  C3 = @{ br='AE0101/v1-6-c3'; title='history and activity log (WP-03, WP-04)';          deps=@('C2');     base='AE0101/v1-6-c2'; merge=@();                range='86,92' }
  AF = @{ br='AE0101/v1-6-af'; title='fixes for the first-pass review of lane A'; deps=@(); base='origin/main'; merge=@(); range='28,34'; promptFile="$hb\prompt-AF.md" }
  RC4 = @{ br='(detached review)'; title='first-pass security and UX review of lane C4b after its fix (read-only)'; deps=@(); base='origin/main'; merge=@(); range='78,84'; review=$true; reviewRef='origin/AE0101/v1-6-c4b'; gate='C4b'; promptFileReview="$hb\prompt-review-C4.md" }
  AG = @{ br='AE0101/v1-6-ag'; title='lane A screenshots script fix and WP-12 database-view restore check'; deps=@(); base='origin/main'; merge=@(); range='28,34'; promptFile="$hb\prompt-AG-fix.md" }
  C4d = @{ br='AE0101/v1-6-c4d'; title='fixes for the first-pass review of lane C4b'; deps=@(); base='origin/main'; merge=@(); range='78,84'; promptFile="$hb\prompt-C4d-merge.md"; fix=$true }
  RC2 = @{ br='(detached review)'; title='first-pass security and UX review of merged lane C2 (read-only)'; deps=@(); base='origin/main'; merge=@(); range='70,76'; review=$true; reviewRef='origin/main'; promptFileReview="$hb\prompt-review-C2.md" }
  C2d = @{ br='AE0101/v1-6-c2d'; title='fixes for the first-pass review of lane C2 and the double-save regression'; deps=@(); base='origin/main'; merge=@(); range='70,76'; promptFile="$hb\prompt-C2d-merge.md" }
  RC5 = @{ br='(detached review)'; title='first-pass SECURITY review of lane C5 and the bridge D together (read-only)'; deps=@(); base='origin/main'; merge=@(); range='95,101'; review=$true; reviewRef='origin/AE0101/v1-6-c5'; promptFileReview="$hb\prompt-review-C5.md" }
  C3u = @{ br='AE0101/v1-6-c3u'; title='fix the c3-ui-check page-options selector'; deps=@(); base='origin/main'; merge=@(); range='86,92'; promptFile="$hb\prompt-C3u.md" }
  W = @{ br='AE0101/v1-6-w'; title='whole-app test plan (phase 1)'; deps=@(); base='origin/main'; merge=@(); range='28,34'; promptFile="$hb\prompt-W4.md" }
  P1 = @{ br='AE0101/v1-6-p1'; title='large-database speed, stage 1 (P1 to P5, WP-16)'; deps=@(); base='origin/main'; merge=@(); range='28,34'; promptFile="$hb\prompt-P1b.md" }
  S = @{ br='AE0101/v1-6-s'; title='security pass (lane S, WP-17), area by area'; deps=@(); base='origin/main'; merge=@(); range='28,34'; promptFile="$hb\prompt-S3.md"; model='gpt-6.1-sol'; effort='xhigh' }
  X1 = @{ br='AE0101/v1-6-x1'; title='fix defect X-1: blank white window at Start with Windows (reported by Mohamed Sayed)'; deps=@(); base='origin/main'; merge=@(); range='28,34'; promptFile="$hb\prompt-X1c.md"; credit='Reported by Mohamed Sayed (AbyssWatcher2) via Ahmed, 2026-11-09 07:38:18 UTC' }
  SF1 = @{ br='AE0101/v1-6-sf1'; title='security fixes S1-M1 and S1-M2 from the lane S area 1 review'; deps=@(); base='origin/main'; merge=@(); range='28,34'; promptFile="$hb\prompt-SF1.md" }
  SF2 = @{ br='AE0101/v1-6-sf2'; title='native confirmations for five rare steps and a keyfile locality rule (A51, S1-H1, S1-H2, S1-Q1)'; deps=@('SF1'); base='origin/main'; merge=@(); range='28,34'; promptFile="$hb\prompt-SF2.md" }
  SF3 = @{ br='AE0101/v1-6-sf3'; title='anchor and purge fixes for S2-M1, S2-M2, S2-M3 (tamper evidence, delete for good)'; deps=@(); base='origin/main'; merge=@(); range='28,34'; promptFile="$hb\prompt-SF3.md" }
  R1 = @{ br='AE0101/v1-6-r1'; title='records catch-up: Session Summary, ledger rows, EX entries, claims, consistency pass (docs only)'; deps=@(); base='origin/main'; merge=@(); range='28,34'; promptFile="$hb\prompt-R1.md" }
  N1 = @{ br='AE0101/v1-6-n1'; title='Notion import with real data: read-only investigation (WP-19, A52)'; deps=@(); base='origin/main'; merge=@(); range='28,34'; promptFile="$hb\prompt-N1.md" }
  N2 = @{ br='AE0101/v1-6-n2'; title='Notion import fixes: correctness and arrangement (WP-19, A53, lane N2)'; deps=@(); base='origin/main'; merge=@('origin/AE0101/v1-6-n1'); range='28,34'; promptFile="$hb\prompt-N2b.md"; model='gpt-6.1-sol'; effort='xhigh' }
  P6b = @{ br='AE0101/v1-6-p6b'; title='fixes to P6 before it merges: undo of an added row, auto-scroll drop, check-script defects (lane P6b)'; deps=@(); base='origin/AE0101/v1-6-p6'; merge=@('origin/main'); range='28,34'; promptFile="$hb\prompt-P6b.md"; model='gpt-6.1-sol'; effort='xhigh' }
  P6 = @{ br='AE0101/v1-6-p6'; title='virtual rows in the database Table (WP-16 stage 2, P6)'; deps=@(); base='origin/main'; merge=@(); range='28,34'; promptFile="$hb\prompt-P6.md" }
  RA = @{ br='(detached review)'; title='first-pass security and UX review of merged lane A (read-only)'; deps=@(); base='origin/main'; merge=@(); range='28,34'; review=$true }
  C5 = @{ br='AE0101/v1-6-c5'; title='the AI server (WP-06 a,b,c,e,f,g,h)';              deps=@('C3');     base='AE0101/v1-6-c3'; merge=@();                range='95,101'; promptFile="$hb\prompt-C5d.md" }
}

# ---- lock (one heartbeat at a time)
$lockFs = $null
try { $lockFs = [System.IO.File]::Open("$hb\lock", 'OpenOrCreate', 'ReadWrite', 'None') } catch { Add-Content $logPath ("{0} skipped: another heartbeat is running" -f (NowS)); exit 0 }

try {
  # ---- state
  if (Test-Path $statePath) { $state = ToHash (Get-Content $statePath -Raw | ConvertFrom-Json) } else { $state = @{} }
  if (-not $state.lanes) {
    $state.lanes = @{}
    foreach ($l in 'A','B','C1','D','E') { $state.lanes[$l] = @{ status='running'; legacy=$true; followupDone=$false; startedUtc='2026-11-08 00:24:00Z' } }
    foreach ($l in 'C2','C4','C3','C5') { $state.lanes[$l] = @{ status='pending'; legacy=$false; followupDone=$false } }
  }
  if (-not $state.limit) { $state.limit = @{ active=$false; resetUtc=$null; note='' } }
  foreach ($k in $lanes.Keys) { if (-not $state.lanes[$k]) { $state.lanes[$k] = @{ status='pending'; legacy=$false; followupDone=$false } } }

  # ---- process table
  $procs = @(Get-CimInstance Win32_Process -Filter "Name = 'codex.exe'" | Where-Object { $_.CommandLine -match ' exec ' -and $_.CommandLine -match 'Notaya-lanes' })
  function Alive($l) { return (@($procs | Where-Object { $_.CommandLine -match [regex]::Escape("-C D:\Notaya-lanes\$l ") }).Count -gt 0) }
  function PathsFor($l, $tag) {
    $ls = $state.lanes[$l]
    if ($ls.legacy -and $tag -eq $l) { return @{ md="$legacy\$tag.md"; log="$legacy\$tag.log"; err="$legacy\$tag.err" } }
    return @{ md="$out\$tag.md"; log="$out\$tag.log"; err="$out\$tag.err" }
  }
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
      if ($slx -and $slx.active -and $slx.resetEpoch -and ([string]$slx.reason -notmatch 'predicted' -or [long]$state.liftedForWindow -eq [long]$slx.resetEpoch -or [double]$state.rate.p -ge 95)) { $rem = [long]$slx.resetEpoch - [DateTimeOffset]::UtcNow.ToUnixTimeSeconds(); return ($rem -gt -60 -and $rem -le 19800) }
    } catch { }
    return $false
  }
  function ModelNote($l) {
    # model, effort and the Codex meter before (stored at lane start) and now (last reading), for the ledger: real Terra versus Sol cost (A54)
    $ls = $state.lanes[$l]; $mdl = [string]$lanes[$l].model; if (-not $mdl) { $mdl = 'gpt-6.1-sol' }; $eff = [string]$lanes[$l].effort; if (-not $eff) { $eff = 'xhigh' }
    $now5 = ''; $nowW = ''; $at = ''; if ($state.rate) { $now5 = $state.rate.p; $nowW = $state.rate.w; $at = ([string]$state.rate.ts); if ($at.Length -ge 19) { $at = $at.Substring(11, 8) + 'Z' } }
    return (" | model={0} effort={1} | Codex meter 5h {2}% -> {3}%, weekly {4}% -> {5}% (last reading {6})" -f $mdl, $eff, $ls.meterBefore5h, $now5, $ls.meterBeforeW, $nowW, $at)
  }
  function Read-CodexRate {
    # newest 'codex' rate_limits event across ALL recent Codex session files (not only the active lane's)
    $best = $null
    $sf = Get-ChildItem "$env:USERPROFILE\.codex\sessions" -Recurse -Filter *.jsonl -ErrorAction SilentlyContinue | Where-Object { $_.LastWriteTime -gt (Get-Date).AddHours(-6) } | Sort-Object LastWriteTime -Descending | Select-Object -First 12
    foreach ($s in $sf) {
      try {
        $tt = Tail-Text $s.FullName 4000
        $ln = @($tt -split "`n" | Where-Object { $_ -match '"limit_id":"codex"' -and $_ -match '"primary":\{' }) | Select-Object -Last 1
        if ($ln) { $o = $ln | ConvertFrom-Json; $c = @{ f = $s.Name; ts = [string]$o.timestamp; p = $o.payload.rate_limits.primary.used_percent; w = $o.payload.rate_limits.secondary.used_percent; pr = $o.payload.rate_limits.primary.resets_at; wr = $o.payload.rate_limits.secondary.resets_at; plan = [string]$o.payload.rate_limits.plan_type }; if (-not $best -or $c.ts -gt $best.ts) { $best = $c } }
      } catch { }
    }
    return $best
  }
  function Get-CodexRateFresh {
    # newest event; if it is older than 15 minutes (no lane writing events) take ONE tiny non-ephemeral codex exec probe (~29k input tokens, low effort) and read its event
    $r = Read-CodexRate
    $stale = $true; if ($r) { $stale = (((Now) - [datetime]::Parse($r.ts).ToUniversalTime()).TotalMinutes -gt 15) }
    # while the stop line is on (nothing runs until the window resets) a Codex probe tells nothing new: skip it (2026-11-09, Ahmed)
    if (PausedWindow) { $stale = $false }
    if ($stale -and -not $DryRun) {
      try {
        New-Item -ItemType Directory -Force 'D:\Notaya Tests\codex-probe' | Out-Null; Push-Location 'D:\Notaya Tests\codex-probe'
        cmd /c "`"C:\Users\Ahmed\AppData\Local\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\npm\codex.cmd`" exec --json -m gpt-6.1-sol -c model_reasoning_effort=`"low`" -s read-only --skip-git-repo-check `"Reply with ok.`" > `"$out\codex-probe.jsonl`" 2>&1"
        Pop-Location
        $r2 = Read-CodexRate; if ($r2) { $r2.probe = $true; $r = $r2 }
      } catch { Pop-Location -ErrorAction SilentlyContinue }
    }
    return $r
  }  function AliveNow($k) { return (@(Get-CimInstance Win32_Process -Filter "Name='codex.exe'" | Where-Object { $_.CommandLine -match ' exec ' -and $_.CommandLine -match [regex]::Escape("-C D:\Notaya-lanes\$k ") }).Count -gt 0) }
  function StopLaneClean($k) {
    # clean stop at the 5% stop line: kill the lane's codex tree, then WIP checkpoint commit + plain push of the lane branch (no force)
    $ld = $lanes[$k]; $wt = "$root\$k"
    foreach ($pr in @(Get-CimInstance Win32_Process -Filter "Name='codex.exe'" | Where-Object { $_.CommandLine -match ' exec ' -and $_.CommandLine -match [regex]::Escape("-C D:\Notaya-lanes\$k ") })) { & taskkill /PID $pr.ProcessId /T /F 2>&1 | Out-Null }
    Start-Sleep -Seconds 4
    if (-not $ld.review -and (Test-Path $wt)) { Checkpoint $k; & git -C $wt push -u origin $ld.br 2>&1 | Out-Null }
    $state.lanes[$k].status = 'paused-stopline'; $state.lanes[$k].resume = $true
  }
  # 2026-11-09 13:30Z (Ahmed, via the coordinator): predictive stop line. The stop line used to fire only when the measured meter was already >= 95%; with three xhigh lanes burning about 3 points a minute and a 90 s sample interval the meter overshot (97, 98, 99%). Now the burn rate is estimated from the stored readings of the same window and the line counts as reached when used% + rate x 12 minutes >= 95.
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
    # Ahmed 2026-11-08 11:48:49Z: use the Codex 5-hour window until 5% is left (95% used), then stop; resume at the reset. Banked resets are never used.
    if ($DryRun -or -not $r) { return }
    $nowEpoch = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    RateHistAdd $r; $burn = BurnPerMin $r; $pred = [double]$r.p + $burn * $LookAheadMin; $reached = ($r.p -ge 95) -or ($burn -gt 0 -and $pred -ge 95)
    $sl = $state.stopLine; if (-not $sl) { $sl = @{ active = $false }; $state.stopLine = $sl }
    # the stop-line guard (separate scheduled task, stopline-guard.ps1) may have stopped lanes while this tick was busy: adopt its record
    $gfp = "$hb\stopline-guard.json"
    if (Test-Path $gfp) {
      try {
        $gg = ToHash (Get-Content $gfp -Raw | ConvertFrom-Json)
        if ($gg.active -and [long]$gg.resetEpoch -gt $nowEpoch -and -not $sl.active) {
          $sl.active = $true; $sl.resetEpoch = [long]$gg.resetEpoch; $sl.resetUtc = $gg.resetUtc; $sl.since = $gg.since; $sl.reading = $gg.reading; $sl.overshoot = [math]::Max(0, [math]::Round([double]$gg.reading - 95, 1)); $sl.burnPerMin = $gg.burnPerMin; $sl.predicted12 = $gg.predicted12; $sl.reason = "guard-$($gg.reason)"
          foreach ($gk in @($gg.lanes)) { if ($state.lanes[$gk]) { $state.lanes[$gk].status = 'paused-stopline'; $state.lanes[$gk].resume = $true } }
          Ev '-' 'stopline-pause' ("adopted the stop-line guard's record: lanes {0} were stopped by the guard at {1}% ({2}); everything resumes at the window reset {3}" -f (@($gg.lanes) -join ','), $gg.reading, $gg.reason, $gg.resetUtc)
        }
      } catch { }
    }
    if ($reached -and $r.pr -gt $nowEpoch) {
      if (-not $sl.active) {
        $sl.active = $true; $sl.resetEpoch = $r.pr; $sl.resetUtc = [DateTimeOffset]::FromUnixTimeSeconds([long]$r.pr).UtcDateTime.ToString('yyyy-MM-dd HH:mm:ss') + 'Z'; $sl.since = NowS; $sl.reading = $r.p; $sl.overshoot = [math]::Max(0, [math]::Round([double]$r.p - 95, 1)); $sl.burnPerMin = $burn; $sl.predicted12 = [math]::Round($pred, 1); $sl.reason = $(if ($r.p -ge 95) { 'measured' } else { 'predicted' })
        Ev '-' 'stopline-pause' ("Codex 5-hour window at {0}% used (reading {1}); the 95% stop line is reached ({4}: burn {5} points a minute, predicted {6}% in 5 minutes; overshoot {2} points): no new lane starts, running lanes are stopped cleanly; everything resumes at the window reset {3}. No banked reset is used." -f $r.p, $r.ts, $sl.overshoot, $sl.resetUtc, $sl.reason, $burn, $sl.predicted12)
      }
      foreach ($k in @($lanes.Keys)) { $st = $state.lanes[$k].status; if (($st -eq 'running' -or $st -eq 'followup-running') -and (AliveNow $k)) { StopLaneClean $k; Ev $k 'paused-at-stopline' (("stopped cleanly at the 5% stop line (meter {0}%): WIP checkpoint committed and branch pushed; resumes at the window reset" -f $r.p) + (ModelNote $k)) } }
    } elseif ($sl.active -and ($nowEpoch -ge [long]$sl.resetEpoch -or [long]$r.pr -ne [long]$sl.resetEpoch)) {
      $sl.active = $false
      if (Test-Path "$hb\stopline-guard.json") { try { Remove-Item "$hb\stopline-guard.json" -Force } catch { } }
      foreach ($k in @($lanes.Keys)) { if ($state.lanes[$k].status -eq 'paused-stopline') { $state.lanes[$k].status = 'pending'; $state.lanes[$k].resume = $true } }
      Ev '-' 'stopline-resume' ("Codex 5-hour window is at {0}% used again / its reset time has passed; lanes paused at the stop line are resumed" -f $r.p)
    } elseif ($sl.active -and [string]$sl.reason -match 'predicted' -and [long]$r.pr -eq [long]$sl.resetEpoch -and [long]$state.liftedForWindow -ne [long]$r.pr -and [double]$r.p -lt 90 -and ([double]$r.p + $burn * $LookAheadMin) -lt 95 -and -not (@(Get-CimInstance Win32_Process -Filter "Name='codex.exe'" | Where-Object { $_.CommandLine -match ' exec ' -and $_.CommandLine -match 'Notaya-lanes' }).Count)) {
      $state.liftedForWindow = [long]$r.pr   # at most ONE predicted lift per window: the first lift (19:33Z) gave S two minutes of work and 5 points of meter before the line fired again at 89%, i.e. mostly relaunch cost
      # 2026-11-09 19:40Z (Ahmed: use as much of the window as possible): a line held only because of the PREDICTED rule lifts again when nothing runs, used% < 90 and used + burn x look-ahead < 95. A line held at MEASURED >= 95 stays until the reset.
      $sl.active = $false
      if (Test-Path "$hb\stopline-guard.json") { try { Remove-Item "$hb\stopline-guard.json" -Force } catch { } }
      foreach ($k in @($lanes.Keys)) { if ($state.lanes[$k].status -eq 'paused-stopline') { $state.lanes[$k].status = 'pending'; $state.lanes[$k].resume = $true } }
      Ev '-' 'stopline-resume' ("predicted stop line lifted: no lane runs, Codex 5-hour window at {0}% used (< 90), burn {1} points a minute, used + {2} x burn < 95; lanes paused at the stop line are resumed (at most 2 at once from 60%). A measured 95% line would have stayed until the reset." -f $r.p, $burn, $LookAheadMin)
    }
  }  # ---- (b) limit detection from newest logs (only lines near the end of recently written logs)
  $limRx = 'hit your usage limit|usage limit (has been )?reached|reached your .{0,30}limit|rate limit exceeded|rate_limit_exceeded|insufficient_quota|429 Too Many Requests'
  $found = $null
  $cut = (Get-Date).AddHours(-2)
  if ($state.limitIgnoreBefore) { $ib = [datetime]::Parse([string]$state.limitIgnoreBefore).ToLocalTime(); if ($ib -gt $cut) { $cut = $ib } }
  foreach ($d in $out, $legacy) {
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
  # Banked resets are NEVER used or assumed here. At weekly 100% the heartbeat only pauses and reports; Ahmed decides.
  $rate = Get-CodexRateFresh
  try {
    $sf = Get-ChildItem "$env:USERPROFILE\.codex\sessions" -Recurse -Filter *.jsonl -ErrorAction SilentlyContinue | Where-Object { $_.LastWriteTime -gt (Get-Date).AddHours(-6) } | Sort-Object LastWriteTime -Descending | Select-Object -First 6
    foreach ($s in $sf) {
      $tt = Tail-Text $s.FullName 60
      $ln = @($tt -split "`n" | Where-Object { $_ -match '"limit_id":"codex"' -and $_ -match '"primary":\{' }) | Select-Object -Last 1
      if ($ln) { $o = $ln | ConvertFrom-Json; $c = @{ f = $s.Name; ts = [string]$o.timestamp; p = $o.payload.rate_limits.primary.used_percent; w = $o.payload.rate_limits.secondary.used_percent; pr = $o.payload.rate_limits.primary.resets_at; wr = $o.payload.rate_limits.secondary.resets_at }; if (-not $rate -or $c.ts -gt $rate.ts) { $rate = $c } }
    }
  } catch { }
  if ($rate) {
    $state.rate = $rate
    $nowEpoch = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    if ($rate.w -ge 100 -and $rate.wr -gt $nowEpoch) {
      if (-not $state.weeklyLimit) { $state.weeklyLimit = $true; Ev '-' 'weekly-limit' ("Codex WEEKLY limit is at {0}% (reading {1}); heartbeat PAUSED, no banked reset is used or assumed; Ahmed decides" -f $rate.w, $rate.ts) }
      $limitActive = $true
    } elseif ($state.weeklyLimit) { $state.weeklyLimit = $false; Ev '-' 'weekly-resume' ("weekly reading is {0}% again (reading {1}); heartbeat resumes" -f $rate.w, $rate.ts) }
    if ($rate.p -ge 99.9 -and $rate.pr -gt $nowEpoch -and -not $state.limit.active) {
      $rs = [DateTimeOffset]::FromUnixTimeSeconds([long]$rate.pr).UtcDateTime.ToString('yyyy-MM-dd HH:mm:ss') + 'Z'
      $state.limit = @{ active = $true; resetUtc = $rs; note = ("rate_limits reading {0}: 5-hour window {1}% used" -f $rate.ts, $rate.p) }
      Ev '-' 'limit-pause' ("5-hour window at {0}% by Codex's own reading; reset {1}" -f $rate.p, $rs); $limitActive = $true
    }
  }

  # ---- Claude plan meter (exact). A tiny non-interactive call (haiku, no tools, about 900 tokens) returns a rate_limit_event with the
  # account's 5-hour and weekly utilization. Refresh claude-rate.json when it is older than 25 minutes (a status-line file counts too).
  # utilization is returned as a fraction (0.28 = 28%); we store percent = utilization x 100.
  $probeNote = ''
  try {
    $crf = "$hb\claude-rate.json"; $needs = $true
    if (Test-Path $crf) { try { $needs = (((Now) - ([datetime]::Parse((Get-Content $crf -Raw | ConvertFrom-Json).ts)).ToUniversalTime()).TotalMinutes -gt 5) } catch { $needs = $true } }
    # while everything is paused at the stop line the Claude meter is not needed either: skip the probe unless the stored reading is older than 2 hours; at the reset (+1 minute) PausedWindow ends and the normal 5-minute rule resumes
    if ($needs -and (PausedWindow) -and (Test-Path $crf)) { try { if ((((Now) - ([datetime]::Parse((Get-Content $crf -Raw | ConvertFrom-Json).ts)).ToUniversalTime()).TotalMinutes -le 120)) { $needs = $false } } catch { } }
    if ($needs -and -not $DryRun) {
      $cl = $null; $gc = Get-Command claude -ErrorAction SilentlyContinue; if ($gc) { $cl = $gc.Source }; if (-not $cl) { $cl = 'C:\Users\Ahmed\.local\bin\claude.exe' }
      $pd = 'D:\Notaya Tests\claude-probe'; New-Item -ItemType Directory -Force $pd | Out-Null
      $po = "$out\claude-probe-stream.jsonl"
      Push-Location $pd
      cmd /c "`"$cl`" -p ok --model haiku --tools `"`" --strict-mcp-config --disable-slash-commands --no-session-persistence --system-prompt `"Reply with ok.`" --output-format stream-json --verbose > `"$po`" 2>&1"
      Pop-Location
      $evl = Get-Content $po -ErrorAction SilentlyContinue | Where-Object { $_ -match 'unifiedWindows' } | Select-Object -Last 1
      if ($evl) {
        $ei = ($evl | ConvertFrom-Json).rate_limit_info.unifiedWindows
        $raw = Get-Content $po -Raw
        function LastNum($rx) { $m = [regex]::Matches($raw, $rx); if ($m.Count) { return [int]$m[$m.Count - 1].Groups[1].Value } else { return 0 } }
        $tin = LastNum '"input_tokens":(\d+)'; $tcc = LastNum '"cache_creation_input_tokens":(\d+)'; $tcr = LastNum '"cache_read_input_tokens":(\d+)'; $tout = LastNum '"output_tokens":(\d+)'
        $probeNote = ('one tiny claude -p probe (haiku, no tools): input {0}, cache-creation {1}, cache-read {2}, output {3} tokens' -f $tin, $tcc, $tcr, $tout)
        $obj = [ordered]@{ ts = (Now).ToString('o'); source = 'claude -p probe, rate_limit_event (utilization x 100)'; five_hour = [ordered]@{ used_percentage = [math]::Round([double]$ei.five_hour.utilization * 100, 1); raw_utilization = [double]$ei.five_hour.utilization; resets_at = $ei.five_hour.resetsAt }; seven_day = [ordered]@{ used_percentage = [math]::Round([double]$ei.seven_day.utilization * 100, 1); raw_utilization = [double]$ei.seven_day.utilization; resets_at = $ei.seven_day.resetsAt }; probe_usage = $probeNote }
        $tmp = "$crf.tmp"; ($obj | ConvertTo-Json -Depth 4) | Set-Content $tmp -Encoding UTF8; Move-Item $tmp $crf -Force
      } else { Add-Content $logPath ("{0} claude probe returned no rate_limit_event (see {1})" -f (NowS), $po) }
    }
  } catch { Pop-Location -ErrorAction SilentlyContinue; Add-Content $logPath ("{0} claude probe failed: {1}" -f (NowS), ($_.Exception.Message -replace '\s+', ' ')) }
  # ---- Claude quota control: weekly >= 70% => claude_pause (the cloud thread stops Claude-side work until the weekly reset); >= 60% => warning.
  try {
    if (Test-Path "$hb\claude-rate.json") {
      $cj0 = Get-Content "$hb\claude-rate.json" -Raw | ConvertFrom-Json
      $age0 = (Now) - [datetime]::Parse($cj0.ts).ToUniversalTime(); $fresh0 = ($age0.TotalMinutes -le 30)
      $pw = [double]$cj0.seven_day.used_percentage; $p5 = [double]$cj0.five_hour.used_percentage
      $wr = [DateTimeOffset]::FromUnixTimeSeconds([long]$cj0.seven_day.resets_at).UtcDateTime.ToString('yyyy-MM-dd HH:mm:ss') + 'Z'; $r5s = [DateTimeOffset]::FromUnixTimeSeconds([long]$cj0.five_hour.resets_at).UtcDateTime.ToString('yyyy-MM-dd HH:mm:ss') + 'Z'
      $cq = $state.claude_quota_state; if (-not $cq) { $cq = @{ claude_pause = $false; claude_warn = $false } }
      if ($fresh0) {
        $np = ($pw -ge 100); $nw = ($pw -ge 60)   # A50 (Ahmed 2026-11-09): the 70% Claude pause line is lifted for 1.6; 60% stays as information only
        if ($np -ne [bool]$cq.claude_pause) { Ev '-' $(if ($np) { 'claude-pause' } else { 'claude-resume' }) ("Claude WEEKLY plan meter {0}% (pause line 100% since A50; 60% is information only); weekly resets {1}; 5-hour {2}% resets {3}. {4}" -f $pw, $wr, $p5, $r5s, $(if ($np) { 'claude_pause=true: stop Claude-side work until the weekly reset.' } else { 'claude_pause=false.' })) }
        elseif (-not $np -and ($nw -ne [bool]$cq.claude_warn)) { Ev '-' $(if ($nw) { 'claude-warn' } else { 'claude-warn-clear' }) ("Claude WEEKLY plan meter {0}% (warning at 60%, pause at 70%); weekly resets {1}" -f $pw, $wr) }
        $cq.claude_pause = $np; $cq.claude_warn = $nw
      }
      $cq.weekly_used_pct = $pw; $cq.five_hour_used_pct = $p5; $cq.weekly_resets_utc = $wr; $cq.five_hour_resets_utc = $r5s; $cq.reading_ts = [string]$cj0.ts; $cq.fresh = $fresh0; $cq.source = [string]$cj0.source
      $state.claude_quota_state = $cq
    }
  } catch { Add-Content $logPath ("{0} claude quota control failed: {1}" -f (NowS), ($_.Exception.Message -replace '\s+', ' ')) }  # ---- ledger rows: one row per tick for each meter we can read, appended to a LOCAL csv outside the repo.
  # Columns match docs/ledger/quota-readings.csv: time_utc,tool,account,limit,reading,resets_utc,read_by,source,notes
  try {
    $ledgerDir = "$root\ledger"; New-Item -ItemType Directory -Force $ledgerDir | Out-Null
    $csv = "$ledgerDir\quota-readings-pc.csv"
    if (-not (Test-Path $csv)) { Set-Content $csv 'time_utc,tool,account,limit,reading,resets_utc,read_by,source,notes' -Encoding UTF8 }
    function Q($v) { '"' + ([string]$v).Replace('"', '""') + '"' }
    $nowT = (Now).ToString('yyyy-MM-ddTHH:mm') + 'Z'
    if ($rate) {
      $age = (Now) - [datetime]::Parse($rate.ts).ToUniversalTime()
      $stale = ''; if ($age.TotalMinutes -gt 15) { $stale = 'stale (newest Codex event is {0} min old)' -f [int]$age.TotalMinutes }
      $r5 = [DateTimeOffset]::FromUnixTimeSeconds([long]$rate.pr).UtcDateTime.ToString('yyyy-MM-ddTHH:mm:ss') + 'Z'; $rw = [DateTimeOffset]::FromUnixTimeSeconds([long]$rate.wr).UtcDateTime.ToString('yyyy-MM-ddTHH:mm:ss') + 'Z'
      $cells = @($nowT, 'Codex', 'Ahmed (AE0101)', 'five-hour window and weekly limit', ('5-hour {0}% used, weekly {1}% used' -f $rate.p, $rate.w), ('5-hour {0}; weekly {1}' -f $r5, $rw), 'heartbeat tick (Gaming-PC)', ('heartbeat tick, Codex rate_limits event {0} at {1}{2}' -f $rate.f, $rate.ts, $(if ($rate.probe) { ' (read via a tiny codex exec probe because no session event was newer than 15 minutes)' } else { '' })), $stale)
    } else {
      $cells = @($nowT, 'Codex', 'Ahmed (AE0101)', 'five-hour window and weekly limit', 'no Codex rate_limits event found in the last 6 hours', 'not recorded', 'heartbeat tick (Gaming-PC)', 'heartbeat tick, ~/.codex/sessions', 'stale')
    }
    Add-Content $csv (($cells | ForEach-Object { Q $_ }) -join ',') -Encoding UTF8
    if ($rate -and $rate.probe) { $cx = @($nowT, 'Codex', 'Ahmed (AE0101)', 'usage by the heartbeat probe', 'one tiny codex exec probe (gpt-6.1-sol, low effort): about 29,000 input tokens (about 13,700 cached), 5 output tokens', 'not applicable', 'heartbeat tick (Gaming-PC)', 'heartbeat tick, codex exec probe', ''); Add-Content $csv (($cx | ForEach-Object { Q $_ }) -join ',') -Encoding UTF8 }
    $cr = "$hb\claude-rate.json"
    if (Test-Path $cr) {
      $cj = Get-Content $cr -Raw | ConvertFrom-Json; $cage = (Now) - [datetime]::Parse($cj.ts).ToUniversalTime()
      $cs = ''; if ($cage.TotalMinutes -gt 15) { $cs = 'stale (newest Claude status-line reading is {0} min old)' -f [int]$cage.TotalMinutes }
      $c5 = [DateTimeOffset]::FromUnixTimeSeconds([long]$cj.five_hour.resets_at).UtcDateTime.ToString('yyyy-MM-ddTHH:mm:ss') + 'Z'; $cw = [DateTimeOffset]::FromUnixTimeSeconds([long]$cj.seven_day.resets_at).UtcDateTime.ToString('yyyy-MM-ddTHH:mm:ss') + 'Z'
      $cc = @($nowT, 'Claude', 'Ahmed (AE0101)', 'five-hour window and weekly limit', ('5-hour {0}% used, weekly {1}% used' -f $cj.five_hour.used_percentage, $cj.seven_day.used_percentage), ('5-hour {0}; weekly {1}' -f $c5, $cw), 'heartbeat tick (Gaming-PC)', $(if ($cj.source) { 'heartbeat tick, Claude ' + $cj.source + ' at ' + $cj.ts } else { 'heartbeat tick, Claude Code status-line rate_limits written at ' + $cj.ts }), $cs)
      Add-Content $csv (($cc | ForEach-Object { Q $_ }) -join ',') -Encoding UTF8
      if ($probeNote) { $cp = @($nowT, 'Claude', 'Ahmed (AE0101)', 'usage by the heartbeat probe', $probeNote, 'not applicable', 'heartbeat tick (Gaming-PC)', 'heartbeat tick, claude -p probe stream-json', ''); Add-Content $csv (($cp | ForEach-Object { Q $_ }) -join ',') -Encoding UTF8 }
    }
  } catch { Add-Content $logPath ("{0} ledger row failed: {1}" -f (NowS), $_.Exception.Message) }
  # ---- hourly push of the new ledger rows: branch AE0101/ledger-readings, worktree D:\Notaya-lanes\LEDGER, ONE commit, no force.
  try {
    if (-not $DryRun -and (Test-Path "$root\ledger\quota-readings-pc.csv")) {
      $lg = $state.ledger; if (-not $lg) { $lg = @{ lastLine = 1; lastPushUtc = $null }; $state.ledger = $lg }
      $due = $true; if ($lg.lastPushUtc) { $due = (((Now) - [datetime]::Parse($lg.lastPushUtc).ToUniversalTime()).TotalMinutes -ge 60) }
      $all = @(Get-Content "$root\ledger\quota-readings-pc.csv" -Encoding UTF8)
      $newRows = @(); if ($all.Count -gt [int]$lg.lastLine) { $newRows = @($all[[int]$lg.lastLine..($all.Count - 1)]) }
      if ($due -and $newRows.Count -gt 0) {
        $lw = "$root\ledger-wt"; $lb = 'AE0101/ledger-readings'
        & git -C D:\Notaya fetch origin 2>&1 | Out-Null
        if (-not (Test-Path $lw)) {
          $remoteHas = [bool](& git -C D:\Notaya ls-remote --heads origin $lb 2>$null)
          $base = 'origin/main'; if ($remoteHas) { $base = "origin/$lb" }
          & git -C D:\Notaya worktree add -B $lb $lw $base 2>&1 | Out-Null
        }
        $mg = & git -C $lw merge --no-edit origin/main 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) {
          # keep both sides: only docs/ledger/quota-readings.csv may conflict (everyone appends rows at its end)
          $cf = @(& git -C $lw diff --name-only --diff-filter=U 2>$null | Where-Object { $_ })
          if ($cf.Count -eq 1 -and $cf[0] -eq 'docs/ledger/quota-readings.csv') {
            $cp = "$lw\docs\ledger\quota-readings.csv"; $seen = New-Object System.Collections.Generic.HashSet[string]; $keep = New-Object System.Collections.Generic.List[string]
            foreach ($ln in [System.IO.File]::ReadAllLines($cp)) { if ($ln -match '^(<<<<<<<|=======|>>>>>>>)') { continue }; if ($seen.Add($ln)) { $keep.Add($ln) } }
            [System.IO.File]::WriteAllText($cp, (($keep -join "`n") + "`n"), (New-Object System.Text.UTF8Encoding($false)))
            & git -C $lw add -- docs/ledger/quota-readings.csv 2>&1 | Out-Null
            & git -C $lw commit --no-edit 2>&1 | Out-Null
            Ev '-' 'ledger-merge' 'merge conflict in quota-readings.csv resolved by keeping both sides (duplicate lines removed)'
          } else { & git -C $lw merge --abort 2>&1 | Out-Null; throw "ledger merge of origin/main failed: $mg" }
        }
        $target = "$lw\docs\ledger\quota-readings.csv"
        if (-not (Test-Path $target)) { throw 'docs/ledger/quota-readings.csv not found on origin/main' }
        $cur = [System.IO.File]::ReadAllText($target); $eol = "`n"; if ($cur.Contains("`r`n")) { $eol = "`r`n" }
        $prefix = ''; if (-not ($cur.EndsWith("`n"))) { $prefix = $eol }
        [System.IO.File]::AppendAllText($target, $prefix + (($newRows) -join $eol) + $eol, (New-Object System.Text.UTF8Encoding($false)))
        & git -C $lw add -- docs/ledger/quota-readings.csv 2>&1 | Out-Null
        $lm = "Quota readings from the Gaming-PC heartbeat ($($newRows.Count) rows)`n`n- Appended by Claude's heartbeat on Ahmed's PC: one row per tick (every 10 minutes) with the Codex 5-hour and weekly percentages read from Codex's own rate_limits events (and the Claude plan meter when its status-line file exists).`n- Rows are appended at the end of docs/ledger/quota-readings.csv, same columns; stale readings are marked in the notes column.`n- Keep both sides on a merge conflict.`n`nSide: Ahmed (AE0101) $([char]0x00B7) Machine: Gaming-PC`n`nCo-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
        $lmf = "$out\ledger.commitmsg.txt"; [System.IO.File]::WriteAllText($lmf, $lm, (New-Object System.Text.UTF8Encoding($false)))
        & git -C $lw commit -F $lmf 2>&1 | Out-Null
        $okp = $false; foreach ($i in 0..3) { & git -C $lw push -u origin $lb 2>&1 | Out-Null; if ($LASTEXITCODE -eq 0) { $okp = $true; break }; Start-Sleep -Seconds ([math]::Pow(2, $i + 1)) }
        if ($okp) { $lg.lastLine = $all.Count; $lg.lastPushUtc = (NowS); Ev '-' 'ledger-push' ("pushed {0} quota rows to {1} ({2})" -f $newRows.Count, $lb, (& git -C $lw rev-parse --short HEAD).Trim()) } else { Ev '-' 'attention' 'ledger push failed (will retry next tick)' }
      }
    }
  } catch { Add-Content $logPath ("{0} ledger push failed: {1}" -f (NowS), ($_.Exception.Message -replace '\s+', ' ')); Ev '-' 'attention' ("ledger push error: " + ($_.Exception.Message -replace '\s+', ' ').Substring(0, [Math]::Min(180, $_.Exception.Message.Length))) }
  # ---- helpers
  function Make-Prompt($l, $resume) {
    $ld = $lanes[$l]
    if (-not $ld.range) {
      $p = Get-Content "$hb\prompt-first-$l.md" -Raw
      if ($resume) { $p += "`n`nRESUME: an earlier run of this lane stopped before finishing (usage limit or crash). Inspect git status and git diff in the worktree, keep the existing work, finish what is missing, and re-run the checks. Do not redo finished work." }
      $pf = "$hb\prompt-$l.md"; Set-Content $pf $p -Encoding utf8; return $pf
    }
    $b = Get-Content "$hb\briefs.md"
    $sec0 = ($b[8..19]) -join "`n"
    $r = $ld.range -split ','; $sec = ($b[([int]$r[0] - 1)..([int]$r[1] - 1)]) -join "`n"
    $dep = ''; if ($ld.base -ne 'origin/main') { $dep = "Your branch already contains the earlier lanes' work (base $($ld.base)" + $(if ($ld.merge.Count) { ", merged with $($ld.merge -join ', ')" } else { '' }) + "). Read their code and hand-back state with git log before you start; build on it, do not redo it.`n" }
    $p = @"
You are Codex (gpt-6.1-sol, xhigh), the lead developer hand for Notaya 1.6, lane $l. Your working root is the git worktree D:\Notaya-lanes\$l (branch $($ld.br)). $dep You are working for Ahmed Emara (GitHub AE0101); Claude on his PC (machine Gaming-PC) runs you, inspects, commits and pushes.

First read, in this worktree: CLAUDE.md, AGENTS.md, docs/STATUS.md (Continuation part), docs/plan/approval-and-amendments.md (A37 to A44), docs/handoff/claims.md, docs/plan/2026-11-08-v1-6-spec.md (the sections your lane names below, in full), and docs/security-design.md if you touch crypto, storage, commands, CSP, clipboard or auto-lock.

Operating rules for this run (they override the shared rules where they differ):
- Do NOT run git commit, git push, git checkout, git merge or open PRs; Claude does the commits. You may run git status, git diff and git log. Never git add -A.
- Do NOT edit docs/handoff/claims.md or any record under docs/ (execution summaries, STATUS, sessions). Never edit docs/plan/approved-plan-2026-10-22.md. Rule 2 of the shared rules (claim row, branch, draft PR) is handled by Claude; ignore it.
- Stay strictly inside your lane's write area. If you need a file outside it, stop and say so in your final message instead of editing it.
- Node packages resolve from D:\Notaya-lanes\node_modules. Do not install or update any package, do not run npm install/ci, do not add a Rust crate. No network access is available.
- For cargo, CARGO_TARGET_DIR is already set to D:\Notaya-lanes\_target (shared); use 'cargo ... --offline'. Other lanes may hold the lock briefly; wait.
- Test safety: throwaway data only, in D:\Notaya Tests\v1-6-$l with NOTAYA_HOME pointing there. Do not launch the real Notaya.exe, open no debug port, touch no vault or notes outside that folder, never touch drive E:. UI checks use 'npm run mock' only.
- Report honestly: anything you did not run is "not run", never "passed".
- Your FINAL message must be the hand-back note: (1) files changed, grouped; (2) each check you ran with exact command and exact result; (3) what you did not run; (4) open questions; (5) anything you added that nobody asked for, marked ADDITION.

Shared rules for every lane (docs/plan/2026-11-08-v1-6-codex-briefs.md, section 0):
$sec0

YOUR LANE BRIEF:
$sec
$add13
Work until the lane's stop point is fully done, with its tests, and the checks the brief lists have been run on the final source. Then stop.
"@
    if ($resume) { $p += "`n`nRESUME: an earlier run of this lane stopped before finishing (usage limit or crash). Inspect git status and git diff in the worktree, keep the existing work, finish what is missing, and re-run the checks." }
    $pf = "$hb\prompt-$l.md"; Set-Content $pf $p -Encoding utf8; return $pf
  }
  # ---- lean prompts (coordinator decision 2026-11-08 01:00Z: Codex's WEEKLY limit is the real constraint)
  $doneNote = @{
    A  = 'Lane A was already pushed at 9543b87 (not merged): WP-08, WP-14, WP-09, WP-10, WP-11, WP-13 code and WP-12 code (last_place in model.rs, vault.rs, the store, navHistory; set_last_place registered in lib.rs), two check scripts dev-mock/lane-a-check.cjs and lane-a-contrast-check.cjs; tsc, build and cargo test passed outside your sandbox. Meanwhile C1 (action core), B and D were merged into origin/main.'
    B  = 'Already in the worktree: all six files under src-tauri/src/export edited (doc, docx, html, markdown, mod, pdf; Block::Database, row cap, missing-database text, Markdown reference) and about a dozen new tests (parsing, rows and RTL order, filtered view, 1,000-row cap, missing or binned database, ZIP with page links, escaping). NOT done: nothing has been compiled or run. Run `cargo test --lib --offline` in src-tauri, fix what fails inside your write area, and confirm P25, P28 and the existing export tests still pass.'
    C1 = 'Already in the worktree: src-tauri/src/actions/{mod.rs, registry.rs (88 action names), actor.rs} and command files automation, calculator, database, notes, search_history, stickies rewritten into thin wrappers around core functions taking the actor, plus lib.rs (mod actions; list_actions). NOT done or unverified: nothing has been compiled; all 67 #[tauri::command] must be checked as wrappers (check the other command files too), names, arguments and answers byte-identical, and all 517 existing tests must pass WITHOUT edits; registry tests (unique area.action names, level/confirm/undo present, never-list has no action, wrapper and core answer the same).'
    D  = 'Already in the worktree: bridge/{Cargo.toml, src/main.rs, mcp.rs, pipe.rs, tools.rs} (13 tests), the workspace change in src-tauri/Cargo.toml and Cargo.lock, tools/windows-build/build-windows.ps1, packaging/{bridge-configuration.md, test-bridge-packaging.ps1, test-bridge-stdio.py}. NOT done: nothing has been compiled; the dual-era change below; the hand-back citations.'
    E  = 'Lane E was pushed at c36397b (13 files; AI screens against mock fakes). Its browser check dev-mock/ai-ui-check.cjs fails as written for three script reasons only (the product behaviour is fine).'
    AF = 'A previous run of this lane was stopped by Claude about 3 minutes after it started, so that the decision for finding 2 could change; keep any edits already in the worktree and continue.'
    C4b = 'Lane C4b was pushed at 0beb356 (merge of origin/main, three bid assertion fixes, cargo test passing). The cloud thread inspected it and wants the two fixes in the additional requirements; everything else stays.'
    C3 = 'A previous run of this lane was killed about 2 minutes after it started (an infrastructure fault of the launcher, not a decision); keep every edit already in the worktree and continue.'
    C2 = ''; C5 = ''
  }
  function Make-Prompt($l, $resume) {
    $ld = $lanes[$l]; if ($ld.review) { if ($ld.promptFileReview) { return $ld.promptFileReview }; return "$hb\prompt-review-A.md" }; if ($ld.promptFile) { return $ld.promptFile }
    $b = Get-Content "$hb\briefs.md"
    $r = $ld.range -split ','; $sec = ($b[([int]$r[0] - 1)..([int]$r[1] - 1)]) -join "`n"
    $dep = ''; if ($ld.base -ne 'origin/main') { $dep = "Your branch already contains the earlier lanes' work (base $($ld.base)" + $(if ($ld.merge.Count) { ", merged with $($ld.merge -join ', ')" } else { '' }) + "). Build on it, do not redo it (git log shows it).`n" }
    $done = ''
    if ($resume -and $doneNote[$l]) { $done = "`nSTATE OF THE WORKTREE (an earlier run was cut off by the usage limit; keep this work, do not redo it): $($doneNote[$l])`n" }
    $mi = ''; if ($state.lanes[$l].mergeInfo -and $state.lanes[$l].mergeInfo -ne 'no conflicts') { $mi = "`nA MERGE OF origin/main IS IN PROGRESS in the worktree (git merge --no-commit): resolve the conflicted files listed here, leaving no conflict markers: $($state.lanes[$l].mergeInfo)`n" }
    $done = $done + $mi
    $fu = ''; $fp = "$hb\followup-$l.md"
    if (Test-Path $fp) { $fu = "`nADDITIONAL REQUIREMENTS FOR THIS RUN (decisions made after your first run):`n" + (Get-Content $fp -Raw) + "`n"; $state.lanes[$l].followupDone = $true }
    $sec1 = ''; $sp = (Select-String -InputObject $sec -Pattern 'Read:\*\*\s*(.+)' | Select-Object -First 1); if ($sp) { $sec1 = $sp.Matches[0].Groups[1].Value }
    $add13 = ""; if ($l -in "C2","C3","C4b","C5") { $b13 = ($b[112..($b.Count - 1)]) -join "`n"; $add13 = "`nADDENDUM TO THE BRIEFS (section 13, decisions after the first review; it widens your write area where it says so):`n$b13`n" }
    $p = @"
Read AGENTS.md and docs/handoff/codex-partner-briefing.md first (if the briefing file is not in your worktree yet, say so in your hand-back and go on). Then:`nYou are Codex, the lead developer hand for Notaya 1.6, lane $l, working in the git worktree D:\Notaya-lanes\$l (branch $($ld.br)). You work for Ahmed Emara (AE0101); Claude on his PC (Gaming-PC) runs you and does all commits and pushes. $dep
Be economical: your usage quota is limited. Apart from AGENTS.md and the partner briefing, do NOT re-read the repository records (CLAUDE.md, docs/STATUS.md, docs/sessions.md, the amendments, the claims board, the plan snapshot); the rules you need are below. Read only the design sections this brief names (it says "Read:") from docs/plan/2026-11-08-v1-6-spec.md, and docs/security-design.md only if you touch crypto, storage, commands, the CSP, the clipboard or auto-lock. Read source files only as needed for the change.
$done$fu
RULES:
- Stay inside the lane's write area (below), EXCEPT for purely mechanical edits outside it that the change needs (test assertion updates, struct-initializer defaults, an import line, a registration line): make them yourself and list every such file in the hand-back under the heading 'outside write area, mechanical'. A stop at a write-area boundary must never idle the lane. Stop and ask only for a behaviour change, a design choice, a security-relevant change, a new dependency, or a change that could alter how Notaya looks or feels.
- Do NOT run git commit/push/checkout/merge; the git directory is outside your sandbox. After each finished work package print one line `CHECKPOINT: <work package> done` (Claude's heartbeat makes the commit for you every 30 minutes). Never git add -A.
- No new library, no package install/update/npm ci; no network. Node packages resolve from D:\Notaya-lanes\node_modules. Stable, exact versions only.
- Cargo: CARGO_TARGET_DIR is set (shared); use `cargo ... --offline`. Other lanes may hold the lock briefly; wait.
- Security by design: keys and secrets stay in the Rust core; the AI can never change its own permissions; no network code; no note content in error text or the error log; the pipe has no way to answer a confirmation.
- Test safety: throwaway data only in D:\Notaya Tests\v1-6-$l with NOTAYA_HOME there; do not launch the real Notaya.exe, open no debug port, never touch any real vault or notes or drive E:. UI checks use `npm run mock` only.
- Do not edit docs/ records or docs/plan/approved-plan-2026-10-22.md.
- Report honestly: anything not run is "not run", never "passed".
- Checks to run on the final source and report: cargo test --lib --offline (in src-tauri, all 517 existing tests must still pass without edits unless the brief says moved code), npx tsc --noEmit -p tsconfig.json, npm run build, npm audit; npm run mock with scripted clicks for UI work (if a tool such as Playwright is missing, say so; do not install it).
- FINAL message = hand-back note: (1) files changed, grouped; (2) each check with the exact command and exact result; (3) what you did not run; (4) open questions; (5) anything you added that nobody asked for, marked ADDITION.

YOUR LANE BRIEF (design sections to read: $sec1):
$sec
$add13
Work until the lane's stop point is fully done, with its tests, and the checks have been run on the final source. Then stop.
"@
    $pf = "$hb\prompt-$l.md"; Set-Content $pf $p -Encoding utf8; return $pf
  }
  function ProbeNow($why) {
    # one tiny claude -p probe right now; writes claude-rate.json and the Claude ledger rows (notes = why). Returns the reading or $null.
    if ($DryRun) { return $null }
    try {
      $cl = $null; $gc = Get-Command claude -ErrorAction SilentlyContinue; if ($gc) { $cl = $gc.Source }; if (-not $cl) { $cl = 'C:\Users\Ahmed\.local\bin\claude.exe' }
      $pd = 'D:\Notaya Tests\claude-probe'; New-Item -ItemType Directory -Force $pd | Out-Null; $po = "$out\claude-probe-stream.jsonl"
      Push-Location $pd
      cmd /c "`"$cl`" -p ok --model haiku --tools `"`" --strict-mcp-config --disable-slash-commands --no-session-persistence --system-prompt `"Reply with ok.`" --output-format stream-json --verbose > `"$po`" 2>&1"
      Pop-Location
      $evl = Get-Content $po -ErrorAction SilentlyContinue | Where-Object { $_ -match 'unifiedWindows' } | Select-Object -Last 1
      if (-not $evl) { Add-Content $logPath ("{0} claude probe ({1}) returned no rate_limit_event" -f (NowS), $why); return $null }
      $ei = ($evl | ConvertFrom-Json).rate_limit_info.unifiedWindows; $raw = Get-Content $po -Raw
      $pn = 'one tiny claude -p probe (haiku, no tools)'
      $obj = [ordered]@{ ts = (Now).ToString('o'); source = 'claude -p probe, rate_limit_event (utilization x 100)'; five_hour = [ordered]@{ used_percentage = [math]::Round([double]$ei.five_hour.utilization * 100, 1); raw_utilization = [double]$ei.five_hour.utilization; resets_at = $ei.five_hour.resetsAt }; seven_day = [ordered]@{ used_percentage = [math]::Round([double]$ei.seven_day.utilization * 100, 1); raw_utilization = [double]$ei.seven_day.utilization; resets_at = $ei.seven_day.resetsAt }; probe_usage = $pn }
      $crf = "$hb\claude-rate.json"; $tmp = "$crf.tmp"; ($obj | ConvertTo-Json -Depth 4) | Set-Content $tmp -Encoding UTF8; Move-Item $tmp $crf -Force
      $q = { param($v) '"' + ([string]$v).Replace('"', '""') + '"' }
      $nowT = (Now).ToString('yyyy-MM-ddTHH:mm') + 'Z'
      $r5 = [DateTimeOffset]::FromUnixTimeSeconds([long]$ei.five_hour.resetsAt).UtcDateTime.ToString('yyyy-MM-ddTHH:mm:ss') + 'Z'; $rw = [DateTimeOffset]::FromUnixTimeSeconds([long]$ei.seven_day.resetsAt).UtcDateTime.ToString('yyyy-MM-ddTHH:mm:ss') + 'Z'
      $row = @($nowT, 'Claude', 'Ahmed (AE0101)', 'five-hour window and weekly limit', ('5-hour {0}% used, weekly {1}% used (raw {2} and {3})' -f $obj.five_hour.used_percentage, $obj.seven_day.used_percentage, $ei.five_hour.utilization, $ei.seven_day.utilization), ('5-hour {0}; weekly {1}' -f $r5, $rw), 'heartbeat tick (Gaming-PC)', ('heartbeat tick, Claude claude -p probe rate_limit_event at ' + $obj.ts), $why)
      Add-Content "$root\ledger\quota-readings-pc.csv" (($row | ForEach-Object { & $q $_ }) -join ',') -Encoding UTF8
      return $obj
    } catch { Pop-Location -ErrorAction SilentlyContinue; Add-Content $logPath ("{0} claude probe ({1}) failed: {2}" -f (NowS), $why, ($_.Exception.Message -replace '\s+', ' ')); return $null }
  }
  function ClaudeDeltaRow($l, $before, $after) {
    try {
      $q = { param($v) '"' + ([string]$v).Replace('"', '""') + '"' }
      $reading = ('weekly {0}% -> {1}% (+{2}); 5-hour {3}% -> {4}% (+{5}) between {6} and {7}' -f $before.pw, $after.seven_day.used_percentage, ([math]::Round($after.seven_day.used_percentage - $before.pw, 1)), $before.p5, $after.five_hour.used_percentage, ([math]::Round($after.five_hour.used_percentage - $before.p5, 1)), $before.ts, $after.ts)
      $row = @(((Now).ToString('yyyy-MM-ddTHH:mm') + 'Z'), 'Claude', 'Ahmed (AE0101)', ("meter change while lane $l ran"), $reading, 'not applicable', 'heartbeat tick (Gaming-PC)', "heartbeat before/after probes around lane $l", 'the Claude meter is account-wide: the change includes every other Claude session during the same time, not only this lane')
      Add-Content "$root\ledger\quota-readings-pc.csv" (($row | ForEach-Object { & $q $_ }) -join ',') -Encoding UTF8
      Ev $l 'claude-meter' $reading
    } catch { }
  }  function Launch($l, $promptFile, $tag) {
    if ($DryRun) { return }
    $stamp = (Get-Date).ToString('HHmmss'); foreach ($ext in 'md','log') { $o = "$out\$tag.$ext"; if (Test-Path $o) { Move-Item $o "$out\$tag.prev-$stamp.$ext" -Force } }
    # Launch through WMI so the lane's process tree is NOT a child of this tick's job object: a tick started by Task Scheduler (or a background job) kills its children when it ends.
    $mdl = [string]$lanes[$l].model; if (-not $mdl) { $mdl = 'gpt-6.1-sol' }; $eff = [string]$lanes[$l].effort; if (-not $eff) { $eff = 'xhigh' }   # per-lane model and effort (A54); default Sol xhigh
    $cmdline = "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$hb\run-lane.ps1`" $l `"$promptFile`" $tag $mdl $eff"
    try {
      $si = New-CimInstance -ClassName Win32_ProcessStartup -ClientOnly -Property @{ ShowWindow = [uint16]0 }
      $cr = Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{ CommandLine = $cmdline; CurrentDirectory = $hb; ProcessStartupInformation = $si }
      if ($cr.ReturnValue -ne 0) { throw "Win32_Process.Create returned $($cr.ReturnValue)" }
    } catch { Add-Content $logPath ("{0} WMI launch failed for {1} ({2}); falling back to Start-Process" -f (NowS), $l, $_.Exception.Message); Start-Process powershell -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File',"$hb\run-lane.ps1",$l,$promptFile,$tag,$mdl,$eff -WindowStyle Hidden }
  }
  function GitC($wt) { return (& git.exe -C $wt @args 2>&1 | Out-String).Trim() }
  function Ensure-Worktree($l) {
    $ld = $lanes[$l]; $wt = "$root\$l"
    if (Test-Path $wt) { return $true }
    & git -C D:\Notaya fetch --all --prune 2>&1 | Out-Null
    $o = & git -C D:\Notaya worktree add -b $ld.br $wt $ld.base 2>&1 | Out-String
    if (-not (Test-Path $wt)) { Ev $l 'attention' "worktree add failed: $o"; return $false }
    foreach ($m in $ld.merge) {
      $mo = & git -C $wt merge --no-edit $m 2>&1 | Out-String
      if ($LASTEXITCODE -ne 0) { & git -C $wt merge --abort 2>&1 | Out-Null; Ev $l 'attention' "merge of $m conflicted; merge aborted, lane NOT started (needs a decision)"; return $false }
    }
    $state.lanes[$l].workBase = (& git -C $wt rev-parse HEAD).Trim()
    return $true
  }
  function Run-Check($name, $cmd, $dir, $lane) {
    $lf = "$out\$lane.check-$name.log"
    if ($DryRun) { return @{ name=$name; exit=-1; log=$lf; skipped=$true } }
    $env:NOTAYA_HOME = "D:\Notaya Tests\v1-6-$lane"; $env:CARGO_TARGET_DIR = "$root\_target_checks"; $env:CARGO_NET_OFFLINE = 'true'
    Push-Location $dir; cmd /c "$cmd > `"$lf`" 2>&1"; $ec = $LASTEXITCODE; Pop-Location
    return @{ name=$name; exit=$ec; log=$lf; tail=((Get-Content $lf -Tail 6 -ErrorAction SilentlyContinue) -join ' | ') }
  }
  function Checkpoint($l) {
    # local WIP commit of a lane's work so a cut-off loses little; Codex cannot write the git dir itself. Never pushes.
    if ($DryRun) { return }
    $wt = "$root\$l"; if (-not (Test-Path $wt)) { return }
    & git -C $wt rev-parse -q --verify MERGE_HEAD 2>&1 | Out-Null; if ($LASTEXITCODE -eq 0) { return }   # merge in progress: never commit conflict markers
    $porc = (& git -C $wt status --porcelain 2>&1) -split "`n" | Where-Object { $_.Trim() -and $_ -notmatch '^warning' }
    $paths = @(); foreach ($line in $porc) { $p = $line.Substring(3).Trim('"'); if ($p -match '(?i)\.(nota|clf|key|csv|env)$|(^|/)\.devdata/|(^|/)\.env' ) { continue }; $paths += $p }
    if (-not $paths.Count) { return }
    & git -C $wt add -- @paths 2>&1 | Out-Null
    $msg = "WIP checkpoint, lane ${l} (automatic, $(NowS))`n`n- Work in progress by Codex (gpt-6.1-sol, reasoning xhigh); not finished, not checked. Made by Claude's heartbeat so a usage-limit cut-off loses little.`n- $($paths.Count) file(s) added or changed since the previous checkpoint.`n`nSide: Ahmed (AE0101) $([char]0x00B7) Machine: Gaming-PC`n`nCo-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
    $mf = "$out\$l.checkpoint-msg.txt"; [System.IO.File]::WriteAllText($mf, $msg, (New-Object System.Text.UTF8Encoding($false)))
    & git -C $wt commit -F $mf 2>&1 | Out-Null
  }
  function MergeLatestMain($l) {
    # Before a lane is finished: make sure it builds on the newest origin/main. Returns $true when it is merged (or already up to date).
    $ld = $lanes[$l]; $wt = "$root\$l"; $ls = $state.lanes[$l]
    & git -C D:\Notaya fetch origin main 2>&1 | Out-Null
    & git -C $wt rev-parse -q --verify MERGE_HEAD 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
      # an earlier merge is still open: refuse to commit conflict markers, then complete it
      foreach ($cf in @(([string]$ls.mergeInfo) -split ',\s*' | Where-Object { $_ -and $_ -ne 'no conflicts' })) { $cp = Join-Path $wt $cf; if ((Test-Path $cp) -and (Select-String -Path $cp -Pattern '^(<<<<<<<|>>>>>>>) ' -Quiet)) { $ls.status = 'attention'; Ev $l 'attention' "conflict markers left in $cf; needs a fix"; return $false } }
      $porc = (& git -C $wt status --porcelain 2>&1) -split "`n" | Where-Object { $_.Trim() -and $_ -notmatch '^warning' }
      $ps = @(); foreach ($line in $porc) { $p = $line.Substring(3).Trim('"'); if ($p -match '(?i)\.(nota|clf|key|csv|env)$|(^|/)\.devdata/|(^|/)\.env') { continue }; $ps += $p }
      if ($ps.Count) { & git -C $wt add -- @ps 2>&1 | Out-Null }
      & git -C $wt commit -m "WIP: complete the open merge of origin/main (heartbeat, $(NowS))" 2>&1 | Out-Null
    } else { Checkpoint $l }
    $mo = & git -C $wt merge --no-commit --no-ff origin/main 2>&1 | Out-String
    $conf = @(& git -C $wt diff --name-only --diff-filter=U 2>$null | Where-Object { $_ })
    if ($conf.Count) {
      $ls.mergeInfo = $conf -join ', '; $ls.status = 'pending'; $ls.resume = $true
      Ev $l 'merge-conflict' ("merging the newest origin/main into $($ld.br) conflicts in: $($ls.mergeInfo); lane relaunched to resolve them (mechanical only)"); return $false
    }
    $ls.mergeInfo = 'no conflicts'; Ev $l 'merged-main' "merged the newest origin/main into $($ld.br) before finishing"; return $true
  }  function Finish-Lane($l) {
    $ld = $lanes[$l]; $wt = "$root\$l"; $ls = $state.lanes[$l]
    $pa = ProbeNow "after lane $l finished"; if ($pa -and $ls.claudeBefore) { ClaudeDeltaRow $l $ls.claudeBefore $pa; $ls.claudeBefore = $null }
    if ($ld.review) { $ls.status = 'done'; Ev $l 'finished' "review run ended; findings file: D:\Notaya-lanes\review-A\findings.md (exists: $(Test-Path 'D:\Notaya-lanes\review-A\findings.md'))"; return }
    $porc = (& git -C $wt status --porcelain 2>&1) -split "`n" | Where-Object { $_.Trim() -and $_ -notmatch '^warning' }
    $paths = @(); foreach ($line in $porc) { $p = $line.Substring(3).Trim('"'); if ($p -match '(?i)\.(nota|clf|key|csv|env)$|(^|/)\.devdata/|(^|/)\.env' ) { continue }; $paths += $p }
    # own work = commits after the worktree was prepared (merges of earlier lanes do not count as this lane's work)
    $wb = $ld.base; if ($ls.workBase) { $wb = $ls.workBase }
    $ahead = [int](& git -C $wt rev-list --first-parent --no-merges --count "$wb..HEAD" 2>$null); $cnames = @(& git -C $wt diff --name-only "$wb..HEAD" 2>$null); $allnames = @($paths) + $cnames
    $changedRust = [bool]($allnames | Where-Object { $_ -like 'src-tauri/*' }); $changedBridge = [bool]($allnames | Where-Object { $_ -like 'bridge/*' }); $changedTs = [bool]($allnames | Where-Object { $_ -like 'src/*' -or $_ -like 'dev-mock/*' })
    $res = @()
    if ($changedTs -or $l -in 'A','E') { $res += Run-Check 'tsc' 'npx tsc --noEmit -p tsconfig.json' $wt $l; $res += Run-Check 'build' 'npm run build' $wt $l }
    if ($changedRust) { $res += Run-Check 'cargo-test' 'cargo test --lib --offline' "$wt\src-tauri" $l }
    if ($changedBridge -and (Test-Path "$wt\bridge")) { $res += Run-Check 'bridge-test' 'cargo test --offline' "$wt\bridge" $l }
    $res += Run-Check 'audit' 'npm audit' $wt $l
    $failed = @($res | Where-Object { $_.exit -ne 0 -and $_.name -ne 'audit' })
    $status = 'pushed'
    if ($paths.Count -eq 0 -and $ahead -eq 0) { $ls.stalls = [int]$ls.stalls + 1; if ($ls.stalls -le 2) { $ls.status = 'pending'; $ls.resume = $true; Ev $l 'idle-stop' ("run ended with exit 0 and NO file changes (boundary stop, not finished); relaunch #{0} queued; see hand-back {1}" -f $ls.stalls, $out) } else { $ls.status = 'attention'; Ev $l 'attention' 'three runs in a row ended with no file changes; needs a decision' }; return }
    if (-not $DryRun) {
      if ($paths.Count) { & git -C $wt add -- @paths 2>&1 | Out-Null }
      $stat = GitC $wt diff --cached --stat $wb
      $hand = ''; $mdp = (PathsFor $l ($(if ($ls.followupDone -and (Test-Path "$out\$l.fu.md")) { "$l.fu" } else { $l }))).md; if (Test-Path $mdp) { $hand = (Get-Content $mdp -TotalCount 25) -join "`n" }
      $msg = "Notaya 1.6 lane ${l}: $($ld.title)`n`n- Written by Codex (gpt-6.1-sol, reasoning xhigh) from the lane brief; run and committed by Claude on Ahmed's PC.`n- Files in this commit:`n$(($stat -split "`n" | ForEach-Object { '  ' + $_ }) -join "`n")`n- Checks run on this exact source: " + (($res | ForEach-Object { "$($_.name)=" + $(if ($_.exit -eq 0) {'pass'} else {'FAIL(' + $_.exit + ')'}) }) -join ', ') + "`n$(if ($ld.credit) { "- Credit: $($ld.credit).`n" })- Not merged, not released; the cloud thread inspects and opens the pull request.`n`nSide: Ahmed (AE0101) $([char]0x00B7) Machine: Gaming-PC`n`nCo-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
      $mf = "$out\$l.commitmsg.txt"; [System.IO.File]::WriteAllText($mf, $msg, (New-Object System.Text.UTF8Encoding($false)))
      if ($paths.Count) { & git -C $wt commit -F $mf 2>&1 | Out-Null }
      $hash =(& git -C $wt rev-parse --short HEAD).Trim()
      $pushed = $false; foreach ($i in 0..3) { & git -C $wt push -u origin $ld.br 2>&1 | Out-Null; if ($LASTEXITCODE -eq 0) { $pushed = $true; break }; Start-Sleep -Seconds ([math]::Pow(2, $i + 1)) }
      $rep = "# Lane $l report ($(NowS))`n`nBranch: $($ld.br)  Commit: $hash  Pushed: $pushed`n`n## Checks`n" + (($res | ForEach-Object { "- $($_.name): exit $($_.exit) ($($_.log)) :: $($_.tail)" }) -join "`n") + "`n`n## Files`n$stat`n`n## Codex hand-back (first lines)`n$hand`n"
      Set-Content "$out\$l-report.md" $rep -Encoding utf8
      $ls.commit = $hash; $ls.pushed = $pushed; $ls.mergeInfo = $null; if ($pushed) { $ls.reviewReady = $true }
      if ($pushed -and $failed.Count -eq 0) { $ls.status = 'pushed'; $ls.checksFailed = $false; Ev $l 'finished' ("branch $($ld.br) commit $hash pushed; checks: " + (($res | ForEach-Object { "$($_.name)=$($_.exit)" }) -join ' ') + (ModelNote $l)) }
      elseif ($pushed) { $ls.status = 'pushed'; $ls.checksFailed = $true; Ev $l 'finished-checks-failed' ("branch $($ld.br) commit $hash pushed; FAILED: " + (($failed | ForEach-Object { $_.name }) -join ',') + " ; see $out\$l-report.md" + (ModelNote $l)) }
      else { $ls.status = 'attention'; Ev $l 'attention' "commit $hash made but push failed" }
    }
  }

  if ($env:HB_PROMPT_TEST) { foreach ($tl in $env:HB_PROMPT_TEST -split ',') { $null = Make-Prompt $tl $true }; Add-Content $logPath ("{0} prompt test written for {1}" -f (NowS), $env:HB_PROMPT_TEST); exit 0 }
  # ---- (a)(d)(e)(f) per-lane pass
  EnforceStopLine $rate
  # at most 2 lanes start/run in parallel while the 5-hour meter is at 60% or more (Ahmed, via the coordinator, 2026-11-09)
  $effMax = $maxAlive; if ($rate -and [double]$rate.p -ge 60) { $effMax = [Math]::Min($maxAlive, 2) }
  $planBurn = if ($rate) { BurnPerMin $rate } else { 0.0 }
  if ($DryRun) { Write-Host ("DRYRUN lane models: " + (($lanes.Keys | Where-Object { $state.lanes[$_].status -in 'pending','paused-stopline' } | ForEach-Object { $mm = [string]$lanes[$_].model; if (-not $mm) { $mm = 'gpt-6.1-sol' }; $ee = [string]$lanes[$_].effort; if (-not $ee) { $ee = 'xhigh' }; "$_=$mm/$ee" }) -join ', ')) }
  if ($DryRun -and $rate -and $state.stopLine -and $state.stopLine.active) { $anyLane = @(Get-CimInstance Win32_Process -Filter "Name='codex.exe'" | Where-Object { $_.CommandLine -match ' exec ' -and $_.CommandLine -match 'Notaya-lanes' }).Count; $liftOk = ([string]$state.stopLine.reason -match 'predicted' -and [long]$rate.pr -eq [long]$state.stopLine.resetEpoch -and [double]$rate.p -lt 90 -and ([double]$rate.p + $planBurn * $LookAheadMin) -lt 95 -and -not $anyLane); Write-Host ("DRYRUN stop-line lift check: held because '{0}' since {1}, reset {2}; used {3}% burn {4}/min lanes running {5} -> {6}" -f $state.stopLine.reason, $state.stopLine.since, $state.stopLine.resetUtc, $rate.p, $planBurn, $anyLane, $(if ($liftOk) { 'WOULD LIFT (predicted line, nothing runs, used < 90, used + look-ahead x burn < 95)' } else { 'stays held' })) }
  if ($DryRun -and $rate) { Write-Host ("DRYRUN stop-line plan: used {0}% burn {1} points/min predicted(5 min) {2}% -> line {3}; effective max lanes {4}; watch loop {5}" -f $rate.p, $planBurn, [math]::Round([double]$rate.p + $planBurn * $LookAheadMin, 1), $(if ([double]$rate.p -ge 95 -or ($planBurn -gt 0 -and [double]$rate.p + $planBurn * $LookAheadMin -ge 95)) { 'REACHED' } else { 'not reached' }), $effMax, $(if ([double]$rate.p -ge 70 -or ($planBurn -gt 0 -and [double]$rate.p + $planBurn * 10 -ge 85)) { 'on (60 s samples)' } else { 'off' })) }
  $script:launched = 0; $script:launchedFix = 0; $lineParts = @()
  foreach ($l in @('C5','P1','N2','P6b','S','X1','SF2','SF3','N1','SF1','R1','P6','W','RC5','C3u','C2d','C4d','RC2','C2','AG','C4b','AF','RC4','E','RA','C3','C5','C1','A','B','D')) {
    $ls = $state.lanes[$l]; $wt = "$root\$l"
    $alive = Alive $l
    $fuTag = "$l.fu"
    if ($ls.status -eq 'running' -and -not $alive -and -not (AliveNow $l)) {
      $mdp = (PathsFor $l $l).md; $lgp = (PathsFor $l $l).log
      $logT = Tail-Text $lgp 30
      if ((Test-Path $mdp) -and ($logT -match 'EXIT 0|EXIT')) {
        $fu = "$hb\followup-$l.md"
        if ((Test-Path $fu) -and -not $ls.followupDone) {
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
      $quotaOk = $true; if ($rate -and $rate.p -ge 95 -and $rate.pr -gt [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()) { $quotaOk = $false }; if ($state.stopLine -and $state.stopLine.active) { $quotaOk = $false }
      if ($rate -and $planBurn -gt 0 -and ([double]$rate.p + $planBurn * $LookAheadMin -ge 95)) { $quotaOk = $false }


      if ($depsOk -and $quotaOk -and -not $limitActive -and $aliveCount -lt $effMax) {
        if ($lanes[$l].review -and -not (Test-Path "$root\$l")) { & git -C D:\Notaya fetch --all --prune 2>&1 | Out-Null; & git -C D:\Notaya worktree add --detach "$root\$l" $lanes[$l].reviewRef 2>&1 | Out-Null }
        if ($lanes[$l].review -or (Ensure-Worktree $l)) {
          if ($ls.premerge -and -not $DryRun) {
            & git -C "$root\$l" fetch origin main 2>&1 | Out-Null
            if ($ls.premergeExtra) { $xo = & git -C "$root\$l" merge --no-edit $ls.premergeExtra 2>&1 | Out-String; if ($LASTEXITCODE -ne 0) { & git -C "$root\$l" merge --abort 2>&1 | Out-Null; $ls.status = 'attention'; Ev $l 'attention' ("merge of " + $ls.premergeExtra + " failed: " + (($xo -replace '\s+', ' ').Substring(0, [Math]::Min(160, $xo.Length)))); continue }; $ls.premergeExtra = $null }
            $ls.workBase = (& git -C "$root\$l" rev-parse HEAD).Trim()
            $mo = & git -C "$root\$l" merge --no-commit --no-ff origin/main 2>&1 | Out-String
            $conf = @(& git -C "$root\$l" diff --name-only --diff-filter=U 2>$null | Where-Object { $_ })
            $ls.mergeInfo = if ($conf.Count) { $conf -join ', ' } else { 'no conflicts' }
            $ls.premerge = $false; Ev $l 'premerge' "merged origin/main into $($lanes[$l].br) without committing; conflicts: $($ls.mergeInfo)"
          }
          $pb = ProbeNow "before lane $l starts"; if ($pb) { $ls.claudeBefore = @{ ts = $pb.ts; p5 = $pb.five_hour.used_percentage; pw = $pb.seven_day.used_percentage } }
          $pf = @(Make-Prompt $l ([bool]$ls.resume))[-1]; $ls.legacy = $false; $ls.status = 'running'; $ls.startedUtc = NowS; $ls.resume = $false
          Launch $l $pf $l; $script:launched++; if ($lanes[$l].fix) { $script:launchedFix++ }; if ($rate) { $ls.meterBefore5h = $rate.p; $ls.meterBeforeW = $rate.w }; Ev $l 'started' ("launched on $($lanes[$l].br)" + (ModelNote $l))
        } else { $ls.status = 'attention' }
      }
    }
    if ($ls.status -in 'running','pending','followup-running') { Checkpoint $l }
    $changed = 0; if (Test-Path $wt) { $changed = @((& git -C $wt status --porcelain 2>&1) | Where-Object { $_ }).Count; $last = (& git -C $wt log -1 --format='%h' 2>&1) } else { $last = '-' }
    $lineParts += ("{0}:{1}{2} chg={3} {4}" -f $l, $ls.status, $(if ($alive) { '(alive)' } else { '' }), $changed, $last)
  }
  $aliveNow = $script:launched + @(@("C1","A","B","D","C4b","C2","C3","C5","E","RA","AF","RC4","AG","C4d","RC2","C2d","RC5","C3u","W","P1","S","X1") | Where-Object { Alive $_ }).Count
  if ($aliveNow -lt 2 -and -not $DryRun) { $blocked = @($lanes.Keys | Where-Object { $state.lanes[$_].status -in "pending","hold","attention" }) -join ","; Ev "-" "idle-check" ("fewer than 2 Codex lanes running at tick end (alive={0}); limit={1}; lanes not running: {2}" -f $aliveNow, $state.limit.active, $blocked) }
  $lim = if ($state.limit.active) { "LIMIT until $($state.limit.resetUtc)" } else { 'no-limit' }; if ($state.weeklyLimit) { $lim += " WEEKLY-LIMIT" }; if ($state.claude_quota_state) { $cqs = $state.claude_quota_state; $lim += (" [claude 5h={0}% wk={1}%{2}{3}]" -f $cqs.five_hour_used_pct, $cqs.weekly_used_pct, $(if ($cqs.claude_pause) { " PAUSE" } elseif ($cqs.claude_warn) { " WARN" } else { "" }), $(if (-not $cqs.fresh) { " stale" } else { "" })) }; if ($state.rate) { $lim += (" [codex 5h={0}% wk={1}% @{2}]" -f $state.rate.p, $state.rate.w, ([string]$state.rate.ts).Substring(11,8)) }
  Add-Content $logPath ("{0} {1} | {2}" -f (NowS), $lim, ($lineParts -join ' | '))
  if (-not $DryRun -and $rate -and ([double]$rate.p -ge 70 -or ($planBurn -gt 0 -and [double]$rate.p + $planBurn * 10 -ge 85))) {
    $wEnd = (Get-Date).AddMinutes(8)
    while ((Get-Date) -lt $wEnd) {
      $anyAlive = @(Get-CimInstance Win32_Process -Filter "Name='codex.exe'" | Where-Object { $_.CommandLine -match ' exec ' -and $_.CommandLine -match 'Notaya-lanes' }).Count
      if ($anyAlive -eq 0) { break }
      Start-Sleep -Seconds 60
      $rr = Read-CodexRate
      if ($rr) { $state.rate = $rr; EnforceStopLine $rr; if ($state.stopLine -and $state.stopLine.active) { break } }
    }
  }
  Save-State
} finally {
  if ($lockFs) { $lockFs.Close() }
}
















































