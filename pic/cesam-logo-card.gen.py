#!/usr/bin/env python3
"""Génère la version « carte » du logo CESAM-Lab pour l'affichage GitHub.

Le logo d'origine (`Logo-CESAM-Couleur-vect.png`, fond transparent, corps
anthracite) disparaît sur fond sombre. Plutôt que d'altérer les couleurs de la
marque (noir + orange), on pose le logo intact sur une **pastille blanche
arrondie** : contraste constant en thème clair ET sombre, donc un seul fichier
(pas de <picture> ni de variante sombre à maintenir). Les coins arrondis sont
transparents.

Usage :  python3 cesam-logo-card.gen.py
Produit : Logo-CESAM-Couleur-vect-card.png
"""
from PIL import Image, ImageDraw

SRC = "Logo-CESAM-Couleur-vect.png"
DST = "Logo-CESAM-Couleur-vect-card.png"

PAD = 70        # marge blanche autour du logo (px source)
RADIUS = 80     # rayon des coins arrondis
MARGIN = 6      # frange transparente autour de la carte (évite de rogner les coins)
CARD = (255, 255, 255, 255)

im = Image.open(SRC).convert("RGBA")
bb = im.getchannel("A").getbbox()           # boîte englobante du contenu réel
x0, y0, x1, y1 = bb[0] - PAD, bb[1] - PAD, bb[2] + PAD, bb[3] + PAD

W = (x1 - x0) + 2 * MARGIN
H = (y1 - y0) + 2 * MARGIN
out = Image.new("RGBA", (W, H), (0, 0, 0, 0))
# Carte blanche arrondie.
ImageDraw.Draw(out).rounded_rectangle(
    [MARGIN, MARGIN, W - MARGIN - 1, H - MARGIN - 1], radius=RADIUS, fill=CARD)
# Logo posé dessus, recalé dans la carte.
out.alpha_composite(im, (MARGIN - x0, MARGIN - y0))
out.save(DST)
print("écrit:", DST, out.size)
