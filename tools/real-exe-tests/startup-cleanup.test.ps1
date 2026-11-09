# No processes are started/stopped: extract the production helper and use fakes.
$ErrorActionPreference = 'Stop'
$tokens = $null; $errors = $null
$tree = [Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot 'run-startup-minimized.ps1'), [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw 'Runner has parse errors.' }
$helper = $tree.Find({ param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Stop-OwnedStartupChild' }, $true)
if (-not $helper) { throw 'Missing cleanup helper.' }
. ([scriptblock]::Create($helper.Extent.Text))

function Test-Child([string]$Case, $Start, $Recorded, [bool]$Exited, [int]$RecordedId, [int]$Expected) {
  $child = [pscustomobject]@{ Id = 101; HasExited = $Exited; Killed = 0 }
  if ($Case -eq 'throwing time') {
    $child | Add-Member ScriptProperty StartTime { throw 'unreadable' }
  } else { $child | Add-Member NoteProperty StartTime $Start }
  $child | Add-Member ScriptMethod Kill { $this.Killed++ }
  Stop-OwnedStartupChild @{ id = $RecordedId; started = $Recorded; process = $child }
  if ($child.Killed -ne $Expected) { throw "Failed: $Case" }
  "PASS: $Case"
}
$time = [DateTime]::UtcNow
$ticks = $time.ToUniversalTime().Ticks
Test-Child 'matching time' $time $ticks $false 101 1
Test-Child 'null time' $null $ticks $false 101 1
Test-Child 'throwing time' $null $ticks $false 101 1
Test-Child 'unreadable discovery time' $time $null $false 101 1
Test-Child 'reused id with different time' $time ($ticks - 1) $false 101 0
Test-Child 'unrelated id and null time' $null $ticks $false 102 0
Test-Child 'exited child' $null $ticks $true 101 0
Stop-OwnedStartupChild @{ id = 101; started = $ticks; process = $null }
'PASS: missing process'
