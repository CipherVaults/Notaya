# Extract the actual getter and use fake child metadata/output. No real process or Node is invoked.
param([string]$TraySource = (Join-Path $PSScriptRoot 'run-tray.ps1'))
$ErrorActionPreference = 'Stop'
$tokens=$null; $errors=$null
$tree=[Management.Automation.Language.Parser]::ParseFile($TraySource,[ref]$tokens,[ref]$errors)
if ($errors.Count) { throw 'Tray source parse errors' }
$helper=$tree.Find({param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq 'Get-VisibleWindowCount'},$true)
if (!$helper) { throw 'Actual visibility getter missing' }
if ($helper.Parameters.Count -ne 1 -or $helper.Parameters[0].Name.VariablePath.UserPath -cne 'p') { throw 'Actual getter parameter contract changed' }
$script:visibilityGetterBody = $helper.Body.GetScriptBlock()
function Get-VisibleWindowCount($p) { & $script:visibilityGetterBody }
$port=9337; $script:passed=0; $script:calls=0; $script:throws=$false
function Expect([bool]$Condition,[string]$Name) { if (!$Condition) { throw "FAIL $Name" }; $script:passed++; "PASS $Name" }
function Invoke-NodeCheck([string]$ScriptPath,[object[]]$Arguments=@()) {
  $script:calls++
  if ([IO.Path]::GetFileName($ScriptPath) -cne 'main-window-visible.mjs' -or $Arguments.Count -ne 1 -or $Arguments[0] -ne 9337) { throw 'Getter bypassed the shared reader/port binding' }
  if ($script:throws) { throw 'synthetic reader exit failure' }
  foreach ($line in $script:output) { Write-Output -NoEnumerate $line }
}
$p=[pscustomobject]@{HasExited=$false;Refreshes=0}
$p | Add-Member ScriptMethod Refresh { $this.Refreshes++ }
foreach ($case in @(@{output='true';want=1},@{output='false';want=0})) {
  $script:output=@($case.output)
  Expect ((Get-VisibleWindowCount $p) -eq $case.want) ('single boolean '+$case.output+' is preserved')
}
foreach ($case in @(@{name='empty';output=@()},@{name='multiple';output=@('true','false')},@{name='null';output=@($null)},@{name='string null';output=@('null')},@{name='NaN';output=@([double]::NaN)},@{name='Infinity';output=@([double]::PositiveInfinity)},@{name='numeric zero';output=@(0)},@{name='boolean object';output=@($false)},@{name='whitespace';output=@(' false ')},@{name='wrong case';output=@('False')})) {
  $script:output=$case.output; $refused=$false
  try { Get-VisibleWindowCount $p | Out-Null } catch { $refused=$true }
  Expect $refused ($case.name+' reader output cannot become hidden')
}
$script:throws=$true; $refused=$false
try { Get-VisibleWindowCount $p | Out-Null } catch { $refused=$true }
Expect $refused 'reader failure propagates'; $script:throws=$false
$p.HasExited=$true; $before=$script:calls
Expect ((Get-VisibleWindowCount $p) -eq -1 -and $script:calls -eq $before) 'proven exited child remains -1 without reading visibility'
$p.HasExited=$null; $refused=$false
try { Get-VisibleWindowCount $p | Out-Null } catch { $refused=$true }
Expect $refused 'unknown child exit state fails closed'
"ALL OK ($script:passed visibility boundary assertions; synthetic only)"