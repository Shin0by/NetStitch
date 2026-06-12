param(
    [string]$PortableDir = "",
    [string]$WebUrl = "http://127.0.0.1:46473/",
    [string]$OutputDir = "",
    [int]$Width = 1350,
    [int]$Height = 867,
    [switch]$KeepRunning
)

# Captures desktop and browser-shell views of the NetStitch main window,
# crops key regions, and writes diff artifacts plus a JSON summary.
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
if ([string]::IsNullOrWhiteSpace($PortableDir)) {
    $PortableDir = Join-Path $repoRoot "dist\\NetStitch-win64-portable"
}
if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $repoRoot "temp\\visual-debug\\main-window"
}

$portablePath = Resolve-Path $PortableDir
$outputRoot = [System.IO.Path]::GetFullPath($OutputDir)
$desktopExe = Join-Path $portablePath "NetStitch.exe"

if (-not (Test-Path $desktopExe)) {
    throw "Desktop executable not found: $desktopExe"
}

function Get-EdgePath {
    $candidate = Get-Command msedge -ErrorAction SilentlyContinue
    if ($candidate) {
        return $candidate.Source
    }
    $fallback = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
    if (Test-Path $fallback) {
        return $fallback
    }
    throw "Microsoft Edge was not found. Install Edge or put msedge.exe on PATH."
}

function Ensure-Directory {
    param([string]$Path)
    if (-not (Test-Path $Path)) {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
    }
}

function Wait-HttpReady {
    param(
        [string]$Url,
        [int]$TimeoutSeconds = 20
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        try {
            $response = Invoke-WebRequest -Uri $Url -UseBasicParsing -TimeoutSec 3
            if ($response.StatusCode -ge 200 -and $response.StatusCode -lt 400) {
                return $true
            }
        } catch {
            Start-Sleep -Milliseconds 300
        }
    }
    return $false
}

function Invoke-JsonPost {
    param(
        [string]$Url,
        [hashtable]$Body
    )

    $json = $Body | ConvertTo-Json -Compress
    return Invoke-RestMethod -Method Post -Uri $Url -ContentType "application/json" -Body $json -TimeoutSec 5
}

function Resolve-WebShellUrl {
    param(
        [string]$PreferredUrl,
        [int]$WatcherPort = 46473
    )

    if (Wait-HttpReady -Url $PreferredUrl -TimeoutSeconds 2) {
        return $PreferredUrl
    }

    $apiBase = "http://127.0.0.1:$WatcherPort"
    try {
        $webUrlPayload = Invoke-RestMethod -Method Get -Uri "$apiBase/v1/web-url" -TimeoutSec 5
        $candidate = $webUrlPayload.url
        if (-not [string]::IsNullOrWhiteSpace($candidate)) {
            if (Wait-HttpReady -Url $candidate -TimeoutSeconds 3) {
                return $candidate
            }
            Invoke-JsonPost -Url "$apiBase/v1/settings" -Body @{
                key = "web.localhost_enabled"
                value = "true"
            } | Out-Null
            Start-Sleep -Milliseconds 500
            $webUrlPayload = Invoke-RestMethod -Method Get -Uri "$apiBase/v1/web-url" -TimeoutSec 5
            $candidate = $webUrlPayload.url
            if (-not [string]::IsNullOrWhiteSpace($candidate) -and (Wait-HttpReady -Url $candidate -TimeoutSeconds 6)) {
                return $candidate
            }
        }
    } catch {
    }

    throw "Browser shell is not reachable at $PreferredUrl and watcher API could not provide or enable a live web URL."
}

if (-not ("NetStitch.Visual.Native" -as [type])) {
    Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;

namespace NetStitch.Visual {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    public static class Native {
        public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

        [DllImport("user32.dll")]
        public static extern bool EnumWindows(EnumWindowsProc callback, IntPtr lParam);

        [DllImport("user32.dll", CharSet = CharSet.Unicode)]
        public static extern int GetWindowText(IntPtr hWnd, StringBuilder text, int maxCount);

        [DllImport("user32.dll")]
        public static extern bool IsWindowVisible(IntPtr hWnd);

        [DllImport("user32.dll")]
        public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);

        [DllImport("user32.dll")]
        public static extern bool SetForegroundWindow(IntPtr hWnd);

        [DllImport("user32.dll")]
        public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    }
}
"@
}

Add-Type -AssemblyName System.Drawing

function Find-NetstitchWindow {
    $matches = [System.Collections.Generic.List[IntPtr]]::new()
    $callback = [NetStitch.Visual.Native+EnumWindowsProc]{
        param([IntPtr]$hWnd, [IntPtr]$lParam)

        if (-not [NetStitch.Visual.Native]::IsWindowVisible($hWnd)) {
            return $true
        }

        $buffer = New-Object System.Text.StringBuilder 512
        [void][NetStitch.Visual.Native]::GetWindowText($hWnd, $buffer, $buffer.Capacity)
        $title = $buffer.ToString()
        if ($title -like "NetStitch ver.*") {
            $matches.Add($hWnd)
        }
        return $true
    }
    [void][NetStitch.Visual.Native]::EnumWindows($callback, [IntPtr]::Zero)
    return $matches | Select-Object -First 1
}

function Wait-NetstitchWindow {
    param([int]$TimeoutSeconds = 20)

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        $window = Find-NetstitchWindow
        if ($window) {
            return $window
        }
        Start-Sleep -Milliseconds 250
    }
    throw "NetStitch window was not found within $TimeoutSeconds seconds."
}

function Capture-WindowImage {
    param(
        [IntPtr]$Handle,
        [string]$Path
    )

    [void][NetStitch.Visual.Native]::ShowWindow($Handle, 9)
    [void][NetStitch.Visual.Native]::SetForegroundWindow($Handle)
    Start-Sleep -Milliseconds 350

    $rect = New-Object NetStitch.Visual.RECT
    if (-not [NetStitch.Visual.Native]::GetWindowRect($Handle, [ref]$rect)) {
        throw "Failed to get NetStitch window bounds."
    }

    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top
    $bitmap = New-Object System.Drawing.Bitmap $width, $height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size)
    $bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose()
    $bitmap.Dispose()

    return [pscustomobject]@{
        Left = $rect.Left
        Top = $rect.Top
        Width = $width
        Height = $height
        Path = $Path
    }
}

function Crop-ImageRegion {
    param(
        [string]$SourcePath,
        [hashtable]$Region,
        [string]$OutputPath
    )

    $source = [System.Drawing.Bitmap]::FromFile($SourcePath)
    try {
        $width = [Math]::Min($Region.Width, $source.Width - $Region.X)
        $height = [Math]::Min($Region.Height, $source.Height - $Region.Y)
        if ($width -le 0 -or $height -le 0) {
            throw "Region $($Region.Name) falls outside $SourcePath"
        }
        $rect = New-Object System.Drawing.Rectangle($Region.X, $Region.Y, $width, $height)
        $target = $source.Clone($rect, $source.PixelFormat)
        try {
            $target.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
        } finally {
            $target.Dispose()
        }
    } finally {
        $source.Dispose()
    }
}

function Crop-DesktopClientArea {
    param(
        [string]$SourcePath,
        [int]$ClientWidth,
        [int]$ClientHeight,
        [string]$OutputPath
    )

    $source = [System.Drawing.Bitmap]::FromFile($SourcePath)
    try {
        if ($source.Width -lt $ClientWidth -or $source.Height -lt $ClientHeight) {
            throw "Desktop screenshot is smaller than expected client area ${ClientWidth}x${ClientHeight}"
        }

        $horizontalInset = [Math]::Max([int][Math]::Floor(($source.Width - $ClientWidth) / 2), 0)
        $verticalInset = [Math]::Max($source.Height - $ClientHeight - $horizontalInset, 0)
        $rect = New-Object System.Drawing.Rectangle($horizontalInset, $verticalInset, $ClientWidth, $ClientHeight)
        $target = $source.Clone($rect, $source.PixelFormat)
        try {
            $target.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
        } finally {
            $target.Dispose()
        }
        return [pscustomobject]@{
            X = $horizontalInset
            Y = $verticalInset
            Width = $ClientWidth
            Height = $ClientHeight
            Path = $OutputPath
        }
    } finally {
        $source.Dispose()
    }
}

function Get-DebugRectFromDom {
    param(
        [string]$Dom,
        [string]$Id
    )

    $escapedId = [Regex]::Escape($Id)
    $patterns = @(
        "<[^>]*id=""$escapedId""[^>]*data-debug-rect=""(?<rect>\d+,\d+,\d+,\d+)""[^>]*>",
        "<[^>]*data-debug-rect=""(?<rect>\d+,\d+,\d+,\d+)""[^>]*id=""$escapedId""[^>]*>"
    )

    foreach ($pattern in $patterns) {
        $match = [Regex]::Match($Dom, $pattern, [System.Text.RegularExpressions.RegexOptions]::Singleline)
        if ($match.Success) {
            $parts = $match.Groups["rect"].Value.Split(",") | ForEach-Object { [int]$_ }
            return [pscustomobject]@{
                X = $parts[0]
                Y = $parts[1]
                Width = $parts[2]
                Height = $parts[3]
            }
        }
    }

    throw "Could not find data-debug-rect for DOM id '$Id'."
}

function Get-BrowserVisualRegions {
    param(
        [string]$Dom
    )

    $shellRect = Get-DebugRectFromDom -Dom $Dom -Id "app-shell"
    $headerRect = Get-DebugRectFromDom -Dom $Dom -Id "browser-header"
    $workspaceRect = Get-DebugRectFromDom -Dom $Dom -Id "browser-workspace"
    $observationsRect = Get-DebugRectFromDom -Dom $Dom -Id "browser-observations-panel"
    $footerRect = Get-DebugRectFromDom -Dom $Dom -Id "browser-footer"

    return [pscustomobject]@{
        Shell = $shellRect
        Regions = @(
            @{
                Name = "header"
                X = $headerRect.X
                Y = $headerRect.Y
                Width = $headerRect.Width
                Height = $headerRect.Height
            }
            @{
                Name = "top-panels"
                X = $workspaceRect.X
                Y = $workspaceRect.Y
                Width = $shellRect.Width
                Height = [Math]::Max($observationsRect.Y - $workspaceRect.Y, 0)
            }
            @{
                Name = "monitoring"
                X = $observationsRect.X
                Y = $observationsRect.Y
                Width = $observationsRect.Width
                Height = $observationsRect.Height
            }
            @{
                Name = "footer"
                X = $footerRect.X
                Y = $footerRect.Y
                Width = $footerRect.Width
                Height = $footerRect.Height
            }
        )
    }
}

function Compare-Images {
    param(
        [string]$LeftPath,
        [string]$RightPath,
        [string]$DiffPath
    )

    $left = [System.Drawing.Bitmap]::FromFile($LeftPath)
    $right = [System.Drawing.Bitmap]::FromFile($RightPath)
    try {
        $width = [Math]::Min($left.Width, $right.Width)
        $height = [Math]::Min($left.Height, $right.Height)
        $diff = New-Object System.Drawing.Bitmap $width, $height
        try {
            $differentPixels = 0
            for ($y = 0; $y -lt $height; $y++) {
                for ($x = 0; $x -lt $width; $x++) {
                    $leftPixel = $left.GetPixel($x, $y)
                    $rightPixel = $right.GetPixel($x, $y)
                    if ($leftPixel.ToArgb() -ne $rightPixel.ToArgb()) {
                        $differentPixels++
                        $diff.SetPixel($x, $y, [System.Drawing.Color]::FromArgb(255, 255, 64, 64))
                    } else {
                        $diff.SetPixel($x, $y, [System.Drawing.Color]::FromArgb(255, 24, 24, 24))
                    }
                }
            }
            $diff.Save($DiffPath, [System.Drawing.Imaging.ImageFormat]::Png)
            return [pscustomobject]@{
                Width = $width
                Height = $height
                DifferentPixels = $differentPixels
                TotalPixels = ($width * $height)
                DifferenceRatio = if ($width * $height -eq 0) { 0 } else { [Math]::Round($differentPixels / ($width * $height), 6) }
            }
        } finally {
            $diff.Dispose()
        }
    } finally {
        $left.Dispose()
        $right.Dispose()
    }
}

function Dump-BrowserArtifacts {
    param(
        [string]$EdgePath,
        [string]$Url,
        [string]$ScreenshotPath,
        [string]$DomPath
    )

    & $EdgePath `
        --headless=new `
        --disable-gpu `
        --hide-scrollbars `
        --force-device-scale-factor=1 `
        --high-dpi-support=1 `
        --window-size="$Width,$Height" `
        --virtual-time-budget=3000 `
        "--screenshot=$ScreenshotPath" `
        $Url | Out-Null

    $dom = & $EdgePath `
        --headless=new `
        --disable-gpu `
        --force-device-scale-factor=1 `
        --high-dpi-support=1 `
        --virtual-time-budget=3000 `
        --dump-dom `
        $Url
    Set-Content -Path $DomPath -Value $dom -Encoding UTF8
}

Ensure-Directory $outputRoot
$runStamp = Get-Date -Format "yyyyMMdd-HHmmss"
$artifactDir = Join-Path $outputRoot $runStamp
Ensure-Directory $artifactDir

$edgePath = Get-EdgePath
$desktopStartedByScript = $false
$desktopProcess = $null

$desktopWindow = Find-NetstitchWindow
if (-not $desktopWindow) {
    $desktopProcess = Start-Process -FilePath $desktopExe -PassThru
    $desktopStartedByScript = $true
    $desktopWindow = Wait-NetstitchWindow -TimeoutSeconds 25
}

$resolvedWebUrl = Resolve-WebShellUrl -PreferredUrl $WebUrl

$desktopFullPath = Join-Path $artifactDir "desktop-main-window.png"
$desktopClientPath = Join-Path $artifactDir "desktop-main-window-client.png"
$browserFullPath = Join-Path $artifactDir "browser-main-window.png"
$browserClientPath = Join-Path $artifactDir "browser-main-window-client.png"
$browserDomPath = Join-Path $artifactDir "browser-main-window.html"

$desktopShot = Capture-WindowImage -Handle $desktopWindow -Path $desktopFullPath
$desktopClient = Crop-DesktopClientArea -SourcePath $desktopFullPath -ClientWidth $Width -ClientHeight $Height -OutputPath $desktopClientPath
Dump-BrowserArtifacts -EdgePath $edgePath -Url $resolvedWebUrl -ScreenshotPath $browserFullPath -DomPath $browserDomPath
$browserVisual = $null
if ((Test-Path $browserDomPath) -and ((Get-Item $browserDomPath).Length -gt 0)) {
    try {
        $browserDom = Get-Content -Path $browserDomPath -Raw
        $browserVisual = Get-BrowserVisualRegions -Dom $browserDom
        Crop-ImageRegion -SourcePath $browserFullPath -Region @{
            Name = "browser-client"
            X = $browserVisual.Shell.X
            Y = $browserVisual.Shell.Y
            Width = $browserVisual.Shell.Width
            Height = $browserVisual.Shell.Height
        } -OutputPath $browserClientPath
    } catch {
        $browserVisual = $null
    }
}

$regions = @(
    @{ Name = "header"; X = 0; Y = 0; Width = $Width; Height = 56 }
    @{ Name = "top-panels"; X = 0; Y = 56; Width = $Width; Height = 390 }
    @{ Name = "monitoring"; X = 0; Y = 446; Width = $Width; Height = 387 }
    @{ Name = "footer"; X = 0; Y = 833; Width = $Width; Height = 34 }
)

$browserRegionMap = @{}
if ($browserVisual) {
    foreach ($region in $browserVisual.Regions) {
        $browserRegionMap[$region.Name] = $region
    }
} else {
    Copy-Item -Path $browserFullPath -Destination $browserClientPath -Force
    foreach ($region in $regions) {
        $browserRegionMap[$region.Name] = $region
    }
}

$regionResults = @()
foreach ($region in $regions) {
    $desktopRegionPath = Join-Path $artifactDir ("desktop-{0}.png" -f $region.Name)
    $browserRegionPath = Join-Path $artifactDir ("browser-{0}.png" -f $region.Name)
    $diffRegionPath = Join-Path $artifactDir ("diff-{0}.png" -f $region.Name)

    Crop-ImageRegion -SourcePath $desktopClientPath -Region $region -OutputPath $desktopRegionPath
    Crop-ImageRegion -SourcePath $browserFullPath -Region $browserRegionMap[$region.Name] -OutputPath $browserRegionPath
    $diff = Compare-Images -LeftPath $desktopRegionPath -RightPath $browserRegionPath -DiffPath $diffRegionPath

    $regionResults += [pscustomobject]@{
        Name = $region.Name
        DesktopPath = $desktopRegionPath
        BrowserPath = $browserRegionPath
        DiffPath = $diffRegionPath
        DifferenceRatio = $diff.DifferenceRatio
        DifferentPixels = $diff.DifferentPixels
        TotalPixels = $diff.TotalPixels
    }
}

$fullDiffPath = Join-Path $artifactDir "diff-main-window.png"
$fullDiff = Compare-Images -LeftPath $desktopClientPath -RightPath $browserClientPath -DiffPath $fullDiffPath

$summary = [pscustomobject]@{
    created_at = (Get-Date).ToString("o")
    web_url = $resolvedWebUrl
    width = $Width
    height = $Height
    desktop_window = [pscustomobject]@{
        path = $desktopFullPath
        left = $desktopShot.Left
        top = $desktopShot.Top
        width = $desktopShot.Width
        height = $desktopShot.Height
        client_path = $desktopClientPath
        client_x = $desktopClient.X
        client_y = $desktopClient.Y
        client_width = $desktopClient.Width
        client_height = $desktopClient.Height
    }
    browser_window = [pscustomobject]@{
        path = $browserFullPath
        client_path = $browserClientPath
        dom_path = $browserDomPath
        shell_x = if ($browserVisual) { $browserVisual.Shell.X } else { $null }
        shell_y = if ($browserVisual) { $browserVisual.Shell.Y } else { $null }
        shell_width = if ($browserVisual) { $browserVisual.Shell.Width } else { $null }
        shell_height = if ($browserVisual) { $browserVisual.Shell.Height } else { $null }
        used_dom_debug_rects = [bool]$browserVisual
    }
    full_window_diff = [pscustomobject]@{
        path = $fullDiffPath
        difference_ratio = $fullDiff.DifferenceRatio
        different_pixels = $fullDiff.DifferentPixels
        total_pixels = $fullDiff.TotalPixels
    }
    regions = $regionResults
}

$summaryPath = Join-Path $artifactDir "summary.json"
$summary | ConvertTo-Json -Depth 6 | Set-Content -Path $summaryPath -Encoding UTF8

if ($desktopStartedByScript -and $desktopProcess -and -not $KeepRunning) {
    try {
        $null = $desktopProcess.CloseMainWindow()
        if (-not $desktopProcess.WaitForExit(5000)) {
            Stop-Process -Id $desktopProcess.Id -Force
        }
    } catch {
        Stop-Process -Id $desktopProcess.Id -Force -ErrorAction SilentlyContinue
    }
}

$summary
