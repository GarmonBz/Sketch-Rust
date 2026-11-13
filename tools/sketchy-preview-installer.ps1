param(
    [Parameter(Mandatory)][string]$StageDirectory,
    [string]$OutputPath,
    [string]$WixDirectory
)
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$stage = (Resolve-Path -LiteralPath $StageDirectory).Path
$manifest = Get-Content -LiteralPath (Join-Path $stage 'manifest.json') -Raw | ConvertFrom-Json
if (-not $OutputPath) {
    $OutputPath = Join-Path $repoRoot "dist\SketchRust-$($manifest.version)-sketchy-preview-x64.msi"
}
$OutputPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputPath)
if (-not $WixDirectory) { $WixDirectory = Join-Path $repoRoot 'target\wix-tools\x' }
$wix = Join-Path $WixDirectory 'wix\tools\net6.0\any\wix.dll'
$ui = Join-Path $WixDirectory 'wixtoolset.ui.wixext\wixext5\WixToolset.UI.wixext.dll'
if (-not (Test-Path -LiteralPath $wix -PathType Leaf) -or -not (Test-Path -LiteralPath $ui -PathType Leaf)) {
    throw 'Extract the WiX 5.0.2 wix and wixtoolset.ui.wixext NuGet packages into target\wix-tools\x or pass -WixDirectory.'
}

$work = Join-Path $repoRoot 'target\sketchy-installer'
New-Item -ItemType Directory -Path $work,(Split-Path -Parent $OutputPath) -Force | Out-Null
$files = @('<?xml version="1.0" encoding="utf-8"?>',
    '<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">', '  <Fragment>',
    '    <ComponentGroup Id="AppFiles" Directory="INSTALLFOLDER">')
foreach ($name in @($manifest.files.name) + @('manifest.json')) {
    $source = [IO.Path]::GetFullPath((Join-Path $stage $name))
    if (-not $source.StartsWith($stage + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Manifest entry is outside the staging folder: $name"
    }
    $entry = $manifest.files | Where-Object name -eq $name
    if ($entry -and (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -ne $entry.sha256) {
        throw "Staged file differs from its manifest: $name"
    }
    $id = 'F_' + ($name -replace '[^A-Za-z0-9]', '_')
    $escaped = [Security.SecurityElement]::Escape($source)
    $escapedName = [Security.SecurityElement]::Escape($name)
    $files += "      <Component Id=`"C_$id`"><File Id=`"$id`" Name=`"$escapedName`" Source=`"$escaped`" /></Component>"
}
$files += '    </ComponentGroup>', '  </Fragment>', '</Wix>'
$fileSource = Join-Path $work 'Files.wxs'
Set-Content -LiteralPath $fileSource -Value $files -Encoding utf8

$notice = Join-Path $work 'Notice.rtf'
$terms = Get-Content -LiteralPath (Join-Path $repoRoot 'LICENSE') -Raw
$terms = $terms.Replace('\','\\').Replace('{','\{').Replace('}','\}') -replace '\r?\n', '\par '
Set-Content -LiteralPath $notice -Encoding ascii -Value ('{\rtf1\ansi\deff0{\fonttbl{\f0 Segoe UI;}}\f0\fs20 ' + $terms + '}')
$definition = Join-Path $repoRoot 'installer\SketchyPreview.wxs'
$icon = Join-Path $repoRoot 'assets\app-icon.ico'
& dotnet $wix build $definition $fileSource -arch x64 -ext $ui `
    -d "Version=$($manifest.version)" -d "IconFile=$icon" -d "NoticeRtf=$notice" `
    -intermediateFolder $work -pdbtype none -o $OutputPath
if ($LASTEXITCODE -ne 0) { throw 'WiX failed to build the installer.' }
"$((Get-FileHash -LiteralPath $OutputPath -Algorithm SHA256).Hash.ToLowerInvariant())  $([IO.Path]::GetFileName($OutputPath))" |
    Set-Content -LiteralPath "$OutputPath.sha256" -Encoding ascii
Write-Output "Installer: $OutputPath"
