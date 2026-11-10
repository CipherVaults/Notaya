# Registers the Notaya stop-line guard as a per-user Windows scheduled task: one run every 2 minutes, indefinitely, hidden (through wscript.exe and
# stopline-guard-hidden.vbs, window style 0), only while you are logged on, no admin, no password stored. Same style as register-task-hidden.ps1.
# Run it yourself in a normal (non-admin) PowerShell, from the folder D:\Notaya-lanes\heartbeat:
#     cd D:\Notaya-lanes\heartbeat
#     powershell -NoProfile -ExecutionPolicy Bypass -File .\register-guard-task.ps1 -DryRun     (changes nothing, prints what it would do)
#     powershell -NoProfile -ExecutionPolicy Bypass -File .\register-guard-task.ps1             (registers the task)
# Safe to run again: it replaces the guard task, it never duplicates it, and it never touches the 'Notaya heartbeat' task.
# Undo: Unregister-ScheduledTask -TaskName 'Notaya stop-line guard' -Confirm:$false   (an earlier definition, if any, is saved in task-backup first)
param([switch]$DryRun)
$ErrorActionPreference = 'Stop'
$name = 'Notaya stop-line guard'
$h = 'D:\Notaya-lanes\heartbeat'
foreach ($f in 'stopline-guard.ps1', 'stopline-guard-hidden.vbs') { if (-not (Test-Path "$h\$f")) { throw "$f not found in $h" } }

$action   = New-ScheduledTaskAction -Execute 'wscript.exe' -Argument "//B //Nologo `"$h\stopline-guard-hidden.vbs`"" -WorkingDirectory $h
$trigger  = New-ScheduledTaskTrigger -Once -At (Get-Date).AddMinutes(1) -RepetitionInterval (New-TimeSpan -Minutes 2) -RepetitionDuration (New-TimeSpan -Days 3650)
# never start a second copy while one run is still going; a run is short (seconds), but stopping lanes pushes branches: allow 10 minutes
$settings = New-ScheduledTaskSettingsSet -MultipleInstances IgnoreNew -StartWhenAvailable -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit (New-TimeSpan -Minutes 10) -Hidden
$principal = New-ScheduledTaskPrincipal -UserId "$env:USERDOMAIN\$env:USERNAME" -LogonType Interactive -RunLevel Limited
$existing = Get-ScheduledTask -TaskName $name -ErrorAction SilentlyContinue

if ($DryRun) {
  Write-Host 'DRY RUN: nothing was registered, changed or stopped.'
  Write-Host ("Would register task '{0}' for {1}: wscript.exe //B //Nologo {2}\stopline-guard-hidden.vbs (starts powershell stopline-guard.ps1 with window style 0), every 2 minutes, indefinitely, hidden, only while you are logged on, no admin, no password, one instance at a time, run limit 10 minutes." -f $name, "$env:USERDOMAIN\$env:USERNAME", $h)
  Write-Host ("Trigger repetition interval: {0}; duration: {1}" -f $trigger.Repetition.Interval, $trigger.Repetition.Duration)
  Write-Host ("Existing task '{0}' now: {1}" -f $name, $(if ($existing) { 'yes, its XML would be saved to task-backup and then replaced' } else { 'none' }))
  Write-Host "The 'Notaya heartbeat' task is not touched."
  return
}

if ($existing) {
  New-Item -ItemType Directory -Force "$h\task-backup" | Out-Null
  $stamp = (Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssZ')
  schtasks /Query /TN $name /XML > "$h\task-backup\stopline-guard-before-$stamp.xml"
  Unregister-ScheduledTask -TaskName $name -Confirm:$false
}
Register-ScheduledTask -TaskName $name -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Description 'Notaya stop-line guard: every 2 minutes, stops running Codex lanes cleanly when the 5-hour window is at the stop line (used >= 95% or used + burn rate x 12 min >= 95%). Never starts lanes, never probes, never uses a banked reset. Works only in D:\Notaya-lanes and D:\Notaya Tests.' | Out-Null
Start-ScheduledTask -TaskName $name
Write-Host 'Notaya stop-line guard registered: every 2 minutes (hidden)'
Write-Host "Check it:   Get-ScheduledTask -TaskName '$name' | Get-ScheduledTaskInfo"
Write-Host "Log:        $h\logs\stopline-guard.log    Events: $h\events.log"
Write-Host "Remove it:  Unregister-ScheduledTask -TaskName '$name' -Confirm:`$false"
