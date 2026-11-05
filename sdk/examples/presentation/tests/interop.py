#!/usr/bin/env python3
"""Host-only PPM output inspection and PNG evidence, using standard Python only."""
import hashlib
from pathlib import Path
import struct
import subprocess
import sys
import zlib


def png_chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


def main():
    binary = Path(sys.argv[1]).resolve()
    evidence = Path(sys.argv[2]).resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    source = (Path(__file__).resolve().parents[1] / "fixtures/deck.txt").read_bytes()
    for scale in [1, 2]:
        frames = []
        width, height = 640 * scale, 360 * scale
        header = f"P6\n{width} {height}\n255\n".encode()
        for selected in [1, 2]:
            result = subprocess.run(
                [str(binary), "--slide", str(selected), "--scale", str(scale)],
                input=source, capture_output=True, check=True,
            )
            assert not result.stderr and result.stdout.startswith(header)
            rgb = result.stdout[len(header):]
            assert len(rgb) == width * height * 3
            assert rgb[:3] == b"\xff\xff\xff"
            offset = ((88 * scale) * width + 32 * scale) * 3
            assert rgb[offset:offset + 3] == bytes.fromhex("5449cf")
            for top, bottom in [(24, 80), (108, 200), (332, 359)]:
                region = rgb[top * scale * width * 3:bottom * scale * width * 3]
                assert sum(value != 255 for value in region) > 30, (selected, scale, top)
            frames.append(rgb)
            rows = b"".join(
                b"\0" + rgb[y * width * 3:(y + 1) * width * 3] for y in range(height)
            )
            png = (
                b"\x89PNG\r\n\x1a\n"
                + png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
                + png_chunk(b"IDAT", zlib.compress(rows))
                + png_chunk(b"IEND", b"")
            )
            path = evidence / f"slide-{selected}-scale-{scale}.png"
            path.write_bytes(png)
            print(f"{width}x{height} page={selected} sha256(PPM)={hashlib.sha256(result.stdout).hexdigest()} {path.name}")
        assert frames[0] != frames[1]
    print("4 actual raster outputs inspected; PNGs are evidence only, not a product encoder")


if __name__ == "__main__":
    main()
