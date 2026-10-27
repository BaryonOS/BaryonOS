#!/usr/bin/env python3
"""Refresh screenshot hashes and dimensions from their encoded image formats."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import struct
import tempfile

ROOT = Path(__file__).resolve().parents[1]
VERIFICATION = ROOT / "docs/verification"
INDEX = VERIFICATION / "evidence-files.json"
PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"
JPEG_START = b"\xff\xd8"
JPEG_SOF_MARKERS = {
    0xC0, 0xC1, 0xC2, 0xC3, 0xC5, 0xC6, 0xC7, 0xC9, 0xCA, 0xCB, 0xCD, 0xCE, 0xCF,
}


def image_metadata(data: bytes) -> dict:
    if data.startswith(PNG_SIGNATURE):
        if len(data) < 24 or data[12:16] != b"IHDR":
            raise ValueError("Invalid PNG IHDR")
        width, height = struct.unpack_from(">II", data, 16)
        image_format = "png"
    elif data.startswith(JPEG_START):
        width, height = _jpeg_dimensions(data)
        image_format = "jpeg"
    else:
        raise ValueError("Unsupported image signature")
    if width == 0 or height == 0:
        raise ValueError("Invalid zero-sized image")
    return {"format": image_format, "dimensions": [width, height]}


def _jpeg_dimensions(data: bytes) -> tuple[int, int]:
    offset = 2
    while offset < len(data):
        if data[offset] != 0xFF:
            raise ValueError("Invalid JPEG marker")
        while offset < len(data) and data[offset] == 0xFF:
            offset += 1
        if offset >= len(data):
            break
        marker = data[offset]
        offset += 1
        if marker in (0xD9, 0xDA):
            break
        if marker == 0x01 or 0xD0 <= marker <= 0xD8:
            continue
        if offset + 2 > len(data):
            raise ValueError("Truncated JPEG segment length")
        segment_length = struct.unpack_from(">H", data, offset)[0]
        if segment_length < 2 or offset + segment_length > len(data):
            raise ValueError("Invalid JPEG segment length")
        if marker in JPEG_SOF_MARKERS:
            if segment_length < 8:
                raise ValueError("Truncated JPEG frame header")
            height, width = struct.unpack_from(">HH", data, offset + 3)
            return width, height
        offset += segment_length
    raise ValueError("JPEG frame dimensions not found")


def update_index(index_path: Path = INDEX, verification_dir: Path = VERIFICATION) -> int:
    index = json.loads(index_path.read_text(encoding="utf-8"))
    if not isinstance(index, dict):
        raise ValueError("Evidence index must be a JSON object")
    root = verification_dir.resolve()
    updated = 0
    for relative_path, record in index.items():
        if not relative_path.startswith("screenshots/"):
            continue
        image_path = verification_dir / relative_path
        image_path.resolve().relative_to(root)
        data = image_path.read_bytes()
        record.update({
            "sha256": hashlib.sha256(data).hexdigest(),
            "bytes": len(data),
            **image_metadata(data),
        })
        updated += 1
    if not updated:
        raise ValueError("Evidence index contains no screenshot records")
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(
            "w", encoding="utf-8", dir=index_path.parent,
            prefix=f".{index_path.name}.", suffix=".tmp", delete=False,
        ) as stream:
            temporary = Path(stream.name)
            json.dump(index, stream, indent=2, ensure_ascii=False)
            stream.write("\n")
        os.replace(temporary, index_path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)
    return updated


if __name__ == "__main__":
    print(f"Updated {update_index()} screenshot records in {INDEX}")
