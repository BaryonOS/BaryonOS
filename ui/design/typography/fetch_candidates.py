#!/usr/bin/env python3
"""Fetch the candidate typefaces compared in docs/design/TYPOGRAPHY.md.

Files come from github.com/google/fonts at a pinned commit and are verified by
SHA-256 before they are written. All are SIL OFL 1.1. They land in the
git-ignored ui/design/typography/candidates/ and are used only by the specimen
page and measure_fonts.py; only the selected family is vendored in ui/design/fonts/.

Usage: python3 -I ui/design/typography/fetch_candidates.py
"""
import hashlib
import sys
import urllib.parse
import urllib.request
from pathlib import Path

COMMIT = "29b4ec6df2522a8aee279414b78a07910b065874"  # google/fonts main, 13 Nov 2026
FILES = {
    ("sora", "Sora[wght].ttf"): "84ff7096ae3ec6c8be47d906d1a0ba4de7f2ce78c615275c77301964a316e16c",
    ("exo2", "Exo2[wght].ttf"): "205a448676a2586f9c57c25f3d5c58ca8db7e6cf5edf7506783a010c6fe2bfb5",
    ("spacegrotesk", "SpaceGrotesk[wght].ttf"): "acad6de1fc93436f5c0f1f4137751ef04f1aea3063e7036535970ffcfbd79f72",
    ("inter", "Inter[opsz,wght].ttf"): "29160a80ff49ddcab2c97711247e08b1fab27a484a329ce8b813d820dc559031",
    ("atkinsonhyperlegiblenext", "AtkinsonHyperlegibleNext[wght].ttf"): "5a455d1cfa099b601ab70751bb9673e8fe1854dc4500c80e1a220d0d75e31745",
    ("saira", "Saira[wdth,wght].ttf"): "9d050fc5a01c85f74c4257c207d50b55d1e40c37308c642f974a2c5003231dde",
    ("manrope", "Manrope[wght].ttf"): "3ae11c49db0455a3cc33e37d380f20fdb8c7f8b41dc07625c177e3d87a9d6ae6",
    ("lexend", "Lexend[wght].ttf"): "3add53e641fbc81da64da4bb254285e2831b52b029527bc0714e2b9610832ee6",
    ("geist", "Geist[wght].ttf"): "73894e0448cae90a92b6c2f8732b7bb9acb7b94c418bff559dad4a18e1de9659",
    ("oxanium", "Oxanium[wght].ttf"): "2ce01d946e1e1ffc8d7eecfffbda8623bedd63eaf811a20488c4b69af45babb0",
}


def main() -> int:
    out = Path(__file__).resolve().parent / "candidates"
    out.mkdir(exist_ok=True)
    failed = 0
    for (family, name), digest in FILES.items():
        url = f"https://raw.githubusercontent.com/google/fonts/{COMMIT}/ofl/{family}/{urllib.parse.quote(name)}"
        data = urllib.request.urlopen(url, timeout=60).read()
        if hashlib.sha256(data).hexdigest() != digest:
            print(f"hash mismatch, not written: {name}", file=sys.stderr)
            failed += 1
            continue
        (out / name).write_bytes(data)
        lic = urllib.request.urlopen(url.rsplit("/", 1)[0] + "/OFL.txt", timeout=60).read()
        (out / f"OFL-{family}.txt").write_bytes(lic)
        print(f"ok {name}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
