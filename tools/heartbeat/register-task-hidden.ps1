# HIDDEN-LAUNCHER VERSION (2026-11-09): same task, but the action is wscript.exe running heartbeat-hidden.vbs (window style 0), so no console window flashes every 10 minutes. The old definition is saved in task-backup\Notaya-heartbeat-before-hidden-2026-11-09.xml; to undo, run register-task.ps1.
# Registers the Notaya 1.6 heartbeat as a per-user Windows scheduled task: one tick every 10 minutes, indefinitely.
# Run it yourself in a normal (non-admin) PowerShell:
#     powershell -NoProfile -ExecutionPolicy Bypass -File D:\Notaya-lanes\heartbeat\register-task.ps1
# Dry run (changes nothing, only checks the pieces and prints what it would do):
#     powershell -NoProfile -ExecutionPolicy Bypass -File D:\Notaya-lanes\heartbeat\register-task.ps1 -DryRun
# Safe to run again: it replaces the task, it never duplicates it.
param([switch]$DryRun)
$ErrorActionPreference = 'Stop'
$name = 'Notaya heartbeat'
$h = 'D:\Notaya-lanes\heartbeat'
if (-not (Test-Path "$h\heartbeat.ps1")) { throw "heartbeat.ps1 not found in $h" }

$action   = New-ScheduledTaskAction -Execute 'wscript.exe' -Argument "//B //Nologo `"$h\heartbeat-hidden.vbs`"" -WorkingDirectory $h
# start in 1 minute, then every 10 minutes for ten years (= indefinitely)
$trigger  = New-ScheduledTaskTrigger -Once -At (Get-Date).AddMinutes(1) -RepetitionInterval (New-TimeSpan -Minutes 10) -RepetitionDuration (New-TimeSpan -Days 3650)
# never start a second copy while one tick is still running; allow a tick up to 3 hours (checks can be long); hidden
$settings = New-ScheduledTaskSettingsSet -MultipleInstances IgnoreNew -StartWhenAvailable -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit (New-TimeSpan -Hours 3) -Hidden
# your own user, only while you are logged on, no elevation
$principal = New-ScheduledTaskPrincipal -UserId "$env:USERDOMAIN\$env:USERNAME" -LogonType Interactive -RunLevel Limited

if ($DryRun) {
  Write-Host "DRY RUN: nothing was registered or stopped."
  Write-Host ("Would register task '{0}' for {1}: wscript.exe {2}\heartbeat-hidden.vbs, every 10 minutes, indefinitely, hidden, only when you are logged on, no admin, one instance at a time." -f $name, "$env:USERDOMAIN\$env:USERNAME", $h)
  Write-Host ("Would remove the older task 'Notaya16-Heartbeat' if it exists, and stop the old session loop (loop.ps1) if it is running.")
  Write-Host ("Trigger repetition interval: {0}; duration: {1}" -f $trigger.Repetition.Interval, $trigger.Repetition.Duration)
  return
}

# idempotent: remove any earlier version first
foreach ($old in @($name, 'Notaya16-Heartbeat')) { if (Get-ScheduledTask -TaskName $old -ErrorAction SilentlyContinue) { Unregister-ScheduledTask -TaskName $old -Confirm:$false } }
# the old session loop (30-minute cadence) must not double-run next to the task
Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match 'heartbeat\\loop\.ps1' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }

Register-ScheduledTask -TaskName $name -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Description 'Notaya 1.6 overnight heartbeat: restarts stopped Codex lanes, respects the Codex 5-hour and weekly limits, never merges, never uses banked resets. Works only in D:\Notaya-lanes and D:\Notaya Tests.' | Out-Null
Start-ScheduledTask -TaskName $name
Write-Host 'Notaya heartbeat registered: every 10 minutes'
Write-Host "Check it:   Get-ScheduledTask -TaskName '$name' | Get-ScheduledTaskInfo   (or open Task Scheduler and look for '$name')"
Write-Host "Log:        $h\heartbeat.log   (one line per tick)    Events: $h\events.log"
Write-Host "Remove it:  Unregister-ScheduledTask -TaskName '$name' -Confirm:`$false"
