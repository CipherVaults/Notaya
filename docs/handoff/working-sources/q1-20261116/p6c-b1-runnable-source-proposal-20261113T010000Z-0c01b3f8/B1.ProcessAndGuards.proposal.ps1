# Shared finite process runner adapted from reviewed Root source15522A/EA53; not independently approved.
function B1Need([bool]$value,[string]$message){if(!$value){throw $message}}
function B1NoFollow([string]$path){
 $full=[IO.Path]::GetFullPath($path);B1Need ($full-ceq$path-and$path-match'^[A-Za-z]:\\'-and$path.IndexOf(':',2)-lt0) 'Canonical local path required'
 $at=[IO.Path]::GetPathRoot($full)
 foreach($part in $full.Substring($at.Length).Split('\',[StringSplitOptions]::RemoveEmptyEntries)){$at=[IO.Path]::Combine($at,$part);B1Need (([IO.File]::GetAttributes($at)-band([IO.FileAttributes]::ReparsePoint-bor[IO.FileAttributes]::Encrypted-bor[IO.FileAttributes]::Offline))-eq0) 'No-follow path hold'}
}
function B1Hash([string]$path){B1NoFollow $path;$stream=[IO.File]::Open($path,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read);try{return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($stream))}finally{$stream.Dispose()}}
function B1Pin([string]$path,[string]$expected){B1Need ($expected-cmatch'^[A-F0-9]{64}$'-and(B1Hash $path)-ceq$expected) ('Exact pin drift '+$path)}
function B1NewText([string]$path,[string]$text){$bytes=[Text.UTF8Encoding]::new($false).GetBytes($text);$stream=[IO.File]::Open($path,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read);try{$stream.Write($bytes);$stream.Flush($true)}finally{$stream.Dispose()}}
function B1Save([string]$path,$value){B1NewText $path ($value|ConvertTo-Json -Depth 30)}
function B1Read([string]$path,[string]$hash){B1Pin $path $hash;return [IO.File]::ReadAllText($path)|ConvertFrom-Json}
function B1Environment([string]$run,[string]$hostDirectory){
 $envMap=[ordered]@{SystemRoot='C:\Windows';WINDIR='C:\Windows';SystemDrive='C:';ComSpec='C:\Windows\System32\cmd.exe';OS='Windows_NT';PATH=($hostDirectory+';C:\Windows\System32');PSModulePath=([IO.Path]::Combine($hostDirectory,'Modules'));TEMP=(Join-Path $run 'temp');TMP=(Join-Path $run 'temp');USERPROFILE=(Join-Path $run 'home');HOME=(Join-Path $run 'home');APPDATA=(Join-Path $run 'appdata');LOCALAPPDATA=(Join-Path $run 'localappdata');XDG_CACHE_HOME=(Join-Path $run 'cache');XDG_CONFIG_HOME=(Join-Path $run 'config');XDG_DATA_HOME=(Join-Path $run 'data');NOTAYA_HOME=(Join-Path $run 'notaya-home');CIPHERLEAF_HOME=(Join-Path $run 'cipherleaf-home');NODE_DISABLE_COMPILE_CACHE='1';WS_NO_BUFFER_UTIL='1';WS_NO_UTF_8_VALIDATE='1';PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD='1';PLAYWRIGHT_BROWSERS_PATH=(Join-Path $run 'browser-cache');NAPI_RS_ENFORCE_VERSION_CHECK='1';POWERSHELL_TELEMETRY_OPTOUT='1';DOTNET_CLI_TELEMETRY_OPTOUT='1';DOTNET_NOLOGO='1'}
 foreach($relative in @('cwd','temp','home','appdata','localappdata','cache','config','data','notaya-home','cipherleaf-home','browser-cache','logs')){[IO.Directory]::CreateDirectory((Join-Path $run $relative))|Out-Null}
 return $envMap
}
function B1BoundedProcess([string]$image,[string[]]$arguments,[string]$cwd,$environment,[string]$outputRoot,$limits){
 $state=[ordered]@{started=$false;pid=$null;startTimeUtc=$null;exitCode=$null;exitObserved=$false;terminationAttempted=$false;unresolvedOwnedPID=$null;drainsCompleted=$false;cancelRequested=$false;logsClosed=$false;resourcesRetained=$false;stopReason=$null;failure=$null}
 function AddFailure([string]$message){if($state.failure){$state.failure+='; '+$message}else{$state.failure=$message}}
 $stdout=Join-Path $outputRoot 'stdout.bin';$stderr=Join-Path $outputRoot 'stderr.bin'
 $outStream=[IO.File]::Open($stdout,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 $errStream=[IO.File]::Open($stderr,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 $psi=[Diagnostics.ProcessStartInfo]::new();$psi.FileName=$image;$psi.WorkingDirectory=$cwd;$psi.UseShellExecute=$false;$psi.CreateNoWindow=$true;$psi.RedirectStandardInput=$true;$psi.RedirectStandardOutput=$true;$psi.RedirectStandardError=$true;$psi.Environment.Clear()
 foreach($key in $environment.Keys){$psi.Environment[$key]=[string]$environment[$key]}
 foreach($argument in $arguments){$psi.ArgumentList.Add($argument)}
 $process=[Diagnostics.Process]::new();$process.StartInfo=$psi;$cancel=[Threading.CancellationTokenSource]::new()
 $cancel.CancelAfter([int]($limits.wallMilliseconds+$limits.processExitGraceMilliseconds+$limits.pipeDrainGraceMilliseconds+$limits.pipeCancellationGraceMilliseconds))
 $outCopy=$null;$errCopy=$null;$samples=[Collections.Generic.List[object]]::new();$sampleErrors=[Collections.Generic.List[string]]::new();$cpu=$null;$peak=[long]0;$watch=[Diagnostics.Stopwatch]::StartNew()
 try{
  B1Need ($process.Start()) 'Owned process failed to start';$state.started=$true;$state.pid=$process.Id;$state.startTimeUtc=$process.StartTime.ToUniversalTime().ToString('o');$process.StandardInput.Close()
  B1Save (Join-Path $outputRoot 'owned-process-start.json') ([ordered]@{image=$image;arguments=$arguments;cwd=$cwd;pid=$state.pid;startTimeUtc=$state.startTimeUtc;observedUtc=[DateTime]::UtcNow.ToString('o')})
  $outCopy=$process.StandardOutput.BaseStream.CopyToAsync($outStream,81920,$cancel.Token);$errCopy=$process.StandardError.BaseStream.CopyToAsync($errStream,81920,$cancel.Token)
  while(!$process.HasExited){
   try{$process.Refresh();$cpu=$process.TotalProcessorTime.TotalMilliseconds;$peak=[Math]::Max($peak,$process.PeakWorkingSet64);$samples.Add([pscustomobject]@{wallMilliseconds=$watch.Elapsed.TotalMilliseconds;cpuMilliseconds=$cpu;observedOSPeakCounterBytes=$peak})}catch{if(!$process.HasExited){if($sampleErrors.Count-lt16){$sampleErrors.Add($_.Exception.Message)};$state.stopReason='resource-observation-failed'}}
   if($watch.Elapsed.TotalMilliseconds-ge$limits.wallMilliseconds){$state.stopReason='wall-budget'}elseif($null-ne$cpu-and$cpu-ge$limits.processCPUMilliseconds){$state.stopReason='CPU-budget'}elseif($peak-ge$limits.maximumObservedPeakCounterBytes){$state.stopReason='observed-peak-budget'}elseif(([IO.FileInfo]::new($stdout)).Length-gt$limits.stdoutBytes-or([IO.FileInfo]::new($stderr)).Length-gt$limits.stderrBytes){$state.stopReason='sampled-output-budget'}
   if($state.stopReason){break};[Threading.Thread]::Sleep([int]$limits.sampleIntervalMilliseconds)
  }
 }catch{AddFailure $_.Exception.ToString()}
 finally{
  if($state.started){
   try{
    if(!$process.HasExited){$state.terminationAttempted=$true;try{$process.Kill($true)}catch{AddFailure ('owned termination: '+$_.Exception.Message)};if(!$process.WaitForExit([int]$limits.processExitGraceMilliseconds)){$state.unresolvedOwnedPID=$state.pid;if(!$state.stopReason){$state.stopReason='exit-grace-expired'}}}
    if($process.HasExited){$state.exitObserved=$true;$state.exitCode=$process.ExitCode;$state.unresolvedOwnedPID=$null}
   }catch{AddFailure ('exit observation: '+$_.Exception.Message);$state.unresolvedOwnedPID=$state.pid}
  }
  $copies=@();foreach($copy in @($outCopy,$errCopy)){if($null-ne$copy){$copies+=$copy}}
  if($copies.Count){
   $all=[Threading.Tasks.Task]::WhenAll([Threading.Tasks.Task[]]$copies)
   try{$state.drainsCompleted=$all.Wait([int]$limits.pipeDrainGraceMilliseconds)}catch{AddFailure ('pipe drain: '+$_.Exception.Message);$state.drainsCompleted=$all.IsCompleted}
   if(!$state.drainsCompleted){$state.cancelRequested=$true;$cancel.Cancel();if(!$state.stopReason){$state.stopReason='pipe-drain-grace'};try{$state.drainsCompleted=$all.Wait([int]$limits.pipeCancellationGraceMilliseconds)}catch{AddFailure ('pipe cancellation: '+$_.Exception.Message);$state.drainsCompleted=$all.IsCompleted}}
   foreach($copy in $copies){if($copy.IsCompleted){try{[void]$copy.GetAwaiter().GetResult()}catch{AddFailure ('completed pump: '+$_.Exception.Message)}}else{AddFailure 'Unresolved owned pump'}}
  }else{$state.drainsCompleted=$true}
  if($state.drainsCompleted){
   $disposed=$true
   foreach($stream in @($outStream,$errStream)){try{$stream.Flush($true)}catch{AddFailure ('log flush: '+$_.Exception.Message)}finally{try{$stream.Dispose()}catch{$disposed=$false;AddFailure ('log disposal: '+$_.Exception.Message)}}}
   $state.logsClosed=$disposed
   try{$cancel.Dispose();$process.Dispose()}catch{AddFailure ('completed resource disposal: '+$_.Exception.Message)}
  }else{$state.resourcesRetained=$true;$script:B1UnsettledResources=@($process,$outStream,$errStream,$cancel,$outCopy,$errCopy);AddFailure 'Pending resources retained until owned controller exit; no active Flush/Dispose/hash'}
  $watch.Stop()
 }
 $outBytes=([IO.FileInfo]::new($stdout)).Length;$errBytes=([IO.FileInfo]::new($stderr)).Length
 if($outBytes-gt$limits.stdoutBytes-or$errBytes-gt$limits.stderrBytes){$state.stopReason='closed-output-budget'}
 $samplePath=Join-Path $outputRoot 'resource-samples.json';B1Save $samplePath $samples.ToArray()
 $record=[ordered]@{scope='p6c-b1-bounded-owned-process-v1';passed=($state.started-and$state.exitObserved-and$state.drainsCompleted-and$state.logsClosed-and$state.exitCode-eq0-and!$state.stopReason-and!$state.failure);state=$state;image=$image;arguments=$arguments;cwd=$cwd;environment=$environment;wallMilliseconds=$watch.Elapsed.TotalMilliseconds;lastObservedCPUMilliseconds=$cpu;maximumObservedPeakBytes=$peak;sampleCount=$samples.Count;sampleErrors=$sampleErrors.ToArray();limits=$limits;stdoutPath=$stdout;stdoutBytes=$outBytes;stdoutSHA256=$(if($state.logsClosed){B1Hash $stdout}else{$null});stderrPath=$stderr;stderrBytes=$errBytes;stderrSHA256=$(if($state.logsClosed){B1Hash $stderr}else{$null});resourceSamplesPath=$samplePath;resourceSamplesSHA256=(B1Hash $samplePath);descendantLimit='Immediate owned PID exit observed. Kill(true) is one bounded owned-tree termination attempt, not an exhaustive descendant/process or native-I/O attestation.'}
 B1Save (Join-Path $outputRoot 'process-summary.json') $record
 return [pscustomobject]$record
}
