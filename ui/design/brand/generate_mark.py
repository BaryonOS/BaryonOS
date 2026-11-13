#!/usr/bin/env python3
"""Generate the refined BaryonOS mark and its variants as deterministic SVG.

The mark keeps the structure of the original logo: three coloured nodes on a
bonded triangle, a silver bond whose corners are carved by a clearance ring
around every node, and three tapered blades that sweep clockwise from node to
node. All geometry derives from one circumradius, so the native renderer can
reproduce it from the same constants (see docs/design/BRAND.md).

Usage: python3 -B ui/design/brand/generate_mark.py  (writes ui/design/brand/*.svg)
"""
from __future__ import annotations

import math
from pathlib import Path

OUT = Path(__file__).resolve().parent

# Construction constants, relative to the node circumradius R.
ROTATION = -7.0   # degrees; the original mark leans slightly anticlockwise
NODE = 0.22       # node radius
CLEAR = 0.07      # clearance ring between a node and any bond
BOND_OUTER = 0.8  # circumradius of the bond triangle (corners carved by clearance)
BOND = 0.15       # bond stroke thickness (perpendicular)
GAP = 0.07        # gap between bond and blade
BLADE = 0.22      # blade width at its root

RED, GREEN, BLUE = "#FF4A3D", "#33C46B", "#2585FF"
SILVER_HI, SILVER, SILVER_LO = "#F4F6F9", "#D7DCE3", "#A7B0BC"
SPACE = "#090B11"
INK = "#1A1F27"
LIGHT_RED, LIGHT_GREEN, LIGHT_BLUE = "#D9342A", "#14894A", "#1061D6"


def unit(deg: float) -> tuple[float, float]:
    rad = math.radians(deg)
    return math.cos(rad), math.sin(rad)


def geometry(cx: float, cy: float, r: float, *, compact: bool = False) -> dict:
    """Return nodes, clearance radius, bond rings and blade polygons."""
    node = NODE * r * (1.35 if compact else 1.0)
    clear = CLEAR * r * (1.3 if compact else 1.0)
    bond = BOND * r * (1.6 if compact else 1.0)
    angles = {"red": -90 + ROTATION, "blue": 30 + ROTATION, "green": 150 + ROTATION}
    centres = {k: (cx + r * unit(a)[0], cy + r * unit(a)[1]) for k, a in angles.items()}
    outer = {k: (cx + BOND_OUTER * r * unit(a)[0], cy + BOND_OUTER * r * unit(a)[1])
             for k, a in angles.items()}
    inner_r = BOND_OUTER * r - 2 * bond  # offsetting an equilateral edge by t moves vertices 2t
    inner = {k: (cx + inner_r * unit(a)[0], cy + inner_r * unit(a)[1]) for k, a in angles.items()}
    blades = []
    if not compact:
        # Clockwise sweep: wide at A, sharp tip just outside B's clearance ring.
        for a, b in (("red", "blue"), ("blue", "green"), ("green", "red")):
            (ax, ay), (bx, by) = outer[a], outer[b]
            dx, dy = bx - ax, by - ay
            length = math.hypot(dx, dy)
            dx, dy = dx / length, dy / length
            mx, my = (ax + bx) / 2, (ay + by) / 2
            nx, ny = mx - cx, my - cy
            nl = math.hypot(nx, ny)
            nx, ny = nx / nl, ny / nl
            line = (mx + nx * GAP * r, my + ny * GAP * r)
            # Tip: where the blade's inner line meets B's clearance circle (nearest A).
            ncx, ncy = centres[b]
            px, py = line[0] - ncx, line[1] - ncy
            qb = px * dx + py * dy
            qc = px * px + py * py - (node + clear) ** 2
            s_tip = -qb - math.sqrt(max(qb * qb - qc, 0.0))
            # Root: start well inside A's clearance so the ring carves a concave end.
            s_root = -(math.hypot(centres[a][0] - centres[b][0], centres[a][1] - centres[b][1]) / 2)
            width = BLADE * r
            root_in = (line[0] + dx * s_root, line[1] + dy * s_root)
            root_out = (root_in[0] + nx * width, root_in[1] + ny * width)
            tip = (line[0] + dx * s_tip, line[1] + dy * s_tip)
            blades.append([root_in, root_out, tip])
    return {"centres": centres, "node": node, "clear": node + clear,
            "outer": outer, "inner": inner, "blades": blades}


def fmt(points) -> str:
    return " ".join(f"{x:.2f},{y:.2f}" for x, y in points)


def mark(cx, cy, r, *, ids: str, nodes=(RED, GREEN, BLUE), bond=None,
         compact=False, glow=False) -> str:
    g = geometry(cx, cy, r, compact=compact)
    order = ("red", "blue", "green")
    o = [g["outer"][k] for k in order]
    i = [g["inner"][k] for k in order]
    holes = "".join(
        f'<circle cx="{x:.2f}" cy="{y:.2f}" r="{g["clear"]:.2f}" fill="#000"/>'
        for x, y in g["centres"].values())
    bond_fill = bond or f"url(#{ids}-silver)"
    shapes = [f'<path fill-rule="evenodd" d="M{fmt(o)}Z M{fmt(i)}Z"/>']
    shapes += [f'<polygon points="{fmt(p)}"/>' for p in g["blades"]]
    colours = dict(zip(("red", "green", "blue"), nodes))
    glow_attr = f' filter="url(#{ids}-glow)"' if glow else ""
    node_svg = "".join(
        f'<circle cx="{x:.2f}" cy="{y:.2f}" r="{g["node"]:.2f}" fill="{colours[k]}"{glow_attr}/>'
        for k, (x, y) in g["centres"].items())
    glow_def = (f'<filter id="{ids}-glow" x="-1" y="-1" width="3" height="3">'
                f'<feGaussianBlur stdDeviation="{r * 0.05:.2f}" result="b"/>'
                f'<feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>'
                f'</filter>') if glow else ""
    return (
        f'<defs><mask id="{ids}-clear" maskUnits="userSpaceOnUse">'
        f'<rect x="-10000" y="-10000" width="20000" height="20000" fill="#fff"/>{holes}</mask>'
        f'<linearGradient id="{ids}-silver" x1="0" y1="0" x2="1" y2="1">'
        f'<stop offset="0" stop-color="{SILVER_HI}"/><stop offset=".55" stop-color="{SILVER}"/>'
        f'<stop offset="1" stop-color="{SILVER_LO}"/></linearGradient>{glow_def}</defs>'
        f'<g fill="{bond_fill}" mask="url(#{ids}-clear)">{"".join(shapes)}</g>{node_svg}'
    )


FONT = OUT.parent / "fonts" / "Archivo[wdth,wght].ttf"
TRACE_SIZE = 800  # px per em while tracing; outlines are simplified to 1.2 px at this size


def _trace(text: str, weight: int, width: int = 125) -> tuple[list, float]:
    """Rasterise one run with PIL and trace pixel edges into simplified polygons.

    Returns (contours in px, advance in px), baseline at y=0. Used so the wordmark
    is shipped as outlines and never depends on an installed font.
    """
    import numpy as np
    from PIL import Image, ImageDraw, ImageFont

    font = ImageFont.truetype(str(FONT), TRACE_SIZE)
    font.set_variation_by_axes([weight, width])  # fvar order: Weight, Width
    x0, y0, x1, y1 = font.getbbox(text, anchor="ls")
    pad = 4
    img = Image.new("L", (x1 - x0 + 2 * pad, y1 - y0 + 2 * pad), 0)
    ImageDraw.Draw(img).text((pad - x0, pad - y0), text, font=font, fill=255, anchor="ls")
    on = np.pad(np.asarray(img) >= 128, 1)  # one-pixel background border
    edges: dict = {}
    ys, xs = np.nonzero(on)
    for y, x in zip(ys.tolist(), xs.tolist()):  # foreground kept on the right-hand side
        if not on[y - 1, x]:
            edges.setdefault((x, y), []).append((x + 1, y))
        if not on[y, x + 1]:
            edges.setdefault((x + 1, y), []).append((x + 1, y + 1))
        if not on[y + 1, x]:
            edges.setdefault((x + 1, y + 1), []).append((x, y + 1))
        if not on[y, x - 1]:
            edges.setdefault((x, y + 1), []).append((x, y))
    contours = []
    while edges:
        start = next(iter(edges))
        loop, here, prev = [start], start, None
        while True:
            outs = edges[here]
            if len(outs) > 1 and prev is not None:  # diagonal pixel pair: turn right
                dx, dy = here[0] - prev[0], here[1] - prev[1]
                outs.sort(key=lambda p: (p[0] - here[0]) * -dy + (p[1] - here[1]) * dx)
            nxt = outs.pop(0)
            if not outs:
                del edges[here]
            if nxt == start:
                break
            loop.append(nxt)
            prev, here = here, nxt
        contours.append(_simplify(loop, 1.2))
    shift_x, shift_y = x0 - pad - 1, y0 - pad - 1
    contours = [[(x + shift_x, y + shift_y) for x, y in c] for c in contours if len(c) > 2]
    return contours, font.getlength(text)


def _simplify(loop: list, eps: float) -> list:
    """Douglas-Peucker on a closed loop split at its two farthest-apart points."""
    def dp(pts):
        if len(pts) < 3:
            return pts
        (ax, ay), (bx, by) = pts[0], pts[-1]
        length = math.hypot(bx - ax, by - ay) or 1.0
        best, idx = 0.0, 0
        for i in range(1, len(pts) - 1):
            d = abs((bx - ax) * (ay - pts[i][1]) - (ax - pts[i][0]) * (by - ay)) / length
            if d > best:
                best, idx = d, i
        if best <= eps:
            return [pts[0], pts[-1]]
        return dp(pts[: idx + 1])[:-1] + dp(pts[idx:])
    half = len(loop) // 2
    return dp(loop[: half + 1])[:-1] + dp(loop[half:] + [loop[0]])[:-1]


def wordmark(x: float, baseline: float, size: float, fill: str, *, anchor="start") -> str:
    """'Baryon' semibold + 'OS' light, Archivo at 125 % width, as outlined paths."""
    bold, bold_adv = _trace("Baryon", 600)
    light, light_adv = _trace("OS", 300)
    s = size / TRACE_SIZE
    total = (bold_adv + light_adv) * s
    left = x - total / 2 if anchor == "middle" else x
    def path(contours, dx):
        d = " ".join("M" + " L".join(f"{(px + dx) * s + left:.2f},{py * s + baseline:.2f}"
                                    for px, py in c) + "Z" for c in contours)
        return f'<path d="{d}"/>'
    return f'<g fill="{fill}">{path(bold, 0)}{path(light, bold_adv)}</g>'


def svg(width, height, body, *, background=None, title="BaryonOS") -> str:
    bg = f'<rect width="{width}" height="{height}" fill="{background}"/>' if background else ""
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" '
            f'width="{width}" height="{height}" role="img" aria-label="{title}">'
            f'<title>{title}</title>{bg}{body}</svg>\n')


def centre(width: float, height: float, r: float, *, compact: bool = False) -> tuple[float, float]:
    """Optical centre: centre the bounding box of the nodes, not the triangle centroid."""
    g = geometry(0, 0, r, compact=compact)
    n = g["node"]
    xs = [x for x, _ in g["centres"].values()]
    ys = [y for _, y in g["centres"].values()]
    return (width - (min(xs) + max(xs))) / 2, (height - (min(ys) - n + max(ys) + n)) / 2


# Vector reconstruction of the original raster logo supplied by the project owner on
# 13 November 2026, traced from measured coordinates. Kept as a selectable option;
# replace with the source artwork when it is added to the repository.
ORIGINAL = (
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1932 1932" width="1000" height="1000" '
    'role="img" aria-label="BaryonOS original mark"><title>BaryonOS original mark</title>'
    '<rect width="1932" height="1932" fill="#0F1015"/><g fill="#D5D9DE">'
    '<path d="M853 545 L500 1342 Q425 1308 345 1305 Z"/>'
    '<path d="M1218 435 L1530 1050 L1100 553 Q1180 505 1218 435 Z"/>'
    '<path d="M630 1575 L1467 1355 Q1495 1405 1558 1437 Z"/>'
    '<path d="M928 562 L1010 572 Q1030 576 1048 596 L1478 1105 L1447 1160 L962 660 '
    'L668 1422 L1437 1190 L1437 1290 L608 1505 Q590 1438 556 1391 Z"/></g>'
    '<circle cx="1007" cy="343" r="167" fill="#FC4539"/>'
    '<circle cx="1648" cy="1238" r="151" fill="#087EFC"/>'
    '<circle cx="370" cy="1545" r="178" fill="#31BC5C"/></svg>\n')


def main() -> None:
    files = {}
    r = 330
    cx, cy = centre(1000, 1000, r)
    files["mark-primary-dark.svg"] = svg(1000, 1000, mark(cx, cy, r, ids="p"), background=SPACE)
    files["mark-primary.svg"] = svg(1000, 1000, mark(cx, cy, r, ids="t"))
    files["mark-light.svg"] = svg(
        1000, 1000, mark(cx, cy, r, ids="l", nodes=(LIGHT_RED, LIGHT_GREEN, LIGHT_BLUE), bond=INK),
        background="#F3F5F8")
    files["mark-mono-silver.svg"] = svg(
        1000, 1000, mark(cx, cy, r, ids="ms", nodes=(SILVER_HI,) * 3, bond=SILVER), background=SPACE)
    files["mark-mono-ink.svg"] = svg(
        1000, 1000, mark(cx, cy, r, ids="mi", nodes=(INK,) * 3, bond=INK))
    # Reduced size (<= 32 px): no blades, heavier bond, larger nodes and clearances.
    cr = 350
    ccx, ccy = centre(1000, 1000, cr, compact=True)
    files["mark-compact.svg"] = svg(1000, 1000, mark(ccx, ccy, cr, ids="c", compact=True))
    files["mark-compact-mono.svg"] = svg(
        1000, 1000, mark(ccx, ccy, cr, ids="cm", nodes=("#FFFFFF",) * 3, bond="#FFFFFF", compact=True))
    # App / launcher icon: deep-space tile with a quiet top light.
    tile = (
        '<defs><radialGradient id="tile-light" cx=".3" cy="0" r="1.1">'
        '<stop offset="0" stop-color="#26324A"/><stop offset=".55" stop-color="#111827"/>'
        '<stop offset="1" stop-color="#080A10"/></radialGradient>'
        '<linearGradient id="tile-rim" x1="0" y1="0" x2="0" y2="1">'
        '<stop offset="0" stop-color="#FFFFFF" stop-opacity=".22"/>'
        '<stop offset=".25" stop-color="#FFFFFF" stop-opacity="0"/></linearGradient></defs>'
        '<rect width="1024" height="1024" rx="232" fill="url(#tile-light)"/>'
        '<rect x="2" y="2" width="1020" height="1020" rx="230" fill="none" '
        'stroke="url(#tile-rim)" stroke-width="4"/>')
    ir = 285
    icx, icy = centre(1024, 1024, ir)
    files["app-icon.svg"] = svg(1024, 1024, tile + mark(icx, icy, ir, ids="a"),
                                title="BaryonOS app icon")
    # Boot / splash: emissive nodes and the wordmark on deep space.
    sr = 150
    scx, scy = centre(1920, 760, sr)
    splash = (
        '<defs><radialGradient id="sp" cx=".5" cy=".42" r=".7">'
        '<stop offset="0" stop-color="#121A2C"/><stop offset=".6" stop-color="#090B12"/>'
        '<stop offset="1" stop-color="#05060A"/></radialGradient></defs>'
        '<rect width="1920" height="1080" fill="url(#sp)"/>'
        + mark(scx, scy + 40, sr, ids="s", glow=True)
        + wordmark(960, 800, 52, "#E9EDF2", anchor="middle"))
    files["splash.svg"] = svg(1920, 1080, splash, title="BaryonOS boot splash")
    # Horizontal lockups: mark at cap-height scale, wordmark set on its baseline.
    lr = 92
    lcx, lcy = centre(300, 300, lr)
    files["lockup-dark.svg"] = svg(
        1200, 300, mark(lcx, lcy, lr, ids="ld") + wordmark(300, 186, 96, "#E9EDF2"),
        background=SPACE, title="BaryonOS lockup")
    files["lockup-light.svg"] = svg(
        1200, 300, mark(lcx, lcy, lr, ids="ll", nodes=(LIGHT_RED, LIGHT_GREEN, LIGHT_BLUE), bond=INK)
        + wordmark(300, 186, 96, INK), background="#F3F5F8", title="BaryonOS lockup")
    files["mark-original.svg"] = ORIGINAL
    for name, text in files.items():
        (OUT / name).write_text(text, encoding="utf-8")
    print("\n".join(sorted(files)))


if __name__ == "__main__":
    main()
