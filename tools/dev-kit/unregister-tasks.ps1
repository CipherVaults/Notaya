# Lists the two root tasks. A literal typed yes and XML backup precede removal.
param([switch]$DryRun, [string]$ConfigPath = (Join-Path $PSScriptRoot 'config.ps1'))
$ErrorActionPreference = 'Stop'
. $ConfigPath
if ($Cfg -isnot [hashtable]) { throw 'Config must define $Cfg as a hashtable.' }
$names = @($Cfg.TaskNameHeartbeat, $Cfg.TaskNameGuard) | Select-Object -Unique
foreach ($name in $names) {
  if (-not $name -or $name -match '[\\*?\[\]"\r\n]') { throw 'Invalid task name.' }
}
if ($DryRun) {
  foreach ($name in $names) { Write-Host "DRY RUN: would list root task '$name', ask for typed yes, back up its XML to KitDir/task-backup, then unregister it if present." }
  Write-Host 'DRY RUN: no task query, prompt, unregister, stop or file write performed.'
  return
}
$tasks = @()
foreach ($name in $names) {
  $task = Get-ScheduledTask -TaskName $name -TaskPath '\' -ErrorAction SilentlyContinue
  if ($task) { Write-Host "Task: $name; state: $($task.State)"; $tasks += $task }
  else { Write-Host "Task: $name; absent" }
}
if (-not $tasks.Count) { return }
if ((Read-Host 'Type yes to back up and unregister the listed tasks') -cne 'yes') { Write-Host 'Cancelled; nothing changed.'; return }
# Back up ALL selected tasks before removing ANY task.
$dir = Join-Path $Cfg.KitDir 'task-backup'
New-Item -ItemType Directory -Force $dir | Out-Null
$i = 0
foreach ($task in $tasks) {
  $xml = Export-ScheduledTask -TaskName $task.TaskName -TaskPath $task.TaskPath
  if (-not $xml -or $xml -notmatch '<Task\b') { throw 'XML export failed; no task removed.' }
  $file = Join-Path $dir ('task-' + $i + '-before-unregister-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ') + '-' + [guid]::NewGuid().ToString('N') + '.xml')
  [IO.File]::WriteAllText($file, ($xml -join "`r`n"), (New-Object Text.UTF8Encoding($false)))
  if (-not (Test-Path -LiteralPath $file) -or (Get-Item -LiteralPath $file).Length -eq 0) { throw 'Task backup failed; no task removed.' }
  Write-Host "Backed up '$($task.TaskName)' to $file"; $i++
}
foreach ($task in $tasks) {
  Unregister-ScheduledTask -TaskName $task.TaskName -TaskPath $task.TaskPath -Confirm:$false
  Write-Host "Unregistered '$($task.TaskName)' (running processes are not stopped)."
}
