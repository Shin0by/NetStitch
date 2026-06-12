# Signs NetStitch-owned Windows binaries using the local signing config.
param(
    [string]$ConfigPath = $env:NETSTITCH__SIGNING_CONFIG,
    [switch]$EnsureTrustedForCurrentUser,
    [switch]$SkipMissing
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

function Resolve-SigningConfigPath {
    param([string]$ExplicitPath)

    if ($ExplicitPath) {
        return $ExplicitPath
    }

    $localPath = Join-Path (Get-Location) "config\signing.local.toml"
    if (Test-Path $localPath) {
        return $localPath
    }

    return Join-Path (Get-Location) "config\signing.example.toml"
}

function Read-TomlScalar {
    param(
        [string[]]$Lines,
        [string]$Section,
        [string]$Key,
        [string]$Default = ""
    )

    $inSection = $false
    foreach ($line in $Lines) {
        $trimmed = $line.Trim()
        if ($trimmed -match '^\[(.+)\]$') {
            $inSection = ($Matches[1] -eq $Section)
            continue
        }

        if (-not $inSection) {
            continue
        }

        if ($trimmed -match "^$([regex]::Escape($Key))\s*=\s*`"([^`"]*)`"") {
            return $Matches[1]
        }

        if ($trimmed -match "^$([regex]::Escape($Key))\s*=\s*(true|false)") {
            return $Matches[1]
        }
    }

    return $Default
}

function Read-TomlArray {
    param(
        [string[]]$Lines,
        [string]$Section,
        [string]$Key
    )

    $inSection = $false
    $inArray = $false
    $items = @()
    foreach ($line in $Lines) {
        $trimmed = $line.Trim()
        if ($trimmed -match '^\[(.+)\]$') {
            $inSection = ($Matches[1] -eq $Section)
            $inArray = $false
            continue
        }

        if (-not $inSection) {
            continue
        }

        if (-not $inArray -and $trimmed -match "^$([regex]::Escape($Key))\s*=\s*\[") {
            $inArray = $true
            continue
        }

        if (-not $inArray) {
            continue
        }

        if ($trimmed -match '^\]') {
            break
        }

        if ($trimmed -match '^"([^"]+)"') {
            $items += $Matches[1]
        }
    }

    return $items
}

function Find-CodeSigningCertificate {
    param(
        [string]$PublisherName,
        [string]$Thumbprint,
        [string]$StoreScope
    )

    $storeLocation = [System.Security.Cryptography.X509Certificates.StoreLocation]::$StoreScope
    $store = [System.Security.Cryptography.X509Certificates.X509Store]::new(
        [System.Security.Cryptography.X509Certificates.StoreName]::My,
        $storeLocation
    )
    $store.Open([System.Security.Cryptography.X509Certificates.OpenFlags]::ReadOnly)
    try {
        $certificates = @($store.Certificates) | Where-Object { Test-CodeSigningCertificate $_ }
        if ($Thumbprint) {
            $normalized = $Thumbprint -replace '\s', ''
            return $certificates |
                Where-Object { ($_.Thumbprint -replace '\s', '') -ieq $normalized } |
                Select-Object -First 1
        }

        $subject = "CN=$PublisherName"
        return $certificates |
            Where-Object { $_.Subject -eq $subject -and $_.NotAfter -gt (Get-Date) } |
            Sort-Object NotAfter -Descending |
            Select-Object -First 1
    }
    finally {
        $store.Close()
    }
}

function Test-CodeSigningCertificate {
    param([System.Security.Cryptography.X509Certificates.X509Certificate2]$Certificate)

    foreach ($extension in $Certificate.Extensions) {
        if ($extension.Oid.Value -ne "2.5.29.37") {
            continue
        }

        $eku = [System.Security.Cryptography.X509Certificates.X509EnhancedKeyUsageExtension]$extension
        foreach ($oid in $eku.EnhancedKeyUsages) {
            if ($oid.Value -eq "1.3.6.1.5.5.7.3.3") {
                return $true
            }
        }
    }

    return $false
}

function New-TestCodeSigningCertificate {
    param(
        [string]$PublisherName,
        [string]$StoreScope
    )

    if ($StoreScope -ne "CurrentUser") {
        throw "Self-signed test certificate creation only supports CurrentUser store. Current value: $StoreScope"
    }

    $certificate = New-SelfSignedCertificate `
        -Type CodeSigningCert `
        -Subject "CN=$PublisherName" `
        -CertStoreLocation "Cert:\CurrentUser\My" `
        -FriendlyName "NetStitch Test Code Signing" `
        -KeyAlgorithm RSA `
        -KeyLength 3072 `
        -HashAlgorithm SHA256 `
        -KeyUsage DigitalSignature `
        -KeyExportPolicy Exportable `
        -NotAfter (Get-Date).AddYears(3)

    return Find-CodeSigningCertificate `
        -PublisherName $PublisherName `
        -Thumbprint $certificate.Thumbprint `
        -StoreScope $StoreScope
}

function Trust-TestCertificateForCurrentUser {
    param([System.Security.Cryptography.X509Certificates.X509Certificate2]$Certificate)

    $publicCertificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new(
        $Certificate.Export([System.Security.Cryptography.X509Certificates.X509ContentType]::Cert)
    )
    $stores = @(
        [System.Security.Cryptography.X509Certificates.StoreName]::Root,
        [System.Security.Cryptography.X509Certificates.StoreName]::TrustedPublisher
    )

    try {
        foreach ($storeName in $stores) {
            $store = [System.Security.Cryptography.X509Certificates.X509Store]::new(
                $storeName,
                [System.Security.Cryptography.X509Certificates.StoreLocation]::CurrentUser
            )
            $store.Open([System.Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
            try {
                $store.Add($publicCertificate)
            }
            finally {
                $store.Close()
            }
        }
    }
    finally {
        $publicCertificate.Dispose()
    }
}

Push-Location $PSScriptRoot\..
try {
    $resolvedConfig = Resolve-SigningConfigPath -ExplicitPath $ConfigPath
    if (-not (Test-Path $resolvedConfig)) {
        throw "Signing config is missing: $resolvedConfig"
    }

    $lines = Get-Content -LiteralPath $resolvedConfig
    $publisherName = Read-TomlScalar -Lines $lines -Section "identity" -Key "publisher_name"
    $method = Read-TomlScalar -Lines $lines -Section "windows.authenticode" -Key "method"
    $thumbprint = Read-TomlScalar -Lines $lines -Section "windows.authenticode" -Key "certificate_thumbprint"
    $storeScope = Read-TomlScalar -Lines $lines -Section "windows.authenticode" -Key "certificate_store" -Default "CurrentUser"
    $timestampUrl = Read-TomlScalar -Lines $lines -Section "windows.authenticode" -Key "timestamp_url"
    $fileDigest = Read-TomlScalar -Lines $lines -Section "windows.authenticode" -Key "file_digest" -Default "sha256"
    $artifacts = Read-TomlArray -Lines $lines -Section "artifacts" -Key "windows_artifacts"

    if (-not $publisherName -or $publisherName -like "TODO*") {
        throw "identity.publisher_name must be set in $resolvedConfig"
    }

    if ($method -ne "test_certificate") {
        throw "scripts/sign_windows.ps1 currently implements only method = 'test_certificate'. Current method: $method"
    }

    if (-not $artifacts) {
        throw "artifacts.windows_artifacts is empty in $resolvedConfig"
    }

    $certificate = Find-CodeSigningCertificate `
        -PublisherName $publisherName `
        -Thumbprint $thumbprint `
        -StoreScope $storeScope

    if (-not $certificate) {
        $certificate = New-TestCodeSigningCertificate -PublisherName $publisherName -StoreScope $storeScope
        Write-Host "Created self-signed test code-signing certificate: $($certificate.Thumbprint)"
    } else {
        Write-Host "Using existing code-signing certificate: $($certificate.Thumbprint)"
    }

    if ($EnsureTrustedForCurrentUser) {
        Trust-TestCertificateForCurrentUser -Certificate $certificate
        Write-Host "Trusted test certificate for CurrentUser Root and TrustedPublisher stores."
    }

    $signed = @()
    foreach ($artifact in $artifacts) {
        $path = Join-Path (Get-Location) $artifact
        if (-not (Test-Path $path)) {
            if ($SkipMissing) {
                Write-Warning "Skipping missing signing artifact: $artifact"
                continue
            }
            throw "Signing artifact is missing: $artifact"
        }

        $arguments = @{
            FilePath = $path
            Certificate = $certificate
            HashAlgorithm = $fileDigest
        }
        if ($timestampUrl) {
            $arguments.TimestampServer = $timestampUrl
        }

        $result = Set-AuthenticodeSignature @arguments
        $signed += [pscustomobject]@{
            Path = $artifact
            Status = $result.Status
            StatusMessage = $result.StatusMessage
            SignerThumbprint = $result.SignerCertificate.Thumbprint
        }
    }

    $signed | Format-Table -AutoSize
}
finally {
    Pop-Location
}
