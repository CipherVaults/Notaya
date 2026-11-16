<#
  run-tray.ps1 - real-exe test of the V1.4.0 tray: close hides Notaya instead of quitting, and a second start brings it back.

  Purpose : starts a COPY of Notaya.exe with "close to tray" ON on a throwaway vault, closes the main window at once after
            typing, and checks: the process keeps running, no visible window is left, a vault with a master password is LOCKED,
            a vault with security off stays open, nothing typed is lost, and starting the exe again shows the window of the
            running copy (the single-instance path the tray's "Open Notaya" shares). Then it turns "Start with Windows" on and
            off through the app's own commands and reads the per-user Run value.
  Usage   : powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-tray.ps1 -Exe <path to Notaya.exe>
  Needs   : Node 24, Windows PowerShell 5.1, WebView2.
  Safety  : throwaway vaults in a temp folder (set -Root to put it elsewhere, for example the AV-excluded test folder), passphrase in a
            temp file deleted at the end. Debug port 9337 for the child process only. It REFUSES to run if a Run value "Notaya"
            already exists (a real "Start with Windows" entry of the owner); it removes the value it wrote, and only if it
            names the test exe. The tray icon and its menu are not clicked (not automated): look at them by hand.
#>
param(
  [string]$Exe = (Join-Path $PSScriptRoot '..\..\src-tauri\target\release\Notaya.exe'),
  [string]$Root = $env:TEMP
)

. (Join-Path $PSScriptRoot 'common.ps1')
$port = 9337
$Exe = Resolve-TestExe $Exe
Assert-PortFree $port
$runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
if (Get-ItemProperty -Path $runKey -Name 'Notaya' -ErrorAction SilentlyContinue) { throw 'A Run value "Notaya" already exists (start with Windows is on for a real copy). Refusing to run.' }

function Get-VisibleWindowCount($p) { $p.Refresh(); if ($p.HasExited) { return -1 }; return [int]($p.MainWindowHandle -ne [IntPtr]::Zero) }

$pw = New-PassphraseFile
$dirs = @()
$p = $null
try {
  foreach ($mode in @('secure', 'open')) {
    $dir = Join-Path $Root "clf-tray-$mode-$(Get-Random)"
    $dirs += $dir
    New-Item -ItemType Directory $dir | Out-Null
    Copy-TestExe $Exe $dir $true
    "exe sha256: " + (Get-FileHash "$dir\Notaya.exe" -Algorithm SHA256).Hash
    "== $mode : setup"
    $p = Start-TestApp "$dir\Notaya.exe" $port; Start-Sleep 6
    node "$PSScriptRoot\tray.mjs" $port setup $mode $pw
    "== $mode : type, then close the window"
    node "$PSScriptRoot\tray.mjs" $port close $mode $pw
    $p.Refresh()
    "app still running after the close button: " + (-not $p.HasExited)
    "visible main window after the close button (0 = hidden): " + (Get-VisibleWindowCount $p)
    node "$PSScriptRoot\tray.mjs" $port status $mode $pw
    "== $mode : start the exe again (brings the window back)"
    $null = Start-Process "$dir\Notaya.exe" -PassThru; Start-Sleep 4
    "visible main window after the second start (1 = shown): " + (Get-VisibleWindowCount $p)
    "Notaya processes from this folder: " + @(Get-Process Notaya -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($dir) }).Count + " (1 = the second start handed over and ended)"
    node "$PSScriptRoot\tray.mjs" $port check $mode $pw
    if ($mode -eq 'open') {
      "== open : start with Windows on, then off"
      node "$PSScriptRoot\tray.mjs" $port prefs on $pw
      $v = (Get-ItemProperty -Path $runKey -Name 'Notaya' -ErrorAction SilentlyContinue).Notaya
      "Run value while on: $v"
      node "$PSScriptRoot\tray.mjs" $port prefs off $pw
      "Run value after off: " + [bool](Get-ItemProperty -Path $runKey -Name 'Notaya' -ErrorAction SilentlyContinue)
      "shell-prefs.json: " + (Get-Content "$dir\data\shell-prefs.json" -Raw)
    }
    Stop-TestApp $p $dir
    $p = $null
  }
} finally {
  Stop-TestApp $p ($dirs | Select-Object -Last 1)
  foreach ($d in $dirs) { Get-Process Notaya -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($d) } | Stop-Process -Force }
  $leftover = (Get-ItemProperty -Path $runKey -Name 'Notaya' -ErrorAction SilentlyContinue).Notaya
  if ($leftover) { foreach ($d in $dirs) { if ($leftover.Contains($d)) { Remove-ItemProperty -Path $runKey -Name 'Notaya'; 'removed the test Run value' } } }
  "Run value Notaya left behind: " + [bool](Get-ItemProperty -Path $runKey -Name 'Notaya' -ErrorAction SilentlyContinue)
  "port $port listening after test: " + (Get-ListenerCount $port)
  "WEBVIEW2 env var left set: " + [bool]$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Remove-Item $pw -Force -ErrorAction SilentlyContinue
  foreach ($d in $dirs) { "temp folder removed: " + (Remove-TestFolder $d) }
  "passphrase file removed: " + (-not (Test-Path $pw))
}
