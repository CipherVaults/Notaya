Notaya 1.6 heartbeat (Claude on Gaming-PC; Ahmed's orders 2026-11-08).
Scheduled task: 'Notaya heartbeat' (per user, no admin, every 10 minutes, hidden, one instance at a time). Registered by Ahmed running register-task.ps1 himself.
Each tick (heartbeat.ps1): lists lanes, restarts stopped lanes (a boundary stop with exit 0 and no file changes = idle-stop, relaunched up to 2 times), starts queued lanes when fewer than 3 Codex lane processes run and quota is available,
reads the Codex 5-hour and weekly percentages from Codex's own session files, pauses on the 5-hour limit until its reset time, pauses at weekly 100% and reports (NEVER uses or assumes a banked reset: Ahmed decides),
makes WIP checkpoint commits, and after a lane finishes runs the checks, commits and pushes its branch. Never merges. Works only in D:\Notaya-lanes and D:\Notaya Tests.
Files: heartbeat.log (one line per tick), events.log (idle-check, limit-pause/resume, weekly-limit, started, finished), heartbeat-state.json.
Remove the task:  Unregister-ScheduledTask -TaskName 'Notaya heartbeat' -Confirm:$false

2026-11-09 11:55Z: the scheduled task now starts heartbeat-hidden.vbs through wscript.exe (hidden, no console flash). Registered by Ahmed with register-task-hidden.ps1; backup of the old task XML in task-backup\; undo with register-task.ps1.
