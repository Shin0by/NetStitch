param(
    [string]$WslDistro = "NetStitch-Linux-Test",
    [switch]$SkipWindows,
    [switch]$SkipLinux
)

# Builds the two Module SDK showcase examples and refreshes the installable
# archives tracked under docs/module-sdk/packages.

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$RepoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
$ExamplesRoot = Join-Path $RepoRoot "docs\module-sdk\examples"
$PackagesRoot = Join-Path $RepoRoot "docs\module-sdk\packages"
$BuildRoot = Join-Path $RepoRoot "temp\module-sdk-build"
$StageRoot = Join-Path $RepoRoot "temp\module-sdk-packages"

$RustExample = Join-Path $ExamplesRoot "ui-entity-showcase-rust"
$CppExample = Join-Path $ExamplesRoot "ui-entity-showcase-cpp"

function Assert-InRepo {
    param([string]$Path)
    $fullPath = [System.IO.Path]::GetFullPath($Path)
    if (-not $fullPath.StartsWith($RepoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to touch path outside repo: $fullPath"
    }
    return $fullPath
}

function Reset-Directory {
    param([string]$Path)
    $fullPath = Assert-InRepo $Path
    if (Test-Path -LiteralPath $fullPath) {
        Remove-Item -LiteralPath $fullPath -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $fullPath | Out-Null
}

function Invoke-Logged {
    param(
        [string]$Label,
        [scriptblock]$Script
    )
    Write-Host ""
    Write-Host "== $Label =="
    & $Script
}

function Convert-ToWslPath {
    param([string]$Path)
    $fullPath = [System.IO.Path]::GetFullPath($Path)
    if ($fullPath -notmatch "^([A-Za-z]):\\(.*)$") {
        throw "Only drive-letter Windows paths can be converted to WSL paths: $fullPath"
    }
    $drive = $Matches[1].ToLowerInvariant()
    $rest = $Matches[2].Replace("\", "/")
    return "/mnt/$drive/$rest"
}

function Get-SingleFile {
    param(
        [string]$Root,
        [string]$Filter
    )
    $file = Get-ChildItem -LiteralPath $Root -Recurse -File -Filter $Filter |
        Sort-Object FullName |
        Select-Object -First 1
    if (-not $file) {
        throw "Expected build output '$Filter' under $Root"
    }
    return $file.FullName
}

function Copy-ExampleSource {
    param(
        [string]$SourceDir,
        [string]$DestinationDir
    )
    Copy-Item -LiteralPath $SourceDir -Destination $DestinationDir -Recurse -Force
    foreach ($generatedName in @("bin", "build", "target")) {
        $generatedPath = Join-Path $DestinationDir $generatedName
        if (Test-Path -LiteralPath $generatedPath) {
            Remove-Item -LiteralPath $generatedPath -Recurse -Force
        }
    }
}

function New-ModuleArchive {
    param(
        [string]$ModuleName,
        [string]$Platform,
        [string]$SourceDir,
        [string]$BinaryPath
    )

    $archivePath = Join-Path $PackagesRoot "$ModuleName-$Platform.zip"
    $archiveStageRoot = Join-Path $StageRoot "$ModuleName-$Platform"
    $moduleStage = Join-Path $archiveStageRoot $ModuleName
    $binStage = Join-Path $moduleStage "bin"

    Reset-Directory $archiveStageRoot
    Copy-ExampleSource -SourceDir $SourceDir -DestinationDir $moduleStage
    New-Item -ItemType Directory -Force -Path $binStage | Out-Null
    Copy-Item -LiteralPath $BinaryPath -Destination $binStage -Force

    if (Test-Path -LiteralPath $archivePath) {
        Remove-Item -LiteralPath $archivePath -Force
    }
    Compress-Archive -Path (Join-Path $archiveStageRoot "*") -DestinationPath $archivePath -Force
    Write-Host "Archive: $archivePath"
}

New-Item -ItemType Directory -Force -Path $PackagesRoot | Out-Null
Reset-Directory $BuildRoot
Reset-Directory $StageRoot

$built = @{}

if (-not $SkipWindows) {
    Invoke-Logged "Build Rust showcase for Windows" {
        $rustTarget = Join-Path $BuildRoot "windows\rust"
        & cargo build --manifest-path (Join-Path $RustExample "Cargo.toml") --release --target-dir $rustTarget
        $built["rust-windows-x86_64"] = Get-SingleFile -Root (Join-Path $rustTarget "release") -Filter "ui_entity_showcase_rust.dll"
    }

    Invoke-Logged "Build C++ showcase for Windows" {
        $cppTarget = Join-Path $BuildRoot "windows\cpp"
        & cmake -S $CppExample -B $cppTarget -DCMAKE_BUILD_TYPE=Release
        & cmake --build $cppTarget --config Release
        $built["cpp-windows-x86_64"] = Get-SingleFile -Root $cppTarget -Filter "ui_entity_showcase_cpp.dll"
    }
}

if (-not $SkipLinux) {
    Invoke-Logged "Resolve WSL repo path" {
        $repoWslPath = Convert-ToWslPath $RepoRoot
        if (-not $repoWslPath) {
            throw "Could not resolve repo path inside WSL distro $WslDistro"
        }
        $script:RepoWslPath = $repoWslPath
    }

    Invoke-Logged "Build Rust showcase for Linux" {
        $command = "cd '$RepoWslPath' && cargo build --manifest-path docs/module-sdk/examples/ui-entity-showcase-rust/Cargo.toml --release --target-dir temp/module-sdk-build/linux/rust"
        & wsl -d $WslDistro -- bash -lc $command
        $built["rust-linux-x86_64"] = Get-SingleFile -Root (Join-Path $BuildRoot "linux\rust\release") -Filter "libui_entity_showcase_rust.so"
    }

    Invoke-Logged "Build C++ showcase for Linux" {
        $command = "cd '$RepoWslPath' && cmake -S docs/module-sdk/examples/ui-entity-showcase-cpp -B temp/module-sdk-build/linux/cpp -DCMAKE_BUILD_TYPE=Release && cmake --build temp/module-sdk-build/linux/cpp --config Release"
        & wsl -d $WslDistro -- bash -lc $command
        $built["cpp-linux-x86_64"] = Get-SingleFile -Root (Join-Path $BuildRoot "linux\cpp") -Filter "libui_entity_showcase_cpp.so"
    }
}

if ($built.ContainsKey("rust-windows-x86_64")) {
    New-ModuleArchive -ModuleName "ui-entity-showcase-rust" -Platform "windows-x86_64" -SourceDir $RustExample -BinaryPath $built["rust-windows-x86_64"]
}
if ($built.ContainsKey("rust-linux-x86_64")) {
    New-ModuleArchive -ModuleName "ui-entity-showcase-rust" -Platform "linux-x86_64" -SourceDir $RustExample -BinaryPath $built["rust-linux-x86_64"]
}
if ($built.ContainsKey("cpp-windows-x86_64")) {
    New-ModuleArchive -ModuleName "ui-entity-showcase-cpp" -Platform "windows-x86_64" -SourceDir $CppExample -BinaryPath $built["cpp-windows-x86_64"]
}
if ($built.ContainsKey("cpp-linux-x86_64")) {
    New-ModuleArchive -ModuleName "ui-entity-showcase-cpp" -Platform "linux-x86_64" -SourceDir $CppExample -BinaryPath $built["cpp-linux-x86_64"]
}

Write-Host ""
Write-Host "Module SDK example archives refreshed in $PackagesRoot"
