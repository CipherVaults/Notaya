# Pure startup-entry policy fixtures: registry adapters are replaced before any call.
param([string]$GuardSource = (Join-Path $PSScriptRoot 'startup-entry-guard.ps1'))
$ErrorActionPreference = 'Stop'
. $GuardSource
$script:passed = 0; $script:deletes = 0; $script:readError = ''; $script:views = @()
$testExe = 'D:\synthetic-startup-guard\Notaya.exe'
$expected = '"' + $testExe + '" --minimized'
function Expect([bool]$Ok, [string]$Name) { if (!$Ok) { throw "FAIL $Name" }; $script:passed++; "PASS $Name" }
function Refuses([scriptblock]$Action, [string]$Name) {
  $before = $script:deletes; $failed = $false
  try { & $Action | Out-Null } catch { $failed = $true }
  Expect ($failed -and $script:deletes -eq $before) $Name
}
function Absent { [pscustomobject]@{ Present = $false; Kind = $null; Data = $null } }
function Entry($Data, $Kind = [Microsoft.Win32.RegistryValueKind]::String) { [pscustomobject]@{ Present = $true; Kind = $Kind; Data = $Data } }
function Reset($Entry64 = (Absent), $Entry32 = (Absent)) { $script:entries = @{ Registry64 = $Entry64; Registry32 = $Entry32 }; $script:readError = ''; $script:views = @() }
function Assert-OwnedTestFolder([string]$Path) { if ($Path -cne 'D:\synthetic-startup-guard') { throw 'Synthetic ownership refused' } }
function Read-NotayaStartupEntry([Microsoft.Win32.RegistryView]$View) {
  $script:views += $View.ToString()
  if ($script:readError -ceq $View.ToString()) { throw 'Synthetic registry read failure' }
  return $script:entries[$View.ToString()]
}
function Remove-NotayaStartupValue([string]$ExpectedCommand) {
  $script:deletes++
  $script:entries.Registry64 = Absent
  if ($script:entries.Registry32.Data -ceq $ExpectedCommand) { $script:entries.Registry32 = Absent }
}
Reset
Assert-NotayaStartupAbsent
Expect (($script:views -join ',') -ceq 'Registry64,Registry32') 'absence checks both explicit views'
foreach ($preexisting in @((Entry ''), (Entry $expected), (Entry 'foreign'))) {
  Reset $preexisting
  Refuses { Assert-NotayaStartupAbsent } 'any pre-existing named value prevents on, including empty data'
}
Reset (Absent) (Entry '')
Refuses { Assert-NotayaStartupAbsent } 'a 32-bit empty named entry also prevents on'
foreach ($view in @('Registry64','Registry32')) {
  Reset; $script:readError = $view
  Refuses { Assert-NotayaStartupAbsent } ($view + ' unavailable is not absence')
}
Reset (Entry $expected)
Assert-NotayaStartupOwned $testExe
Expect $true 'exact 64-bit REG_SZ command with absent 32-bit view is owned'
Reset (Entry $expected) (Entry $expected)
Assert-NotayaStartupOwned $testExe
Expect $true 'exact mirrored command is owned'
Reset (Entry $expected)
Refuses { Remove-NotayaTestStartupEntry $testExe $false } 'cleanup cannot delete without an attempted-on flag'
Reset
$before = $script:deletes; Remove-NotayaTestStartupEntry $null $false
Expect ($script:deletes -eq $before) 'cleanup without on preserves already-absent state'
foreach ($mirror in @($false,$true)) {
  Reset (Entry $expected) $(if ($mirror) { Entry $expected } else { Absent })
  $before = $script:deletes; Remove-NotayaTestStartupEntry $testExe $true
  Expect ($script:deletes -eq $before + 1 -and !$script:entries.Registry64.Present -and !$script:entries.Registry32.Present) 'attempted-on exact owned entry is removed and absence confirmed'
}
Reset
$before = $script:deletes; Remove-NotayaTestStartupEntry $testExe $true
Expect ($script:deletes -eq $before) 'already-removed test entry needs no deletion'
foreach ($bad in @(
  @{ name = 'empty'; entry = (Entry '') },
  @{ name = 'foreign prefix'; entry = (Entry ('prefix ' + $expected)) },
  @{ name = 'extra suffix'; entry = (Entry ($expected + ' --unexpected')) },
  @{ name = 'different exe'; entry = (Entry '"D:\foreign\Notaya.exe" --minimized') },
  @{ name = 'unquoted'; entry = (Entry ($testExe + ' --minimized')) },
  @{ name = 'ExpandString'; entry = (Entry $expected ([Microsoft.Win32.RegistryValueKind]::ExpandString)) },
  @{ name = 'DWord'; entry = (Entry 7 ([Microsoft.Win32.RegistryValueKind]::DWord)) },
  @{ name = 'untyped kind'; entry = (Entry $expected 'String') },
  @{ name = 'unknown presence'; entry = ([pscustomobject]@{ Present = $null; Kind = $null; Data = $null }) },
  @{ name = 'NaN presence'; entry = ([pscustomobject]@{ Present = [double]::NaN; Kind = $null; Data = $null }) },
  @{ name = 'Infinity data'; entry = (Entry ([double]::PositiveInfinity)) }
)) {
  Reset $bad.entry
  Refuses { Remove-NotayaTestStartupEntry $testExe $true } ($bad.name + ' entry is preserved during cleanup')
}
Reset (Entry $expected) (Entry 'foreign')
Refuses { Remove-NotayaTestStartupEntry $testExe $true } 'unexpected 32-bit entry prevents deletion of either view'
foreach ($view in @('Registry64','Registry32')) {
  Reset (Entry $expected); $script:readError = $view
  Refuses { Remove-NotayaTestStartupEntry $testExe $true } ($view + ' cleanup read error preserves all entries')
}
Reset (Absent) (Entry $expected)
Refuses { Assert-NotayaStartupAbsent } 'restoration fails if only the 32-bit entry remains'
# Exercise the actual name/kind/no-expansion reader with a fake key, never the OS.
$key = [pscustomobject]@{ Names = @('OtherName','nOtAyA'); Kind = [Microsoft.Win32.RegistryValueKind]::String; Data = ''; Reads = 0; Options = $null }
$key | Add-Member ScriptMethod GetValueNames { return $this.Names }
$key | Add-Member ScriptMethod GetValueKind { param($Name) if ($Name -cne 'nOtAyA') { throw 'Wrong value name' }; return $this.Kind }
$key | Add-Member ScriptMethod GetValue { param($Name,$Default,$Options) $this.Reads++; $this.Options = $Options; return $this.Data }
$entry = Get-NotayaStartupEntryFromKey $key
Expect ($entry.Present -and $entry.Data -ceq '' -and $key.Reads -eq 1 -and $key.Options -eq [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames) 'exact name ignores case and empty data is read without expansion'
$key.Names = @('NotayaSuffix','PrefixNotaya'); $key.Reads = 0
$entry = Get-NotayaStartupEntryFromKey $key
Expect (!$entry.Present -and $key.Reads -eq 0) 'other value names are neither matched nor read'
$key.Names = @($null)
Refuses { Get-NotayaStartupEntryFromKey $key } 'unknown value-name metadata fails closed'
# Exercise the exact production writable-key boundary, including its second ownership read.
$key | Add-Member NoteProperty Deletes 0
$key | Add-Member NoteProperty DeletedName ''
$key | Add-Member NoteProperty ThrowIfMissing $false
$key | Add-Member ScriptMethod DeleteValue { param($Name,$ThrowIfMissing) $this.Deletes++; $this.DeletedName = $Name; $this.ThrowIfMissing = $ThrowIfMissing }
$key.Names = @('nOtAyA'); $key.Kind = [Microsoft.Win32.RegistryValueKind]::String; $key.Data = $expected
Remove-NotayaStartupValueFromKey $key $expected
Expect ($key.Deletes -eq 1 -and $key.DeletedName -ceq 'Notaya' -and $key.ThrowIfMissing -ceq $true) 'actual writable-key boundary deletes the exact owned name once'
foreach ($case in @(
  @{name='foreign prefix'; names=@('nOtAyA'); kind=[Microsoft.Win32.RegistryValueKind]::String; data=('prefix '+$expected)},
  @{name='extra suffix'; names=@('nOtAyA'); kind=[Microsoft.Win32.RegistryValueKind]::String; data=($expected+' --unexpected')},
  @{name='empty'; names=@('nOtAyA'); kind=[Microsoft.Win32.RegistryValueKind]::String; data=''},
  @{name='ExpandString'; names=@('nOtAyA'); kind=[Microsoft.Win32.RegistryValueKind]::ExpandString; data=$expected},
  @{name='absent'; names=@(); kind=[Microsoft.Win32.RegistryValueKind]::String; data=$expected}
)) {
  $key.Names=$case.names; $key.Kind=$case.kind; $key.Data=$case.data
  $before=$key.Deletes; $failed=$false
  try { Remove-NotayaStartupValueFromKey $key $expected } catch { $failed=$true }
  Expect ($failed -and $key.Deletes -eq $before) ('actual writable-key boundary preserves '+$case.name)
}
$key.Names=@('unexpected duplicate','Notaya','nOtAyA'); $before=$key.Deletes; $failed=$false
try { Remove-NotayaStartupValueFromKey $key $expected } catch { $failed=$true }
Expect ($failed -and $key.Deletes -eq $before) 'actual writable-key revalidation error cannot delete'
"ALL OK ($script:passed startup-entry guard assertions; synthetic only)"
