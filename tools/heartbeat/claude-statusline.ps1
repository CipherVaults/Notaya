# Claude Code status-line command. Reads the session JSON on stdin; when the claude.ai plan limits are present
# (rate_limits.five_hour / seven_day, Pro/Max/Ultra only, only after the first API response) it writes them to
# D:\Notaya-lanes\heartbeat\claude-rate.json for the heartbeat, and prints one short line for the status bar.
$raw = [Console]::In.ReadToEnd()
$out = if ($env:CLAUDE_RATE_OUT) { $env:CLAUDE_RATE_OUT } else { 'D:\Notaya-lanes\heartbeat\claude-rate.json' }
try { $j = $raw | ConvertFrom-Json } catch { Write-Output 'status line: bad input'; exit 0 }
$rl = $j.rate_limits
if ($rl -and $rl.five_hour -and $rl.seven_day) {
  $o = [ordered]@{ ts = (Get-Date).ToUniversalTime().ToString('o'); five_hour = [ordered]@{ used_percentage = $rl.five_hour.used_percentage; resets_at = $rl.five_hour.resets_at }; seven_day = [ordered]@{ used_percentage = $rl.seven_day.used_percentage; resets_at = $rl.seven_day.resets_at } }
  $tmp = "$out.tmp"; ($o | ConvertTo-Json -Depth 4) | Set-Content $tmp -Encoding UTF8; Move-Item $tmp $out -Force
  Write-Output ('5h {0}% | 7d {1}%' -f [int]$rl.five_hour.used_percentage, [int]$rl.seven_day.used_percentage)
} else { Write-Output 'plan limits not available yet' }
