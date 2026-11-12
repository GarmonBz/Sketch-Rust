param([string]$ReferenceExe)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
$testRoot = Join-Path $projectRoot 'target/apex-file-tests'
New-Item -ItemType Directory -Path $testRoot -Force | Out-Null
$compiler = Join-Path $env:SystemRoot 'Microsoft.NET/Framework64/v4.0.30319/csc.exe'
$codec = Join-Path $projectRoot 'crates/sketch-app/src/apex_file.cs'
$geometry = Join-Path $projectRoot 'crates/sketch-app/src/apex_geometry.cs'
$drafts = Join-Path $projectRoot 'crates/sketch-app/src/apex_drafts.cs'
$codes = Join-Path $projectRoot 'crates/sketch-app/src/apex_codes.cs'
$tests = Join-Path $projectRoot 'tests/apex-compat/NativeFileChecks.cs'
$testExe = Join-Path $testRoot 'NativeFileChecks.exe'
& $compiler /nologo /target:exe /reference:System.Web.Extensions.dll "/out:$testExe" $codec $geometry $drafts $codes $tests
if ($LASTEXITCODE -ne 0) { throw 'Native codec test compilation failed.' }
& $testExe (Join-Path $projectRoot 'target/apex-native-fixtures') (Join-Path $testRoot 'output') (Join-Path $projectRoot 'assets/symbols.json')
if ($LASTEXITCODE -ne 0) { throw 'Native codec checks failed.' }

if ($ReferenceExe) {
    $referenceSource = Join-Path $projectRoot 'tests/apex-compat/ReferenceFileChecks.cs'
    $referenceTest = Join-Path $testRoot 'ReferenceFileChecks.exe'
    & $compiler /nologo /platform:x86 "/out:$referenceTest" $referenceSource
    if ($LASTEXITCODE -ne 0) { throw 'Reference validator compilation failed.' }
    & $referenceTest $ReferenceExe (Join-Path $testRoot 'output') (Join-Path $projectRoot 'target/apex-native-fixtures')
    if ($LASTEXITCODE -ne 0) { throw 'Reference Apex reader rejected codec output.' }
}
