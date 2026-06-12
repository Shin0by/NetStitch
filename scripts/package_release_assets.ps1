# Builds a portable package and creates zip release assets from it.
param(
    [string]$Version = "",
    [string]$PortableDir = "dist\NetStitch-win64-portable",
    [string]$ReleaseDir = "dist\release-assets",
    [string]$WinDivertDir = $env:NETSTITCH__WINDIVERT_DIR,
    [switch]$SkipPackage,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

function Assert-PathInside {
    param(
        [string]$Child,
        [string]$Parent
    )

    $childFull = [System.IO.Path]::GetFullPath($Child).TrimEnd('\')
    $parentFull = [System.IO.Path]::GetFullPath($Parent).TrimEnd('\')
    if (-not $childFull.StartsWith("$parentFull\", [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to modify path outside release folder: $childFull"
    }
}

function Get-PortableManifestVersion {
    param([string]$PortablePath)

    $manifestPath = Join-Path (Join-Path $PortablePath "config") "version-manifest.json"
    if (-not (Test-Path $manifestPath)) {
        return ""
    }

    try {
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        $appVersion = [string]$manifest.modules.app.full_version
        if ($appVersion.Trim()) {
            return $appVersion.Trim()
        }
    }
    catch {
        return ""
    }

    return ""
}

function Sync-ReleaseConnectorApps {
    param([string]$PortablePath)

    $sourceAppsDir = Join-Path (Join-Path (Join-Path (Get-Location) "resources") "connectors") "apps"
    $sourceIconDir = Join-Path (Join-Path (Get-Location) "resources") "connectors\icons"
    $targetAppsDir = Join-Path $PortablePath "apps"
    $targetIconDir = Join-Path $targetAppsDir "icons"
    if (-not (Test-Path $sourceAppsDir)) {
        throw "Missing shipped connector apps: $sourceAppsDir"
    }
    if (-not (Test-Path $sourceIconDir)) {
        throw "Missing shipped connector icons: $sourceIconDir"
    }

    New-Item -ItemType Directory -Force -Path $targetAppsDir | Out-Null
    foreach ($item in Get-ChildItem -LiteralPath $targetAppsDir -File -ErrorAction SilentlyContinue) {
        if ($item.Extension -in @(".app", ".toml")) {
            Assert-PathInside -Child $item.FullName -Parent $PortablePath
            Remove-Item -LiteralPath $item.FullName -Force
        }
    }
    foreach ($app in Get-ChildItem -LiteralPath $sourceAppsDir -Filter "*.app" -File) {
        $target = Join-Path $targetAppsDir $app.Name
        Assert-PathInside -Child $target -Parent $targetAppsDir
        Copy-Item -LiteralPath $app.FullName -Destination $target -Force
    }
    foreach ($docName in @("README_RU.txt", "README_EN.txt")) {
        $docPath = Join-Path $sourceAppsDir $docName
        if (Test-Path $docPath) {
            Copy-Item -LiteralPath $docPath -Destination (Join-Path $targetAppsDir $docName) -Force
        }
    }

    if (Test-Path $targetIconDir) {
        Assert-PathInside -Child $targetIconDir -Parent $PortablePath
        Remove-Item -LiteralPath $targetIconDir -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $targetIconDir | Out-Null

    foreach ($icon in Get-ChildItem -LiteralPath $sourceIconDir -Filter "*.svg" -File) {
        $target = Join-Path $targetIconDir $icon.Name
        Assert-PathInside -Child $target -Parent $targetIconDir
        Copy-Item -LiteralPath $icon.FullName -Destination $target -Force
    }
}

function Remove-ReleaseModuleRuntimeData {
    param([string]$PortablePath)

    $integrationsDir = Join-Path $PortablePath "integrations"
    if (-not (Test-Path $integrationsDir)) {
        return
    }

    foreach ($moduleDir in Get-ChildItem -LiteralPath $integrationsDir -Directory -ErrorAction SilentlyContinue) {
        $runtimeDataDir = Join-Path $moduleDir.FullName "data"
        if (-not (Test-Path $runtimeDataDir)) {
            continue
        }
        Assert-PathInside -Child $runtimeDataDir -Parent $moduleDir.FullName
        Remove-Item -LiteralPath $runtimeDataDir -Recurse -Force
    }
}

function New-ReleaseStagingPortable {
    param(
        [string]$PortablePath,
        [string]$ReleaseRoot
    )

    $stagingRoot = Join-Path $ReleaseRoot ".release-staging"
    if (Test-Path $stagingRoot) {
        Assert-PathInside -Child $stagingRoot -Parent $ReleaseRoot
        Remove-Item -LiteralPath $stagingRoot -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $stagingRoot | Out-Null
    Copy-Item -LiteralPath $PortablePath -Destination $stagingRoot -Recurse -Force
    $stagedPortablePath = Join-Path $stagingRoot (Split-Path -Leaf $PortablePath)
    Sync-ReleaseConnectorApps -PortablePath $stagedPortablePath
    $stagedStoragePath = Join-Path $stagedPortablePath "storage"
    if (Test-Path $stagedStoragePath) {
        Assert-PathInside -Child $stagedStoragePath -Parent $stagedPortablePath
        Remove-Item -LiteralPath $stagedStoragePath -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path (Join-Path $stagedStoragePath "exports") | Out-Null
    Remove-ReleaseModuleRuntimeData -PortablePath $stagedPortablePath
    $stagedPortablePath
}

function Assert-WindowsReleaseRuntimeFiles {
    param([string]$PortablePath)

    $required = @(
        "WinDivert.dll",
        "WinDivert64.sys",
        "libs\netstitch-tool\bin\windows-x86_64\netstitch_tool.dll",
        "libs\netstitch-watcher\bin\windows-x86_64\netstitch_watcher.dll"
    )

    $missing = @()
    foreach ($relativePath in $required) {
        if (-not (Test-Path (Join-Path $PortablePath $relativePath))) {
            $missing += $relativePath
        }
    }
    if ($missing) {
        throw "Windows portable release is missing required runtime files:`n$($missing -join "`n")"
    }
}

Push-Location $PSScriptRoot\..
try {
    $releaseRoot = Join-Path (Get-Location) $ReleaseDir
    New-Item -ItemType Directory -Force -Path $releaseRoot | Out-Null
    foreach ($item in Get-ChildItem -LiteralPath $releaseRoot -Filter "NetStitch-win64-portable-*.zip" -Force -ErrorAction SilentlyContinue) {
        Assert-PathInside -Child $item.FullName -Parent $releaseRoot
        Remove-Item -LiteralPath $item.FullName -Recurse -Force
    }

    if (-not $SkipPackage) {
        $packageArgs = @{
            OutputDir = $PortableDir
            WinDivertDir = $WinDivertDir
        }
        if ($SkipBuild) {
            $packageArgs.SkipBuild = $true
        }
        & (Join-Path $PSScriptRoot "package_portable.ps1") @packageArgs
    }

    $portablePath = (Resolve-Path $PortableDir).Path
    $resolvedVersion = if ($Version.Trim()) { $Version.Trim() } else { Get-PortableManifestVersion -PortablePath $portablePath }
    if (-not $resolvedVersion) {
        $resolvedVersion = "dev"
    }
    $safeVersion = $resolvedVersion -replace '[^A-Za-z0-9_.-]', '-'
    $zipPath = Join-Path $releaseRoot "NetStitch-win64-portable-$safeVersion.zip"
    $stagedPortablePath = New-ReleaseStagingPortable -PortablePath $portablePath -ReleaseRoot $releaseRoot
    Assert-WindowsReleaseRuntimeFiles -PortablePath $stagedPortablePath
    Compress-Archive -Path $stagedPortablePath -DestinationPath $zipPath -Force
    Remove-Item -LiteralPath (Split-Path -Parent $stagedPortablePath) -Recurse -Force

    Get-Item -LiteralPath $zipPath | Select-Object FullName, Length, LastWriteTime
}
finally {
    Pop-Location
}
