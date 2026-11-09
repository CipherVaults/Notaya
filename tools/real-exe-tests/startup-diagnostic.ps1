<#
  Read-only X-1 diagnostic for Mohamed. Does not launch/contact Notaya, open a
  vault, attach a debug port, change the registry, or send anything. Prints JSON.
  Optional -LogFile points ONLY to data\logs\notaya-log.txt (never to a vault).
  Paths, usernames, raw event messages, dumps and arbitrary log text are omitted.
#>
param([string]$LogFile = '')
$ErrorActionPreference = 'Stop'
function Get-CommandShape([string]$Command) {
  if (-not $Command) { return 'not recorded' }
  if ($Command -match '^"[^"\r\n]+\\(?:Notaya|Cipherleaf)\.exe"(?<flag> --minimized)?$') {
    return '"<exe path omitted>"' + $Matches.flag
  }
  return 'unexpected command shape (raw text omitted)'
}
$run = (Get-ItemProperty -LiteralPath 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name Notaya -ErrorAction SilentlyContinue).Notaya
$processes = @(Get-CimInstance Win32_Process -Filter "Name='Notaya.exe' OR Name='Cipherleaf.exe'" -ErrorAction SilentlyContinue)
$live = @($processes | ForEach-Object {
  $version = $null
  if ($_.ExecutablePath) {
    $info = (Get-Item -LiteralPath $_.ExecutablePath -ErrorAction SilentlyContinue).VersionInfo
    if ($info.ProductVersion -match '^\d+(\.\d+){1,3}$') { $version = $info.ProductVersion }
  }
  [ordered]@{ pid = $_.ProcessId; command = Get-CommandShape $_.CommandLine; version = $version;
    matchesRunCommand = [bool]($run -and $_.CommandLine -and [string]::Equals($run.Trim(), $_.CommandLine.Trim(), [StringComparison]::OrdinalIgnoreCase)) }
})
$versions = @()
foreach ($base in @('HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients', 'HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients', 'HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients')) {
  foreach ($key in @(Get-ChildItem -LiteralPath $base -ErrorAction SilentlyContinue)) {
    $value = Get-ItemProperty -LiteralPath $key.PSPath -ErrorAction SilentlyContinue
    if ($value.name -like '*WebView2*' -and $value.pv -match '^\d+(\.\d+){1,3}$') { $versions += $value.pv }
  }
}
if (-not $LogFile) {
  # Infer only the app log location. No directory listing or access to vault files.
  $candidateExe = if ($processes.Count) { $processes[0].ExecutablePath } elseif ($run -match '^"(?<exe>[^"\r\n]+\\Notaya\.exe)"(?: --minimized)?$') { $Matches.exe } else { $null }
  if ($candidateExe -and (Test-Path -LiteralPath (Join-Path (Split-Path -Parent $candidateExe) 'portable.txt'))) {
    $LogFile = Join-Path (Split-Path -Parent $candidateExe) 'data\logs\notaya-log.txt'
  } else { $LogFile = Join-Path $env:LOCALAPPDATA 'Notaya Notes\data\logs\notaya-log.txt' }
}
$logs = @(); $logStatus = 'not found'
if ([IO.Path]::GetFileName($LogFile) -ne 'notaya-log.txt' -or (Split-Path -Leaf (Split-Path -Parent $LogFile)) -ne 'logs') {
  throw 'LogFile must be logs\notaya-log.txt. No other file will be read.'
}
if (Test-Path -LiteralPath $LogFile -PathType Leaf) {
  $logStatus = 'read; only fixed startup/error classes exported'
  foreach ($line in (Get-Content -LiteralPath $LogFile -Tail 120)) {
    # Parse just the fixed prefix; never export any raw message or source path.
    if ($line -match '^(?<time>\d{4}-\d\d-\d\d[T ][0-9:.Z+\-]+)\s+(?<level>INFO|WARN|ERROR|PANIC)\s+v(?<version>[0-9.]+)\s+\[[^\]]*\]\s+(?<body>.*)$') {
      $stamp = $Matches.time; $level = $Matches.level; $version = $Matches.version; $body = $Matches.body
      $kind = switch -Regex ($body) {
        '^Notaya started \(installed\)\.' { 'started-installed'; break }
        '^Notaya started \(portable\)\.' { 'started-portable'; break }
        '^Notaya closed normally\.' { 'clean-exit'; break }
        '^The previous run did not end cleanly' { 'previous-unclean-exit'; break }
        '^Start failed: the data folder' { 'data-folder-failure'; break }
        '^Start failed: the window' { 'webview-creation-failure'; break }
        '^UI error \(error\)' { 'ui-error'; break }
        '^UI error \(unhandledrejection\)' { 'ui-promise-rejection'; break }
        '^UI error \(react\)' { 'react-error'; break }
        '^A command failed' { 'backend-error'; break }
        '^Panic' { 'panic'; break }
        default { $null }
      }
      if ($kind) { $logs += [ordered]@{ utc = $stamp; level = $level; version = $version; kind = $kind } }
    }
  }
}
$events = @(); $eventStatus = 'read'
try {
  $recent = Get-WinEvent -FilterHashtable @{ LogName = 'Application'; Id = @(1000, 1001); StartTime = (Get-Date).AddDays(-3) } -MaxEvents 200 -ErrorAction Stop
  foreach ($event in $recent) {
    $xml = [xml]$event.ToXml(); $fields = @{}
    foreach ($datum in $xml.Event.EventData.Data) { if ($datum.Name) { $fields[[string]$datum.Name] = [string]$datum.'#text' } }
    $appName = $fields.AppName
    if (-not $appName) { $appName = $fields.P1 }
    if ($appName -notmatch '^(Notaya|Cipherleaf|msedgewebview2)\.exe$') { continue }
    $safe = [ordered]@{ utc = $event.TimeCreated.ToUniversalTime().ToString('o'); id = $event.Id; app = $appName }
    foreach ($field in @('AppVersion', 'ModuleVersion', 'ExceptionCode', 'FaultingOffset')) {
      if ($fields[$field] -match '^[0-9a-fA-Fx.\-]{1,40}$') { $safe[$field] = $fields[$field] }
    }
    if ($fields.ModuleName -match '^[a-zA-Z0-9_.-]{1,80}\.(dll|exe)$') { $safe.module = $fields.ModuleName }
    $events += $safe
  }
} catch { $eventStatus = 'no matching events or access unavailable (raw error omitted)' }
[ordered]@{ utc = [DateTime]::UtcNow.ToString('o'); runCommand = Get-CommandShape $run;
  liveProcesses = $live; webview2Versions = @($versions | Select-Object -Unique);
  appLogStatus = $logStatus; appLogLines = $logs; eventLogStatus = $eventStatus; eventLogLines = $events;
  privacy = 'Exe paths, usernames, notes, passwords, raw log text and raw event messages omitted. No upload performed.'
} | ConvertTo-Json -Depth 6
