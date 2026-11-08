[CmdletBinding()]
param(
    [ValidateSet('Core', 'Live')][string]$Mode = 'Core',
    [switch]$NativeBridgeTests,
    [string]$AppExe,
    [string]$BridgeExe
)
$ErrorActionPreference = 'Stop'
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$testRoot = 'D:\Notaya Tests\v1-6-C5-smoke'
$runHome = Join-Path $testRoot ('run-' + [Guid]::NewGuid().ToString('N'))

function Assert-PlainPath([string]$Path) {
    $cursor = [IO.Path]::GetFullPath($Path)
    while ($cursor) {
        if (Test-Path -LiteralPath $cursor) {
            if ((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
                throw 'Smoke folders must not traverse a junction or symbolic link.'
            }
        }
        $parent = [IO.Path]::GetDirectoryName($cursor)
        if ($parent -eq $cursor) { break }
        $cursor = $parent
    }
}
function Invoke-Cargo([string[]]$CargoArgs) {
    & cargo @CargoArgs
    if ($LASTEXITCODE -ne 0) { throw "cargo failed (exit $LASTEXITCODE)." }
}
function Send-Rpc($Message) {
    $script:bridge.StandardInput.WriteLine(($Message | ConvertTo-Json -Depth 30 -Compress))
    $script:bridge.StandardInput.Flush()
}
function Receive-Rpc([int]$Id) {
    $until = [DateTime]::UtcNow.AddSeconds(310)
    while ([DateTime]::UtcNow -lt $until) {
        if (-not $script:pendingLine) { $script:pendingLine = $script:bridge.StandardOutput.ReadLineAsync() }
        if (-not $script:pendingLine.Wait(250)) { continue }
        $line = $script:pendingLine.Result
        $script:pendingLine = $null
        if ($null -eq $line) { throw 'Bridge exited before the MCP reply.' }
        $message = $line | ConvertFrom-Json
        if ($message.id -eq $Id -and ($message.result -or $message.error)) { return $message }
        # Tool-list and progress notifications are expected while the window asks.
    }
    throw 'Timed out waiting for the bridge. No success is recorded.'
}
function Invoke-Page([int]$Id, [string]$Action, $Params, [switch]$Preview) {
    $arguments = @{ action = $Action; params = $Params }
    if ($Preview) { $arguments.preview = $true }
    Send-Rpc @{ jsonrpc = '2.0'; id = $Id; method = 'tools/call'; params = @{
        name = 'notaya_page'; arguments = $arguments
    }}
    Receive-Rpc $Id
}
function Assert-Success($Reply) {
    if ($Reply.error -or $Reply.result.isError -or -not $Reply.result) { throw 'MCP call did not succeed.' }
}
function Assert-Refusal($Reply, [string]$Code) {
    if (-not $Reply.result.isError -or -not ([string]$Reply.result.content[0].text).StartsWith($Code + ':')) {
        throw "Expected $Code refusal was not received."
    }
}

$oldHome = $env:NOTAYA_HOME
$oldFallback = $env:CIPHERLEAF_HOME
$oldWebview = $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
$oldCargoOffline = $env:CARGO_NET_OFFLINE
$app = $null
$script:bridge = $null
$script:pendingLine = $null
try {
    Assert-PlainPath $testRoot
    New-Item -ItemType Directory -Path $runHome | Out-Null
    $env:NOTAYA_HOME = $runHome
    $env:CIPHERLEAF_HOME = $runHome
    $env:CARGO_NET_OFFLINE = 'true'
    Remove-Item Env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ErrorAction SilentlyContinue
    Write-Host "Throwaway NOTAYA_HOME: $runHome"
    Write-Host ('Source HEAD: ' + (& git -C $repo rev-parse HEAD))
    Write-Host 'Results apply to these working-tree files; heartbeat checks the eventual merge commit.'
    Push-Location (Join-Path $repo 'src-tauri')
    try {
        if ($Mode -eq 'Core') {
            Invoke-Cargo -CargoArgs @('test', '--lib', '--offline', 'ai::smoke::bridge_core_smoke', '--', '--nocapture')
            if ($NativeBridgeTests) {
                Invoke-Cargo -CargoArgs @('test', '--offline', '-p', 'notaya-bridge', 'pipe::native::tests', '--', '--include-ignored', '--nocapture')
            }
            Write-Host 'CHECKPOINT: core smoke done'
            return
        }
    } finally { Pop-Location }

    if ($NativeBridgeTests) { throw 'Use NativeBridgeTests with Mode Core.' }
    if (-not $AppExe -or -not $BridgeExe) { throw 'Live mode needs AppExe and BridgeExe built from the same reviewed source.' }
    foreach ($file in @($AppExe, $BridgeExe)) {
        if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw 'A supplied executable does not exist.' }
    }
    # Single-instance guards can otherwise hand off to a real window. Do not send it anything.
    if (Get-Process -Name 'Notaya', 'Cipherleaf' -ErrorAction SilentlyContinue) {
        throw 'Close existing Notaya/Cipherleaf yourself before Live mode. No existing process was touched.'
    }
    $bin = New-Item -ItemType Directory -Path (Join-Path $runHome 'bin')
    Copy-Item -LiteralPath $AppExe -Destination (Join-Path $bin.FullName 'Notaya.exe')
    Copy-Item -LiteralPath $BridgeExe -Destination (Join-Path $bin.FullName 'notaya-bridge.exe')
    $appPath = Join-Path $bin.FullName 'Notaya.exe'
    $app = Start-Process -FilePath $appPath -WorkingDirectory $bin.FullName -WindowStyle Normal -PassThru
    Write-Host 'In the NEW throwaway window: start empty notes, create a notebook and one page titled C5 smoke.'
    Write-Host 'Open Settings > AI access; turn it on by typing ALLOW. Keep this page free of private text.'
    $null = Read-Host 'Press Enter after the page and AI access are ready'
    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = Join-Path $bin.FullName 'notaya-bridge.exe'
    $start.Arguments = '--home "' + $runHome + '"'
    $start.WorkingDirectory = $bin.FullName
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardInput = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.StandardOutputEncoding = New-Object Text.UTF8Encoding($false)
    $candidate = New-Object Diagnostics.Process
    $candidate.StartInfo = $start
    try { $null = $candidate.Start() } catch { $candidate.Dispose(); throw }
    $script:bridge = $candidate
    $stderr = $script:bridge.StandardError.ReadToEndAsync()
    Send-Rpc @{ jsonrpc = '2.0'; id = 1; method = 'initialize'; params = @{
        protocolVersion = '2025-11-25'; capabilities = @{}; clientInfo = @{ name = 'C5 PC smoke'; version = '1' }
    }}
    Assert-Success (Receive-Rpc 1)
    Send-Rpc @{ jsonrpc = '2.0'; method = 'notifications/initialized' }
    Write-Host 'Allow C5 PC smoke in the Notaya pairing dialog. It starts at Look.'
    Send-Rpc @{ jsonrpc = '2.0'; id = 2; method = 'tools/list'; params = @{} }
    Assert-Success (Receive-Rpc 2)
    Write-Host 'CHECKPOINT: live pairing done'
    Write-Host 'In AI access raise ONLY C5 PC smoke to Edit. Leave the page for Settings and wait at least 4 seconds.'
    $null = Read-Host 'Press Enter after changing the level, leaving the editor and waiting'
    # Get the generated id from a harmless list; no person needs to copy it from disk.
    Send-Rpc @{ jsonrpc = '2.0'; id = 3; method = 'tools/call'; params = @{
        name = 'notaya_notes'; arguments = @{ action = 'page.recent'; params = @{} }
    }}
    $recent = Receive-Rpc 3
    Assert-Success $recent
    $pages = @($recent.result.structuredContent.value | Where-Object { $_.title -eq 'C5 smoke' })
    if ($pages.Count -ne 1) { throw 'Create exactly one throwaway page titled C5 smoke.' }
    $pageId = [string]$pages[0].id
    $read = Invoke-Page 4 'page.get' @{ id = $pageId; format = 'blocks' }
    Assert-Success $read
    $revision = $read.result.structuredContent.value.rev
    Write-Host 'CHECKPOINT: live page read done'
    Write-Host 'Next call: DENY the page-to-bin confirmation in Notaya.'
    Assert-Refusal (Invoke-Page 5 'page.trash' @{ id = $pageId }) 'not_confirmed'
    $afterDeny = Invoke-Page 6 'page.get' @{ id = $pageId; format = 'blocks' }
    Assert-Success $afterDeny
    if ($afterDeny.result.structuredContent.value.rev -ne $revision) { throw 'Denied edit changed the revision.' }
    Write-Host 'CHECKPOINT: live denied edit with unchanged revision done'
    Write-Host 'After Enter, switch to C5 smoke in the TEST window and type steadily for up to 20 seconds.'
    Write-Host 'The script will preview the bin edit until it sees your typing lease; previews never edit or open confirmation.'
    $null = Read-Host 'Press Enter, then switch to the test editor and type'
    $editingUntil = [DateTime]::UtcNow.AddSeconds(20)
    $sawEditing = $false
    $previewId = 100
    while ([DateTime]::UtcNow -lt $editingUntil) {
        $previewReply = Invoke-Page $previewId 'page.trash' @{ id = $pageId } -Preview
        $previewId++
        if ($previewReply.result.isError) {
            Assert-Refusal $previewReply 'person_editing'
            $sawEditing = $true
            break
        }
        Assert-Success $previewReply
        Start-Sleep -Milliseconds 100
    }
    if (-not $sawEditing) { throw 'No person-editing refusal observed within 20 seconds. The live case did not pass.' }
    Write-Host 'CHECKPOINT: live person-editing refusal done'
    Write-Host 'In Notaya leave the editor for Settings. Wait at least 4 seconds for its lease to expire.'
    $null = Read-Host 'Press Enter after waiting; then ALLOW the page-to-bin confirmation'
    Assert-Success (Invoke-Page 8 'page.trash' @{ id = $pageId })
    Assert-Refusal (Invoke-Page 9 'page.get' @{ id = $pageId; format = 'blocks' }) 'not_found'
    Write-Host 'In the test window confirm C5 smoke is in the recycle bin. Restore it and check the text.'
    $null = Read-Host 'Press Enter after inspecting the bin and restoring the page'
    Assert-Success (Invoke-Page 10 'page.get' @{ id = $pageId; format = 'blocks' })
    Write-Host 'CHECKPOINT: live confirmed edit and restored page read done'
} finally {
  try {
    if ($script:bridge) {
        if (-not $script:bridge.HasExited) {
            $script:bridge.StandardInput.Close()
            if (-not $script:bridge.WaitForExit(3000)) { $script:bridge.Kill(); $script:bridge.WaitForExit() }
        }
        $script:bridge.Dispose()
    }
    $script:pendingLine = $null
    if ($app) {
        $app.Refresh()
        if (-not $app.HasExited) {
            Write-Host 'Quit the throwaway Notaya from its tray menu (closing its window may only hide it).'
            $null = Read-Host 'Press Enter after Quit'
            $app.Refresh()
            if (-not $app.HasExited) { Write-Warning "Test Notaya is still running at PID $($app.Id). Quit it before another run." }
        }
        $app.Dispose()
    }
  } finally {
    $env:NOTAYA_HOME = $oldHome
    $env:CIPHERLEAF_HOME = $oldFallback
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $oldWebview
    $env:CARGO_NET_OFFLINE = $oldCargoOffline
    Write-Host "Evidence and throwaway data kept at $runHome. No debug port was requested."
  }
}
