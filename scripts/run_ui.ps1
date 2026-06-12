# Runs the NetStitch Dioxus Desktop UI from the workspace root.
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CargoArgs
)

$ErrorActionPreference = "Stop"

Push-Location $PSScriptRoot\..
try {
    & cargo run -p netstitch-ui -- @CargoArgs
}
finally {
    Pop-Location
}
