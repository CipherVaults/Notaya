<# PC only; refuses an existing Notaya/Cipherleaf. Never drives a real window or registry.
   -BuildCommit is Claude's provenance declaration for the supplied binary, not hash-to-source proof.
   powershell -NoProfile -File .\tools\whole-app\run-group-b.ps1 -Exe <reviewed exe> -BuildCommit <commit>
#>
param([Parameter(Mandatory=$true)][string]$Exe,
      [Parameter(Mandatory=$true)][string]$BuildCommit,
      [ValidateRange(20,3600)][int]$ScriptTimeoutSeconds = 180)
. (Join-Path $PSScriptRoot 'common.ps1')
. (Join-Path $script:WARepo 'tools\real-exe-tests\common.ps1')
[Environment]::SetEnvironmentVariable('WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS', $null, 'Process')
$run = New-WARun 'group-b'; $report = New-WAReport 'B' $run
$node = $null
$envs = Get-WAEnvironment $run
$app = $null; $port = $null; $manifest = $null
try {
    $node = (Get-Command node -ErrorAction Stop).Source
    $manifestPath = Join-Path $run.output 'manifest.json'
    $inventory = Invoke-WACommand $node @("$PSScriptRoot\catalog.cjs", 'write-manifest', $manifestPath) $envs "$($run.output)\inventory.log" 30
    if ($inventory.status -ne 'passed') { throw 'Could not generate manifest' }
    $manifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
    Assert-WANoApp
    $Exe = (Resolve-Path -LiteralPath $Exe).Path
    $report.binary = @{ path = $Exe; sha256 = (Get-FileHash -LiteralPath $Exe -Algorithm SHA256).Hash; declaredBuildCommit = $BuildCommit }
    foreach ($entry in $manifest.legacyB) {
        Assert-WANoApp
        $fixtureHome = Join-Path $run.home $entry.name
        $artifacts = Join-Path $run.output $entry.name
        New-Item -ItemType Directory -Path $artifacts -Force | Out-Null
        $copy = New-WATestApp $Exe $fixtureHome
        $password = New-ThrowawayPassphrase
        $childEnv = Get-WAEnvironment $run @{ NOTAYA_HOME = $fixtureHome; CIPHERLEAF_HOME = $fixtureHome; NOTAYA_TEST_PASSWORD = $password }
        $port = Get-WAFreePort
        $suite = @{ script = "tools/real-exe-tests/$($entry.file)"; name = $entry.name; status = 'passed'; checkIds = @($entry.checkIds)
                    home = $fixtureHome; evidence = $artifacts; coverage = 'partial'; steps = @(); port = $port; durationSeconds = 0 }
        $watch = [Diagnostics.Stopwatch]::StartNew()
        try {
            foreach ($phase in $entry.phases) {
                if ($phase -eq 'reopen') { break } # Explicit readback below for every suite.
                if ($null -eq $app) {
                    $app = Start-WATestApp $copy $fixtureHome $port $run
                    Wait-WAHttp $port $app '/json'
                }
                $phaseDir = Join-Path $artifacts $phase
                $legacyArgs = if ($entry.file -eq 'optionalsecurity.mjs') { @("$port", 'environment-credential', "$fixtureHome\data", $phase) }
                    elseif ($entry.file -eq 'restoreflow.mjs') { @("$port", 'environment-credential') }
                    else { @("$port", $phase, $entry.scenario, 'environment-credential') }
                $r = Invoke-WACommand $node (@("$PSScriptRoot\legacy-adapter.mjs", $entry.file, $phaseDir) + $legacyArgs) $childEnv "$artifacts\$phase.log" $ScriptTimeoutSeconds
                $r.phase = $phase; $suite.steps += $r
                if ($r.status -ne 'passed') { $suite.status = $r.status; break }
                $needsClose = $entry.file -ne 'lockflow.mjs' -or ($entry.scenario -eq 'close' -and $phase -eq 'type')
                if ($needsClose) {
                    $graceful = Close-WATestApp $app "$artifacts\$phase-app.log"; $app = $null
                    $closed = Wait-WAPortClosed $port
                    $suite.steps += @{ phase = "$phase-quit"; status = $(if ($graceful -and $closed) { 'passed' } else { 'failed' }); gracefulQuit = $graceful; portClosed = $closed }
                    if (-not ($graceful -and $closed)) { $suite.status = 'failed'; break }
                }
            }
            if ($suite.status -eq 'passed') {
                if ($null -ne $app) {
                    $graceful = Close-WATestApp $app "$artifacts\before-readback-app.log"; $app = $null
                    if (-not $graceful) { throw 'App needed forced cleanup before persistence readback' }
                }
                if (-not (Wait-WAPortClosed $port)) { throw 'Debug port still listening before readback' }
                $app = Start-WATestApp $copy $fixtureHome $port $run
                Wait-WAHttp $port $app '/json'
                $mode = if ($entry.file -eq 'lockflow.mjs') { 'lockflow' } else { $entry.name }
                $commandArguments = @("$PSScriptRoot\live-readback.mjs", "$port", $mode)
                if ($mode -eq 'lockflow') { $commandArguments += $entry.scenario }
                $r = Invoke-WACommand $node $commandArguments $childEnv "$artifacts\reopen-readback.log" $ScriptTimeoutSeconds
                $r.phase = 'reopen-readback'; $suite.steps += $r; $suite.status = $r.status
            }
        } catch { $suite.status = 'failed'; $suite.reason = $_.Exception.Message }
        finally {
            $graceful = Close-WATestApp $app "$artifacts\final-app.log"; $app = $null
            $closed = Wait-WAPortClosed $port
            $suite.cleanup = @{ portClosed = $closed; gracefulQuit = $graceful }
            $report.cleanup["$($entry.name)-port-closed"] = $closed
            if (-not $closed -or -not $graceful) { $suite.status = 'failed' }
            $password = $null; $childEnv.Remove('NOTAYA_TEST_PASSWORD')
            $watch.Stop(); $suite.durationSeconds = [math]::Round($watch.Elapsed.TotalSeconds, 3)
            Complete-WASuite $suite
            $report.suites += $suite
            Write-WAJson $report "$($run.output)\in-progress.json"
        }
        Write-Output "CHECKPOINT: $($entry.name) $($suite.status) and port cleanup done"
        if (-not $closed) { throw 'FI-01 cleanup failed; no further app launches' }
    }
    # The installed-path probe must exercise the resolver without either home override.
    # LOCALAPPDATA still points ONLY at this run's disposable profile. No NSIS/registry work.
    $pathHome = Join-Path $run.home 'installed-paths'
    $artifacts = Join-Path $run.output 'installed-paths'
    $program = Join-Path $pathHome 'program'
    $isolatedProfile = Join-Path $pathHome 'localappdata'
    $portable = Join-Path $pathHome 'portable'
    foreach ($dir in @($artifacts, $program, "$isolatedProfile\Notaya Notes\data")) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
    Copy-Item -LiteralPath $Exe -Destination "$program\Notaya.exe"
    [IO.File]::WriteAllText("$isolatedProfile\Notaya Notes\data\shell-prefs.json", '{"closeToTray":false,"startMinimized":false}')
    $portableCopy = New-WATestApp $Exe $portable
    $suite = @{ script = 'tools/real-exe-tests/installed.mjs'; name = 'installed-paths'; status = 'passed'; checkIds = @('W217')
                home = $pathHome; evidence = $artifacts; coverage = 'partial'; steps = @(); durationSeconds = 0 }
    $port = Get-WAFreePort; $watch = [Diagnostics.Stopwatch]::StartNew()
    try {
        foreach ($phase in @('first', 'second', 'portable', 'portable-reopen')) {
            $isPortable = $phase -like 'portable*'
            $copy = if ($isPortable) { $portableCopy } else { "$program\Notaya.exe" }
            $launchHome = if ($isPortable) { $portable } else { $program }
            $data = if ($isPortable) { "$portable\data" } else { "$isolatedProfile\Notaya Notes\data" }
            $local = if ($isPortable) { "$pathHome\portable-profile" } else { $isolatedProfile }
            New-Item -ItemType Directory -Path $local -Force | Out-Null
            $overrides = @{ NOTAYA_HOME = $null; CIPHERLEAF_HOME = $null; LOCALAPPDATA = $local }
            $app = Start-WATestApp $copy $launchHome $port $run $overrides
            Wait-WAHttp $port $app '/json'
            $legacyPhase = if ($isPortable) { 'portable' } else { $phase }
            $arguments = @("$PSScriptRoot\legacy-adapter.mjs", 'installed.mjs', "$artifacts\$phase", "$port", $legacyPhase, $data)
            $r = Invoke-WACommand $node $arguments $envs "$artifacts\$phase.log" $ScriptTimeoutSeconds
            $r.phase = $phase; $suite.steps += $r
            if ($r.status -ne 'passed') { $suite.status = $r.status; break }
            $graceful = Close-WATestApp $app "$artifacts\$phase-app.log"; $app = $null
            if (-not $graceful -or -not (Wait-WAPortClosed $port)) { throw 'Path-mode test failed normal quit/port closure' }
            if (-not (Test-Path -LiteralPath "$data\vault.nota")) { throw 'Expected disposable vault was not written' }
            if (-not $isPortable -and (Test-Path -LiteralPath "$program\data")) { throw 'Installed exe incorrectly wrote beside itself' }
            if ($isPortable -and @(Get-ChildItem -LiteralPath $local -Force | Where-Object { $_.Name -ne 'Microsoft' }).Count) { throw 'Portable app wrote unexpected files into disposable LocalAppData' }
        }
    } catch { $suite.status = 'failed'; $suite.reason = $_.Exception.Message }
    finally {
        if ($null -ne $app) { $null = Close-WATestApp $app "$artifacts\final-app.log"; $app = $null }
        $report.cleanup.installedPathsPortClosed = Wait-WAPortClosed $port
        if (-not $report.cleanup.installedPathsPortClosed) { $suite.status = 'failed' }
        $watch.Stop(); $suite.durationSeconds = $watch.Elapsed.TotalSeconds
        Complete-WASuite $suite
        $report.suites += $suite
    }
    Write-Output "CHECKPOINT: installed and portable path assertions $($suite.status) done"
} catch { $report.fatalError = $_.Exception.Message }
finally {
    Stop-WAProcess $app
    [Environment]::SetEnvironmentVariable('WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS', $null, 'Process')
    $report.cleanup.webviewEnvironmentCleared = -not [bool]$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
    if ($null -ne $port) { $report.cleanup.finalPortClosed = Wait-WAPortClosed $port }
    $rows = if ($null -ne $manifest) { $manifest.checks } else { @(Get-WAPlanRows) }
    foreach ($row in @($rows | Where-Object { $_.kind -eq 'B' })) {
            $suites = @($report.suites | Where-Object { $row.id -in $_.checkIds })
            $failed = @($suites | Where-Object { $_.status -eq 'failed' })
            $report.checks += @{ id = $row.id; kind = 'B'; status = $(if ($failed.Count) { 'failed' } else { 'not run' }); coverage = 'partial'
                script = @($suites | ForEach-Object { $_.script }); evidence = @($suites | ForEach-Object { $_.evidence }); route = $row.route
                defectGroup = $(if ($failed.Count) { 'HARNESS-B' } else { '' }); reason = $row.reason }
    }
    Save-WAReport $report $run 'group-b-results.json'
}
Write-Output "CHECKPOINT: group B JSON and cleanup report done: $script:WAOut\group-b-results.json"
if ($report.fatalError -or @($report.suites | Where-Object { $_.status -ne 'passed' }).Count -or @($report.cleanup.Values | Where-Object { $_ -eq $false }).Count) { exit 1 }
