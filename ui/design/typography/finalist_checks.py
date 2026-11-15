#!/usr/bin/env python3
"""Tie-break checks for the three typography finalists (TYPOGRAPHY.md section 7).

Reports, per face: characters of the prototype copy missing from the cmap, glyph
names of zero variants and rasters of the default and tnum zero, I/l rasters
at the UI and small token sizes with the shipping features, pair distinctness
at the token sizes (two-size mean, then per size), and word-space gaps at 100 %
and 200 %, also with the space advance 2 px wider (a proposed atlas
correction): first each role on its own specimen line, then every role record
(and Inter's 22sb, 44sb and 48r) on all four specimen lines, with the sites of
ties and collapses. All through the same Pillow/raqm path and whole-glyph
advance placement native_specimen.py uses. Build-time only.
Usage: python3 -I ui/design/typography/finalist_checks.py
"""
import struct
import sys
from pathlib import Path

from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).parent))
import measure_fonts as mf  # noqa: E402
import native_specimen as ns  # noqa: E402

ROOT = Path(__file__).resolve().parents[3]
C = ROOT / 'ui/design/typography/candidates'
FACES = {  # name: (file, UI px at equal x-height, shipping features)
    'Inter': ('Inter[opsz,wght].ttf', 15, ['cv05', 'cv08', 'ss01']),
    'Atkinson Hyperlegible Next': ('AtkinsonHyperlegibleNext[wght].ttf', 16, []),
    'Exo 2': ('Exo2[wght].ttf', 17, []),
}
COPY = ['ui/design/prototype/strings.js', 'ui/design/prototype/data.js']
# type-1.0 records outside the specimen roles (TYPOGRAPHY.md section 8.2): 22sb, 44sb, 48r.
EXTRA_RECORDS = {'Inter': [(600, 22), (600, 44), (400, 48)]}


def post_names(data: bytes, t: dict) -> list:
    off, length = t['post']
    if struct.unpack('>I', data[off:off + 4])[0] != 0x20000:
        return []
    n = struct.unpack('>H', data[off + 32:off + 34])[0]
    idx = struct.unpack(f'>{n}H', data[off + 34:off + 34 + 2 * n])
    p, names = off + 34 + 2 * n, []
    while p < off + length:
        size = data[p]
        names.append(data[p + 1:p + 1 + size].decode('latin-1'))
        p += 1 + size
    return [names[i - 258] for i in idx if i >= 258]


def raster(font, ch: str, feats: list) -> list:
    left, top, right, bottom = font.getbbox(ch, anchor='ls', features=feats)
    im = Image.new('L', (right - left + 2, bottom - top + 2), 0)
    ImageDraw.Draw(im).text((1 - left, 1 - top), ch, font=font, fill=255, anchor='ls', features=feats)
    return [''.join(' .:-=+*#%@'[min(9, im.getpixel((x, y)) // 26)] for x in range(im.width)) for y in range(im.height)]


def gaps(text: str, font, feats: list, space_px: int = 0) -> tuple:
    """Smallest ink gap across a space and largest ink gap inside a word, in pixels.
    space_px widens the space advance, simulating a corrected atlas record."""
    g = gap_list(text, font, feats, space_px)
    return min(d for d, _, s in g if s), max(d for d, _, s in g if not s)


def gap_list(text: str, font, feats: list, space_px: int = 0) -> list:
    """(gap px, characters either side, crosses a space) for each pair of inked neighbours."""
    advance, spans = 0, []
    for ch in text:
        f = feats + (ns.DIGIT_FEATURES if ch.isdigit() else [])
        left, top, right, bottom = font.getbbox(ch, anchor='ls', features=f)  # advance box, not ink
        ink = None
        if ch != ' ' and right > left and bottom > top:
            mask = Image.new('L', (right - left, bottom - top), 0)
            ImageDraw.Draw(mask).text((-left, -top), ch, font=font, fill=255, anchor='ls', features=f)
            ink = mask.point(lambda v: 255 if (v + 8) // 17 else 0).getbbox()  # pixels the 4-bit atlas keeps
        spans.append(None if ink is None else (advance // 64 + left + ink[0], advance // 64 + left + ink[2]))
        advance += round(font.getlength(ch, features=f) * 64) + (space_px * 64 if ch == ' ' else 0)
    out, prev, start, crossed = [], None, 0, False
    for i, span in enumerate(spans):
        if span is None:
            crossed = True
            continue
        if prev is not None:
            out.append((span[0] - prev[1], text[start:i + 1], crossed))
        prev, start, crossed = span, i, False
    return out


def worst(text: str, font, feats: list, space_px: int = 0) -> tuple:
    """((smallest gap across a space, where), (largest gap inside a word, where))."""
    g = gap_list(text, font, feats, space_px)
    sp = min(((d, w) for d, w, s in g if s), key=lambda dw: dw[0])
    wd = max(((d, w) for d, w, s in g if not s), key=lambda dw: dw[0])
    return sp, wd


def main() -> None:
    copy = ''.join((ROOT / p).read_text() for p in COPY)
    chars = sorted({c for c in copy if ord(c) > 126})
    print('Non-ASCII characters in the prototype copy:', ''.join(chars))
    for name, (file, px, feats) in FACES.items():
        path = C / file
        data = path.read_bytes()
        t = mf.sfnt_tables(data)
        cps = mf.cmap_codepoints(data, t)
        missing = ''.join(c for c in chars if ord(c) not in cps)
        zeros = [g for g in post_names(data, t) if g.startswith('zero')]
        print(f'\n{name} ({px} px, features {feats or "none"})')
        print('  copy characters missing:', missing or 'none')
        print('  zero glyphs:', zeros or 'post table has no names')
        sizes = ns.roles(path.stem.split('[')[0])
        assert sizes['ui'][1] == px, (name, sizes)
        for scale in (1, 2):
            for role, text in ns.LINES:
                weight, size = sizes[role]
                font = ns.font_at(path, size * scale, weight)
                (sp, wd), (sp2, _) = gaps(text, font, feats), gaps(text, font, feats, 2)
                print(f'  {role:6} {weight} {size * scale:2d} px: smallest word-space gap {sp} px '
                      f'({sp2} px with the space 2 px wider), largest gap inside a word {wd} px')
        font = ns.font_at(path, sizes['ui'][1], 400)
        zero, zero_tf = raster(font, '0', feats), raster(font, '0', feats + ['tnum'])
        print(f'  0 at {sizes["ui"][1]} px, default glyph | tnum glyph:')
        for ra, rb in zip(zero, zero_tf):
            print(f'    {ra}   {rb}')
        axes = mf.fvar_axes(data, t)
        token_px = (sizes['small'][1], sizes['ui'][1])  # equal x-height, unlike section 4's 11-16 px
        d = {f'{a}/{b}': sum(mf.distinctness(path, axes, a, b, s, feats) for s in token_px) / 2 for a, b in mf.PAIRS}
        print(f'  pair distinctness at {token_px} px: mean {sum(d.values()) / len(d):.3f}, weakest '
              + ', '.join(f'{k} {v:.3f}' for k, v in sorted(d.items(), key=lambda kv: kv[1])[:3]))
        for role in ('ui', 'small'):
            weight, size = sizes[role]
            font = ns.font_at(path, size, weight)
            a, b = raster(font, 'I', feats), raster(font, 'l', feats)
            h = max(len(a), len(b))
            a += [' ' * len(a[0])] * (h - len(a))
            b += [' ' * len(b[0])] * (h - len(b))
            print(f'  I | l at {size} px ({role}), 4-bit coverage as characters:')
            for ra, rb in zip(a, b):
                print(f'    {ra}   {rb}')
        for s in token_px:  # per size: the two-size mean above hides swings between sizes
            ds = {f'{a}/{b}': mf.distinctness(path, axes, a, b, s, feats) for a, b in mf.PAIRS}
            print(f'  pair distinctness at {s} px only: mean {sum(ds.values()) / len(ds):.3f}, weakest '
                  + ', '.join(f'{k} {v:.3f}' for k, v in sorted(ds.items(), key=lambda kv: kv[1])[:3]))
        records = [(w, s * scale) for scale in (1, 2) for w, s in (sizes[r] for r, _ in ns.LINES)]
        records += EXTRA_RECORDS.get(name, [])
        print('  word space on every specimen line, per record: smallest gap across a space / largest gap '
              'inside a word (px); "=" tie, "<" collapse; then the same with the space 2 px wider')
        flagged = {0: [], 2: []}
        for weight, size in records:
            font = ns.font_at(path, size, weight)
            cells = {0: [], 2: []}
            for role, text in ns.LINES:
                for extra in (0, 2):
                    (sp, sw), (wd, ww) = worst(text, font, feats, extra)
                    mark = '<' if sp < wd else '=' if sp == wd else ' '
                    cells[extra].append(f'{role} {sp}/{wd}{mark}')
                    if mark != ' ':
                        flagged[extra].append(f'{size}{"sb" if weight == 600 else "r"} on the {role} line: '
                                              f'{sp} px at "{sw}" against {wd} px at "{ww}"')
            print(f'    {size:2d}{"sb" if weight == 600 else "r ":2}  ' + '  '.join(cells[0])
                  + '  | +2 px: ' + '  '.join(cells[2]))
        for extra in (0, 2):
            print(f'  ties and collapses{" with the space 2 px wider" if extra else ""}:',
                  ('\n    ' + '\n    '.join(flagged[extra])) if flagged[extra] else 'none')


if __name__ == '__main__':
    main()
