# scafgen one-liner installer (Windows / PowerShell).
#
#   irm https://raw.githubusercontent.com/sunerpy/scaffold-gen/main/scripts/install.ps1 | iex
#
# Env overrides:
#   TOOL_VERSION      pin a release (e.g. 0.1.0 or v0.1.0); default: latest
#   TOOL_INSTALL_DIR  install destination; default: $env:LOCALAPPDATA\Programs\scafgen

$ErrorActionPreference = "Stop"

$Repo = "sunerpy/scaffold-gen"
$Bin = "scafgen"

function Fail($msg) {
    Write-Error "error: $msg"
    exit 1
}

# Detect arch.
$archEnv = $env:PROCESSOR_ARCHITECTURE
switch ($archEnv) {
    "AMD64" { $archPart = "x86_64" }
    "ARM64" { $archPart = "aarch64" }
    default { Fail "unsupported arch: $archEnv (supported: AMD64, ARM64)" }
}

$target = "$archPart-pc-windows-msvc"
$ext = "zip"

# Resolve version: env override or latest-release API.
if ($env:TOOL_VERSION) {
    $version = $env:TOOL_VERSION -replace '^v', ''
} else {
    Write-Host "Resolving latest release..."
    $api = "https://api.github.com/repos/$Repo/releases/latest"
    # GitHub API requires a User-Agent header.
    $headers = @{ "User-Agent" = "scafgen-installer" }
    try {
        $release = Invoke-RestMethod -Uri $api -Headers $headers
    } catch {
        Fail "could not resolve latest release from $api : $_"
    }
    $tag = $release.tag_name
    if (-not $tag) { Fail "could not resolve latest release tag from $api" }
    $version = $tag -replace '^v', ''
}

# IMPORTANT: this asset name must match what the release workflow produces.
$asset = "$Bin-$version-$target.$ext"
$url = "https://github.com/$Repo/releases/download/v$version/$asset"

if ($env:TOOL_INSTALL_DIR) {
    $installDir = $env:TOOL_INSTALL_DIR
} else {
    $installDir = Join-Path $env:LOCALAPPDATA "Programs\scafgen"
}

Write-Host "Installing $Bin v$version ($target)"
Write-Host "  from: $url"
Write-Host "  to:   $installDir\$Bin.exe"

$tmp = Join-Path $env:TEMP ("scafgen-" + [System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

try {
    $archivePath = Join-Path $tmp $asset
    # GitHub API requires a User-Agent header.
    $headers = @{ "User-Agent" = "scafgen-installer" }
    try {
        Invoke-WebRequest -Uri $url -OutFile $archivePath -Headers $headers
    } catch {
        Fail "download failed: $url : $_"
    }

    try {
        Expand-Archive -Path $archivePath -DestinationPath $tmp -Force
    } catch {
        Fail "failed to extract $asset : $_"
    }

    $exeSrc = Join-Path $tmp "$Bin.exe"
    if (-not (Test-Path $exeSrc)) { Fail "expected $Bin.exe in archive but it was not found" }

    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
    Move-Item -Path $exeSrc -Destination (Join-Path $installDir "$Bin.exe") -Force

    Write-Host "Installed $Bin to $installDir\$Bin.exe"

    # Add the install dir to the USER PATH if missing.
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $userPath) { $userPath = "" }
    $parts = $userPath -split ';' | Where-Object { $_ -ne "" }
    if ($parts -notcontains $installDir) {
        $newPath = if ($userPath -eq "") { $installDir } else { "$userPath;$installDir" }
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Host "NOTE: added $installDir to your user PATH. Restart your terminal for it to take effect."
    }
} finally {
    if (Test-Path $tmp) { Remove-Item -Recurse -Force $tmp }
}
