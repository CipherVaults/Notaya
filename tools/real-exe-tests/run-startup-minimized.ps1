<#
  X-1, reported by Mohamed Sayed (AbyssWatcher2), via Ahmed.
  Run OUTSIDE Codex's sandbox. Node 24 + Windows PowerShell 5.1; no installs.
  Never writes the Run key. -Mode RunKey reads it and refuses anything except
  exactly the supplied exe, quoted, followed by --minimized. All starts use a
  newly-created home under D:\Notaya Tests\v1-6-X1, including a Run-key start.
  -DelayPageMs holds bundle requests on a hidden reload (not a real sign-in).
  -HoldFramesMs holds animation callbacks on that reload (test-only injection).
  Artifacts/data are kept, not deleted. Debug env belongs only to child processes.
#>
param(
  [string]$Exe,
  [string]$Root = 'D:\Notaya Tests\v1-6-X1',
  [ValidateSet('Minimized', 'Normal', 'RunKey')][string]$Mode = 'Minimized',
  [ValidateRange(0, 60000)][int]$DelayPageMs = 0,
  [ValidateRange(0, 60000)][int]$HoldFramesMs = 0,
  [ValidateRange(0, 60)][int]$LaunchDelaySeconds = 0,
  [ValidateRange(1024, 65535)][int]$Port = 9341
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'common.ps1')
# Fail closed if listener enumeration itself is unavailable (not "zero listeners").
function Get-ListenerCount([int]$Port) {
  @(Get-NetTCPConnection -State Listen -ErrorAction Stop | Where-Object { $_.LocalPort -eq $Port }).Count
}
$Exe = Resolve-TestExe $Exe
# The legacy guard is global: even a different-home test must not hand over to a real app.
if (@(Get-Process Notaya, Cipherleaf -ErrorAction SilentlyContinue).Count) {
  throw 'Quit all Notaya/Cipherleaf copies yourself first. No existing process will be contacted or stopped.'
}
Assert-PortFree $Port
$allowedRoot = [IO.Path]::GetFullPath('D:\Notaya Tests\v1-6-X1').TrimEnd('\')
$rootPath = [IO.Path]::GetFullPath($Root).TrimEnd('\')
if ($rootPath -ne $allowedRoot -and -not $rootPath.StartsWith($allowedRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
  throw 'Root must be D:\Notaya Tests\v1-6-X1 or a child of it.'
}
# Reject junctions/symlinks before creating anything; never redirect test data elsewhere.
$ancestor = $rootPath
while ($ancestor) {
  if (Test-Path -LiteralPath $ancestor) {
    if ((Get-Item -LiteralPath $ancestor).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Test root has a reparse-point ancestor.' }
  }
  $parent = Split-Path -Parent $ancestor
  if ($parent -eq $ancestor) { break }; $ancestor = $parent
}
$homePath = Join-Path $rootPath ('startup-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ') + '-' + [guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path (Join-Path $homePath 'data')
$command = '"' + $Exe + '"' + $(if ($Mode -eq 'Normal') { '' } else { ' --minimized' })
if ($Mode -eq 'RunKey') {
  $actual = (Get-ItemProperty -LiteralPath 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name Notaya -ErrorAction Stop).Notaya
  if (-not [string]::Equals($actual, $command, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Run-key command does not exactly match the supplied test exe plus --minimized. Nothing launched; do not change the real entry for this test.'
  }
  $command = $actual
}

if (-not ('NotayaStartupWindows' -as [type])) {
  Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class NotayaStartupWindows {
  public delegate bool Callback(IntPtr hwnd, IntPtr data);
  [DllImport("user32.dll")] static extern bool EnumWindows(Callback cb, IntPtr data);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] static extern bool IsIconic(IntPtr hwnd);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int size);
  public class Window { public string title; public bool visible; public bool minimized; }
  public static Window[] ForProcess(int id) {
    var found = new List<Window>();
    EnumWindows((h, _) => {
      uint pid; GetWindowThreadProcessId(h, out pid);
      if (pid == id) {
        var text = new StringBuilder(256); GetWindowText(h, text, text.Capacity);
        if (text.ToString() == "Notaya") found.Add(new Window { title=text.ToString(), visible=IsWindowVisible(h), minimized=IsIconic(h) });
      }
      return true;
    }, IntPtr.Zero);
    return found.ToArray();
  }
}
'@
}

function Start-StartupChild([string]$Arguments, [bool]$Debug) {
  $info = New-Object Diagnostics.ProcessStartInfo
  $info.FileName = $Exe
  $info.Arguments = $Arguments
  $info.UseShellExecute = $false
  $info.CreateNoWindow = $true
  $info.WorkingDirectory = Split-Path -Parent $Exe
  $info.EnvironmentVariables['NOTAYA_HOME'] = $homePath
  $info.EnvironmentVariables['CIPHERLEAF_HOME'] = $homePath
  $info.EnvironmentVariables['WEBVIEW2_USER_DATA_FOLDER'] = Join-Path $homePath 'data\webview'
  if ($Debug) {
    $info.EnvironmentVariables['WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS'] = "--remote-debugging-address=127.0.0.1 --remote-debugging-port=$Port"
  } else { $info.EnvironmentVariables.Remove('WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS') }
  [Diagnostics.Process]::Start($info)
}
function Invoke-StartupProbe([string]$Phase, [string]$Expected, [int]$PageDelay = 0, [int]$Frames = 0) {
  & node (Join-Path $PSScriptRoot 'startup-minimized.mjs') --port $Port --phase $Phase --expect $Expected --delay-page-ms $PageDelay --hold-frames-ms $Frames --out (Join-Path $homePath "$Phase.json")
  if ($LASTEXITCODE -ne 0) { throw "Startup probe '$Phase' failed. Evidence retained in $homePath" }
}
function Save-NativeWindows([string]$Phase) {
  $windows = @([NotayaStartupWindows]::ForProcess($script:appProcess.Id))
  $windows | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $homePath "$Phase-windows.json") -Encoding UTF8
  Write-Output ("$Phase native windows: " + (ConvertTo-Json -InputObject $windows -Compress))
  if ($windows.Count -ne 1) { throw 'Expected exactly one Notaya top-level window for the child.' }
  $expected = $Mode -eq 'Normal' -or $Phase -eq 'restored'
  if ($windows[0].visible -ne $expected -or $windows[0].minimized) { throw "Unexpected native window state in $Phase" }
}
function Get-OwnedChildren {
  # Remember descendants while the parent still exists. Never kill by name or port.
  if ($script:appProcess.HasExited) { return @() }
  try { $all = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId) }
  catch { Write-Warning 'Could not enumerate child processes; normal Quit and port check still required.'; return @() }
  if ($script:appProcess.HasExited) { return @() }
  $known = @($script:appProcess.Id); $owned = @()
  do {
    $next = @($all | Where-Object { $_.ParentProcessId -in $known -and $_.ProcessId -notin $known })
    foreach ($child in $next) {
      $known += [int]$child.ProcessId
      try {
        $p = Get-Process -Id $child.ProcessId -ErrorAction Stop
        # Retain a handle to this exact descendant, rather than reopening its
        # PID at cleanup (the OS could have reused that PID in the meantime).
        $null = $p.Handle
        $started = $null
        try { if ($p.StartTime) { $started = $p.StartTime.ToUniversalTime().Ticks } } catch { }
        $owned += @{ id = $p.Id; started = $started; process = $p }
      } catch { }
    }
  } while ($next.Count)
  return $owned
}
function Stop-OwnedStartupChild($Owned) {
  $child = $Owned.process
  if (-not $child -or $child.Id -ne $Owned.id) { return }
  try {
    if ($child.HasExited) { return }
    $started = $null
    try { if ($child.StartTime) { $started = $child.StartTime.ToUniversalTime().Ticks } } catch { }
    if ($null -ne $started -and $null -ne $Owned.started -and $started -ne $Owned.started) { return }
    # If StartTime is unreadable, the recorded ID plus retained process handle
    # still refer to the owned child. Never find/stop a process by PID alone.
    $child.Kill()
  } catch { Write-Warning 'Could not stop an owned child; the listener check still decides cleanup success.' }
}

$appProcess = $null; $second = $null; $failed = $false
try {
  $metadata = [ordered]@{ utc = [DateTime]::UtcNow.ToString('o'); reporter = 'Mohamed Sayed (AbyssWatcher2)';
    commandLine = $command; launchMode = $Mode; exeSha256 = (Get-FileHash -LiteralPath $Exe -Algorithm SHA256).Hash;
    home = $homePath; port = $Port; launchDelaySeconds = $LaunchDelaySeconds;
    note = 'CDP can affect timing. DOM/layout is not compositor-pixel or real-sign-in proof.' }
  $metadata | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $homePath 'launch.json') -Encoding UTF8
  "Evidence: $homePath"
  "Command line: $command"
  if ($LaunchDelaySeconds) { Start-Sleep -Seconds $LaunchDelaySeconds }
  # ProcessStartInfo quotes FileName itself and passes the validated Run-key argument
  # suffix verbatim. No shell, registry write, command evaluation or wrapper executable.
  $argumentSuffix = if ($Mode -eq 'Normal') { '' } else { $command.Substring($command.LastIndexOf('"') + 1).TrimStart() }
  $appProcess = Start-StartupChild $argumentSuffix $true
  $expected = if ($Mode -eq 'Normal') { 'visible' } else { 'hidden' }
  Invoke-StartupProbe 'initial' $expected
  Save-NativeWindows 'initial'
  if ($DelayPageMs -or $HoldFramesMs) {
    Invoke-StartupProbe 'delayed-reload' $expected $DelayPageMs $HoldFramesMs
    Save-NativeWindows 'delayed-reload'
  }
  # Same native show_main path as Open Notaya in the tray; do not call window_ready.
  $second = Start-StartupChild '' $false
  if (-not $second.WaitForExit(10000)) { throw 'Second start did not hand over within 10 seconds.' }
  $appProcess.Refresh()
  if ($appProcess.HasExited) { throw 'First process exited during single-instance handover.' }
  if ($second.ExitCode -ne 0) { throw "Second start exit code $($second.ExitCode)" }
  Invoke-StartupProbe 'restored' 'visible'
  Save-NativeWindows 'restored'
  'CHECKPOINT: X-1 real-exe DOM, visibility and single-instance checks done; inspect pixels and actual tray by hand.'
} catch {
  $failed = $true
  Write-Warning $_.Exception.Message
} finally {
  $ownedChildren = if ($appProcess) { @(Get-OwnedChildren) } else { @() }
  if ($appProcess -and -not $appProcess.HasExited) {
    # Best-effort normal Quit so pending writes flush. Fall back only for this PID.
    & node (Join-Path $PSScriptRoot 'startup-minimized.mjs') --port $Port --phase quit --expect visible --timeout-ms 2000
    if (-not $appProcess.WaitForExit(8000)) { Stop-Process -Id $appProcess.Id -Force; $appProcess.WaitForExit() }
  }
  if ($second -and -not $second.HasExited) { Stop-Process -Id $second.Id -Force }
  foreach ($owned in $ownedChildren) {
    Stop-OwnedStartupChild $owned
  }
  for ($i = 0; $i -lt 20 -and (Get-ListenerCount $Port) -gt 0; $i++) { Start-Sleep -Milliseconds 250 }
  $listeners = Get-ListenerCount $Port
  "Debug port $Port listeners after cleanup: $listeners"
  'Parent WebView2 environment was never changed. Throwaway evidence retained.'
  if ($listeners) { $failed = $true; Write-Warning 'Debug listener remains: cleanup FAILED; inspect the child WebView2 process before any other test.' }
}
if ($failed) { exit 1 }
