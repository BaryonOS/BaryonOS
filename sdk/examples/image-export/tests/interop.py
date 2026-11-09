#!/usr/bin/env python3
"""Host-only BMP reader/oracle using existing independent fixture references."""
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys


def main():
    binary = Path(sys.argv[1]).resolve()
    fixtures = Path(__file__).resolve().parents[4] / "services/image-codec/fixtures"
    records = json.loads((fixtures / "manifest.json").read_text())["images"]
    count = 0
    for record in records:
        if record["name"] == "cmyk.jpg":
            continue  # The existing product decoder explicitly rejects CMYK.
        original = (fixtures / record["name"]).read_bytes()
        reference = (fixtures / (record["name"] + ".rgba")).read_bytes()
        assert hashlib.sha256(original).hexdigest() == record["input_sha256"]
        assert hashlib.sha256(reference).hexdigest() == record["reference_rgba_sha256"]
        sw, sh = record["width"], record["height"]
        rotate = record["name"] == "baseline.jpg"  # Fixture generator fixes EXIF 6.
        w, h = (sh, sw) if rotate else (sw, sh)
        stride = ((24 * w + 31) // 32) * 4
        for background in [(0, 0, 0), (255, 255, 255), (18, 52, 86)]:
            result = subprocess.run(
                [str(binary), "--background", "".join(f"{c:02x}" for c in background)],
                input=original, capture_output=True, check=True,
            )
            assert not result.stderr
            bmp = result.stdout
            assert struct.unpack_from("<2sIHHI", bmp) == (b"BM", len(bmp), 0, 0, 54)
            assert struct.unpack_from("<IiiHHIIiiII", bmp, 14) == (
                40, w, h, 1, 24, 0, h * stride, 0, 0, 0, 0,
            )
            assert len(bmp) == 54 + h * stride
            worst = 0
            for y in range(h):
                row = bmp[54 + (h - 1 - y) * stride:54 + (h - y) * stride]
                assert row[w * 3:] == bytes(stride - w * 3)
                for x in range(w):
                    sx, sy = (y, sh - 1 - x) if rotate else (x, y)
                    offset = (sy * sw + sx) * 4
                    r, g, b, alpha = reference[offset:offset + 4]
                    expected = [
                        (value * alpha + bg * (255 - alpha) + 127) // 255
                        for value, bg in zip((r, g, b), background)
                    ]
                    actual = row[x * 3:x * 3 + 3][::-1]
                    worst = max(worst, *(abs(a - e) for a, e in zip(actual, expected)))
            assert worst <= record["reference_tolerance"], (record["name"], worst)
            count += 1
            print(f"{record['name']} {w}x{h} background={background} max_error={worst} OK")
        assert (fixtures / record["name"]).read_bytes() == original
    print(f"{count} actual host CLI exports independently verified; no native claim")


if __name__ == "__main__":
    main()
