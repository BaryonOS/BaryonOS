"""Independent Pillow decode of synthetic RGB fixtures; no native evidence."""
from io import BytesIO
from pathlib import Path
import sys

import PIL
from PIL import Image

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools"))
from capture_graphics import encode_png

print("Synthetic host interoperability check; no guest/framebuffer execution.")
print("Pillow", PIL.__version__)
for width, height in ((1, 1), (3, 2), (96, 224), (800, 600)):
    pixels = bytes((index * 37 + index // 13) % 256 for index in range(width * height * 3))
    source = f"P6\n{width} {height}\n255\n".encode() + pixels
    encoded = encode_png(source)
    with Image.open(BytesIO(encoded)) as decoded:
        decoded.load()
        assert decoded.mode == "RGB"
        assert decoded.size == (width, height)
        assert decoded.tobytes() == pixels
    print(f"PASS: {width}x{height}, all {len(pixels)} original RGB bytes")
