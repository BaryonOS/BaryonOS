"""Independent M6b journal oracle; restricted disposable volumes only.

No Rust codec imports. FNV diagnoses corruption, not authenticity.
Host seeds vary canaries, not native document data.
"""
from __future__ import annotations
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import random
import re
import secrets
import stat
import struct
import uuid
from common import BUILD, save_json, sha256

SIZE, SECTOR, FIRST, SLOTS, OBJECTS, VALUE = 65536, 512, 4, 32, 4, 128
MAGIC = b"NOSM6B01"


def digest(data: bytes) -> str:
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return f"{value:016x}"


def fixture_value(object_id: int, revision: int) -> bytes:
    return bytes((0x31 + object_id * 29 + revision * 7 + index * 3) & 255
                 for index in range(VALUE))


def parse_record(data: bytes, slot: int):
    if len(data) != SECTOR or not 0 <= slot < SLOTS:
        raise ValueError("Invalid M6b sector length/index")
    if data == bytes(SECTOR):
        return None
    version, header, flags, sequence, request_id, object_id, revision, operation, length = (
        struct.unpack_from("<HHIQQQQHH", data, 8))
    check = data[:504] + bytes(8)
    if (data[:8] != MAGIC or (version, header, flags) != (1, 64, 0)
            or sequence != slot + 1 or request_id == 0 or object_id >= OBJECTS
            or not 1 <= revision <= sequence or operation not in (2, 3)
            or length > VALUE or operation == 3 and length != 0
            or any(data[52:64]) or any(data[64+length:504])
            or struct.unpack_from("<Q", data, 504)[0] != int(digest(check), 16)):
        raise ValueError(f"Corrupt occupied M6b journal record {slot}")
    return {"sequence": sequence, "request_id": request_id, "object": object_id,
            "revision": revision, "operation": operation, "payload": data[64:64+length]}


def scan_records(data: bytes):
    if len(data) != SIZE:
        raise ValueError("M6b volume must stay exactly 64KiB")
    state = {"sequence": 0, "objects": [dict(revision=0, present=False, length=0,
                digest=digest(b"")) for _ in range(OBJECTS)]}
    records, ended = [], False
    for slot in range(SLOTS):
        entry = parse_record(data[(FIRST+slot)*SECTOR:(FIRST+slot+1)*SECTOR], slot)
        if entry is None:
            ended = True
            continue
        if ended:
            raise ValueError("Nonempty record follows an empty journal slot")
        old = state["objects"][entry["object"]]
        if (entry["revision"] != old["revision"] + 1
                or entry["operation"] == 3 and not old["present"]):
            raise ValueError("Invalid per-object revision/delete transition")
        state["objects"][entry["object"]] = dict(revision=entry["revision"],
            present=entry["operation"] == 2, length=len(entry["payload"]),
            digest=digest(entry["payload"]))
        state["sequence"] = entry["sequence"]
        records.append(entry)
    return state, records


def scan(data: bytes):
    return scan_records(data)[0]


def synthetic_initial(seed: int, variant="normal") -> bytes:
    if type(seed) is not int or not 0 <= seed < 1 << 64:
        raise ValueError("M6b seed must be u64")
    if variant not in ("normal", "corrupt"):
        raise ValueError("Unknown M6b fixture variant")
    data = bytearray(random.Random(seed).randbytes(SIZE))
    data[:1024] = bytes(1024)
    label = b"NOS-M6B-TEST-VOLUME" + bytes(1)
    data[:len(label)] = label
    data[FIRST*SECTOR:(FIRST+SLOTS)*SECTOR] = bytes(SLOTS*SECTOR)
    if variant == "corrupt":
        data[FIRST*SECTOR] = 0x80
    return bytes(data)


@dataclass(frozen=True)
class Volume:
    directory: Path
    seed: int
    initial: bytes
    variant: str
    identity: str

    @property
    def path(self):
        return self.directory / "test-disk.img"

    def guard(self):
        folder = self.directory.resolve(strict=True)
        folder.relative_to((BUILD / "run").resolve(strict=True))
        if folder != self.directory or self.directory.is_symlink() or self.path.is_symlink():
            raise ValueError("M6b volume identity/path changed")
        info = self.path.stat()
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or info.st_size != SIZE:
            raise ValueError("M6b requires its own regular single-link 64KiB test image")
        saved = json.loads((folder / "volume.json").read_text(encoding="utf-8"))
        expected = {"scope": "fresh synthetic M6b volume", "identity": self.identity,
                    "seed": self.seed, "variant": self.variant, "bytes": SIZE,
                    "bdf": "00:05.0", "initial_sha256": hashlib.sha256(self.initial).hexdigest()}
        if (saved != expected or type(saved.get("seed")) is not int
                or self.initial != synthetic_initial(self.seed, self.variant)):
            raise ValueError("Not this harness-created M6b volume")
        return self.path


def create_volume(label: str, *, variant="normal", seed=None) -> Volume:
    if not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,60}", label):
        raise ValueError("M6b labels cannot contain paths")
    if seed is None:
        seed = secrets.randbits(64)
    initial = synthetic_initial(seed, variant)
    parent = (BUILD / "run").resolve()
    parent.mkdir(parents=True, exist_ok=True)
    identity = uuid.uuid4().hex
    directory = parent / f"m6b-volume-{label}-{identity[:10]}"
    directory.mkdir()
    with (directory / "test-disk.img").open("xb") as output:
        output.write(initial)
    save_json(directory / "volume.json", {"scope": "fresh synthetic M6b volume", "identity": identity,
        "seed": seed, "variant": variant, "bytes": SIZE, "bdf": "00:05.0",
        "initial_sha256": hashlib.sha256(initial).hexdigest()})
    volume = Volume(directory, seed, initial, variant, identity)
    volume.guard()
    return volume


def state_markers(stdout: str):
    specs = {"M6B:SEQUENCE": r"[0-9]{1,2}", "M6B:ACKS": r"[0-9]{1,2}",
             "M6B:REJECTED_NO_IO": r"[0-9]{1,3}"}
    for object_id in range(OBJECTS):
        for field, regex in (("REVISION", r"[0-9]{1,2}"), ("PRESENT", "[01]"),
                             ("LENGTH", r"[0-9]{1,3}"), ("DIGEST", "[0-9a-f]{16}")):
            specs[f"M6B:OBJECT{object_id}:{field}"] = regex
    observed = {}
    for line in stdout.splitlines():
        if line.startswith(("M6B:SEQUENCE", "M6B:OBJECT", "M6B:ACKS", "M6B:REJECTED_NO_IO")):
            key, sep, value = line.rpartition(":")
            if not sep or key not in specs or key in observed or not re.fullmatch(specs[key], value):
                raise ValueError("Unknown/duplicate/malformed M6b object evidence")
            observed[key] = value
    if not observed:
        return None
    if set(observed) != set(specs):
        raise ValueError("Incomplete M6b object evidence")
    objects = [dict(revision=int(observed[f"M6B:OBJECT{i}:REVISION"]),
                    present=observed[f"M6B:OBJECT{i}:PRESENT"] == "1",
                    length=int(observed[f"M6B:OBJECT{i}:LENGTH"]),
                    digest=observed[f"M6B:OBJECT{i}:DIGEST"]) for i in range(OBJECTS)]
    return {"state": {"sequence": int(observed["M6B:SEQUENCE"]), "objects": objects},
            "acks": int(observed["M6B:ACKS"]), "rejected": int(observed["M6B:REJECTED_NO_IO"])}


def verify_volume(volume: Volume, stdout: str, *, case: str, before: bytes):
    data = volume.guard().read_bytes()
    low, high = FIRST*SECTOR, (FIRST+SLOTS)*SECTOR
    checks = {"outside_journal_unchanged": data[:low] == volume.initial[:low]
              and data[high:] == volume.initial[high:]}
    try:
        state, records = scan_records(data)
        corrupt = False
    except ValueError:
        state, records, corrupt = None, [], True
    checks["journal_valid_or_explicit_corrupt_case"] = corrupt == (case == "corrupt")
    try:
        observed = state_markers(stdout)
        no_values = case in ("corrupt", "vm-cut-before-ack", "vm-cut-after-flush")
        checks["guest_matches_independent_disk_state"] = observed is None if no_values else (
            observed is not None and observed["state"] == state)
    except ValueError:
        observed = None
        checks["guest_matches_independent_disk_state"] = False
    checks["read_only_stage_unchanged"] = case not in ("recover", "corrupt", "vm-cut-before-ack") or data == before
    checks["record_payloads_match_native_scenario"] = all(
        entry["payload"] == (fixture_value(entry["object"], entry["revision"])
                             if entry["operation"] == 2 else b"") for entry in records)
    if state is not None:
        seq, objects = state["sequence"], state["objects"]
        if case in ("normal", "errors"):
            checks["expected_final_state"] = seq == 8 and all(
                obj == dict(revision=2, present=i % 2 == 0, length=128 if i % 2 == 0 else 0,
                            digest=digest(fixture_value(i, 2) if i % 2 == 0 else b""))
                for i, obj in enumerate(objects))
        elif case == "full":
            checks["expected_final_state"] = seq == 32 and all(
                obj == dict(revision=16 if i % 2 == 0 else 0, present=i % 2 == 0,
                            length=128 if i % 2 == 0 else 0,
                            digest=digest(fixture_value(i, 16) if i % 2 == 0 else b""))
                for i, obj in enumerate(objects))
        elif case == "recover":
            checks["expected_final_state"] = state == scan(before)
        elif case == "vm-cut-before-ack":
            checks["expected_final_state"] = seq == 0
        elif case == "vm-cut-after-flush":
            checks["expected_final_state"] = (seq == 1 and records[0]["object"] in (0, 2)
                                                and records[0]["revision"] == 1)
        else:
            checks["expected_final_state"] = False
    if observed:
        expected_acks = 32 if case == "full" else 8 if case in ("normal", "errors") else 0
        checks["exact_mutation_ack_count"] = observed["acks"] == expected_acks
        if case == "errors":
            checks["all_negative_requests_without_io"] = observed["rejected"] == 34
    return {"scope": "independent Python parser over actual synthetic 64KiB volume",
            "identity": volume.identity, "seed": volume.seed, "variant": volume.variant,
            "sha256_initial": hashlib.sha256(volume.initial).hexdigest(), "sha256_after": sha256(volume.path),
            "state": state, "observed": observed, "checks": checks, "passed": all(checks.values())}
