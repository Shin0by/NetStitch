# Runs a host-side HTTPS/web-key smoke against Windows and Linux portable apps.
param(
    [ValidateSet("windows", "linux", "both")]
    [string]$Target = "both",
    [string]$WslDistro = "NetStitch-Linux-Test",
    [int]$Port = 46473,
    [switch]$AttachExisting
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$BaseUrl = "https://127.0.0.1:$Port"
$DesktopClientHeader = "x-netstitch-client"
$DesktopClientValue = "desktop-ui"
$SmokeRoot = Join-Path $RepoRoot "temp\web-access-smoke"

function New-SmokeHttpClient {
    param([System.Net.CookieContainer]$CookieContainer, [switch]$DesktopClient)

    $handler = [System.Net.Http.HttpClientHandler]::new()
    $handler.ServerCertificateCustomValidationCallback = { $true }
    $handler.CookieContainer = $CookieContainer
    $client = [System.Net.Http.HttpClient]::new($handler)
    $client.Timeout = [TimeSpan]::FromSeconds(8)
    if ($DesktopClient) {
        $client.DefaultRequestHeaders.Remove($DesktopClientHeader) | Out-Null
        $client.DefaultRequestHeaders.Add($DesktopClientHeader, $DesktopClientValue)
    }
    return $client
}

function Invoke-SmokeRequest {
    param(
        [System.Net.Http.HttpClient]$Client,
        [string]$Method,
        [string]$Url,
        [string]$Body
    )

    $request = [System.Net.Http.HttpRequestMessage]::new([System.Net.Http.HttpMethod]::new($Method), $Url)
    if ($Body) {
        $request.Content = [System.Net.Http.StringContent]::new(
            $Body,
            [System.Text.Encoding]::UTF8,
            "application/json"
        )
    }
    try {
        return $Client.SendAsync($request).GetAwaiter().GetResult()
    }
    finally {
        $request.Dispose()
    }
}

function Read-SmokeBody {
    param([System.Net.Http.HttpResponseMessage]$Response)
    return $Response.Content.ReadAsStringAsync().GetAwaiter().GetResult()
}

function Assert-SmokeStatus {
    param(
        [System.Net.Http.HttpResponseMessage]$Response,
        [int[]]$Expected,
        [string]$Label
    )
    $actual = [int]$Response.StatusCode
    if ($Expected -notcontains $actual) {
        $body = Read-SmokeBody $Response
        throw "$Label returned HTTP $actual, expected $($Expected -join '/'): $body"
    }
}

function Wait-SmokeHealth {
    param([string]$Label)

    $cookies = [System.Net.CookieContainer]::new()
    $client = New-SmokeHttpClient -CookieContainer $cookies
    try {
        $deadline = [DateTimeOffset]::Now.AddSeconds(35)
        do {
            try {
                $response = Invoke-SmokeRequest -Client $client -Method "GET" -Url "$BaseUrl/health"
                if ([int]$response.StatusCode -eq 200) {
                    $response.Dispose()
                    return
                }
                $response.Dispose()
            }
            catch {
                Start-Sleep -Milliseconds 500
            }
        } while ([DateTimeOffset]::Now -lt $deadline)
    }
    finally {
        $client.Dispose()
    }

    throw "$Label did not expose $BaseUrl/health within timeout"
}

function Test-PortIsFree {
    $connection = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($connection) {
        throw "Port $Port is already in use before smoke launch. Close NetStitch or the conflicting process first."
    }
}

function Get-WebKeyFromUrl {
    param([string]$Url)
    $match = [regex]::Match($Url, "(?:#|&|\?)web_key=([^&]+)")
    if (!$match.Success) {
        throw "Web URL does not contain web_key fragment: $Url"
    }
    return [Uri]::UnescapeDataString($match.Groups[1].Value)
}

function Invoke-WebAccessProbe {
    param([string]$Label)

    Wait-SmokeHealth -Label $Label

    $browserCookies = [System.Net.CookieContainer]::new()
    $browserClient = New-SmokeHttpClient -CookieContainer $browserCookies
    $desktopClient = New-SmokeHttpClient -CookieContainer ([System.Net.CookieContainer]::new()) -DesktopClient
    try {
        $root = Invoke-SmokeRequest -Client $browserClient -Method "GET" -Url "$BaseUrl/"
        Assert-SmokeStatus -Response $root -Expected @(200) -Label "$Label root"
        $root.Dispose()

        $unauthorized = Invoke-SmokeRequest -Client $browserClient -Method "GET" -Url "$BaseUrl/v1/snapshot"
        Assert-SmokeStatus -Response $unauthorized -Expected @(401) -Label "$Label browser snapshot without key"
        $unauthorized.Dispose()

        $webUrlResponse = Invoke-SmokeRequest -Client $desktopClient -Method "GET" -Url "$BaseUrl/v1/web-url"
        Assert-SmokeStatus -Response $webUrlResponse -Expected @(200) -Label "$Label desktop web-url"
        $webUrlPayload = Read-SmokeBody $webUrlResponse | ConvertFrom-Json
        $webUrlResponse.Dispose()

        $webKey = Get-WebKeyFromUrl -Url $webUrlPayload.url
        $authBody = @{ key = $webKey } | ConvertTo-Json -Compress
        $authResponse = Invoke-SmokeRequest -Client $browserClient -Method "POST" -Url "$BaseUrl/v1/web-auth" -Body $authBody
        Assert-SmokeStatus -Response $authResponse -Expected @(200) -Label "$Label browser web-auth"
        $authPayload = Read-SmokeBody $authResponse | ConvertFrom-Json
        $authResponse.Dispose()
        if ($authPayload.ok -ne $true) {
            throw "$Label browser web-auth did not return ok=true"
        }

        $snapshot = Invoke-SmokeRequest -Client $browserClient -Method "GET" -Url "$BaseUrl/v1/snapshot"
        Assert-SmokeStatus -Response $snapshot -Expected @(200) -Label "$Label browser snapshot after auth"
        $snapshotPayload = Read-SmokeBody $snapshot | ConvertFrom-Json
        $snapshot.Dispose()
        if ($null -eq $snapshotPayload.ui) {
            throw "$Label browser snapshot payload does not contain ui"
        }

        Write-Host "$Label web access smoke passed: $($webUrlPayload.url)"
    }
    finally {
        $browserClient.Dispose()
        $desktopClient.Dispose()
    }
}

function Start-WindowsPortable {
    Test-PortIsFree
    $exePath = Join-Path $RepoRoot "dist\NetStitch-win64-portable\NetStitch.exe"
    if (!(Test-Path $exePath)) {
        throw "Windows portable executable is missing: $exePath"
    }
    $dataDir = Join-Path $SmokeRoot "windows-storage"
    New-Item -ItemType Directory -Force -Path $dataDir | Out-Null

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $exePath
    $startInfo.WorkingDirectory = Split-Path $exePath -Parent
    $startInfo.UseShellExecute = $false
    $startInfo.Environment["NETSTITCH__DATA_DIR"] = $dataDir
    $startInfo.Environment["NETSTITCH__WEB_UI"] = "1"
    $startInfo.Environment["NETSTITCH__WEB_SCHEME"] = "https"
    $startInfo.Environment.Remove("NETSTITCH__WATCHER_ADDR") | Out-Null

    try {
        $process = [System.Diagnostics.Process]::Start($startInfo)
    }
    catch {
        if ($_.Exception.Message -match "requires elevation|требует повышения") {
            throw "Windows portable requires elevation. Start NetStitch manually, enable Web server if needed, then run: tests\web_access_smoke.ps1 -Target windows -AttachExisting"
        }
        throw
    }
    return @{ Kind = "windows"; Process = $process }
}

function Stop-WindowsPortable {
    param($Handle)
    if ($Handle -and $Handle.Process -and !$Handle.Process.HasExited) {
        Stop-Process -Id $Handle.Process.Id -Force -ErrorAction SilentlyContinue
        $Handle.Process.WaitForExit(5000) | Out-Null
    }
}

function ConvertTo-WslPath {
    param([string]$WindowsPath)
    $escaped = $WindowsPath.Replace("'", "'\''")
    return (& wsl -d $WslDistro -- bash -lc "wslpath -a '$escaped'").Trim()
}

function Start-LinuxPortable {
    Test-PortIsFree
    $portableDir = Join-Path $RepoRoot "dist\NetStitch-linux64-portable"
    $exePath = Join-Path $portableDir "NetStitch"
    if (!(Test-Path $exePath)) {
        throw "Linux portable executable is missing: $exePath"
    }
    $dataDir = Join-Path $SmokeRoot "linux-storage"
    New-Item -ItemType Directory -Force -Path $dataDir | Out-Null

    $portableWsl = ConvertTo-WslPath $portableDir
    $dataWsl = ConvertTo-WslPath $dataDir
    $logPath = "/tmp/netstitch-web-access-smoke.log"
    $command = "cd '$portableWsl' && mkdir -p '$dataWsl' && (env -u NETSTITCH__WATCHER_ADDR NETSTITCH__DATA_DIR='$dataWsl' NETSTITCH__WEB_UI=1 NETSTITCH__WEB_SCHEME=https ./NetStitch > '$logPath' 2>&1 & echo `$!)"
    $linuxProcessId = (& wsl -d $WslDistro -- bash -lc $command).Trim()
    if (!$linuxProcessId -or $linuxProcessId -notmatch "^\d+$") {
        throw "Failed to start Linux portable in WSL. Output: $linuxProcessId"
    }
    return @{ Kind = "linux"; Pid = $linuxProcessId; LogPath = $logPath }
}

function Stop-LinuxPortable {
    param($Handle)
    if ($Handle -and $Handle.Pid) {
        & wsl -d $WslDistro -- bash -lc "kill '$($Handle.Pid)' >/dev/null 2>&1 || true; sleep 1; kill -9 '$($Handle.Pid)' >/dev/null 2>&1 || true" | Out-Null
    }
}

function Invoke-TargetSmoke {
    param([string]$Kind)

    if ($AttachExisting) {
        Invoke-WebAccessProbe -Label "$Kind portable existing"
        return
    }

    $existing = Get-Process -Name "NetStitch" -ErrorAction SilentlyContinue
    if ($Kind -eq "windows" -and $existing) {
        throw "NetStitch.exe is already running. Close it before Windows portable smoke."
    }

    $handle = $null
    try {
        if ($Kind -eq "windows") {
            $handle = Start-WindowsPortable
            Invoke-WebAccessProbe -Label "Windows portable"
        } else {
            $handle = Start-LinuxPortable
            Invoke-WebAccessProbe -Label "Linux portable"
        }
    }
    finally {
        if ($Kind -eq "windows") {
            Stop-WindowsPortable $handle
        } else {
            Stop-LinuxPortable $handle
        }
        Start-Sleep -Milliseconds 750
    }
}

if ($Target -in @("windows", "both")) {
    Invoke-TargetSmoke -Kind "windows"
}
if ($Target -in @("linux", "both")) {
    Invoke-TargetSmoke -Kind "linux"
}
