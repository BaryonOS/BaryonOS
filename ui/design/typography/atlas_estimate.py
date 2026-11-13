#!/usr/bin/env python3
"""Estimate native atlas glyph-data bytes for a candidate typeface.

Uses the run-length encoding of ui/graphics/scripts/generate_font.py (rows of
(run length - 1) << 4 | 4-bit coverage, runs up to 16) for three record sets and
three character sets. Glyph and row tables are not included. Glyphs larger than
the generator's 96 px bound are skipped and counted.

Usage: python3 -I ui/design/typography/atlas_estimate.py FONT.ttf [feature,feature]
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
LATIN1 = [chr(n) for n in range(32, 127)] + [chr(n) for n in range(160, 256)] + ['…', '€', '�']
EXT_A = [chr(n) for n in range(0x100, 0x180)] + list('ȘșȚț')
PUNCT = list('‘’‚“”„–—•′″‹›←↑→↓−≤≥≠  ™')
GREEK = [chr(n) for n in range(0x386, 0x3cf) if n not in (0x387, 0x38b, 0x38d, 0x3a2)]
CYR = [chr(n) for n in range(0x410, 0x450)] + ['Ѐ', 'Ѝ', 'ѐ', 'ѝ']
SETS = {'latin1 (today)': LATIN1, 'EU Latin': LATIN1 + EXT_A + PUNCT, 'EU + Greek + Cyrillic': LATIN1 + EXT_A + PUNCT + GREEK + CYR}
def font_at(path, size, weight, opsz):
    f = ImageFont.truetype(str(path), size)
    try: axes = f.get_variation_axes()
    except OSError: return f
    vals = []
    for a in axes:
        n = a.get('name', b''); n = n.decode() if isinstance(n, bytes) else str(n)
        v = weight if n.lower().startswith('weight') else opsz if n.lower().startswith('optical') else a['default']
        vals.append(max(a['minimum'], min(a['maximum'], v)))
    f.set_variation_by_axes(vals); return f
def encode(path, specs, chars, feats):
    total = 0; missing = 0; too_big = []
    for weight, size in specs:
        f = font_at(path, size, weight, min(32, max(14, size)))
        for ch in chars:
            fe = feats + (['tnum'] if ch.isdigit() else [])
            l, t, r, b = f.getbbox(ch, anchor='ls', features=fe); w, h = r - l, b - t
            if w > 96 or h > 96: too_big.append((size, ch)); continue
            if not w or not h: continue
            m = Image.new('L', (w, h), 0); ImageDraw.Draw(m).text((-l, -t), ch, font=f, fill=255, anchor='ls', features=fe)
            px = m.load()
            for y in range(h):
                x = 0
                while x < w:
                    a = min(15, (px[x, y] + 8) // 17); run = 1
                    while x + run < w and run < 16 and min(15, (px[x + run, y] + 8) // 17) == a: run += 1
                    total += 1; x += run
    return total, too_big
path = Path(sys.argv[1]); feats = sys.argv[2].split(',') if len(sys.argv) > 2 and sys.argv[2] else []
TODAY = [(400, 12), (400, 16), (400, 24), (400, 32), (600, 16), (600, 32), (600, 64)]
FULL = [(400, s) for s in (12, 13, 15, 16, 24, 26, 30, 32)] + [(600, s) for s in (13, 15, 18, 22, 28, 26, 30, 36, 44, 56)] + [(400, 48), (400, 96)]
REDUCED = [(400, 12), (400, 15), (600, 15), (600, 22), (400, 48), (400, 24), (400, 30), (600, 30), (600, 44)]
for name, specs in (('today records', TODAY), ('full scale', FULL), ('reduced set', REDUCED)):
    for cname, chars in SETS.items():
        n, big = encode(path, specs, chars, feats)
        print(f"{path.stem.split('[')[0]:28} {name:15} {cname:24} {len(chars):4} chars {len(specs):2} records  {n/1024:7.1f} KiB" + (f"  >96px: {len(big)}" if big else ""))
