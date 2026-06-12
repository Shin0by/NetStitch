# Scans tracked source files for high-confidence secret patterns before commit/release.
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$binaryExtensions = @(
    ".ico", ".png", ".jpg", ".jpeg", ".gif", ".webp", ".exe", ".dll", ".sys",
    ".pdb", ".lib", ".rlib", ".rmeta", ".zip", ".7z", ".gz", ".tar"
)
$allowlistedFiles = @(
    "config/signing.example.toml",
    "scripts/check_secrets.ps1"
)
$patterns = @(
    @{ Name = "private key block"; Regex = "-----BEGIN (RSA |DSA |EC |OPENSSH |PGP )?PRIVATE KEY-----" },
    @{ Name = "GitHub classic token"; Regex = "gh[pousr]_[A-Za-z0-9_]{36,}" },
    @{ Name = "GitHub fine-grained token"; Regex = "github_pat_[A-Za-z0-9_]{22,}_[A-Za-z0-9_]{59,}" },
    @{ Name = "GitLab token"; Regex = "glpat-[A-Za-z0-9_-]{20,}" },
    @{ Name = "Slack token"; Regex = "xox[baprs]-[A-Za-z0-9-]{20,}" },
    @{ Name = "AWS access key"; Regex = "AKIA[0-9A-Z]{16}" },
    @{ Name = "OpenAI API key"; Regex = "sk-[A-Za-z0-9]{32,}" }
)

Push-Location $repoRoot
try {
    $safeDirectory = ([System.IO.Path]::GetFullPath($repoRoot)).Replace('\', '/')
    $files = git -c "safe.directory=$safeDirectory" ls-files --cached --others --exclude-standard
    $findings = @()
    foreach ($file in $files) {
        $normalized = $file -replace "\\", "/"
        if ($allowlistedFiles -contains $normalized) {
            continue
        }

        $extension = [System.IO.Path]::GetExtension($file).ToLowerInvariant()
        if ($binaryExtensions -contains $extension) {
            continue
        }

        $content = Get-Content -LiteralPath $file -Raw -ErrorAction SilentlyContinue
        if ($null -eq $content) {
            continue
        }

        foreach ($pattern in $patterns) {
            if ($content -match $pattern.Regex) {
                $findings += "$file matched $($pattern.Name)"
            }
        }
    }

    if ($findings.Count -gt 0) {
        $message = "Potential secrets found:`n" + ($findings -join "`n")
        throw $message
    }

    Write-Host "Secret scan passed for tracked and unignored untracked files."
}
finally {
    Pop-Location
}
