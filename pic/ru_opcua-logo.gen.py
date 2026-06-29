#!/usr/bin/env python3
"""Génère le logo ORUE (régulateur OPC UA) — icône carrée + lockup horizontal.

Marque : **ORUE** = *Open Regulator UA Emulator* (nom technique : RU/OPC UA).


Style CESAM-Lab, cohérent avec ORME et OSNE : orange #F29400, anthracite #1A171B,
blanc, gris. Motif : le **cadran de régulation** d'ORME (RU = Regulation Unit, même
identité de régulateur), **enveloppé d'un anneau de nœuds OPC UA** — des nœuds
répartis le long de l'arc, reliés en chaîne par des références, évoquant l'**espace
d'adressage** d'OPC UA qui entoure le régulateur. C'est ce qui distingue cet
instrument d'ORME (bus Modbus) et d'OSNE (agitateur).

Sorties :
  - ru_opcua-icon.svg   (source vectorielle de l'icône carrée 256×256)
  - ru_opcua-logo.svg   (lockup horizontal 760×240 : icône + texte)
  - ru_opcua-icon.png   (rastérisation 256×256 via Pillow — asset embarqué + bureau)

Le SVG est la source de design ; le PNG est l'asset réellement embarqué
(`branding.rs`) et installé sur le bureau (`scripts/install-desktop.sh ru_opcua`).
Rastériser via Pillow car aucune chaîne SVG→PNG n'est supposée présente.
"""
import math

ORANGE = "#F29400"
DARK = "#1A171B"
WHITE = "#FFFFFF"
GREY = "#6E6A70"
TICK = "#4A464C"

# Géométrie du cadran (réduit pour laisser place à l'anneau de nœuds autour).
R_ARC, R_FACE, ARC_W = 78, 62, 13
# Anneau OPC UA : nœuds répartis le long de l'arc (de 135° à 27° via le haut), trou
# en bas comme l'ouverture du cadran. Reliés en chaîne (références de l'espace d'adressage).
RING = 100
RING_ANGLES = [135, 171, 207, 243, 279, 315, 351, 27]


def pt(cx, cy, r, a_deg):
    a = math.radians(a_deg)
    return (cx + r * math.cos(a), cy + r * math.sin(a))


def f(x):
    return f"{x:.2f}".rstrip("0").rstrip(".")


def dial(cx, cy, scale=1.0):
    """Renvoie le <g> du cadran régulateur enveloppé de l'anneau de nœuds OPC UA."""
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
    # Anneau OPC UA : chaîne de nœuds le long de l'arc + nœuds.
    ring_pts = [pt(cx, cy, RING * scale, a) for a in RING_ANGLES]
    for (x1, y1), (x2, y2) in zip(ring_pts, ring_pts[1:]):
        el.append(f'<line x1="{f(x1)}" y1="{f(y1)}" x2="{f(x2)}" y2="{f(y2)}" '
                  f'stroke="{ORANGE}" stroke-width="{f(3.5*scale)}" stroke-linecap="round"/>')
    for nx, ny in ring_pts:
        el.append(f'<circle cx="{f(nx)}" cy="{f(ny)}" r="{f(7*scale)}" fill="{ORANGE}"/>')
        el.append(f'<circle cx="{f(nx)}" cy="{f(ny)}" r="{f(2.8*scale)}" fill="{DARK}"/>')
    return "<g>\n    " + "\n    ".join(el) + "\n  </g>"


# --- Icône carrée 256×256 (cadran + anneau centrés) ---
icon = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <title>ORUE — Open Regulator UA Emulator</title>
  {dial(128, 128)}
</svg>
'''
open("ru_opcua-icon.svg", "w").write(icon)

# --- Lockup horizontal 760×240 (icône + texte) ---
# Marque **ORUE** (Open Regulator UA Emulator), même schéma qu'ORME/OSNE : grand
# titre 4 lettres, deux dernières en orange (« OR » encre + « UE » orange). Le
# sous-titre est figé à une largeur fixe via `textLength` + `lengthAdjust` : il
# occupe toujours la même place QUELLE QUE SOIT la police du visiteur (GitHub n'a
# pas DejaVu Sans et substitue une police plus large, ce qui couperait le texte).
FONT = "'DejaVu Sans','Segoe UI',Helvetica,Arial,sans-serif"
SUBTITLE_W = 410  # largeur fixe du sous-titre -> fin à x=252+410=662 < 760
# Couleurs du texte adaptées au thème : sur fond sombre (GitHub dark mode), l'encre
# anthracite et le sous-titre gris deviennent illisibles -> on les éclaircit via
# @media (prefers-color-scheme: dark). L'orange et le cadran fonctionnent sur les
# deux fonds. « OR » en encre, « UE » en orange (régulateur vs OPC UA).
logo = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 760 240" width="760" height="240">
  <title>ORUE — Open Regulator UA Emulator</title>
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
        class="ink" letter-spacing="2">OR<tspan fill="{ORANGE}">UE</tspan></text>
  <text x="252" y="158" font-family="{FONT}" font-size="27" font-weight="600"
        class="sub" letter-spacing="0.5"
        textLength="{SUBTITLE_W}" lengthAdjust="spacingAndGlyphs">Open Regulator UA Emulator</text>
  <text x="252" y="192" font-family="{FONT}" font-size="23" font-weight="700"
        fill="{ORANGE}" font-style="italic">« Unifiez le procédé. »</text>
</svg>
'''
open("ru_opcua-logo.svg", "w").write(logo)


# --- Rastérisation PNG 256×256 (Pillow, supersampling ×4 pour l'anticrénelage) ---
def render_png(path, size=256, ss=4):
    from PIL import Image, ImageDraw

    S = size * ss
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    sc = ss  # 1 unité du repère 256 = `ss` pixels
    cx, cy = 128 * sc, 128 * sc  # cadran + anneau centrés (comme l'icône SVG)

    def hexc(h):
        h = h.lstrip("#")
        return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))

    orange, dark, tick = hexc(ORANGE), hexc(DARK), hexc(TICK)

    R_arc, R_face, arc_w = R_ARC * sc, R_FACE * sc, ARC_W * sc
    # Face anthracite.
    d.ellipse([cx - R_face, cy - R_face, cx + R_face, cy + R_face], fill=dark + (255,))
    # Arc orange ouvert 270° (135°→45° via le haut), épais à bouts ronds.
    d.arc([cx - R_arc, cy - R_arc, cx + R_arc, cy + R_arc], 135, 45, fill=orange + (255,), width=int(arc_w))
    for a in (135, 45):
        ex, ey = pt(cx, cy, R_arc, a)
        d.ellipse([ex - arc_w / 2, ey - arc_w / 2, ex + arc_w / 2, ey + arc_w / 2], fill=orange + (255,))
    # Graduations (135°→405° par pas de 27°).
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
    # Aiguille (consigne) vers le haut-droite, a=312.
    a_needle = 312
    tip = pt(cx, cy, 50 * sc, a_needle)
    bl = pt(cx, cy, 10 * sc, a_needle - 90)
    br = pt(cx, cy, 10 * sc, a_needle + 90)
    tail = pt(cx, cy, 18 * sc, a_needle + 180)
    d.polygon([tip, bl, tail, br], fill=orange + (255,))
    # Moyeu central.
    d.ellipse([cx - 13 * sc, cy - 13 * sc, cx + 13 * sc, cy + 13 * sc], fill=orange + (255,))
    d.ellipse([cx - 5 * sc, cy - 5 * sc, cx + 5 * sc, cy + 5 * sc], fill=dark + (255,))
    # Anneau OPC UA : chaîne de nœuds + nœuds.
    ring_pts = [pt(cx, cy, RING * sc, ang) for ang in RING_ANGLES]
    for p1, p2 in zip(ring_pts, ring_pts[1:]):
        d.line([p1, p2], fill=orange + (255,), width=int(3.5 * sc))
    for nx, ny in ring_pts:
        d.ellipse([nx - 7 * sc, ny - 7 * sc, nx + 7 * sc, ny + 7 * sc], fill=orange + (255,))
        d.ellipse([nx - 2.8 * sc, ny - 2.8 * sc, nx + 2.8 * sc, ny + 2.8 * sc], fill=dark + (255,))

    img = img.resize((size, size), Image.LANCZOS)
    img.save(path)


render_png("ru_opcua-icon.png")
print("écrit: ru_opcua-icon.svg, ru_opcua-logo.svg, ru_opcua-icon.png")
