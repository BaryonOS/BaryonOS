#!/usr/bin/env python3
"""Host-only fixture encoder/oracle. Never part of the OS decode path."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import struct
import zlib
from PIL import Image, ImageCms, PngImagePlugin, __version__ as pillow_version

HERE = Path(__file__).resolve().parent


def chunk(kind: bytes, data: bytes) -> bytes:
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


def png(w: int, h: int, samples: bytes, depth: int = 8, channels: int = 3,
        interlace: bool = False, extra: bytes = b"") -> bytes:
    # Fixture-only encoder, filter None. Adam7 layout follows PNG's seven passes.
    pixel_bytes = channels * (depth // 8)
    passes = ((0, 0, 1, 1),) if not interlace else (
        (0, 0, 8, 8), (4, 0, 8, 8), (0, 4, 4, 8), (2, 0, 4, 4),
        (0, 2, 2, 4), (1, 0, 2, 2), (0, 1, 1, 2))
    scan = bytearray()
    for x0, y0, dx, dy in passes:
        for y in range(y0, h, dy):
            scan.append(0)
            for x in range(x0, w, dx):
                offset = (y * w + x) * pixel_bytes
                scan.extend(samples[offset:offset + pixel_bytes])
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, depth,
            {1: 0, 2: 4, 3: 2, 4: 6}[channels], 0, 0, int(interlace))) + extra +
            chunk(b"IDAT", zlib.compress(scan, 9)) + chunk(b"IEND", b""))


def pattern(w: int, h: int, mode: str = "RGB") -> Image.Image:
    image = Image.new(mode, (w, h))
    values = []
    for y in range(h):
        for x in range(w):
            rgb = ((x * 3 + y * 2) % 256, (x + y * 5) % 256, (x * 7 + y) % 256)
            values.append(rgb if mode == "RGB" else rgb + ((x * 13 + y * 17) % 256,))
    image.putdata(values)
    return image


def main() -> None:
    images = []
    def record(name: str, image: Image.Image | None, **save) -> None:
        path = HERE / name
        if image is not None:
            image.save(path, **save)
        with Image.open(path) as reference:
            reference.load()
            rgba = reference.convert("RGBA").tobytes()
            (HERE / (name + ".rgba")).write_bytes(rgba)
            images.append({"name": name, "width": reference.width, "height": reference.height,
                "format": reference.format, "input_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "reference_rgba_sha256": hashlib.sha256(rgba).hexdigest(),
                "reference_tolerance": 0 if reference.format == "PNG" else 3})

    text = PngImagePlugin.PngInfo()
    text.add_text("Fixture", "Nieuw OS codec fixture; no personal content", zip=True)
    profile = bytearray(ImageCms.ImageCmsProfile(ImageCms.createProfile("sRGB")).tobytes())
    profile[24:36] = struct.pack(">6H", 2026, 1, 1, 0, 0, 0)  # Deterministic ICC date.
    record("rgb.png", pattern(231, 149), pnginfo=text, icc_profile=bytes(profile))
    record("alpha.png", pattern(128, 96, "RGBA"))
    palette = Image.new("P", (73, 47))
    palette.putpalette([c for i in range(256) for c in (i, (i * 3) % 256, 255 - i)])
    palette.putdata([(x + y * 3) % 16 for y in range(47) for x in range(73)])
    record("palette4.png", palette, bits=4, transparency=bytes(i * 17 for i in range(16)))
    mono = Image.new("1", (65, 33))
    mono.putdata([255 if (x // 3 + y // 2) % 2 else 0 for y in range(33) for x in range(65)])
    record("mono1.png", mono)
    gray = Image.new("LA", (97, 65))
    gray.putdata([((x * 3 + y) % 256, (x + y * 7) % 256) for y in range(65) for x in range(97)])
    record("gray-alpha.png", gray)
    source = pattern(129, 77)
    (HERE / "adam7.png").write_bytes(png(source.width, source.height, source.tobytes(), interlace=True))
    record("adam7.png", None)
    rgba16 = bytearray()
    for y in range(65):
        for x in range(97):
            rgba16.extend(struct.pack(">4H", (x * 997 + y * 317) % 65536,
                (x * 131 + y * 1031) % 65536, (x * 701 + y * 199) % 65536,
                (x * 431 + y * 787) % 65536))
    (HERE / "rgba16.png").write_bytes(png(97, 65, rgba16, depth=16, channels=4))
    record("rgba16.png", None)
    (HERE / "rgba16.png.rgba16be").write_bytes(rgba16)
    images[-1]["reference_rgba16be_sha256"] = hashlib.sha256(rgba16).hexdigest()
    # Display rounds full samples to 8 bit; Pillow's PNG8 view truncates instead.
    rounded = bytes((int.from_bytes(rgba16[i:i+2], "big") + 128) // 257
                    for i in range(0, len(rgba16), 2))
    (HERE / "rgba16.png.rgba").write_bytes(rounded)
    images[-1]["reference_rgba_sha256"] = hashlib.sha256(rounded).hexdigest()
    exif = Image.Exif()
    exif[274] = 6  # Preserved; orientation rendering remains explicitly open.
    record("baseline.jpg", pattern(257, 129), quality=95, subsampling=0, optimize=True, exif=exif)
    record("progressive.jpg", pattern(193, 113), quality=95, subsampling=2, progressive=True)
    record("gray.jpg", pattern(113, 67).convert("L"), quality=95)
    record("cmyk.jpg", pattern(61, 43).convert("CMYK"), quality=95)
    # Small compressed metadata with a large expansion, separate from pixel data.
    bomb = chunk(b"iCCP", b"profile\x00\x00" + zlib.compress(b"X" * (1024 * 1024), 9))
    (HERE / "icc-bomb.png").write_bytes(png(8, 8, b"\x20\x40\x60" * 64, extra=bomb))
    result = {"purpose": "Synthetic host fixtures/oracles; no native decoding by Pillow",
              "generator": "generate.py", "pillow_version": pillow_version,
              "zlib_version": zlib.ZLIB_RUNTIME_VERSION, "images": images}
    (HERE / "manifest.json").write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
