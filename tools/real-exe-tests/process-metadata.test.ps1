# Exercise the actual discovery helper with fake process objects only. No OS query or app launch.
param(
  [string]$CommonPath = (Join-Path $PSScriptRoot 'common.ps1'),
  [switch]$BeforeOnly
)
$ErrorActionPreference = 'Stop'
$tokens = $null; $errors = $null
$tree = [Management.Automation.Language.Parser]::ParseFile($CommonPath, [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw 'Common helper has parse errors.' }
$helpers = @($tree.FindAll({ param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Get-NotayaProcesses' }, $true))
if ($helpers.Count -ne 1) { throw 'Expected exactly one process-discovery helper.' }
. ([scriptblock]::Create($helpers[0].Extent.Text))
$script:passed = 0
function Expect([bool]$Condition, [string]$Name) {
  if (-not $Condition) { throw "FAIL $Name" }
  $script:passed++; "PASS $Name"
}
function Expect-Refusal([string]$Name) {
  $refused = $false
  try { Get-NotayaProcesses | Out-Null } catch { $refused = $true }
  Expect $refused $Name
}
function Get-Process {
  [CmdletBinding()]param()
  if ($ErrorActionPreference -ne 'Stop') { throw 'Process discovery must fail closed on query errors.' }
  if ($script:QueryFails) { throw 'synthetic process enumeration denied' }
  return $script:FakeProcesses
}
function Start-Process { throw 'No process launch is allowed in this fixture.' }
function Stop-Process { throw 'No process stop is allowed in this fixture.' }
function Get-NetTCPConnection { throw 'No listener query is allowed in this fixture.' }
$script:QueryFails = $false

$exited = [pscustomobject]@{ ProcessName = 'Notaya'; HasExited = $true; Path = $null }
if ($BeforeOnly) {
  $script:FakeProcesses = @($exited)
  Expect (@(Get-NotayaProcesses).Count -eq 0) 'an exited process with a missing path is excluded'
  "ALL OK ($passed process-metadata assertions; synthetic only)"
  return
}

$live = [pscustomobject]@{ ProcessName = 'Notaya'; HasExited = $false; Path = 'G:\synthetic\Notaya.exe' }
$legacy = [pscustomobject]@{ ProcessName = 'Cipherleaf'; HasExited = $false; Path = 'G:\synthetic\Cipherleaf.exe' }
$unrelated = [pscustomobject]@{ ProcessName = 'Unrelated'; HasExited = $null; Path = $null }
$script:FakeProcesses = @($live, $legacy, $unrelated)
$found = @(Get-NotayaProcesses)
Expect ($found.Count -eq 2 -and [object]::ReferenceEquals($found[0], $live) -and [object]::ReferenceEquals($found[1], $legacy)) 'live Notaya and Cipherleaf retain their identities; unrelated names are excluded'
$script:FakeProcesses = @($exited)
Expect (@(Get-NotayaProcesses).Count -eq 0) 'an exited process with a missing path is excluded'

$exitedBeforePath = [pscustomobject]@{ ProcessName = 'Notaya'; HasExited = $true; PathReads = 0 }
$exitedBeforePath | Add-Member ScriptProperty Path { $this.PathReads++; throw 'path must not be read after proven exit' }
$script:FakeProcesses = @($exitedBeforePath)
Expect (@(Get-NotayaProcesses).Count -eq 0 -and $exitedBeforePath.PathReads -eq 0) 'proven exit is checked before any path access'

$race = [pscustomobject]@{ ProcessName = 'Notaya'; ExitReads = 0; Path = $null }
$race | Add-Member ScriptProperty HasExited { $this.ExitReads++; return ($this.ExitReads -ge 2) }
$script:FakeProcesses = @($race)
Expect (@(Get-NotayaProcesses).Count -eq 0 -and $race.ExitReads -eq 2) 'a missing path followed by proven exit is excluded'

$script:FakeProcesses = @([pscustomobject]@{ ProcessName = 'Notaya'; HasExited = $false; Path = $null })
Expect-Refusal 'a live process with a missing path is rejected'
$script:FakeProcesses = @([pscustomobject]@{ ProcessName = 'Notaya'; HasExited = $null; Path = $live.Path })
Expect-Refusal 'an unknown exit state is rejected even with a known path'
$unreadable = [pscustomobject]@{ ProcessName = 'Cipherleaf'; Path = $live.Path }
$unreadable | Add-Member ScriptProperty HasExited { throw 'synthetic exit metadata access denied' }
$script:FakeProcesses = @($unreadable)
Expect-Refusal 'an unreadable exit state is rejected'
$unreadablePath = [pscustomobject]@{ ProcessName = 'Notaya'; HasExited = $false }
$unreadablePath | Add-Member ScriptProperty Path { throw 'synthetic path metadata access denied' }
$script:FakeProcesses = @($unreadablePath)
Expect-Refusal 'a live process with unreadable path metadata is rejected'

$script:FakeProcesses = @($live, [pscustomobject]@{ ProcessName = 'Notaya'; HasExited = $false; Path = $null })
$script:EmittedBeforeFailure = 0
$refused = $false
try { Get-NotayaProcesses | ForEach-Object { $script:EmittedBeforeFailure++ } } catch { $refused = $true }
Expect ($refused -and $script:EmittedBeforeFailure -eq 0) 'metadata failure emits no partially validated process list'
$script:QueryFails = $true
Expect-Refusal 'a denied process query is rejected'
$script:QueryFails = $false; $script:FakeProcesses = @()
Expect (@(Get-NotayaProcesses).Count -eq 0) 'a genuine empty enumeration remains empty'
"ALL OK ($passed process-metadata assertions; synthetic only)"