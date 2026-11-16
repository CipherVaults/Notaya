<#
  run-installed.ps1 - real-exe test of where Notaya keeps its notes: installed copy vs portable copy.

  Purpose : (1) starts a COPY of Notaya.exe in a temp "program folder" with NO portable.txt and NO data
            folder (what the setup produces), with LOCALAPPDATA pointed at another temp folder for that
            process only. Checks the first screen (Start new notes / Import), that the notes land in
            <temp>\Notaya Notes\data and not next to the exe, and that a second start opens them by itself.
            (2) starts a copy WITH portable.txt and checks that it still keeps its notes in its own data
            folder, leaves the (temp) LOCALAPPDATA alone and refuses the import command.
  Usage   : powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-installed.ps1 `
              -Exe E:\Project\Notaya\Notaya\src-tauri\target\release\Notaya.exe
            -Exe is required; pass an independently verified build explicitly.
  Needs   : Node 24 (global fetch and WebSocket), Windows PowerShell 5.1, WebView2.
  Safety  : never touches a real vault or the real %LOCALAPPDATA%: that variable is changed for the child
            process only. Everything lives in %TEMP%\clf-installed-*. Debug port 9335 is set for the child
            process only. The finally block stops only the processes this script started, then reports:
            port closed, env var not set, run home retained.
  Not covered (needs a person or UI automation): the native folder dialog of "Import notes from a portable
            copy", and the NSIS setup itself (see docs/execution/2026-10-28-windows-installer.md).
#>
param([string]$Exe)

. (Join-Path $PSScriptRoot 'common.ps1')
$ErrorActionPreference = 'Stop'
$port = 9335
$Exe = Resolve-TestExe $Exe
Assert-PortFree $port

$base = New-TestRunFolder $env:TEMP 'clf-installed'
$prog = "$base\program"; $local = "$base\localappdata"
$prog2 = "$base\portable"; $local2 = "$base\localappdata2"
New-Item -ItemType Directory $prog, $local, $prog2, $local2 | Out-Null
Copy-Item $Exe "$prog\Notaya.exe"                    # installed-style: no marker, no data folder
# Since V1.4.0 the close button hides to the tray; this script closes the window to test the real quit path,
# so the installed-style copy starts with "close to tray" off (its own notes folder, in the throwaway LOCALAPPDATA).
New-Item -ItemType Directory -Force -Path "$local\Notaya Notes\data" | Out-Null
[System.IO.File]::WriteAllText("$local\Notaya Notes\data\shell-prefs.json", '{"closeToTray":false,"startMinimized":true}')
Copy-TestExe $Exe $prog2                             # portable-style: with portable.txt
"exe sha256: " + (Get-FileHash "$prog\Notaya.exe" -Algorithm SHA256).Hash
$realLocal = $env:LOCALAPPDATA
$p = $null
try {
  "== installed copy, first start"
  $p = Start-TestApp "$prog\Notaya.exe" $port '' '' @{ LOCALAPPDATA = $local }
  Start-Sleep 6
  Invoke-NodeCheck "$PSScriptRoot\installed.mjs" @($port, 'first', "$local\Notaya Notes\data")
  Stop-TestApp $p $prog
  Assert-Test (Test-Path -LiteralPath "$local\Notaya Notes\data\vault.nota") 'installed notes exist under throwaway LOCALAPPDATA'
  Assert-Test (-not (Test-Path -LiteralPath "$prog\data")) 'installed copy did not make portable data'

  "== installed copy, second start"
  $p = Start-TestApp "$prog\Notaya.exe" $port '' '' @{ LOCALAPPDATA = $local }
  Start-Sleep 6
  Invoke-NodeCheck "$PSScriptRoot\installed.mjs" @($port, 'second', "$local\Notaya Notes\data")
  Stop-TestApp $p $prog
  Assert-Test (-not (Test-Path -LiteralPath "$prog\data")) 'installed restart did not make portable data'

  "== portable copy"
  $p = Start-TestApp "$prog2\Notaya.exe" $port '' '' @{ LOCALAPPDATA = $local2 }
  Start-Sleep 6
  Invoke-NodeCheck "$PSScriptRoot\installed.mjs" @($port, 'portable', "$prog2\data")
  Stop-TestApp $p $prog2
  Assert-Test (Test-Path -LiteralPath "$prog2\data\vault.nota") 'portable notes exist beside the copied executable'
  Assert-Test (@(Get-ChildItem -LiteralPath $local2 -Force | Where-Object { $_.Name -ne 'Microsoft' }).Count -eq 0) 'portable copy did not create application data under LOCALAPPDATA'
} catch {
  Write-Error -ErrorRecord $_ -ErrorAction Continue
  throw
} finally {
  Invoke-TestCleanup { Stop-TestApp $p $base }
  Invoke-TestCleanup { Assert-Test ((Get-ListenerCount $port) -eq 0) 'debug listener closed' }
  Invoke-TestCleanup { Assert-Test (-not [bool]$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS) 'debug environment is clear' }
  Invoke-TestCleanup { Assert-Test ($env:LOCALAPPDATA -eq $realLocal) 'parent LOCALAPPDATA restored' }
  Invoke-TestCleanup { Assert-Test (@(Get-NotayaProcesses | Where-Object { $_.Path -and $_.Path.StartsWith($base.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase) }).Count -eq 0) 'owned Notaya processes exited' }
  Invoke-TestCleanup { Confirm-TestArtifactsRetained $base }
  Complete-TestCleanup
}
