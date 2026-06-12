# Runs the project-local Cloudflare Wrangler install from .local without touching global PATH.
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$WranglerArgs
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$nodeDir = Join-Path $repoRoot ".local\nodejs"
$wranglerCmd = Join-Path $repoRoot ".local\wrangler\node_modules\.bin\wrangler.cmd"
$xdgConfig = Join-Path $repoRoot ".local\xdg-config"
$xdgCache = Join-Path $repoRoot ".local\xdg-cache"

if (-not (Test-Path (Join-Path $nodeDir "node.exe"))) {
    throw "Project-local Node.js was not found at $nodeDir. Install portable Node.js into .local\nodejs first."
}

if (-not (Test-Path $wranglerCmd)) {
    throw "Project-local Wrangler was not found at $wranglerCmd. Install it with project-local npm first."
}

New-Item -ItemType Directory -Force -Path $xdgConfig, $xdgCache | Out-Null

$env:Path = "$nodeDir;$env:Path"
$env:XDG_CONFIG_HOME = $xdgConfig
$env:XDG_CACHE_HOME = $xdgCache

& $wranglerCmd @WranglerArgs
