#!/usr/bin/env bash
#
# Installe l'entrée de bureau ORME pour l'utilisateur courant (Linux/Wayland & X11).
#
# Pourquoi : sous Wayland, l'icône de la barre des tâches n'est PAS prise depuis
# le binaire (`with_icon` est ignoré). Le compositeur associe la fenêtre à son
# `app_id` (« orme », défini dans main.rs) au fichier `orme.desktop` du même nom,
# et affiche l'icône `Icon=orme` résolue via le thème d'icônes (hicolor).
#
# Ce script copie donc :
#   - pic/orme-icon.png            -> ~/.local/share/icons/hicolor/256x256/apps/orme.png
#   - packaging/orme.desktop       -> ~/.local/share/applications/orme.desktop
#
# L'exécutable « orme » doit être dans le PATH (ou ajustez `Exec=` du .desktop).
#
# Usage :  scripts/install-desktop.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

ICON_SRC="$ROOT/pic/orme-icon.png"
DESKTOP_SRC="$ROOT/packaging/orme.desktop"

ICON_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/256x256/apps"
APP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"

[ -f "$ICON_SRC" ]    || { echo "✗ Icône introuvable : $ICON_SRC"; exit 1; }
[ -f "$DESKTOP_SRC" ] || { echo "✗ .desktop introuvable : $DESKTOP_SRC"; exit 1; }

mkdir -p "$ICON_DIR" "$APP_DIR"
install -m 644 "$ICON_SRC" "$ICON_DIR/orme.png"
install -m 644 "$DESKTOP_SRC" "$APP_DIR/orme.desktop"

# Rafraîchit les caches (sans échec si les outils sont absents).
command -v gtk-update-icon-cache >/dev/null 2>&1 && \
    gtk-update-icon-cache -f -t "${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor" >/dev/null 2>&1 || true
command -v update-desktop-database >/dev/null 2>&1 && \
    update-desktop-database "$APP_DIR" >/dev/null 2>&1 || true

echo "✓ Entrée de bureau ORME installée."
echo "  Icône   : $ICON_DIR/orme.png"
echo "  Lanceur : $APP_DIR/orme.desktop"
echo "  (Si l'icône ne s'affiche pas tout de suite, relancez la session Wayland.)"
