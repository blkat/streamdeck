# Supprime sons, images et programmation des touches (base SQLite + fichiers).
# Usage:
#   .\scripts\reset-user-data.ps1              # projet + dist\Streamdeck
#   .\scripts\reset-user-data.ps1 -Target project
#   .\scripts\reset-user-data.ps1 -Target dist
#
# Fermez sd-rs.exe avant d'exécuter ce script.

param(
    [ValidateSet("project", "dist", "all")]
    [string]$Target = "all"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

$BaseDirs = @()
if ($Target -eq "project" -or $Target -eq "all") {
    $BaseDirs += $Root
}
if ($Target -eq "dist" -or $Target -eq "all") {
    $BaseDirs += (Join-Path $Root "dist\Streamdeck")
}

function Clear-DirContents {
    param([string]$Dir)
    if (-not (Test-Path $Dir)) {
        New-Item -ItemType Directory -Path $Dir -Force | Out-Null
        return
    }
    Get-ChildItem -LiteralPath $Dir -Force -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -ne ".gitkeep" } |
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
}

foreach ($base in $BaseDirs) {
    if (-not (Test-Path $base)) {
        Write-Host "Ignore (absent): $base"
        continue
    }
    Write-Host "Nettoyage: $base"

    $db = Join-Path $base "soundboard.db"
    if (Test-Path $db) {
        Remove-Item -LiteralPath $db -Force
        Write-Host "  - soundboard.db supprime"
    }

    Clear-DirContents (Join-Path $base "assets\sounds")
    Clear-DirContents (Join-Path $base "assets\images")
    Write-Host "  - assets\sounds et assets\images vides"

    foreach ($sub in @("logs", "temp")) {
        $p = Join-Path $base $sub
        if (Test-Path $p) {
            Clear-DirContents $p
            Write-Host "  - $sub vide"
        }
    }
}

Write-Host ""
Write-Host "Termine. Au prochain lancement, une base neuve sera creee (page Accueil vide)."
Write-Host "Pour le livrable: .\scripts\prepare-release.ps1 puis distribuez dist\Streamdeck\ sans soundboard.db."
