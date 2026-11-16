param(
 [Parameter(Mandatory)][string]$TestPath,
 [Parameter(Mandatory)][string]$TestSHA256,
 [Parameter(Mandatory)][string]$BrowserPath,
 [Parameter(Mandatory)][string]$BrowserSHA256
)
$ErrorActionPreference='Stop'
$basePath='D:\Codex Projects\Notaya\resume-20261111T190954Z-ae1f50c8'
$controlPath=Join-Path $basePath 'control'
$hostPath='C:\Program Files\WindowsApps\OpenAI.CodexPrimaryRuntime.v26-1110-778-0_26.1110.778.0_x64__3k8sg7r9htsxt\dependencies\native\powershell\pwsh.exe'
if([Environment]::ProcessPath-cne$hostPath){throw 'Qualified host required'}
$sharedPath=Join-Path $controlPath 'p6c-b1-runnable-source-proposal-20261113T010000Z-0c01b3f8\B1.ProcessAndGuards.proposal.ps1'
if((Get-FileHash -LiteralPath $sharedPath -Algorithm SHA256).Hash-cne'3617482E005062CB304BCAF793033FA070047D749DBF3378EB05C391D750F30F'){throw 'Shared helper drift'}
. $sharedPath
$oldBrowser=Join-Path $controlPath 'q1-undo-frozen-generation-20261116T052338Z-f192a174\browser.Q1.undo-model-rebind.UNAPPROVED.v9.cjs'
$nodePath='C:\Program Files\nodejs\node.exe'
foreach($candidatePath in @($TestPath,$BrowserPath)){
 B1Need ([IO.Path]::IsPathFullyQualified($candidatePath)-and!$candidatePath.StartsWith('\\',[StringComparison]::Ordinal)) 'Local absolute source only'
 B1Need ([IO.Path]::GetFullPath($candidatePath).StartsWith($controlPath+'\',[StringComparison]::Ordinal)) 'Only owned CONTROL test/source paths'
}
$TestPath=[IO.Path]::GetFullPath($TestPath)
$BrowserPath=[IO.Path]::GetFullPath($BrowserPath)
$pins=@(
 @{path=$hostPath;hash='2A6769A3330DC1084D0154BFDE7D164AB750CA7F700F313F9D417C1BE78B2233'},
 @{path=$sharedPath;hash='3617482E005062CB304BCAF793033FA070047D749DBF3378EB05C391D750F30F'},
 @{path=$nodePath;hash='63C259C81E5D472B5F11C8D506070130CB04A1ECF84B80377A34ED6EC9048088'},
 @{path=$oldBrowser;hash='318F1E9C199D90AD2D833F6F51D1EBC0F937F98D94E41D0A3CEF00EFC657B51D'},
 @{path=(Join-Path $controlPath 'q1-inline-runtime-q1-observe-20261116T054753Z-f0bc7ed7\browser-evidence\q1-audit-00612.json');hash='2F519F6B17FC13C8FADBC3033829D172F2D2AD835C2A2A98EA0AFCE964E27240'},
 @{path=$BrowserPath;hash=$BrowserSHA256},
 @{path=$TestPath;hash=$TestSHA256}
)
$leases=[Collections.Generic.List[IO.FileStream]]::new()
$runPath=Join-Path 'D:\' ('nq1-status-unit-'+[DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
B1Need (!(Test-Path -LiteralPath $runPath)) 'Fresh output'
[void][IO.Directory]::CreateDirectory($runPath)
$result=$null;$launchRequested=$false;$knownClosed=$false
try{
 foreach($row in $pins){
  B1NoFollow $row.path
  $stream=[IO.File]::Open($row.path,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read)
  $leases.Add($stream)
  B1Need ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($stream))-ceq$row.hash) ('Pin drift '+$row.path)
 }
 $environment=B1Environment $runPath ([IO.Path]::GetDirectoryName($hostPath))
 $limits=[ordered]@{wallMilliseconds=120000;processCPUMilliseconds=60000;maximumObservedPeakCounterBytes=2147483648;stdoutBytes=1048576;stderrBytes=1048576;sampleIntervalMilliseconds=100;processExitGraceMilliseconds=5000;pipeDrainGraceMilliseconds=5000;pipeCancellationGraceMilliseconds=2000}
 $launchRequested=$true
 $result=B1BoundedProcess $nodePath @('--report-exclude-network','--report-exclude-env','--max-old-space-size=768',$TestPath,$BrowserPath) $runPath $environment (Join-Path $runPath 'logs') $limits
 $knownClosed=($null-ne$result-and$result.state.exitObserved-and$result.state.drainsCompleted-and$result.state.logsClosed-and$null-eq$result.state.unresolvedOwnedPID-and!$result.state.resourcesRetained)
 B1Need $knownClosed 'Known owned Node exit/readers required before source readback'
 foreach($row in $pins){B1Pin $row.path $row.hash}
}finally{
 if($launchRequested-and!$knownClosed){
  # Deliberate live retention: never return a false terminal state or release the
  # source leases after an uncertain child/pump result. Root observes this live
  # controller, verifies exact owned process/readers externally, then closes the
  # controller only after that closure evidence. No extra kill/retry is issued here.
  $global:Q1StatusRetainedSourceLeases=$leases
  try{B1Save (Join-Path $runPath 'Root-status-unit-resource-hold.json') ([ordered]@{
   scope='FAILED/UNCERTAIN owned unit resource closure; live controller retains source leases'
   recordedUTC=[DateTime]::UtcNow.ToString('o');controllerPID=$PID;process=$result
   selectedSourceLeases=$leases.Count;postRunSourceReadsRequested=$false;leasesReleased=$false
   automaticRetry=$false;manualKnownOwnedClosureRequiredBeforeControllerStop=$true
  })
  [Console]::Out.WriteLine('Q1_STATUS_RESOURCE_HOLD '+$runPath)
  [Console]::Out.Flush()}catch{$global:Q1StatusRetentionError=$_.Exception.Message}
  while($true){[Threading.Thread]::Sleep(500)}
 }
 foreach($lease in $leases){$lease.Dispose()}
}
B1Save (Join-Path $runPath 'Root-status-unit.json') ([ordered]@{
 scope='Pure exact-source Q1 status bookkeeping grader regressions, not app/UI'
 testPath=$TestPath;testSHA256=$TestSHA256;browserPath=$BrowserPath;browserSHA256=$BrowserSHA256
 oldBrowserPath=$oldBrowser;oldBrowserSHA256='318F1E9C199D90AD2D833F6F51D1EBC0F937F98D94E41D0A3CEF00EFC657B51D'
 selectedPinnedInputs=7;closedLeases=$leases.Count;process=$result
 noApplicationOrNetworkRequested=$true;browserLaunched=$false;productAccepted=$false
})
[ordered]@{runPath=$runPath;passed=$result.passed;state=$result.state;stdoutBytes=$result.stdoutBytes;stderrBytes=$result.stderrBytes}|ConvertTo-Json -Depth 5
if(!$result.passed){exit 1}
