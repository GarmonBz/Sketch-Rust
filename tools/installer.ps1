# Builds dist\SketchRust-Setup-<version>.exe: the per-user installer
# (crates/sketch-setup) with the staged package embedded in it.
#
#   tools\installer.ps1                 package first, then build the installer
#   tools\installer.ps1 -SkipPackage    reuse today's staged package
#
# The finished installer is checked with its own --selftest, which installs
# into, verifies and removes from a scratch folder under target\ using its
# own shortcut folders and registry key. It does not touch the Start Menu,
# the desktop or a real installation.
param([switch]$SkipPackage)
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $repoRoot
try {
    $version = [regex]::Match((Get-Content -LiteralPath 'Cargo.toml' -Raw), 'version\s*=\s*"([^"]+)"').Groups[1].Value
    if (-not $SkipPackage) {
        try { & (Join-Path $PSScriptRoot 'package.ps1') -NativePreview | Out-Null }
        catch { Write-Warning "package.ps1: $($_.Exception.Message)" }
    }
    $stage = Get-ChildItem -LiteralPath 'target\package' -Directory -Filter "SketchRust-$version-*" |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if (-not $stage) { throw "No staged package for $version under target\package. Run tools\package.ps1 first." }
    if (-not (Test-Path -LiteralPath (Join-Path $stage.FullName 'sketch-app.exe'))) { throw "$($stage.FullName) has no sketch-app.exe." }

    $env:SKETCH_SETUP_PAYLOAD = $stage.FullName
    try {
        cargo build --release -p sketch-setup --offline
        if ($LASTEXITCODE -ne 0) { throw 'Installer build failed.' }
    } finally { Remove-Item Env:SKETCH_SETUP_PAYLOAD -ErrorAction SilentlyContinue }

    $dist = Join-Path $repoRoot 'dist'
    New-Item -ItemType Directory -Path $dist -Force | Out-Null
    $setup = Join-Path $dist "SketchRust-Setup-$version.exe"
    Copy-Item -LiteralPath 'target\release\sketch-setup.exe' -Destination $setup -Force

    $scratch = Join-Path $repoRoot 'target\setup-selftest'
    if (Test-Path -LiteralPath $scratch) { Remove-Item -LiteralPath $scratch -Recurse -Force }
    New-Item -ItemType Directory -Path $scratch -Force | Out-Null
    $run = Start-Process -FilePath $setup -ArgumentList '--selftest', "`"$scratch`"" -Wait -PassThru
    $report = Join-Path $scratch 'selftest.txt'
    $result = if (Test-Path -LiteralPath $report) { Get-Content -LiteralPath $report -Raw } else { 'no report written' }
    if ($run.ExitCode -ne 0) { throw "Installer self-test failed: $result" }

    "$((Get-FileHash -LiteralPath $setup -Algorithm SHA256).Hash.ToLowerInvariant())  $([IO.Path]::GetFileName($setup))" |
        Set-Content -LiteralPath "$setup.sha256" -Encoding ascii
    Write-Output "Self-test: $result"
    Write-Output "Payload: $($stage.Name)"
    Write-Output "Installer: $setup ($([math]::Round((Get-Item -LiteralPath $setup).Length / 1MB, 1)) MB)"
} finally { Pop-Location }
