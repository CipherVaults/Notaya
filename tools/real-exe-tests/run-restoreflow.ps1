<#
  run-restoreflow.ps1 - real-exe test of "restore the vault from a backup".

  Purpose : starts a COPY of Cipherleaf.exe in a fresh temp folder with a throwaway vault, makes two
            backups with different text, then checks restore_vault (locked and unlocked, refusals for
            bad paths and a wrong password) through the WebView2 debug port. Prints PASS / FAIL lines.
  Usage   : powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-restoreflow.ps1 `
              -Exe E:\Project\cipherleaf\src-tauri\target\release\Cipherleaf.exe
            -Exe is optional; the default is ..\..\src-tauri\target\release\Cipherleaf.exe relative to
            this script (the release build from "npm run tauri build").
  Needs   : Node 24 (global fetch and WebSocket), Windows PowerShell 5.1, WebView2.
  Safety  : never touches a real vault. The vault lives in %TEMP%\clf-restoreflow-*; the passphrase is
            generated fresh for each run, kept in a temp file and deleted at the end (never printed).
            The debug port 9335 is set for the child process only. The finally block stops only the
            process this script started, then reports: port closed, env var not set, folder removed.
#>
param([string]$Exe = (Join-Path $PSScriptRoot '..\..\src-tauri\target\release\Cipherleaf.exe'))

. (Join-Path $PSScriptRoot 'common.ps1')
$port = 9335
$Exe = Resolve-TestExe $Exe
Assert-PortFree $port

$dir = Join-Path $env:TEMP "clf-restoreflow-$(Get-Random)"
New-Item -ItemType Directory $dir | Out-Null
Copy-Item $Exe "$dir\Cipherleaf.exe"
"exe sha256: " + (Get-FileHash "$dir\Cipherleaf.exe" -Algorithm SHA256).Hash
$pw = New-PassphraseFile
$p = $null
try {
  $p = Start-TestApp "$dir\Cipherleaf.exe" $port
  Start-Sleep 6
  node "$PSScriptRoot\restoreflow.mjs" $port $pw
  "--- data folder after the test:"
  Get-ChildItem "$dir\data" -File | Select-Object Name,Length | Format-Table -AutoSize | Out-String
  "--- backups:"
  Get-ChildItem "$dir\data\backups" -File | Select-Object Name,Length | Format-Table -AutoSize | Out-String
  "restore-check.tmp left behind: " + (Test-Path "$dir\data\restore-check.tmp")
} finally {
  Stop-TestApp $p $dir
  "port $port listening after test: " + (Get-ListenerCount $port)
  "WEBVIEW2 env var left set: " + [bool]$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  "Cipherleaf processes from the temp folder remaining: " + @(Get-Process Cipherleaf -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($dir) }).Count
  Remove-Item $pw -Force -ErrorAction SilentlyContinue
  "temp folder removed: " + (Remove-TestFolder $dir)
  "passphrase file removed: " + (-not (Test-Path $pw))
}
