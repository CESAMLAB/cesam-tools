#!/usr/bin/env bash
#
# Build « prod » complet, entièrement DEPUIS LINUX. Produit en une commande :
#
#   1. dist/orme-linux-x86_64       Linux x86_64, AVEC IHM   (cross)
#   2. dist/orme-windows-x86_64.exe Windows x86_64, AVEC IHM (cross mingw)
#   3. dist/orme-rpi-arm64          Raspberry Pi 64b, AVEC IHM (cross)
#   4. Image Docker headless multi-arch (amd64 + arm64) à déployer n'importe où
#
# Prérequis :
#   - Rust (rustup), Docker (démon actif), et `cross` :  cargo install cross
#
# Usage :
#   scripts/build-prod.sh                          # exes + image Docker locale (amd64, chargée)
#   IMAGE=ghcr.io/moi/orme:latest \
#       scripts/build-prod.sh                      # + push image Docker MULTI-ARCH vers un registre
#
# Variables :
#   IMAGE        registre/nom:tag -> build multi-arch (amd64+arm64) poussé (`--push`)
#   DOCKER_TAG   tag de l'image locale si IMAGE non défini (défaut: orme:headless)
#
set -euo pipefail

# PKG = nom du paquet Cargo (sélecteur `-p`). BIN = nom de l'exécutable produit (ORME).
PKG="mock_bin_ru_modbustcp"
BIN="orme"
IMAGE="${IMAGE:-}"
DOCKER_TAG="${DOCKER_TAG:-orme:headless}"

LINUX_TARGET="x86_64-unknown-linux-gnu"
WIN_TARGET="x86_64-pc-windows-gnu"
ARM_TARGET="aarch64-unknown-linux-gnu"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
DIST="$ROOT/dist"

# --- Prérequis -----------------------------------------------------------------
need() { command -v "$1" >/dev/null 2>&1 || { echo "✗ Outil manquant : $1"; exit 1; }; }
need cargo; need cross; need docker
docker info >/dev/null 2>&1 || { echo "✗ Démon Docker inaccessible"; exit 1; }
rustup target add "$WIN_TARGET" "$ARM_TARGET" "$LINUX_TARGET" >/dev/null 2>&1 || true

mkdir -p "$DIST" "$DIST/_docker/amd64" "$DIST/_docker/arm64"

# --- Exécutables AVEC interface graphique --------------------------------------
# IMPORTANT : tous les builds passent par `cross` (même Linux x86_64). Mélanger
# `cargo` natif (toolchain de l'hôte) et `cross` (toolchain du conteneur) dans le
# même `target/` corrompt les proc-macros (ABI incompatible) -> erreurs
# « can't find crate for … _derive ». Un seul toolchain = builds fiables.
echo "▶ 1/4  Linux x86_64 (IHM) — cross"
cross build --release --target "$LINUX_TARGET" -p "$PKG"
cp "target/$LINUX_TARGET/release/$BIN" "$DIST/$BIN-linux-x86_64"

echo "▶ 2/4  Windows x86_64 (IHM) — cross"
cross build --release --target "$WIN_TARGET" -p "$PKG"
cp "target/$WIN_TARGET/release/$BIN.exe" "$DIST/$BIN-windows-x86_64.exe"

echo "▶ 3/4  Raspberry Pi arm64 (IHM) — cross"
cross build --release --target "$ARM_TARGET" -p "$PKG"
cp "target/$ARM_TARGET/release/$BIN" "$DIST/$BIN-rpi-arm64"

# --- Binaires headless (portables, glibc baseline de cross) pour le Docker -----
echo "▶ 4/4  Docker headless multi-arch"
# NB : on (re)compile en headless pour amd64 et arm64 via cross afin que les
# binaires tournent dans une image debian-slim, quelle que soit la glibc de l'hôte.
cross build --release --target "$LINUX_TARGET" -p "$PKG" --no-default-features
cp "target/$LINUX_TARGET/release/$BIN" "$DIST/_docker/amd64/$BIN"
cross build --release --target "$ARM_TARGET" -p "$PKG" --no-default-features
cp "target/$ARM_TARGET/release/$BIN" "$DIST/_docker/arm64/$BIN"

# Builder buildx dédié (driver docker-container = multi-plateforme).
docker buildx inspect prodbuilder >/dev/null 2>&1 \
  || docker buildx create --name prodbuilder --driver docker-container >/dev/null

if [ -n "$IMAGE" ]; then
  echo "   → build multi-arch (amd64+arm64) et push vers $IMAGE"
  docker buildx build --builder prodbuilder \
    --platform linux/amd64,linux/arm64 \
    -f docker/Dockerfile.headless -t "$IMAGE" --push .
  DOCKER_RESULT="image multi-arch poussée : $IMAGE"
else
  echo "   → build image locale amd64 (chargée dans Docker)"
  docker buildx build --builder prodbuilder \
    --platform linux/amd64 \
    -f docker/Dockerfile.headless -t "$DOCKER_TAG" --load .
  DOCKER_RESULT="image locale : $DOCKER_TAG  (multi-arch : relancer avec IMAGE=registre/nom:tag)"
fi

# --- Récapitulatif -------------------------------------------------------------
echo
echo "✓ Build prod terminé."
echo "  Exécutables (dist/) :"
for f in "$BIN-linux-x86_64" "$BIN-windows-x86_64.exe" "$BIN-rpi-arm64"; do
  [ -f "$DIST/$f" ] && printf "    %-42s %s\n" "$f" "$(du -h "$DIST/$f" | cut -f1)"
done
echo "  Docker : $DOCKER_RESULT"
