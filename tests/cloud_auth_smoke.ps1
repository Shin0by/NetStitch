# Live smoke test for the Cloudflare Worker Google OAuth/quota/tag surface.
param(
    [string]$BaseUrl = "",
    [string]$ClientIdentifier = ""
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($BaseUrl)) {
    $BaseUrl = $env:NETSTITCH__CLOUD_BASE_URL
}
if ([string]::IsNullOrWhiteSpace($BaseUrl)) {
    $BaseUrl = "https://netstitch-sync.warfactory.workers.dev"
}
$BaseUrl = $BaseUrl.Trim().TrimEnd("/")

function ConvertTo-Base64Url {
    param([byte[]]$Bytes)

    [Convert]::ToBase64String($Bytes).TrimEnd("=").Replace("+", "-").Replace("/", "_")
}

function New-ClientPublicKeyJwk {
    $ecdsa = [Security.Cryptography.ECDsa]::Create()
    try {
        if ($null -eq $ecdsa) {
            throw "Unable to create ECDSA provider"
        }
        $ecdsa.GenerateKey([Security.Cryptography.ECCurve+NamedCurves]::nistP256)
        $parameters = $ecdsa.ExportParameters($false)
        @{
            kty = "EC"
            crv = "P-256"
            x = ConvertTo-Base64Url $parameters.Q.X
            y = ConvertTo-Base64Url $parameters.Q.Y
        } | ConvertTo-Json -Compress
    }
    finally {
        if ($null -ne $ecdsa) {
            $ecdsa.Dispose()
        }
    }
}

function Invoke-CloudJson {
    param(
        [string]$Method,
        [string]$Path,
        [object]$Body = $null,
        [string]$BearerToken = ""
    )

    $headers = @{}
    if (-not [string]::IsNullOrWhiteSpace($BearerToken)) {
        $headers["Authorization"] = "Bearer $BearerToken"
    }

    $request = @{
        Uri = "$BaseUrl$Path"
        Method = $Method
        Headers = $headers
        UseBasicParsing = $true
    }
    if ($null -ne $Body) {
        $request["ContentType"] = "application/json"
        $request["Body"] = ($Body | ConvertTo-Json -Compress -Depth 12)
    }

    try {
        $response = Invoke-WebRequest @request
    }
    catch {
        $content = $_.ErrorDetails.Message
        if ([string]::IsNullOrWhiteSpace($content) -and $_.Exception.Response) {
            $stream = $_.Exception.Response.GetResponseStream()
            if ($stream) {
                $reader = [IO.StreamReader]::new($stream)
                $content = $reader.ReadToEnd()
            }
        }
        if ([string]::IsNullOrWhiteSpace($content)) {
            $content = $_.Exception.Message
        }
        throw "Cloud request failed: $Method $Path :: $content"
    }

    if ([string]::IsNullOrWhiteSpace($response.Content)) {
        return $null
    }
    $response.Content | ConvertFrom-Json
}

function Assert-Truthy {
    param(
        [object]$Value,
        [string]$Message
    )
    if (-not $Value) {
        throw $Message
    }
}

$suffix = [Guid]::NewGuid().ToString("N").Substring(0, 12)
if ([string]::IsNullOrWhiteSpace($ClientIdentifier)) {
    $ClientIdentifier = "nsclid_v1_smoke_$suffix"
}

$clientPublicKeyJwk = New-ClientPublicKeyJwk

Write-Host "[cloud-auth-smoke] BaseUrl=$BaseUrl"
Write-Host "[cloud-auth-smoke] ClientIdentifier=$ClientIdentifier"

$health = Invoke-CloudJson -Method GET -Path "/v1/health"
Assert-Truthy ($health.ok -eq $true) "Health endpoint did not return ok=true"
Write-Host "[ok] health"

$authStart = Invoke-CloudJson -Method POST -Path "/v1/auth/google/start" -Body @{
    client_identifier = $ClientIdentifier
    client_public_key_jwk = $clientPublicKeyJwk
    ui_language = "en-en"
}
Assert-Truthy $authStart.state "OAuth start response is missing state"
Assert-Truthy $authStart.poll_secret "OAuth start response is missing poll_secret"
Assert-Truthy ($authStart.browser_url -match "/v1/auth/google/authorize") "OAuth start response browser_url mismatch"
Write-Host "[ok] google oauth start"

$rateLimitClientIdentifier = "$ClientIdentifier-rate-$suffix"
for ($index = 0; $index -lt 5; $index += 1) {
    $rateStart = Invoke-CloudJson -Method POST -Path "/v1/auth/google/start" -Body @{
        client_identifier = $rateLimitClientIdentifier
        client_public_key_jwk = $clientPublicKeyJwk
        ui_language = "en-en"
    }
    Assert-Truthy $rateStart.state "OAuth start rate-limit setup response is missing state"
}
$rateLimitedStart = Invoke-WebRequest `
    -Uri "$BaseUrl/v1/auth/google/start" `
    -Method POST `
    -ContentType "application/json" `
    -Body (@{
        client_identifier = $rateLimitClientIdentifier
        client_public_key_jwk = $clientPublicKeyJwk
        ui_language = "en-en"
    } | ConvertTo-Json -Compress) `
    -SkipHttpErrorCheck `
    -UseBasicParsing
Assert-Truthy ($rateLimitedStart.StatusCode -eq 429) "OAuth start should be rate-limited after 5 starts"
$rateLimitedStartBody = $rateLimitedStart.Content | ConvertFrom-Json
Assert-Truthy ($rateLimitedStartBody.error.code -eq "auth_start_rate_limited") "OAuth start rate-limit error code mismatch"
Write-Host "[ok] google oauth start rate limit"

$authorizePage = Invoke-WebRequest -Uri $authStart.browser_url -UseBasicParsing
Assert-Truthy ($authorizePage.StatusCode -eq 200) "OAuth authorize page did not return HTTP 200"
Assert-Truthy ($authorizePage.Content -match 'class="brand-title"') "OAuth authorize page brand title mismatch"
Assert-Truthy ($authorizePage.Content -match 'class="brand-logo"') "OAuth authorize page brand logo mismatch"
Assert-Truthy ($authorizePage.Content -match "NetStitch") "OAuth authorize page content mismatch"
Assert-Truthy ($authorizePage.Content -match "Confirm sign-in with Google") "OAuth authorize page language mismatch"
Assert-Truthy ($authorizePage.Content -match ">Submit<") "OAuth authorize page captcha submit label mismatch"
Assert-Truthy ($authorizePage.Content -match "captcha_answer") "OAuth authorize page captcha input mismatch"
Assert-Truthy ($authorizePage.Content -match "autofocus") "OAuth authorize page captcha input autofocus mismatch"
Assert-Truthy ($authorizePage.Content -match "captcha-sliced") "OAuth authorize page captcha tile container mismatch"
Assert-Truthy ($authorizePage.Content -match "data-captcha-tiles") "OAuth authorize page captcha tile payload mismatch"
Assert-Truthy ($authorizePage.Content -match "data:image/svg\+xml;base64") "OAuth authorize page captcha tile image mismatch"
Assert-Truthy ($authorizePage.Content -match "captcha-piece") "OAuth authorize page captcha script mismatch"
Assert-Truthy ($authorizePage.Content -match "data-captcha-refresh") "OAuth authorize page captcha refresh action mismatch"
Assert-Truthy ($authorizePage.Content -match "data-captcha-open") "OAuth authorize page captcha zoom opener mismatch"
Assert-Truthy ($authorizePage.Content -match "data-captcha-zoom-backdrop") "OAuth authorize page captcha zoom backdrop mismatch"
Assert-Truthy ($authorizePage.Content -match "data-captcha-submit disabled") "OAuth authorize page captcha submit button mismatch"
Assert-Truthy ($authorizePage.Content -notmatch "accounts.google.com") "OAuth authorize page must not expose Google URL before captcha"
Assert-Truthy ($authorizePage.Content -match "browser_challenge_id") "OAuth authorize page browser challenge field mismatch"
Assert-Truthy ($authorizePage.Content -match "buildBrowserVerificationProof") "OAuth authorize page browser verification script mismatch"
Assert-Truthy ($authorizePage.Content -match "interaction_pointer_x_pct") "OAuth authorize page pointer signal mismatch"
Assert-Truthy ($authorizePage.Content -match "seeded-memory-walk-v1") "OAuth authorize page memory challenge mismatch"
Assert-Truthy ($authorizePage.Content -match "new Worker") "OAuth authorize page worker proof mismatch"
Assert-Truthy ($authorizePage.Content -match "grid-template-columns: 1fr") "OAuth authorize page captcha single-column layout mismatch"
Assert-Truthy ($authorizePage.Content -match "confirm-text") "OAuth authorize page confirm text placement hook mismatch"
Assert-Truthy ($authorizePage.Content -notmatch "Submit is accepted 5 seconds") "OAuth authorize page should not show captcha timing help"
Assert-Truthy ($authorizePage.Content -match "border-radius: 5px") "OAuth authorize page control radius mismatch"
Write-Host "[ok] google oauth authorize page"

try {
    Invoke-WebRequest -Uri "$BaseUrl/v1/auth/google/authorize" -UseBasicParsing | Out-Null
    throw "OAuth authorize page unexpectedly accepted a missing state"
}
catch {
    $response = $_.Exception.Response
    if ($null -eq $response -or [int]$response.StatusCode -ne 400) {
        throw
    }
}
Write-Host "[ok] google oauth authorize missing state rejected gracefully"

$redirectHandler = [Net.Http.HttpClientHandler]::new()
$redirectHandler.AllowAutoRedirect = $false
$redirectClient = [Net.Http.HttpClient]::new($redirectHandler)
try {
    $redirectContent = [Net.Http.StringContent]::new(
        "state=$([Uri]::EscapeDataString($authStart.state))",
        [Text.Encoding]::UTF8,
        "application/x-www-form-urlencoded"
    )
    $redirectResponse = $redirectClient.PostAsync("$BaseUrl/v1/auth/google/authorize", $redirectContent).GetAwaiter().GetResult()
    $authorizePost = [pscustomobject]@{
        StatusCode = [int]$redirectResponse.StatusCode
        Headers = @{
            Location = if ($redirectResponse.Headers.Location) { $redirectResponse.Headers.Location.ToString() } else { "" }
        }
        Content = $redirectResponse.Content.ReadAsStringAsync().GetAwaiter().GetResult()
    }
}
finally {
    $redirectClient.Dispose()
    $redirectHandler.Dispose()
}
Assert-Truthy ($authorizePost.StatusCode -eq 401) "OAuth authorize POST without captcha should return HTTP 401"
Assert-Truthy ($authorizePost.Content -match 'class="brand-title"') "OAuth authorize POST captcha rejection should return branded HTML error"
Assert-Truthy ($authorizePost.Content -match "NetStitch") "OAuth authorize POST captcha rejection should return HTML error"
Write-Host "[ok] google oauth authorize post requires captcha without worker exception"

$missingPoll = Invoke-WebRequest `
    -Uri "$BaseUrl/v1/auth/session/poll" `
    -Method POST `
    -ContentType "application/json" `
    -Body (@{ state = "oauth_missing_smoke"; poll_secret = "missing" } | ConvertTo-Json -Compress) `
    -SkipHttpErrorCheck `
    -UseBasicParsing
Assert-Truthy ($missingPoll.StatusCode -eq 401) "OAuth poll missing state should return HTTP 401"
$missingPollBody = $missingPoll.Content | ConvertFrom-Json
Assert-Truthy ($missingPollBody.error.code -eq "invalid_auth_poll") "OAuth poll missing state should return invalid_auth_poll"
Write-Host "[ok] oauth poll missing state returns precise error"

$poll = Invoke-CloudJson -Method POST -Path "/v1/auth/session/poll" -Body @{
    state = $authStart.state
    poll_secret = $authStart.poll_secret
}
Assert-Truthy ($poll.status -eq "pending") "OAuth poll should be pending before browser callback"
Write-Host "[ok] oauth poll pending"

$encodedClientId = [Uri]::EscapeDataString($ClientIdentifier)
$quota = Invoke-CloudJson -Method GET -Path "/v1/client/quota?client_identifier=$encodedClientId"
Assert-Truthy ($quota.upload.limit_count -eq 0) "Upload quota limit mismatch"
Assert-Truthy ($quota.download.limit_count -eq 0) "Download quota limit mismatch"
Write-Host "[ok] quota"

$apps = Invoke-CloudJson -Method GET -Path "/v1/apps?query=smoke"
Assert-Truthy ($null -ne $apps.items) "Apps response is missing items"
Write-Host "[ok] apps"

$tags = Invoke-CloudJson -Method GET -Path "/v1/tags"
Assert-Truthy ($null -ne $tags.items) "Tags response is missing items"
Assert-Truthy ($tags.own_count -eq 0) "Anonymous tags response must report own_count=0"
Assert-Truthy ($tags.limit -eq 100) "Tags response account limit mismatch"
foreach ($tag in $tags.items) {
    Assert-Truthy ($tag.user_count -ge 1) "Tags response item is missing user_count"
    Assert-Truthy ($tag.is_own -eq $false) "Anonymous tags response must not mark any tag as owned"
}
Write-Host "[ok] tags catalog ownership"

$directTagCreate = Invoke-WebRequest `
    -Uri "$BaseUrl/v1/tags" `
    -Method POST `
    -ContentType "application/json" `
    -Body (@{ tag = "smoke" } | ConvertTo-Json -Compress) `
    -SkipHttpErrorCheck `
    -UseBasicParsing
Assert-Truthy ($directTagCreate.StatusCode -eq 404) "Direct tag creation should return HTTP 404"
$directTagCreateBody = $directTagCreate.Content | ConvertFrom-Json
Assert-Truthy ($directTagCreateBody.error.code -eq "not_found") "Direct tag creation must be unavailable outside upload"

$directTagRename = Invoke-WebRequest `
    -Uri "$BaseUrl/v1/tags/rename" `
    -Method POST `
    -ContentType "application/json" `
    -Body (@{ from = "smoke"; to = "smoke-renamed" } | ConvertTo-Json -Compress) `
    -SkipHttpErrorCheck `
    -UseBasicParsing
Assert-Truthy ($directTagRename.StatusCode -eq 404) "Direct tag rename should return HTTP 404"
$directTagRenameBody = $directTagRename.Content | ConvertFrom-Json
Assert-Truthy ($directTagRenameBody.error.code -eq "not_found") "Direct tag rename must be unavailable outside upload"
Write-Host "[ok] tag creation is upload-only"

$anonymousTagDelete = Invoke-WebRequest `
    -Uri "$BaseUrl/v1/tags/delete" `
    -Method POST `
    -ContentType "application/json" `
    -Body (@{ tag = "smoke" } | ConvertTo-Json -Compress) `
    -SkipHttpErrorCheck `
    -UseBasicParsing
Assert-Truthy ($anonymousTagDelete.StatusCode -eq 401) "Anonymous tag delete should return HTTP 401"
$anonymousTagDeleteBody = $anonymousTagDelete.Content | ConvertFrom-Json
Assert-Truthy ($anonymousTagDeleteBody.error.code -eq "auth_required") "Anonymous tag delete error code mismatch"
Write-Host "[ok] tag delete requires authorization"

try {
    Invoke-CloudJson -Method GET -Path "/v1/users/me/apps" | Out-Null
    throw "users/me/apps unexpectedly succeeded without session"
}
catch {
    if ($_.Exception.Message -notmatch "auth_required") {
        throw
    }
}
Write-Host "[ok] auth-required endpoint rejected anonymous request"

Write-Host "[cloud-auth-smoke] Passed"
