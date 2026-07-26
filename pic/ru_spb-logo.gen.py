#!/usr/bin/env python3
"""Génère le logo ORSE (régulateur Sparkplug B) — icône carrée + lockup horizontal.

Marque : **ORSE** = *Open Regulator Sparkplug Emulator* (nom technique : RU/Sparkplug B).

Style CESAM-Lab, cohérent avec ORME/OSNE/ORUE : orange #F29400, anthracite #1A171B,
blanc, gris. Motif : le **cadran de régulation** d'ORME (RU = Regulation Unit, même
identité de régulateur), surmonté de l'**éclair Sparkplug** (la « spark ») et entouré
de **nœuds-abonnés non reliés** disséminés autour du cadran — l'éclair *publie*, les
nœuds *souscrivent* : c'est le **pub/sub MQTT** de Sparkplug B. C'est ce qui distingue
cet instrument d'ORME (bus Modbus), d'OSNE (agitateur) et d'ORUE (anneau de nœuds OPC UA
reliés en chaîne).

Sorties :
  - ru_spb-icon.svg   (source vectorielle de l'icône carrée 256×256)
  - ru_spb-logo.svg   (lockup horizontal 760×240 : icône + texte)
  - ru_spb-icon.png   (rastérisation 256×256 via Pillow — asset embarqué + bureau)

Le SVG est la source de design ; le PNG est l'asset réellement embarqué
(`branding.rs`) et installé sur le bureau (`scripts/install-desktop.sh ru_spb`).
Rastériser via Pillow car aucune chaîne SVG→PNG n'est supposée présente.
"""
import math

ORANGE = "#F29400"
DARK = "#1A171B"
WHITE = "#FFFFFF"
GREY = "#6E6A70"
TICK = "#4A464C"

# Géométrie du cadran (réduit pour laisser place à l'éclair + nœuds autour).
R_ARC, R_FACE, ARC_W = 78, 62, 13
# Nœuds-abonnés MQTT disséminés autour du cadran (NON reliés — pub/sub diffusé).
# Trou au sommet, où se place l'éclair (le « publisher »).
RING = 100
RING_ANGLES = [171, 207, 243, 279, 315, 351, 27]
# Éclair Sparkplug (la « spark »), pointe vers le bas, centré au sommet du cadran.
BOLT_CX, BOLT_CY = 128, 30
BOLT = [(-3, -22), (9, -22), (1, -3), (11, -3), (-6, 24), (0, 1), (-10, 1)]


def pt(cx, cy, r, a_deg):
    a = math.radians(a_deg)
    return (cx + r * math.cos(a), cy + r * math.sin(a))


def f(x):
    return f"{x:.2f}".rstrip("0").rstrip(".")


def dial(cx, cy, scale=1.0):
    """Renvoie le <g> du cadran régulateur + éclair Sparkplug + nœuds-abonnés."""
    R_arc, R_face, arc_w = R_ARC * scale, R_FACE * scale, ARC_W * scale
    sx, sy = pt(cx, cy, R_arc, 135)
    ex, ey = pt(cx, cy, R_arc, 45)
    el = []
    el.append(f'<circle cx="{f(cx)}" cy="{f(cy)}" r="{f(R_face)}" fill="{DARK}"/>')
    # Arc ouvert 270° (trou en bas), de a=135 à a=45 via le haut — comme ORME.
    el.append(
        f'<path d="M {f(sx)} {f(sy)} A {f(R_arc)} {f(R_arc)} 0 1 1 {f(ex)} {f(ey)}" '
        f'fill="none" stroke="{ORANGE}" stroke-width="{f(arc_w)}" stroke-linecap="round"/>'
    )
    # Graduations (ticks) sur la face, de 135 à 405 par pas de 27°.
    r1, r2 = 42 * scale, 54 * scale
    a = 135
    while a <= 405.5:
        x1, y1 = pt(cx, cy, r1, a)
        x2, y2 = pt(cx, cy, r2, a)
        major = abs((a - 135) % 270) < 0.1 or abs(a - 270) < 0.1 or abs(a - 405) < 0.1
        col = ORANGE if major else TICK
        w = 5 * scale if major else 3 * scale
        el.append(f'<line x1="{f(x1)}" y1="{f(y1)}" x2="{f(x2)}" y2="{f(y2)}" '
                  f'stroke="{col}" stroke-width="{f(w)}" stroke-linecap="round"/>')
        a += 27
    # Aiguille (consigne) vers le haut-droite, a=312.
    a_needle = 312
    tipx, tipy = pt(cx, cy, 50 * scale, a_needle)
    bl = pt(cx, cy, 10 * scale, a_needle - 90)
    br = pt(cx, cy, 10 * scale, a_needle + 90)
    tail = pt(cx, cy, 18 * scale, a_needle + 180)
    el.append(f'<polygon points="{f(tipx)},{f(tipy)} {f(bl[0])},{f(bl[1])} '
              f'{f(tail[0])},{f(tail[1])} {f(br[0])},{f(br[1])}" fill="{ORANGE}"/>')
    # Moyeu central.
    el.append(f'<circle cx="{f(cx)}" cy="{f(cy)}" r="{f(13*scale)}" fill="{ORANGE}"/>')
    el.append(f'<circle cx="{f(cx)}" cy="{f(cy)}" r="{f(5*scale)}" fill="{DARK}"/>')
    # Nœuds-abonnés MQTT (NON reliés) disséminés autour du cadran.
    for ang in RING_ANGLES:
        nx, ny = pt(cx, cy, RING * scale, ang)
        el.append(f'<circle cx="{f(nx)}" cy="{f(ny)}" r="{f(7*scale)}" fill="{ORANGE}"/>')
        el.append(f'<circle cx="{f(nx)}" cy="{f(ny)}" r="{f(2.8*scale)}" fill="{DARK}"/>')
    # Éclair Sparkplug (la « spark », le publisher), au sommet.
    bx, by = cx + (BOLT_CX - 128) * scale, cy + (BOLT_CY - 128) * scale
    pts = " ".join(f"{f(bx + dx*scale)},{f(by + dy*scale)}" for dx, dy in BOLT)
    el.append(f'<polygon points="{pts}" fill="{ORANGE}"/>')
    return "<g>\n    " + "\n    ".join(el) + "\n  </g>"


# --- Icône carrée 256×256 (cadran + éclair + nœuds centrés) ---
icon = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <title>ORSE — Open Regulator Sparkplug Emulator</title>
  {dial(128, 128)}
</svg>
'''
open("ru_spb-icon.svg", "w").write(icon)

# --- Lockup horizontal 760×240 (icône + texte) ---
# Marque **ORSE** (Open Regulator Sparkplug Emulator), même schéma qu'ORME/OSNE/ORUE :
# grand titre 4 lettres, deux dernières en orange (« OR » encre + « SE » orange). Le
# sous-titre est figé à une largeur fixe via `textLength` + `lengthAdjust` (police du
# visiteur variable : GitHub n'a pas DejaVu Sans et substitue une police plus large).
FONT = "'DejaVu Sans','Segoe UI',Helvetica,Arial,sans-serif"
SUBTITLE_W = 410  # largeur fixe du sous-titre
logo = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 760 240" width="760" height="240">
  <title>ORSE — Open Regulator Sparkplug Emulator</title>
  <style>
    .ink {{ fill: {DARK}; }}
    .sub {{ fill: {GREY}; }}
    @media (prefers-color-scheme: dark) {{
      .ink {{ fill: #ECECEC; }}
      .sub {{ fill: #B7B3B9; }}
    }}
  </style>
  <g transform="translate(2,0)">
    {dial(120, 120)}
  </g>
  <text x="250" y="118" font-family="{FONT}" font-size="104" font-weight="800"
        class="ink" letter-spacing="2">OR<tspan fill="{ORANGE}">SE</tspan></text>
  <text x="252" y="158" font-family="{FONT}" font-size="27" font-weight="600"
        class="sub" letter-spacing="0.5"
        textLength="{SUBTITLE_W}" lengthAdjust="spacingAndGlyphs">Open Regulator Sparkplug Emulator</text>
  <text x="252" y="192" font-family="{FONT}" font-size="23" font-weight="700"
        fill="{ORANGE}" font-style="italic">« Publiez le procédé. »</text>
</svg>
'''
open("ru_spb-logo.svg", "w").write(logo)


# --- Rastérisation PNG 256×256 (Pillow, supersampling ×4 pour l'anticrénelage) ---
def render_png(path, size=256, ss=4):
    from PIL import Image, ImageDraw

    S = size * ss
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    sc = ss  # 1 unité du repère 256 = `ss` pixels
    cx, cy = 128 * sc, 128 * sc

    def hexc(h):
        h = h.lstrip("#")
        return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))

    orange, dark, tick = hexc(ORANGE), hexc(DARK), hexc(TICK)

    R_arc, R_face, arc_w = R_ARC * sc, R_FACE * sc, ARC_W * sc
    d.ellipse([cx - R_face, cy - R_face, cx + R_face, cy + R_face], fill=dark + (255,))
    d.arc([cx - R_arc, cy - R_arc, cx + R_arc, cy + R_arc], 135, 45, fill=orange + (255,), width=int(arc_w))
    for a in (135, 45):
        ex, ey = pt(cx, cy, R_arc, a)
        d.ellipse([ex - arc_w / 2, ey - arc_w / 2, ex + arc_w / 2, ey + arc_w / 2], fill=orange + (255,))
    r1, r2 = 42 * sc, 54 * sc
    a = 135
    while a <= 405.5:
        x1, y1 = pt(cx, cy, r1, a)
        x2, y2 = pt(cx, cy, r2, a)
        major = abs((a - 135) % 270) < 0.1 or abs(a - 270) < 0.1 or abs(a - 405) < 0.1
        col = orange if major else tick
        w = 5 * sc if major else 3 * sc
        d.line([(x1, y1), (x2, y2)], fill=col + (255,), width=int(w))
        a += 27
    a_needle = 312
    tip = pt(cx, cy, 50 * sc, a_needle)
    bl = pt(cx, cy, 10 * sc, a_needle - 90)
    br = pt(cx, cy, 10 * sc, a_needle + 90)
    tail = pt(cx, cy, 18 * sc, a_needle + 180)
    d.polygon([tip, bl, tail, br], fill=orange + (255,))
    d.ellipse([cx - 13 * sc, cy - 13 * sc, cx + 13 * sc, cy + 13 * sc], fill=orange + (255,))
    d.ellipse([cx - 5 * sc, cy - 5 * sc, cx + 5 * sc, cy + 5 * sc], fill=dark + (255,))
    # Nœuds-abonnés MQTT (NON reliés).
    for ang in RING_ANGLES:
        nx, ny = pt(cx, cy, RING * sc, ang)
        d.ellipse([nx - 7 * sc, ny - 7 * sc, nx + 7 * sc, ny + 7 * sc], fill=orange + (255,))
        d.ellipse([nx - 2.8 * sc, ny - 2.8 * sc, nx + 2.8 * sc, ny + 2.8 * sc], fill=dark + (255,))
    # Éclair Sparkplug (la « spark »).
    bx, by = BOLT_CX * sc, BOLT_CY * sc
    d.polygon([(bx + dx * sc, by + dy * sc) for dx, dy in BOLT], fill=orange + (255,))

    img = img.resize((size, size), Image.LANCZOS)
    img.save(path)


render_png("ru_spb-icon.png")
print("écrit: ru_spb-icon.svg, ru_spb-logo.svg, ru_spb-icon.png")
