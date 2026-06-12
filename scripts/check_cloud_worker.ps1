# Syntax-checks the Cloudflare Worker entrypoint without deploying or using network.
param()

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$localNode = Join-Path $repoRoot ".local\nodejs\node.exe"
$workerRoot = Join-Path $repoRoot "src\netstitch-cloud-worker"

if (Test-Path $localNode) {
    $node = $localNode
} else {
    $node = "node"
}

Push-Location $workerRoot
try {
    & $node --check "src\worker.js"
}
finally {
    Pop-Location
}
