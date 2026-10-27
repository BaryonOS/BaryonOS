"""Synthetic readonly M5 volume and external data oracle. No real disk paths."""
from __future__ import annotations
import random
import re
import secrets
from pathlib import Path
from common import BUILD, sha256

DISK_BYTES = 64 * 1024
SECTOR_BYTES = 512


def fnv64(data: bytes) -> str:
    """Diagnostic test checksum, not cryptography or adversarial integrity."""
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return f"{value:016x}"


def create_disk(run_directory: Path, seed: int | None = None):
    run_directory = run_directory.resolve(strict=True)
    run_directory.relative_to((BUILD / "run").resolve())
    if seed is None:
        seed = secrets.randbits(64)
    if not isinstance(seed, int) or not 0 <= seed <= 0xffffffffffffffff:
        raise ValueError("M5 synthetic seed must be u64")
    data = bytearray(random.Random(seed).randbytes(DISK_BYTES))
    # This is not a boot volume: deterministic empty MBR/GPT headers prevent a
    # random partition signature from distracting firmware from the real ESP.
    data[:1024] = bytes(1024)
    header = b"NOS-M5-TEST-VOLUME\0"
    data[:len(header)] = header
    path = run_directory / "test-disk.img"
    if path.exists():
        raise ValueError("Refuse to overwrite an existing M5 volume")
    path.write_bytes(data)
    return path, {"scope": "synthetic readonly per-run raw image; no host disk",
                  "seed": seed, "bytes": DISK_BYTES, "bdf": "00:05.0", "readonly": True,
                  "sha256_before": sha256(path),
                  "expected_digests": {str(sector): fnv64(data[sector*512:(sector+1)*512])
                                       for sector in (4, 6)}}


def verify_disk(path: Path, info: dict, stdout: str, *, data_required: bool):
    data = path.read_bytes()
    expected = {str(sector): fnv64(data[sector*512:(sector+1)*512]) for sector in (4, 6)}
    lines = [line for line in stdout.splitlines() if line.startswith("M5:SECTOR")]
    parsed = [re.fullmatch(r"M5:SECTOR([0-9]+)_DIGEST:([0-9a-fA-F]{16})", line) for line in lines]
    well_formed = all(match is not None for match in parsed)
    matches = [match.groups() for match in parsed if match is not None]
    observed = {sector: value.lower() for sector, value in matches}
    unique = well_formed and len(matches) == len(observed)
    unchanged = len(data) == DISK_BYTES and sha256(path) == info["sha256_before"]
    metadata_matches = expected == info["expected_digests"]
    oracle = (unique and observed == expected) if data_required else not lines
    return {**info, "sha256_after": sha256(path), "unchanged": unchanged,
            "data_required": data_required, "observed_digests": observed,
            "unique_digest_lines": unique, "oracle_passed": oracle,
            "metadata_matches_image": metadata_matches,
            "passed": unchanged and metadata_matches and oracle}
