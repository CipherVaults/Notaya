# Exercise the real release script with inert fixture executables. No build, install,
# network call or application launch occurs. All output stays under NOTAYA_HOME.
$ErrorActionPreference = 'Stop'
if ($env:NOTAYA_HOME -ne 'D:\Notaya Tests\v1-6-D') { throw 'Set NOTAYA_HOME to the lane D throwaway folder first.' }
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$fixture = Join-Path $env:NOTAYA_HOME ('packaging-' + [Guid]::NewGuid().ToString('N'))
$fixtureTarget = Join-Path $fixture 'target'
$fixtureVersion = (Get-Content (Join-Path $repo 'package.json') -Raw | ConvertFrom-Json).version
$release = Join-Path $fixtureTarget 'release'
$bundle = Join-Path $release 'bundle\nsis'
New-Item -ItemType Directory -Path $bundle | Out-Null
[IO.File]::WriteAllText((Join-Path $release 'Notaya.exe'), 'INERT APP FIXTURE')
[IO.File]::WriteAllText((Join-Path $release 'notaya-bridge.exe'), 'INERT BRIDGE FIXTURE')
[IO.File]::WriteAllText((Join-Path $bundle "Notaya_${fixtureVersion}_x64-setup.exe"), 'INERT SETUP FIXTURE')

function cargo {
  if ($args -notcontains '--offline' -or $args -notcontains '--locked') { throw 'Cargo must use the locked offline workspace.' }
  $global:LASTEXITCODE = 0
  if ($args[0] -eq 'metadata') { @{ target_directory = $fixtureTarget } | ConvertTo-Json -Compress }
  elseif ($args[0] -ne 'build' -or $args -notcontains 'notaya-bridge') { throw 'Unexpected Cargo invocation.' }
}
function npm {
  if ($args[0] -ne 'run' -or $args[1] -ne 'tauri' -or $args[2] -ne 'build') { throw 'Unexpected npm invocation.' }
  if ($args -notcontains '--offline' -or $args -notcontains '--locked') { throw 'App build must also be offline and locked.' }
  $configIndex = [Array]::IndexOf($args, '--config')
  if ($configIndex -lt 0) { throw 'Setup build must use the companion resource map.' }
  $config = Get-Content -LiteralPath $args[$configIndex + 1] -Raw | ConvertFrom-Json
  $properties = @($config.bundle.resources.PSObject.Properties)
  if ($properties.Count -ne 1 -or $properties[0].Value -ne 'notaya-bridge.exe' -or
      $properties[0].Name -ne (Join-Path $fixtureTarget 'release\notaya-bridge.exe')) { throw 'Bridge is not mapped beside the app.' }
  $global:LASTEXITCODE = 0
}
function git {
  if ($args[0] -eq 'rev-parse') { 'fixture-commit' }
  elseif ($args[0] -ne 'status') { throw 'Unexpected git invocation.' }
  $global:LASTEXITCODE = 0
}

$out = Join-Path $fixture 'output'
& (Join-Path $repo 'tools\windows-build\build-windows.ps1') -OutDir $out
$portable = Join-Path $out 'Notaya'
if ([IO.File]::ReadAllText((Join-Path $portable 'notaya-bridge.exe')) -ne 'INERT BRIDGE FIXTURE') { throw 'Wrong portable bridge.' }
if (-not (Test-Path -LiteralPath (Join-Path $portable 'AI-connection.md'))) { throw 'Missing connection guide.' }
if (Test-Path -LiteralPath (Join-Path $portable 'data')) { throw 'A portable release must contain no data folder.' }
if ((Get-Content (Join-Path $out 'hashes.txt') -Raw) -notmatch 'notaya-bridge.exe') { throw 'Bridge hash not recorded.' }
$zip = [IO.Compression.ZipFile]::OpenRead((Join-Path $out "Notaya-V${fixtureVersion}-portable.zip"))
try {
  $names = @($zip.Entries | ForEach-Object FullName)
  if ($names -notcontains 'Notaya/notaya-bridge.exe' -or $names -notcontains 'Notaya/Notaya.exe' -or $names -notcontains 'Notaya/AI-connection.md') { throw 'Portable ZIP does not contain both executables and the guide.' }
  if (@($names | Where-Object { $_ -match '\.(nota|clf|key)$|/data/' }).Count) { throw 'Data leaked into the fixture release.' }
} finally { $zip.Dispose() }
Write-Host 'PASS: offline release invocations, setup resource map, portable files, ZIP entries and bridge hash (inert fixtures only).'
