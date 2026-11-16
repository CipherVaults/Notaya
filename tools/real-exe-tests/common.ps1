# common.ps1 - helpers shared by the real-exe test runners (dot-sourced, not run on its own).
#
# Purpose : one place for the safety-relevant helpers: a fresh random throwaway passphrase,
#           a check that the debug port is free, and "stop only the Notaya I started".
# Usage   : . (Join-Path $PSScriptRoot 'common.ps1')          # from run-*.ps1 in this folder
# Needs   : Windows PowerShell 5.1. Owned throwaway run homes are retained for diagnosis.

$script:OwnedTestFolders = @()
$script:CleanupFailures = New-Object System.Collections.Generic.List[string]

$script:WordList = @(
  'anchor','birch','cobalt','dune','ember','fjord','garnet','harbor','ivory','juniper','kestrel','lantern',
  'meadow','nectar','onyx','pebble','quartz','raven','saffron','thistle','umber','velvet','willow','yarrow',
  'zephyr','amber','basalt','cedar','delta','falcon','glacier','hazel','indigo','jasper','kelp','lagoon',
  'maple','nimbus','orchid','prairie','quill','ridge','sparrow','tundra','upland','violet','walnut','xenon',
  'yonder','zenith','alder','bramble','cinder','drift','estuary','fennel','granite','heron','iris','jade',
  'kindle','lotus','marble','nutmeg'
)

# Returns a fresh random passphrase: 5 different words (Title-case), 4 random digits, joined by hyphens,
# e.g. "Word-Word-Word-Word-Word-1234" (the words come from a built-in list). About 43 bits of randomness from a cryptographic RNG;
# long enough for the app's rule (at least 10 characters, zxcvbn score 3). Never print it.
function New-ThrowawayPassphrase {
  if ($script:WordList.Count -ne 64) { throw 'word list must have exactly 64 entries' }
  $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
  try {
    $chosen = New-Object System.Collections.Generic.List[string]
    $buf = New-Object byte[] 1
    while ($chosen.Count -lt 5) {
      $rng.GetBytes($buf)
      $w = $script:WordList[$buf[0] -band 63]            # 256 is a multiple of 64: no bias
      if (-not $chosen.Contains($w)) { $chosen.Add($w.Substring(0,1).ToUpper() + $w.Substring(1)) }
    }
    $digits = ''
    while ($digits.Length -lt 4) {
      $rng.GetBytes($buf)
      if ($buf[0] -lt 250) { $digits += [string]($buf[0] % 10) }   # reject 250..255: no bias
    }
    return (($chosen -join '-') + '-' + $digits)
  } finally { $rng.Dispose() }
}

# Generates a test-only passphrase inside an owned, outside-repository run home.
# The synthetic vault and credential are retained together; never print the value.
function New-PassphraseFile([string]$Dir) {
  Assert-OwnedTestFolder $Dir
  Assert-OutsideRepository $Dir
  $path = Join-Path $Dir ("test-passphrase-" + [guid]::NewGuid().ToString('N') + ".txt")
  $bytes = [Text.Encoding]::UTF8.GetBytes((New-ThrowawayPassphrase))
  $stream = [IO.File]::Open($path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
  try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
  return $path
}

# Resolves and checks the path to the Notaya.exe that will be tested.
function Resolve-TestExe([string]$Exe) {
  if (-not $Exe) { throw 'Pass -Exe <path to Notaya.exe> (a release build from npm run tauri build).' }
  if (-not (Test-Path -LiteralPath $Exe -PathType Leaf)) { throw "Notaya.exe not found: $Exe" }
  Assert-PlainPath $Exe
  return [IO.Path]::GetFullPath($Exe)
}

# Copies the exe into the fresh test folder as a PORTABLE copy: with the portable.txt marker next to it,
# like the portable ZIP, so the vault is made in "$Dir\data". Without the marker a bare folder counts as an
# INSTALLED copy, which keeps its notes under %LOCALAPPDATA% (see run-installed.ps1). Never leave the marker out
# for a test that is not about installed mode: it would write into the real profile.
#
# Since V1.4.0 the close button hides Notaya to the tray by default. The older scripts close the window to test the
# real quit path, so the test copy starts with "close to tray" OFF (data\shell-prefs.json); run-tray.ps1 turns it on.
function Copy-TestExe([string]$Exe, [string]$Dir, [bool]$CloseToTray = $false) {
  Assert-OwnedTestFolder $Dir
  [IO.File]::Copy($Exe, (Join-Path $Dir 'Notaya.exe'), $false)
  [System.IO.File]::WriteAllText("$Dir\portable.txt", "test copy (portable)")
  New-Item -ItemType Directory -Force -Path "$Dir\data" | Out-Null
  $flag = if ($CloseToTray) { 'true' } else { 'false' }
  [System.IO.File]::WriteAllText("$Dir\data\shell-prefs.json", "{`"closeToTray`":$flag,`"startMinimized`":true}")
}

function Assert-Test($Condition, [string]$Message) {
  if ($Condition -isnot [bool] -or -not $Condition) { throw "FAIL $Message" }
  "PASS $Message"
}

function Assert-PlainPath([string]$Path) {
  $cursor = [IO.Path]::GetFullPath($Path)
  while ($cursor) {
    $entry = Get-Item -LiteralPath $cursor -Force -ErrorAction Stop
    if ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Refusing linked test path: $cursor" }
    $parent = [IO.Path]::GetDirectoryName($cursor)
    if ($parent -eq $cursor) { break }
    $cursor = $parent
  }
}

function Register-TestFolder([string]$Dir) {
  Assert-OutsideRepository $Dir
  Assert-PlainPath $Dir
  $script:OwnedTestFolders += [IO.Path]::GetFullPath($Dir)
}

function Assert-OutsideRepository([string]$Dir) {
  if (-not $Dir) { throw 'An explicit outside-repository run home is required' }
  $full = [IO.Path]::GetFullPath($Dir)
  $repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..')).TrimEnd('\')
  if ($full.Equals($repository, [StringComparison]::OrdinalIgnoreCase) -or
      $full.StartsWith($repository + '\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Throwaway vaults and their credentials must remain outside the repository'
  }
}

function New-TestRunFolder([string]$Root, [string]$Prefix) {
  Assert-OutsideRepository $Root
  Assert-PlainPath $Root
  if ($Prefix -notmatch '^[A-Za-z0-9_-]+$') { throw 'Test folder label contains unsupported characters' }
  $dir = Join-Path ([IO.Path]::GetFullPath($Root)) ($Prefix + '-' + [guid]::NewGuid().ToString('N'))
  if (Test-Path -LiteralPath $dir) { throw 'Refusing to reuse a test run folder' }
  [IO.Directory]::CreateDirectory($dir) | Out-Null
  Register-TestFolder $dir
  return $dir
}

function Assert-OwnedTestFolder([string]$Dir) {
  $full = [IO.Path]::GetFullPath($Dir)
  $owned = @($script:OwnedTestFolders | Where-Object {
    $full.Equals($_, [StringComparison]::OrdinalIgnoreCase) -or
    $full.StartsWith($_.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)
  })
  if (-not $owned.Count) { throw "Refusing unowned test path: $full" }
  Assert-PlainPath $full
}

function Get-ListenerCount([ValidateRange(1,65535)][int]$Port) {
  # Enumerate first, then filter. A failed query is not evidence of a free port.
  @(Get-NetTCPConnection -ErrorAction Stop | Where-Object { $_.State -eq 'Listen' -and $_.LocalPort -eq $Port }).Count
}

function Assert-PortFree([int]$Port) {
  Assert-Test ((Get-ListenerCount $Port) -eq 0) "debug port $Port is free"
}

# All legacy wrappers use this one native-exit pattern. Failure throws through
# the wrapper's finally cleanup; a later successful command cannot erase it.
function Invoke-NodeCheck([string]$ScriptPath, [object[]]$Arguments = @()) {
  $node = (Get-Command node -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
  Assert-PlainPath $node
  Assert-PlainPath $ScriptPath
  $saved = @{}
  try {
    foreach ($name in 'NODE_OPTIONS','NODE_PATH','NODE_COMPILE_CACHE','NODE_DISABLE_COMPILE_CACHE') {
      $saved[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
      [Environment]::SetEnvironmentVariable($name, $null, 'Process')
    }
    $env:NODE_DISABLE_COMPILE_CACHE = '1'
    & $node $ScriptPath @Arguments
    $nativeExit = $LASTEXITCODE
    if ($nativeExit -ne 0) { throw "Node check failed (exit $nativeExit): $ScriptPath" }
  } finally {
    foreach ($name in $saved.Keys) { [Environment]::SetEnvironmentVariable($name, $saved[$name], 'Process') }
  }
}

# A second instance must never hand over to the person's real application.
# Missing process metadata is unknown, not evidence that no copy is running.
function Get-NotayaProcesses {
  $found = @(Get-Process -ErrorAction Stop | Where-Object { $_.ProcessName -in 'Notaya','Cipherleaf' })
  $live = @()
  foreach ($process in $found) {
    $exited = $process.HasExited
    if ($exited -isnot [bool]) { throw 'Cannot establish whether an existing Notaya/Cipherleaf process has exited' }
    if ($exited) { continue }
    if (-not $process.Path) {
      # A process can finish between enumeration and its path lookup.
      $exited = $process.HasExited
      if ($exited -is [bool] -and $exited) { continue }
      throw 'Cannot establish the path of an existing Notaya/Cipherleaf process'
    }
    $live += $process
  }
  return $live
}

function Assert-NoForeignNotaya {
  foreach ($process in @(Get-NotayaProcesses)) {
    $inside = @($script:OwnedTestFolders | Where-Object {
      $process.Path.StartsWith($_.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)
    })
    if (-not $inside.Count) { throw 'Quit existing Notaya/Cipherleaf copies yourself first; this test will not contact or stop them' }
  }
}

# Child data/profile paths are fresh and owned. Inherited Notaya home or WebView
# overrides must not redirect a supposedly throwaway test to somebody's vault.
function Start-TestApp([string]$ExePath, [int]$Port, [string]$StdoutFile = '', [string]$StderrFile = '', [hashtable]$ExtraEnv = @{}) {
  $dir = Split-Path -Parent ([IO.Path]::GetFullPath($ExePath))
  Assert-OwnedTestFolder $dir
  Assert-NoForeignNotaya
  $launchId = [guid]::NewGuid().ToString('N')
  if (-not $StdoutFile) { $StdoutFile = Join-Path $dir ("app-$launchId.stdout.log") }
  if (-not $StderrFile) { $StderrFile = Join-Path $dir ("app-$launchId.stderr.log") }
  foreach ($log in $StdoutFile, $StderrFile) {
    Assert-OwnedTestFolder ([IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($log)))
    if (Test-Path -LiteralPath $log) { throw 'Refusing to overwrite an existing test output log' }
  }
  $profile = Join-Path $dir '.runner-profile'
  $childEnv = @{
    NOTAYA_HOME = $null; CIPHERLEAF_HOME = $null
    WEBVIEW2_USER_DATA_FOLDER = $null
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$Port"
    USERPROFILE = $profile
    HOME = $profile
    APPDATA = (Join-Path $profile 'AppData\Roaming')
    LOCALAPPDATA = (Join-Path $profile 'AppData\Local')
    TEMP = (Join-Path $profile 'AppData\Local\Temp'); TMP = (Join-Path $profile 'AppData\Local\Temp')
  }
  foreach ($name in $ExtraEnv.Keys) {
    if ($name -in 'NOTAYA_HOME','CIPHERLEAF_HOME','WEBVIEW2_USER_DATA_FOLDER','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS') { throw "Test home/debug override is not allowed: $name" }
    $childEnv[$name] = $ExtraEnv[$name]
  }
  foreach ($name in 'USERPROFILE','HOME','APPDATA','LOCALAPPDATA','TEMP','TMP') {
    $profileHome = [IO.Path]::GetFullPath($childEnv[$name])
    $registered = @($script:OwnedTestFolders | Where-Object {
      $profileHome.Equals($_, [StringComparison]::OrdinalIgnoreCase) -or
      $profileHome.StartsWith($_.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)
    })
    if (-not $registered.Count) { throw "Refusing unowned test profile path: $profileHome" }
    $ownedAncestor = $profileHome
    while (-not (Test-Path -LiteralPath $ownedAncestor)) { $ownedAncestor = [IO.Path]::GetDirectoryName($ownedAncestor) }
    Assert-OwnedTestFolder $ownedAncestor
    [IO.Directory]::CreateDirectory($profileHome) | Out-Null
    Assert-OwnedTestFolder $profileHome
  }
  $saved = @{}
  try {
    foreach ($name in $childEnv.Keys) {
      $saved[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
      [Environment]::SetEnvironmentVariable($name, $childEnv[$name], 'Process')
    }
    $p = Start-Process -FilePath $ExePath -WindowStyle Hidden -PassThru -RedirectStandardOutput $StdoutFile -RedirectStandardError $StderrFile
  } finally {
    foreach ($name in $saved.Keys) { [Environment]::SetEnvironmentVariable($name, $saved[$name], 'Process') }
  }
  return $p
}

function Stop-TestApp($p, [string]$Dir) {
  Assert-OwnedTestFolder $Dir
  $prefix = [IO.Path]::GetFullPath($Dir).TrimEnd('\') + '\'
  if ($p) {
    $p.Refresh()
    if (-not $p.HasExited) {
      if (-not $p.Path.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { throw 'Refusing to stop a process outside the test folder' }
      $null = $p.CloseMainWindow(); Start-Sleep 3; $p.Refresh()
    }
    if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force -ErrorAction Stop }
  }
  Get-NotayaProcesses |
    Where-Object { $_.Path -and $_.Path.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase) } |
    Stop-Process -Force -ErrorAction Stop
  Start-Sleep 2
}

function Confirm-TestArtifactsRetained([string]$Dir) {
  Assert-OwnedTestFolder $Dir
  Assert-OutsideRepository $Dir
  Assert-Test (Test-Path -LiteralPath $Dir -PathType Container) 'owned throwaway run home retained'
  'Retained synthetic test artifacts: ' + [IO.Path]::GetFullPath($Dir)
}

function Invoke-TestCleanup([scriptblock]$Action) {
  try { & $Action } catch {
    $script:CleanupFailures.Add($_.Exception.Message)
    Write-Warning $_.Exception.Message
  }
}

function Complete-TestCleanup {
  if ($script:CleanupFailures.Count) { throw ('Cleanup failed: ' + ($script:CleanupFailures -join '; ')) }
}
