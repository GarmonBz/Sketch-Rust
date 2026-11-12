param([string]$ArchivePath)
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if (-not $ArchivePath) {
    $ArchivePath = (Get-ChildItem -LiteralPath (Join-Path $repoRoot 'dist') -Filter 'SketchRust-*-windows-x64.zip' |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
}
if (-not $ArchivePath) { throw 'No packaged archive found.' }
$work = Join-Path $repoRoot 'target\package-smoke'
$unpacked = Join-Path $work 'unpacked'
$emptyCwd = Join-Path $work 'empty-cwd'
$missing = Join-Path $work 'missing-library'
New-Item -ItemType Directory -Path $work,$unpacked,$emptyCwd,$missing -Force | Out-Null
Expand-Archive -LiteralPath $ArchivePath -DestinationPath $unpacked -Force
$manifest = Get-Content -LiteralPath (Join-Path $unpacked 'manifest.json') -Raw | ConvertFrom-Json
foreach ($entry in $manifest.files) {
    $filePath = [IO.Path]::GetFullPath((Join-Path $unpacked $entry.name))
    if (-not $filePath.StartsWith($unpacked + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Manifest path escapes the extracted package.'
    }
    if ((Get-FileHash -LiteralPath $filePath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256) {
        throw "Checksum mismatch: $($entry.name)"
    }
}
$exe = Join-Path $unpacked 'sketch-app.exe'
$peBytes = [IO.File]::ReadAllBytes($exe)
$peOffset = [BitConverter]::ToInt32($peBytes, 0x3c)
$subsystem = [BitConverter]::ToUInt16($peBytes, $peOffset + 92)
if ($subsystem -ne 2) { throw "Expected Windows GUI subsystem (2), got $subsystem." }
$exeHash = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash
if ($exeHash -ne (Get-FileHash -LiteralPath (Join-Path $repoRoot 'target\release\sketch-app.exe') -Algorithm SHA256).Hash) {
    throw 'Archive executable does not match the current optimized build.'
}
function Invoke-InstallationCheck([string]$FilePath, [string]$Label) {
    $stdout = Join-Path $work "$Label-stdout.txt"
    $stderr = Join-Path $work "$Label-stderr.txt"
    $probe = Start-Process -FilePath $FilePath -ArgumentList '--check-installation' -WorkingDirectory $emptyCwd `
        -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    if (-not $probe.WaitForExit(30000)) {
        $probe.Kill()
        throw "Installation check timed out: $Label"
    }
    # Refresh the process object before reading ExitCode on Windows.
    $probe.Refresh()
    [pscustomobject]@{ exitCode=$probe.ExitCode; stdout=(Get-Content -LiteralPath $stdout -Raw); stderr=(Get-Content -LiteralPath $stderr -Raw) }
}
$positive = Invoke-InstallationCheck $exe 'installed'
if ($positive.exitCode -ne 0 -or $positive.stdout -notmatch 'OK; \d+ external symbols; native view; SKX round trip') {
    throw "Packaged installation failed: $($positive | ConvertTo-Json -Compress)"
}
$missingExe = Join-Path $missing 'sketch-app.exe'
Copy-Item -LiteralPath $exe -Destination $missingExe -Force
$negative = Invoke-InstallationCheck $missingExe 'missing-library'
if ($negative.exitCode -eq 0 -or $negative.stderr -notmatch 'Installation check failed:') {
    throw "Missing-library check failed: $($negative | ConvertTo-Json -Compress)"
}
$contractOut = Join-Path $work 'host-contract-stdout.txt'
$contractErr = Join-Path $work 'host-contract-stderr.txt'
$contract = Start-Process -FilePath $missingExe -ArgumentList '--check-apex-inprocess' -WorkingDirectory $emptyCwd `
    -WindowStyle Hidden -PassThru -RedirectStandardOutput $contractOut -RedirectStandardError $contractErr
if (-not $contract.WaitForExit(30000)) { $contract.Kill(); throw 'Embedded host-contract check timed out.' }
$contract.Refresh()
if ($contract.ExitCode -ne 0 -or (Get-Content -LiteralPath $contractOut -Raw) -notmatch 'runs inside sketch-app.exe') {
    throw "Embedded host-contract check failed: $(Get-Content -LiteralPath $contractErr -Raw)"
}
$result = [ordered]@{
    checkedAtUtc=[DateTime]::UtcNow.ToString('o'); archive=$ArchivePath
    archiveSha256=(Get-FileHash -LiteralPath $ArchivePath -Algorithm SHA256).Hash
    executableSha256=$exeHash; windowsSubsystem=$subsystem; manifestFilesVerified=$manifest.files.Count
    installation=$positive.stdout.Trim(); missingLibraryExitCode=$negative.exitCode
    embeddedHostContract=(Get-Content -LiteralPath $contractOut -Raw).Trim()
}
$result | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $work 'result.json') -Encoding utf8
$result | ConvertTo-Json
