<#!
.SYNOPSIS
Bootstrap GitHub SSH push access for this repo with a project-specific deploy key.

.DESCRIPTION
Creates a dedicated deploy key, installs official GitHub SSH host keys into the
current user's known_hosts via the GitHub Meta API, and configures repo-local
core.sshCommand so git push uses this exact key without depending on ssh-agent.
#>
[CmdletBinding()]
param(
    [string]$KeyPath = (Join-Path $HOME ".ssh/netstitch_github_deploy"),
    [switch]$GenerateKey,
    [switch]$InstallKnownHosts,
    [switch]$ConfigureRepo,
    [switch]$PrintPublicKey,
    [switch]$TestGitHub,
    [switch]$UseHttpsPort443
)

$ErrorActionPreference = "Stop"

function Ensure-SshDirectory {
    $sshDir = Split-Path -Parent $KeyPath
    if (-not (Test-Path $sshDir)) {
        New-Item -ItemType Directory -Force -Path $sshDir | Out-Null
    }
}

function Ensure-DeployKey {
    Ensure-SshDirectory
    if (Test-Path $KeyPath) {
        Write-Host "Deploy key already exists: $KeyPath"
        return
    }

    Start-Process -FilePath ssh-keygen -ArgumentList @(
        "-q",
        "-t", "ed25519",
        "-C", "NetStitch-development-deploy",
        "-f", $KeyPath,
        "-N", '""'
    ) -Wait -NoNewWindow

    if (-not (Test-Path $KeyPath)) {
        throw "ssh-keygen did not create the private key at $KeyPath"
    }

    $publicKeyPath = "$KeyPath.pub"
    & ssh-keygen -y -f $KeyPath | Set-Content -Path $publicKeyPath -Encoding ascii
    if (-not (Test-Path $publicKeyPath) -or (Get-Item $publicKeyPath).Length -le 0) {
        throw "ssh-keygen did not create the deploy public key at $publicKeyPath"
    }

    Write-Host "Created deploy key: $KeyPath"
}

function Get-PublicKeyText {
    $publicKeyPath = "$KeyPath.pub"
    if (-not (Test-Path $publicKeyPath)) {
        throw "Public key not found: $publicKeyPath"
    }
    return (Get-Content $publicKeyPath -Raw).Trim()
}

function Install-GitHubKnownHosts {
    Ensure-SshDirectory
    $meta = Invoke-RestMethod -Uri "https://api.github.com/meta" -Headers @{ "User-Agent" = "NetStitch setup_github_push.ps1" }
    if (-not $meta.ssh_keys -or $meta.ssh_keys.Count -eq 0) {
        throw "GitHub meta API returned no ssh_keys."
    }

    $knownHostsPath = Join-Path (Split-Path -Parent $KeyPath) "known_hosts"
    $existing = @()
    if (Test-Path $knownHostsPath) {
        $existing = (Get-Content $knownHostsPath -Raw) -split "\r?\n"
    }

    $entries = New-Object System.Collections.Generic.List[string]
    foreach ($key in $meta.ssh_keys) {
        $entries.Add("github.com $key")
        $entries.Add("[ssh.github.com]:443 $key")
    }

    $merged = [System.Collections.Generic.List[string]]::new()
    foreach ($line in $existing) {
        if ($line -match '^(github\.com|\[ssh\.github\.com\]:443)\s') {
            continue
        }
        if (-not [string]::IsNullOrWhiteSpace($line) -and -not $merged.Contains($line)) {
            $merged.Add($line)
        }
    }

    foreach ($line in $entries) {
        if (-not [string]::IsNullOrWhiteSpace($line) -and -not $merged.Contains($line)) {
            $merged.Add($line)
        }
    }

    $knownHostsContent = ($merged -join [Environment]::NewLine) + [Environment]::NewLine
    Set-Content -Path $knownHostsPath -Value $knownHostsContent -Encoding ascii
    Write-Host "Updated known_hosts: $knownHostsPath"
}

function Set-RepoSshCommand {
    $normalizedKeyPath = $KeyPath.Replace('\', '/')
    $sshArgs = @(
        "ssh"
        "-i `"$normalizedKeyPath`""
        "-o IdentitiesOnly=yes"
        "-o StrictHostKeyChecking=yes"
    )
    if ($UseHttpsPort443) {
        $sshArgs += @(
            "-o HostName=ssh.github.com"
            "-p 443"
        )
    }
    $sshCommand = $sshArgs -join " "
    & git config core.sshCommand $sshCommand
    Write-Host "Configured repo-local core.sshCommand:"
    Write-Host $sshCommand
}

function Test-GitHubSsh {
    $testArgs = @(
        "-i", $KeyPath,
        "-o", "IdentitiesOnly=yes",
        "-o", "StrictHostKeyChecking=yes"
    )
    if ($UseHttpsPort443) {
        $testArgs += @("-o", "HostName=ssh.github.com", "-p", "443")
    }
    $testArgs += @("-T", "git@github.com")
    & ssh @testArgs
}

if (-not ($GenerateKey -or $InstallKnownHosts -or $ConfigureRepo -or $PrintPublicKey -or $TestGitHub)) {
    $GenerateKey = $true
    $InstallKnownHosts = $true
    $ConfigureRepo = $true
    $PrintPublicKey = $true
}

if ($GenerateKey) { Ensure-DeployKey }
if ($InstallKnownHosts) { Install-GitHubKnownHosts }
if ($ConfigureRepo) { Set-RepoSshCommand }
if ($PrintPublicKey) {
    Write-Host ""
    Write-Host "Public key for GitHub Deploy key:"
    Write-Output (Get-PublicKeyText)
}
if ($TestGitHub) {
    Test-GitHubSsh
}
