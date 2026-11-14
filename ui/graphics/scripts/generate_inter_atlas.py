#!/usr/bin/env python3
"""Offline Inter 4.001 atlas for the `inter-atlas` cargo feature.

Run: python3 -I ui/graphics/scripts/generate_inter_atlas.py
Reads only the vendored ui/design/fonts/Inter[opsz,wght].ttf (SHA-256 checked
against ui/design/fonts/FONTS.md) and writes src/atlas_inter.rs,
assets/inter-atlas.rle and assets/inter-atlas-provenance.json. It never touches
atlas.rs or the Open Sans assets. Same RLE encoding, 96x96 glyph bound and table
layout as generate_font.py; native code never parses a font file.

Alternates (cv05 cv08 ss01) and tabular digits are baked per character, a 1:1
substitution: native text draws no shaping, kerning or contextual forms.
"""
from pathlib import Path
import ctypes
import ctypes.util
import hashlib
import json
from PIL import Image, ImageDraw, ImageFont, features, __version__ as PILLOW_VERSION

ROOT = Path(__file__).resolve().parents[1]
REPO = ROOT.parents[1]
FONT = 'ui/design/fonts/Inter[opsz,wght].ttf'
FONT_SHA256 = '29160a80ff49ddcab2c97711247e08b1fab27a484a329ce8b813d820dc559031'
LICENSE = 'ui/graphics/licenses/OFL-Inter.txt'
LICENSE_SHA256 = '5b9321a4298cfeb6b34354164a1c3afc3db114569984c502b9b35d988fd58c57'
EXTRA = [0x2013, 0x2014, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2026, 0x20AC, 0x203A, 0x2191, 0x2193, 0xFFFD]
# Sorted for the binary search in text.rs; U+FFFD is the largest and the fallback.
CHARS = sorted([chr(n) for n in range(32, 127)] + [chr(n) for n in range(160, 256)] + [chr(n) for n in EXTRA])
FEATURES = ['cv05', 'cv08', 'ss01']
DIGITS = '0123456789'  # not str.isdigit(): that also matches superscripts in Latin-1
BLANK = {' ', ' ', '­'}
# (weight, px, line height, extra space px) in FONTS order, which text.rs indexes:
# 13r 15r 15sb 22sb 48r at 100 %, then 26r 30r 30sb 44sb at 200 % (display is 48r magnified).
# Line heights are the type-1.0 roles (TYPOGRAPHY.md 8.2), doubled at 200 %.
# Space widening is the section 6.4/8.2 word-space correction for 600 at body size.
SPECS = [(400, 13, 18, 0), (400, 15, 22, 0), (600, 15, 22, 2), (600, 22, 28, 0), (400, 48, 56, 0),
         (400, 26, 36, 0), (400, 30, 44, 0), (600, 30, 44, 4), (600, 44, 56, 0)]
ENCODING = 'Rows of (run length minus1) in high nibble and 4bit coverage in low nibble; 16 levels'


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_sources():
    assert features.check('raqm'), 'Pillow without raqm cannot apply OpenType features'
    record = [line for line in (REPO / 'ui/design/fonts/FONTS.md').read_text().splitlines()
              if line.startswith('| `Inter[opsz,wght].ttf`')]
    assert len(record) == 1 and f'`{FONT_SHA256}`' in record[0], 'FONTS.md Inter row does not record the pinned hash'
    assert sha256(REPO / FONT) == FONT_SHA256, 'vendored Inter differs from FONTS.md'
    assert sha256(REPO / LICENSE) == LICENSE_SHA256, 'licence copy differs from ui/design/fonts/OFL-Inter.txt'


def font_at(px, weight):
    f = ImageFont.truetype(str(REPO / FONT), px, layout_engine=ImageFont.Layout.RAQM)
    values = []
    for axis in f.get_variation_axes():
        want = {b'Optical size': px, b'Weight': weight}[axis['name']]
        values.append(max(axis['minimum'], min(axis['maximum'], want)))
    f.set_variation_by_axes(values)
    return f, values


def feats(ch):
    return FEATURES + (['tnum'] if ch in DIGITS else [])


def raster(f, ch, fe):
    left, top, right, bottom = f.getbbox(ch, anchor='ls', features=fe)
    w, h = right - left, bottom - top
    mask = Image.new('L', (max(1, w), max(1, h)), 0)
    if w and h:
        ImageDraw.Draw(mask).text((-left, -top), ch, font=f, fill=255, anchor='ls', features=fe)
    return left, top, w, h, mask


def harfbuzz_version():
    name = ctypes.util.find_library('harfbuzz')
    if not name:
        return None
    lib = ctypes.CDLL(name)
    lib.hb_version_string.restype = ctypes.c_char_p
    return lib.hb_version_string().decode()


def main():
    check_sources()
    assert len(CHARS) == 204 and CHARS[-1] == '�' and CHARS == sorted(set(CHARS))
    data = bytearray(); rows = []; glyphs = []; fonts = []; records = []; unwidened = []
    for weight, px, line, extra in SPECS:
        f, axes = font_at(px, weight)
        ascent, descent = f.getmetrics()
        # Half-leading floored: the glyphs sit in the line box the way callers centre
        # `Font::height()`; a negative leading (48r) moves them up, never clips them.
        baseline = (line - (ascent + descent)) // 2 + ascent
        notdef = raster(f, '\U0010fffd', [])
        digits = set()
        for ch in 'lI46':  # every record must show the baked alternates, ss01 also under tnum
            assert raster(f, ch, FEATURES)[4].tobytes() != raster(f, ch, [])[4].tobytes(), (px, ch)
        for ch in '46':
            assert raster(f, ch, feats(ch))[4].tobytes() != raster(f, ch, ['tnum'])[4].tobytes(), (px, ch)
        first = len(glyphs)
        for ch in CHARS:
            fe = feats(ch)
            left, top, w, h, mask = raster(f, ch, fe)
            assert (w == 0 or h == 0) == (ch in BLANK), (px, ch)
            # Inter 4.001 maps no U+FFFD: its slot holds the visible .notdef bar. Every
            # other character must be a real glyph, not .notdef.
            is_notdef = (left, top, w, h, mask.tobytes()) == notdef[:4] + (notdef[4].tobytes(),)
            assert is_notdef == (ch == '�'), (px, ch)
            assert 0 <= w <= 96 and 0 <= h <= 96 and -128 <= left < 128 and -128 <= baseline + top < 128, (px, ch)
            advance = f.getlength(ch, features=fe) * 64
            assert advance == int(advance)
            advance = int(advance)
            if ch == ' ':
                unwidened.append(advance)
                advance += extra * 64
            if ch in DIGITS:
                digits.add(advance)
            assert advance < 65536
            begin = len(data); row = len(rows)
            for y in range(h):
                rows.append(len(data) - begin)
                x = 0
                while x < w:
                    alpha = min(15, (mask.getpixel((x, y)) + 8) // 17); run = 1
                    while x + run < w and run < 16 and min(15, (mask.getpixel((x + run, y)) + 8) // 17) == alpha:
                        run += 1
                    data.append(((run - 1) << 4) | alpha); x += run
            rows.append(len(data) - begin)
            assert len(data) - begin < 65536
            glyphs.append((begin, row, left, baseline + top, w, h, advance))
        assert len(digits) == 1, ('tabular digits differ', px, weight)
        fonts.append((first, px, line))
        records.append({'weight': weight, 'px': px, 'axes': {'opsz': axes[0], 'wght': axes[1]},
                        'line_height': line, 'ascent': ascent, 'descent': descent,
                        'baseline_from_line_top': baseline, 'space_extra_px': extra,
                        'space_advance_64': {'unwidened': unwidened[-1], 'stored': unwidened[-1] + extra * 64}})
    (ROOT / 'assets/inter-atlas.rle').write_bytes(data)
    rs = ['// Generated by scripts/generate_inter_atlas.py from the hash-checked',
          '// ui/design/fonts/Inter[opsz,wght].ttf (Inter 4.001, OFL-1.1). Do not edit.',
          '// Compiled only with the `inter-atlas` feature, in place of atlas.rs.',
          'use super::{FontRecord, Glyph};',
          f'pub const GLYPH_COUNT: usize = {len(CHARS)};',
          'pub static CHARS: [u32; GLYPH_COUNT] = [']
    for i in range(0, len(CHARS), 24):
        rs.append(','.join(str(ord(c)) for c in CHARS[i:i + 24]) + ',')
    rs += ['];', f'pub static FONTS: [FontRecord; {len(fonts)}] = [']
    rs += [f'FontRecord{{first:{a},size:{b},height:{c}}},' for a, b, c in fonts]
    rs += ['];', f'pub static GLYPHS: [Glyph; {len(glyphs)}] = [']
    rs += ['Glyph{offset:%s,rows:%s,left:%s,top:%s,width:%s,height:%s,advance:%s},' % g for g in glyphs]
    rs += ['];', f'pub static ROWS: [u16; {len(rows)}] = [']
    for i in range(0, len(rows), 24):
        rs.append(','.join(str(v) for v in rows[i:i + 24]) + ',')
    rs += ['];', 'pub static DATA: &[u8] = include_bytes!("../assets/inter-atlas.rle");',
           '/// Space advances (26.6) before the word-space correction, per FONTS record.',
           '#[cfg(test)]',
           f'pub const UNWIDENED_SPACE_64: [u16; {len(unwidened)}] = [' + ','.join(map(str, unwidened)) + '];']
    (ROOT / 'src/atlas_inter.rs').write_text('\n'.join(rs) + '\n')
    meta = {
        'font': 'Inter', 'version': '4.001', 'source': FONT, 'sha256': FONT_SHA256,
        'source_record': 'ui/design/fonts/FONTS.md (google/fonts commit 29b4ec6d)',
        'copyright': 'Copyright 2020 The Inter Project Authors (https://github.com/rsms/inter)',
        'license': 'OFL-1.1', 'license_file': LICENSE, 'license_sha256': LICENSE_SHA256,
        'reserved_font_name': None, 'derivative': 'Rasterised 4-bit coverage atlas of the font software',
        'cargo_feature': 'inter-atlas', 'features': FEATURES, 'digit_features': ['tnum'], 'digits': DIGITS,
        'fallback': 'U+FFFD slot; Inter 4.001 maps no U+FFFD, so it holds the font\'s visible .notdef ("NO GLYPH") glyph',
        'line_height_rule':'FontRecord.height = type-1.0 role line height; glyph top includes floor((height - (ascent + descent)) / 2)',
        'records': records, 'characters': len(CHARS), 'glyphs': len(glyphs), 'row_offsets': len(rows),
        'data_bytes': len(data), 'data_sha256': hashlib.sha256(data).hexdigest(),
        'glyph_table_bytes': len(glyphs) * 16, 'row_table_bytes': len(rows) * 2, 'glyph_bound_px': 96,
        'encoding': ENCODING, 'native_parser': 'None; fixed checked metadata and bounded row decoder',
        'build_tools': {'pillow': PILLOW_VERSION, 'freetype2': features.version('freetype2'),
                        'raqm': features.version('raqm'), 'harfbuzz': harfbuzz_version()},
        'build_time_only': 'Pillow/FreeType/raqm/HarfBuzz; not native dependencies',
    }
    (ROOT / 'assets/inter-atlas-provenance.json').write_text(json.dumps(meta, indent=2) + '\n')
    print(json.dumps({k: meta[k] for k in ['characters', 'glyphs', 'row_offsets', 'data_bytes', 'data_sha256']}))


if __name__ == '__main__':
    main()
