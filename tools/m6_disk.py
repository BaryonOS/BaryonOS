"""M6a synthetic volume, independent journal parser and external byte oracle.

No public API accepts an existing arbitrary disk. The runner consumes only a
fresh Volume made here, and recovery reuses that exact guarded image.
"""
from __future__ import annotations
from dataclasses import dataclass
import hashlib
from pathlib import Path
import random
import re
import secrets
import struct
import uuid
from common import BUILD, save_json, sha256

SIZE = 65536
SECTOR = 512
FIRST = 4
SLOTS = 4
MAGIC = b"NOSM6A01"
VALUE_SIZE = 128
SCRATCH = 12


def digest(data: bytes) -> str:
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return f"{value:016x}"


def next_value(previous: bytes, generation: int) -> bytes:
    if len(previous) != VALUE_SIZE or not 2 <= generation <= SLOTS:
        raise ValueError("invalid bounded synthetic transaction")
    return bytes((byte + 0x31 + generation) & 255 for byte in previous)


def record(slot: int, value: bytes) -> bytes:
    if not 0 <= slot < SLOTS or len(value) != VALUE_SIZE:
        raise ValueError("invalid M6a record")
    data = bytearray(SECTOR)
    data[:8] = MAGIC
    struct.pack_into("<HHIQQII", data, 8, 1, 64, 0, slot + 1, slot + 1, VALUE_SIZE, slot)
    data[64:192] = value
    struct.pack_into("<Q", data, 504, int(digest(data), 16))
    return bytes(data)


def parse_record(data: bytes, slot: int):
    """Independent Python oracle; never imports or invokes the Rust codec."""
    if len(data) != SECTOR or not 0 <= slot < SLOTS or data[:8] != MAGIC:
        return None
    version, header, flags, generation, request_id, length, stored_slot = struct.unpack_from("<HHIQQII", data, 8)
    if ((version, header, flags, generation, length, stored_slot) != (1, 64, 0, slot + 1, VALUE_SIZE, slot)
            or request_id == 0 or any(data[40:64]) or any(data[192:504])):
        return None
    saved = struct.unpack_from("<Q", data, 504)[0]
    check = data[:504] + bytes(8)
    if int(digest(check), 16) != saved:
        return None
    return {"generation": generation, "request_id": request_id,
            "value_digest": digest(data[64:192]), "payload": data[64:192]}


def scan(data: bytes):
    if len(data) != SIZE:
        raise ValueError("M6a volume must stay exactly 64KiB")
    records = [parse_record(data[(FIRST + i)*SECTOR:(FIRST + i + 1)*SECTOR], i) for i in range(SLOTS)]
    return next((entry for entry in reversed(records) if entry is not None), None)


@dataclass(frozen=True)
class Volume:
    directory: Path
    seed: int
    initial: bytes
    variant: str
    scratch: bool
    identity: str

    @property
    def path(self):
        return self.directory / "test-disk.img"

    def guard(self):
        folder = self.directory.resolve(strict=True)
        folder.relative_to((BUILD / "run").resolve())
        if folder != self.directory or self.directory.is_symlink() or self.path.is_symlink():
            raise ValueError("M6a volume identity/path changed")
        manifest = self.directory / "volume.json"
        import json
        saved = json.loads(manifest.read_text(encoding="utf-8"))
        if (saved.get("identity") != self.identity or saved.get("seed") != self.seed
                or type(saved.get("seed")) is not int or saved.get("variant") != self.variant
                or saved.get("scratch") is not self.scratch or saved.get("bytes") != SIZE
                or saved.get("bdf") != "00:05.0"
                or saved.get("initial_sha256") != hashlib.sha256(self.initial).hexdigest()
                or saved.get("scope") != "fresh synthetic M6a volume"
                or self.path.stat().st_size != SIZE or len(self.initial) != SIZE):
            raise ValueError("Not this harness-created fixed-size M6a volume")
        return self.path

    def expected_records(self):
        initial = parse_record(self.initial[FIRST*SECTOR:(FIRST+1)*SECTOR], 0)
        if initial is None:
            return []
        value = initial["payload"]
        result = [record(0, value)]
        for generation in range(2, SLOTS + 1):
            value = next_value(value, generation)
            result.append(record(generation - 1, value))
        return result


def synthetic_initial(seed, variant="normal"):
    """Reconstruct host fixture bytes without opening or formatting any image."""
    if variant not in ("normal", "torn-tail", "corrupt-all"):
        raise ValueError("Unknown M6a synthetic initialization")
    if type(seed) is not int or not 0 <= seed < 1 << 64:
        raise ValueError("M6a seed must be u64")
    rng = random.Random(seed)
    data = bytearray(rng.randbytes(SIZE))
    data[:1024] = bytes(1024)
    header = b"NOS-M6A-TEST-VOLUME\0"
    data[:len(header)] = header
    data[FIRST*SECTOR:(FIRST+SLOTS)*SECTOR] = bytes(SLOTS*SECTOR)
    value = rng.randbytes(VALUE_SIZE)
    data[FIRST*SECTOR:(FIRST+1)*SECTOR] = record(0, value)
    if variant == "torn-tail":
        new = record(1, next_value(value, 2))
        torn = new[:256] + bytes(256)
        if parse_record(torn, 1) is not None:
            raise ValueError("Synthetic prefix unexpectedly forms a valid record")
        data[(FIRST+1)*SECTOR:(FIRST+2)*SECTOR] = torn
    elif variant == "corrupt-all":
        data[FIRST*SECTOR] ^= 0x80
    return bytes(data)


def create_volume(label: str, *, variant="normal", scratch=False, seed=None) -> Volume:
    if not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,60}", label):
        raise ValueError("M6a labels cannot contain paths")
    if seed is None:
        seed = secrets.randbits(64)
    initial = synthetic_initial(seed, variant)
    parent = (BUILD / "run").resolve()
    parent.mkdir(parents=True, exist_ok=True)
    identity = uuid.uuid4().hex
    directory = parent / f"m6-volume-{label}-{identity[:10]}"
    directory.mkdir()
    path = directory / "test-disk.img"
    with path.open("xb") as output:
        output.write(initial)
    save_json(directory / "volume.json", {"scope": "fresh synthetic M6a volume", "identity": identity,
              "seed": seed, "initial_sha256": hashlib.sha256(initial).hexdigest(), "variant": variant,
              "bytes": SIZE, "scratch": bool(scratch), "bdf": "00:05.0"})
    volume = Volume(directory, seed, initial, variant, bool(scratch), identity)
    volume.guard()
    return volume


def value_markers(stdout: str):
    specs = {"generation": ("M6A:GENERATION:", r"M6A:GENERATION:([1-4])"),
             "request_id": ("M6A:REQUEST_ID:", r"M6A:REQUEST_ID:([0-9]{1,20})"),
             "value_digest": ("M6A:VALUE_DIGEST:", r"M6A:VALUE_DIGEST:([0-9a-fA-F]{16})")}
    parsed = {}
    for name, (prefix, pattern) in specs.items():
        lines = [line for line in stdout.splitlines() if line.startswith(prefix.rstrip(":"))]
        if not lines:
            continue
        if len(lines) != 1 or (match := re.fullmatch(pattern, lines[0])) is None:
            raise ValueError("Duplicate/malformed M6a value evidence")
        parsed[name] = match[1].lower() if name == "value_digest" else int(match[1])
    if parsed and set(parsed) != set(specs):
        raise ValueError("Incomplete M6a value evidence")
    return parsed


def verify_volume(volume: Volume, stdout: str, *, allowed_generations, data_required=True,
                  unchanged=False, before=None, allow_torn=False, require_full_prefix=False):
    data = volume.guard().read_bytes()
    selected = scan(data)
    allowed_generations = set(allowed_generations)
    observed_generation = selected["generation"] if selected else None
    canaries = True
    for sector in range(SIZE // SECTOR):
        if FIRST <= sector < FIRST + SLOTS or volume.scratch and sector == SCRATCH:
            continue
        start = sector * SECTOR
        canaries &= data[start:start+SECTOR] == volume.initial[start:start+SECTOR]
    expected = volume.expected_records()
    records_valid = True
    for slot in range(SLOTS):
        at = (FIRST + slot) * SECTOR
        current, original = data[at:at+SECTOR], volume.initial[at:at+SECTOR]
        if current == original:
            continue
        if not expected:
            records_valid = False
            continue
        valid = current == expected[slot]
        if allow_torn and slot == 1:
            valid |= all(byte in (old, new) for byte, old, new in zip(current, original, expected[slot]))
        records_valid &= valid
    scratch_valid = (not volume.scratch or data[SCRATCH*SECTOR:(SCRATCH+1)*SECTOR]
                     == bytes([0x3c])*255 + bytes([0xa7])*257)
    prefix_valid = not require_full_prefix or len(expected) == SLOTS and all(
        data[(FIRST+slot)*SECTOR:(FIRST+slot+1)*SECTOR] == expected[slot] for slot in range(SLOTS))
    try:
        markers = value_markers(stdout)
        marker_valid = (selected is not None and markers == {key: selected[key] for key in
                        ("generation", "request_id", "value_digest")}) if data_required else not markers
    except ValueError:
        markers, marker_valid = {}, False
    stable = not unchanged or data == (volume.initial if before is None else before)
    checks = {"outside_journal_unchanged": bool(canaries), "journal_records_bounded": bool(records_valid),
              "scratch_snapshot_exact": bool(scratch_valid), "allowed_generation": observed_generation in allowed_generations,
              "required_committed_prefix": bool(prefix_valid),
              "guest_value_matches_actual_record": bool(marker_valid), "required_unchanged": bool(stable)}
    return {"scope": "actual synthetic 64KiB bytes; independent Python journal oracle",
            "volume_directory": str(volume.directory), "identity": volume.identity, "seed": volume.seed,
            "variant": volume.variant, "scratch": volume.scratch, "bytes": len(data),
            "sha256_initial": hashlib.sha256(volume.initial).hexdigest(), "sha256_after": sha256(volume.path),
            "allowed_generations": sorted(allowed_generations, key=lambda x: -1 if x is None else x),
            "data_required": data_required, "allow_torn": allow_torn, "unchanged_required": unchanged,
            "require_full_prefix": require_full_prefix,
            "actual_generation": observed_generation, "observed": markers, "checks": checks,
            "passed": all(checks.values())}
