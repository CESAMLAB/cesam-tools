#!/usr/bin/env bash
#
# Construit les INSTALLEURS d'un instrument à partir des exécutables release
# déjà présents dans `dist/` (produits par scripts/build-prod.sh) :
#
#   - dist/<bin>_<version>_amd64.deb   (depuis dist/<bin>-linux-x86_64)
#   - dist/<bin>_<version>_arm64.deb   (depuis dist/<bin>-rpi-arm64)
#   - dist/<bin>-setup-x86_64.exe      (NSIS, depuis dist/<bin>-windows-x86_64.exe)
#
# Les `.deb` installent le binaire dans /usr/bin, l'entrée de bureau et l'icône
# (menu d'applications Linux). L'installeur Windows pose l'exe + des raccourcis
# (menu Démarrer/bureau) + un désinstalleur.
#
# Dégradation gracieuse : chaque cible absente (artefact manquant ou outil non
# installé) est **avertie et sautée**, sans faire échouer le script.
#
# Prérequis :
#   - .deb     : dpkg-deb (présent sur Debian/Ubuntu)
#   - Windows  : makensis (paquet `nsis` : sudo apt install nsis)
#   - .ico     : python3 + Pillow (sinon raccourcis sans icône de marque)
#
# Usage :  scripts/make-installers.sh <bin> [version]
set -euo pipefail

BIN="${1:?usage: make-installers.sh <bin> [version]}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
DIST="$ROOT/dist"
WORK="$DIST/_installer/$BIN"

# Version : argument, sinon [workspace.package].version du Cargo.toml racine.
VERSION="${2:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
VERSION="${VERSION:-0.0.0}"

DESKTOP="$ROOT/packaging/$BIN.desktop"
ICON_PNG="$ROOT/pic/$BIN-icon.png"
[ -f "$DESKTOP" ] || { echo "✗ $DESKTOP introuvable"; exit 1; }

# Métadonnées lues dans le .desktop (nom affiché, description).
dval() { sed -n "s/^$1=//p" "$DESKTOP" | head -1; }
PRODNAME="$(dval Name)";        PRODNAME="${PRODNAME:-$BIN}"
GENERIC="$(dval GenericName)"
COMMENT="$(dval Comment)";      COMMENT="${COMMENT:-$PRODNAME}"

mkdir -p "$WORK"

made=()

# --- Icône .ico (pour les raccourcis/installeur Windows) -----------------------
ICO=""
if [ -f "$ICON_PNG" ] && command -v python3 >/dev/null 2>&1; then
  ICO="$WORK/$BIN.ico"
  if ! python3 - "$ICON_PNG" "$ICO" <<'PY' 2>/dev/null
import sys
from PIL import Image
src, dst = sys.argv[1], sys.argv[2]
img = Image.open(src).convert("RGBA")
img.save(dst, sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
PY
  then
    echo "  ⚠ Pillow indisponible : .ico non généré (raccourcis sans icône de marque)"
    ICO=""
  fi
fi

# --- .deb ----------------------------------------------------------------------
build_deb() {
  local arch="$1" srcbin="$2"
  if [ ! -f "$srcbin" ]; then
    echo "  ⚠ .deb $arch sauté : $srcbin absent (lancez build-prod.sh d'abord)"
    return
  fi
  if ! command -v dpkg-deb >/dev/null 2>&1; then
    echo "  ⚠ .deb $arch sauté : dpkg-deb absent"
    return
  fi
  local pkgroot="$WORK/deb-$arch"
  rm -rf "$pkgroot"
  mkdir -p "$pkgroot/DEBIAN" \
           "$pkgroot/usr/bin" \
           "$pkgroot/usr/share/applications" \
           "$pkgroot/usr/share/icons/hicolor/256x256/apps" \
           "$pkgroot/usr/share/doc/$BIN"

  install -m 755 "$srcbin" "$pkgroot/usr/bin/$BIN"
  install -m 644 "$DESKTOP" "$pkgroot/usr/share/applications/$BIN.desktop"
  [ -f "$ICON_PNG" ] && install -m 644 "$ICON_PNG" \
      "$pkgroot/usr/share/icons/hicolor/256x256/apps/$BIN.png"
  [ -f "$ROOT/LICENSE" ] && install -m 644 "$ROOT/LICENSE" \
      "$pkgroot/usr/share/doc/$BIN/copyright"

  cat > "$pkgroot/DEBIAN/control" <<EOF
Package: $BIN
Version: $VERSION
Architecture: $arch
Maintainer: CESAM-Lab <t.menard@cesam-lab.com>
Section: utils
Priority: optional
Depends: libc6
Recommends: libgl1, libxkbcommon0, libwayland-client0
Description: $PRODNAME — ${GENERIC:-$COMMENT}
 $COMMENT
 .
 Instrument simulé de la boîte à outils cesam-tools (CESAM-Lab).
EOF

  # Rafraîchit les caches d'icônes / base .desktop à l'installation et au retrait.
  cat > "$pkgroot/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
command -v update-desktop-database >/dev/null 2>&1 && \
    update-desktop-database -q /usr/share/applications || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && \
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
exit 0
EOF
  cp "$pkgroot/DEBIAN/postinst" "$pkgroot/DEBIAN/postrm"
  chmod 755 "$pkgroot/DEBIAN/postinst" "$pkgroot/DEBIAN/postrm"

  local out="$DIST/${BIN}_${VERSION}_${arch}.deb"
  dpkg-deb --root-owner-group --build "$pkgroot" "$out" >/dev/null
  made+=("$(basename "$out")")
}

build_deb amd64 "$DIST/$BIN-linux-x86_64"
build_deb arm64 "$DIST/$BIN-rpi-arm64"

# --- Installeur Windows (NSIS) -------------------------------------------------
build_nsis() {
  local srcexe="$DIST/$BIN-windows-x86_64.exe"
  if [ ! -f "$srcexe" ]; then
    echo "  ⚠ setup Windows sauté : $srcexe absent"
    return
  fi
  if ! command -v makensis >/dev/null 2>&1; then
    echo "  ⚠ setup Windows sauté : makensis absent (sudo apt install nsis)"
    return
  fi
  local out="$DIST/$BIN-setup-x86_64.exe"
  local args=(-DBIN="$BIN" -DPRODNAME="$PRODNAME" -DVERSION="$VERSION"
              -DSRCEXE="$srcexe" -DOUTFILE="$out")
  [ -n "$ICO" ] && args+=(-DICO="$ICO")
  makensis -V2 "${args[@]}" "$ROOT/packaging/windows/installer.nsi" >/dev/null
  made+=("$(basename "$out")")
}

build_nsis

# --- Récapitulatif -------------------------------------------------------------
if [ "${#made[@]}" -eq 0 ]; then
  echo "  (aucun installeur produit pour $BIN)"
else
  echo "  Installeurs $BIN :"
  for f in "${made[@]}"; do printf "    %-40s %s\n" "$f" "$(du -h "$DIST/$f" | cut -f1)"; done
fi
