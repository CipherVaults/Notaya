<# Claude PC executor only. No installs/network; all Cargo calls are offline/locked.
   powershell -NoProfile -File .\tools\whole-app\run-group-a.ps1
   Writes latest group-a-results.json plus immutable per-run logs/artifacts/manifest.
#>
param([ValidateRange(10,3600)][int]$ScriptTimeoutSeconds = 240,
      [ValidateRange(10,7200)][int]$CargoTimeoutSeconds = 900)
. (Join-Path $PSScriptRoot 'common.ps1')
[Environment]::SetEnvironmentVariable('WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS', $null, 'Process')
$run = New-WARun 'group-a'
$report = New-WAReport 'A' $run
$envs = Get-WAEnvironment $run @{ CARGO_TARGET_DIR = 'D:\Notaya-lanes\_target'; NOTAYA_SHOTS_HEAD = $report.source.head }
$node = $null
$servers = @(); $ports = @(); $manifest = $null
function Save-AProgress {
    Write-WAJson $report (Join-Path $run.output 'in-progress.json')
}
try {
    $node = (Get-Command node -ErrorAction Stop).Source
    $manifestPath = Join-Path $run.output 'manifest.json'
    $inventory = Invoke-WACommand $node @("$PSScriptRoot\catalog.cjs", 'write-manifest', $manifestPath) $envs "$($run.output)\inventory.log" 30
    if ($inventory.status -ne 'passed') { throw 'Could not generate manifest; inspect inventory.log' }
    $manifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
    # Refuse an unclassified executable suite instead of silently dropping newly added coverage.
    $known = @($manifest.mocks.file) + @($manifest.helpers)
    $extra = @(Get-ChildItem "$script:WARepo\dev-mock" -File | Where-Object { $_.Extension -in '.cjs','.mjs','.mts' -and $_.Name -notin $known })
    if ($extra.Count) { throw "Unclassified dev-mock files: $($extra.Name -join ', ')" }
    $mockPort = Get-WAFreePort; $ports += $mockPort
    $viewsPort = Get-WAFreePort
    while ($viewsPort -eq $mockPort) { $viewsPort = Get-WAFreePort }
    $ports += $viewsPort
    $mockUrl = "http://127.0.0.1:$mockPort"; $viewsUrl = "http://127.0.0.1:$viewsPort"
    $available = @{ mock = $false; views = $false }
    # Run npm's existing CLI through Node, avoiding cmd.exe quoting and download-on-demand.
    $npmCmd = (Get-Command npm.cmd -ErrorAction Stop).Source
    $npmCli = Join-Path (Split-Path $npmCmd) 'node_modules\npm\bin\npm-cli.js'
    foreach ($mode in @('mock', 'views')) {
        $server = $null; $port = if ($mode -eq 'mock') { $mockPort } else { $viewsPort }
        try {
            if (-not (Test-WAPortClosed $port)) { throw "Port $port was taken before launch" }
            $commandArgs = if ($mode -eq 'mock') {
                @($npmCli, 'run', 'mock', '--', '--configLoader', 'native', '--host', '127.0.0.1', '--port', "$port", '--strictPort')
            } else {
                @("$script:WARepo\node_modules\vite\bin\vite.js", '--config', 'vite.views.config.ts', '--configLoader', 'native', '--host', '127.0.0.1', '--port', "$port", '--strictPort')
            }
            $server = Start-WAProcess $node $commandArgs $envs
            $servers += @{ handle = $server; name = $mode; port = $port }
            Wait-WAHttp $port $server
            $available[$mode] = $true
        } catch { $report.suites += @{ script = "$mode server"; status = 'not run'; reason = $_.Exception.Message; checkIds = @() } }
    }
    foreach ($entry in $manifest.mocks) {
        $name = [IO.Path]::GetFileNameWithoutExtension($entry.file)
        $artifacts = Join-Path $run.output $name
        New-Item -ItemType Directory -Path $artifacts -Force | Out-Null
        $log = Join-Path $artifacts 'suite.log'
        if ($entry.mode -ne 'node' -and -not $available[$entry.mode]) {
            $result = @{ status = 'not run'; exitCode = $null; durationSeconds = 0; evidence = $log; last30Lines = @('Required mock server unavailable') }
        } else {
            $arguments = if ($entry.file.EndsWith('.cjs')) {
                @("$PSScriptRoot\mock-adapter.cjs", $entry.file, $artifacts, $mockUrl, $viewsUrl)
            } else { @('--experimental-strip-types', "$script:WARepo\dev-mock\$($entry.file)") }
            $result = Invoke-WACommand $node $arguments $envs $log $ScriptTimeoutSeconds
        }
        $result.script = "dev-mock/$($entry.file)"; $result.checkIds = @($entry.checkIds)
        $result.coverage = 'supporting'; $report.suites += $result
        Save-AProgress
        Write-Output "CHECKPOINT: $($entry.file) $($result.status) done"
    }
    foreach ($helper in $manifest.helpers) {
        $report.suites += @{ script = "dev-mock/$helper"; status = 'not run'; checkIds = @(); reason = 'Helper, executed through its consumers; not a standalone suite.' }
    }
    # Discover exact tests first. A zero-match filtered Cargo invocation must never count as a pass.
    $cargo = (Get-Command cargo -ErrorAction Stop).Source
    $base = @('test', '--manifest-path', 'src-tauri/Cargo.toml', '--lib', '--offline', '--locked')
    $discovery = Invoke-WACommand $cargo ($base + @('--', '--list')) $envs "$($run.output)\rust-list.log" $CargoTimeoutSeconds
    $discovery.script = 'cargo test --lib --offline --locked -- --list'; $discovery.checkIds = @()
    $report.suites += $discovery
    $testNames = @()
    if ($discovery.status -eq 'passed') {
        $testNames = @(Get-Content -LiteralPath $discovery.evidence | ForEach-Object { if ($_ -match '^(.+): test$') { $Matches[1] } })
    }
    foreach ($area in $manifest.rustAreas.PSObject.Properties) {
        foreach ($prefix in $area.Value) {
            $safeName = $prefix.Replace('::', '-')
            $matches = @($testNames | Where-Object { $_.StartsWith($prefix, [StringComparison]::Ordinal) })
            if ($matches.Count -eq 0) {
                $result = @{ status = 'not run'; exitCode = $null; durationSeconds = 0; evidence = $discovery.evidence; last30Lines = @('No discovered tests, or discovery failed'); matchedTests = @() }
            } else {
                $result = Invoke-WACommand $cargo ($base + @($prefix, '--', '--test-threads=1')) $envs "$($run.output)\rust-$safeName.log" $CargoTimeoutSeconds
                $result.matchedTests = $matches
                if ($result.status -eq 'passed' -and -not (Select-String -LiteralPath $result.evidence -Pattern 'test result: ok\. [1-9][0-9]* passed' -Quiet)) {
                    $result.status = 'not run'; $result.reason = 'No nonzero passing test count in Cargo output'
                }
            }
            $result.script = "cargo test --lib --offline --locked $prefix"; $result.area = $area.Name
            $result.checkIds = @(); $result.coverage = 'supporting'; $report.suites += $result
            Save-AProgress
            Write-Output "CHECKPOINT: Rust $prefix $($result.status) done"
        }
    }
    $bridgeBase = @('test', '--manifest-path', 'src-tauri/Cargo.toml', '-p', 'notaya-bridge', '--offline', '--locked')
    foreach ($variant in @('normal', 'native-ignored')) {
        $commandArguments = if ($variant -eq 'normal') { $bridgeBase } else { $bridgeBase + @('pipe::native::tests', '--', '--include-ignored', '--test-threads=1') }
        $result = Invoke-WACommand $cargo $commandArguments $envs "$($run.output)\bridge-$variant.log" $CargoTimeoutSeconds
        $result.script = "cargo bridge $variant"; $result.checkIds = @(); $result.coverage = 'supporting'
        if ($result.status -eq 'passed' -and -not (Select-String -LiteralPath $result.evidence -Pattern 'test result: ok\. [1-9][0-9]* passed' -Quiet)) {
            $result.status = 'not run'; $result.reason = 'No nonzero passing test count in Cargo output'
        }
        $report.suites += $result; Save-AProgress
    }
    foreach ($command in @(
        @{ name = 'typescript'; file = $node; argv = @("$script:WARepo\node_modules\typescript\bin\tsc", '--noEmit', '-p', 'tsconfig.json') },
        @{ name = 'frontend-build'; file = $node; argv = @($npmCli, 'run', 'build', '--', '--configLoader', 'native') },
        @{ name = 'clippy'; file = $cargo; argv = @('clippy', '--manifest-path', 'src-tauri/Cargo.toml', '--all-targets', '--offline', '--locked') }
    )) {
        $result = Invoke-WACommand $command.file $command.argv $envs "$($run.output)\$($command.name).log" $CargoTimeoutSeconds
        $result.script = $command.name; $result.checkIds = @(); $result.coverage = 'supporting'
        $report.suites += $result; Save-AProgress
    }
} catch { $report.fatalError = $_.Exception.Message }
finally {
    foreach ($server in $servers) {
        Stop-WAProcess $server.handle
        $null = Complete-WAProcess $server.handle "$($run.output)\$($server.name)-server.log" 1
    }
    [Environment]::SetEnvironmentVariable('WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS', $null, 'Process')
    foreach ($port in $ports) { $report.cleanup["port-$port-closed"] = Wait-WAPortClosed $port }
    $report.cleanup.webviewEnvironmentCleared = -not [bool]$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
    $rows = if ($null -ne $manifest) { $manifest.checks } else { @(Get-WAPlanRows) }
    foreach ($row in @($rows | Where-Object { $_.kind -eq 'A' })) {
            $suites = @($report.suites | Where-Object { $row.id -in $_.checkIds })
            $failed = @($suites | Where-Object { $_.status -eq 'failed' })
            $complete = $false
            if ($null -ne $manifest -and $manifest.completeA.PSObject.Properties[$row.id]) {
                $required = @($manifest.completeA.PSObject.Properties[$row.id].Value)
                $passed = @($suites | Where-Object { $_.status -eq 'passed' } | ForEach-Object { [IO.Path]::GetFileName($_.script) })
                $complete = @($required | Where-Object { $_ -notin $passed }).Count -eq 0
            }
            $report.checks += @{ id = $row.id; kind = 'A'; status = $(if ($failed.Count) { 'failed' } elseif ($complete) { 'passed' } else { 'not run' })
                coverage = $(if ($complete) { 'complete' } else { 'partial' }); script = @($suites | ForEach-Object { $_.script }); evidence = @($suites | ForEach-Object { if ($_.ContainsKey('evidence')) { $_.evidence } })
                defectGroup = $(if ($failed.Count) { 'HARNESS-A' } else { '' })
                reason = $(if ($complete) { 'All declared plan variants asserted by the required suite.' } else { 'Existing suites provide supporting assertions only; full plan variants not yet correlated/implemented. Inspect suite statuses and logs.' }) }
        }
    Save-WAReport $report $run 'group-a-results.json'
}
Write-Output "CHECKPOINT: group A JSON and cleanup report done: $script:WAOut\group-a-results.json"
if ($report.fatalError -or @($report.suites | Where-Object { $_.status -ne 'passed' -and -not ($_.ContainsKey('reason') -and $_.reason -like 'Helper,*') }).Count -or @($report.cleanup.Values | Where-Object { $_ -eq $false }).Count) { exit 1 }
