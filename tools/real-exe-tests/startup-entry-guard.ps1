# Test-only exact HKCU Run\Notaya guard. Importing this file performs no registry access.
function Get-NotayaStartupEntryFromKey($Key) {
  $allNames = @($Key.GetValueNames())
  if (@($allNames | Where-Object { $_ -isnot [string] }).Count) { throw 'Startup-entry names are unknown' }
  $names = @($allNames | Where-Object { [string]::Equals($_, 'Notaya', [StringComparison]::OrdinalIgnoreCase) })
  if ($names.Count -gt 1) { throw 'Ambiguous Notaya startup-entry name' }
  if (!$names.Count) { return [pscustomobject]@{ Present = $false; Kind = $null; Data = $null } }
  return [pscustomobject]@{
    Present = $true
    Kind = $Key.GetValueKind($names[0])
    Data = $Key.GetValue($names[0], $null, [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
  }
}
function Read-NotayaStartupEntry([Microsoft.Win32.RegistryView]$View) {
  $base = $null; $key = $null
  try {
    $base = [Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::CurrentUser, $View)
    $key = $base.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Run', $false)
    if ($null -eq $key) { throw 'Run key unavailable' }
    Get-NotayaStartupEntryFromKey $key
  } catch { throw 'Cannot establish the Notaya startup-entry state' }
  finally { if ($null -ne $key) { $key.Dispose() }; if ($null -ne $base) { $base.Dispose() } }
}
function Get-NotayaStartupCommand([string]$TestExe) {
  if (!$TestExe -or $TestExe -notmatch '^[A-Za-z]:\\' -or $TestExe -match '["\r\n]' -or [IO.Path]::GetFullPath($TestExe) -cne $TestExe -or [IO.Path]::GetFileName($TestExe) -cne 'Notaya.exe') { throw 'Canonical owned test executable required' }
  Assert-OwnedTestFolder ([IO.Path]::GetDirectoryName($TestExe))
  return '"' + $TestExe + '" --minimized'
}
function Assert-NotayaStartupAbsent {
  foreach ($view in @([Microsoft.Win32.RegistryView]::Registry64, [Microsoft.Win32.RegistryView]::Registry32)) {
    $entry = Read-NotayaStartupEntry $view
    if ($null -eq $entry -or $entry.Present -isnot [bool] -or $entry.Present) { throw 'Notaya startup entry exists or its absence is unknown; refusing to change it' }
  }
}
function Assert-NotayaStartupOwned([string]$TestExe) {
  $expected = Get-NotayaStartupCommand $TestExe
  foreach ($view in @([Microsoft.Win32.RegistryView]::Registry64, [Microsoft.Win32.RegistryView]::Registry32)) {
    $entry = Read-NotayaStartupEntry $view
    if ($null -eq $entry -or $entry.Present -isnot [bool]) { throw 'Notaya startup-entry ownership is unknown' }
    if ($view -eq [Microsoft.Win32.RegistryView]::Registry32 -and !$entry.Present) { continue }
    if (!$entry.Present -or $entry.Kind -isnot [Microsoft.Win32.RegistryValueKind] -or $entry.Kind -ne [Microsoft.Win32.RegistryValueKind]::String -or $entry.Data -isnot [string] -or $entry.Data -cne $expected) { throw 'Notaya startup entry is not exactly the owned REG_SZ test command; preserving it' }
  }
}
function Remove-NotayaStartupValueFromKey($Key, [string]$ExpectedCommand) {
  $entry = Get-NotayaStartupEntryFromKey $Key
  if ($entry.Present -isnot [bool] -or !$entry.Present -or $entry.Kind -isnot [Microsoft.Win32.RegistryValueKind] -or $entry.Kind -ne [Microsoft.Win32.RegistryValueKind]::String -or $entry.Data -isnot [string] -or $entry.Data -cne $ExpectedCommand) { throw 'Startup entry changed before removal' }
  $Key.DeleteValue('Notaya', $true)
}
function Remove-NotayaStartupValue([string]$ExpectedCommand) {
  $base = $null; $key = $null
  try {
    $base = [Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::CurrentUser, [Microsoft.Win32.RegistryView]::Registry64)
    $key = $base.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Run', $true)
    if ($null -eq $key) { throw 'Run key unavailable' }
    Remove-NotayaStartupValueFromKey $key $ExpectedCommand
  } catch { throw 'Verified test startup-entry removal failed; foreign or unknown state was preserved' }
  finally { if ($null -ne $key) { $key.Dispose() }; if ($null -ne $base) { $base.Dispose() } }
}
function Remove-NotayaTestStartupEntry([string]$TestExe, [bool]$AttemptedOn) {
  if (!$AttemptedOn) { Assert-NotayaStartupAbsent; return }
  $entry = Read-NotayaStartupEntry ([Microsoft.Win32.RegistryView]::Registry64)
  if ($null -eq $entry -or $entry.Present -isnot [bool]) { throw 'Notaya startup-entry cleanup state is unknown' }
  if (!$entry.Present) { Assert-NotayaStartupAbsent; return }
  Assert-NotayaStartupOwned $TestExe
  Remove-NotayaStartupValue (Get-NotayaStartupCommand $TestExe)
  Assert-NotayaStartupAbsent
}
