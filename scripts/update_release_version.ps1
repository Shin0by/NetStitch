# Updates the tracked release version from the locally tested portable manifest.
param(
    [string]$ManifestPath = "dist\NetStitch-win64-portable\config\version-manifest.json",
    [string]$OutputPath = "config\release-version.json"
)

$ErrorActionPreference = "Stop"

Push-Location $PSScriptRoot\..
try {
    if (-not (Test-Path $ManifestPath)) {
        throw "Portable version manifest not found: $ManifestPath"
    }

    $manifest = Get-Content -LiteralPath $ManifestPath -Raw | ConvertFrom-Json
    $fullVersion = [string]$manifest.modules.app.full_version
    $packageVersion = [string]$manifest.package_version
    if (-not $fullVersion -or -not $packageVersion) {
        throw "Manifest must contain package_version and modules.app.full_version"
    }

    $parts = $fullVersion.Split(".")
    if ($parts.Length -ne 4) {
        throw "Full version must look like 1.1.0.440, got: $fullVersion"
    }
    $revision = 0
    if (-not [int]::TryParse($parts[3], [ref]$revision) -or $revision -le 0) {
        throw "Release revision must be a positive integer, got: $($parts[3])"
    }
    if ("$($parts[0]).$($parts[1]).$($parts[2])" -ne $packageVersion) {
        throw "Full version $fullVersion does not match package version $packageVersion"
    }

    $payload = [ordered]@{
        version = $fullVersion
        package_version = $packageVersion
        revision = $revision
        source_manifest = ($ManifestPath -replace "\\", "/")
    }

    $parent = Split-Path -Parent $OutputPath
    if ($parent) {
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }
    ($payload | ConvertTo-Json -Depth 4) | Set-Content -LiteralPath $OutputPath -Encoding UTF8
    Get-Item -LiteralPath $OutputPath | Select-Object FullName, Length, LastWriteTime
}
finally {
    Pop-Location
}
