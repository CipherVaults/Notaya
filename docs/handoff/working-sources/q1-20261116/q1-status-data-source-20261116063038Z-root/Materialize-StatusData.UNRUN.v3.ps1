param([Parameter(Mandatory)][string]$newBrowserPath,[Parameter(Mandatory)][string]$newBrowserSHA256,[Parameter(Mandatory)][string]$newReviewPath,[Parameter(Mandatory)][string]$newReviewSHA256)

$ErrorActionPreference='Stop'
$c='D:\Codex Projects\Notaya\resume-20261111T190954Z-ae1f50c8\control'
$prior=Join-Path $c 'q1-undo-ready-20261116T053948Z-333d3ee5'
function ControlFile([string]$file,[long]$maximum=2097152){
 if(-not [IO.Path]::IsPathFullyQualified($file) -or $file.StartsWith('\\',[StringComparison]::Ordinal)){throw 'Local absolute CONTROL file only'}
 $full=[IO.Path]::GetFullPath($file)
 if(-not $full.StartsWith($c+'\',[StringComparison]::Ordinal)){throw 'Outside owned CONTROL'}
 $pathRoot=[IO.Path]::GetPathRoot($full);$part=$pathRoot
 foreach($segment in $full.Substring($pathRoot.Length).Split([IO.Path]::DirectorySeparatorChar)){
  $part=Join-Path $part $segment
  $item=Get-Item -LiteralPath $part -Force
  if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw ('Reparse path held: '+$part)}
  if($part -cne $full -and -not $item.PSIsContainer){throw 'Non-directory ancestor'}
 }
 if($item.PSIsContainer -or $item.Length -le 0 -or $item.Length -gt $maximum){throw 'Ordinary bounded nonempty file required'}
 $full
}
function CaptureControlFile([string]$file,[string]$sha,[long]$maximum=2097152){
 $file=ControlFile $file $maximum
 $s=[IO.File]::Open($file,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read)
 try{
  if($s.Length -le 0 -or $s.Length -gt $maximum){throw 'Captured file exceeds role bounds'}
  $bytes=[byte[]]::new([int]$s.Length);$s.ReadExactly($bytes)
  if([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)) -cne $sha){throw ('Captured content drift '+$file)}
 }finally{$s.Dispose()}
 [pscustomobject]@{path=$file;sha256=$sha;bytes=$bytes}
}
function ReadData([string]$file,[string]$sha){
 $capture=CaptureControlFile $file $sha
 [Text.UTF8Encoding]::new($false,$true).GetString($capture.bytes)|ConvertFrom-Json -DateKind String
}
function Info([string]$file,[string]$sha,[long]$maximum=2097152){
 $capture=CaptureControlFile $file $sha $maximum
 [pscustomobject]@{path=$capture.path;sha256=$capture.sha256;bytes=$capture.bytes.Length}
}
$p=ReadData (Join-Path $prior 'product.undo-worktree.ready.v8.json') 'B9FCAD5675F318848BA13BC9DDFFC92E603A9A41151401DDC0B4E5CD713C30E5'
$r=ReadData (Join-Path $prior 'runtime.undo-worktree.ready.v8.json') 'F8E00F2F9FA6537897B4FDAE693B10FD194DC9689040628D96ACFEFE97A3F133'
$b=ReadData (Join-Path $prior 'budget.undo-worktree.ready.v8.json') '71C761EA4D845B8884B0BEA01C4B5D94D5EE0E5123ECBD2AF618FA581B29EA4F'
$new=Info $newBrowserPath $newBrowserSHA256 131072
$review=Info $newReviewPath $newReviewSHA256
$reviewData=ReadData $review.path $review.sha256
if($reviewData.scope -isnot [string] -or $reviewData.scope -cne 'q1-inline-runtime-CJS-source-review-v1' -or $reviewData.approved -isnot [bool] -or $reviewData.approved -ne $true -or $reviewData.nonAuthor -isnot [bool] -or $reviewData.nonAuthor -ne $true -or $reviewData.executionApproved -isnot [bool] -or $reviewData.executionApproved -ne $false -or $reviewData.browserPath -isnot [string] -or $reviewData.browserPath -cne $new.path -or $reviewData.browserSHA256 -cne $new.sha256){throw 'Unqualified source review'}
foreach($role in @('policy','server','coordinator','observer')){
 if($reviewData.($role+'SHA256') -cne $p.($role+'SHA256')){throw ('Unchanged role review mismatch '+$role)}
}
if($reviewData.Q1BindingDataSHA256 -cne $p.Q1BindingDataSHA256){throw 'Binding identity mismatch'}
$oldPath=$p.browserPath;$oldHash=$p.browserSHA256
if($oldHash -cne '318F1E9C199D90AD2D833F6F51D1EBC0F937F98D94E41D0A3CEF00EFC657B51D'){throw 'Old browser identity'}
$catalogue=@($p.CJSExpectedSources|Where-Object{$_.path -ceq $oldPath})
$syntax=@($b.syntax.sources|Where-Object{$_.path -ceq $oldPath})
$targets=@($b.targets|Where-Object{$_.path -ceq $oldPath})
if($catalogue.Count-ne1-or$syntax.Count-ne1-or$targets.Count-ne1){throw 'Exact active browser row associations'}
foreach($row in @($catalogue[0],$syntax[0],$targets[0])){
 if($row.sha256-cne$oldHash-or[long]$row.bytes-ne46696){throw 'Old source byte/hash identity'}
 $row.path=$new.path;$row.sha256=$new.sha256;$row.bytes=[long]$new.bytes
}
$delta=[long]$new.bytes-46696
$catalogue[0].provenance='Independently reviewed status-dispatch/capture bookkeeping successor; same actor/oracle/deadline/resource budgets and full source entries.'
$p.browserPath=$new.path;$p.browserSHA256=$new.sha256
$p.CJSMaximumFreshPayloadBytes=[long]$p.CJSMaximumFreshPayloadBytes+$delta
$r.CJSSourceReviewPath=$review.path;$r.CJSSourceReviewSHA256=$review.sha256
$r.syntaxFreshBytes=[long]$r.syntaxFreshBytes+$delta
$r.runtimeFreshBytes=[long]$r.runtimeFreshBytes+$delta
$b.summary.runtimeFreshBytes=[long]$b.summary.runtimeFreshBytes+$delta
$b.summary.maximumFreshBytesAfterBoundedGeneratedInput=[long]$b.summary.maximumFreshBytesAfterBoundedGeneratedInput+$delta
$b.summary.CJSExplicitFreshBytes=[long]$b.summary.CJSExplicitFreshBytes+$delta
$b.syntax.freshSourceBytes=[long]$b.syntax.freshSourceBytes+$delta
$b.counts.freshBytes=[long]$b.counts.freshBytes+$delta
$b.CJS.maximumNodeExplicitCONTROLAndPWBytes=[long]$b.CJS.maximumNodeExplicitCONTROLAndPWBytes+$delta
$b.exactDeltaFromDBB3.newFreshBytes=[long]$b.exactDeltaFromDBB3.newFreshBytes+$delta
$b.independentSourceReviews.CJS=$review
$b.independentlyReviewed=$false
if($b.CJS.maximumNodeExplicitCONTROLAndPWBytes-gt$b.CJS.maximumAllowedNodeExplicitBytes){throw 'Existing CJS budget exceeded'}
if($catalogue[0].bytes-gt131072){throw 'Existing source read cap exceeded'}
if(@($p.CJSExpectedSources).Count-ne137-or@($b.targets).Count-ne264-or@($b.syntax.sources).Count-ne5){throw 'Original target counts changed'}
$lineage=[pscustomobject]@{
 kind='Status call bookkeeping correction, not a larger arbitrary count tolerance'
 priorProductSHA256='B9FCAD5675F318848BA13BC9DDFFC92E603A9A41151401DDC0B4E5CD713C30E5'
 priorRuntimeSHA256='F8E00F2F9FA6537897B4FDAE693B10FD194DC9689040628D96ACFEFE97A3F133'
 priorBudgetSHA256='71C761EA4D845B8884B0BEA01C4B5D94D5EE0E5123ECBD2AF618FA581B29EA4F'
 priorBrowserPath=$oldPath;priorBrowserSHA256=$oldHash;browser=$new;sourceReview=$review
 controlByteDelta=$delta;currentApplicationModelUnchanged=$true
 modelSHA256='0E1A53718E717DD418D2AC06D862D37B8C2C55E635F0E2391548B67D972C185D'
 bindingSHA256='160F45A6DCC507E68769A04A93345BFE61968646D4C0D75386E66FBA6D604AAC'
 allProductActorsAndNinePhasesUnchanged=$true;resourceCapsUnchanged=$true
 sourceReviewOnly=$true;functionalExecutionQualified=$false
}
foreach($obj in @($p,$r,$b)){$obj|Add-Member -NotePropertyName StatusBookkeepingSuccessor -NotePropertyValue $lineage}
$d=Join-Path $c ('q1-status-ready-'+[DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
if(Test-Path -LiteralPath $d){throw 'Fresh no-overwrite output'}
[void][IO.Directory]::CreateDirectory($d)
function Save([string]$name,$obj){
 $file=Join-Path $d $name
 $bytes=[Text.UTF8Encoding]::new($false).GetBytes(($obj|ConvertTo-Json -Depth 100))
 $s=[IO.File]::Open($file,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{$s.Write($bytes);$s.Flush($true)}finally{$s.Dispose()}
 [pscustomobject]@{path=$file;sha256=(Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash;bytes=$bytes.Length}
}
$product=Save 'product.status-bookkeeping.ready.v1.json' $p
$r.productManifestPath=$product.path;$r.productManifestSHA256=$product.sha256
$runtime=Save 'runtime.status-bookkeeping.ready.v1.json' $r
$b.product=$product;$b.runtime=$runtime
$budget=Save 'budget.status-bookkeeping.ready.v1.json' $b
$expected=[ordered]@{scope='Root status bookkeeping data successor; independent exact data readback required';recordedUTC=[DateTime]::UtcNow.ToString('o');folder=$d;lineage=$lineage;product=$product;runtime=$runtime;budget=$budget;runtimeTargets=264;freshTargets=232;historicalTargets=32;CJSCount=137;syntaxLaunches=5;runtimeFreshBytes=$r.runtimeFreshBytes;syntaxFreshBytes=$r.syntaxFreshBytes;maximumFreshAfterBoundedInput=$b.summary.maximumFreshBytesAfterBoundedGeneratedInput;readyDataReviewPending=$true;executionApproved=$false}
$joins=Save 'Root-current-status-source-expected-data.v1.json' $expected
[pscustomobject]@{folder=$d;product=$product;runtime=$runtime;budget=$budget;joins=$joins;delta=$delta}|ConvertTo-Json -Depth 6
