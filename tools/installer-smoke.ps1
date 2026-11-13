param([Parameter(Mandatory)][string]$InstallerPath)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'assert-release-executable.ps1')
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$msi = (Resolve-Path -LiteralPath $InstallerPath).Path
$work = Join-Path $repoRoot "target\sketchy-msi-check\$PID"
$unpacked = Join-Path $work 'unpacked'
New-Item -ItemType Directory -Path $work,$unpacked -Force | Out-Null

# Read the database and unpack its embedded cabinet without registering a product.
$installer = New-Object -ComObject WindowsInstaller.Installer
$database = $installer.OpenDatabase($msi, 0)
$summary = $database.SummaryInformation(0)
if ($summary.Property(7) -notmatch '^x64;') { throw 'Expected an x64 Windows Installer package.' }
$properties = $database.OpenView('SELECT `Property`, `Value` FROM `Property`')
$properties.Execute()
$values = @{}
while ($record = $properties.Fetch()) { $values[$record.StringData(1)] = $record.StringData(2) }
$properties.Close()
if ($values.ProductName -ne 'Sketch Rust Sketchy Preview' -or $values.ALLUSERS) {
    throw 'Expected a separate per-user Sketchy preview.'
}
$media = $database.OpenView('SELECT `Cabinet` FROM `Media`')
$media.Execute()
$cabinet = $media.Fetch().StringData(1)
$media.Close()
if (-not $cabinet.StartsWith('#')) { throw 'Expected an embedded installer cabinet.' }
$stream = $database.OpenView('SELECT `Name`, `Data` FROM `_Streams` WHERE `Name` = ?')
$param = $installer.CreateRecord(1)
$param.StringData(1) = $cabinet.Substring(1)
$stream.Execute($param)
$cabRecord = $stream.Fetch()
if (-not $cabRecord) { throw 'Embedded cabinet stream was not found.' }
$cabPath = Join-Path $work 'payload.cab'
$output = [IO.File]::Create($cabPath)
try {
    # Windows Installer automation format 1 returns one character per byte.
    $left = $cabRecord.DataSize(2)
    while ($left -gt 0) {
        $size = [Math]::Min($left, 65536)
        $chunk = $cabRecord.ReadStream(2, $size, 1)
        if ($chunk.Length -ne $size) { throw 'Could not read the complete MSI stream.' }
        $bytes = [Text.Encoding]::Latin1.GetBytes($chunk)
        $output.Write($bytes, 0, $bytes.Length)
        $left -= $size
    }
} finally { $output.Dispose(); $stream.Close() }
& expand.exe $cabPath '-F:*' $unpacked | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Could not expand the installer cabinet.' }

$files = $database.OpenView('SELECT `File`, `FileName` FROM `File`')
$files.Execute()
while ($record = $files.Fetch()) {
    $id = $record.StringData(1)
    $name = ($record.StringData(2) -split '\|')[-1]
    if ($name -ne [IO.Path]::GetFileName($name)) { throw 'Unexpected path in MSI File table.' }
    Move-Item -LiteralPath (Join-Path $unpacked $id) -Destination (Join-Path $unpacked $name)
}
$files.Close()
Assert-ReleaseExecutable (Join-Path $unpacked 'sketch-app.exe')
$manifest = Get-Content -LiteralPath (Join-Path $unpacked 'manifest.json') -Raw | ConvertFrom-Json
foreach ($entry in $manifest.files) {
    if ((Get-FileHash -LiteralPath (Join-Path $unpacked $entry.name) -Algorithm SHA256).Hash -ne $entry.sha256) {
        throw "MSI payload differs from release manifest: $($entry.name)"
    }
}

$sample = Join-Path $work 'disposable.sketchy.json'
Copy-Item -LiteralPath (Join-Path $unpacked 'try-sketchy.sketchy.json') -Destination $sample
$stdout = Join-Path $work 'sketchy-stdout.txt'
$stderr = Join-Path $work 'sketchy-stderr.txt'
$probe = Start-Process -FilePath (Join-Path $unpacked 'sketch-app.exe') `
    -ArgumentList "--check-sketchy-workfile `"$sample`"" -WorkingDirectory $work `
    -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
if (-not $probe.WaitForExit(30000)) { $probe.Kill(); throw 'Sketchy workfile check timed out.' }
$probe.Refresh()
$result = Get-Content -LiteralPath $stdout -Raw
if ($probe.ExitCode -ne 0 -or $result -notmatch 'Sketchy open, edit, save in place, and reopen') {
    throw "MSI Sketchy workfile check failed: $(Get-Content -LiteralPath $stderr -Raw)"
}
Write-Output "$($values.ProductName) $($values.ProductVersion): verified $($manifest.files.Count) files. $($result.Trim())"
