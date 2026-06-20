<#
.SYNOPSIS
    Build natif Windows (MSVC) du régulateur simulé, avec interface graphique.

.DESCRIPTION
    À exécuter SUR une machine Windows. Produit un exécutable natif
    `x86_64-pc-windows-msvc` (la meilleure option pour l'IHM sous Windows) et le
    copie dans `dist\`.

    Prérequis à installer une fois :
      1. Rust (rustup)            https://rustup.rs   (hôte par défaut : x86_64-pc-windows-msvc)
      2. Visual Studio Build Tools — charge de travail « Développement Desktop en C++ »
         (fournit le linker `link.exe` + le Windows SDK).
         https://visualstudio.microsoft.com/visual-cpp-build-tools/

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\build-windows.ps1
#>

$ErrorActionPreference = "Stop"

$Pkg = "mock_bin_ru_modbustcp"
$Target = "x86_64-pc-windows-msvc"

# Racine du dépôt (le script est dans scripts\).
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root
$Dist = Join-Path $Root "dist"

# --- Vérification des prérequis -------------------------------------------------
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo introuvable. Installer Rust : https://rustup.rs"
}
# S'assure que la cible MSVC est disponible (généralement par défaut sous Windows).
rustup target add $Target | Out-Null

New-Item -ItemType Directory -Force -Path $Dist | Out-Null

Write-Host "▶ Build natif Windows ($Target, avec IHM)..."
cargo build --release --target $Target -p $Pkg

$Out = Join-Path $Dist "$Pkg-windows-x86_64.exe"
Copy-Item "target\$Target\release\$Pkg.exe" $Out -Force

Write-Host ""
Write-Host "✓ Exécutable produit :"
Get-Item $Out | Format-List Name, @{Name="Taille(Mo)";Expression={[math]::Round($_.Length/1MB,1)}}, FullName
