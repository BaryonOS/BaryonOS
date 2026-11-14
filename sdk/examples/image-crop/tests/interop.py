#!/usr/bin/env python3
"""Independent zlib/PNG oracle; no production encoder or decoder used here."""
import binascii
import hashlib
import json
import pathlib
import struct
import subprocess
import sys
import zlib

SIGNATURE = b"\x89PNG\r\n\x1a\n"
CHOICES = ["--drop-metadata", "--unmanaged-colour"]


def chunk(name, data):
    return struct.pack(">I", len(data)) + name + data + struct.pack(">I", binascii.crc32(name + data))


def png(width, height, rgba, metadata=False):
    rows = b"".join(b"\0" + rgba[y * width * 4:(y + 1) * width * 4] for y in range(height))
    result = SIGNATURE + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
    if metadata:
        result += chunk(b"tEXt", b"Comment\0private metadata must not survive")
        result += chunk(b"gAMA", struct.pack(">I", 45455))
    return result + chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b"")


def decode_png(data):
    assert data.startswith(SIGNATURE)
    at, compressed, kinds = 8, bytearray(), []
    while at < len(data):
        size = struct.unpack_from(">I", data, at)[0]
        kind = data[at + 4:at + 8]
        content = data[at + 8:at + 8 + size]
        assert struct.unpack_from(">I", data, at + 8 + size)[0] == binascii.crc32(kind + content)
        kinds.append(kind)
        if kind == b"IHDR":
            width, height, depth, colour, compression, filtering, interlace = struct.unpack(">IIBBBBB", content)
            assert (depth, colour, compression, filtering, interlace) == (8, 6, 0, 0, 0)
        elif kind == b"IDAT":
            compressed.extend(content)
        else:
            assert kind == b"IEND" and content == b""
        at += size + 12
    assert at == len(data) and kinds[0] == b"IHDR" and kinds[-1] == b"IEND"
    decoder = zlib.decompressobj()
    raw = decoder.decompress(compressed) + decoder.flush()
    assert decoder.eof and not decoder.unused_data and not decoder.unconsumed_tail
    stride = width * 4
    assert len(raw) == height * (stride + 1)
    output = bytearray()
    previous = bytearray(stride)
    for y in range(height):
        start = y * (stride + 1)
        filtering = raw[start]
        assert filtering in range(5)
        row = bytearray(raw[start + 1:start + 1 + stride])
        for i in range(stride):
            left = row[i - 4] if i >= 4 else 0
            above = previous[i]
            upper_left = previous[i - 4] if i >= 4 else 0
            if filtering == 1:
                row[i] = (row[i] + left) & 255
            elif filtering == 2:
                row[i] = (row[i] + above) & 255
            elif filtering == 3:
                row[i] = (row[i] + (left + above) // 2) & 255
            elif filtering == 4:
                p = left + above - upper_left
                distances = (abs(p - left), abs(p - above), abs(p - upper_left))
                predictor = (left, above, upper_left)[distances.index(min(distances))]
                row[i] = (row[i] + predictor) & 255
        output.extend(row)
        previous = row
    return width, height, bytes(output)


def select(rgba, width, rect):
    x, y, w, h = rect
    return b"".join(rgba[((y + row) * width + x) * 4:((y + row) * width + x + w) * 4]
                    for row in range(h))


def orient(rgba, width, height, orientation):
    swapped = orientation >= 5
    out_width, out_height = (height, width) if swapped else (width, height)
    output = bytearray(len(rgba))
    for y in range(height):
        for x in range(width):
            positions = [
                (x, y), (width - 1 - x, y), (width - 1 - x, height - 1 - y),
                (x, height - 1 - y), (y, x), (height - 1 - y, x),
                (height - 1 - y, width - 1 - x), (y, width - 1 - x),
            ]
            dx, dy = positions[orientation - 1]
            target = (dy * out_width + dx) * 4
            source = (y * width + x) * 4
            output[target:target + 4] = rgba[source:source + 4]
    return out_width, out_height, bytes(output)


def exif(jpeg, orientation):
    assert jpeg[:2] == b"\xff\xd8"
    tiff = b"II" + struct.pack("<HIH", 42, 8, 1)
    tiff += struct.pack("<HHI", 0x112, 3, 1) + struct.pack("<H", orientation) + b"\0\0" + b"\0" * 4
    payload = b"Exif\0\0" + tiff
    return jpeg[:2] + b"\xff\xe1" + struct.pack(">H", len(payload) + 2) + payload + jpeg[2:]


def invoke(binary, data, rect, success=True, choices=CHOICES, extra=()):
    args = [binary, "--x", str(rect[0]), "--y", str(rect[1]), "--width", str(rect[2]),
            "--height", str(rect[3]), *choices, *extra]
    result = subprocess.run(args, input=data, capture_output=True, timeout=10)
    if success:
        assert result.returncode == 0 and not result.stderr, result.stderr
        return result.stdout
    assert result.returncode != 0 and result.stdout == b"" and result.stderr, result
    return None


def main():
    binary = str(pathlib.Path(sys.argv[1]).resolve())
    target = pathlib.Path(sys.argv[2])
    target.parent.mkdir(parents=True, exist_ok=True)
    positives = 0
    for width, height in ((1, 1), (2, 3), (17, 19), (37, 23)):
        rgba = bytes((x * 53 + y * 17 + component * 71) & 255
                     for y in range(height) for x in range(width) for component in range(4))
        for metadata in (False, True):
            source = png(width, height, rgba, metadata)
            for rect in ((0, 0, width, height), (width - 1, height - 1, 1, 1),
                         (width // 2, 0, width - width // 2, height)):
                output = invoke(binary, source, rect)
                w, h, pixels = decode_png(output)
                assert (w, h) == rect[2:]
                assert pixels == select(rgba, width, rect)
                target.write_bytes(output)
                positives += 1
    fixtures = pathlib.Path(__file__).resolve().parents[4] / "services/image-codec/fixtures"
    manifest = json.loads((fixtures / "edges.json").read_text())
    # Existing independently generated Pillow/libjpeg references; no fixture regeneration.
    for entry in manifest["images"]:
        width, height = entry["width"], entry["height"]
        source = (fixtures / entry["name"]).read_bytes()
        reference = (fixtures / (entry["name"] + ".rgba")).read_bytes()
        assert hashlib.sha256(source).hexdigest() == entry["input_sha256"]
        assert hashlib.sha256(reference).hexdigest() == entry["reference_sha256"]
        base = decode_png(invoke(binary, source, (0, 0, width, height)))[2]
        assert len(base) == len(reference)
        assert max(abs(a - b) for a, b in zip(base, reference)) <= entry["reference_tolerance"]
        positives += 1
        for orientation in range(1, 9):
            dw, dh, expected = orient(base, width, height, orientation)
            oriented_source = exif(source, orientation)
            for rect in ((0, 0, dw, dh), (dw - 1, dh - 1, 1, 1),
                         (dw // 2, dh // 2, dw - dw // 2, dh - dh // 2)):
                w, h, pixels = decode_png(invoke(binary, oriented_source, rect))
                assert (w, h) == rect[2:]
                assert pixels == select(expected, dw, rect), (entry["name"], orientation, rect)
                positives += 1
    denials = 0
    boundary = png(1024, 1024, b"\x21\x43\x65\x87" * (1024 * 1024))
    assert decode_png(invoke(binary, boundary, (1023, 1023, 1, 1))) == (1, 1, b"\x21\x43\x65\x87")
    positives += 1
    data = png(2, 3, bytes(range(24)))
    for rect in ((0, 0, 0, 1), (0, 0, 1, 0), (2, 0, 1, 1), (0, 3, 1, 1),
                 (-1, 0, 1, 1), ("1e2", 0, 1, 1), (2 ** 64, 0, 1, 1)):
        invoke(binary, data, rect, False)
        denials += 1
    for choices in ([], CHOICES[:1], CHOICES[1:]):
        invoke(binary, data, (0, 0, 1, 1), False, choices)
        denials += 1
    for extra in (("--x", "0"), ("--unknown",), ("--drop-metadata",)):
        invoke(binary, data, (0, 0, 1, 1), False, extra=extra)
        denials += 1
    for stop in range(len(data)):
        invoke(binary, data[:stop], (0, 0, 1, 1), False)
        denials += 1
    for bad in (b"x" * (8 * 1024 * 1024 + 1), b"not an image",
                png(1025, 1024, b"\x21\x43\x65\x87" * (1025 * 1024)),
                png(4097, 1, b"\x21\x43\x65\x87" * 4097),
                (fixtures / "rgba16.png").read_bytes(),
                (fixtures / "cmyk.jpg").read_bytes()):
        invoke(binary, bad, (0, 0, 1, 1), False)
        denials += 1
    print(f"{positives} PNG outputs verified independently (RGBA/alpha exact; JPEG reference tolerance <=3); {denials} explicit denials")


if __name__ == "__main__":
    main()
