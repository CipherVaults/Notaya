# Shared by the whole-app runners. Windows PowerShell 5.1; no downloads or installs.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:WARepo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$script:WAOut = 'D:\Notaya-lanes\out\whole-app'
$script:WATests = 'D:\Notaya Tests\whole-app'

function Write-WAJson($Value, [string]$Path) {
    $json = ConvertTo-Json -InputObject $Value -Depth 40
    [IO.File]::WriteAllText($Path, $json, (New-Object Text.UTF8Encoding $false))
}

function New-WARun([string]$Group) {
    # Never reuse, clear or delete a previous run. Both destinations are fixed, external roots.
    $id = (Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssfffZ') + '-' + [guid]::NewGuid().ToString('N')
    $run = Join-Path $script:WAOut "$Group-$id"
    $fixtureHome = Join-Path $script:WATests "$Group-$id"
    foreach ($dir in @($run, $fixtureHome, "$fixtureHome\temp", "$fixtureHome\profile")) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
    [pscustomobject]@{ id = $id; output = $run; home = $fixtureHome }
}

function Get-WASource {
    Push-Location $script:WARepo
    try {
        $head = (& git rev-parse HEAD | Out-String).Trim()
        $merge = (& git rev-parse -q --verify MERGE_HEAD 2>$null | Out-String).Trim()
        $status = @(& git status --porcelain=v1)
        $hashes = @()
        # Hash staged AND unstaged tracked changes plus all runner inputs, including untracked files.
        $paths = @(& git diff HEAD --name-only) + @('docs/plan/2026-11-09-whole-app-test-plan.md')
        $paths += @(Get-ChildItem $PSScriptRoot -File | ForEach-Object { $_.FullName })
        $paths += @('vite.mock.config.ts', 'vite.views.config.ts')
        $paths += @(Get-ChildItem "$script:WARepo\dev-mock", "$script:WARepo\tools\real-exe-tests" -File | ForEach-Object { $_.FullName })
        foreach ($path in @($paths | Sort-Object -Unique)) {
            if (Test-Path -LiteralPath $path -PathType Leaf) {
                $hashes += @{ path = $path; sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash }
            }
        }
        @{ head = $head; mergeHead = $merge; status = $status; dirtyAndRunnerHashes = $hashes }
    } finally { Pop-Location }
}

function Get-WAEnvironment($Run, [hashtable]$Extra = @{}) {
    $e = @{
        NOTAYA_HOME = $Run.home; CIPHERLEAF_HOME = $Run.home
        TEMP = "$($Run.home)\temp"; TMP = "$($Run.home)\temp"
        LOCALAPPDATA = "$($Run.home)\profile"; CARGO_NET_OFFLINE = 'true'
        WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $null
        NOTAYA_PLAYWRIGHT_MODULE = 'D:\Notaya-lanes\tools\node_modules\playwright-core'
        NOTAYA_CHROMIUM_PATH = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
    }
    foreach ($key in $Extra.Keys) { $e[$key] = $Extra[$key] }
    return $e
}

# A private Windows job owns every process we start and all its descendants. Closing it kills
# only that tree, including Chrome/WebView children after a timeout or an early parent exit.
if (-not ('WholeAppJob' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
public sealed class WholeAppJob : IDisposable {
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode)] static extern IntPtr CreateJobObject(IntPtr a, string n);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool SetInformationJobObject(IntPtr h, int c, IntPtr p, uint n);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool AssignProcessToJobObject(IntPtr j, IntPtr p);
    [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
    [StructLayout(LayoutKind.Sequential)] struct Basic { public long a,b; public uint flags; public UIntPtr min,max; public uint count; public UIntPtr affinity; public uint priority,scheduling; }
    [StructLayout(LayoutKind.Sequential)] struct IO { public ulong a,b,c,d,e,f; }
    [StructLayout(LayoutKind.Sequential)] struct Extended { public Basic basic; public IO io; public UIntPtr a,b,c,d; }
    IntPtr handle;
    public WholeAppJob() {
        handle=CreateJobObject(IntPtr.Zero,null);
        if(handle==IntPtr.Zero) throw new Win32Exception();
        var info=new Extended(); info.basic.flags=0x2000; // KILL_ON_JOB_CLOSE
        int size=Marshal.SizeOf(info); IntPtr ptr=Marshal.AllocHGlobal(size);
        try { Marshal.StructureToPtr(info,ptr,false); if(!SetInformationJobObject(handle,9,ptr,(uint)size)) throw new Win32Exception(); }
        catch { Dispose(); throw; } finally { Marshal.FreeHGlobal(ptr); }
    }
    public void Assign(IntPtr process) { if(!AssignProcessToJobObject(handle,process)) throw new Win32Exception(); }
    public void Dispose() { if(handle!=IntPtr.Zero) { CloseHandle(handle); handle=IntPtr.Zero; } }
}
'@
}

function ConvertTo-WAArgument([string]$Value) {
    # Windows CommandLineToArgvW quoting, including quotes and terminal backslashes.
    '"' + ([regex]::Replace([regex]::Replace($Value, '(\\*)"', '$1$1\"'), '(\\+)$', '$1$1')) + '"'
}

function Start-WAProcess([string]$File, [string[]]$Arguments, [hashtable]$Environment, [string]$WorkingDirectory = $script:WARepo) {
    $psi = New-Object Diagnostics.ProcessStartInfo
    $psi.FileName = $File
    $psi.Arguments = (@($Arguments | ForEach-Object { ConvertTo-WAArgument $_ }) -join ' ')
    $psi.WorkingDirectory = $WorkingDirectory
    $psi.UseShellExecute = $false; $psi.CreateNoWindow = $true
    $psi.RedirectStandardOutput = $true; $psi.RedirectStandardError = $true
    foreach ($key in $Environment.Keys) {
        if ($null -eq $Environment[$key]) { $psi.EnvironmentVariables.Remove($key) }
        else { $psi.EnvironmentVariables[$key] = [string]$Environment[$key] }
    }
    $job = New-Object WholeAppJob
    $p = New-Object Diagnostics.Process
    $p.StartInfo = $psi
    $watch = [Diagnostics.Stopwatch]::StartNew()
    try {
        if (-not $p.Start()) { throw "Could not start $File" }
        $job.Assign($p.Handle)
        [pscustomobject]@{
            process = $p; job = $job; watch = $watch
            environment = $Environment
            stdout = $p.StandardOutput.ReadToEndAsync(); stderr = $p.StandardError.ReadToEndAsync()
        }
    } catch {
        $job.Dispose()
        try { if (-not $p.HasExited) { $p.Kill() } } catch {}
        throw
    }
}

function Stop-WAProcess($Handle) {
    if ($null -eq $Handle) { return }
    $Handle.job.Dispose()
    $null = $Handle.process.WaitForExit(10000)
}

function Complete-WAProcess($Handle, [string]$Log, [int]$TimeoutSeconds = 180) {
    $timedOut = -not $Handle.process.WaitForExit($TimeoutSeconds * 1000)
    # Also reap descendants if the parent exited by itself but leaked a browser.
    Stop-WAProcess $Handle
    $Handle.watch.Stop()
    $out = $Handle.stdout.GetAwaiter().GetResult()
    $err = $Handle.stderr.GetAwaiter().GetResult()
    $body = $out + "`r`n[stderr]`r`n" + $err
    [IO.File]::WriteAllText($Log, $body, (New-Object Text.UTF8Encoding $false))
    $code = $Handle.process.ExitCode
    $status = if ($timedOut -or $code -ne 0 -or $body -match '(?m)^\s*(FAIL\b|FAILED\b|not ok\b)') { 'failed' } else { 'passed' }
    @{ status = $status; exitCode = $code; timedOut = $timedOut
       durationSeconds = [math]::Round($Handle.watch.Elapsed.TotalSeconds, 3)
       evidence = $Log; last30Lines = @($body -split '\r?\n' | Select-Object -Last 30) }
}

function Invoke-WACommand([string]$File, [string[]]$Arguments, [hashtable]$Environment, [string]$Log, [int]$TimeoutSeconds = 180) {
    $handle = $null
    try {
        $handle = Start-WAProcess $File $Arguments $Environment
        Complete-WAProcess $handle $Log $TimeoutSeconds
    } catch {
        $failureText = $_.Exception.Message
        try { [IO.File]::WriteAllText($Log, $failureText, (New-Object Text.UTF8Encoding $false)) } catch {}
        @{ status = $(if ($null -ne $handle) { 'failed' } else { 'not run' }); exitCode = $null; timedOut = $false; durationSeconds = 0
           evidence = $Log; last30Lines = @($failureText); reason = $failureText }
    } finally { Stop-WAProcess $handle }
}

function Complete-WASuite($Suite) {
    $Suite.exitCode = if ($Suite.status -eq 'passed') { 0 } else { $null }
    $Suite.timedOut = @($Suite.steps | Where-Object { $_.ContainsKey('timedOut') -and $_.timedOut }).Count -gt 0
    $Suite.last30Lines = @($Suite.steps | ForEach-Object { if ($_.ContainsKey('last30Lines')) { $_.last30Lines } } | Select-Object -Last 30)
    foreach ($step in $Suite.steps) {
        if ($step.ContainsKey('exitCode') -and $null -ne $step.exitCode -and $step.exitCode -ne 0) { $Suite.exitCode = $step.exitCode }
    }
}

function Get-WAFreePort {
    $listener = New-Object Net.Sockets.TcpListener([Net.IPAddress]::Loopback, 0)
    $listener.Start()
    try { return $listener.LocalEndpoint.Port } finally { $listener.Stop() }
}

function Test-WAPortClosed([int]$Port) {
    # Query all addresses: a wildcard/IPv6 listener is also a failure. Errors must fail closed.
    $all = @(Get-NetTCPConnection -State Listen -ErrorAction Stop)
    return @($all | Where-Object { $_.LocalPort -eq $Port }).Count -eq 0
}

function Wait-WAPortClosed([int]$Port) {
    for ($i = 0; $i -lt 40; $i++) {
        try { if (Test-WAPortClosed $Port) { return $true } } catch { return $false }
        Start-Sleep -Milliseconds 250
    }
    return $false
}

function Wait-WAHttp([int]$Port, $Handle, [string]$Path = '/') {
    for ($i = 0; $i -lt 100; $i++) {
        if ($Handle.process.HasExited) { throw 'Child exited before its loopback endpoint was ready' }
        $listeners = @(Get-NetTCPConnection -State Listen -ErrorAction Stop | Where-Object { $_.LocalPort -eq $Port })
        if (-not $listeners.Count) { Start-Sleep -Milliseconds 300; continue }
        # Do not attach to a process that raced us for the selected free port. Also reject
        # wildcard bindings: a debug endpoint must be loopback only, even on disposable data.
        $processes = @(Get-CimInstance Win32_Process)
        $ids = New-Object 'System.Collections.Generic.HashSet[int]'
        $null = $ids.Add($Handle.process.Id)
        do {
            $added = $false
            foreach ($p in $processes) { if ($ids.Contains([int]$p.ParentProcessId) -and $ids.Add([int]$p.ProcessId)) { $added = $true } }
        } while ($added)
        foreach ($listener in $listeners) {
            if (-not $ids.Contains([int]$listener.OwningProcess) -or $listener.LocalAddress -notin @('127.0.0.1','::1')) {
                throw "Port $Port is not a loopback listener owned by this test process tree"
            }
        }
        try {
            $r = Invoke-WebRequest -UseBasicParsing -Uri "http://127.0.0.1:$Port$Path" -TimeoutSec 1
            if ($r.StatusCode -eq 200) { return }
        } catch {}
        Start-Sleep -Milliseconds 300
    }
    throw "Loopback endpoint $Port did not become ready"
}

function New-WAReport([string]$Group, $Run) {
    @{ schemaVersion = 1; group = $Group; runId = $Run.id; startedUtc = [DateTime]::UtcNow.ToString('o')
       executor = "$env:USERNAME@$env:COMPUTERNAME (Claude PC runner)"; home = $Run.home
       source = Get-WASource; suites = @(); checks = @(); cleanup = @{}; fatalError = $null }
}

function Save-WAReport($Report, $Run, [string]$Name) {
    $Report.finishedUtc = [DateTime]::UtcNow.ToString('o')
    try {
        $Report.sourceAfter = Get-WASource
        $Report.sourceChangedDuringRun = (ConvertTo-Json $Report.source -Depth 20 -Compress) -ne (ConvertTo-Json $Report.sourceAfter -Depth 20 -Compress)
    } catch { $Report.sourceAfterError = $_.Exception.Message; $Report.sourceChangedDuringRun = $true }
    Write-WAJson $Report (Join-Path $Run.output $Name)
    # Unique run is the archive; fixed file is only the latest pointer/copy.
    Copy-Item -LiteralPath (Join-Path $Run.output $Name) -Destination (Join-Path $script:WAOut $Name) -Force
}

function Assert-WANoApp {
    $apps = @(Get-Process -Name Notaya,Cipherleaf -ErrorAction SilentlyContinue)
    if ($apps.Count) { throw 'An existing Notaya/Cipherleaf is running. Refusing to inspect or launch against it; close it yourself first.' }
}

function New-WATestApp([string]$Exe, [string]$fixtureHome) {
    if (-not (Test-Path -LiteralPath $Exe -PathType Leaf)) { throw "Missing reviewed exe: $Exe" }
    New-Item -ItemType Directory -Path "$fixtureHome\data" -Force | Out-Null
    Copy-Item -LiteralPath $Exe -Destination "$fixtureHome\Notaya.exe"
    [IO.File]::WriteAllText("$fixtureHome\portable.txt", 'whole-app disposable copy')
    [IO.File]::WriteAllText("$fixtureHome\data\shell-prefs.json", '{"closeToTray":false,"startMinimized":false}')
    return "$fixtureHome\Notaya.exe"
}

function Start-WATestApp([string]$Exe, [string]$fixtureHome, [int]$Port, $Run, [hashtable]$Overrides = @{}) {
    Assert-WANoApp
    if (-not (Test-WAPortClosed $Port)) { throw "Port $Port is already listening" }
    $envs = Get-WAEnvironment $Run @{
        NOTAYA_HOME = $fixtureHome; CIPHERLEAF_HOME = $fixtureHome
        WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-address=127.0.0.1 --remote-debugging-port=$Port"
    }
    foreach ($key in $Overrides.Keys) { $envs[$key] = $Overrides[$key] }
    Start-WAProcess $Exe @() $envs $fixtureHome
}

function Get-WAPlanRows {
    foreach ($line in Get-Content -LiteralPath "$script:WARepo\docs\plan\2026-11-09-whole-app-test-plan.md") {
        if ($line -match '^\| (W\d{3}) \| ([ABC]) \|') {
            [pscustomobject]@{ id = $Matches[1]; kind = $Matches[2]; route = 'not run'; reason = 'Runner prerequisites unavailable; see fatalError.' }
        }
    }
}

function Close-WATestApp($Handle, [string]$Log, [int]$Port = 0, [bool]$RequireAlreadyExited = $false) {
    if ($null -eq $Handle) { return $false }
    $p = $Handle.process
    $quit = @{ pid = $p.Id; port = $Port; alreadyExited = $p.HasExited; requiredScenarioClose = $RequireAlreadyExited }
    # The close scenario must prove its own UI close, never be rescued by another quit.
    if ($RequireAlreadyExited -and -not $p.HasExited) { $null = $p.WaitForExit(10000) }
    if (-not $p.HasExited -and -not $RequireAlreadyExited) {
        if ($Port -gt 0) {
            $node = (Get-Command node -ErrorAction Stop).Source
            $quit.request = Invoke-WACommand $node @("$PSScriptRoot\quit-app.mjs", "$Port") $Handle.environment "$Log-quit-request.log" 20
            $null = $p.WaitForExit(20000)
        }
        if (-not $p.HasExited) {
            $quit.windowCloseRequested = $p.CloseMainWindow()
            $null = $p.WaitForExit(20000)
        }
    }
    $graceful = $p.HasExited -and $p.ExitCode -eq 0
    $quit.gracefulExit = $graceful
    $quit.forcedOwnedTreeCleanup = -not $p.HasExited
    if ($p.HasExited) { $quit.exitCode = $p.ExitCode }
    $null = Complete-WAProcess $Handle $Log 1
    Write-WAJson $quit "$Log-quit.json"
    return $graceful
}
