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
            file retained inside the owned run home. Debug port 9337 for the child process only. It REFUSES to run if a Run value "Notaya"
            already exists (a real "Start with Windows" entry of the owner); it removes the value it wrote, and only if it
            names the test exe. The tray icon and its menu are not clicked (not automated): look at them by hand.
#>
param(
  [string]$Exe,
  [string]$Root = $env:TEMP
)

. (Join-Path $PSScriptRoot 'common.ps1')
$ErrorActionPreference = 'Stop'
$port = 9337
$Exe = Resolve-TestExe $Exe
Assert-PortFree $port
. (Join-Path $PSScriptRoot 'startup-entry-guard.ps1')
Assert-NotayaStartupAbsent
$startupAttemptedOn = $false
$startupTestExe = $null

function Get-VisibleWindowCount($p) {
  $p.Refresh()
  $exited = $p.HasExited
  if ($exited -isnot [bool]) { throw 'Cannot establish whether the owned test process has exited' }
  if ($exited) { return -1 }
  $visible = @(Invoke-NodeCheck (Join-Path $PSScriptRoot 'main-window-visible.mjs') @($port))
  if ($visible.Count -ne 1 -or $visible[0] -isnot [string] -or $visible[0] -cnotin @('true','false')) {
    throw 'Expected exactly one boolean from the labelled main window visibility reader'
  }
  return [int]($visible[0] -ceq 'true')
}

$runHome = New-TestRunFolder $Root 'clf-tray'
$pw = New-PassphraseFile $runHome
$dirs = @()
$p = $null
try {
  foreach ($mode in @('secure', 'open')) {
    $dir = Join-Path $runHome $mode
    $dirs += $dir
    New-Item -ItemType Directory $dir | Out-Null
    Register-TestFolder $dir
    Copy-TestExe $Exe $dir $true
    "exe sha256: " + (Get-FileHash "$dir\Notaya.exe" -Algorithm SHA256).Hash
    "== $mode : setup"
    $p = Start-TestApp "$dir\Notaya.exe" $port; Start-Sleep 6
    Invoke-NodeCheck "$PSScriptRoot\tray.mjs" @($port, 'setup', $mode, $pw)
    "== $mode : type, then close the window"
    Invoke-NodeCheck "$PSScriptRoot\tray.mjs" @($port, 'close', $mode, $pw)
    $p.Refresh()
    Assert-Test (-not $p.HasExited) 'close-to-tray keeps the test process running'
    Assert-Test ((Get-VisibleWindowCount $p) -eq 0) 'close-to-tray hides the main window'
    Invoke-NodeCheck "$PSScriptRoot\tray.mjs" @($port, 'status', $mode, $pw)
    "== $mode : start the exe again (brings the window back)"
    $second = Start-TestApp "$dir\Notaya.exe" $port; Start-Sleep 4
    Assert-Test ((Get-VisibleWindowCount $p) -eq 1) 'second start shows the existing main window'
    Assert-Test (@(Get-NotayaProcesses | Where-Object { $_.Path -and $_.Path.StartsWith($dir.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase) }).Count -eq 1) 'second start handed over and exited'
    Invoke-NodeCheck "$PSScriptRoot\tray.mjs" @($port, 'check', $mode, $pw)
    if ($mode -eq 'open') {
      "== open : start with Windows on, then off"
      $startupTestExe = "$dir\Notaya.exe"
      [void](Get-NotayaStartupCommand $startupTestExe)
      Assert-NotayaStartupAbsent
      $startupAttemptedOn = $true
      [Console]::Out.WriteLine('STARTUP_ENTRY_TEST_ON ' + (@{ testExe = $startupTestExe } | ConvertTo-Json -Compress))
      [Console]::Out.Flush()
      Invoke-NodeCheck "$PSScriptRoot\tray.mjs" @($port, 'prefs', 'on', $pw)
      Assert-NotayaStartupOwned $startupTestExe
      Assert-Test $true 'Start with Windows points only to the test executable'
      Assert-NotayaStartupOwned $startupTestExe
      Invoke-NodeCheck "$PSScriptRoot\tray.mjs" @($port, 'prefs', 'off', $pw)
      Assert-NotayaStartupAbsent
      Assert-Test $true 'Start with Windows entry was removed'
      "shell-prefs.json: " + (Get-Content "$dir\data\shell-prefs.json" -Raw)
    }
    Stop-TestApp $p $dir
    $p = $null
  }
} catch {
  Write-Error -ErrorRecord $_ -ErrorAction Continue
  throw
} finally {
  Invoke-TestCleanup { Stop-TestApp $p ($dirs | Select-Object -Last 1) }
  Invoke-TestCleanup { foreach ($d in $dirs) { Stop-TestApp $null $d } }
  Invoke-TestCleanup { Remove-NotayaTestStartupEntry $startupTestExe $startupAttemptedOn }
  Invoke-TestCleanup { Assert-NotayaStartupAbsent; Assert-Test $true 'no test Start-with-Windows value remains' }
  Invoke-TestCleanup { Assert-Test ((Get-ListenerCount $port) -eq 0) 'debug listener closed' }
  Invoke-TestCleanup { Assert-Test (-not [bool]$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS) 'debug environment is clear' }
    Invoke-TestCleanup { Confirm-TestArtifactsRetained $runHome }
  Invoke-TestCleanup { Assert-Test (Test-Path -LiteralPath $pw -PathType Leaf) 'synthetic credential retained inside the owned run home' }
  Complete-TestCleanup
}
