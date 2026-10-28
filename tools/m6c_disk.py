"""Independent named-document journal oracle; only guarded disposable images.

No Rust codec or native fixture module is imported. FNV is diagnostic only.
The decoder independently enforces canonical v2 records and rename history.
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
from m6c_faults import FAULT_CASES as M6C_FAULT_CASES, outcome_valid

SIZE, SECTOR, FIRST, SLOTS, OBJECTS, VALUE, NAME = 65536, 512, 4, 32, 4, 128, 32
MAGIC = b"NOSM6C02"


def digest(data: bytes) -> str:
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return f"{value:016x}"


def fixture_value(object_id: int, revision: int) -> bytes:
    return bytes((0x31 + object_id * 29 + revision * 7 + index * 3) & 255
                 for index in range(VALUE))


def fixture_name(object_id: int, revision: int) -> bytes:
    names = ("Café A", "Werk A", "Café B", "Werk B") if revision == 1 else (
        "Café hernoemd A", "Werk hernoemd A", "Café hernoemd B", "Werk hernoemd B")
    return names[object_id].encode("utf-8")


def valid_name(name: bytes) -> bool:
    if not 1 <= len(name) <= NAME:
        return False
    try:
        text = name.decode("utf-8", "strict")
    except UnicodeDecodeError:
        return False
    return (text not in (".", "..") and all(
        ord(char) >= 32 and not 127 <= ord(char) <= 159
        and char not in "/\\" and not 0x202a <= ord(char) <= 0x202e
        and not 0x2066 <= ord(char) <= 0x2069 for char in text))


def parse_record(data: bytes, slot: int):
    if len(data) != SECTOR or not 0 <= slot < SLOTS:
        raise ValueError("Invalid M6c sector length/index")
    if data == bytes(SECTOR):
        return None
    fields = struct.unpack_from("<HHIQQQQHHH", data, 8)
    version, header, flags, sequence, request_id, object_id, revision, operation, length, name_length = fields
    if (data[:8] != MAGIC or (version, header, flags) != (2, 64, 0)
            or sequence != slot + 1 or request_id == 0 or object_id >= OBJECTS
            or not 1 <= revision <= sequence or operation not in (2, 3, 4)
            or length > VALUE or name_length > NAME
            or any(data[54:64]) or any(data[64+length:192])
            or any(data[192+name_length:504])
            or struct.unpack_from("<Q", data, 504)[0] != int(digest(data[:504]+bytes(8)), 16)):
        raise ValueError(f"Corrupt occupied M6c journal record {slot}")
    payload, name = data[64:64+length], data[192:192+name_length]
    if operation == 3:
        if payload or name:
            raise ValueError("Tombstone retains live metadata")
    elif not valid_name(name):
        raise ValueError("Invalid document label")
    return {"sequence": sequence, "request_id": request_id, "object": object_id,
            "revision": revision, "operation": operation, "payload": payload, "name": name}


def object_summary(revision=0, present=False, payload=b"", name=b""):
    return dict(revision=revision, present=present, length=len(payload), digest=digest(payload),
                name_length=len(name), name_digest=digest(name))


def scan_records(data: bytes):
    if len(data) != SIZE:
        raise ValueError("M6c volume must stay exactly 64KiB")
    state = {"sequence": 0, "objects": [object_summary() for _ in range(OBJECTS)]}
    payloads = [b""] * OBJECTS
    records, ended = [], False
    for slot in range(SLOTS):
        entry = parse_record(data[(FIRST+slot)*SECTOR:(FIRST+slot+1)*SECTOR], slot)
        if entry is None:
            ended = True
            continue
        if ended:
            raise ValueError("Nonempty record follows an empty journal slot")
        obj, op = entry["object"], entry["operation"]
        old = state["objects"][obj]
        if (entry["revision"] != old["revision"] + 1
                or op in (3, 4) and not old["present"]
                or op == 4 and entry["payload"] != payloads[obj]):
            raise ValueError("Invalid revision/delete/rename predecessor or changed rename content")
        state["objects"][obj] = object_summary(entry["revision"], op != 3, entry["payload"], entry["name"])
        payloads[obj] = entry["payload"]
        state["sequence"] = entry["sequence"]
        records.append(entry)
    return state, records


def scan(data: bytes):
    return scan_records(data)[0]


def legacy_record() -> bytes:
    """One independently encoded, otherwise valid M6b-v1 record, not random damage."""
    record = bytearray(SECTOR)
    record[:8] = b"NOSM6B01"
    struct.pack_into("<HHIQQQQHH", record, 8, 1, 64, 0, 1, 1, 0, 1, 2, VALUE)
    record[64:192] = fixture_value(0, 1)
    struct.pack_into("<Q", record, 504, int(digest(record), 16))
    return bytes(record)


def synthetic_initial(seed: int, variant="normal") -> bytes:
    if type(seed) is not int or not 0 <= seed < 1 << 64:
        raise ValueError("M6c seed must be u64")
    if variant not in ("normal", "corrupt", "legacy"):
        raise ValueError("Unknown M6c fixture variant")
    data = bytearray(random.Random(seed).randbytes(SIZE))
    data[:1024] = bytes(1024)
    label = b"NOS-M6C-TEST-VOLUME\0"
    data[:len(label)] = label
    data[FIRST*SECTOR:(FIRST+SLOTS)*SECTOR] = bytes(SLOTS*SECTOR)
    if variant == "corrupt":
        data[FIRST*SECTOR] = 0x80
    elif variant == "legacy":
        data[FIRST*SECTOR:(FIRST+1)*SECTOR] = legacy_record()
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
            raise ValueError("M6c volume identity/path changed")
        info = self.path.stat()
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or info.st_size != SIZE:
            raise ValueError("M6c requires its own regular single-link 64KiB test image")
        saved = json.loads((folder / "volume.json").read_text(encoding="utf-8"))
        expected = {"scope": "fresh synthetic M6c volume", "identity": self.identity,
                    "seed": self.seed, "variant": self.variant, "bytes": SIZE,
                    "bdf": "00:05.0", "initial_sha256": hashlib.sha256(self.initial).hexdigest()}
        if (saved != expected or type(saved.get("seed")) is not int
                or self.initial != synthetic_initial(self.seed, self.variant)):
            raise ValueError("Not this harness-created M6c volume")
        return self.path


def create_volume(label: str, *, variant="normal", seed=None) -> Volume:
    if not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,60}", label):
        raise ValueError("M6c labels cannot contain paths")
    if seed is None:
        seed = secrets.randbits(64)
    initial = synthetic_initial(seed, variant)
    parent = (BUILD / "run").resolve()
    parent.mkdir(parents=True, exist_ok=True)
    identity = uuid.uuid4().hex
    directory = parent / f"m6c-volume-{label}-{identity[:10]}"
    directory.mkdir()
    with (directory / "test-disk.img").open("xb") as output:
        output.write(initial)
    save_json(directory / "volume.json", {"scope": "fresh synthetic M6c volume", "identity": identity,
        "seed": seed, "variant": variant, "bytes": SIZE, "bdf": "00:05.0",
        "initial_sha256": hashlib.sha256(initial).hexdigest()})
    volume = Volume(directory, seed, initial, variant, identity)
    volume.guard()
    return volume


def state_markers(stdout: str):
    specs = {"M6C:SEQUENCE": r"[0-9]{1,2}", "M6C:ACKS": r"[0-9]{1,2}",
             "M6C:REJECTED_NO_IO": r"[0-9]{1,3}", "M6C:CATALOG_REPLIES": r"[0-9]{1,3}",
             "M6C:DOMAIN0:REVISION": r"[0-9]{1,2}", "M6C:DOMAIN1:REVISION": r"[0-9]{1,2}"}
    fields = (("REVISION", r"[0-9]{1,2}"), ("PRESENT", "[01]"),
              ("LENGTH", r"[0-9]{1,3}"), ("DIGEST", "[0-9a-f]{16}"),
              ("NAME_LENGTH", r"[0-9]{1,2}"), ("NAME_DIGEST", "[0-9a-f]{16}"))
    for object_id in range(OBJECTS):
        for field, regex in fields:
            specs[f"M6C:OBJECT{object_id}:{field}"] = regex
    observed = {}
    prefixes = ("M6C:SEQUENCE", "M6C:OBJECT", "M6C:ACKS", "M6C:REJECTED_NO_IO", "M6C:CATALOG_REPLIES", "M6C:DOMAIN")
    for line in stdout.splitlines():
        if line.startswith(prefixes):
            key, sep, value = line.rpartition(":")
            if not sep or key not in specs or key in observed or not re.fullmatch(specs[key], value):
                raise ValueError("Unknown/duplicate/malformed M6c object evidence")
            observed[key] = value
    if not observed:
        return None
    if set(observed) != set(specs):
        raise ValueError("Incomplete M6c object evidence")
    objects = [dict(revision=int(observed[f"M6C:OBJECT{i}:REVISION"]),
                    present=observed[f"M6C:OBJECT{i}:PRESENT"] == "1",
                    length=int(observed[f"M6C:OBJECT{i}:LENGTH"]),
                    digest=observed[f"M6C:OBJECT{i}:DIGEST"],
                    name_length=int(observed[f"M6C:OBJECT{i}:NAME_LENGTH"]),
                    name_digest=observed[f"M6C:OBJECT{i}:NAME_DIGEST"]) for i in range(OBJECTS)]
    return {"state": {"sequence": int(observed["M6C:SEQUENCE"]), "objects": objects},
            "acks": int(observed["M6C:ACKS"]), "rejected": int(observed["M6C:REJECTED_NO_IO"]),
            "domain_revisions": [int(observed[f"M6C:DOMAIN{i}:REVISION"]) for i in range(2)],
            "catalog_replies": int(observed["M6C:CATALOG_REPLIES"])}


def verify_volume(volume: Volume, stdout: str, *, case: str, before: bytes):
    if case not in ("normal", "recover", "errors", "full", "corrupt", "legacy",
                    "vm-cut-before-rename", "vm-cut-after-rename") + M6C_FAULT_CASES:
        raise ValueError("Unknown M6c oracle case")
    data = volume.guard().read_bytes()
    low, high = FIRST*SECTOR, (FIRST+SLOTS)*SECTOR
    checks = {"outside_journal_unchanged": data[:low] == volume.initial[:low]
              and data[high:] == volume.initial[high:]}
    checks["case_matches_volume_variant"] = volume.variant == (
        case if case in ("corrupt", "legacy") else "normal")
    checks["fresh_mutator_before_image"] = case == "recover" or before == volume.initial
    try:
        state, records = scan_records(data)
        corrupt = False
    except ValueError:
        state, records, corrupt = None, [], True
    checks["journal_valid_or_explicit_rejection_case"] = corrupt == (case in ("corrupt", "legacy"))
    try:
        # The uncertain service cannot claim the disk's latest state. Keep its
        # counters separate from successful GET/LIST/recovery state evidence.
        guest_state = "\n".join(line for line in stdout.splitlines() if not (
            case in M6C_FAULT_CASES and line.startswith((
                "M6C:ACKS", "M6C:REJECTED_NO_IO", "M6C:CATALOG_REPLIES"))))
        observed = state_markers(guest_state)
        no_values = case in ("corrupt", "legacy", "vm-cut-before-rename", "vm-cut-after-rename") + M6C_FAULT_CASES
        checks["guest_matches_independent_disk_state"] = observed is None if no_values else (
            observed is not None and observed["state"] == state)
    except ValueError:
        observed = None
        checks["guest_matches_independent_disk_state"] = False
    checks["read_only_stage_unchanged"] = case not in ("recover", "corrupt", "legacy") or data == before
    checks["record_payloads_and_names_match_native_scenario"] = all(
        entry["payload"] == (b"" if entry["operation"] == 3 else fixture_value(
            entry["object"], entry["revision"] - (1 if entry["operation"] == 4 else 0)))
        and entry["name"] == (b"" if entry["operation"] == 3 else fixture_name(entry["object"], entry["revision"]))
        for entry in records)
    if state is not None:
        seq, objects = state["sequence"], state["objects"]
        if case in ("normal", "errors"):
            expected = [object_summary(3, True, fixture_value(i, 3), fixture_name(i, 3))
                        if i % 2 == 0 else object_summary(2) for i in range(OBJECTS)]
            checks["expected_final_state"] = seq == 10 and objects == expected
        elif case == "full":
            expected = [object_summary(16, True, fixture_value(i, 16), fixture_name(i, 16))
                        if i % 2 == 0 else object_summary() for i in range(OBJECTS)]
            checks["expected_final_state"] = seq == 32 and objects == expected
        elif case == "recover":
            checks["expected_final_state"] = state == scan(before)
        elif case in ("vm-cut-before-rename", "vm-cut-after-rename"):
            revision = 1 if case == "vm-cut-before-rename" else 2
            expected = [object_summary(revision, True, fixture_value(0, 1), fixture_name(0, revision)),
                        *[object_summary() for _ in range(3)]]
            checks["expected_final_state"] = seq == revision and objects == expected
        elif case in M6C_FAULT_CASES:
            possible = ((1,) if case == "bad-descriptor" else (1, 2)
                        if case in ("write-timeout", "reset-timeout") else (2,))
            expected = [object_summary(seq, True, fixture_value(0, 1), fixture_name(0, seq)),
                        *[object_summary() for _ in range(3)]]
            checks["fault_prefix_matches_publication_boundary"] = seq in possible and objects == expected
            checks["fault_records_are_only_confirmed_put_and_optional_rename"] = (
                [(r["sequence"], r["request_id"], r["object"], r["revision"], r["operation"]) for r in records]
                == [(1, 10, 0, 1, 2), (2, 11, 0, 2, 4)][:seq] and seq in possible)
    if case in M6C_FAULT_CASES:
        checks["uncertainty_has_no_success_ack_or_replay"] = outcome_valid(stdout, case)
    if observed:
        expected_acks = 32 if case == "full" else 10 if case in ("normal", "errors") else 0
        expected_lists = 4 if case == "full" else 10 if case in ("normal", "errors") else 2
        checks["exact_mutation_ack_count"] = observed["acks"] == expected_acks
        checks["exact_catalog_reply_count"] = observed["catalog_replies"] == expected_lists
        checks["domain_revisions_exclude_foreign_mutations"] = state is not None and observed["domain_revisions"] == [
            sum(obj["revision"] for obj in state["objects"][start:start+2]) for start in (0, 2)]
        if case in ("normal", "errors", "full"):
            checks["all_negative_requests_without_io"] = observed["rejected"] == {
                "normal": 6, "errors": 80, "full": 8}[case]
    return {"scope": "independent Python v2 parser over actual synthetic 64KiB volume",
            "identity": volume.identity, "seed": volume.seed, "variant": volume.variant,
            "sha256_initial": hashlib.sha256(volume.initial).hexdigest(), "sha256_after": sha256(volume.path),
            "state": state, "observed": observed, "checks": checks, "passed": all(checks.values())}
