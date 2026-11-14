#!/usr/bin/env python3
"""Render logotype candidates beside the BaryonOS mark for a blind comparison.

Each candidate is set as a lockup at 96, 40, 20 and 14 px, in an "Over BaryonOS"
panel next to 15 px Inter UI text, and as a boot line, in dark and light, on the
surface and text tokens of the selected Orbitaal concept. The mark is drawn
flat from generate_mark.geometry() (4x supersampled); text uses Pillow/raqm, with
pair kerning kept and tracking added between glyphs. Build-time study tool for
docs/design/BRAND.md section 4 and docs/verification/logotype-review-20261114/;
nothing here ships.

Usage: python3 -I ui/design/brand/logotype_study.py OUT_DIR [--blind]
--blind names the sheets by neutral letter codes and prints the key.
"""
import hashlib
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import generate_mark as gm  # noqa: E402

FONTS = HERE.parent / "fonts"
CAND = HERE.parent / "typography" / "candidates"
INTER = FONTS / "Inter[opsz,wght].ttf"
# id: (font file, [(text, weight, second colour?)], tracking in em, width axis)
VARIANTS = {
    "inter-twotone": (INTER, [("Baryon", 600, False), ("OS", 600, True)], -0.01, 100),
    "inter-weight": (INTER, [("Baryon", 600, False), ("OS", 300, False)], -0.01, 100),
    "inter-heavy-wide": (INTER, [("Baryon", 700, False), ("OS", 700, True)], 0.03, 100),
    "exo2-twotone": (CAND / "Exo2[wght].ttf", [("Baryon", 600, False), ("OS", 600, True)], 0.0, 100),
    "saira-wide-twotone": (CAND / "Saira[wdth,wght].ttf", [("Baryon", 600, False), ("OS", 600, True)], 0.0, 125),
    "oxanium-twotone": (CAND / "Oxanium[wght].ttf", [("Baryon", 600, False), ("OS", 600, True)], 0.0, 100),
    "archivo-v1": (FONTS / "Archivo[wdth,wght].ttf", [("Baryon", 600, False), ("OS", 300, False)], 0.0, 125),
}
THEMES = {  # Orbitaal v1 tokens (ui/design/concepts/orbitaal/index.html .dk/.lt): s0, tx1, tx2, tx1, tx2, nodes, bond, s2
    "dark": ("#070912", "#EEF1F8", "#A9B2CB", "#EEF1F8", "#A9B2CB",
             (gm.RED, gm.GREEN, gm.BLUE), gm.SILVER, "#12162A"),
    "light": ("#E4E9F2", "#111726", "#424B62", "#111726", "#424B62",
              (gm.LIGHT_RED, gm.LIGHT_GREEN, gm.LIGHT_BLUE), gm.INK, "#FBFCFE"),
}
UI_FEATURES = ["cv05", "cv08", "ss01"]


def font(path: Path, size: int, weight: int, width: int = 100) -> ImageFont.FreeTypeFont:
    f = ImageFont.truetype(str(path), size)
    want = {"weight": weight, "optical": size, "width": width}
    vals = []
    for a in f.get_variation_axes():
        name = a["name"].decode() if isinstance(a["name"], bytes) else str(a["name"])
        vals.append(max(a["minimum"], min(a["maximum"], want.get(name.lower().split()[0], a["default"]))))
    f.set_variation_by_axes(vals)
    return f


def draw_mark(img: Image.Image, cx: float, cy: float, r: float, theme: str, bg: str = "") -> None:
    """Flat mark (bond, blades, clearance, nodes), 4x supersampled, pasted centred on (cx, cy)."""
    bg, nodes, bond = bg or THEMES[theme][0], THEMES[theme][5], THEMES[theme][6]
    k, half = 4, int(r * 1.35) + 2
    patch = Image.new("RGB", (2 * half * k,) * 2, bg)
    d = ImageDraw.Draw(patch)
    g = gm.geometry(half * k, half * k, r * k)
    order = ("red", "blue", "green")
    d.polygon([g["outer"][n] for n in order], fill=bond)
    d.polygon([g["inner"][n] for n in order], fill=bg)
    for blade in g["blades"]:
        d.polygon(blade, fill=bond)
    colours = dict(zip(("red", "green", "blue"), nodes))  # same mapping as generate_mark.mark()
    for name, (x, y) in g["centres"].items():
        d.ellipse((x - g["clear"], y - g["clear"], x + g["clear"], y + g["clear"]), fill=bg)
        d.ellipse((x - g["node"], y - g["node"], x + g["node"], y + g["node"]), fill=colours[name])
    img.paste(patch.resize((2 * half,) * 2, Image.LANCZOS), (round(cx - half), round(cy - half)))


def wordmark(d: ImageDraw.ImageDraw, x: float, baseline: float, size: int, variant: str, theme: str) -> float:
    """Draw the wordmark glyph by glyph (kerning kept, tracking added); return its advance width."""
    path, runs, track, width = VARIANTS[variant]
    main, second = THEMES[theme][1:3]
    start = x
    for i, (text, weight, two_tone) in enumerate(runs):
        f = font(path, size, weight, width)
        x += f.getlength(" ") + track * size if i else 0  # a word space, as in the original wordmark
        for j, ch in enumerate(text):
            # Pair kerning moves the left glyph's advance, so place glyph j at the end of text[:j+1] minus its own advance.
            gx = x + f.getlength(text[:j + 1]) - f.getlength(ch) + j * track * size
            d.text((gx, baseline), ch, font=f, fill=second if two_tone else main, anchor="ls")
        x += f.getlength(text) + (len(text) - 1) * track * size
    return x - start


def lockup(img, d, x, y_mid, size, variant, theme, bg="") -> None:
    """Mark at the v1 lockup proportion (r = 0.96 x size); wordmark cap height centred on the mark."""
    draw_mark(img, x + 1.6 * size, y_mid, 0.96 * size, theme, bg)
    path, runs, _, width = VARIANTS[variant]
    _, top, _, bottom = font(path, size, runs[0][1], width).getbbox("H", anchor="ls")
    wordmark(d, x + 3.3 * size, y_mid - (top + bottom) / 2, size, variant, theme)


def sheet(variant: str, theme: str) -> Image.Image:
    bg, _, _, ui, muted, _, _, panel = THEMES[theme]
    img = Image.new("RGB", (1400, 900), bg)
    d = ImageDraw.Draw(img)
    y = 150
    for size, step in ((96, 210), (40, 110), (20, 70), (14, 90)):
        lockup(img, d, 40, y, size, variant, theme)
        y += step
    # "Over BaryonOS" panel: lockup at 28 px next to 15 px Inter UI text (fictional data).
    d.rounded_rectangle((40, y, 760, y + 230), radius=14, fill=panel)
    lockup(img, d, 64, y + 52, 28, variant, theme, panel)
    body, strong = font(INTER, 15, 400), font(INTER, 15, 600)
    d.text((72, y + 120), "Versie 0.9.3 · bijgewerkt op 13 november 2026", font=body, fill=ui, features=UI_FEATURES)
    d.text((72, y + 146), "Dit apparaat · Thuis-NAS verbonden", font=body, fill=muted, features=UI_FEATURES)
    d.rounded_rectangle((72, y + 176, 236, y + 210), radius=8, outline=muted)
    d.text((88, y + 199), "Updates zoeken", font=strong, fill=ui, features=UI_FEATURES, anchor="ls")
    # Boot line: mark above, wordmark centred below, as in splash.svg.
    bx, by = 1080, y + 70
    draw_mark(img, bx, by, 60, theme)
    width = wordmark(ImageDraw.Draw(Image.new("RGB", (1, 1))), 0, 0, 34, variant, theme)
    wordmark(d, bx - width / 2, by + 120, 34, variant, theme)
    return img


def main() -> None:
    args = sys.argv[1:]
    blind = "--blind" in args
    out = Path(next(a for a in args if a != "--blind"))
    out.mkdir(parents=True, exist_ok=True)
    ids = sorted(VARIANTS, key=lambda v: hashlib.sha256(v.encode()).hexdigest())
    assert len(ids) <= 7
    for i, variant in enumerate(ids):
        code = "LMNPQRS"[i] if blind else variant
        if blind:
            print(f"key {code} = {variant}")
        for theme in THEMES:
            sheet(variant, theme).save(out / f"{code}-{theme}.png", optimize=True)
    print(out)


if __name__ == "__main__":
    main()
