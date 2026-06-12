# Clears all client-origin data from the NetStitch Cloudflare D1 database while
# preserving schema/service metadata so the database remains ready for use.
param(
    [string]$DatabaseName = "netstitch",
    [string]$ConfigPath = "",
    [switch]$Local,
    [switch]$Yes
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")
$Wrangler = Join-Path $ScriptDir "wrangler.ps1"

if (-not (Test-Path -LiteralPath $Wrangler)) {
    throw "Missing project Wrangler wrapper: $Wrangler"
}

if ([string]::IsNullOrWhiteSpace($ConfigPath)) {
    $ConfigPath = Join-Path $RepoRoot "src\netstitch-cloud-worker\wrangler.local.toml"
}

$ResolvedConfig = Resolve-Path -LiteralPath $ConfigPath

if (-not $Yes) {
    Write-Host "This will delete all NetStitch client-origin cloud data from D1 '$DatabaseName'."
    Write-Host "Preserved: D1 schema/migrations and Cloudflare service metadata."
    Write-Host "Deleted: users, client identifiers, sessions, keys, authors, app catalog, observations, OAuth/captcha/rate-limit/audit/cache rows."
    $Expected = "CLEAR $DatabaseName"
    $Confirmation = Read-Host "Type '$Expected' to continue"
    if ($Confirmation -ne $Expected) {
        Write-Host "Cancelled."
        exit 2
    }
}

$Tables = @(
    "audit_events",
    "auth_attempt_windows",
    "browser_verification_challenges",
    "captcha_challenges",
    "captcha_failure_buckets",
    "captcha_refresh_pools",
    "client_quota_windows",
    "domain_verifications",
    "jwt_replay_cache",
    "oauth_start_windows",
    "oauth_states",
    "observation_submissions",
    "observation_author_rows",
    "observations",
    "author_profiles",
    "user_sessions",
    "client_keys",
    "user_identities",
    "clients",
    "users",
    "app_catalog"
)

$CleanupSql = @"
PRAGMA foreign_keys = OFF;
DELETE FROM audit_events;
DELETE FROM auth_attempt_windows;
DELETE FROM browser_verification_challenges;
DELETE FROM captcha_challenges;
DELETE FROM captcha_failure_buckets;
DELETE FROM captcha_refresh_pools;
DELETE FROM client_quota_windows;
DELETE FROM domain_verifications;
DELETE FROM jwt_replay_cache;
DELETE FROM oauth_start_windows;
DELETE FROM oauth_states;
DELETE FROM observation_submissions;
DELETE FROM observation_author_rows;
DELETE FROM observations;
DELETE FROM author_profiles;
DELETE FROM user_sessions;
DELETE FROM client_keys;
DELETE FROM user_identities;
DELETE FROM clients;
DELETE FROM users;
DELETE FROM app_catalog;
DELETE FROM sqlite_sequence;
PRAGMA foreign_keys = ON;
"@

$VerifyTerms = $Tables | ForEach-Object {
    "(SELECT COUNT(*) FROM $($_))"
}
$VerifySql = "SELECT ($($VerifyTerms -join '+')) AS client_rows_left;"

$TempSql = Join-Path ([System.IO.Path]::GetTempPath()) ("netstitch-clear-cloud-{0}.sql" -f ([Guid]::NewGuid().ToString("N")))

function Invoke-NetStitchWrangler {
    param([string[]]$Arguments)

    & $Wrangler @Arguments
    $ExitCode = if ($null -eq $LASTEXITCODE) { 0 } else { $LASTEXITCODE }
    if ($ExitCode -ne 0) {
        throw "Wrangler command failed with exit code $ExitCode`: $($Arguments -join ' ')"
    }
}

try {
    Set-Content -LiteralPath $TempSql -Value $CleanupSql -Encoding UTF8

    $ExecuteArgs = @("d1", "execute", $DatabaseName)
    if (-not $Local) {
        $ExecuteArgs += "--remote"
    }
    $ExecuteArgs += @("--config", $ResolvedConfig.Path, "--file", $TempSql)

    Invoke-NetStitchWrangler -Arguments $ExecuteArgs

    $VerifyArgs = @("d1", "execute", $DatabaseName)
    if (-not $Local) {
        $VerifyArgs += "--remote"
    }
    $VerifyArgs += @("--config", $ResolvedConfig.Path, "--command", $VerifySql)

    Invoke-NetStitchWrangler -Arguments $VerifyArgs
}
finally {
    if (Test-Path -LiteralPath $TempSql) {
        Remove-Item -LiteralPath $TempSql -Force
    }
}
