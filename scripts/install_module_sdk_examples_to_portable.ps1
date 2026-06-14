# Installs the packaged Module SDK showcase examples into a local portable tree
# for manual UI smoke checks while preserving module-owned data directories.
param(
    [string]$PortableDir = "dist\NetStitch-win64-portable",
    [string]$PackagesDir = "docs\module-sdk\packages",
    [string]$StageDir = "temp\module-install-refresh",
    [string]$Platform = "windows-x86_64"
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$RepoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path

function Assert-InRepo {
    param([string]$Path)
    $candidate = if ([System.IO.Path]::IsPathRooted($Path)) {
        $Path
    } else {
        Join-Path $RepoRoot $Path
    }
    $fullPath = [System.IO.Path]::GetFullPath($candidate)
    $repoFull = [System.IO.Path]::GetFullPath($RepoRoot).TrimEnd(
        [System.IO.Path]::DirectorySeparatorChar,
        [System.IO.Path]::AltDirectorySeparatorChar
    )
    if ($fullPath -ne $repoFull -and -not $fullPath.StartsWith(
        "$repoFull$([System.IO.Path]::DirectorySeparatorChar)",
        [System.StringComparison]::OrdinalIgnoreCase
    )) {
        throw "Refusing to touch path outside repo: $fullPath"
    }
    return $fullPath
}

function Assert-ChildPath {
    param(
        [string]$Child,
        [string]$Parent
    )
    $childFull = [System.IO.Path]::GetFullPath($Child)
    $parentFull = [System.IO.Path]::GetFullPath($Parent).TrimEnd(
        [System.IO.Path]::DirectorySeparatorChar,
        [System.IO.Path]::AltDirectorySeparatorChar
    )
    if ($childFull -ne $parentFull -and -not $childFull.StartsWith(
        "$parentFull$([System.IO.Path]::DirectorySeparatorChar)",
        [System.StringComparison]::OrdinalIgnoreCase
    )) {
        throw "Refusing to touch path outside expected parent: $childFull"
    }
}

function Reset-Directory {
    param([string]$Path)
    Assert-ChildPath -Child $Path -Parent $RepoRoot
    if (Test-Path -LiteralPath $Path) {
        Remove-Item -LiteralPath $Path -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $Path | Out-Null
}

function Test-PortableProcessRunning {
    param([string]$PortableRoot)
    $portableFull = [System.IO.Path]::GetFullPath($PortableRoot)
    $process = Get-Process NetStitch -ErrorAction SilentlyContinue |
        Where-Object {
            $_.Path -and
            [System.IO.Path]::GetFullPath($_.Path).StartsWith(
                $portableFull,
                [System.StringComparison]::OrdinalIgnoreCase
            )
        } |
        Select-Object -First 1
    return $null -ne $process
}

function Sync-ModulePackage {
    param(
        [string]$ModuleName,
        [string]$SourceModuleDir,
        [string]$PortableIntegrationsDir
    )

    if (-not (Test-Path -LiteralPath (Join-Path $SourceModuleDir "module.json"))) {
        throw "Extracted module package is missing module.json: $SourceModuleDir"
    }

    $destinationModuleDir = Join-Path $PortableIntegrationsDir $ModuleName
    New-Item -ItemType Directory -Force -Path $destinationModuleDir | Out-Null
    Assert-ChildPath -Child $destinationModuleDir -Parent $PortableIntegrationsDir

    foreach ($item in Get-ChildItem -LiteralPath $destinationModuleDir -Force -ErrorAction SilentlyContinue) {
        if ($item.Name -eq "data") {
            continue
        }
        Assert-ChildPath -Child $item.FullName -Parent $destinationModuleDir
        Remove-Item -LiteralPath $item.FullName -Recurse -Force
    }

    foreach ($item in Get-ChildItem -LiteralPath $SourceModuleDir -Force) {
        if ($item.Name -eq "data") {
            continue
        }
        Copy-Item -LiteralPath $item.FullName -Destination (Join-Path $destinationModuleDir $item.Name) -Recurse -Force
    }

    New-Item -ItemType Directory -Force -Path (Join-Path $destinationModuleDir "data") | Out-Null
    Write-Host "Installed $ModuleName into $destinationModuleDir"
}

$portableRoot = Assert-InRepo $PortableDir
$packagesRoot = Assert-InRepo $PackagesDir
$stageRoot = Assert-InRepo $StageDir

if (-not (Test-Path -LiteralPath $portableRoot)) {
    throw "Portable directory not found: $portableRoot"
}

if (Test-PortableProcessRunning -PortableRoot $portableRoot) {
    throw "NetStitch is running from $portableRoot. Close it before refreshing installed modules."
}

$portableIntegrationsDir = Join-Path $portableRoot "integrations"
New-Item -ItemType Directory -Force -Path $portableIntegrationsDir | Out-Null

Reset-Directory -Path $stageRoot

foreach ($moduleName in @("ui-entity-showcase-rust", "ui-entity-showcase-cpp")) {
    $archivePath = Join-Path $packagesRoot "$moduleName-$Platform.zip"
    if (-not (Test-Path -LiteralPath $archivePath)) {
        throw "Module package archive not found: $archivePath"
    }

    $moduleStageRoot = Join-Path $stageRoot "$moduleName-$Platform"
    Reset-Directory -Path $moduleStageRoot
    Expand-Archive -LiteralPath $archivePath -DestinationPath $moduleStageRoot -Force

    Sync-ModulePackage `
        -ModuleName $moduleName `
        -SourceModuleDir (Join-Path $moduleStageRoot $moduleName) `
        -PortableIntegrationsDir $portableIntegrationsDir
}
