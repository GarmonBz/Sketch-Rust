param()
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $repoRoot
try {
    $selection = Get-Content -LiteralPath 'refs/caddillo/selection.json' -Raw | ConvertFrom-Json
    $archive = Join-Path $repoRoot 'refs/caddillo/caddillo-blocks-library.zip'
    if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $selection.sha256) {
        throw 'CADdillo source archive does not match the verified download.'
    }
    $extract = Join-Path $repoRoot 'target/caddillo-source'
    Expand-Archive -LiteralPath $archive -DestinationPath $extract -Force
    $env:CARGO_INCREMENTAL = '0'
    cargo run -p sketch-core --example import_caddillo --offline -- (Join-Path $extract 'caddillo-blocks')
    if ($LASTEXITCODE -ne 0) { throw 'Catalog import failed.' }
} finally { Pop-Location }
