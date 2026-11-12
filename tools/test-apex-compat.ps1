param([string]$SdkPath, [string]$NativeExe, [switch]$Live)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
if (-not $SdkPath) {
    $SdkPath = Get-ChildItem -LiteralPath "$env:SystemRoot/assembly/GAC_MSIL/Apex.Integration.DotNet" -Filter Apex.Integration.DotNet.dll -Recurse | Sort-Object FullName -Descending | Select-Object -First 1 -ExpandProperty FullName
}
if (-not $SdkPath -or -not (Test-Path -LiteralPath $SdkPath)) { throw 'Set -SdkPath to the host integration SDK assembly.' }
$testRoot = Join-Path $projectRoot ('target/apex-contract-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $testRoot | Out-Null
$testExe = Join-Path $testRoot 'CompatChecks.exe'
$compiler = Join-Path $env:SystemRoot 'Microsoft.NET/Framework64/v4.0.30319/csc.exe'
$component = Join-Path $projectRoot 'crates/sketch-app/src/apex_compat.cs'
$legacy = Join-Path $projectRoot 'crates/sketch-app/src/apex_legacy.cs'
$codec = Join-Path $projectRoot 'crates/sketch-app/src/apex_file.cs'
$geometry = Join-Path $projectRoot 'crates/sketch-app/src/apex_geometry.cs'
$drafts = Join-Path $projectRoot 'crates/sketch-app/src/apex_drafts.cs'
$codes = Join-Path $projectRoot 'crates/sketch-app/src/apex_codes.cs'
$tests = Join-Path $projectRoot 'tests/apex-compat/CompatChecks.cs'
& $compiler /nologo /target:exe /main:CompatChecks /reference:System.Windows.Forms.dll /reference:System.Drawing.dll /reference:System.Web.Extensions.dll "/reference:$SdkPath" "/resource:$SdkPath,SketchRust.HostContract" "/out:$testExe" $component $codec $geometry $drafts $codes $legacy $tests
if ($LASTEXITCODE -ne 0) { throw 'Compatibility test compilation failed.' }
& $testExe (Join-Path $testRoot 'fixtures')
if ($LASTEXITCODE -ne 0) { throw 'Compatibility contract tests failed.' }
if ($NativeExe) {
    $NativeExe = (Resolve-Path -LiteralPath $NativeExe).Path
    $transportSource = Join-Path $projectRoot 'tests/apex-compat/TransportCheck.cs'
    $transportExe = Join-Path $testRoot 'TransportCheck.exe'
    & $compiler /nologo /target:exe /main:Check /reference:System.Windows.Forms.dll /reference:System.Drawing.dll /reference:System.Web.Extensions.dll "/reference:$SdkPath" "/out:$transportExe" $transportSource
    if ($LASTEXITCODE -ne 0) { throw 'Transport test compilation failed.' }
    & $transportExe $NativeExe (Join-Path $testRoot 'transport')
    if ($LASTEXITCODE -ne 0) { throw 'Native SDK transport test failed. Local IPC requires an unsandboxed test environment.' }
}

if ($Live) {
    if (-not $NativeExe) { throw '-Live requires -NativeExe.' }
    $liveSource = Join-Path $projectRoot 'tests/apex-compat/LiveTransportCheck.cs'
    $liveExe = Join-Path $testRoot 'LiveTransportCheck.exe'
    & $compiler /nologo /target:exe /main:LiveCheck /reference:System.Windows.Forms.dll /reference:System.Drawing.dll /reference:System.Web.Extensions.dll "/reference:$SdkPath" "/out:$liveExe" $codec $geometry $drafts $codes $liveSource
    if ($LASTEXITCODE -ne 0) { throw 'Live transport test compilation failed.' }
    & $liveExe $NativeExe (Join-Path $testRoot 'live') (Join-Path $projectRoot 'target/apex-file-tests/output/rectangle.ax7')
    if ($LASTEXITCODE -ne 0) { throw 'Native live editor API test failed.' }
}

if ($Live) {
    $codesExe = Join-Path $testRoot 'HostCodesCheck.exe'
    $codesSource = Join-Path $projectRoot 'tests/apex-compat/HostCodesCheck.cs'
    & $compiler /nologo /target:exe /main:HostCodesCheck /reference:System.Web.Extensions.dll "/out:$codesExe" $codec $geometry $drafts $codes $codesSource
    if ($LASTEXITCODE -ne 0) { throw 'Host code-table test compilation failed.' }
    & $codesExe $NativeExe (Join-Path $testRoot 'codes')
    if ($LASTEXITCODE -ne 0) { throw 'Host code-table URL test failed.' }
    $batch = Join-Path $testRoot 'batch'
    New-Item -ItemType Directory -Path $batch | Out-Null
    Copy-Item -LiteralPath (Join-Path $projectRoot 'target/apex-file-tests/output/rectangle.ax7') -Destination (Join-Path $batch 'first.ax7')
    Copy-Item -LiteralPath (Join-Path $projectRoot 'target/apex-file-tests/output/metric.ax7') -Destination (Join-Path $batch 'second.ax7')
    $batchOut = Join-Path $batch 'stdout.txt'
    $batchErr = Join-Path $batch 'stderr.txt'
    $batchProcess = Start-Process -FilePath $NativeExe -ArgumentList @('--check-native-batch', ('"' + (Join-Path $batch 'first.ax7') + '"'), ('"' + (Join-Path $batch 'second.ax7') + '"')) -WindowStyle Hidden -PassThru -RedirectStandardOutput $batchOut -RedirectStandardError $batchErr
    if (-not $batchProcess.WaitForExit(45000)) { $batchProcess.Kill(); throw 'Multiple-file startup test timed out.' }
    $batchProcess.Refresh()
    Get-Content -LiteralPath $batchOut
    if ($batchProcess.ExitCode -ne 0) { throw "Multiple-file startup failed: $(Get-Content -LiteralPath $batchErr -Raw)" }
}

if ($Live) {
    foreach ($draftVersion in @(1,23,38)) {
        $draftPath = Join-Path $testRoot ("repair-v" + $draftVersion + '.ax7')
        Copy-Item -LiteralPath (Join-Path $projectRoot ("target/apex-file-tests/output/unfinished-v" + $draftVersion + '.ax7')) -Destination $draftPath
        $draftOut = Join-Path $testRoot ("repair-v" + $draftVersion + '-out.txt')
        $draftErr = Join-Path $testRoot ("repair-v" + $draftVersion + '-err.txt')
        $repair = Start-Process -FilePath $NativeExe -ArgumentList @('--check-native-draft', ('"' + $draftPath + '"')) -WindowStyle Hidden -PassThru -RedirectStandardOutput $draftOut -RedirectStandardError $draftErr
        if (-not $repair.WaitForExit(45000)) { $repair.Kill(); throw 'Native draft repair test timed out.' }
        $repair.Refresh()
        Get-Content -LiteralPath $draftOut
        if ($repair.ExitCode -ne 0) { throw "Native draft repair failed: $(Get-Content -LiteralPath $draftErr -Raw)" }
        & $codesExe --verify-repair $draftPath
        if ($LASTEXITCODE -ne 0) { throw 'Native repaired file validation failed.' }
    }
}
