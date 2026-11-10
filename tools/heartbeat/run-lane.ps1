param([string]$L, [string]$Prompt, [string]$Tag, [string]$Model = 'gpt-6.1-sol', [string]$Effort = 'xhigh')
# Runs one Codex lane non-interactively. Lives outside the repo. Started by heartbeat.ps1.
$root = 'D:\Notaya-lanes'; $out = "$root\out"
$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
if (-not $env:USERPROFILE) { $env:USERPROFILE = 'C:\Users\Ahmed' }
$env:Path = [Environment]::ExpandEnvironmentVariables($env:Path) + ';C:\Users\Ahmed\AppData\Roaming\npm;C:\Users\Ahmed\.cargo\bin;C:\Users\Ahmed\.local\bin'
$env:CARGO_TARGET_DIR = "$root\_target"; $env:CARGO_NET_OFFLINE = 'true'
$env:NOTAYA_HOME = "D:\Notaya Tests\v1-6-$L"
$env:NOTAYA_PLAYWRIGHT_MODULE = 'D:\Notaya-lanes\tools\node_modules\playwright-core'; $env:NOTAYA_CHROMIUM_PATH = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
New-Item -ItemType Directory -Force $env:NOTAYA_HOME | Out-Null
$log = "$out\$Tag.log"
Get-Content $Prompt -Raw | & 'C:\Users\Ahmed\AppData\Local\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\npm\codex.ps1' exec -C "$root\$L" -m $Model -c ('model_reasoning_effort="' + $Effort + '"') -s workspace-write --add-dir "$root\_target" --add-dir $env:NOTAYA_HOME --add-dir "$env:USERPROFILE\.cargo" --add-dir "$root\out" --add-dir "$root\review-A" --add-dir "$root\review-C4" --add-dir "$root\review-C2" --add-dir "$root\review-C5" -o "$out\$Tag.md" - *> $log
"EXIT $LASTEXITCODE" | Add-Content $log








