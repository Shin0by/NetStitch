# Verifies Cloudflare Worker configuration names without printing secret values.
param(
    [string]$ConfigPath = "src\netstitch-cloud-worker\wrangler.local.toml"
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptDir "..")
$wrangler = Join-Path $scriptDir "wrangler.ps1"
$configFullPath = Join-Path $repoRoot $ConfigPath

if (-not (Test-Path $configFullPath)) {
    throw "Cloudflare local config was not found: $configFullPath"
}

$secretOutput = & $wrangler secret list --config $configFullPath
$secrets = $secretOutput | ConvertFrom-Json
$secretNames = @($secrets | ForEach-Object { $_.name })
$requiredSecrets = @(
    "NETSTITCH_IDENTITY_PEPPER"
)

$missingSecrets = @($requiredSecrets | Where-Object { $secretNames -notcontains $_ })
if ($missingSecrets.Count -gt 0) {
    throw "Missing Cloudflare Worker secrets: $($missingSecrets -join ', ')"
}

$configText = Get-Content -LiteralPath $configFullPath -Raw
$requiredConfigTokens = @(
    "binding = `"DB`"",
    "database_name = `"netstitch`"",
    "database_id = "
)

$missingConfig = @($requiredConfigTokens | Where-Object { -not $configText.Contains($_) })
if ($missingConfig.Count -gt 0) {
    throw "Missing Cloudflare local config tokens: $($missingConfig -join ', ')"
}

$googleClientIdConfigured =
    $secretNames -contains "NETSTITCH_GOOGLE_CLIENT_ID" -or
    $configText.Contains("NETSTITCH_GOOGLE_CLIENT_ID")
if (-not $googleClientIdConfigured) {
    throw "Missing Cloudflare Worker Google OAuth client id: NETSTITCH_GOOGLE_CLIENT_ID"
}

$googleClientSecretConfigured = $secretNames -contains "NETSTITCH_GOOGLE_CLIENT_SECRET"
if (-not $googleClientSecretConfigured) {
    throw "Missing Cloudflare Worker secret: NETSTITCH_GOOGLE_CLIENT_SECRET"
}

Write-Host "Cloudflare config check passed. Required secret names and D1 binding are present."
