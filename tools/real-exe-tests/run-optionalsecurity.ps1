<#
  run-optionalsecurity.ps1 - real-exe test of "notes first, security optional" (amendment A21).

  Purpose : starts a COPY of Notaya.exe in a fresh temp folder with NO vault, so the first run opens
            straight to notes, then goes through five phases: security off (nothing locks), turn security on
            (earlier open backups deleted), lock and unlock, turn it off again, turn it on keeping backups,
            restart on the lock screen, turn it off, restart straight to the notes. Between phases the app is
            closed with its window close button (the real quit path) and started again.
            Prints PASS / FAIL lines.
  Usage   : powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-optionalsecurity.ps1 `
              -Exe E:\path\to\Notaya.exe
  Needs   : Node 24 (global fetch and WebSocket, node:sqlite), Windows PowerShell 5.1, WebView2.
  Safety  : never touches a real vault. Everything lives in %TEMP%\clf-optsec-*; the passphrase is generated
            fresh for each run, kept in a file retained inside the owned run home (never printed). The debug port
            9335 is set for the child process only. The finally block stops only processes this script
            started, then reports: port closed, env var not set, run home retained.
#>
param([string]$Exe)

. (Join-Path $PSScriptRoot 'common.ps1')
$ErrorActionPreference = 'Stop'
$port = 9335
$Exe = Resolve-TestExe $Exe
Assert-PortFree $port

$dir = New-TestRunFolder $env:TEMP 'clf-optsec'
Copy-TestExe $Exe $dir
"exe sha256: " + (Get-FileHash "$dir\Notaya.exe" -Algorithm SHA256).Hash
$pw = New-PassphraseFile $dir
$p = $null
try {
  foreach ($phase in 'p1','p2','p3','p4','p5') {
    $p = Start-TestApp "$dir\Notaya.exe" $port
    Start-Sleep 6
    Invoke-NodeCheck "$PSScriptRoot\optionalsecurity.mjs" @($port, $pw, "$dir\data", $phase)
    # Close like a person would: the window's close button, which saves, backs up and quits.
    $null = $p.CloseMainWindow()
    Assert-Test ($p.WaitForExit(40000)) "$phase app exited within the existing 40-second limit"
    Start-Sleep 2
    Assert-Test ((Get-ListenerCount $port) -eq 0) "$phase closed its debug listener"
  }
  "--- data folder after the test:"
  Get-ChildItem "$dir\data" -File | Select-Object Name,Length | Format-Table -AutoSize | Out-String
  "--- backups:"
  Get-ChildItem "$dir\data\backups" -File | Select-Object Name,Length | Format-Table -AutoSize | Out-String
  'All five optional-security phases completed without a failed child or exit check.'
} catch {
  Write-Error -ErrorRecord $_ -ErrorAction Continue
  throw
} finally {
  Invoke-TestCleanup { Stop-TestApp $p $dir }
  Invoke-TestCleanup { Assert-Test ((Get-ListenerCount $port) -eq 0) 'debug listener closed' }
  Invoke-TestCleanup { Assert-Test (-not [bool]$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS) 'debug environment is clear' }
  Invoke-TestCleanup { Assert-Test (@(Get-NotayaProcesses | Where-Object { $_.Path -and $_.Path.StartsWith($dir.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase) }).Count -eq 0) 'owned Notaya processes exited' }
    Invoke-TestCleanup { Confirm-TestArtifactsRetained $dir }
  Invoke-TestCleanup { Assert-Test (Test-Path -LiteralPath $pw -PathType Leaf) 'synthetic credential retained inside the owned run home' }
  Complete-TestCleanup
}
