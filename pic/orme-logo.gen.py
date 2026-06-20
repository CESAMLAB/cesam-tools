#!/usr/bin/env python3
"""Génère le logo ORME (cadran de régulation) — icône carrée + lockup horizontal.
Style CESAM-Lab : orange #F29400, anthracite #1A171B, blanc."""
import math

ORANGE = "#F29400"
DARK   = "#1A171B"
WHITE  = "#FFFFFF"
GREY   = "#6E6A70"

def pt(cx, cy, r, a_deg):
    a = math.radians(a_deg)
    return (cx + r * math.cos(a), cy + r * math.sin(a))

def f(x):  # format compact
    return f"{x:.2f}".rstrip("0").rstrip(".")

def dial(cx, cy, scale=1.0):
    """Renvoie le <g> du cadran (sans texte). a=0 droite, 90 bas, 270 haut (y vers le bas)."""
    R_arc = 92 * scale
    R_face = 74 * scale
    arc_w = 17 * scale
    # Arc ouvert 270° : trou en bas, de a=135 à a=405 (=45) via le haut.
    sx, sy = pt(cx, cy, R_arc, 135)
    ex, ey = pt(cx, cy, R_arc, 45)
    el = []
    el.append(f'<circle cx="{f(cx)}" cy="{f(cy)}" r="{f(R_face)}" fill="{DARK}"/>')
    # Arc orange (épais, bouts ronds).
    el.append(
        f'<path d="M {f(sx)} {f(sy)} A {f(R_arc)} {f(R_arc)} 0 1 1 {f(ex)} {f(ey)}" '
        f'fill="none" stroke="{ORANGE}" stroke-width="{f(arc_w)}" stroke-linecap="round"/>'
    )
    # Graduations (ticks) sur la face, de 135 à 405 par pas de 27°.
    r1, r2 = 50 * scale, 64 * scale
    a = 135
    while a <= 405.5:
        x1, y1 = pt(cx, cy, r1, a)
        x2, y2 = pt(cx, cy, r2, a)
        major = abs((a - 135) % 270) < 0.1 or abs(a - 270) < 0.1 or abs(a - 405) < 0.1
        col = ORANGE if major else "#4A464C"
        w = 5 * scale if major else 3 * scale
        el.append(f'<line x1="{f(x1)}" y1="{f(y1)}" x2="{f(x2)}" y2="{f(y2)}" '
                  f'stroke="{col}" stroke-width="{f(w)}" stroke-linecap="round"/>')
        a += 27
    # Aiguille (consigne) vers le haut-droite, a=312.
    a_needle = 312
    tipx, tipy = pt(cx, cy, 60 * scale, a_needle)
    bl = pt(cx, cy, 11 * scale, a_needle - 90)
    br = pt(cx, cy, 11 * scale, a_needle + 90)
    tail = pt(cx, cy, 20 * scale, a_needle + 180)
    el.append(f'<polygon points="{f(tipx)},{f(tipy)} {f(bl[0])},{f(bl[1])} '
              f'{f(tail[0])},{f(tail[1])} {f(br[0])},{f(br[1])}" fill="{ORANGE}"/>')
    # Moyeu central.
    el.append(f'<circle cx="{f(cx)}" cy="{f(cy)}" r="{f(15*scale)}" fill="{ORANGE}"/>')
    el.append(f'<circle cx="{f(cx)}" cy="{f(cy)}" r="{f(6*scale)}" fill="{DARK}"/>')
    # Bus Modbus : descente depuis le cadran à travers l'ouverture + 3 nœuds.
    by = cy + 112 * scale
    stub_top = cy + R_face
    el.append(f'<line x1="{f(cx)}" y1="{f(stub_top)}" x2="{f(cx)}" y2="{f(by)}" '
              f'stroke="{ORANGE}" stroke-width="{f(6*scale)}" stroke-linecap="round"/>')
    el.append(f'<line x1="{f(cx-34*scale)}" y1="{f(by)}" x2="{f(cx+34*scale)}" y2="{f(by)}" '
              f'stroke="{ORANGE}" stroke-width="{f(6*scale)}" stroke-linecap="round"/>')
    for dx in (-34, 0, 34):
        nx = cx + dx * scale
        el.append(f'<circle cx="{f(nx)}" cy="{f(by)}" r="{f(8*scale)}" fill="{ORANGE}"/>')
        el.append(f'<circle cx="{f(nx)}" cy="{f(by)}" r="{f(3.5*scale)}" fill="{DARK}"/>')
    return "<g>\n    " + "\n    ".join(el) + "\n  </g>"

# --- Icône carrée 256×256 ---
icon = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <title>ORME — Open Regulator Modbus Emulator</title>
  {dial(128, 110)}
</svg>
'''
open("orme-icon.svg", "w").write(icon)

# --- Lockup horizontal 760×220 (icône + texte) ---
FONT = "'DejaVu Sans','Segoe UI',Helvetica,Arial,sans-serif"
logo = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 760 220" width="760" height="220">
  <title>ORME — Open Regulator Modbus Emulator</title>
  <g transform="translate(20,0)">
    {dial(110, 110)}
  </g>
  <text x="250" y="118" font-family="{FONT}" font-size="104" font-weight="800"
        fill="{DARK}" letter-spacing="2">OR<tspan fill="{ORANGE}">ME</tspan></text>
  <text x="252" y="158" font-family="{FONT}" font-size="27" font-weight="600"
        fill="{GREY}" letter-spacing="0.5">Open Regulator Modbus Emulator</text>
  <text x="252" y="192" font-family="{FONT}" font-size="23" font-weight="700"
        fill="{ORANGE}" font-style="italic">« Ouvrez le bus. »</text>
</svg>
'''
open("orme-logo.svg", "w").write(logo)
print("écrit: orme-icon.svg, orme-logo.svg")
