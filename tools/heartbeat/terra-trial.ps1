# One-line trial of the Terra model at high effort (A54), run right after the Codex window reset (22:51:08Z) and before the heartbeat starts lanes (next tick about 23:00:56Z).
# No repo write: read-only sandbox, empty throwaway folder. Result in D:\Notaya-lanes\out\terra-trial.txt
$target = [DateTime]::Parse('2026-11-09T22:51:25Z').ToUniversalTime()
while ((Get-Date).ToUniversalTime() -lt $target) { Start-Sleep -Seconds 5 }
$out = 'D:\Notaya-lanes\out'; $res = "$out\terra-trial.txt"; $jl = "$out\terra-trial.jsonl"
New-Item -ItemType Directory -Force 'D:\Notaya Tests\codex-probe' | Out-Null
"START $((Get-Date).ToUniversalTime().ToString('o'))" | Set-Content $res
$cands = @('gpt-6.1-terra')
foreach ($m in $cands) {
  Push-Location 'D:\Notaya Tests\codex-probe'
  $env:Path = [Environment]::ExpandEnvironmentVariables([Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')) + ';C:\Users\Ahmed\AppData\Roaming\npm'
  cmd /c "`"C:\Users\Ahmed\AppData\Local\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\npm\codex.cmd`" exec --json -m $m -c model_reasoning_effort=`"high`" -s read-only --skip-git-repo-check `"Reply with ok.`" > `"$jl`" 2>&1"
  $code = $LASTEXITCODE; Pop-Location
  "MODEL TRIED $m exit=$code" | Add-Content $res
  $lines = Get-Content $jl -ErrorAction SilentlyContinue
  $errs = @($lines | Where-Object { $_ -match '"type":"error"|unknown model|not found|does not exist|invalid model|unsupported' }) | Select-Object -First 3
  foreach ($e in $errs) { "ERRLINE " + $e.Substring(0, [Math]::Min(300, $e.Length)) | Add-Content $res }
  $tc = @($lines | Where-Object { $_ -match '"token_count"' -and $_ -match '"primary"' })
  if ($tc.Count) {
    $first = $tc[0] | ConvertFrom-Json; $last = $tc[-1] | ConvertFrom-Json
    "METER after the trial: 5h {0}% (event {1}), weekly {2}%; first event 5h {3}%" -f $last.payload.rate_limits.primary.used_percent, $last.timestamp, $last.payload.rate_limits.secondary.used_percent, $first.payload.rate_limits.primary.used_percent | Add-Content $res
    $u = $last.payload.info.total_token_usage; if ($u) { "TOKENS input {0} cached {1} output {2} reasoning {3} total {4}" -f $u.input_tokens, $u.cached_input_tokens, $u.output_tokens, $u.reasoning_output_tokens, $u.total_tokens | Add-Content $res }
  } else { "no token_count event with rate limits found" | Add-Content $res }
  $modelSeen = @($lines | Where-Object { $_ -match '"model"\s*:\s*"[^"]+"' } | ForEach-Object { [regex]::Match($_, '"model"\s*:\s*"([^"]+)"').Groups[1].Value } | Select-Object -Unique) -join ','
  "MODEL ids seen in the event stream: $modelSeen" | Add-Content $res
  "REPLY: " + (@($lines | Where-Object { $_ -match '"agent_message"|"item.completed"' } | Select-Object -Last 1) -join '').Substring(0, [Math]::Min(200, (@($lines | Where-Object { $_ -match '"agent_message"|"item.completed"' } | Select-Object -Last 1) -join '').Length)) | Add-Content $res
}
"DONE $((Get-Date).ToUniversalTime().ToString('o'))" | Add-Content $res
