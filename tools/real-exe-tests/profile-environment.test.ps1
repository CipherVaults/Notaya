# Synthetic launch-boundary fixture. Start-Process is stubbed; no app/network/OS query.
param(
  [Parameter(Mandatory)][string]$CommonPath,
  [Parameter(Mandatory)][ValidatePattern('^[A-Fa-f0-9]{64}$')][string]$ExpectedCommonSHA256,
  [Parameter(Mandatory)][string]$ScratchRoot,
  [Parameter(Mandatory)][string]$ScratchParent,
  [switch]$BeforeOnly
)
$ErrorActionPreference = 'Stop'
if ((Get-FileHash -LiteralPath $CommonPath -Algorithm SHA256).Hash -cne $ExpectedCommonSHA256.ToUpperInvariant()) { throw 'Fixture common-source pin mismatch' }
. $CommonPath
$script:passed = 0
function Expect([bool]$Condition, [string]$Name) {
  if (-not $Condition) { throw "FAIL $Name" }
  $script:passed++; "PASS $Name"
}
function Get-NotayaProcesses { return @() }
function Get-Process { throw 'No OS process query is allowed in this fixture' }
function Stop-Process { throw 'No process termination is allowed in this fixture' }
function Get-NetTCPConnection { throw 'No listener query is allowed in this fixture' }
$script:CaptureNames = @('NOTAYA_HOME','CIPHERLEAF_HOME','USERPROFILE','HOME','APPDATA','LOCALAPPDATA','TEMP','TMP','WEBVIEW2_USER_DATA_FOLDER','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS')
$script:Launches = [Collections.Generic.List[object]]::new()
$script:LaunchThrows = $false
$script:ForbidForeignProbe = $false
$script:ForeignProbeObserved = $false
$script:ForbiddenHome = $null

function Start-Process {
  param([string]$FilePath, [string]$WindowStyle, [switch]$PassThru, [string]$RedirectStandardOutput, [string]$RedirectStandardError)
  $captured = [ordered]@{}
  foreach ($name in $script:CaptureNames) { $captured[$name] = [Environment]::GetEnvironmentVariable($name, 'Process') }
  $script:Launches.Add([pscustomobject]@{environment=$captured;windowStyle=$WindowStyle;stdout=$RedirectStandardOutput;stderr=$RedirectStandardError})
  if ($script:LaunchThrows) { throw 'planted launch failure' }
  return [pscustomobject]@{Id=-1}
}
$allowedParent = [IO.Path]::GetFullPath($ScratchParent)
if ($allowedParent -cne $ScratchParent) { throw 'Canonical explicit scratch parent required' }
Assert-OutsideRepository $allowedParent
if ([IO.Path]::GetFullPath($ScratchRoot) -cne $ScratchRoot -or [IO.Path]::GetDirectoryName($ScratchRoot) -cne $allowedParent -or [IO.Path]::GetFileName($ScratchRoot) -cnotmatch '^profile-env-(before|after|seed)-[a-f0-9]{12}$') { throw 'Explicit fresh owned scratch child required' }
Assert-PlainPath $allowedParent
if (Test-Path -LiteralPath $ScratchRoot) { throw 'Do not reuse or overwrite a fixture run' }
[void][IO.Directory]::CreateDirectory($ScratchRoot)
Assert-PlainPath $ScratchRoot
$fixture = Join-Path $ScratchRoot 'registered'
[void][IO.Directory]::CreateDirectory($fixture)
Register-TestFolder $fixture
$dummyExe = Join-Path $fixture 'Notaya.exe'
$dummy = [IO.File]::Open($dummyExe,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None)
try { $bytes=[Text.Encoding]::UTF8.GetBytes('NOT AN EXECUTABLE: Start-Process is stubbed');$dummy.Write($bytes,0,$bytes.Length) } finally { $dummy.Dispose() }
$profileHome = Join-Path $fixture '.runner-profile'
$parentProfile = Join-Path $fixture 'synthetic-parent-profile'
[void][IO.Directory]::CreateDirectory($parentProfile)
$saved = @{}
$sentinels = @{}
$restoration = [Collections.Generic.List[object]]::new()
foreach ($name in $script:CaptureNames) {
  $saved[$name] = [Environment]::GetEnvironmentVariable($name,'Process')
  $sentinels[$name] = if ($name -eq 'WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS') { 'synthetic-parent-debug-sentinel' } else { Join-Path $parentProfile $name.ToLowerInvariant() }
}
function Expect-ParentRestored([string]$Phase) {
  $same = $true
  foreach ($name in $script:CaptureNames) {
    if ([Environment]::GetEnvironmentVariable($name,'Process') -cne $sentinels[$name]) { $same=$false }
  }
  $restoration.Add([pscustomobject]@{phase=$Phase;allWatchedVariablesRestored=$same})
  Expect $same ('all parent variables including HOME restored '+$Phase)
}
try {
  foreach ($name in $script:CaptureNames) { [Environment]::SetEnvironmentVariable($name,$sentinels[$name],'Process') }
  $null = Start-TestApp $dummyExe 9335
  $first = $script:Launches[0].environment
  Expect ($script:Launches.Count -eq 1 -and $script:Launches[0].windowStyle -ceq 'Hidden') 'the unmodified hidden launch boundary is captured once'
  if ($BeforeOnly) {
    Expect ($first.USERPROFILE -ceq (Join-Path $profileHome 'home')) 'before: USERPROFILE points at the old home child'
    Expect ($first.HOME -ceq $sentinels.HOME -and $first.HOME -cne $first.USERPROFILE) 'before: HOME is inherited synthetic parent state, not the child USERPROFILE'
    Expect ($first.APPDATA -ceq (Join-Path $profileHome 'roaming') -and $first.LOCALAPPDATA -ceq (Join-Path $profileHome 'local') -and $first.TEMP -ceq (Join-Path $profileHome 'temp')) 'before: AppData and Temp are siblings outside USERPROFILE'
  } else {
    Expect ($first.USERPROFILE -ceq $profileHome -and $first.HOME -ceq $profileHome) 'default USERPROFILE and HOME share the owned profile root'
    Expect ($first.APPDATA -ceq (Join-Path $profileHome 'AppData\Roaming')) 'default roaming AppData is canonical beneath the profile'
    Expect ($first.LOCALAPPDATA -ceq (Join-Path $profileHome 'AppData\Local')) 'default local AppData is canonical beneath the profile'
    Expect ($first.TEMP -ceq (Join-Path $profileHome 'AppData\Local\Temp') -and $first.TMP -ceq $first.TEMP) 'default TEMP and TMP are canonical beneath local AppData'
    foreach ($name in 'USERPROFILE','HOME','APPDATA','LOCALAPPDATA','TEMP','TMP') {
      Assert-OwnedTestFolder $first[$name]
      Expect $true ('child '+$name+' passed actual owned/no-link checks')
    }
  }
  Expect ([string]::IsNullOrEmpty($first.NOTAYA_HOME) -and [string]::IsNullOrEmpty($first.CIPHERLEAF_HOME) -and [string]::IsNullOrEmpty($first.WEBVIEW2_USER_DATA_FOLDER)) 'owner vault-home and WebView overrides remain cleared for the child'
  Expect ($first.WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ceq '--remote-debugging-port=9335') 'the original debug argument is unchanged'
  Expect-ParentRestored 'after successful launch'

  $explicitLocal = Join-Path $fixture 'explicit-installed-localappdata'
  foreach ($attempt in 1,2) {
    $null = Start-TestApp $dummyExe 9335 '' '' @{LOCALAPPDATA=$explicitLocal}
    $last = $script:Launches[$script:Launches.Count-1].environment
    Expect ($last.LOCALAPPDATA -ceq $explicitLocal) 'explicit installed LOCALAPPDATA survives first start and restart unchanged'
    if (-not $BeforeOnly) { Expect ($last.HOME -ceq $profileHome -and $last.USERPROFILE -ceq $profileHome) 'explicit LOCALAPPDATA does not silently remap profile defaults' }
    Assert-OwnedTestFolder $explicitLocal
    Expect-ParentRestored 'after explicit LOCALAPPDATA launch'
  }

  $script:LaunchThrows=$true
  $refused=$false
  try { Start-TestApp $dummyExe 9335 | Out-Null } catch { $refused=$true }
  Expect $refused 'a failed launch remains a failure'
  $failed = $script:Launches[$script:Launches.Count-1].environment
  Expect ($failed.USERPROFILE -ceq $first.USERPROFILE -and $failed.HOME -ceq $first.HOME) 'the failed launch was reached with the same captured child profile'
  Expect-ParentRestored 'after failed launch'
  $script:LaunchThrows=$false

  # This unregistered parent is synthetic and inside this fixture-owned outer root.
  # It is deliberately outside the helper's RegisteredTestFolders allowlist.
  $unregistered = Join-Path $ScratchRoot 'not-registered'
  [void][IO.Directory]::CreateDirectory($unregistered)
  $foreignHome = Join-Path $unregistered 'must-not-be-created'
  $countBefore=$script:Launches.Count
  $refused=$false
  $script:ForbiddenHome=$foreignHome
  $script:ForeignProbeObserved=$false
  if ((Get-Command Test-Path).CommandType -ne 'Cmdlet') { throw 'Unexpected existing Test-Path override' }
function Test-Path {
  [CmdletBinding()]param([string]$LiteralPath)
  if ($script:ForbidForeignProbe -and $LiteralPath -ceq $script:ForbiddenHome) {
    $script:ForeignProbeObserved = $true
    throw 'planted forbidden HOME probe; no filesystem query performed'
  }
  Microsoft.PowerShell.Management\Test-Path -LiteralPath $LiteralPath -ErrorAction Stop
}
  $script:ForbidForeignProbe=$true
  try { Start-TestApp $dummyExe 9335 '' '' @{HOME=$foreignHome} | Out-Null } catch { $refused=$true }
  finally { $script:ForbidForeignProbe=$false; Remove-Item -LiteralPath 'Function:\Test-Path' -ErrorAction Stop }
  if ($BeforeOnly) {
    Expect (-not $refused -and $script:Launches.Count -eq $countBefore+1 -and $script:Launches[$script:Launches.Count-1].environment.HOME -ceq $foreignHome) 'before: ExtraEnv HOME bypasses the folder-ownership loop'
  } else {
    Expect ($refused -and $script:Launches.Count -eq $countBefore) 'unowned HOME is refused before any launch'
    Expect (-not $script:ForeignProbeObserved) 'unowned HOME is refused before any existence probe'
  }
  Expect (-not (Test-Path -LiteralPath $foreignHome)) 'unowned HOME is not created while checking ownership'
  Expect-ParentRestored 'after HOME ownership case'
  if (-not $BeforeOnly) {
    $allowedHome=Join-Path $fixture 'allowed-extra-home'
    $null=Start-TestApp $dummyExe 9335 '' '' @{HOME=$allowedHome}
    $accepted=$script:Launches[$script:Launches.Count-1].environment
    Expect ($accepted.HOME -ceq $allowedHome) 'an explicitly owned HOME override is accepted unchanged'
    Assert-OwnedTestFolder $allowedHome
    Expect (Test-Path -LiteralPath $allowedHome) 'the accepted owned HOME is created within the registered run'
    Expect-ParentRestored 'after accepted HOME override'
  }
} finally {
  foreach ($name in $saved.Keys) { [Environment]::SetEnvironmentVariable($name,$saved[$name],'Process') }
  $capturePath=Join-Path $ScratchRoot 'synthetic-child-environment-capture.json'
  $record=[ordered]@{source=(Get-FileHash -LiteralPath $CommonPath -Algorithm SHA256).Hash;beforeSourceMode=[bool]$BeforeOnly;appLaunches=0;stubLaunches=$script:Launches.Count;assertions=$script:passed;capturedSyntheticChildren=$script:Launches.ToArray();parentRestoration=$restoration.ToArray();retained=$true}
  $out=[IO.File]::Open($capturePath,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None)
  try { $data=[Text.UTF8Encoding]::new($false).GetBytes(($record|ConvertTo-Json -Depth 8)+"`n");$out.Write($data,0,$data.Length) } finally { $out.Dispose() }
}
"ALL OK ($script:passed canonical-profile assertions; synthetic launch stub only)"
