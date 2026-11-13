#!/usr/bin/env python3
"""Render candidate typefaces the way the native BaryonOS atlas would.

Mirrors ui/graphics/scripts/generate_font.py and ui/graphics/src/text.rs:
per-glyph Pillow/FreeType rasters, 4-bit coverage (16 levels), 26.6 advances
accumulated and floored per glyph, no kerning, no shaping, no OpenType
shaping. OpenType alternates in FEATURES are applied per glyph, the way a
build-time atlas would bake a 1:1 substitution. Sizes are the proposed type
tokens (docs/design/TYPOGRAPHY.md); 200 % doubles them. This is a build-time
simulation of the native output, not a QEMU capture.

Usage: python3 -I ui/design/typography/native_specimen.py [--blind] OUT_DIR FONT_DIR [FONT_DIR...]
--blind labels rows with neutral codes and prints the key, for unbiased review.
"""
import hashlib
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

THEMES = {"dark": ((11, 14, 20), (232, 236, 242), (155, 165, 180)),
          "light": ((255, 255, 255), (20, 26, 35), (86, 96, 111))}
LINES = [  # (role, text); Latin-1 plus … and €, the native character set
    ("ui", "Bestanden   Notities   Instellingen   Op dit apparaat   09:41 vr 13 nov"),
    ("read", "Thuis-NAS is niet bereikbaar. Je werk staat veilig op dit apparaat."),
    ("strong", "Webvenster wil Tuinplan versturen naar tuinplanner.example"),
    ("small", "Il1| O0o rn m cl d 6 8 5 S B 8  € 1.234,56  13-11-2026 09:41:07  Größe Ålesund"),
]
# role -> (weight, size at 100 %); 200 % doubles the size, as the native atlas does.
ROLES = {"ui": (400, 15), "read": (400, 16), "strong": (600, 15), "small": (400, 13)}
# Alternates baked into the atlas per family (font file stem prefix). Digits
# additionally get tnum so clocks and columns do not jitter; only digits, because
# Inter's tnum also makes the hyphen tabular ("Thuis -NAS").
FEATURES = {"Inter": ["cv05", "cv08", "ss01"]}
DIGIT_FEATURES = ["tnum"]


def font_at(path: Path, size: int, weight: int):
    f = ImageFont.truetype(str(path), size)
    try:
        axes = f.get_variation_axes()
    except OSError:
        return f
    vals = []
    for a in axes:
        name = a.get("name", b"")
        name = name.decode() if isinstance(name, bytes) else str(name)
        if name.lower().startswith("weight"):
            vals.append(max(a["minimum"], min(a["maximum"], weight)))
        elif name.lower().startswith("optical"):
            vals.append(max(a["minimum"], min(a["maximum"], size)))
        elif name.lower().startswith("width"):
            vals.append(100)
        else:
            vals.append(a["default"])
    f.set_variation_by_axes(vals)
    return f


def draw_native(img: Image.Image, x: int, y: int, text: str, font, colour, features=None) -> None:
    ascent, _ = font.getmetrics()
    advance = 0  # 26.6 fixed point, as in text.rs
    px = img.load()
    for ch in text:
        feats = (features or []) + (DIGIT_FEATURES if ch.isdigit() else [])
        left, top, right, bottom = font.getbbox(ch, anchor="ls", features=feats)
        w, h = right - left, bottom - top
        if w > 0 and h > 0:
            mask = Image.new("L", (w, h), 0)
            ImageDraw.Draw(mask).text((-left, -top), ch, font=font, fill=255, anchor="ls", features=feats)
            gx, gy = x + advance // 64 + left, y + ascent + top
            for yy in range(h):
                for xx in range(w):
                    level = min(15, (mask.getpixel((xx, yy)) + 8) // 17)
                    if level and 0 <= gx + xx < img.width and 0 <= gy + yy < img.height:
                        a = level * 17
                        bg = px[gx + xx, gy + yy]
                        px[gx + xx, gy + yy] = tuple((c * a + b * (255 - a)) // 255 for c, b in zip(colour, bg))
        advance += round(font.getlength(ch, features=feats) * 64)


def sheet(fonts: list, theme: str, scale: int, codes=None) -> Image.Image:
    bg, fg, muted = THEMES[theme]
    label_font = ImageFont.load_default(12 * scale)
    row_h = 112 * scale
    img = Image.new("RGB", (980 * scale, 16 * scale + row_h * len(fonts)), bg)
    d = ImageDraw.Draw(img)
    for i, path in enumerate(fonts):
        y0 = 8 * scale + i * row_h
        family = path.stem.split("[")[0]
        feats = next((v for k, v in FEATURES.items() if family.startswith(k)), None)
        label = f"Specimen {codes[i]}" if codes else family + (f"  ({' '.join(feats)})" if feats else "")
        d.text((12 * scale, y0), label, font=label_font, fill=muted)
        y = y0 + 18 * scale
        for role, text in LINES:
            weight, size = ROLES[role]
            f = font_at(path, size * scale, weight)
            draw_native(img, 12 * scale, y, text, f, muted if role == "small" else fg, feats)
            y += (size + 8) * scale
    return img


def main() -> None:
    args = sys.argv[1:]
    blind = args[:1] == ["--blind"]
    out, dirs = Path(args[blind]), args[blind + 1:]
    fonts = sorted({p.name: p for d in dirs for p in Path(d).glob("*.ttf")}.values(), key=lambda p: p.name)
    codes = None
    if blind:  # neutral, order-independent codes: rows are shuffled by a hash of the file name
        fonts.sort(key=lambda p: hashlib.sha256(p.name.encode()).hexdigest())
        codes = [str(i + 1) for i in range(len(fonts))]  # digits: browser specimens use letters
        for c, p in zip(codes, fonts):
            print(f"key {c} = {p.stem}")
    out.mkdir(parents=True, exist_ok=True)
    stem = "blind" if blind else "native"
    for theme in THEMES:
        for scale in (1, 2):
            sheet(fonts, theme, scale, codes).save(out / f"{stem}-{theme}-{scale}x.png", optimize=True)
            print(out / f"{stem}-{theme}-{scale}x.png")


if __name__ == "__main__":
    main()
