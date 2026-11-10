# Run -DryRun first. Actual invocation replaces/starts only the configured task.
param([switch]$DryRun, [string]$ConfigPath = (Join-Path $PSScriptRoot 'config.ps1'))
$ErrorActionPreference = 'Stop'
. $ConfigPath
if ($Cfg -isnot [hashtable]) { throw 'Config must define $Cfg as a hashtable.' }
$name = [string]$Cfg.TaskNameHeartbeat
$minutes = [double]$Cfg.TickMinutes
if (-not $name -or $name -match '[\\*?\[\]"\r\n]' -or $minutes -lt 1) { throw 'Invalid task name or interval.' }
$h = [IO.Path]::GetFullPath($Cfg.KitDir)
$config = (Resolve-Path -LiteralPath $ConfigPath).ProviderPath
if (($h + $config) -match '["\r\n]') { throw 'Task paths cannot contain quotes or newlines.' }
foreach ($f in 'heartbeat.ps1', 'heartbeat-hidden.vbs') {
  if (-not (Test-Path -LiteralPath (Join-Path $h $f))) { throw "$f not found in KitDir" }
}
function Backup-Task($task) {
  $xml = Export-ScheduledTask -TaskName $task.TaskName -TaskPath $task.TaskPath
  if (-not $xml -or $xml -notmatch '<Task\b') { throw 'Task XML export failed; task unchanged.' }
  $dir = Join-Path $h 'task-backup'
  New-Item -ItemType Directory -Force $dir | Out-Null
  $file = Join-Path $dir ('heartbeat-before-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ') + '-' + [guid]::NewGuid().ToString('N') + '.xml')
  $document = New-Object System.Xml.XmlDocument
  $document.XmlResolver = $null
  $document.LoadXml(($xml -join "`r`n"))
  if ($document.FirstChild -is [System.Xml.XmlDeclaration]) { $document.FirstChild.Encoding = 'utf-8' }
  [IO.File]::WriteAllText($file, $document.OuterXml, (New-Object Text.UTF8Encoding($false)))
  if (-not (Test-Path -LiteralPath $file) -or (Get-Item -LiteralPath $file).Length -eq 0) { throw 'Task backup failed; task unchanged.' }
  Write-Host "Backed up '$($task.TaskName)' to $file"
}
if ($DryRun) {
  Write-Host "DRY RUN: would back up existing root task '$name' to KitDir/task-backup before replacement."
  Write-Host "DRY RUN: would register '$name': wscript.exe //B //Nologo `"$h\heartbeat-hidden.vbs`" `"$config`"; every $minutes minutes; hidden; interactive limited user; IgnoreNew; 180-minute limit; then start it."
  Write-Host 'DRY RUN: no task query, registration, process query, stop or file write performed.'
  return
}
$rootTasks = @(Get-ScheduledTask -ErrorAction Stop | Where-Object { $_.TaskPath -eq '\' })
$existing = $rootTasks | Where-Object { $_.TaskName -eq $name }
if ($existing) { Backup-Task $existing }
$action = New-ScheduledTaskAction -Execute 'wscript.exe' -Argument "//B //Nologo `"$h\heartbeat-hidden.vbs`" `"$config`"" -WorkingDirectory $h
$trigger = New-ScheduledTaskTrigger -Once -At (Get-Date).AddMinutes(1) -RepetitionInterval (New-TimeSpan -Minutes $minutes) -RepetitionDuration (New-TimeSpan -Days 3650)
$settings = New-ScheduledTaskSettingsSet -MultipleInstances IgnoreNew -StartWhenAvailable -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit (New-TimeSpan -Minutes 180) -Hidden
$principal = New-ScheduledTaskPrincipal -UserId "$env:USERDOMAIN\$env:USERNAME" -LogonType Interactive -RunLevel Limited
# Optional extra contract key: LegacyTaskNames = @() identifies old root tasks to retire.
foreach ($old in @($Cfg.LegacyTaskNames)) {
  if (-not $old -or $old -eq $name -or $old -eq $Cfg.TaskNameGuard) { continue }
  if ($old -match '[\\*?\[\]"\r\n]') { throw 'Invalid legacy task name.' }
  $prior = $rootTasks | Where-Object { $_.TaskName -eq $old }
  if ($prior) { Backup-Task $prior; Unregister-ScheduledTask -TaskName $old -TaskPath '\' -Confirm:$false }
}
# The prior foreground loop must not run alongside the scheduled heartbeat.
$loopPath = Join-Path $h 'loop.ps1'
$rx = '(?i)(?:^|\s)-File\s+(?:"' + [regex]::Escape($loopPath) + '"|' + [regex]::Escape($loopPath) + ')(?=\s|$)'
foreach ($proc in @(Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match $rx })) {
  if ($Cfg.EnableProcessStop -eq $true) { Stop-Process -Id $proc.ProcessId -Force }
  else { Write-Host "EnableProcessStop=false: would stop legacy loop PID $($proc.ProcessId)" }
}
Register-ScheduledTask -TaskName $name -TaskPath '\' -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Description 'Dev kit heartbeat; configured paths and stop-line limits; no banked resets.' -Force | Out-Null
Start-ScheduledTask -TaskName $name -TaskPath '\'
Write-Host "Registered '$name': every $minutes minutes (hidden)."
