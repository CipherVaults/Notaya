# One-line model check. Real invocation calls the signed-in CLI and consumes quota.
param(
  [string]$Model = '', [ValidateRange(0,86400)][int]$WaitSeconds = 0,
  [string]$Effort = '', [switch]$DryRun,
  [string]$ConfigPath = (Join-Path $PSScriptRoot 'config.ps1')
)
$ErrorActionPreference = 'Stop'
. $ConfigPath
if ($Cfg -isnot [hashtable]) { throw 'Config must define $Cfg as a hashtable.' }
if (-not $Model) { $Model = $Cfg.DefaultModel }
if (-not $Effort) { $Effort = $Cfg.DefaultEffort }
if ($Model -notmatch '^[a-zA-Z0-9_.-]+$' -or $Effort -notin @('minimal','low','medium','high','xhigh','max')) { throw 'Invalid model or effort.' }
$probe = Join-Path $Cfg.TestsDir 'model-probe'
$jl = Join-Path $Cfg.OutDir 'terra-trial.jsonl'
$res = Join-Path $Cfg.OutDir 'terra-trial.txt'
if ($DryRun) {
  Write-Host "DRY RUN: wait $WaitSeconds seconds; CLI '$($Cfg.CodexCmd)' exec --json -m $Model -c model_reasoning_effort=$Effort -s read-only --skip-git-repo-check -C '$probe' 'Reply with ok.'"
  Write-Host "DRY RUN: would write '$jl' and '$res'; no wait, directory write or CLI invocation performed."
  return
}
if ($WaitSeconds) { Start-Sleep -Seconds $WaitSeconds }
New-Item -ItemType Directory -Force $probe, $Cfg.OutDir | Out-Null
"START $([DateTime]::UtcNow.ToString('o'))" | Set-Content -LiteralPath $res
Push-Location -LiteralPath $probe
try {
  & $Cfg.CodexCmd exec --json -m $Model -c "model_reasoning_effort=`"$Effort`"" -s read-only --skip-git-repo-check -C $probe 'Reply with ok.' > $jl 2>&1
  $code = $LASTEXITCODE
} finally { Pop-Location }
"MODEL TRIED $Model exit=$code" | Add-Content -LiteralPath $res
$lines = @(Get-Content -LiteralPath $jl)
$errorsSeen = @($lines | Where-Object { $_ -match '"type":"error"|unknown model|not found|does not exist|invalid model|unsupported' } | Select-Object -First 3)
foreach ($e in $errorsSeen) { ('ERRLINE ' + $e.Substring(0, [Math]::Min(300, $e.Length))) | Add-Content -LiteralPath $res }
$tc = @($lines | Where-Object { $_ -match '"token_count"' -and $_ -match '"primary"' })
if ($tc.Count) {
  $first = $tc[0] | ConvertFrom-Json; $last = $tc[-1] | ConvertFrom-Json
  ('METER after trial: 5h {0}% (event {1}), weekly {2}%; first event 5h {3}%' -f $last.payload.rate_limits.primary.used_percent, $last.timestamp, $last.payload.rate_limits.secondary.used_percent, $first.payload.rate_limits.primary.used_percent) | Add-Content -LiteralPath $res
  $u = $last.payload.info.total_token_usage
  if ($u) { ('TOKENS input {0} cached {1} output {2} reasoning {3} total {4}' -f $u.input_tokens, $u.cached_input_tokens, $u.output_tokens, $u.reasoning_output_tokens, $u.total_tokens) | Add-Content -LiteralPath $res }
} else { 'no token_count event with rate limits found' | Add-Content -LiteralPath $res }
$models = @($lines | ForEach-Object { if ($_ -match '"model"\s*:\s*"([^"]+)"') { $Matches[1] } } | Select-Object -Unique)
"MODEL ids seen in event stream: $($models -join ',')" | Add-Content -LiteralPath $res
$reply = @($lines | Where-Object { $_ -match '"agent_message"|"item.completed"' } | Select-Object -Last 1) -join ''
('REPLY: ' + $reply.Substring(0, [Math]::Min(200, $reply.Length))) | Add-Content -LiteralPath $res
"DONE $([DateTime]::UtcNow.ToString('o'))" | Add-Content -LiteralPath $res
Get-Content -LiteralPath $res
if ($code -ne 0) { throw "Model trial failed with exit $code; inspect output." }
