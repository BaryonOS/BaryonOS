#!/usr/bin/env python3
"""H06-T6 removable-media HW-INV image: a deterministic raw GPT disk with one FAT32 ESP.

Packs the ESP directory of a paired native-hw-inventory/hw-inventory build into a
fixed-size raw image (protective MBR, GPT, one EFI system partition formatted as
FAT32 with 8.3 upper-case names only) and records its SHA-256 and byte length so
protocol HW-INV step S1 can compare the medium against the recorded value.

The same ESP payload always yields the same bytes: GUIDs and the volume serial
are derived from the payload, and every timestamp is fixed at 1980-01-01. The
image is only written to a regular file in the build directory; this tool has
no device-writing mode. GPT/FAT32 removable media is an assumption pending owner
decision OD-HW-BOOT. QEMU-tested only; not a physical boot or a hardware record.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import struct
import subprocess
import tempfile
import uuid
import zlib
from pathlib import Path

from common import BUILD, ROOT, sha256_bytes, source_hash
from package_windows_qemu import check_esp

SECTOR = 512
IMAGE_BYTES = 64 * 1024 * 1024
TOTAL = IMAGE_BYTES // SECTOR
PART_START = 2048  # 1 MiB alignment
PART_END = TOTAL - 34  # last usable LBA: backup entries and header follow
PART_SECTORS = PART_END - PART_START + 1
RESERVED = 32
FATS = 2
ESP_TYPE = uuid.UUID("c12a7328-f81f-11d2-ba4b-00a0c93ec93b")
FAT_DATE = (0 << 9) | (1 << 5) | 1  # 1980-01-01, time 00:00:00
EOC = 0x0FFFFFFF
NAME = re.compile(r"[A-Z0-9_\-]{1,8}(\.[A-Z0-9_\-]{1,3})?")


class ImageError(ValueError):
    pass


def _short(name: str) -> bytes:
    """Return the 11-byte 8.3 directory name; fail closed on anything else."""
    upper = name.upper()
    if NAME.fullmatch(upper) is None:
        raise ImageError(f"ESP name {name!r} does not fit an 8.3 upper-case short name")
    base, _, ext = upper.partition(".")
    return base.ljust(8).encode("ascii") + ext.ljust(3).encode("ascii")


def _entry(short: bytes, attr: int, cluster: int, size: int) -> bytes:
    return struct.pack("<11sBBBHHHHHHHI", short, attr, 0, 0, 0, FAT_DATE, FAT_DATE,
                       cluster >> 16, 0, FAT_DATE, cluster & 0xFFFF, size)


def _payload(esp: Path) -> dict:
    """Read the ESP tree as {8.3 name: bytes | dict}; reject links, specials and name clashes."""
    tree: dict = {}
    for child in sorted(esp.iterdir(), key=lambda p: p.name):
        if child.is_symlink() or not (child.is_file() or child.is_dir()):
            raise ImageError(f"{child} is not a regular file or directory")
        key = _short(child.name)
        if key in tree:
            raise ImageError(f"{child} collides with another 8.3 name")
        tree[key] = _payload(child) if child.is_dir() else child.read_bytes()
    return tree


def _manifest(tree: dict, prefix: str = "") -> dict[str, str]:
    files = {}
    for key, value in tree.items():
        base, ext = key[:8].decode().rstrip(), key[8:].decode().rstrip()
        path = prefix + base + ("." + ext if ext else "")
        if isinstance(value, dict):
            files |= _manifest(value, path + "/")
        else:
            files[path] = sha256_bytes(value)
    return files


def _fat_sectors() -> int:
    # Microsoft FAT specification sizing, one sector per cluster.
    return -(-(PART_SECTORS - RESERVED) // ((256 + FATS) // 2))


def build_image(esp: Path) -> bytes:
    return _image(_payload(Path(esp)))


def _image(tree: dict) -> bytes:
    """Pack one in-memory ESP snapshot; every recorded hash must come from the same snapshot."""
    if not tree:
        raise ImageError("empty ESP")
    digest = hashlib.sha256(json.dumps(_manifest(tree), sort_keys=True).encode()).digest()
    disk_guid = uuid.UUID(bytes=digest[:16], version=4)
    part_guid = uuid.UUID(bytes=digest[16:32], version=4)
    serial = struct.unpack("<I", digest[:4])[0]

    fat_size = _fat_sectors()
    data_start = RESERVED + FATS * fat_size
    clusters = PART_SECTORS - data_start
    if not 65525 <= clusters < 0x0FFFFFF5 or (clusters + 2) * 4 > fat_size * SECTOR:
        raise ImageError("geometry is not a valid FAT32 volume")
    fat = [0x0FFFFFF8, 0x0FFFFFFF]
    data = bytearray()

    def allocate(blob: bytes) -> int:
        count = max(1, -(-len(blob) // SECTOR))
        first = len(fat)
        if first - 2 + count > clusters:
            raise ImageError("payload does not fit the image")
        fat.extend(range(first + 1, first + count))
        fat.append(EOC)
        data.extend(blob.ljust(count * SECTOR, b"\0"))
        return first

    # Clusters are assigned depth-first; each directory is reserved before its children.
    def directory(node: dict, is_root: bool, parent: int) -> int:
        entries = len(node) + (0 if is_root else 2)
        cluster = allocate(b"\0" * (max(1, -(-(entries * 32) // SECTOR)) * SECTOR))
        records = []
        if not is_root:  # "." and ".." (0 means the root directory)
            records += [_entry(b".          ", 0x10, cluster, 0), _entry(b"..         ", 0x10, parent, 0)]
        for key, value in node.items():
            if isinstance(value, dict):
                records.append(_entry(key, 0x10, directory(value, False, 0 if is_root else cluster), 0))
            else:
                records.append(_entry(key, 0x20, allocate(value) if value else 0, len(value)))
        offset = (cluster - 2) * SECTOR
        blob = b"".join(records)
        data[offset:offset + len(blob)] = blob
        return cluster

    if directory(tree, True, 0) != 2:
        raise ImageError("root directory must start at cluster 2")
    used = len(fat) - 2

    boot = bytearray(SECTOR)
    boot[0:3] = b"\xEB\x58\x90"
    boot[3:11] = b"BARYONOS"
    struct.pack_into("<HBHBHHBHHHII", boot, 11, SECTOR, 1, RESERVED, FATS, 0, 0, 0xF8, 0, 32, 64,
                     PART_START, PART_SECTORS)
    struct.pack_into("<IHHIHH", boot, 36, fat_size, 0, 0, 2, 1, 6)
    struct.pack_into("<BBBI11s8s", boot, 64, 0x80, 0, 0x29, serial, b"NO NAME    ", b"FAT32   ")
    boot[510:512] = b"\x55\xAA"
    info = bytearray(SECTOR)
    struct.pack_into("<I", info, 0, 0x41615252)
    struct.pack_into("<III", info, 484, 0x61417272, clusters - used, len(fat))
    struct.pack_into("<I", info, 508, 0xAA550000)

    volume = bytearray(PART_SECTORS * SECTOR)
    for base in (0, 6):  # primary and backup boot sector + FSInfo
        volume[base * SECTOR:(base + 1) * SECTOR] = boot
        volume[(base + 1) * SECTOR:(base + 2) * SECTOR] = info
    table = struct.pack(f"<{len(fat)}I", *fat)
    for copy in range(FATS):
        start = (RESERVED + copy * fat_size) * SECTOR
        volume[start:start + len(table)] = table
    volume[data_start * SECTOR:data_start * SECTOR + len(data)] = data

    entry = bytearray(128)
    entry[0:16] = ESP_TYPE.bytes_le
    entry[16:32] = part_guid.bytes_le
    struct.pack_into("<QQQ", entry, 32, PART_START, PART_END, 0)
    entry[56:128] = "BaryonOS HW-INV".encode("utf-16-le").ljust(72, b"\0")
    entries = bytes(entry) + bytes(127 * 128)
    entries_crc = zlib.crc32(entries)

    def header(mine: int, other: int, table_lba: int) -> bytes:
        raw = bytearray(struct.pack("<8sIIIIQQQQ16sQIII", b"EFI PART", 0x00010000, 92, 0, 0, mine, other,
                                    34, PART_END, disk_guid.bytes_le, table_lba, 128, 128, entries_crc))
        struct.pack_into("<I", raw, 16, zlib.crc32(raw))
        return bytes(raw).ljust(SECTOR, b"\0")

    image = bytearray(IMAGE_BYTES)
    image[446:462] = struct.pack("<B3sB3sII", 0, b"\x00\x02\x00", 0xEE, b"\xFF\xFF\xFF", 1, TOTAL - 1)
    image[510:512] = b"\x55\xAA"
    image[SECTOR:2 * SECTOR] = header(1, TOTAL - 1, 2)
    image[2 * SECTOR:34 * SECTOR] = entries
    image[(TOTAL - 33) * SECTOR:(TOTAL - 1) * SECTOR] = entries
    image[(TOTAL - 1) * SECTOR:] = header(TOTAL - 1, 1, TOTAL - 33)
    image[PART_START * SECTOR:(PART_END + 1) * SECTOR] = volume
    return bytes(image)


def checked_esp(build_directory: Path) -> tuple[Path, dict, str]:
    """Return (esp, build manifest, source fingerprint) of the paired HW-INV build."""
    manifest = json.loads((build_directory / "manifest.json").read_text())
    frozen = source_hash()
    if (manifest.get("kernel_feature") != "native-hw-inventory"
            or manifest.get("loader_feature") != "hw-inventory"
            or manifest.get("source_sha256") != frozen):
        raise ImageError("Build the paired native-hw-inventory profile on the current source first")
    try:  # exactly the inspected loader and kernel: no leftover, linked or special entries
        check_esp(build_directory, manifest)
    except (OSError, ValueError) as error:
        raise ImageError(f"HW-INV ESP rejected: {error}") from error
    return build_directory / "esp", manifest, frozen


def _publish(pairs: list[tuple[Path, bytes]]) -> None:
    """Replace each regular file only after every new file is fully written; never follow a link.

    Old outputs are removed before any rename, so a failure leaves no earlier record
    beside new image bytes (or the reverse): either the new pair, a partial set without
    its partner, or nothing.
    """
    temporaries: list[str] = []
    try:
        for path, data in pairs:
            if path.is_symlink() or (path.exists() and not path.is_file()):
                raise ImageError(f"{path} is not a writable regular file")
            fd, name = tempfile.mkstemp(dir=path.parent, prefix=f".{path.name}.")
            temporaries.append(name)
            with os.fdopen(fd, "wb") as stream:
                os.fchmod(stream.fileno(), 0o644)
                stream.write(data)
        for path, _ in pairs:
            path.unlink(missing_ok=True)
        for (path, _), name in zip(pairs, temporaries):
            os.replace(name, path)
    except OSError as error:
        raise ImageError(f"could not write {[str(path) for path, _ in pairs]}: {error}") from error
    finally:
        for name in temporaries:
            try:
                os.unlink(name)
            except FileNotFoundError:
                pass


def write_image(build_directory: Path = BUILD, output: Path | None = None) -> dict:
    """Write the image (default <build>/hw-inventory.img) and its <stem>-image.json record."""
    build_directory = Path(build_directory).resolve(strict=True)
    esp, manifest, frozen = checked_esp(build_directory)
    tree = _payload(esp)  # one read: the image, `files` and the artifact check use this snapshot
    files = _manifest(tree)
    if (files.get("KERNEL.ELF") != manifest["kernel"]["sha256"]
            or files.get("EFI/BOOT/BOOTX64.EFI") != manifest["loader"]["sha256"] or len(files) != 2):
        raise ImageError("HW-INV ESP changed while it was read")
    image = _image(tree)
    output = build_directory / "hw-inventory.img" if output is None else Path(output)

    def git(*arguments: str) -> str:
        return subprocess.check_output(["git", *arguments], cwd=ROOT, text=True).strip()

    record = dict(
        scope="Host-built HW-INV removable-media image; QEMU-tested only, not a physical boot or hardware record",
        image=output.name, image_sha256=sha256_bytes(image), image_bytes=len(image),
        layout=dict(sector=SECTOR, partition_table="gpt", esp_first_lba=PART_START, esp_last_lba=PART_END,
                    filesystem="fat32", sectors_per_cluster=1, names="8.3 upper-case"),
        files=files, kernel_feature=manifest["kernel_feature"],
        loader_feature=manifest["loader_feature"], kernel_sha256=manifest["kernel"]["sha256"],
        loader_sha256=manifest["loader"]["sha256"], source_fingerprint_version=3, source_sha256=frozen,
        source_commit=git("rev-parse", "HEAD"), clean_checkout=git("status", "--porcelain") == "")
    _publish([(output, image), (output.with_name(output.stem + "-image.json"),
                                (json.dumps(record, indent=2, ensure_ascii=False) + "\n").encode("utf-8"))])
    return record


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build-dir", type=Path, default=BUILD)
    args = parser.parse_args()
    result = write_image(args.build_dir)
    print(f"HW-INV image {result['image_sha256']} ({result['image_bytes']} bytes); "
          f"clean_checkout={result['clean_checkout']}")
