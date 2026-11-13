#!/usr/bin/env python3
"""Render candidate typefaces the way the native BaryonOS atlas would.

Mirrors ui/graphics/scripts/generate_font.py and ui/graphics/src/text.rs:
per-glyph Pillow/FreeType rasters, 4-bit coverage (16 levels), 26.6 advances
accumulated and floored per glyph, no kerning, no shaping, no OpenType
features. Sizes are the native atlas sizes (Regular 12/16, Semibold 16 at
100 %; Regular 24/32, Semibold 32 at 200 %). This is a build-time simulation of
the native output, not a QEMU capture.

Usage: python3 -I ui/design/typography/native_specimen.py OUT_DIR FONT_DIR [FONT_DIR...]
"""
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
ROLES = {"ui": (400, 16), "read": (400, 16), "strong": (600, 16), "small": (400, 12)}


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


def draw_native(img: Image.Image, x: int, y: int, text: str, font, colour) -> None:
    ascent, _ = font.getmetrics()
    advance = 0  # 26.6 fixed point, as in text.rs
    px = img.load()
    for ch in text:
        left, top, right, bottom = font.getbbox(ch, anchor="ls")
        w, h = right - left, bottom - top
        if w > 0 and h > 0:
            mask = Image.new("L", (w, h), 0)
            ImageDraw.Draw(mask).text((-left, -top), ch, font=font, fill=255, anchor="ls")
            gx, gy = x + advance // 64 + left, y + ascent + top
            for yy in range(h):
                for xx in range(w):
                    level = min(15, (mask.getpixel((xx, yy)) + 8) // 17)
                    if level and 0 <= gx + xx < img.width and 0 <= gy + yy < img.height:
                        a = level * 17
                        bg = px[gx + xx, gy + yy]
                        px[gx + xx, gy + yy] = tuple((c * a + b * (255 - a)) // 255 for c, b in zip(colour, bg))
        advance += round(font.getlength(ch) * 64)


def sheet(fonts: list, theme: str, scale: int) -> Image.Image:
    bg, fg, muted = THEMES[theme]
    label_font = ImageFont.load_default(12 * scale)
    row_h = 112 * scale
    img = Image.new("RGB", (980 * scale, 16 * scale + row_h * len(fonts)), bg)
    d = ImageDraw.Draw(img)
    for i, path in enumerate(fonts):
        y0 = 8 * scale + i * row_h
        d.text((12 * scale, y0), path.stem.split("[")[0], font=label_font, fill=muted)
        y = y0 + 18 * scale
        for role, text in LINES:
            weight, size = ROLES[role]
            f = font_at(path, size * scale, weight)
            draw_native(img, 12 * scale, y, text, f, muted if role == "small" else fg)
            y += (size + 8) * scale
    return img


def main() -> None:
    out = Path(sys.argv[1])
    fonts = sorted({p.name: p for d in sys.argv[2:] for p in Path(d).glob("*.ttf")}.values(), key=lambda p: p.name)
    out.mkdir(parents=True, exist_ok=True)
    for theme in THEMES:
        for scale in (1, 2):
            sheet(fonts, theme, scale).save(out / f"native-{theme}-{scale}x.png", optimize=True)
            print(out / f"native-{theme}-{scale}x.png")


if __name__ == "__main__":
    main()
