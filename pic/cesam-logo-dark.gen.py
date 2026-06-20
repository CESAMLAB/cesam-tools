#!/usr/bin/env python3
"""Génère la variante « thème sombre » du logo CESAM-Lab pour l'affichage GitHub.

Le logo d'origine (`Logo-CESAM-Couleur-vect.png`, fond transparent) a un corps
**anthracite/noir** (engrenage, barre, pistes) qui disparaît sur fond sombre. On
produit ici une variante où les **neutres sont inversés** (noir -> clair, blanc
-> sombre, pour garder lisibles aussi bien le corps que les chiffres binaires et
les liserés) tandis que l'**orange est conservé**. Les deux PNG sont ensuite
servis via une balise <picture> dans les README (selon prefers-color-scheme).

Usage :  python3 cesam-logo-dark.gen.py
Produit : Logo-CESAM-Couleur-vect-dark.png
"""
from PIL import Image

SRC = "Logo-CESAM-Couleur-vect.png"
DST = "Logo-CESAM-Couleur-vect-dark.png"

# Bornes de l'inversion des neutres : noir d'origine -> LIGHT, blanc -> DARK.
LIGHT = 236  # corps anthracite (L~0) -> gris clair #ECECEC
DARK = 26    # blancs (chiffres, liserés, L~224) -> anthracite #1A1A1A

im = Image.open(SRC).convert("RGBA")
out = Image.new("RGBA", im.size)
sp, dp = im.load(), out.load()
W, H = im.size
for y in range(H):
    for x in range(W):
        r, g, b, a = sp[x, y]
        if a == 0:
            dp[x, y] = (0, 0, 0, 0)
            continue
        sat = max(r, g, b) - min(r, g, b)
        # Pixel chromatique (orange) : conservé tel quel.
        if sat > 40 and r >= g >= b:
            dp[x, y] = (r, g, b, a)
            continue
        # Pixel neutre : inversion linéaire de la luminance (noir<->blanc).
        L = (r + g + b) / 3
        v = round(LIGHT + (L / 224.0) * (DARK - LIGHT))
        v = max(0, min(255, v))
        dp[x, y] = (v, v, v, a)
out.save(DST)
print("écrit:", DST)
