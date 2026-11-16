<#
  run-lockflow.ps1 - real-exe test that unsaved edits survive a lock or a window close.

  Purpose : starts a COPY of Notaya.exe in a fresh temp folder with a throwaway vault, types text into
            a page (scenarios page and close) or a sticky note (scenario sticky), locks or closes the
            window at once (inside the save delay), unlocks and prints whether the text was saved.
  Usage   : powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-lockflow.ps1 `
              -Exe E:\Project\Notaya\Notaya\src-tauri\target\release\Notaya.exe -Scenarios page,sticky,close
            -Exe is required; pass an independently verified build explicitly.
            -Scenarios  any of page, sticky, close (default: all three).  -Label  a name for the temp folder.
  Needs   : Node 24 (global fetch and WebSocket), Windows PowerShell 5.1, WebView2.
  Safety  : never touches a real vault. Throwaway vault in %TEMP%\clf-lockflow-*; the passphrase is generated
            fresh for each run in a file in the retained owned run home (never printed). Debug port 9335 is
            set for the child process only. The finally block stops only processes this script started,
            then reports: port closed, env var not set, run home retained.
#>
param(
  [string]$Exe,
  [string]$Label = 'run',
  [string[]]$Scenarios = @('page', 'sticky', 'close')
)

. (Join-Path $PSScriptRoot 'common.ps1')
$ErrorActionPreference = 'Stop'
$Scenarios = @($Scenarios | ForEach-Object { $_ -split ',' } | Where-Object { $_ })   # "-File ... -Scenarios a,b" arrives as one string
Assert-Test ($Scenarios.Count -gt 0) 'at least one lock scenario selected'
Assert-Test (@($Scenarios | Where-Object { $_ -notin 'page','sticky','close' }).Count -eq 0) 'only known lock scenarios selected'
$port = 9335
$Exe = Resolve-TestExe $Exe
Assert-PortFree $port

$dir = New-TestRunFolder $env:TEMP ("clf-lockflow-" + $Label)
Copy-TestExe $Exe $dir
"exe sha256: " + (Get-FileHash "$dir\Notaya.exe" -Algorithm SHA256).Hash
$pw = New-PassphraseFile $dir
$p = $null

function Start-App {
  $app = Start-TestApp "$dir\Notaya.exe" $port
  Start-Sleep 6
  return $app
}

try {
  "== $Label : setup"
  $p = Start-App
  Invoke-NodeCheck "$PSScriptRoot\lockflow.mjs" @($port, 'setup', 'page', $pw)
  foreach ($s in $Scenarios) {
    "== $Label : scenario $s"
    if ($s -eq "close") {
      Invoke-NodeCheck "$PSScriptRoot\lockflow.mjs" @($port, 'type', 'close', $pw)
      Start-Sleep 3
      $p.Refresh()
      Assert-Test $p.HasExited 'close request exited the test app'
      Stop-TestApp $p $dir
      $p = Start-App
    } else {
      Invoke-NodeCheck "$PSScriptRoot\lockflow.mjs" @($port, 'type', $s, $pw)
    }
    Invoke-NodeCheck "$PSScriptRoot\lockflow.mjs" @($port, 'check', $s, $pw)
  }
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
