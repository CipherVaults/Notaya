param(
    [Parameter(Mandatory=$true)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_-]*$')][string]$L,
    [Parameter(Mandatory=$true)][string]$Prompt,
    [Parameter(Mandatory=$true)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_.-]*$')][string]$Tag,
    [string]$Model,
    [string]$Effort,
    [string]$ConfigPath = (Join-Path $PSScriptRoot 'config.ps1'),
    [switch]$DryRun
)
# One noninteractive lane. Started through WMI by heartbeat.ps1.
$ErrorActionPreference = 'Stop'
. $ConfigPath
if ($Cfg -isnot [hashtable]) { throw 'Config must define $Cfg as a hashtable.' }
if (-not $Model) { $Model = $Cfg.DefaultModel }
if (-not $Effort) { $Effort = $Cfg.DefaultEffort }
if ($Model -notmatch '^[A-Za-z0-9_.:-]+$' -or $Effort -notin 'minimal','low','medium','high','xhigh','max') { throw 'Invalid model or effort.' }
$wt = Join-Path $Cfg.Root $L
$testHome = Join-Path $Cfg.TestsDir $L
$log = Join-Path $Cfg.OutDir "$Tag.log"
# DryRun stops before directory creation, environment changes or process launch.
if ($DryRun) {
    Write-Output "DRYRUN run-lane: lane=$L model=$Model effort=$Effort worktree=$wt"
    Write-Output 'DRYRUN run-lane: no launch, no writes, no environment changes'
    return
}
if (-not (Test-Path -LiteralPath $wt -PathType Container)) { throw "Worktree missing: $wt" }
if (-not (Test-Path -LiteralPath $Prompt -PathType Leaf)) { throw "Prompt missing: $Prompt" }
if (-not (Test-Path -LiteralPath $Cfg.CodexCmd -PathType Leaf)) { throw 'Configured physical Codex wrapper not found.' }
foreach ($p in @($wt, $Cfg.OutDir, $testHome)) {
    $walk = [IO.Path]::GetFullPath($p)
    while ($walk) {
        if ((Test-Path -LiteralPath $walk) -and ((Get-Item -LiteralPath $walk -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw "Reparse point not allowed: $walk" }
        $walk = Split-Path -Parent $walk
    }
}
$env:Path = [Environment]::ExpandEnvironmentVariables(
    [Environment]::GetEnvironmentVariable('Path','Machine') + ';' +
    [Environment]::GetEnvironmentVariable('Path','User')) + ';' + ($Cfg.ExtraPath -join ';')
$env:CARGO_TARGET_DIR = $Cfg.CargoTargetDir
$env:CARGO_NET_OFFLINE = 'true'
foreach ($key in $Cfg.LaneEnvironment.Keys) {
    $value = ([string]$Cfg.LaneEnvironment[$key]).Replace('{TestsDir}', $Cfg.TestsDir).Replace('{lane}', $L).Replace('{Root}', $Cfg.Root)
    [Environment]::SetEnvironmentVariable($key, $value, 'Process')
}
New-Item -ItemType Directory -Force $testHome, $Cfg.OutDir, $Cfg.CargoTargetDir | Out-Null
$cliArgs = @('exec','-C',$wt,'-m',$Model,'-c',('model_reasoning_effort="' + $Effort + '"'),'-s','workspace-write')
foreach ($dir in @($Cfg.CargoTargetDir, $testHome, $Cfg.OutDir) + @($Cfg.AdditionalWriteDirs)) { $cliArgs += @('--add-dir', $dir) }
$cliArgs += @('-o', (Join-Path $Cfg.OutDir "$Tag.md"), '-')
# Use the physical wrapper from config: an MSIX-packaged app hides the npm
# folder in its virtual filesystem, so an alias visible in the app can fail
# in a scheduler/WMI process. The wrapper must be accessible to that process.
$exitCode = 1
try {
    Get-Content -LiteralPath $Prompt -Raw | & $Cfg.CodexCmd @cliArgs *> $log
    $exitCode = $LASTEXITCODE
} finally { "EXIT $exitCode" | Add-Content -LiteralPath $log }
exit $exitCode
