function Assert-ReleaseExecutable([string]$Path) {
    # The old Windows binary contains these ASCII names in .NET metadata.
    # Catch a stale executable even when a packager was told to skip the build.
    $bytes = [IO.File]::ReadAllBytes($Path)
    $metadata = [Text.Encoding]::ASCII.GetString($bytes)
    foreach ($marker in 'Apex.Integration.DotNet', 'SketchRust.HostContract') {
        if ($metadata.IndexOf($marker, [StringComparison]::OrdinalIgnoreCase) -ge 0) {
            throw "The executable contains the retired Apex component. Rebuild from current source: $Path"
        }
    }
}
