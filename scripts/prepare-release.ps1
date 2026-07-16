# Assemble dist\Streamdeck\ pour distribution (exe + tools + doc).
# Usage:
#   .\scripts\prepare-release.ps1
#   .\scripts\prepare-release.ps1 -SkipBuild
#   .\scripts\prepare-release.ps1 -Version 0.2.0
#   .\scripts\prepare-release.ps1 -Version 0.2.0 -SkipBuild
#   .\scripts\prepare-release.ps1 -Version auto   # lit version dans Cargo.toml

param(
    [switch]$SkipBuild,
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Dist = Join-Path $Root "dist\Streamdeck"
$ReleaseExe = Join-Path $Root "target\release\sd-rs.exe"

function Get-CargoPackageVersion {
    param([string]$ManifestPath)
    if (-not (Test-Path $ManifestPath)) { return $null }
    foreach ($line in Get-Content -LiteralPath $ManifestPath) {
        if ($line -match '^\s*version\s*=\s*"(.+)"\s*$') {
            return $Matches[1]
        }
    }
    return $null
}

function Get-SafeVersionSuffix {
    param([string]$Raw)
    $s = $Raw.Trim()
    if ([string]::IsNullOrWhiteSpace($s)) { return $null }
    return ($s -replace '[^\w\.\-]', '-')
}

Set-Location $Root

$versionSuffix = $null
if ($Version -eq "auto") {
    $versionSuffix = Get-SafeVersionSuffix (Get-CargoPackageVersion (Join-Path $Root "Cargo.toml"))
    if (-not $versionSuffix) {
        throw "Impossible de lire version dans Cargo.toml (utilisez -Version 1.0.0)"
    }
} elseif (-not [string]::IsNullOrWhiteSpace($Version)) {
    $versionSuffix = Get-SafeVersionSuffix $Version
    if (-not $versionSuffix) {
        throw "Numero de release invalide: $Version"
    }
}

if ($versionSuffix) {
    $DistExeName = "sd-rs-$versionSuffix.exe"
} else {
    $DistExeName = "sd-rs.exe"
}
$DistExe = Join-Path $Dist $DistExeName

if (-not $SkipBuild) {
    Write-Host "Compilation release..."
    cargo build --release
}

if (-not (Test-Path $ReleaseExe)) {
    throw "Binaire introuvable: $ReleaseExe"
}

Write-Host "Creation de $Dist ..."
if (Test-Path $Dist) {
    Remove-Item -Recurse -Force $Dist
}
New-Item -ItemType Directory -Path $Dist -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $Dist "assets\sounds") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $Dist "assets\images") -Force | Out-Null

Copy-Item $ReleaseExe $DistExe -Force
Write-Host "OK: $DistExeName"

if ($versionSuffix) {
    Set-Content -LiteralPath (Join-Path $Dist "VERSION.txt") -Value $versionSuffix -Encoding UTF8
    Write-Host "OK: VERSION.txt = $versionSuffix"
}

Copy-Item (Join-Path $Root "tools") (Join-Path $Dist "tools") -Recurse -ErrorAction SilentlyContinue

$UiIcons = Join-Path $Root "ui\icons"
if (Test-Path $UiIcons) {
    Copy-Item $UiIcons (Join-Path $Dist "ui\icons") -Recurse -Force
    Write-Host "OK: ui\icons copie (repli, SVG aussi dans l'exe)"
}

$DocFiles = @("README.md", "GUIDE.md", "INSTALL.md", "PACKAGING.md", "THIRD_PARTY.md", "tools\README.md")
foreach ($f in $DocFiles) {
    $src = Join-Path $Root $f
    if (Test-Path $src) {
        $destDir = Join-Path $Dist (Split-Path $f -Parent)
        if ($destDir -and $destDir -ne $Dist -and -not (Test-Path $destDir)) {
            New-Item -ItemType Directory -Path $destDir -Force | Out-Null
        }
        Copy-Item $src (Join-Path $Dist $f) -Force
    }
}

$Ffmpeg = Join-Path $Dist "tools\ffmpeg\ffmpeg.exe"
$Ffprobe = Join-Path $Dist "tools\ffmpeg\ffprobe.exe"
if (-not (Test-Path $Ffmpeg)) {
    Write-Warning "MANQUANT: $Ffmpeg (voir tools/README.md)"
} else {
    Write-Host "OK: ffmpeg embarque"
}
if (-not (Test-Path $Ffprobe)) {
    Write-Warning "MANQUANT: $Ffprobe"
}
$YtDlp = Join-Path $Dist "tools\yt-dlp\yt-dlp.exe"
if (-not (Test-Path $YtDlp)) {
    Write-Warning "Optionnel: $YtDlp absent"
} else {
    Write-Host "OK: yt-dlp embarque"
}

Write-Host ""
Write-Host "Termine. Lancez:"
Write-Host ('  cd "' + $Dist + '"')
Write-Host ('  .\' + $DistExeName)
