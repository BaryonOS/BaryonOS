#!/usr/bin/env python3
"""Measure candidate UI typefaces for the BaryonOS typography decision.

Reports, per font file: SHA-256, variation axes, OpenType features, character
coverage for Dutch/European use, vertical metrics, width, stroke contrast,
aperture, and the pixel distinctness of easily confused glyph pairs at real UI
sizes (FreeType rasterisation via Pillow, greyscale antialiasing).

These are physical proxies, not perceptual measurements: a higher pair
distinctness means the two rendered bitmaps differ more, which is necessary but
not sufficient for readers to tell them apart.

Usage: python3 -I ui/design/typography/measure_fonts.py FONT_DIR [FONT_DIR...] OUT_PREFIX
Writes OUT_PREFIX.json and OUT_PREFIX.md.
"""
from __future__ import annotations

import hashlib
import json
import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFont

PAIRS = [("I", "l"), ("l", "1"), ("I", "1"), ("O", "0"), ("rn", "m"), ("c", "e"),
         ("6", "8"), ("5", "S"), ("B", "8"), ("a", "o"), ("cl", "d"), ("vv", "w")]
SIZES_1X = [11, 12, 13, 14, 16]  # px at 100 % scale
SIZES_2X = [s * 2 for s in SIZES_1X]  # the same text at 200 % scale
WEIGHT = 400

# Letters beyond ASCII per official EU language (plus Dutch with the € sign).
# Greek and Bulgarian Cyrillic are listed separately: they may come from a named
# fallback family instead of the primary one (see docs/design/TYPOGRAPHY.md).
LANGUAGES = {
    "nl": "éëïöüèáóúÉËÏÖÜ€", "de": "äöüßÄÖÜẞ", "fr": "àâæçéèêëîïôœùûüÿÀÂÆÇÉÈÊËÎÏÔŒÙÛÜŸ«»",
    "es": "ñáéíóúüÑÁÉÍÓÚÜ¿¡", "it": "àèéìíîòóùúÀÈÉÌÍÎÒÓÙÚ", "pt": "ãõâêôáéíóúàçÃÕÂÊÔÁÉÍÓÚÀÇ",
    "pl": "ąćęłńóśźżĄĆĘŁŃÓŚŹŻ", "cs": "áčďéěíňóřšťúůýžÁČĎÉĚÍŇÓŘŠŤÚŮÝŽ",
    "sk": "áäčďéíĺľňóôŕšťúýžÁÄČĎÉÍĹĽŇÓÔŔŠŤÚÝŽ", "hu": "áéíóöőúüűÁÉÍÓÖŐÚÜŰ", "ro": "ăâîșțĂÂÎȘȚ",
    "hr": "čćđšžČĆĐŠŽ", "sl": "čšžČŠŽ", "lt": "ąčęėįšųūžĄČĘĖĮŠŲŪŽ", "lv": "āčēģīķļņšūžĀČĒĢĪĶĻŅŠŪŽ",
    "et": "äöõüšžÄÖÕÜŠŽ", "fi": "äöåÄÖÅšž", "sv": "åäöÅÄÖ", "da": "æøåÆØÅ", "ga": "áéíóúÁÉÍÓÚ",
    "mt": "ċġħżàèìòùĊĠĦŻÀÈÌÒÙ",
}
SCRIPTS = {
    "el": "ΑΒΓΔΕΖΗΘΙΚΛΜΝΞΟΠΡΣΤΥΦΧΨΩαβγδεζηθικλμνξοπρσςτυφχψωάέήίόύώΆΈΉΊΌΎΏϊϋΐΰ",
    "bg": "".join(chr(c) for c in range(0x410, 0x450)),
}
UI_SYMBOLS = "–—…•·×−≤≥≠→←↑↓°‰‘’“”€£\u00a0\u2009\u202f"


def sfnt_tables(data: bytes) -> dict:
    count = struct.unpack(">H", data[4:6])[0]
    out = {}
    for i in range(count):
        tag, _, off, length = struct.unpack(">4sIII", data[12 + 16 * i: 28 + 16 * i])
        out[tag.decode("latin-1")] = (off, length)
    return out


def fvar_axes(data: bytes, t: dict) -> list:
    if "fvar" not in t:
        return []
    off = t["fvar"][0]
    axes_off, _, count, size = struct.unpack(">HHHH", data[off + 4: off + 12])
    axes = []
    for i in range(count):
        a = off + axes_off + i * size
        tag, mn, df, mx = struct.unpack(">4siii", data[a: a + 16])
        axes.append((tag.decode(), mn / 65536, df / 65536, mx / 65536))
    return axes


def feature_tags(data: bytes, t: dict, table: str) -> list:
    if table not in t:
        return []
    off = t[table][0]
    fl = off + struct.unpack(">H", data[off + 6: off + 8])[0]
    n = struct.unpack(">H", data[fl: fl + 2])[0]
    return sorted({data[fl + 2 + 6 * i: fl + 6 + 6 * i].decode("latin-1") for i in range(n)})


def cmap_codepoints(data: bytes, t: dict) -> set:
    off = t["cmap"][0]
    n = struct.unpack(">H", data[off + 2: off + 4])[0]
    best = None
    for i in range(n):
        pid, eid, sub = struct.unpack(">HHI", data[off + 4 + 8 * i: off + 12 + 8 * i])
        fmt = struct.unpack(">H", data[off + sub: off + sub + 2])[0]
        if fmt == 12:
            best = (12, off + sub)
            break
        if fmt == 4 and pid in (0, 3):
            best = best or (4, off + sub)
    cps = set()
    fmt, s = best
    if fmt == 12:
        groups = struct.unpack(">I", data[s + 12: s + 16])[0]
        for g in range(groups):
            a, b, _ = struct.unpack(">III", data[s + 16 + 12 * g: s + 28 + 12 * g])
            cps.update(range(a, b + 1))
    else:
        segx2 = struct.unpack(">H", data[s + 6: s + 8])[0]
        ends = struct.unpack(f">{segx2 // 2}H", data[s + 14: s + 14 + segx2])
        starts = struct.unpack(f">{segx2 // 2}H", data[s + 16 + segx2: s + 16 + 2 * segx2])
        for a, b in zip(starts, ends):
            if a != 0xFFFF:
                cps.update(range(a, b + 1))
    return cps


def vertical_metrics(data: bytes, t: dict) -> dict:
    upm = struct.unpack(">H", data[t["head"][0] + 18: t["head"][0] + 20])[0]
    o = t["OS/2"][0]
    version = struct.unpack(">H", data[o: o + 2])[0]
    asc, desc, gap = struct.unpack(">hhh", data[o + 68: o + 74])
    x_h = cap = None
    if version >= 2:
        x_h, cap = struct.unpack(">hh", data[o + 86: o + 90])
    return {"upm": upm, "x_height": x_h and x_h / upm, "cap_height": cap and cap / upm,
            "x_to_cap": x_h and cap and round(x_h / cap, 3),
            "line_gap_total": round((asc - desc + gap) / upm, 3)}


def load(path: Path, size: int, axes: list, weight: int = WEIGHT, opsz=None):
    f = ImageFont.truetype(str(path), size, layout_engine=ImageFont.Layout.RAQM)
    if axes:
        vals = []
        for tag, mn, df, mx in axes:
            v = df
            if tag == "wght":
                v = weight
            elif tag == "opsz":
                v = opsz if opsz is not None else max(mn, min(mx, size))
            elif tag == "wdth":
                v = 100
            vals.append(max(mn, min(mx, v)))
        f.set_variation_by_axes(vals)
    return f


def raster(font, text: str, w: int, h: int, baseline: int, features=None) -> np.ndarray:
    img = Image.new("L", (w, h), 0)
    x0, _, x1, _ = font.getbbox(text, anchor="ls", features=features)
    ImageDraw.Draw(img).text(((w - (x1 - x0)) / 2 - x0, baseline), text, font=font, fill=255,
                             anchor="ls", features=features)
    return np.asarray(img, dtype=np.float64) / 255.0


# Global alternates the design system would switch on per family (docs/design/TYPOGRAPHY.md).
CONFIGURED = {"Inter": ["cv05", "cv08", "ss01"]}


def distinctness(path: Path, axes: list, a: str, b: str, size: int, features=None) -> float:
    font = load(path, size, axes)
    w, h = size * 3, size * 2
    ra = raster(font, a, w, h, int(size * 1.4), features)
    rb = raster(font, b, w, h, int(size * 1.4), features)
    total = ra.sum() + rb.sum()
    return float(np.abs(ra - rb).sum() / total) if total else 0.0


def shape_metrics(path: Path, axes: list) -> dict:
    size = 400
    font = load(path, size, axes, opsz=32)
    out = {}
    letters = "abcdefghijklmnopqrstuvwxyz"
    out["avg_advance_em"] = round(font.getlength(letters) / (26 * size), 3)
    # Stroke contrast of 'o': thinner / thicker of the side stem and the top bowl.
    o = raster(font, "o", size * 2, size * 2, int(size * 1.4)) > 0.5
    ys, xs = np.nonzero(o)
    cy, cx = (ys.min() + ys.max()) // 2, (xs.min() + xs.max()) // 2
    vert = int(np.argmax(~o[cy, xs.min():])) or 1
    horiz = int(np.argmax(~o[ys.min():, cx])) or 1
    out["stroke_contrast"] = round(min(vert, horiz) / max(vert, horiz), 3)
    # Aperture: largest open gap in the column at 88 % of the glyph width, relative to height.
    for ch in "ce":
        g = raster(font, ch, size * 2, size * 2, int(size * 1.4)) > 0.5
        ys, xs = np.nonzero(g)
        x = xs.min() + int((xs.max() - xs.min()) * 0.88)
        best = run = 0
        for v in g[ys.min(): ys.max() + 1, x]:
            run = 0 if v else run + 1
            best = max(best, run)
        out[f"aperture_{ch}"] = round(best / (ys.max() - ys.min() + 1), 3)
    # Characters per 560 px line at 15 px, Dutch running text.
    sample = ("Thuis-NAS is niet bereikbaar. Je werk staat veilig op dit apparaat "
              "en wordt bijgewerkt zodra de NAS terug is.")
    f15 = load(path, 15, axes)
    out["chars_per_560px_at_15px"] = round(560 / (f15.getlength(sample) / len(sample)), 1)
    return out


def measure(path: Path) -> dict:
    data = path.read_bytes()
    t = sfnt_tables(data)
    axes = fvar_axes(data, t)
    cps = cmap_codepoints(data, t)
    def missing(chars):
        return "".join(c for c in chars if ord(c) not in cps)
    gaps = {k: missing(v) for k, v in LANGUAGES.items()}
    cover = {
        "eu_latin_languages": sum(1 for v in gaps.values() if not v),
        "eu_latin_of": len(LANGUAGES),
        "eu_latin_gaps": {k: v for k, v in gaps.items() if v},
        "scripts": {k: missing(v) == "" for k, v in SCRIPTS.items()},
        "ui_symbols_missing": missing(UI_SYMBOLS),
        "ij_ligature": 0x132 in cps,
    }
    pairs = {}
    for a, b in PAIRS:
        d1 = [distinctness(path, axes, a, b, s) for s in SIZES_1X]
        d2 = [distinctness(path, axes, a, b, s) for s in SIZES_2X]
        pairs[f"{a}/{b}"] = {"1x": round(float(np.mean(d1)), 3), "2x": round(float(np.mean(d2)), 3)}
    feats = feature_tags(data, t, "GSUB")
    # Disambiguation alternates a UI can switch on (Inter: tailed l, serifed I; others: slashed zero).
    # Each feature is measured on its own, only on the pairs it can change.
    alt = [f for f in ("cv05", "cv08", "zero") if f in feats]
    affects = {"cv05": [("I", "l"), ("l", "1"), ("cl", "d")], "cv08": [("I", "l"), ("I", "1")], "zero": [("O", "0")]}
    alt_pairs = {}
    for f in alt:
        alt_pairs[f] = {f"{a}/{b}": round(float(np.mean([distinctness(path, axes, a, b, s, [f]) for s in SIZES_1X])), 3)
                        for a, b in affects[f]}
    # The configuration a family would ship with, measured as a whole (see CONFIGURED).
    conf = next((v for k, v in CONFIGURED.items() if path.name.startswith(k)), None)
    configured = None
    if conf:
        configured = {"features": conf, "pairs": {
            f"{a}/{b}": round(float(np.mean([distinctness(path, axes, a, b, s, conf) for s in SIZES_1X])), 3)
            for a, b in PAIRS}}
    return {
        "configured": configured,
        "file": path.name,
        "sha256": hashlib.sha256(data).hexdigest(),
        "bytes": len(data),
        "axes": [{"tag": a, "min": mn, "default": df, "max": mx} for a, mn, df, mx in axes],
        "gsub_features": feats,
        "gpos_features": feature_tags(data, t, "GPOS"),
        "has_tnum": "tnum" in feats,
        "has_zero": "zero" in feats,
        "glyph_count_cmap": len(cps),
        "coverage": cover,
        "metrics": vertical_metrics(data, t),
        "shape": shape_metrics(path, axes),
        "pairs": pairs,
        "pair_min_1x": min(p["1x"] for p in pairs.values()),
        "pair_mean_1x": round(float(np.mean([p["1x"] for p in pairs.values()])), 3),
        "alternates": alt,
        "pairs_with_alternates_1x": alt_pairs,
    }


def shown(chars: str) -> str:
    return " ".join(c if c.isprintable() and not c.isspace() else f"U+{ord(c):04X}" for c in chars)


def markdown(results: list) -> str:
    keys = [f"{a}/{b}" for a, b in PAIRS]
    lines = ["| Font | x/cap | x-height | width (em) | contrast | aperture c / e | chars/560px | tnum | zero | EU Latin languages | Greek | Bulgarian | UI symbols missing |",
             "|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    for r in results:
        m, s, c = r["metrics"], r["shape"], r["coverage"]
        gaps = ", ".join(c["eu_latin_gaps"]) or "none"
        lines.append(
            f"| {r['file']} | {m['x_to_cap']} | {m['x_height'] and round(m['x_height'], 3)} | "
            f"{s['avg_advance_em']} | {s['stroke_contrast']} | {s['aperture_c']} / {s['aperture_e']} | "
            f"{s['chars_per_560px_at_15px']} | {'yes' if r['has_tnum'] else 'no'} | {'yes' if r['has_zero'] else 'no'} | "
            f"{c['eu_latin_languages']}/{c['eu_latin_of']} (gaps: {gaps}) | {'yes' if c['scripts']['el'] else 'no'} | "
            f"{'yes' if c['scripts']['bg'] else 'no'} | {shown(c['ui_symbols_missing']) or 'none'} |")
    for scale in ("1x", "2x"):
        label = "100 % (mean over 11–16 px)" if scale == "1x" else "200 % (mean over 22–32 px)"
        lines += ["", f"Pair distinctness at {label}; 0 = identical bitmaps:", "",
                  "| Font | " + " | ".join(keys) + " | min | mean |", "|---|" + "---|" * (len(keys) + 2)]
        for r in results:
            vals = [r["pairs"][k][scale] for k in keys]
            lines.append(f"| {r['file']} | " + " | ".join(str(v) for v in vals)
                         + f" | {min(vals)} | {round(float(np.mean(vals)), 3)} |")
    lines += ["", "Disambiguation features, each on its own (100 %, only the pairs a feature can change):", "",
              "| Font | feature | pairs |", "|---|---|---|"]
    for r in results:
        for f, vals in r["pairs_with_alternates_1x"].items():
            lines.append(f"| {r['file']} | {f} | " + ", ".join(
                f"{k} {r['pairs'][k]['1x']} → {v}" for k, v in vals.items()) + " |")
    conf = [r for r in results if r["configured"]]
    if conf:
        keys = list(conf[0]["configured"]["pairs"])
        lines += ["", "Configured sets as shipped, all features together (100 %):", "",
                  "| Font | features | " + " | ".join(keys) + " | min | mean |", "|---|---|" + "---|" * (len(keys) + 2)]
        for r in conf:
            vals = [r["configured"]["pairs"][k] for k in keys]
            lines.append(f"| {r['file']} | {' '.join(r['configured']['features'])} | " + " | ".join(str(v) for v in vals)
                         + f" | {min(vals)} | {round(float(np.mean(vals)), 3)} |")
    return "\n".join(lines) + "\n"


def main() -> None:
    *dirs, prefix = sys.argv[1:]
    prefix = Path(prefix)
    fonts = {p.name: p for d in dirs for p in Path(d).glob("*.ttf")}
    results = [measure(fonts[k]) for k in sorted(fonts)]
    prefix.parent.mkdir(parents=True, exist_ok=True)
    prefix.with_suffix(".json").write_text(json.dumps(results, indent=1, ensure_ascii=False) + "\n")
    prefix.with_suffix(".md").write_text(markdown(results))
    print(markdown(results))


if __name__ == "__main__":
    main()
