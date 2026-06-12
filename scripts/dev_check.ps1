# Runs the standard local verification path for the Rust workspace.
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$maxCargoJobs = 20
$logicalProcessors = [Environment]::ProcessorCount
$cargoJobs = [Math]::Min($maxCargoJobs, $logicalProcessors)
$startProcessorIndex = [Math]::Max(0, $logicalProcessors - $cargoJobs)
$currentProcess = [System.Diagnostics.Process]::GetCurrentProcess()
$originalAffinity = $currentProcess.ProcessorAffinity

function Set-ProcessAffinityWindow {
    param(
        [int]$StartIndex,
        [int]$Count
    )

    if ($Count -le 0) {
        return
    }

    $mask = [UInt64]0
    for ($index = $StartIndex; $index -lt ($StartIndex + $Count); $index++) {
        $mask = $mask -bor (([UInt64]1) -shl $index)
    }

    $currentProcess.ProcessorAffinity = [IntPtr]::new([Int64]$mask)
}

Push-Location $PSScriptRoot\..
try {
    Set-ProcessAffinityWindow -StartIndex $startProcessorIndex -Count $cargoJobs
    & (Join-Path $PSScriptRoot "check_secrets.ps1")
    cargo fmt --all --check

    $env:CARGO_TARGET_DIR = Join-Path (Get-Location) "target-devcheck-backend"
    cargo check --workspace -j $cargoJobs
    & (Join-Path (Get-Location) "tests\run.ps1") -Suite dev -Jobs $cargoJobs -TargetDir $env:CARGO_TARGET_DIR

    $env:CARGO_TARGET_DIR = Join-Path (Get-Location) "target-devcheck-ui"
    cargo check -p netstitch-ui -j $cargoJobs
}
finally {
    $currentProcess.ProcessorAffinity = $originalAffinity
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    Pop-Location
}
