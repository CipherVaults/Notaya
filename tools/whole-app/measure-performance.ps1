<# PC measurement baseline, no pass thresholds or performance claims from mock timing.
   Defaults to 1k workloads and five warm samples. Increase sizes explicitly after inspecting
   the first run, e.g. -NoteCounts '1000,10000' -BlockCounts '1000,10000' -RowCounts '1000,10000,50000'.
#>
param([Parameter(Mandatory=$true)][string]$Exe,
      [Parameter(Mandatory=$true)][string]$BuildCommit,
      [string]$NoteCounts = '1000', [string]$BlockCounts = '1000', [string]$RowCounts = '1000',
      [ValidateRange(5,30)][int]$Samples = 5,
      [ValidateRange(30,7200)][int]$WorkloadTimeoutSeconds = 3600)
. (Join-Path $PSScriptRoot 'common.ps1')
[Environment]::SetEnvironmentVariable('WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS', $null, 'Process')
$run = New-WARun 'performance'; $report = New-WAReport 'performance' $run
$report.workloads = @(); $report.memorySamples = @()
$report.limitations = @('Security-off fixtures only; no cold OS cache claim, protected unlock, backups/restore, input-to-paint/save, attachments, or 10-cycle multiwindow leak assessment.', 'Memory: whole disposable process tree sampled during Node driver work and at idle; sampled peak is a lower bound, not an allocation peak.', 'Startup: process start to first nonzero MainWindowHandle, polled at 25 ms; not first paint or last-place interactive latency.')
$node = $null
$envs = Get-WAEnvironment $run
$app = $null; $port = $null

function Get-WAMemory([int]$RootPid) {
    $processes = @(Get-CimInstance Win32_Process)
    $ids = New-Object 'System.Collections.Generic.HashSet[int]'
    $null = $ids.Add($RootPid)
    do {
        $added = $false
        foreach ($p in $processes) { if ($ids.Contains([int]$p.ParentProcessId) -and $ids.Add([int]$p.ProcessId)) { $added = $true } }
    } while ($added)
    $values = @()
    foreach ($id in $ids) {
        $p = Get-Process -Id $id -ErrorAction SilentlyContinue
        if ($p) { $values += @{ pid = $id; name = $p.ProcessName; workingSet = $p.WorkingSet64; privateBytes = $p.PrivateMemorySize64 } }
    }
    @{ utc = [DateTime]::UtcNow.ToString('o'); rootPid = $RootPid; processes = $values
       workingSet = [long](($values | ForEach-Object { $_.workingSet } | Measure-Object -Sum).Sum)
       privateBytes = [long](($values | ForEach-Object { $_.privateBytes } | Measure-Object -Sum).Sum) }
}

function Invoke-WAPerfDriver([string[]]$Arguments, [string]$Log) {
    $driver = $null
    try {
        $driver = Start-WAProcess $node $Arguments $envs
        while (-not $driver.process.WaitForExit(1000)) {
            $report.memorySamples += Get-WAMemory $app.process.Id
            if ($driver.watch.Elapsed.TotalSeconds -ge $WorkloadTimeoutSeconds) { break }
            # Stop growth on an already pressured test process tree, preserving the fixture.
            if ($report.memorySamples[-1].workingSet -gt 6GB) { throw 'Test tree exceeds 6 GiB working set; stopping larger workloads' }
        }
        Complete-WAProcess $driver $Log 1
    } finally { Stop-WAProcess $driver }
}

function Start-WAPerfApp([string]$Copy, [string]$fixtureHome) {
    $handle = Start-WATestApp $Copy $fixtureHome $port $run
    try {
        while ($handle.watch.Elapsed.TotalSeconds -lt 60) {
            $handle.process.Refresh()
            if ($handle.process.HasExited) { throw 'Test exe exited before a first window' }
            if ($handle.process.MainWindowHandle -ne [IntPtr]::Zero) {
                return @{ handle = $handle; firstWindowMs = $handle.watch.Elapsed.TotalMilliseconds }
            }
            Start-Sleep -Milliseconds 25
        }
        throw 'No first window within 60 seconds'
    } catch { Stop-WAProcess $handle; throw }
}

function Get-WASummary([double[]]$Values) {
    $sorted = @($Values | Sort-Object); $n = $sorted.Count
    if (-not $n) { return $null }
    $mid = [int][math]::Floor($n / 2)
    $median = if ($n % 2) { $sorted[$mid] } else { ($sorted[$mid-1] + $sorted[$mid]) / 2 }
    @{ samples = $n; median = $median; min = $sorted[0]; max = $sorted[-1] }
}

try {
    $node = (Get-Command node -ErrorAction Stop).Source
    Assert-WANoApp
    $Exe = (Resolve-Path -LiteralPath $Exe).Path
    $report.binary = @{ path = $Exe; sha256 = (Get-FileHash -LiteralPath $Exe -Algorithm SHA256).Hash; declaredBuildCommit = $BuildCommit }
    $workloads = @(@{ kind = 'blank'; count = 1 })
    foreach ($setting in @(@{ kind = 'notes'; values = $NoteCounts }, @{ kind = 'long-page'; values = $BlockCounts }, @{ kind = 'database'; values = $RowCounts })) {
        foreach ($value in ($setting.values -split ',')) {
            $size = 0
            if (-not [int]::TryParse($value, [ref]$size) -or $size -lt 1 -or $size -gt 50000) { throw 'Fixture sizes must be integers 1..50000' }
            $workloads += @{ kind = $setting.kind; count = $size }
        }
    }
    foreach ($workload in $workloads) {
        $name = "$($workload.kind)-$($workload.count)"
        $fixtureHome = Join-Path $run.home $name; $artifacts = Join-Path $run.output $name
        New-Item -ItemType Directory -Path $artifacts -Force | Out-Null
        $copy = New-WATestApp $Exe $fixtureHome
        $port = Get-WAFreePort
        $fixture = Join-Path $artifacts 'fixture.json'
        $record = @{ name = $name; home = $fixtureHome; samples = @(); status = 'not run'; evidence = $artifacts }
        $report.workloads += $record
        try {
            $start = Start-WAPerfApp $copy $fixtureHome; $app = $start.handle
            $record.freshFirstWindowMs = $start.firstWindowMs
            Wait-WAHttp $port $app '/json'
            $commandArguments = @("$PSScriptRoot\perf-driver.mjs", "$port", 'seed', $workload.kind, "$($workload.count)", $fixture, "$artifacts\seed.json")
            $r = Invoke-WAPerfDriver $commandArguments "$artifacts\seed.log"; $record.seed = $r
            if ($r.status -ne 'passed') { throw 'Fixture seed failed or timed out; inspect seed.log/seed.json' }
            $record.seedMemory = Get-WAMemory $app.process.Id
            $report.memorySamples += $record.seedMemory
            $graceful = Close-WATestApp $app "$artifacts\seed-app.log"; $app = $null
            if (-not $graceful -or -not (Wait-WAPortClosed $port)) { throw 'Seed app did not quit cleanly' }
            for ($sample = 1; $sample -le $Samples; $sample++) {
                $start = Start-WAPerfApp $copy $fixtureHome; $app = $start.handle
                Wait-WAHttp $port $app '/json'
                $commandArguments = @("$PSScriptRoot\perf-driver.mjs", "$port", 'probe', $workload.kind, "$($workload.count)", $fixture, "$artifacts\sample-$sample.json")
                $r = Invoke-WAPerfDriver $commandArguments "$artifacts\sample-$sample.log"
                $entry = @{ number = $sample; firstWindowMs = $start.firstWindowMs; driver = $r }
                $record.samples += $entry
                if ($r.status -ne 'passed') { throw "Sample $sample failed or timed out" }
                Start-Sleep -Seconds 2
                $entry.idleMemory = Get-WAMemory $app.process.Id
                $report.memorySamples += $entry.idleMemory
                $graceful = Close-WATestApp $app "$artifacts\sample-$sample-app.log"; $app = $null
                if (-not $graceful -or -not (Wait-WAPortClosed $port)) { throw 'Sample app did not quit cleanly' }
                Write-WAJson $report "$($run.output)\in-progress.json"
                Write-Output "CHECKPOINT: performance $name sample $sample done"
            }
            $record.firstWindowSummaryMs = Get-WASummary @($record.samples | ForEach-Object { $_.firstWindowMs })
            $record.driverTimingSummaryMs = @{}
            $observations = @($record.samples | ForEach-Object { Get-Content -Raw -LiteralPath "$artifacts\sample-$($_.number).json" | ConvertFrom-Json })
            foreach ($metric in $observations[0].timingsMs.PSObject.Properties.Name) {
                $record.driverTimingSummaryMs[$metric] = Get-WASummary @($observations | ForEach-Object { $_.timingsMs.$metric })
            }
            $record.dataBytes = (Get-ChildItem -LiteralPath "$fixtureHome\data" -Recurse -File | Measure-Object Length -Sum).Sum
            $record.status = 'measured'
        } catch { $record.status = 'failed'; $record.reason = $_.Exception.Message; throw }
        finally {
            Stop-WAProcess $app; $app = $null
            $closed = Wait-WAPortClosed $port; $report.cleanup["$name-port-closed"] = $closed
        }
    }
    if ($report.memorySamples.Count) {
        $report.sampledPeakWorkingSetBytes = ($report.memorySamples | ForEach-Object { $_.workingSet } | Measure-Object -Maximum).Maximum
        $report.sampledPeakPrivateBytes = ($report.memorySamples | ForEach-Object { $_.privateBytes } | Measure-Object -Maximum).Maximum
    }
} catch { $report.fatalError = $_.Exception.Message }
finally {
    Stop-WAProcess $app
    [Environment]::SetEnvironmentVariable('WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS', $null, 'Process')
    $report.cleanup.webviewEnvironmentCleared = -not [bool]$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
    if ($null -ne $port) { $report.cleanup.finalPortClosed = Wait-WAPortClosed $port }
    Save-WAReport $report $run 'perf-results.json'
}
Write-Output "CHECKPOINT: performance JSON and cleanup report done: $script:WAOut\perf-results.json"
if ($report.fatalError -or @($report.cleanup.Values | Where-Object { $_ -eq $false }).Count) { exit 1 }
