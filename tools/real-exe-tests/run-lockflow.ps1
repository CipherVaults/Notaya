<#
  run-lockflow.ps1 - real-exe test that unsaved edits survive a lock or a window close.

  Purpose : starts a COPY of Notaya.exe in a fresh temp folder with a throwaway vault, types text into
            a page (scenarios page and close) or a sticky note (scenario sticky), locks or closes the
            window at once (inside the save delay), unlocks and prints whether the text was saved.
  Usage   : powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-lockflow.ps1 `
              -Exe E:\Project\Notaya\Notaya\src-tauri\target\release\Notaya.exe -Scenarios page,sticky,close
            -Exe     optional; default ..\..\src-tauri\target\release\Notaya.exe relative to this script.
            -Scenarios  any of page, sticky, close (default: all three).  -Label  a name for the temp folder.
  Needs   : Node 24 (global fetch and WebSocket), Windows PowerShell 5.1, WebView2.
  Safety  : never touches a real vault. Throwaway vault in %TEMP%\clf-lockflow-*; the passphrase is generated
            fresh for each run in a temp file that is deleted at the end (never printed). Debug port 9335 is
            set for the child process only. The finally block stops only processes this script started,
            then reports: port closed, env var not set, folder removed.
#>
param(
  [string]$Exe = (Join-Path $PSScriptRoot '..\..\src-tauri\target\release\Notaya.exe'),
  [string]$Label = 'run',
  [string[]]$Scenarios = @('page', 'sticky', 'close')
)

. (Join-Path $PSScriptRoot 'common.ps1')
$Scenarios = @($Scenarios | ForEach-Object { $_ -split ',' } | Where-Object { $_ })   # "-File ... -Scenarios a,b" arrives as one string
$port = 9335
$Exe = Resolve-TestExe $Exe
Assert-PortFree $port

$dir = Join-Path $env:TEMP "clf-lockflow-$Label-$(Get-Random)"
New-Item -ItemType Directory $dir | Out-Null
Copy-TestExe $Exe $dir
"exe sha256: " + (Get-FileHash "$dir\Notaya.exe" -Algorithm SHA256).Hash
$pw = New-PassphraseFile
$p = $null

function Start-App {
  $app = Start-TestApp "$dir\Notaya.exe" $port
  Start-Sleep 6
  return $app
}

try {
  "== $Label : setup"
  $p = Start-App
  node "$PSScriptRoot\lockflow.mjs" $port setup page $pw
  foreach ($s in $Scenarios) {
    "== $Label : scenario $s"
    if ($s -eq "close") {
      node "$PSScriptRoot\lockflow.mjs" $port type close $pw
      Start-Sleep 3
      $p.Refresh()
      "app still running after close request: " + (-not $p.HasExited)
      Stop-TestApp $p $dir
      $p = Start-App
    } else {
      node "$PSScriptRoot\lockflow.mjs" $port type $s $pw
    }
    node "$PSScriptRoot\lockflow.mjs" $port check $s $pw
  }
} finally {
  Stop-TestApp $p $dir
  "port $port listening after test: " + (Get-ListenerCount $port)
  "WEBVIEW2 env var left set: " + [bool]$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  "Notaya processes from the temp folder remaining: " + @(Get-Process Notaya -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($dir) }).Count
  Remove-Item $pw -Force -ErrorAction SilentlyContinue
  "temp folder removed: " + (Remove-TestFolder $dir)
  "passphrase file removed: " + (-not (Test-Path $pw))
}
