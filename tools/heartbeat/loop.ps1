# DISABLED (2026-11-08). The heartbeat now runs as the scheduled task 'Notaya heartbeat' (see register-task.ps1), every 10 minutes.
# This file used to loop heartbeat.ps1 in a PowerShell window; it deliberately does nothing now so the two cannot double-run.
Write-Host "loop.ps1 is disabled: use register-task.ps1 (scheduled task 'Notaya heartbeat')."
