<#
.SYNOPSIS
  Builds the two Windows downloads of a Notaya release: the setup (installer) and the portable ZIP.

.DESCRIPTION
  Run from a normal (non-elevated) PowerShell on the Windows PC, from a clean checkout of the
  commit to release, after the tests have passed. It runs `npm run tauri build`, which makes
  `Notaya.exe` and the NSIS setup (the first run downloads the NSIS tool, which needs the owner's OK),
  then writes into -OutDir:

    Notaya\                      the portable folder: Notaya.exe, portable.txt, README.txt, Licenses\
    Notaya-V<version>-portable.zip
    Notaya-V<version>-setup.exe
    hashes.txt                   size and SHA-256 of the exe, the ZIP and the setup

  The portable.txt marker is what makes the exe keep its notes in a data folder next to it. The
  setup has no such file, so an installed copy keeps its notes in %LOCALAPPDATA%\Notaya Notes.
  No vault data, keys or test folders are ever copied. Nothing is published or committed.

.EXAMPLE
  .\tools\windows-build\build-windows.ps1 -OutDir "E:\Project\Notaya\Notaya Tests Excluded\Notaya-V1.4.0"
#>
param(
  [Parameter(Mandatory = $true)][string]$OutDir
)

$ErrorActionPreference = 'Stop'
# .NET resolves relative paths against the process folder, not the PowerShell location: make it absolute.
$OutDir = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutDir)
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Set-Location $root

# The setup's file name comes from tauri.conf.json; package.json must agree.
$ver = (Get-Content (Join-Path $root 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json).version
$pkgVer = (Get-Content (Join-Path $root 'package.json') -Raw | ConvertFrom-Json).version
if ($ver -ne $pkgVer) { throw "Versions differ: tauri.conf.json $ver, package.json $pkgVer" }
$commit = (git rev-parse HEAD).Trim()
$dirty = (git status --porcelain --untracked-files=no)
if ($dirty) { Write-Warning "Tracked files have changes; the build does not match commit $commit exactly." }
if (Test-Path $OutDir) { throw "OutDir already exists: $OutDir (choose a new folder; nothing is overwritten)" }

Write-Host "Building Notaya $ver from $commit ..."
npm run tauri build
if ($LASTEXITCODE -ne 0) { throw "npm run tauri build failed ($LASTEXITCODE)" }

$exe = Join-Path $root 'src-tauri\target\release\Notaya.exe'
$setup = Join-Path $root "src-tauri\target\release\bundle\nsis\Notaya_${ver}_x64-setup.exe"
foreach ($f in @($exe, $setup)) { if (-not (Test-Path $f)) { throw "Missing build output: $f" } }

$portable = Join-Path $OutDir 'Notaya'
New-Item -ItemType Directory -Path $portable, (Join-Path $portable 'Licenses') | Out-Null
Copy-Item $exe $portable
Copy-Item (Join-Path $root 'packaging\Licenses\*') (Join-Path $portable 'Licenses')
Copy-Item (Join-Path $root 'THIRD-PARTY-NOTICES.md') (Join-Path $portable 'Licenses')

$exeHash = (Get-FileHash (Join-Path $portable 'Notaya.exe') -Algorithm SHA256).Hash
[IO.File]::WriteAllText((Join-Path $portable 'portable.txt'),
"This file makes Notaya portable: it keeps your notes in the 'data' folder next to Notaya.exe,`r`n" +
"so you can run it from a USB stick and nothing is left on the PC. Do not delete it.`r`n" +
"(The installed version of Notaya has no such file and keeps its notes in %LOCALAPPDATA%\Notaya Notes.)`r`n")
[IO.File]::WriteAllText((Join-Path $portable 'README.txt'),
"Notaya $ver (portable)`r`n" +
"Build commit: $commit`r`n" +
"Notaya.exe SHA-256: $exeHash`r`n`r`n" +
"Run Notaya.exe. Your notes are kept in the 'data' folder next to it, created on first start.`r`n" +
"To move to another PC or a USB stick, close Notaya and copy the whole folder, data included.`r`n" +
"To move to the installed version, install Notaya and choose 'Import notes from a portable copy'`r`n" +
"on its first screen (close this portable copy first; it is copied, never changed).`r`n" +
"Licenses of the parts Notaya is made from are in the Licenses folder.`r`n")

# The ZIP is written entry by entry with '/' names, one top-level folder 'Notaya'.
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zipPath = Join-Path $OutDir "Notaya-V$ver-portable.zip"
$zip = [IO.Compression.ZipFile]::Open($zipPath, 'Create')
try {
  Get-ChildItem $portable -Recurse -File | ForEach-Object {
    $rel = $_.FullName.Substring($portable.Length).TrimStart('\', '/').Replace('\', '/')
    [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $_.FullName, "Notaya/$rel", 'Optimal')
  }
} finally { $zip.Dispose() }

$setupOut = Join-Path $OutDir "Notaya-V$ver-setup.exe"
Copy-Item $setup $setupOut

$lines = foreach ($f in @((Join-Path $portable 'Notaya.exe'), $zipPath, $setupOut)) {
  $i = Get-Item $f
  '{0}  {1,12:N0} bytes  SHA-256 {2}' -f $i.Name, $i.Length, (Get-FileHash $f -Algorithm SHA256).Hash
}
$lines = @("Notaya $ver, commit $commit") + $lines
$lines | Set-Content (Join-Path $OutDir 'hashes.txt') -Encoding UTF8
$lines | ForEach-Object { Write-Host $_ }
