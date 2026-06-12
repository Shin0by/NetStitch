# Runs the full local compile action and refreshes the portable user package.
param(
    [string]$OutputDir = "dist\NetStitch-win64-portable",
    [string]$WinDivertDir = $env:NETSTITCH__WINDIVERT_DIR
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

Push-Location $PSScriptRoot\..
try {
    & (Join-Path $PSScriptRoot "dev_check.ps1")
    & (Join-Path $PSScriptRoot "package_portable.ps1") -OutputDir $OutputDir -WinDivertDir $WinDivertDir
    & (Join-Path $PSScriptRoot "package_release_assets.ps1") -PortableDir $OutputDir -SkipPackage
}
finally {
    Pop-Location
}
