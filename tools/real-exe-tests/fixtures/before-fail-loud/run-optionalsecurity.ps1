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
            fresh for each run, kept in a temp file and deleted at the end (never printed). The debug port
            9335 is set for the child process only. The finally block stops only processes this script
            started, then reports: port closed, env var not set, folder removed.
#>
param([string]$Exe = (Join-Path $PSScriptRoot '..\..\src-tauri\target\release\Notaya.exe'))

. (Join-Path $PSScriptRoot 'common.ps1')
$port = 9335
$Exe = Resolve-TestExe $Exe
Assert-PortFree $port

$dir = Join-Path $env:TEMP "clf-optsec-$(Get-Random)"
New-Item -ItemType Directory $dir | Out-Null
Copy-TestExe $Exe $dir
"exe sha256: " + (Get-FileHash "$dir\Notaya.exe" -Algorithm SHA256).Hash
$pw = New-PassphraseFile
$p = $null
$failed = @()
try {
  foreach ($phase in 'p1','p2','p3','p4','p5') {
    $p = Start-TestApp "$dir\Notaya.exe" $port
    Start-Sleep 6
    node "$PSScriptRoot\optionalsecurity.mjs" $port $pw "$dir\data" $phase
    if ($LASTEXITCODE -ne 0) { $failed += $phase }
    # Close like a person would: the window's close button, which saves, backs up and quits.
    $null = $p.CloseMainWindow()
    if (-not $p.WaitForExit(40000)) { "$phase : the app did not exit within 40 s after the close button"; $failed += "$phase-exit" }
    Start-Sleep 2
    if ((Get-ListenerCount $port) -gt 0 -and -not $p.HasExited) { break }
  }
  "--- data folder after the test:"
  Get-ChildItem "$dir\data" -File | Select-Object Name,Length | Format-Table -AutoSize | Out-String
  "--- backups:"
  Get-ChildItem "$dir\data\backups" -File | Select-Object Name,Length | Format-Table -AutoSize | Out-String
  "phases with failures: " + ($(if ($failed.Count) { $failed -join ', ' } else { 'none' }))
} finally {
  Stop-TestApp $p $dir
  "port $port listening after test: " + (Get-ListenerCount $port)
  "WEBVIEW2 env var left set: " + [bool]$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  "Notaya processes from the temp folder remaining: " + @(Get-Process Notaya -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($dir) }).Count
  Remove-Item $pw -Force -ErrorAction SilentlyContinue
  "temp folder removed: " + (Remove-TestFolder $dir)
  "passphrase file removed: " + (-not (Test-Path $pw))
}
