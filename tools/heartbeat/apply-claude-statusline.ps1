# Adds ONE key ("statusLine") to your Claude Code user settings (%USERPROFILE%\.claude\settings.json) and nothing else.
# Run it yourself in PowerShell (no admin):  powershell -NoProfile -ExecutionPolicy Bypass -File D:\Notaya-lanes\heartbeat\apply-claude-statusline.ps1
# It makes a dated backup first, refuses to run twice, and validates the JSON before writing.
$s = "$env:USERPROFILE\.claude\settings.json"
$raw = Get-Content $s -Raw
if ($raw -match '"statusLine"') { Write-Host 'statusLine already present: nothing changed.'; return }
Copy-Item $s ("$s.backup-" + (Get-Date -Format 'yyyyMMdd-HHmmss')) -Force
$line = '  "statusLine": { "type": "command", "command": "powershell -NoProfile -ExecutionPolicy Bypass -File D:\\Notaya-lanes\\heartbeat\\claude-statusline.ps1" },'
$new = $raw -replace '^\{\s*\r?\n', ("{`r`n" + $line + "`r`n")
$null = $new | ConvertFrom-Json
[System.IO.File]::WriteAllText($s, $new, (New-Object System.Text.UTF8Encoding($false)))
Write-Host 'Claude status line enabled (settings.json backed up next to it). To undo: delete the "statusLine" line from settings.json or restore the .backup- file.'
