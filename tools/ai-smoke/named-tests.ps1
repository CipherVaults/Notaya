# Pure output validation; no commands, files, processes or app state are touched.
function Assert-NamedTests([string[]]$OutputLines, [string[]]$ExpectedTests) {
    if (-not $ExpectedTests -or $ExpectedTests.Count -eq 0) {
        throw 'Expected test names must be nonempty and unique.'
    }
    $expectedSet = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($expectedName in $ExpectedTests) {
        if ([string]::IsNullOrWhiteSpace($expectedName) -or -not $expectedSet.Add($expectedName)) {
            throw 'Expected test names must be nonempty and unique.'
        }
    }
    $passed = @()
    foreach ($line in $OutputLines) {
        if ($line -match '^test (\S+) \.\.\. ok\s*$') { $passed += $Matches[1] }
    }
    $passedSet = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($passedName in $passed) { $null = $passedSet.Add($passedName) }
    if ($passed.Count -ne $ExpectedTests.Count -or $passedSet.Count -ne $ExpectedTests.Count -or
        @($ExpectedTests | Where-Object { -not $passedSet.Contains($_) }).Count -ne 0) {
        throw 'The required named tests did not all run and pass exactly once.'
    }
    $summaries = @($OutputLines | Where-Object { $_ -match '^test result:' })
    if ($summaries.Count -ne 1 -or $summaries[0] -notmatch
        '^test result: ok\. (\d+) passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out;') {
        throw 'A complete successful test summary with no ignored tests is required.'
    }
    if ([int]$Matches[1] -ne $ExpectedTests.Count) { throw 'The test summary count does not match the required names.' }
}
