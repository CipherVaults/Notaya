param([switch]$BeforeOnly)
$ErrorActionPreference = 'Stop'
$base = if ($BeforeOnly) { Join-Path $PSScriptRoot 'fixtures\before-fail-loud' } else { $PSScriptRoot }
. (Join-Path $base 'common.ps1')
$passed = 0
function Expect([bool]$Condition, [string]$Name) {
  if (-not $Condition) { throw "FAIL $Name" }
  $script:passed++; "PASS $Name"
}
# Stub the OS query: no listener or network state is inspected by these tests.
function Get-NetTCPConnection {
  [CmdletBinding()]param([string]$State, [int]$LocalPort)
  if ($script:ListenerMode -eq 'error') { Write-Error 'planted unavailable listener query'; return }
  if ($script:ListenerMode -eq 'busy') { [pscustomobject]@{ LocalPort = 9335; State = 'Listen' }; return }
  if ($script:ListenerMode -eq 'established') { [pscustomobject]@{ LocalPort = 9335; State = 'Established' }; return }
}
$script:ListenerMode = 'error'
if ($BeforeOnly) {
  Expect ((Get-ListenerCount 9335) -eq 0) 'before: unavailable listener query is mistaken for empty'
  $text = [IO.File]::ReadAllText((Join-Path $base 'run-installed.ps1'))
  $line = ($text -split "`n" | Where-Object { $_ -match '^\s*node ' } | Select-Object -First 1)
  Expect ([bool]$line) 'before: actual wrapper native command found'
  function node { $global:LASTEXITCODE = 1; 'planted child failed' }
  $port = 9335; $local = 'synthetic'; $continued = $false
  & ([scriptblock]::Create($line)) | Out-Null
  $continued = $true
  Expect ($continued -and $LASTEXITCODE -eq 1) 'before: failed native child does not terminate wrapper'
  $Scenarios = @()
  $selection = ([IO.File]::ReadAllLines((Join-Path $base 'run-lockflow.ps1')) | Where-Object { $_ -match '^\$Scenarios = ' } | Select-Object -First 1)
  & ([scriptblock]::Create($selection))
  Expect ($Scenarios.Count -eq 0) 'before: empty scenario selection is accepted'
} else {
  $refused = $false; try { Get-ListenerCount 9335 | Out-Null } catch { $refused = $true }
  Expect $refused 'unavailable listener query is a failure'
  $script:ListenerMode = 'empty'
  Expect ((Get-ListenerCount 9335) -eq 0) 'real empty listener result is accepted'
  $script:ListenerMode = 'busy'
  Expect ((Get-ListenerCount 9335) -eq 1) 'busy listener result is counted'
  $refused = $false; try { Assert-PortFree 9335 } catch { $refused = $true }
  Expect $refused 'busy listener prevents startup'
  $script:ListenerMode = 'established'
  Expect ((Get-ListenerCount 9335) -eq 0) 'a non-listening connection is not a listener'
  foreach ($value in @($false, [double]::NaN, [double]::PositiveInfinity, [double]::NegativeInfinity)) {
    $refused = $false; try { Assert-Test $value 'planted bad observation' } catch { $refused = $true }
    Expect $refused 'failed/non-finite observation cannot pass PowerShell assertion'
  }
  Assert-Test $true 'legitimate observation'
  Expect $true 'true observation is accepted'
  $child = Join-Path $PSScriptRoot 'fixtures\native-exit-child.cjs'
  Invoke-NodeCheck $child @('0')
  Expect $true 'good native child is accepted'
  $refused = $false; try { Invoke-NodeCheck $child @('1') } catch { $refused = $true }
  Expect $refused 'failed native child is propagated'
  foreach ($wrapper in 'run-installed.ps1','run-lockflow.ps1','run-restoreflow.ps1','run-tray.ps1','run-optionalsecurity.ps1') {
    $tokens = $null; $errors = $null
    $ast = [Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot $wrapper), [ref]$tokens, [ref]$errors)
    Expect ($errors.Count -eq 0) "$wrapper parses"
    $commands = @($ast.FindAll({ param($n) $n -is [Management.Automation.Language.CommandAst] }, $true))
    Expect (@($commands | Where-Object { $_.GetCommandName() -eq 'node' }).Count -eq 0) "$wrapper has no unchecked native Node command"
    Expect (@($commands | Where-Object { $_.GetCommandName() -eq 'Invoke-NodeCheck' }).Count -gt 0) "$wrapper uses the shared native-exit pattern"
    Expect (@($commands | Where-Object { $_.GetCommandName() -in 'Remove-Item','Remove-TestFolder' }).Count -eq 0) "$wrapper never deletes the retained run data or credential"
    Expect (@($commands | Where-Object { $_.GetCommandName() -eq 'Confirm-TestArtifactsRetained' }).Count -gt 0) "$wrapper checks artifact retention"
    Expect (@($commands | Where-Object { $_.GetCommandName() -eq 'Stop-TestApp' }).Count -gt 0) "$wrapper retains owned-process closure"
    Expect (@($commands | Where-Object { $_.GetCommandName() -eq 'Get-ListenerCount' }).Count -gt 0) "$wrapper retains debug-port closure checks"
  }
  $scratchParent = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..\..\..\runs\safety-graders-20261114T180045Z'))
  if (-not (Test-Path -LiteralPath $scratchParent -PathType Container)) { throw 'The explicitly owned qualification output folder must exist' }
  $scratch = New-TestRunFolder $scratchParent 'common-synthetic'
  $dummyExe = Join-Path $scratch 'Notaya.exe'
  [IO.File]::WriteAllText($dummyExe, 'NOT AN EXECUTABLE: Start-Process is stubbed in this test')
  $script:SimulatedProcesses = @()
  function Get-NotayaProcesses { return $script:SimulatedProcesses }
  function Start-Process {
    param([string]$FilePath, [string]$WindowStyle, [switch]$PassThru, [string]$RedirectStandardOutput, [string]$RedirectStandardError)
    $script:LaunchOutputPaths = @($RedirectStandardOutput, $RedirectStandardError)
    $script:LaunchEnvironment = @{}
    foreach ($name in 'NOTAYA_HOME','CIPHERLEAF_HOME','LOCALAPPDATA','APPDATA','USERPROFILE','TEMP','TMP','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS') {
      $script:LaunchEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
    }
    if ($script:LaunchThrows) { throw 'planted launch failure' }
    return [pscustomobject]@{ Id = -1 }
  }
  $oldHome = $env:NOTAYA_HOME; $oldFallback = $env:CIPHERLEAF_HOME
  try {
    $env:NOTAYA_HOME = 'sentinel-owner-home'; $env:CIPHERLEAF_HOME = 'sentinel-owner-fallback'
    $script:LaunchThrows = $false
    $null = Start-TestApp $dummyExe 9335
    # The app's env_var explicitly ignores empty values; PowerShell may clear an
    # environment entry to the empty string rather than remove its name.
    Expect ([string]::IsNullOrEmpty($script:LaunchEnvironment.NOTAYA_HOME) -and [string]::IsNullOrEmpty($script:LaunchEnvironment.CIPHERLEAF_HOME)) 'child cannot inherit an owner vault-home override'
    Expect ($env:NOTAYA_HOME -eq 'sentinel-owner-home' -and $env:CIPHERLEAF_HOME -eq 'sentinel-owner-fallback') 'parent home overrides restored after launch'
    foreach ($output in $script:LaunchOutputPaths) {
      Expect ([bool]$output -and $output.StartsWith($scratch + '\', [StringComparison]::OrdinalIgnoreCase)) 'application output is retained inside the owned run home'
    }
    foreach ($name in 'LOCALAPPDATA','APPDATA','USERPROFILE','TEMP','TMP') {
      Expect ($script:LaunchEnvironment[$name].StartsWith($scratch + '\', [StringComparison]::OrdinalIgnoreCase)) "child $name is owned throwaway data"
    }
    $script:LaunchThrows = $true
    $refused = $false; try { Start-TestApp $dummyExe 9335 | Out-Null } catch { $refused = $true }
    Expect $refused 'failed launch cannot pass'
    Expect ($env:NOTAYA_HOME -eq 'sentinel-owner-home' -and $env:CIPHERLEAF_HOME -eq 'sentinel-owner-fallback') 'parent home overrides restored after failed launch'
    $script:SimulatedProcesses = @([pscustomobject]@{ Path = 'D:\foreign\Notaya.exe' })
    $refused = $false; try { Assert-NoForeignNotaya } catch { $refused = $true }
    Expect $refused 'an external Notaya process prevents handover'
    $script:SimulatedProcesses = @([pscustomobject]@{ Path = $dummyExe })
    Assert-NoForeignNotaya
    Expect $true 'a previously owned test process may receive the second-instance test'
    $script:SimulatedProcesses = @()
    $refused = $false; try { Confirm-TestArtifactsRetained (Join-Path $scratchParent 'not-owned') | Out-Null } catch { $refused = $true }
    Expect $refused 'retention checks refuse an unowned target'
    $refused = $false; try { New-TestRunFolder $scratchParent '../escape' | Out-Null } catch { $refused = $true }
    Expect $refused 'run-home creation refuses escaping labels'
    $refused = $false; try { Register-TestFolder $PSScriptRoot } catch { $refused = $true }
    Expect $refused 'repository folders cannot hold synthetic vault credentials'
    $credential = New-PassphraseFile $scratch
    Expect ([IO.Path]::GetDirectoryName($credential) -eq $scratch) 'generated credential is inside its owned run home'
    Expect ((Get-Item -LiteralPath $credential).Length -gt 0) 'generated synthetic credential is retained without printing its value'
    $script:CleanupFailures.Clear()
    $script:LaterCleanupRan = $false
    Invoke-TestCleanup { Assert-Test $false 'planted cleanup failure' }
    Invoke-TestCleanup { $script:LaterCleanupRan = $true }
    $refused = $false; try { Complete-TestCleanup } catch { $refused = $true }
    Expect ($refused -and $script:LaterCleanupRan) 'cleanup attempts later actions and reports the earlier failure'
    $script:CleanupFailures.Clear()
  } finally {
    $env:NOTAYA_HOME = $oldHome; $env:CIPHERLEAF_HOME = $oldFallback
    Confirm-TestArtifactsRetained $scratch
    Expect (Test-Path -LiteralPath $dummyExe -PathType Leaf) 'all synthetic helper artifacts remain available after cleanup'
  }
}
"ALL OK ($passed wrapper/port assertions; synthetic only)"
