#!/usr/bin/env python3
"""Génère le logo ORPD (régulateur PROFIBUS DP) — icône carrée + lockup horizontal.

Marque : **ORPD** = *Open Regulator Profibus DP* (nom technique : RU/PBDP ; marque
prononçable à 4 lettres, cohérente avec ORME/OSNE/ORUE/ORSE/ORSS/OREE).

Style CESAM-Lab, cohérent avec les autres instruments : orange #F29400, anthracite
#1A171B, blanc, gris. Motif : le **cadran de régulation** d'ORME (RU = Regulation
Unit, même identité de régulateur), surmontant un **segment de bus** PROFIBUS —
ligne horizontale terminée par deux **résistances de terminaison** (blocs
perpendiculaires aux extrémités, signature électrique du bus RS-485) et trois
**dérivations en T** descendant vers de petits nœuds de station. C'est ce motif de
bus filaire terminé qui distingue cet instrument d'ORME (table de registres),
d'ORSS (rack de modules), d'ORUE (nœuds ronds OPC UA) et d'OREE (anneau fermé
EtherNet/IP) — PROFIBUS DP est avant tout un **bus physique**, pas un protocole
purement applicatif.

Sorties :
  - ru_pbdp-icon.svg   (source vectorielle de l'icône carrée 256×256)
  - ru_pbdp-logo.svg   (lockup horizontal 760×240 : icône + texte)
  - ru_pbdp-icon.png   (rastérisation 256×256 via Pillow — asset embarqué + bureau)

Le SVG est la source de design ; le PNG est l'asset réellement embarqué
(`branding.rs`) et installé sur le bureau (`scripts/install-desktop.sh ru_pbdp`).
Rastériser via Pillow car aucune chaîne SVG→PNG n'est supposée présente.
"""
import math

ORANGE = "#F29400"
DARK = "#1A171B"
WHITE = "#FFFFFF"
GREY = "#6E6A70"
TICK = "#4A464C"

# Géométrie du cadran (réduit et remonté pour laisser place au bus en dessous).
R_ARC, R_FACE, ARC_W = 66, 52, 11
CY_DIAL_OFFSET = -28  # décale le centre du cadran vers le haut
BUS_Y_OFFSET = 78     # position verticale du bus sous le centre de l'icône
BUS_HALF_W = 108       # demi-portée du segment de bus
STUB_XS = [-68, 0, 68]  # abscisses des trois dérivations en T
STUB_LEN = 24
NODE_HS = 8            # demi-côté des nœuds de station


def pt(cx, cy, r, a_deg):
    a = math.radians(a_deg)
    return (cx + r * math.cos(a), cy + r * math.sin(a))


def f(x):
    return f"{x:.2f}".rstrip("0").rstrip(".")


def dial(cx, cy, scale=1.0):
    """Cadran régulateur (identité ORME), sans le rack/anneau des autres marques."""
    R_arc, R_face, arc_w = R_ARC * scale, R_FACE * scale, ARC_W * scale
    sx, sy = pt(cx, cy, R_arc, 135)
    ex, ey = pt(cx, cy, R_arc, 45)
    el = []
    el.append(f'<circle cx="{f(cx)}" cy="{f(cy)}" r="{f(R_face)}" fill="{DARK}"/>')
    el.append(
        f'<path d="M {f(sx)} {f(sy)} A {f(R_arc)} {f(R_arc)} 0 1 1 {f(ex)} {f(ey)}" '
        f'fill="none" stroke="{ORANGE}" stroke-width="{f(arc_w)}" stroke-linecap="round"/>'
    )
    r1, r2 = 36 * scale, 46 * scale
    a = 135
    while a <= 405.5:
        x1, y1 = pt(cx, cy, r1, a)
        x2, y2 = pt(cx, cy, r2, a)
        major = abs((a - 135) % 270) < 0.1 or abs(a - 270) < 0.1 or abs(a - 405) < 0.1
        col = ORANGE if major else TICK
        w = 4.2 * scale if major else 2.4 * scale
        el.append(f'<line x1="{f(x1)}" y1="{f(y1)}" x2="{f(x2)}" y2="{f(y2)}" '
                  f'stroke="{col}" stroke-width="{f(w)}" stroke-linecap="round"/>')
        a += 27
    a_needle = 312
    tipx, tipy = pt(cx, cy, 42 * scale, a_needle)
    bl = pt(cx, cy, 8 * scale, a_needle - 90)
    br = pt(cx, cy, 8 * scale, a_needle + 90)
    tail = pt(cx, cy, 15 * scale, a_needle + 180)
    el.append(f'<polygon points="{f(tipx)},{f(tipy)} {f(bl[0])},{f(bl[1])} '
              f'{f(tail[0])},{f(tail[1])} {f(br[0])},{f(br[1])}" fill="{ORANGE}"/>')
    el.append(f'<circle cx="{f(cx)}" cy="{f(cy)}" r="{f(11*scale)}" fill="{ORANGE}"/>')
    el.append(f'<circle cx="{f(cx)}" cy="{f(cy)}" r="{f(4.2*scale)}" fill="{DARK}"/>')
    return el


def bus(cx, bus_y, scale=1.0):
    """Segment de bus PROFIBUS : trunk + 2 résistances de terminaison + 3 dérivations en T."""
    half = BUS_HALF_W * scale
    x0, x1 = cx - half, cx + half
    el = []
    # Trunk (câble bus).
    el.append(f'<line x1="{f(x0)}" y1="{f(bus_y)}" x2="{f(x1)}" y2="{f(bus_y)}" '
              f'stroke="{ORANGE}" stroke-width="{f(6*scale)}" stroke-linecap="round"/>')
    # Résistances de terminaison (blocs perpendiculaires aux deux extrémités).
    term_h = 22 * scale
    term_w = 9 * scale
    for x in (x0, x1):
        el.append(f'<rect x="{f(x-term_w/2)}" y="{f(bus_y-term_h/2)}" width="{f(term_w)}" '
                  f'height="{f(term_h)}" rx="{f(2*scale)}" fill="{DARK}" '
                  f'stroke="{ORANGE}" stroke-width="{f(2.5*scale)}"/>')
    # Dérivations en T : ligne descendante + nœud de station.
    stub_len = STUB_LEN * scale
    hs = NODE_HS * scale
    for dx in STUB_XS:
        x = cx + dx * scale
        y2 = bus_y + stub_len
        el.append(f'<line x1="{f(x)}" y1="{f(bus_y)}" x2="{f(x)}" y2="{f(y2)}" '
                  f'stroke="{ORANGE}" stroke-width="{f(4*scale)}" stroke-linecap="round"/>')
        el.append(f'<rect x="{f(x-hs)}" y="{f(y2-hs)}" width="{f(2*hs)}" height="{f(2*hs)}" '
                  f'rx="{f(1.5*scale)}" fill="{ORANGE}"/>')
        el.append(f'<rect x="{f(x-hs*0.4)}" y="{f(y2-hs*0.4)}" width="{f(hs*0.8)}" '
                  f'height="{f(hs*0.8)}" fill="{DARK}"/>')
    return el


def artwork(cx, cy, scale=1.0):
    el = dial(cx, cy + CY_DIAL_OFFSET * scale, scale) + bus(cx, cy + BUS_Y_OFFSET * scale, scale)
    return "<g>\n    " + "\n    ".join(el) + "\n  </g>"


# --- Icône carrée 256×256 ---
icon = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <title>ORPD — Open Regulator Profibus DP</title>
  {artwork(128, 128)}
</svg>
'''
open("ru_pbdp-icon.svg", "w").write(icon)

# --- Lockup horizontal 760×240 (icône + texte) ---
FONT = "'DejaVu Sans','Segoe UI',Helvetica,Arial,sans-serif"
SUBTITLE_W = 400
logo = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 760 240" width="760" height="240">
  <title>ORPD — Open Regulator Profibus DP</title>
  <style>
    .ink {{ fill: {DARK}; }}
    .sub {{ fill: {GREY}; }}
    @media (prefers-color-scheme: dark) {{
      .ink {{ fill: #ECECEC; }}
      .sub {{ fill: #B7B3B9; }}
    }}
  </style>
  <g transform="translate(2,0)">
    {artwork(120, 120)}
  </g>
  <text x="250" y="118" font-family="{FONT}" font-size="104" font-weight="800"
        class="ink" letter-spacing="2">OR<tspan fill="{ORANGE}">PD</tspan></text>
  <text x="252" y="158" font-family="{FONT}" font-size="27" font-weight="600"
        class="sub" letter-spacing="0.5"
        textLength="{SUBTITLE_W}" lengthAdjust="spacingAndGlyphs">Open Regulator Profibus DP</text>
  <text x="252" y="192" font-family="{FONT}" font-size="23" font-weight="700"
        fill="{ORANGE}" font-style="italic">« Simulez le bus, pas le matériel. »</text>
</svg>
'''
open("ru_pbdp-logo.svg", "w").write(logo)


# --- Rastérisation PNG 256×256 (Pillow, supersampling ×4 pour l'anticrénelage) ---
def render_png(path, size=256, ss=4):
    from PIL import Image, ImageDraw

    S = size * ss
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    sc = ss
    cx, cy = 128 * sc, 128 * sc

    def hexc(h):
        h = h.lstrip("#")
        return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))

    orange, dark, tick = hexc(ORANGE), hexc(DARK), hexc(TICK)

    # Cadran.
    dcy = cy + CY_DIAL_OFFSET * sc
    R_arc, R_face, arc_w = R_ARC * sc, R_FACE * sc, ARC_W * sc
    d.ellipse([cx - R_face, dcy - R_face, cx + R_face, dcy + R_face], fill=dark + (255,))
    d.arc([cx - R_arc, dcy - R_arc, cx + R_arc, dcy + R_arc], 135, 45, fill=orange + (255,), width=int(arc_w))
    for a in (135, 45):
        ex, ey = pt(cx, dcy, R_arc, a)
        d.ellipse([ex - arc_w / 2, ey - arc_w / 2, ex + arc_w / 2, ey + arc_w / 2], fill=orange + (255,))
    r1, r2 = 36 * sc, 46 * sc
    a = 135
    while a <= 405.5:
        x1, y1 = pt(cx, dcy, r1, a)
        x2, y2 = pt(cx, dcy, r2, a)
        major = abs((a - 135) % 270) < 0.1 or abs(a - 270) < 0.1 or abs(a - 405) < 0.1
        col = orange if major else tick
        w = 4.2 * sc if major else 2.4 * sc
        d.line([(x1, y1), (x2, y2)], fill=col + (255,), width=int(w))
        a += 27
    a_needle = 312
    tip = pt(cx, dcy, 42 * sc, a_needle)
    bl = pt(cx, dcy, 8 * sc, a_needle - 90)
    br = pt(cx, dcy, 8 * sc, a_needle + 90)
    tail = pt(cx, dcy, 15 * sc, a_needle + 180)
    d.polygon([tip, bl, tail, br], fill=orange + (255,))
    d.ellipse([cx - 11 * sc, dcy - 11 * sc, cx + 11 * sc, dcy + 11 * sc], fill=orange + (255,))
    d.ellipse([cx - 4.2 * sc, dcy - 4.2 * sc, cx + 4.2 * sc, dcy + 4.2 * sc], fill=dark + (255,))

    # Bus PROFIBUS.
    bus_y = cy + BUS_Y_OFFSET * sc
    half = BUS_HALF_W * sc
    x0, x1 = cx - half, cx + half
    d.line([(x0, bus_y), (x1, bus_y)], fill=orange + (255,), width=int(6 * sc))
    term_h, term_w = 22 * sc, 9 * sc
    for x in (x0, x1):
        d.rectangle([x - term_w / 2, bus_y - term_h / 2, x + term_w / 2, bus_y + term_h / 2], fill=dark + (255,))
        d.rectangle(
            [x - term_w / 2, bus_y - term_h / 2, x + term_w / 2, bus_y + term_h / 2],
            outline=orange + (255,), width=max(1, int(2.5 * sc)),
        )
    stub_len = STUB_LEN * sc
    hs = NODE_HS * sc
    for dx in STUB_XS:
        x = cx + dx * sc
        y2 = bus_y + stub_len
        d.line([(x, bus_y), (x, y2)], fill=orange + (255,), width=int(4 * sc))
        d.rectangle([x - hs, y2 - hs, x + hs, y2 + hs], fill=orange + (255,))
        d.rectangle([x - hs * 0.4, y2 - hs * 0.4, x + hs * 0.4, y2 + hs * 0.4], fill=dark + (255,))

    img = img.resize((size, size), Image.LANCZOS)
    img.save(path)


render_png("ru_pbdp-icon.png")
print("écrit: ru_pbdp-icon.svg, ru_pbdp-logo.svg, ru_pbdp-icon.png")
