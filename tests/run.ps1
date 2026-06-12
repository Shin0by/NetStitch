param(
    [ValidateSet("dev", "regression", "full", "cloud-live")]
    [string]$Suite = "dev",

    [int]$Jobs = [Math]::Min(20, [Environment]::ProcessorCount),

    [string]$TargetDir = ""
)

# Runs centralized NetStitch test batches by suite.
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$previousTargetDir = $env:CARGO_TARGET_DIR

function Invoke-CloudWorkerCheck {
    & (Join-Path $repoRoot "scripts/check_cloud_worker.ps1")
}

function Invoke-CargoTestPackage {
    param([string]$Package)

    cargo test -p $Package -j $Jobs
}

function Invoke-CloudLiveSmoke {
    & (Join-Path $repoRoot "tests/cloud_auth_smoke.ps1")
}

Push-Location $repoRoot
try {
    if (-not [string]::IsNullOrWhiteSpace($TargetDir)) {
        $env:CARGO_TARGET_DIR = $TargetDir
    }

    switch ($Suite) {
        "dev" {
            Invoke-CloudWorkerCheck
            Invoke-CargoTestPackage "netstitch-cloud"
            Invoke-CargoTestPackage "netstitch-tool"
            Invoke-CargoTestPackage "netstitch-regression"
        }
        "regression" {
            Invoke-CargoTestPackage "netstitch-regression"
        }
        "full" {
            Invoke-CloudWorkerCheck
            foreach ($package in @(
                "netstitch-connectors",
                "netstitch-integrations",
                "netstitch-cloud",
                "netstitch-core",
                "netstitch-watcher",
                "netstitch-tool",
                "netstitch-ui",
                "netstitch-regression"
            )) {
                Invoke-CargoTestPackage $package
            }
        }
        "cloud-live" {
            Invoke-CloudLiveSmoke
        }
    }
}
finally {
    if ($null -eq $previousTargetDir) {
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    } else {
        $env:CARGO_TARGET_DIR = $previousTargetDir
    }
    Pop-Location
}
