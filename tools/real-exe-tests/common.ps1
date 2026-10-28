# common.ps1 - helpers shared by the real-exe test runners (dot-sourced, not run on its own).
#
# Purpose : one place for the safety-relevant helpers: a fresh random throwaway passphrase,
#           a check that the debug port is free, and "stop only the Notaya I started".
# Usage   : . (Join-Path $PSScriptRoot 'common.ps1')          # from run-*.ps1 in this folder
# Needs   : Windows PowerShell 5.1. Nothing is installed or changed outside $env:TEMP.

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

# Generates a passphrase, writes it to a new file under $env:TEMP and returns the file path.
# The caller must delete the file in its finally block.
function New-PassphraseFile {
  $path = Join-Path $env:TEMP ("clf-pass-" + [guid]::NewGuid().ToString('N') + ".txt")
  [System.IO.File]::WriteAllText($path, (New-ThrowawayPassphrase))
  return $path
}

# Resolves and checks the path to the Notaya.exe that will be tested.
function Resolve-TestExe([string]$Exe) {
  if (-not $Exe) { throw 'Pass -Exe <path to Notaya.exe> (a release build from npm run tauri build).' }
  if (-not (Test-Path -LiteralPath $Exe -PathType Leaf)) { throw "Notaya.exe not found: $Exe" }
  return (Resolve-Path -LiteralPath $Exe).Path
}

function Get-ListenerCount([int]$Port) {
  @(Get-NetTCPConnection -State Listen -LocalPort $Port -ErrorAction SilentlyContinue).Count
}

# Refuse to start if anything already listens on the debug port.
function Assert-PortFree([int]$Port) {
  if ((Get-ListenerCount $Port) -gt 0) { throw "Something already listens on port $Port. Refusing to start; find out what it is first." }
}

# Starts the copied exe with the WebView2 debug port set ONLY for that child process.
function Start-TestApp([string]$ExePath, [int]$Port, [string]$StdoutFile = '', [string]$StderrFile = '') {
  try {
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$Port"
    if ($StdoutFile -and $StderrFile) { $p = Start-Process $ExePath -PassThru -RedirectStandardOutput $StdoutFile -RedirectStandardError $StderrFile }
    else { $p = Start-Process $ExePath -PassThru }
  } finally {
    Remove-Item Env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ErrorAction SilentlyContinue
  }
  return $p
}

# Closes the test app politely; force-stops ONLY the process this script started (never another
# Notaya, such as the owner's real window), plus any leftover process running from $Dir.
function Stop-TestApp($p, [string]$Dir) {
  if ($p) {
    $p.Refresh()
    if (-not $p.HasExited) { $null = $p.CloseMainWindow(); Start-Sleep 3; $p.Refresh() }
    if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force }
  }
  if ($Dir) {
    Get-Process Notaya -ErrorAction SilentlyContinue |
      Where-Object { $_.Path -and $_.Path.StartsWith($Dir, [System.StringComparison]::OrdinalIgnoreCase) } |
      Stop-Process -Force
  }
  Start-Sleep 2
}

function Remove-TestFolder([string]$Dir) {
  for ($i = 0; $i -lt 6; $i++) { try { Remove-Item -LiteralPath $Dir -Recurse -Force -ErrorAction Stop; break } catch { Start-Sleep 3 } }
  return (-not (Test-Path -LiteralPath $Dir))
}
