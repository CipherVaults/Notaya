param([string]$LegacyScript)
$ErrorActionPreference = 'Stop'
$proofCount = 0
function Expect([string]$ProofLabel, [scriptblock]$Action, [bool]$ShouldPass) {
    $accepted = $false
    try { & $Action; $accepted = $true } catch { $why = $_.Exception.Message }
    if ($accepted -ne $ShouldPass) { throw "FAIL $ProofLabel (accepted=$accepted expected=$ShouldPass): $why" }
    $script:proofCount++
    Write-Output "PASS $ProofLabel (accepted=$accepted)"
}
$name = 'ai::smoke::bridge_core_smoke_pair_read_confirm_refuse_person_editing_and_ordered_delivery'
$good = @("test $name ... ok", 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 99 filtered out; finished in 0.01s')
$zero = @('running 0 tests', 'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 99 filtered out; finished in 0.00s')
if ($LegacyScript) {
    $tokens = $null; $parseErrors = $null
    $ast = [Management.Automation.Language.Parser]::ParseFile($LegacyScript, [ref]$tokens, [ref]$parseErrors)
    if ($parseErrors.Count) { throw 'Legacy fixture did not parse.' }
    $definition = @($ast.FindAll({ param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Invoke-Cargo' }, $true))
    if ($definition.Count -ne 1) { throw 'Expected the one original Invoke-Cargo function.' }
    . ([scriptblock]::Create($definition[0].Extent.Text))
    function cargo { $global:LASTEXITCODE = 0; $script:fakeCargoOutput }
    $script:fakeCargoOutput = $good
    Expect 'BEFORE good named case accepted' { Invoke-Cargo -CargoArgs @('test') | Out-Null } $true
    $script:fakeCargoOutput = $zero
    Expect 'BEFORE zero tests incorrectly accepted' { Invoke-Cargo -CargoArgs @('test') | Out-Null } $true
    $script:fakeCargoOutput = @('test another::case ... ok', $good[1])
    Expect 'BEFORE wrong name incorrectly accepted' { Invoke-Cargo -CargoArgs @('test') | Out-Null } $true
    $script:fakeCargoOutput = @("test $name ... ok", 'test result: ok. NaN passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;')
    Expect 'BEFORE non-finite count incorrectly accepted' { Invoke-Cargo -CargoArgs @('test') | Out-Null } $true
} else {
    . (Join-Path $PSScriptRoot 'named-tests.ps1')
    Expect 'AFTER good named case accepted' { Assert-NamedTests $good @($name) } $true
    Expect 'AFTER zero tests refused' { Assert-NamedTests $zero @($name) } $false
    Expect 'AFTER empty expected selection refused' { Assert-NamedTests $good @() } $false
    Expect 'AFTER wrong name refused' { Assert-NamedTests @('test another::case ... ok', $good[1]) @($name) } $false
    Expect 'AFTER case-only wrong name refused' { Assert-NamedTests @(('test ' + $name.ToUpperInvariant() + ' ... ok'), $good[1]) @($name) } $false
    Expect 'AFTER duplicate expected names refused' { Assert-NamedTests $good @($name, $name) } $false
    $caseNames = @('pipe::native::tests::foo', 'pipe::native::tests::FOO')
    $caseOutput = @($caseNames | ForEach-Object { "test $_ ... ok" }) + @($good[1].Replace('1 passed', '2 passed'))
    Expect 'AFTER two distinct case-sensitive names accepted' { Assert-NamedTests $caseOutput $caseNames } $true
    Expect 'AFTER planted failed test refused' { Assert-NamedTests @("test $name ... FAILED", $good[1]) @($name) } $false
    Expect 'AFTER ignored test refused' { Assert-NamedTests @("test $name ... ignored", $good[1]) @($name) } $false
    Expect 'AFTER duplicate named pass refused' { Assert-NamedTests @($good[0], $good[0], $good[1]) @($name) } $false
    Expect 'AFTER missing summary refused' { Assert-NamedTests @($good[0]) @($name) } $false
    Expect 'AFTER extra summary refused' { Assert-NamedTests @($good[0], $good[1], $good[1]) @($name) } $false
    Expect 'AFTER summary count mismatch refused' { Assert-NamedTests @($good[0], $good[1].Replace('1 passed', '2 passed')) @($name) } $false
    foreach ($bad in @('NaN', 'Infinity', '-Infinity')) {
        Expect "AFTER non-finite count $bad refused" { Assert-NamedTests @($good[0], $good[1].Replace('1 passed', "$bad passed")) @($name) } $false
    }
    $multiNames = @('pipe::native::tests::one', 'pipe::native::tests::two', 'pipe::native::tests::three')
    $multiOutput = @($multiNames | ForEach-Object { "test $_ ... ok" }) + @($good[1].Replace('1 passed', '3 passed'))
    Expect 'AFTER three distinct native names accepted' { Assert-NamedTests $multiOutput $multiNames } $true

    # Exercise the real runner's function, not only the standalone parser.
    # Extracting a function definition does not execute the runner's app/setup code.
    $tokens = $null; $parseErrors = $null
    $runnerAst = [Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot 'run-pc-check.ps1'), [ref]$tokens, [ref]$parseErrors)
    if ($parseErrors.Count) { throw 'The smoke runner did not parse.' }
    $definition = @($runnerAst.FindAll({ param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Invoke-Cargo' }, $true))
    if ($definition.Count -ne 1) { throw 'Expected the one current Invoke-Cargo function.' }
    . ([scriptblock]::Create($definition[0].Extent.Text))
    function cargo { $global:LASTEXITCODE = $script:fakeNativeExit; $script:fakeCargoOutput }
    $script:fakeNativeExit = 0
    $script:fakeCargoOutput = $good
    Expect 'runner wiring accepts good named output' { Invoke-Cargo @('test') @($name) } $true
    $script:fakeCargoOutput = $zero
    Expect 'runner wiring refuses zero tests' { Invoke-Cargo @('test') @($name) } $false
    $script:fakeCargoOutput = @('test another::case ... ok', $good[1])
    Expect 'runner wiring refuses a renamed test' { Invoke-Cargo @('test') @($name) } $false
    $script:fakeCargoOutput = @($good[0], $good[1].Replace('1 passed', 'NaN passed'))
    Expect 'runner wiring refuses a non-finite count' { Invoke-Cargo @('test') @($name) } $false
    $script:fakeCargoOutput = $good
    Expect 'runner wiring refuses an empty name set' { Invoke-Cargo @('test') @() } $false
    $script:fakeNativeExit = 1
    Expect 'runner wiring preserves native command failure' { Invoke-Cargo @('test') @($name) } $false
    $script:fakeNativeExit = -1
    Expect 'runner wiring preserves negative native command failure' { Invoke-Cargo @('test') @($name) } $false
}
if ($proofCount -eq 0) { throw 'No grader proofs ran.' }
Write-Output "ALL OK ($proofCount grader-output proofs; not Rust or app tests)"
