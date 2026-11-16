<#
  run-restoreflow.ps1 - real-exe test of "restore the vault from a backup".

  Purpose : starts a COPY of Notaya.exe in a fresh temp folder with a throwaway vault, makes two
            backups with different text, then checks restore_vault (locked and unlocked, refusals for
            bad paths and a wrong password) through the WebView2 debug port. Prints PASS / FAIL lines.
  Usage   : powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-restoreflow.ps1 `
              -Exe E:\Project\Notaya\Notaya\src-tauri\target\release\Notaya.exe
            -Exe is required; pass an independently verified build explicitly.
  Needs   : Node 24 (global fetch and WebSocket), Windows PowerShell 5.1, WebView2.
  Safety  : never touches a real vault. The vault lives in %TEMP%\clf-restoreflow-*; the passphrase is
            generated fresh for each run, kept in a file retained inside the owned run home (never printed).
            The debug port 9335 is set for the child process only. The finally block stops only the
            process this script started, then reports: port closed, env var not set, run home retained.
#>
param([string]$Exe)

. (Join-Path $PSScriptRoot 'common.ps1')
$ErrorActionPreference = 'Stop'
$port = 9335
$Exe = Resolve-TestExe $Exe
Assert-PortFree $port

$dir = New-TestRunFolder $env:TEMP 'clf-restoreflow'
Copy-TestExe $Exe $dir
"exe sha256: " + (Get-FileHash "$dir\Notaya.exe" -Algorithm SHA256).Hash
$pw = New-PassphraseFile $dir
$p = $null
try {
  $p = Start-TestApp "$dir\Notaya.exe" $port
  Start-Sleep 6
  Invoke-NodeCheck "$PSScriptRoot\restoreflow.mjs" @($port, $pw)
  "--- data folder after the test:"
  Get-ChildItem "$dir\data" -File | Select-Object Name,Length | Format-Table -AutoSize | Out-String
  "--- backups:"
  Get-ChildItem "$dir\data\backups" -File | Select-Object Name,Length | Format-Table -AutoSize | Out-String
  Assert-Test (-not (Test-Path -LiteralPath "$dir\data\restore-check.tmp")) 'restore staging file was removed'
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
