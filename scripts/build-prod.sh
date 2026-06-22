#!/usr/bin/env bash
#
# Build « prod » complet de TOUS les instruments du workspace, entièrement DEPUIS
# LINUX. Pour chaque instrument (ORME, OSNE…) il produit en une commande :
#
#   1. dist/<bin>-linux-x86_64       Linux x86_64, AVEC IHM   (cross)
#   2. dist/<bin>-windows-x86_64.exe Windows x86_64, AVEC IHM (cross mingw)
#   3. dist/<bin>-rpi-arm64          Raspberry Pi 64b, AVEC IHM (cross)
#   4. Image Docker headless multi-arch (amd64 + arm64) à déployer n'importe où
#
# Prérequis :
#   - Rust (rustup), Docker (démon actif), et `cross` :  cargo install cross
#
# Usage :
#   scripts/build-prod.sh                          # exes + images Docker locales (amd64, chargées)
#   IMAGE_PREFIX=ghcr.io/moi \
#       scripts/build-prod.sh                      # + push images MULTI-ARCH <prefix>/<bin>:latest
#
# Variables :
#   IMAGE_PREFIX  registre/compte -> images multi-arch (amd64+arm64) poussées
#                 sous <prefix>/<bin>:latest (`--push`)
#   ONLY          ne construire qu'un instrument (ex. ONLY=osne)
#
set -euo pipefail

# Instruments du workspace : "pkg:bin:port".
#   pkg  = nom du paquet Cargo (sélecteur `-p`)
#   bin  = nom de l'exécutable produit ([[bin]] du crate)
#   port = port du protocole de terrain (Modbus / NAMUR), pour Docker EXPOSE
INSTRUMENTS=(
  "mock_bin_ru_modbustcp:orme:5502"
  "mock_bin_su_namur:osne:4001"
  "mock_bin_ru_opcua:ru_opcua:4840"
)

IMAGE_PREFIX="${IMAGE_PREFIX:-}"
ONLY="${ONLY:-}"
# Construire aussi les installeurs (.deb amd64/arm64 + setup Windows NSIS) ? (1=oui)
INSTALLERS="${INSTALLERS:-1}"

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

mkdir -p "$DIST"

# Builder buildx dédié (driver docker-container = multi-plateforme), partagé.
docker buildx inspect prodbuilder >/dev/null 2>&1 \
  || docker buildx create --name prodbuilder --driver docker-container >/dev/null

# IMPORTANT : tous les builds passent par `cross` (même Linux x86_64). Mélanger
# `cargo` natif (toolchain de l'hôte) et `cross` (toolchain du conteneur) dans le
# même `target/` corrompt les proc-macros (ABI incompatible) -> erreurs
# « can't find crate for … _derive ». Un seul toolchain = builds fiables.

SUMMARY=()

build_instrument() {
  local pkg="$1" bin="$2" port="$3"

  echo "════════════════════════════════════════════════════════════════"
  echo "  Instrument : $bin  (paquet $pkg, port $port)"
  echo "════════════════════════════════════════════════════════════════"

  echo "▶ 1/4  Linux x86_64 (IHM) — cross"
  cross build --release --target "$LINUX_TARGET" -p "$pkg"
  cp "target/$LINUX_TARGET/release/$bin" "$DIST/$bin-linux-x86_64"

  echo "▶ 2/4  Windows x86_64 (IHM) — cross"
  cross build --release --target "$WIN_TARGET" -p "$pkg"
  cp "target/$WIN_TARGET/release/$bin.exe" "$DIST/$bin-windows-x86_64.exe"

  echo "▶ 3/4  Raspberry Pi arm64 (IHM) — cross"
  cross build --release --target "$ARM_TARGET" -p "$pkg"
  cp "target/$ARM_TARGET/release/$bin" "$DIST/$bin-rpi-arm64"

  # --- Binaires headless (portables, glibc baseline de cross) pour le Docker ---
  echo "▶ 4/4  Docker headless multi-arch"
  mkdir -p "$DIST/_docker/$bin/amd64" "$DIST/_docker/$bin/arm64"
  cross build --release --target "$LINUX_TARGET" -p "$pkg" --no-default-features
  cp "target/$LINUX_TARGET/release/$bin" "$DIST/_docker/$bin/amd64/$bin"
  cross build --release --target "$ARM_TARGET" -p "$pkg" --no-default-features
  cp "target/$ARM_TARGET/release/$bin" "$DIST/_docker/$bin/arm64/$bin"

  if [ -n "$IMAGE_PREFIX" ]; then
    local image="$IMAGE_PREFIX/$bin:latest"
    echo "   → build multi-arch (amd64+arm64) et push vers $image"
    docker buildx build --builder prodbuilder \
      --platform linux/amd64,linux/arm64 \
      --build-arg "BIN=$bin" --build-arg "PORT=$port" \
      -f docker/Dockerfile.headless -t "$image" --push .
    SUMMARY+=("image multi-arch poussée : $image")
  else
    local image="$bin:headless"
    echo "   → build image locale amd64 (chargée dans Docker)"
    docker buildx build --builder prodbuilder \
      --platform linux/amd64 \
      --build-arg "BIN=$bin" --build-arg "PORT=$port" \
      -f docker/Dockerfile.headless -t "$image" --load .
    SUMMARY+=("image locale : $image  (port $port)")
  fi

  # --- Installeurs (.deb Linux/RPi + setup Windows) ----------------------------
  if [ "$INSTALLERS" != "0" ]; then
    echo "▶ Installeurs $bin (.deb + Windows NSIS)"
    "$ROOT/scripts/make-installers.sh" "$bin"
  fi
}

for entry in "${INSTRUMENTS[@]}"; do
  IFS=":" read -r pkg bin port <<<"$entry"
  if [ -n "$ONLY" ] && [ "$ONLY" != "$bin" ] && [ "$ONLY" != "$pkg" ]; then
    continue
  fi
  build_instrument "$pkg" "$bin" "$port"
done

# --- Récapitulatif -------------------------------------------------------------
echo
echo "✓ Build prod terminé."
echo "  Exécutables (dist/) :"
for entry in "${INSTRUMENTS[@]}"; do
  IFS=":" read -r pkg bin port <<<"$entry"
  for f in "$bin-linux-x86_64" "$bin-windows-x86_64.exe" "$bin-rpi-arm64"; do
    [ -f "$DIST/$f" ] && printf "    %-42s %s\n" "$f" "$(du -h "$DIST/$f" | cut -f1)"
  done
done
if [ "$INSTALLERS" != "0" ]; then
  echo "  Installeurs (dist/) :"
  shopt -s nullglob
  for f in "$DIST"/*.deb "$DIST"/*-setup-x86_64.exe; do
    printf "    %-42s %s\n" "$(basename "$f")" "$(du -h "$f" | cut -f1)"
  done
  shopt -u nullglob
fi
echo "  Docker :"
for line in "${SUMMARY[@]}"; do
  echo "    $line"
done
[ -z "$IMAGE_PREFIX" ] && echo "    (multi-arch : relancer avec IMAGE_PREFIX=registre/compte)"
