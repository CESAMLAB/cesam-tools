#!/usr/bin/env python3
"""Génère le logo OSNE (agitateur de laboratoire) — icône carrée + lockup horizontal.

Style CESAM-Lab, cohérent avec ORME : orange #F29400, anthracite #1A171B, blanc,
gris. Motif : un **agitateur** = arbre vertical + hélice (turbine à pales
inclinées) au centre d'un arc orange ouvert (le même cadran que l'icône ORME, ici
suggérant la rotation du mobile dans la cuve).

Sorties :
  - osne-icon.svg   (source vectorielle de l'icône carrée 256×256)
  - osne-logo.svg   (lockup horizontal 760×240 : icône + texte)
  - osne-icon.png   (rastérisation 256×256 via Pillow — asset embarqué + bureau)

Le SVG est la source de design ; le PNG est l'asset réellement embarqué
(`branding.rs`) et installé sur le bureau (`scripts/install-desktop.sh osne`).
Rastériser via Pillow car aucune chaîne SVG→PNG n'est supposée présente.
"""
import math

ORANGE = "#F29400"
DARK = "#1A171B"
WHITE = "#FFFFFF"
GREY = "#6E6A70"
TICK = "#4A464C"


def pt(cx, cy, r, a_deg):
    a = math.radians(a_deg)
    return (cx + r * math.cos(a), cy + r * math.sin(a))


def f(x):
    return f"{x:.2f}".rstrip("0").rstrip(".")


def stirrer(cx, cy, scale=1.0):
    """Renvoie le <g> de l'agitateur (cadran + arbre + hélice). y vers le bas."""
    R_arc = 92 * scale
    R_face = 74 * scale
    arc_w = 17 * scale
    # Arc ouvert 270° (trou en bas), de a=135 à a=45 via le haut — comme ORME.
    sx, sy = pt(cx, cy, R_arc, 135)
    ex, ey = pt(cx, cy, R_arc, 45)
    el = []
    el.append(f'<circle cx="{f(cx)}" cy="{f(cy)}" r="{f(R_face)}" fill="{DARK}"/>')
    el.append(
        f'<path d="M {f(sx)} {f(sy)} A {f(R_arc)} {f(R_arc)} 0 1 1 {f(ex)} {f(ey)}" '
        f'fill="none" stroke="{ORANGE}" stroke-width="{f(arc_w)}" stroke-linecap="round"/>'
    )
    # Deux flèches de rotation (chevrons) aux extrémités de l'arc : sens de rotation.
    for a_tip, a_from in ((45, 27), (225, 207)):
        tx, ty = pt(cx, cy, R_arc, a_tip)
        h1 = pt(cx, cy, R_arc + 14 * scale, a_from)
        h2 = pt(cx, cy, R_arc - 14 * scale, a_from)
        el.append(
            f'<polygon points="{f(tx)},{f(ty)} {f(h1[0])},{f(h1[1])} {f(h2[0])},{f(h2[1])}" '
            f'fill="{ORANGE}"/>'
        )
    # Arbre vertical (de la tête moteur jusqu'au moyeu de l'hélice).
    top = cy - 50 * scale
    el.append(
        f'<line x1="{f(cx)}" y1="{f(top)}" x2="{f(cx)}" y2="{f(cy)}" '
        f'stroke="{ORANGE}" stroke-width="{f(8 * scale)}" stroke-linecap="round"/>'
    )
    # Tête moteur (petit bloc en haut de l'arbre).
    el.append(
        f'<rect x="{f(cx - 20 * scale)}" y="{f(top - 16 * scale)}" '
        f'width="{f(40 * scale)}" height="{f(20 * scale)}" rx="{f(6 * scale)}" fill="{ORANGE}"/>'
    )
    # Hélice : 4 pales inclinées (ellipses) en croix autour du moyeu.
    blade_l = 40 * scale
    blade_w = 15 * scale
    for a in (35, 125, 215, 305):
        bx, by = pt(cx, cy, blade_l * 0.5, a)
        el.append(
            f'<g transform="translate({f(bx)},{f(by)}) rotate({f(a)})">'
            f'<ellipse cx="0" cy="0" rx="{f(blade_l * 0.5)}" ry="{f(blade_w * 0.5)}" fill="{ORANGE}"/></g>'
        )
    # Moyeu central.
    el.append(f'<circle cx="{f(cx)}" cy="{f(cy)}" r="{f(13 * scale)}" fill="{ORANGE}"/>')
    el.append(f'<circle cx="{f(cx)}" cy="{f(cy)}" r="{f(5.5 * scale)}" fill="{DARK}"/>')
    return "<g>\n    " + "\n    ".join(el) + "\n  </g>"


# --- Icône carrée 256×256 ---
icon = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <title>OSNE — Open Stirrer NAMUR Emulator</title>
  {stirrer(128, 128)}
</svg>
'''
open("osne-icon.svg", "w").write(icon)

# --- Lockup horizontal 760×240 (icône + texte) ---
FONT = "'DejaVu Sans','Segoe UI',Helvetica,Arial,sans-serif"
SUBTITLE_W = 470
logo = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 760 240" width="760" height="240">
  <title>OSNE — Open Stirrer NAMUR Emulator</title>
  <style>
    .ink {{ fill: {DARK}; }}
    .sub {{ fill: {GREY}; }}
    @media (prefers-color-scheme: dark) {{
      .ink {{ fill: #ECECEC; }}
      .sub {{ fill: #B7B3B9; }}
    }}
  </style>
  <g transform="translate(8,8)">
    {stirrer(112, 112)}
  </g>
  <text x="250" y="118" font-family="{FONT}" font-size="104" font-weight="800"
        class="ink" letter-spacing="2">OS<tspan fill="{ORANGE}">NE</tspan></text>
  <text x="252" y="158" font-family="{FONT}" font-size="27" font-weight="600"
        class="sub" letter-spacing="0.5"
        textLength="{SUBTITLE_W}" lengthAdjust="spacingAndGlyphs">Open Stirrer NAMUR Emulator</text>
  <text x="252" y="192" font-family="{FONT}" font-size="23" font-weight="700"
        fill="{ORANGE}" font-style="italic">« Agitez le procédé. »</text>
</svg>
'''
open("osne-logo.svg", "w").write(logo)


# --- Rastérisation PNG 256×256 (Pillow, supersampling ×4 pour l'anticrénelage) ---
def render_png(path, size=256, ss=4):
    from PIL import Image, ImageDraw

    S = size * ss
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cx = cy = S / 2
    sc = ss  # 1 unité du repère 256 = `ss` pixels

    def P(x, y):
        return (x, y)

    def hexc(h):
        h = h.lstrip("#")
        return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))

    orange, dark = hexc(ORANGE), hexc(DARK)

    R_arc, R_face, arc_w = 92 * sc, 74 * sc, 17 * sc
    # Face anthracite.
    d.ellipse([cx - R_face, cy - R_face, cx + R_face, cy + R_face], fill=dark + (255,))
    # Arc orange ouvert 270° (135°→405° via le haut), épais à bouts ronds.
    bbox = [cx - R_arc, cy - R_arc, cx + R_arc, cy + R_arc]
    d.arc(bbox, start=135, end=45, fill=orange + (255,), width=int(arc_w))
    # Bouts ronds de l'arc.
    for a in (135, 45):
        ex, ey = pt(cx, cy, R_arc, a)
        d.ellipse([ex - arc_w / 2, ey - arc_w / 2, ex + arc_w / 2, ey + arc_w / 2], fill=orange + (255,))
    # Chevrons (flèches de rotation).
    for a_tip, a_from in ((45, 27), (225, 207)):
        tx, ty = pt(cx, cy, R_arc, a_tip)
        h1 = pt(cx, cy, R_arc + 14 * sc, a_from)
        h2 = pt(cx, cy, R_arc - 14 * sc, a_from)
        d.polygon([P(tx, ty), P(*h1), P(*h2)], fill=orange + (255,))
    # Arbre vertical.
    top = cy - 50 * sc
    d.line([P(cx, top), P(cx, cy)], fill=orange + (255,), width=int(8 * sc))
    d.ellipse([cx - 4 * sc, top - 4 * sc, cx + 4 * sc, top + 4 * sc], fill=orange + (255,))
    # Tête moteur.
    d.rounded_rectangle([cx - 20 * sc, top - 16 * sc, cx + 20 * sc, top + 4 * sc],
                        radius=6 * sc, fill=orange + (255,))
    # Hélice : 4 pales inclinées (ellipses dessinées puis pivotées).
    blade_l, blade_w = 40 * sc, 15 * sc
    for a in (35, 125, 215, 305):
        blade = Image.new("RGBA", (int(blade_l) + 4, int(blade_w) + 4), (0, 0, 0, 0))
        bd = ImageDraw.Draw(blade)
        bd.ellipse([2, 2, blade_l + 2, blade_w + 2], fill=orange + (255,))
        blade = blade.rotate(-a, expand=True, resample=Image.BICUBIC)
        bx, by = pt(cx, cy, blade_l * 0.5, a)
        img.alpha_composite(blade, (int(bx - blade.width / 2), int(by - blade.height / 2)))
    # Moyeu.
    d.ellipse([cx - 13 * sc, cy - 13 * sc, cx + 13 * sc, cy + 13 * sc], fill=orange + (255,))
    d.ellipse([cx - 5.5 * sc, cy - 5.5 * sc, cx + 5.5 * sc, cy + 5.5 * sc], fill=dark + (255,))

    img = img.resize((size, size), Image.LANCZOS)
    img.save(path)


render_png("osne-icon.png")
print("écrit: osne-icon.svg, osne-logo.svg, osne-icon.png")
