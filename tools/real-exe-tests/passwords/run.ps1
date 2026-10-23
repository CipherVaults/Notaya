<#
  run.ps1 - generic runner for the passwords-file backup/restore and Quit real-exe tests.

  Purpose : starts a COPY of Cipherleaf.exe (and, with -WithKP, a COPY of the KeePassXC folder) in a fresh temp
            folder with a throwaway vault, runs one test script from this folder against it through the
            WebView2 debug port, then cleans up. The test prints PASS / FAIL lines and a SUMMARY.
  Usage   : from the project root, in Windows PowerShell 5.1:
              powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\passwords\run.ps1 `
                -Script testA.mjs -WithKP -KeePassXCDir E:\Project\Cipherleaf-portable\KeePassXC
            -Script        testA.mjs | testB.mjs | testC.mjs | testD1.mjs | testE.mjs
            -Exe           path to Cipherleaf.exe; default ..\..\..\src-tauri\target\release\Cipherleaf.exe
                           relative to this script (release build from "npm run tauri build")
            -WithKP        copy the KeePassXC folder into the temp folder (needed by testC and testE)
            -KeePassXCDir  the KeePassXC folder to copy FROM (read only), e.g. the one in the portable copy
            -Extra         extra argument for the script: testC needs c1, c2 or c2q
            -Err           also capture the app's stdout/stderr and print stderr
  Needs   : Node 24, Windows PowerShell 5.1, WebView2; for -WithKP a KeePassXC folder.
  Safety  : never run against a real vault or the portable data folder. Everything lives in %TEMP%\clf-pw-*.
            The passphrase is generated fresh for each run, kept in a temp file and deleted at the end (never
            printed). The debug port 9335 is set for the child process only. Only the Cipherleaf this script
            started is ever stopped. KeePassXC is NEVER force-stopped: the tests close it politely, and if it
            is still running afterwards the temp folder is kept and reported.
#>
param(
  [Parameter(Mandatory = $true)][string]$Script,
  [string]$Exe = (Join-Path $PSScriptRoot '..\..\..\src-tauri\target\release\Cipherleaf.exe'),
  [switch]$WithKP,
  [string]$KeePassXCDir = '',
  [string]$Extra = '',
  [switch]$Err
)

. (Join-Path $PSScriptRoot '..\common.ps1')
$port = 9335
$Exe = Resolve-TestExe $Exe
$scriptPath = Join-Path $PSScriptRoot $Script
if (-not (Test-Path -LiteralPath $scriptPath)) { throw "Test script not found: $scriptPath" }
if ($WithKP -and (-not $KeePassXCDir -or -not (Test-Path -LiteralPath (Join-Path $KeePassXCDir 'KeePassXC.exe')))) {
  throw 'With -WithKP pass -KeePassXCDir <folder that contains KeePassXC.exe> (it is only copied, never changed).'
}
Assert-PortFree $port

$dir = Join-Path $env:TEMP "clf-pw-$(Get-Random)"
New-Item -ItemType Directory $dir | Out-Null
Copy-Item $Exe "$dir\Cipherleaf.exe"
if ($WithKP) { Copy-Item $KeePassXCDir "$dir\KeePassXC" -Recurse }
"exe sha256: " + (Get-FileHash "$dir\Cipherleaf.exe" -Algorithm SHA256).Hash
if ($WithKP) { "KeePassXC.exe sha256: " + (Get-FileHash "$dir\KeePassXC\KeePassXC.exe" -Algorithm SHA256).Hash.Substring(0,8) }
$pw = New-PassphraseFile
$kpBase = @(Get-Process KeePassXC -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })   # e.g. the owner's own KeePassXC: never touched
$outf = Join-Path $env:TEMP ("clf-pw-out-" + [guid]::NewGuid().ToString('N') + ".txt")
$errf = Join-Path $env:TEMP ("clf-pw-err-" + [guid]::NewGuid().ToString('N') + ".txt")
$p = $null
try {
  if ($Err) { $p = Start-TestApp "$dir\Cipherleaf.exe" $port $outf $errf } else { $p = Start-TestApp "$dir\Cipherleaf.exe" $port }
  "cipherleaf pid: $($p.Id)"
  Start-Sleep 6
  node $scriptPath $port $pw $dir $p.Id $Extra
  if ($Err -and (Test-Path $errf)) { "--- stderr:"; Get-Content $errf }
  "--- data folder after the test:"
  if (Test-Path "$dir\data") { Get-ChildItem "$dir\data" -File | Select-Object Name,Length | Format-Table -AutoSize | Out-String }
} finally {
  if ($p) { $p.Refresh() }
  if ($p -and -not $p.HasExited) { $null = $p.CloseMainWindow(); Start-Sleep 5; $p.Refresh() }
  if ($p -and -not $p.HasExited) { "cipherleaf still running, stopping my own process"; Stop-Process -Id $p.Id -Force }
  Start-Sleep 2
  # KeePassXC of my temp folder: report only (never force-stop)
  $kpr = @(Get-Process KeePassXC -ErrorAction SilentlyContinue | Where-Object { $kpBase -notcontains $_.Id })
  "KeePassXC from temp folder still running: " + $kpr.Count
  "port $port listening after test: " + (Get-ListenerCount $port)
  "WEBVIEW2 env var left set: " + [bool]$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  "Cipherleaf processes of mine remaining: " + @(Get-Process Cipherleaf -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($dir) }).Count
  Remove-Item $pw, $outf, $errf -Force -ErrorAction SilentlyContinue
  "passphrase file removed: " + (-not (Test-Path $pw))
  if ($kpr.Count -eq 0) {
    "temp folder removed: " + (Remove-TestFolder $dir)
  } else { "temp folder KEPT (KeePassXC still running, close it politely yourself, then delete): $dir" }
}
