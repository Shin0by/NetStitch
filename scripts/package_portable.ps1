# Builds a local portable folder with the NetStitch app, native runtime modules, and optional WinDivert runtime files.
param(
    [string]$Configuration = "release",
    [string]$TargetDir = "",
    [string]$OutputDir = "dist\NetStitch-win64-portable",
    [string]$WinDivertDir = $env:NETSTITCH__WINDIVERT_DIR,
    [string]$IntegrationTargetOs = $env:NETSTITCH__INTEGRATION_TARGET_OS,
    [string]$IntegrationTargetArch = $env:NETSTITCH__INTEGRATION_TARGET_ARCH,
    [string[]]$PackageModules = @(),
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

function Test-SamePath {
    param(
        [string]$Left,
        [string]$Right
    )

    if (-not $Left -or -not $Right) {
        return $false
    }

    $comparison = if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
        [System.StringComparison]::OrdinalIgnoreCase
    } else {
        [System.StringComparison]::Ordinal
    }

    $leftFull = [System.IO.Path]::GetFullPath($Left).TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    $rightFull = [System.IO.Path]::GetFullPath($Right).TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    return [string]::Equals($leftFull, $rightFull, $comparison)
}

function Get-PackageVersion {
    param([string]$CargoTomlPath)

    if (-not (Test-Path $CargoTomlPath)) {
        throw "Cargo.toml not found: $CargoTomlPath"
    }

    foreach ($line in Get-Content -LiteralPath $CargoTomlPath) {
        if ($line -match '^\s*version\s*=\s*"([^"]+)"\s*$') {
            return $Matches[1]
        }
    }

    throw "Failed to read package version from $CargoTomlPath"
}

function Read-VersionManifest {
    param([string]$ManifestPath)

    if (-not (Test-Path $ManifestPath)) {
        return @{
            package_version = ""
            modules = @{}
        }
    }

    try {
        $rawText = Get-Content -LiteralPath $ManifestPath -Raw
        try {
            $raw = $rawText | ConvertFrom-Json -AsHashtable
        }
        catch {
            $parsed = $rawText | ConvertFrom-Json
            $modules = @{}
            if ($null -ne $parsed.modules) {
                foreach ($property in $parsed.modules.PSObject.Properties) {
                    $entry = $property.Value
                    $modules[$property.Name] = @{
                        artifact = [string]$entry.artifact
                        revision = [int64]$entry.revision
                        full_version = [string]$entry.full_version
                        artifact_hash = [string]$entry.artifact_hash
                    }
                }
            }
            $raw = @{
                package_version = [string]$parsed.package_version
                generated_at = [string]$parsed.generated_at
                modules = $modules
            }
        }
        if ($null -eq $raw) {
            throw "manifest is empty"
        }
        if ($null -eq $raw.modules) {
            $raw.modules = @{}
        }
        return $raw
    }
    catch {
        return @{
            package_version = ""
            modules = @{}
        }
    }
}

function Get-VersionManifestMaxRevision {
    param([hashtable]$Manifest)

    $maxRevision = 0L
    if ($Manifest.modules -is [System.Collections.IDictionary]) {
        foreach ($entry in $Manifest.modules.Values) {
            $entryRevision = 0L
            try {
                if ($entry -is [System.Collections.IDictionary]) {
                    $entryRevision = [int64]$entry["revision"]
                } else {
                    $entryRevision = [int64]$entry.revision
                }
            }
            catch {
                $entryRevision = 0L
            }
            if ($entryRevision -gt $maxRevision) {
                $maxRevision = $entryRevision
            }
        }
    }
    return $maxRevision
}

function Resolve-PreviousVersionManifest {
    param([string[]]$ManifestPaths)

    $bestManifest = @{
        package_version = ""
        modules = @{}
    }
    $bestRevision = 0L

    foreach ($manifestPath in $ManifestPaths) {
        $candidate = Read-VersionManifest -ManifestPath $manifestPath
        $candidateRevision = Get-VersionManifestMaxRevision -Manifest $candidate
        if ($candidateRevision -gt $bestRevision) {
            $bestManifest = $candidate
            $bestRevision = $candidateRevision
        }
    }

    return $bestManifest
}

function New-VersionModuleEntry {
    param(
        [int64]$Revision,
        [string]$ArtifactName,
        [string]$ArtifactPath,
        [string]$PackageVersion
    )

    $artifactHash = (Get-FileHash -LiteralPath $ArtifactPath -Algorithm SHA256).Hash.ToLowerInvariant()

    return [ordered]@{
        artifact = $ArtifactName
        revision = $Revision
        full_version = "$PackageVersion.$Revision"
        artifact_hash = $artifactHash
    }
}

function Get-NextPackageRevision {
    param(
        [hashtable]$PreviousManifest,
        [string]$RevisionPath
    )

    $previousRevision = 0

    if (Test-Path $RevisionPath) {
        try {
            $rawRevision = (Get-Content -LiteralPath $RevisionPath -Raw).Trim()
            if ($rawRevision) {
                $previousRevision = [int64]$rawRevision
            }
        }
        catch {
            $previousRevision = 0
        }
    }

    if ($PreviousManifest.modules -is [System.Collections.IDictionary]) {
        foreach ($entry in $PreviousManifest.modules.Values) {
            $entryRevision = 0
            if ($entry -is [System.Collections.IDictionary]) {
                $entryRevision = [int64]$entry["revision"]
            } else {
                $entryRevision = [int64]$entry.revision
            }
            if ($entryRevision -gt $previousRevision) {
                $previousRevision = $entryRevision
            }
        }
    }

    return ($previousRevision + 1)
}

function Resolve-PackageRevision {
    param(
        [hashtable]$PreviousManifest,
        [string]$RevisionPath
    )

    $explicitRevision = $env:NETSTITCH__PACKAGE_REVISION
    if ($explicitRevision -and $explicitRevision.Trim()) {
        $parsedRevision = 0L
        if (-not [int64]::TryParse($explicitRevision.Trim(), [ref]$parsedRevision) -or $parsedRevision -le 0) {
            throw "NETSTITCH__PACKAGE_REVISION must be a positive integer"
        }
        return $parsedRevision
    }

    return Get-NextPackageRevision `
        -PreviousManifest $PreviousManifest `
        -RevisionPath $RevisionPath
}

function Write-PackageRevision {
    param(
        [string]$RevisionPath,
        [int64]$Revision
    )

    $parent = Split-Path -Parent $RevisionPath
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    Set-Content -LiteralPath $RevisionPath -Value $Revision -Encoding UTF8
}

function Write-VersionManifest {
    param(
        [string]$ManifestPath,
        [string]$PackageVersion,
        [hashtable]$Modules
    )

    $payload = [ordered]@{
        package_version = $PackageVersion
        generated_at = (Get-Date).ToUniversalTime().ToString("o")
        modules = $Modules
    }

    $parent = Split-Path -Parent $ManifestPath
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    ($payload | ConvertTo-Json -Depth 6) | Set-Content -LiteralPath $ManifestPath -Encoding UTF8
}

function Get-SafeProcessPath {
    param([System.Diagnostics.Process]$Process)

    try {
        return $Process.Path
    }
    catch {
        return ""
    }
}

function Assert-PathInside {
    param(
        [string]$Child,
        [string]$Parent
    )

    $comparison = if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
        [System.StringComparison]::OrdinalIgnoreCase
    } else {
        [System.StringComparison]::Ordinal
    }

    $childFull = [System.IO.Path]::GetFullPath($Child).TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    $parentFull = [System.IO.Path]::GetFullPath($Parent).TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    $relative = [System.IO.Path]::GetRelativePath($parentFull, $childFull)
    if ($relative -eq "." -or ($relative -and -not $relative.StartsWith("..$([System.IO.Path]::DirectorySeparatorChar)", $comparison) -and $relative -ne "..")) {
        return
    }
    if (-not $childFull.StartsWith($parentFull + [System.IO.Path]::DirectorySeparatorChar, $comparison)) {
        throw "Refusing to modify path outside portable folder: $childFull"
    }
}

function Assert-ReleaseArtifactsAreFree {
    param(
        [string]$UiPath
    )

    $runningUi = Get-Process NetStitch -ErrorAction SilentlyContinue | Where-Object {
        Test-SamePath (Get-SafeProcessPath $_) $UiPath
    }
    if ($runningUi) {
        $ids = ($runningUi | Select-Object -ExpandProperty Id) -join ", "
        throw "Close the running NetStitch app before packaging. Blocking process id(s): $ids"
    }
}

function Clear-PortablePublishRoot {
    param([string]$PortableDir)

    $allowedNames = @(
        "NetStitch.exe",
        "NetStitch",
        "config",
        "integrations",
        "language",
        "libs",
        "apps",
        "resources",
        "storage",
        "WinDivert.dll",
        "WinDivert64.sys"
    )

    if (-not (Test-Path $PortableDir)) {
        return
    }

    foreach ($item in Get-ChildItem -LiteralPath $PortableDir -Force) {
        if ($allowedNames -contains $item.Name) {
            continue
        }
        Assert-PathInside -Child $item.FullName -Parent $PortableDir
        Remove-Item -LiteralPath $item.FullName -Recurse -Force
    }
}

function Clear-PortableIconCache {
    param([string]$IconDir)

    if (-not (Test-Path $IconDir)) {
        return
    }

    foreach ($item in Get-ChildItem -LiteralPath $IconDir -Force) {
        if ($item.PSIsContainer) {
            Assert-PathInside -Child $item.FullName -Parent $IconDir
            Remove-Item -LiteralPath $item.FullName -Recurse -Force
            continue
        }

        if (($item.Extension -ieq ".svg") -or ($item.Extension -ieq ".png")) {
            continue
        }

        Assert-PathInside -Child $item.FullName -Parent $IconDir
        Remove-Item -LiteralPath $item.FullName -Force
    }
}

function Set-SystemEventCleanupMarker {
    param([string]$StorageDir)

    $markerPath = Join-Path $StorageDir "clear-system-events-on-next-start"
    Set-Content `
        -LiteralPath $markerPath `
        -Value "clear system_events on next NetStitch startup" `
        -Encoding UTF8
}

function Sync-PortableConnectorIcons {
    param(
        [string]$SourceIconDir,
        [string]$IconDir
    )

    if (-not (Test-Path $SourceIconDir)) {
        return
    }

    New-Item -ItemType Directory -Force -Path $IconDir | Out-Null
    Clear-PortableIconCache -IconDir $IconDir
    foreach ($icon in Get-ChildItem -LiteralPath $SourceIconDir -Filter "*.svg" -File) {
        $target = Join-Path $IconDir $icon.Name
        Assert-PathInside -Child $target -Parent $IconDir
        Copy-Item -LiteralPath $icon.FullName -Destination $target -Force
    }
}

function Remove-EmptyPortableStorageObsoleteDirs {
    param([string]$StorageDir)

    $obsoleteDirs = @(
        "external-apps",
        "integrations"
    )

    foreach ($obsoleteName in $obsoleteDirs) {
        $obsoleteDir = Join-Path $StorageDir $obsoleteName
        if (-not (Test-Path $obsoleteDir)) {
            continue
        }
        if ((Get-ChildItem -LiteralPath $obsoleteDir -Force -ErrorAction SilentlyContinue | Select-Object -First 1)) {
            continue
        }
        Assert-PathInside -Child $obsoleteDir -Parent $StorageDir
        Remove-Item -LiteralPath $obsoleteDir -Force
    }
}

function Sync-PortableResources {
    param([string]$PortableDir)

    $resourcesDir = Join-Path $PortableDir "resources"
    if (Test-Path $resourcesDir) {
        Assert-PathInside -Child $resourcesDir -Parent $PortableDir
        Remove-Item -LiteralPath $resourcesDir -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $resourcesDir | Out-Null

    $sourceRoot = Get-Location
    $brandingImage = Join-Path (Join-Path (Join-Path $sourceRoot "resources") "branding") "images\NetStitch.png"
    $appIcon = Join-Path (Join-Path (Join-Path $sourceRoot "src") "netstitch-ui") "assets\shin0by.png"
    if (-not (Test-Path $brandingImage)) {
        throw "Missing branding image: $brandingImage"
    }
    if (-not (Test-Path $appIcon)) {
        throw "Missing app icon image: $appIcon"
    }

    Copy-Item -LiteralPath $brandingImage -Destination (Join-Path $resourcesDir "netstitch.png") -Force
    Copy-Item -LiteralPath $appIcon -Destination (Join-Path $resourcesDir "shin0by.png") -Force
}

function Test-WinDivertRuntimeDir {
    param([string]$CandidateDir)

    if (-not $CandidateDir -or -not (Test-Path $CandidateDir)) {
        return $false
    }

    return (Test-Path (Join-Path $CandidateDir "WinDivert.dll")) -and
        (Test-Path (Join-Path $CandidateDir "WinDivert64.sys"))
}

function Copy-RuntimeFileIfChanged {
    param(
        [Parameter(Mandatory = $true)]
        [string]$SourcePath,
        [Parameter(Mandatory = $true)]
        [string]$DestinationDir
    )

    $targetPath = Join-Path $DestinationDir ([System.IO.Path]::GetFileName($SourcePath))
    if (Test-Path $targetPath) {
        try {
            $sourceHash = (Get-FileHash -LiteralPath $SourcePath -Algorithm SHA256).Hash
            $targetHash = (Get-FileHash -LiteralPath $targetPath -Algorithm SHA256).Hash
            if ($sourceHash -eq $targetHash) {
                return
            }
        }
        catch {
            # Fall through to Copy-Item so changed or unreadable files still fail loudly.
        }
    }

    Copy-Item -LiteralPath $SourcePath -Destination $DestinationDir -Force
}

function Resolve-WinDivertRuntimeDir {
    param(
        [string]$ExplicitWinDivertDir
    )

    if (Test-WinDivertRuntimeDir -CandidateDir $ExplicitWinDivertDir) {
        return [System.IO.Path]::GetFullPath($ExplicitWinDivertDir)
    }

    $bundledRuntimeDir = Join-Path (Join-Path (Join-Path (Get-Location) "resources") "runtime") "windivert\windows-x86_64"
    if (Test-WinDivertRuntimeDir -CandidateDir $bundledRuntimeDir) {
        return [System.IO.Path]::GetFullPath($bundledRuntimeDir)
    }

    return ""
}

function Resolve-IntegrationTargetOs {
    param([string]$TargetOs)

    $normalized = ([string]$TargetOs).Trim().ToLowerInvariant()
    if ($normalized) {
        switch ($normalized) {
            "win" { return "windows" }
            "win32" { return "windows" }
            "windows" { return "windows" }
            "linux" { return "linux" }
            "darwin" { return "macos" }
            "mac" { return "macos" }
            "macos" { return "macos" }
            "osx" { return "macos" }
            default { return $normalized }
        }
    }

    if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
        return "windows"
    }
    if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Linux)) {
        return "linux"
    }
    if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::OSX)) {
        return "macos"
    }

    return ""
}

function Resolve-IntegrationTargetArch {
    param([string]$TargetArch)

    $normalized = ([string]$TargetArch).Trim().ToLowerInvariant()
    if ($normalized) {
        switch ($normalized) {
            "amd64" { return "x86_64" }
            "x64" { return "x86_64" }
            "x86_64" { return "x86_64" }
            "x86" { return "x86" }
            "i386" { return "x86" }
            "i686" { return "x86" }
            "arm64" { return "aarch64" }
            "aarch64" { return "aarch64" }
            "arm" { return "arm" }
            default { return $normalized }
        }
    }

    switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
        "X64" { return "x86_64" }
        "X86" { return "x86" }
        "Arm64" { return "aarch64" }
        "Arm" { return "arm" }
        default { return ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()).ToLowerInvariant() }
    }
}

function Resolve-PackageModuleFilter {
    param([string[]]$ExplicitModules)

    $modules = @()
    if ($ExplicitModules) {
        $modules += $ExplicitModules
    }
    if ($env:NETSTITCH__PACKAGE_MODULES) {
        $modules += ($env:NETSTITCH__PACKAGE_MODULES -split '[,;]' | Where-Object { $_.Trim() })
    }

    $normalized = @()
    foreach ($module in $modules) {
        $value = ([string]$module).Trim().ToLowerInvariant()
        if ($value) {
            $normalized += $value
        }
    }
    return $normalized
}

function Test-PackageModuleIncluded {
    param(
        [string]$ModuleName,
        [string[]]$IncludedModules
    )

    if (-not $IncludedModules -or $IncludedModules.Count -eq 0) {
        return $false
    }
    if ($IncludedModules -contains "*") {
        return $true
    }
    return $IncludedModules -contains $ModuleName.ToLowerInvariant()
}

function Get-IntegrationManifestLibraryPath {
    param(
        $Manifest,
        [string]$TargetOs,
        [string]$TargetArch
    )

    $paths = $Manifest.library_paths
    if ($null -ne $paths) {
        $platformKeys = @()
        if ($TargetOs -and $TargetArch) {
            $platformKeys += "$TargetOs-$TargetArch"
        }
        if ($TargetOs) {
            $platformKeys += $TargetOs
        }
        $platformKeys += "default"

        foreach ($key in $platformKeys) {
            $property = $paths.PSObject.Properties[$key]
            if ($property -and [string]$property.Value) {
                return [string]$property.Value
            }
        }
    }

    return ""
}

function Sync-PortableModuleAssets {
    param(
        [string]$SourceModuleDir,
        [string]$PortableModuleDir
    )

    $sourceAssetsDir = Join-Path $SourceModuleDir "assets"
    if (-not (Test-Path $sourceAssetsDir)) {
        return
    }

    $portableAssetsDir = Join-Path $PortableModuleDir "assets"
    Assert-PathInside -Child $portableAssetsDir -Parent $PortableModuleDir
    if (Test-Path $portableAssetsDir) {
        Remove-Item -LiteralPath $portableAssetsDir -Recurse -Force
    }
    Copy-Item -LiteralPath $sourceAssetsDir -Destination $portableAssetsDir -Recurse -Force
}

function Sync-PortableIntegrations {
    param(
        [string]$SourceRoot,
        [string]$PortableDir,
        [string]$TargetDir,
        [string]$Configuration,
        [string]$TargetOs,
        [string]$TargetArch,
        [string[]]$IncludedModules
    )

    $sourceIntegrationsDir = Join-Path $SourceRoot "integrations"
    if (-not (Test-Path $sourceIntegrationsDir)) {
        return
    }

    $portableIntegrationsDir = Join-Path $PortableDir "integrations"
    New-Item -ItemType Directory -Force -Path $portableIntegrationsDir | Out-Null

    $sourceModuleNames = @()
    foreach ($moduleRoot in Get-ChildItem -LiteralPath $sourceIntegrationsDir -Directory -ErrorAction SilentlyContinue) {
        if (Test-Path (Join-Path $moduleRoot.FullName "module.json")) {
            $sourceModuleNames += $moduleRoot.Name
        }
    }
    foreach ($portableModuleRoot in Get-ChildItem -LiteralPath $portableIntegrationsDir -Directory -ErrorAction SilentlyContinue) {
        if (($sourceModuleNames -contains $portableModuleRoot.Name) -and
            -not (Test-PackageModuleIncluded -ModuleName $portableModuleRoot.Name -IncludedModules $IncludedModules)) {
            Assert-PathInside -Child $portableModuleRoot.FullName -Parent $portableIntegrationsDir
            Remove-Item -LiteralPath $portableModuleRoot.FullName -Recurse -Force
        }
    }

    foreach ($moduleRoot in Get-ChildItem -LiteralPath $sourceIntegrationsDir -Directory -ErrorAction SilentlyContinue) {
        $manifestPath = Join-Path $moduleRoot.FullName "module.json"
        if (-not (Test-Path $manifestPath)) {
            continue
        }

        $moduleName = $moduleRoot.Name
        if (-not (Test-PackageModuleIncluded -ModuleName $moduleName -IncludedModules $IncludedModules)) {
            continue
        }
        $portableModuleDir = Join-Path $portableIntegrationsDir $moduleName
        New-Item -ItemType Directory -Force -Path $portableModuleDir | Out-Null
        Copy-Item -LiteralPath $manifestPath -Destination (Join-Path $portableModuleDir "module.json") -Force

        $portableDataDir = Join-Path $portableModuleDir "data"
        New-Item -ItemType Directory -Force -Path $portableDataDir | Out-Null
        Sync-PortableModuleAssets `
            -SourceModuleDir $moduleRoot.FullName `
            -PortableModuleDir $portableModuleDir

        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        $libraryPath = Get-IntegrationManifestLibraryPath `
            -Manifest $manifest `
            -TargetOs $TargetOs `
            -TargetArch $TargetArch
        if (-not $libraryPath) {
            continue
        }

        $libraryFileName = [System.IO.Path]::GetFileName($libraryPath)
        if (-not $libraryFileName) {
            throw "Invalid integration library path in $manifestPath"
        }

        $portableLibraryPath = Join-Path $portableModuleDir $libraryPath
        Assert-PathInside -Child $portableLibraryPath -Parent $portableModuleDir
        $portableLibraryDir = Split-Path -Parent $portableLibraryPath
        New-Item -ItemType Directory -Force -Path $portableLibraryDir | Out-Null

        $builtLibraryPath = Join-Path $TargetDir $libraryFileName
        $moduleTargetLibraryPath = Join-Path (Join-Path (Join-Path $moduleRoot.FullName "target") $Configuration) $libraryFileName
        $sourceLibraryPath = Join-Path $moduleRoot.FullName $libraryPath
        if (Test-Path $moduleTargetLibraryPath) {
            Copy-Item -LiteralPath $moduleTargetLibraryPath -Destination $portableLibraryPath -Force
        } elseif (Test-Path $builtLibraryPath) {
            Copy-Item -LiteralPath $builtLibraryPath -Destination $portableLibraryPath -Force
        } elseif (Test-Path $sourceLibraryPath) {
            Copy-Item -LiteralPath $sourceLibraryPath -Destination $portableLibraryPath -Force
        } else {
            Write-Warning "Integration module '$moduleName' declares '$libraryPath', but the native library was not found in '$TargetDir' or its module target directory."
        }
    }
}

function Build-PortableIntegrationWorkspaces {
    param(
        [string]$SourceRoot,
        [string]$Configuration,
        [string[]]$IncludedModules
    )

    $sourceIntegrationsDir = Join-Path $SourceRoot "integrations"
    if (-not (Test-Path $sourceIntegrationsDir)) {
        return
    }

    foreach ($moduleRoot in Get-ChildItem -LiteralPath $sourceIntegrationsDir -Directory -ErrorAction SilentlyContinue) {
        $moduleName = $moduleRoot.Name
        if (-not (Test-PackageModuleIncluded -ModuleName $moduleName -IncludedModules $IncludedModules)) {
            continue
        }
        $manifestPath = Join-Path $moduleRoot.FullName "module.json"
        $cargoManifestPath = Join-Path $moduleRoot.FullName "Cargo.toml"
        if (-not (Test-Path $manifestPath) -or -not (Test-Path $cargoManifestPath)) {
            continue
        }

        $moduleTargetDir = Join-Path $moduleRoot.FullName "target"
        cargo build --manifest-path $cargoManifestPath --target-dir $moduleTargetDir --$Configuration -j 20
    }
}

function Get-NativeToolLibraryName {
    param([string]$TargetOs)

    switch ($TargetOs) {
        "windows" { return "netstitch_tool.dll" }
        "linux" { return "libnetstitch_tool.so" }
        "macos" { return "libnetstitch_tool.dylib" }
        default { return "" }
    }
}

function Get-NativeWatcherLibraryName {
    param([string]$TargetOs)

    switch ($TargetOs) {
        "windows" { return "netstitch_watcher.dll" }
        "linux" { return "libnetstitch_watcher.so" }
        "macos" { return "libnetstitch_watcher.dylib" }
        default { return "" }
    }
}

function Get-AppArtifactName {
    param([string]$TargetOs)

    switch ($TargetOs) {
        "windows" { return "NetStitch.exe" }
        default { return "NetStitch" }
    }
}

function Sync-PortableNativeModule {
    param(
        [string]$PortableDir,
        [string]$TargetDir,
        [string]$TargetOs,
        [string]$TargetArch,
        [string]$ModuleId,
        [string]$DisplayName,
        [string]$LibraryName
    )

    if (-not $LibraryName) {
        return
    }

    $sourceLibrary = Join-Path $TargetDir $LibraryName
    if (-not (Test-Path $sourceLibrary)) {
        throw "Required native module library is missing: $sourceLibrary"
    }

    $platformKey = if ($TargetOs -and $TargetArch) { "$TargetOs-$TargetArch" } else { $TargetOs }
    $nativeModulesDir = Join-Path $PortableDir "libs"
    $moduleDir = Join-Path $nativeModulesDir $ModuleId
    $moduleBinDir = Join-Path (Join-Path $moduleDir "bin") $platformKey
    Assert-PathInside -Child $moduleDir -Parent $PortableDir
    if (Test-Path $moduleDir) {
        Remove-Item -LiteralPath $moduleDir -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $moduleBinDir | Out-Null
    Copy-Item -LiteralPath $sourceLibrary -Destination (Join-Path $moduleBinDir $LibraryName) -Force

    $manifest = [ordered]@{
        schema_version = 1
        id = $ModuleId
        display_name = $DisplayName
        transport = "native_library"
        library_paths = [ordered]@{
            $platformKey = "bin/$platformKey/$LibraryName"
        }
    }
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $moduleDir "module.json") -Encoding UTF8
}

function Get-PortableNativeModuleLibraryPath {
    param(
        [string]$PortableDir,
        [string]$TargetOs,
        [string]$TargetArch,
        [string]$ModuleId,
        [string]$LibraryName
    )

    $platformKey = if ($TargetOs -and $TargetArch) { "$TargetOs-$TargetArch" } else { $TargetOs }
    return Join-Path (Join-Path (Join-Path (Join-Path (Join-Path $PortableDir "libs") $ModuleId) "bin") $platformKey) $LibraryName
}

Push-Location $PSScriptRoot\..
try {
    $previousCargoTargetDir = $env:CARGO_TARGET_DIR
    if ($TargetDir) {
        $env:CARGO_TARGET_DIR = Join-Path (Get-Location) $TargetDir
    }

    if ($TargetDir) {
        $targetDir = Join-Path (Join-Path (Get-Location) $TargetDir) $Configuration
    } else {
        $targetDir = Join-Path (Join-Path (Get-Location) "target") $Configuration
    }
    $runtimeTargetOs = Resolve-IntegrationTargetOs -TargetOs ""
    $runtimeTargetArch = Resolve-IntegrationTargetArch -TargetArch ""
    $resolvedIntegrationTargetOs = Resolve-IntegrationTargetOs -TargetOs $IntegrationTargetOs
    $resolvedIntegrationTargetArch = Resolve-IntegrationTargetArch -TargetArch $IntegrationTargetArch
    $includedPackageModules = Resolve-PackageModuleFilter -ExplicitModules $PackageModules
    $appArtifactName = Get-AppArtifactName -TargetOs $runtimeTargetOs
    $watcherLibraryName = Get-NativeWatcherLibraryName -TargetOs $runtimeTargetOs
    $toolLibraryName = Get-NativeToolLibraryName -TargetOs $runtimeTargetOs

    Assert-ReleaseArtifactsAreFree `
        -UiPath (Join-Path $targetDir $appArtifactName)

    $portableDir = Join-Path (Get-Location) $OutputDir
    New-Item -ItemType Directory -Force -Path $portableDir | Out-Null
    $portableConfigDir = Join-Path $portableDir "config"
    $versionManifestPath = Join-Path $portableConfigDir "version-manifest.json"
    $revisionCounterPath = Join-Path $portableConfigDir "package-revision.txt"
    $previousVersionManifest = Resolve-PreviousVersionManifest -ManifestPaths @(
        $versionManifestPath,
        (Join-Path (Get-Location) "dist\NetStitch-portable-windows-x86_64\config\version-manifest.json"),
        (Join-Path (Get-Location) "dist\NetStitch-portable\config\version-manifest.json")
    )
    $packageVersion = Get-PackageVersion -CargoTomlPath (Join-Path (Get-Location) "Cargo.toml")
    $packageRevision = Resolve-PackageRevision `
        -PreviousManifest $previousVersionManifest `
        -RevisionPath $revisionCounterPath
    $env:NETSTITCH__BUILD_FULL_VERSION = "$packageVersion.$packageRevision"

    if (-not $SkipBuild) {
        cargo build --workspace --release -j 20
        Build-PortableIntegrationWorkspaces `
            -SourceRoot (Get-Location) `
            -Configuration $Configuration `
            -IncludedModules $includedPackageModules
    }

    Assert-ReleaseArtifactsAreFree `
        -UiPath (Join-Path $portableDir $appArtifactName)
    Clear-PortablePublishRoot -PortableDir $portableDir

    $requiredBinaries = @(
        @{ Source = Join-Path $targetDir $appArtifactName; Name = $appArtifactName }
    )

    foreach ($binary in $requiredBinaries) {
        if (-not (Test-Path $binary.Source)) {
            throw "Required binary is missing: $($binary.Source)"
        }
        Copy-Item -LiteralPath $binary.Source -Destination (Join-Path $portableDir $binary.Name) -Force
    }

    Sync-PortableNativeModule `
        -PortableDir $portableDir `
        -TargetDir $targetDir `
        -TargetOs $runtimeTargetOs `
        -TargetArch $runtimeTargetArch `
        -ModuleId "netstitch-watcher" `
        -DisplayName "NetStitch Watcher" `
        -LibraryName $watcherLibraryName

    Sync-PortableNativeModule `
        -PortableDir $portableDir `
        -TargetDir $targetDir `
        -TargetOs $runtimeTargetOs `
        -TargetArch $runtimeTargetArch `
        -ModuleId "netstitch-tool" `
        -DisplayName "NetStitch Tool" `
        -LibraryName $toolLibraryName

    $sourceConfigDir = Join-Path (Get-Location) "config"
    New-Item -ItemType Directory -Force -Path $portableConfigDir | Out-Null
    $EndpointProbeConfig = Join-Path $sourceConfigDir "endpoint_probe_targets.txt"
    if (Test-Path $EndpointProbeConfig) {
        Copy-Item -LiteralPath $EndpointProbeConfig -Destination $portableConfigDir -Force
    }

    $sourceLanguageDir = Join-Path (Join-Path (Get-Location) "resources") "language"
    $portableLanguageDir = Join-Path $portableDir "language"
    if (Test-Path $sourceLanguageDir) {
        New-Item -ItemType Directory -Force -Path $portableLanguageDir | Out-Null
        Copy-Item -Path (Join-Path $sourceLanguageDir "*.ini") -Destination $portableLanguageDir -Force
    }

    $sourceAppsDir = Join-Path (Join-Path (Join-Path (Get-Location) "resources") "connectors") "apps"
    $portableAppsDir = Join-Path $portableDir "apps"
    if (Test-Path $sourceAppsDir) {
        New-Item -ItemType Directory -Force -Path $portableAppsDir | Out-Null
        Get-ChildItem -Path $portableAppsDir -File -ErrorAction SilentlyContinue |
            Where-Object { $_.Extension -eq ".toml" } |
            Remove-Item -Force
        Get-ChildItem -Path $sourceAppsDir -Filter "*.app" -File -ErrorAction SilentlyContinue | ForEach-Object {
            Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $portableAppsDir $_.Name) -Force
        }
        Sync-PortableConnectorIcons `
            -SourceIconDir (Join-Path (Join-Path (Join-Path (Get-Location) "resources") "connectors") "icons") `
        -IconDir (Join-Path $portableAppsDir "icons")

    $shippedModuleDocs = @("README_RU.txt", "README_EN.txt")
        foreach ($docName in $shippedModuleDocs) {
            $docPath = Join-Path $sourceAppsDir $docName
            if (Test-Path $docPath) {
                Copy-Item -LiteralPath $docPath -Destination (Join-Path $portableAppsDir $docName) -Force
            }
        }
    }

    Sync-PortableIntegrations `
        -SourceRoot (Get-Location) `
        -PortableDir $portableDir `
        -TargetDir $targetDir `
        -Configuration $Configuration `
        -TargetOs $resolvedIntegrationTargetOs `
        -TargetArch $resolvedIntegrationTargetArch `
        -IncludedModules $includedPackageModules

    Sync-PortableResources -PortableDir $portableDir

    $portableStorageDir = Join-Path $portableDir "storage"
    New-Item -ItemType Directory -Force -Path $portableStorageDir | Out-Null

    $portableStorageDirs = @(
        $portableStorageDir,
        (Join-Path $portableStorageDir "exports")
    )
    foreach ($storageDir in $portableStorageDirs) {
        New-Item -ItemType Directory -Force -Path $storageDir | Out-Null
    }
    Set-SystemEventCleanupMarker -StorageDir $portableStorageDir
    Remove-EmptyPortableStorageObsoleteDirs -StorageDir $portableStorageDir
    $obsoleteStorageIconDir = Join-Path $portableStorageDir "icons"
    if (Test-Path $obsoleteStorageIconDir) {
        Assert-PathInside -Child $obsoleteStorageIconDir -Parent $portableStorageDir
        Remove-Item -LiteralPath $obsoleteStorageIconDir -Recurse -Force
    }

    if ($runtimeTargetOs -eq "windows") {
        $resolvedWinDivertDir = Resolve-WinDivertRuntimeDir `
            -ExplicitWinDivertDir $WinDivertDir

        if ($resolvedWinDivertDir) {
            Write-Host "Using WinDivert runtime from $resolvedWinDivertDir"
        }

        $windivertCandidates = @()
        if ($env:NETSTITCH__WINDIVERT_DLL) {
            $windivertCandidates += $env:NETSTITCH__WINDIVERT_DLL
        }
        if ($resolvedWinDivertDir) {
            $windivertCandidates += Join-Path $resolvedWinDivertDir "WinDivert.dll"
            $windivertCandidates += Join-Path $resolvedWinDivertDir "WinDivert64.sys"
        }

        foreach ($candidate in $windivertCandidates) {
            if ($candidate -and (Test-Path $candidate)) {
                Copy-RuntimeFileIfChanged -SourcePath $candidate -DestinationDir $portableDir
            }
        }

        $dll = Join-Path $portableDir "WinDivert.dll"
        $sys = Join-Path $portableDir "WinDivert64.sys"
        if (-not (Test-Path $dll) -or -not (Test-Path $sys)) {
            Write-Warning "Portable folder was created without a complete WinDivert runtime. Set NETSTITCH__WINDIVERT_DIR or NETSTITCH__WINDIVERT_DLL before packaging UDP/QUIC builds."
        }
    }

    $watcherPortableLibraryPath = Get-PortableNativeModuleLibraryPath `
        -PortableDir $portableDir `
        -TargetOs $runtimeTargetOs `
        -TargetArch $runtimeTargetArch `
        -ModuleId "netstitch-watcher" `
        -LibraryName $watcherLibraryName
    $toolPortableLibraryPath = Get-PortableNativeModuleLibraryPath `
        -PortableDir $portableDir `
        -TargetOs $runtimeTargetOs `
        -TargetArch $runtimeTargetArch `
        -ModuleId "netstitch-tool" `
        -LibraryName $toolLibraryName

    $versionModules = [ordered]@{
        app = New-VersionModuleEntry `
            -Revision $packageRevision `
            -ArtifactName $appArtifactName `
            -ArtifactPath (Join-Path $portableDir $appArtifactName) `
            -PackageVersion $packageVersion
        watcher = New-VersionModuleEntry `
            -Revision $packageRevision `
            -ArtifactName $watcherLibraryName `
            -ArtifactPath $watcherPortableLibraryPath `
            -PackageVersion $packageVersion
        tool = New-VersionModuleEntry `
            -Revision $packageRevision `
            -ArtifactName $toolLibraryName `
            -ArtifactPath $toolPortableLibraryPath `
            -PackageVersion $packageVersion
    }
    Write-PackageRevision `
        -RevisionPath $revisionCounterPath `
        -Revision $packageRevision
    Write-VersionManifest `
        -ManifestPath $versionManifestPath `
        -PackageVersion $packageVersion `
        -Modules $versionModules

    Get-ChildItem -LiteralPath $portableDir | Select-Object Name, Length, LastWriteTime
}
finally {
    Remove-Item Env:NETSTITCH__BUILD_FULL_VERSION -ErrorAction SilentlyContinue
    if ($null -eq $previousCargoTargetDir) {
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    } else {
        $env:CARGO_TARGET_DIR = $previousCargoTargetDir
    }
    Pop-Location
}
