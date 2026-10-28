#!/usr/bin/env python3
"""Independent read-only audit of Nieuw OS M6d native evidence.

Only Python's standard library is imported. No repository parser, runner,
capture helper or source-fingerprint helper is imported or executed. This
program never builds a binary, starts a VM, changes a disk or writes in the
repository. The audit outputs JSON and journal-sector hex only.

Audit existing evidence:
  python3 -B r09_audit_m6d.py --repo /path/to/nieuw-os --out /tmp/r09-audit
Reconstruct the exported image hashes without a repository or binary files:
  python3 -B r09_audit_m6d.py --reconstruct /path/to/m6d-actual-journal-records.json
Exercise this audit's own rejection checks, without claiming native evidence:
  python3 -B r09_audit_m6d.py --self-test
"""
from __future__ import annotations

import argparse
import collections
import hashlib
import json
import math
import random
import re
import stat
import struct
import subprocess
import sys
import unicodedata
from datetime import datetime, timezone
from pathlib import Path

SOURCE_COMMIT = "88e6c04be218b8b09ea4f0a226f6b9719da706a6"
SOURCE_SHA256 = "8ca6d5d5291fde62f91fb47dca7df4d058424604449fb32b9e5899b1d4fbef93"
SIZE, SECTOR, FIRST, SLOTS = 65536, 512, 4, 32
LOW, HIGH = FIRST * SECTOR, (FIRST + SLOTS) * SECTOR
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")
FAULTS = ("write-error", "flush-error", "write-timeout", "flush-timeout",
          "reset-timeout", "bad-descriptor", "used-write", "used-flush")
CUTS = {"vm-cut-before-rename": 1, "vm-cut-after-rename": 2}
FAULT_PREFIXES = ("M6C:FAULT_INJECTED", "M6C:OUTCOME", "M6C:UNKNOWN_REPLIES",
                  "M6C:NO_REPLAY", "M6C:POST_UNKNOWN", "M6C:LIVE_SEQUENCE",
                  "M6C:QUARANTINE")
STATE_PREFIXES = ("M6C:SEQUENCE", "M6C:OBJECT", "M6C:ACKS", "M6C:REJECTED_NO_IO",
                  "M6C:CATALOG_REPLIES", "M6C:DOMAIN")
M6C_ARTIFACTS = frozenset(("esp/EFI/BOOT/BOOTX64.EFI", "esp/kernel.elf",
    "OVMF_CODE.fd", "OVMF_VARS.template.fd", "OVMF_VARS.fd",
    "test-disk.before.img", "test-disk.after.img", "build-manifest.json",
    "volume-manifest.json"))
M1_ARTIFACTS = frozenset(("esp/EFI/BOOT/BOOTX64.EFI", "esp/kernel.elf",
                        "OVMF_CODE.fd", "OVMF_VARS.template.fd"))
HOST_GROUPS = {
    "strict-development-inventory": ("m6c-preflight", "python",
        ("-B", "tools/preflight.py", "--strict")),
    "catalog-core-and-wire-host-tests": ("m6c-host-tests", "cargo",
        ("+1.90.0", "test", "--locked", "--offline", "-p", "catalog-core")),
    "external-oracle-and-crash-harness-tests": ("m6c-harness-tests", "python",
        ("-B", "-m", "unittest", "discover", "-s", "tests", "-p", "test_m6c_*.py")),
}
HOST_SUFFIXES = (".stdout.txt", ".json", ".stderr.txt")
ARCHIVE_FILES = frozenset({"suite.json", "guest-runs.json", "m6c-two-image-check.json"}
    | {stem + suffix for stem, _, _ in HOST_GROUPS.values() for suffix in HOST_SUFFIXES})


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def filehash(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def unique_json_pairs(pairs):
    output = {}
    for key, value in pairs:
        require(key not in output, f"duplicate JSON key: {key}")
        output[key] = value
    return output


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"),
                      object_pairs_hook=unique_json_pairs)


def save(path, data):
    Path(path).write_text(json.dumps(data, ensure_ascii=False, indent=2,
                                    allow_nan=False) + "\n", encoding="utf-8")


def regular(path, base=None, single_link=False):
    path = Path(path)
    require(path.is_absolute(), f"relative evidence path: {path}")
    require(not path.is_symlink() and path.resolve(strict=True) == path,
            f"symlink/noncanonical evidence path: {path}")
    if base is not None:
        require(path.is_relative_to(base), f"evidence escapes expected directory: {path}")
    info = path.stat()
    require(stat.S_ISREG(info.st_mode), f"not a regular file: {path}")
    if single_link:
        require(info.st_nlink == 1, f"hard-linked evidence: {path}")
    return info


def hash_map_valid(value, keys):
    return (isinstance(value, dict) and set(value) == set(keys)
            and all(isinstance(v, str) and re.fullmatch(r"[0-9a-f]{64}", v)
                    for v in value.values()))


def fnv(raw):
    result = 0xcbf29ce484222325
    for byte in raw:
        result = ((result ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return result


def name_ok(raw):
    try:
        name = raw.decode("utf-8")
    except UnicodeDecodeError:
        return False
    return (1 <= len(raw) <= 32 and name not in (".", "..")
            and all(unicodedata.category(c) != "Cc" and c not in "/\\"
                    and not 0x202a <= ord(c) <= 0x202e
                    and not 0x2066 <= ord(c) <= 0x2069 for c in name))


def empty_object():
    return {"revision": 0, "present": False, "name": b"", "payload": b""}


def decode_image(data):
    """Decode actual sector bytes independently, preserving request identities."""
    require(len(data) == SIZE, "wrong volume size")
    objects, records, ended = [empty_object() for _ in range(4)], [], False
    for slot in range(SLOTS):
        raw = data[LOW + SECTOR * slot: LOW + SECTOR * (slot + 1)]
        if not any(raw):
            ended = True
            continue
        require(not ended, "nonempty record follows empty slot")
        require(raw[:8] == b"NOSM6C02", "unexpected journal magic or legacy record")
        version, header, flags = struct.unpack_from("<HHI", raw, 8)
        seq, request_id, oid, rev = struct.unpack_from("<QQQQ", raw, 16)
        op, length, name_length = struct.unpack_from("<HHH", raw, 48)
        require((version, header, flags) == (2, 64, 0), "journal version/header/flags")
        require(seq == slot + 1 and request_id > 0 and oid < 4 and 1 <= rev <= seq,
                "journal sequence/request/object/revision")
        require(op in (2, 3, 4) and length <= 128 and name_length <= 32,
                "journal operation or length")
        require(not any(raw[54:64] + raw[64 + length:192] + raw[192 + name_length:504]),
                "noncanonical journal padding")
        require(struct.unpack_from("<Q", raw, 504)[0] == fnv(raw[:504] + bytes(8)),
                "journal checksum mismatch")
        payload, name = raw[64:64 + length], raw[192:192 + name_length]
        old = objects[oid]
        require(rev == old["revision"] + 1, "object revision not predecessor+1")
        if op == 3:
            require(old["present"] and not payload and not name, "invalid tombstone")
        else:
            require(name_ok(name), "invalid document label")
            if op == 4:
                require(old["present"] and payload == old["payload"],
                        "rename without predecessor or with changed content")
        objects[oid] = {"revision": rev, "present": op != 3,
                        "name": name, "payload": payload}
        records.append({"slot": slot, "sequence": seq, "request_id": request_id,
                        "object": oid, "revision": rev, "operation": op,
                        "name": name, "payload": payload})
    return {"sequence": len(records), "objects": objects}, records


def value(oid, revision):
    return bytes((49 + 29 * oid + 7 * revision + 3 * i) & 255 for i in range(128))


def label(oid, revision):
    names = (["Café A", "Werk A", "Café B", "Werk B"] if revision == 1
             else ["Café hernoemd A", "Werk hernoemd A", "Café hernoemd B", "Werk hernoemd B"])
    return names[oid].encode("utf-8")


def object_summary(obj, extended=False):
    result = {"revision": obj["revision"], "present": obj["present"],
        "length": len(obj["payload"]), "digest": f'{fnv(obj["payload"]):016x}',
        "name_length": len(obj["name"]), "name_digest": f'{fnv(obj["name"]):016x}'}
    if extended:
        result.update(name=obj["name"].decode("utf-8"), payload_sha256=sha(obj["payload"]))
    return result


def summary_state(state, extended=False):
    if state is None:
        return None
    return {"sequence": state["sequence"],
            "objects": [object_summary(obj, extended) for obj in state["objects"]]}


def initial_image(seed, variant):
    require(type(seed) is int and 0 <= seed < 1 << 64, "invalid u64 canary seed")
    require(variant in ("normal", "corrupt", "legacy"), "invalid volume variant")
    raw = bytearray(random.Random(seed).randbytes(SIZE))
    raw[:1024] = bytes(1024)
    marker = b"NOS-M6C-TEST-VOLUME\0"
    raw[:len(marker)] = marker
    raw[LOW:HIGH] = bytes(HIGH - LOW)
    if variant == "corrupt":
        raw[LOW] = 128
    elif variant == "legacy":
        record = bytearray(SECTOR)
        record[:8] = b"NOSM6B01"
        struct.pack_into("<HHIQQQQHH", record, 8, 1, 64, 0, 1, 1, 0, 1, 2, 128)
        record[64:192] = value(0, 1)
        struct.pack_into("<Q", record, 504, fnv(record))
        raw[LOW:LOW + SECTOR] = record
    return bytes(raw)


def fault_disk_check(case, state, records):
    require(case in FAULTS, "not a fault case")
    allowed = ((1,) if case == "bad-descriptor" else (1, 2)
               if case in ("write-timeout", "reset-timeout") else (2,))
    seq = state["sequence"]
    require(seq in allowed, f"{case}: journal violates publication boundary")
    actual = [(r["sequence"], r["request_id"], r["object"], r["revision"], r["operation"])
              for r in records]
    require(actual == [(1, 10, 0, 1, 2), (2, 11, 0, 2, 4)][:seq],
            f"{case}: journal is not exact PUT(request10) then optional RENAME(request11)")
    require(state["objects"] == [{"revision": seq, "present": True,
                "name": label(0, seq), "payload": value(0, 1)}]
                + [empty_object() for _ in range(3)],
            f"{case}: foreign object/content/live-history change")


def verify_fault_markers(lines, case):
    found = [line for line in lines if line.startswith(FAULT_PREFIXES)]
    if case not in FAULTS:
        require(not found, f"unexpected uncertainty markers in {case}")
        return
    expected = {f'M6C:FAULT_INJECTED:{case.upper().replace("-", "_")}',
        "M6C:OUTCOME:UNKNOWN", "M6C:UNKNOWN_REPLIES:11", "M6C:NO_REPLAY:PASS",
        "M6C:POST_UNKNOWN:BLOCKCALLS:0", "M6C:POST_UNKNOWN:PUBLICATIONS:0",
        "M6C:POST_UNKNOWN:COMPLETIONS:0", "M6C:LIVE_SEQUENCE:1"}
    if case == "reset-timeout":
        expected.add("M6C:QUARANTINE:PASS")
    require(len(found) == len(expected) and set(found) == expected,
            f"{case}: incomplete/duplicate/unexpected uncertainty evidence")


def verify_state_markers(lines, state, case):
    actual = {}
    for line in lines:
        if line.startswith(STATE_PREFIXES):
            key, _, val = line.rpartition(":")
            require(key not in actual, "duplicate live-state/counter marker")
            actual[key] = val
    if case in ("corrupt", "legacy") or case in CUTS:
        require(not actual, f"{case}: premature success/state evidence")
        return None
    if case in FAULTS:
        require(actual == {"M6C:ACKS": "1", "M6C:REJECTED_NO_IO": "22",
                           "M6C:CATALOG_REPLIES": "0"},
                f"{case}: false ACK, stale live state, catalog reply or wrong rejection count")
        return None
    canonical = summary_state(state)
    expected = {"M6C:SEQUENCE": str(state["sequence"])}
    for i, obj in enumerate(canonical["objects"]):
        for field, val in (("REVISION", obj["revision"]), ("PRESENT", int(obj["present"])),
            ("LENGTH", obj["length"]), ("DIGEST", obj["digest"]),
            ("NAME_LENGTH", obj["name_length"]), ("NAME_DIGEST", obj["name_digest"])):
            expected[f"M6C:OBJECT{i}:{field}"] = str(val)
    acks = 32 if case == "full" else 10 if case in ("normal", "errors") else 0
    replies = 4 if case == "full" else 10 if case in ("normal", "errors") else 2
    rejected = {"normal": 6, "errors": 80, "full": 8}.get(
        case, sum(not obj["present"] for obj in state["objects"]))
    revisions = [sum(obj["revision"] for obj in state["objects"][2*d:2*d+2]) for d in range(2)]
    expected.update({"M6C:ACKS": str(acks), "M6C:REJECTED_NO_IO": str(rejected),
                     "M6C:CATALOG_REPLIES": str(replies)})
    for d, revision in enumerate(revisions):
        expected[f"M6C:DOMAIN{d}:REVISION"] = str(revision)
    require(actual == expected, f"{case}: serial state/counters differ from actual decoded bytes")
    return {"state": canonical, "acks": acks, "rejected": rejected,
            "domain_revisions": revisions, "catalog_replies": replies}


def controlled_markers(lines):
    prefixes = BOOT + ("M6C:READY", "M6C:CASE:", "M6C:SURVIVOR_PROGRESS",
        "M6C:RESOURCES", "M6C:PASS", "M1:PASS", "M6C:CUT:",
        "M6C:FAULT_INJECTED", "M6C:OUTCOME", "M6C:NO_REPLAY", "M6C:QUARANTINE",
        "M6A:", "M6B:")
    return [line for line in lines if line.startswith(prefixes)
            or line.startswith("M6C:") and ":PASS" in line]


def expected_markers(case):
    if case in CUTS:
        return [*BOOT, "M6C:READY", f"M6C:CUT:{CUTS[case]}"]
    result = list(BOOT)
    if case not in ("corrupt", "legacy"):
        result.append("M6C:READY")
    code = case.upper().replace("-", "_")
    if case in FAULTS:
        result += [f"M6C:FAULT_INJECTED:{code}", "M6C:OUTCOME:UNKNOWN", "M6C:NO_REPLAY:PASS"]
        if case == "reset-timeout":
            result += ["M6C:QUARANTINE:PASS"]
    return result + [f"M6C:CASE:{code}:PASS", "M6C:SURVIVOR_PROGRESS:PASS",
                     "M6C:RESOURCES:PASS", "M6C:PASS", "M1:PASS"]


def coverage():
    result = [(name, None, None) for name in HOST_GROUPS]
    for suffix in ("a", "b"):
        name = "normal-" + suffix
        result += [(name, "normal", None), (name + "-cold-recovery", "recover", name)]
    result += [("same-binary-two-canary-images", None, None)]
    for case in ("errors", "full", "corrupt", "legacy", *CUTS, *FAULTS):
        result += [(case, case, None)]
        if case not in ("corrupt", "legacy"):
            result += [(case + "-cold-recovery", "recover", case)]
    return result + [("m1-regression-after-m6c", "m1", None)]


def source_fingerprint(root):
    digest = hashlib.sha256(b"nieuw-os source inputs v3\n")
    files = []
    for folder in ("boot", "kernel", "crates", "drivers", "services", "userspace", "tools", ".cargo"):
        base = root / folder
        for path in sorted(base.rglob("*")) if base.exists() else []:
            if path.is_file() and not set(path.relative_to(base).parts) & {
                    "__pycache__", "verification", "target", "build", ".git"}:
                regular(path, root)
                rel, raw = path.relative_to(root).as_posix(), path.read_bytes()
                digest.update(rel.encode())
                digest.update(raw)
                files.append({"path": rel, "sha256": sha(raw)})
    for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml"):
        path = root / name
        if path.exists():
            regular(path, root)
            raw = path.read_bytes()
            digest.update(name.encode())
            digest.update(raw)
            files.append({"path": name, "sha256": sha(raw)})
    return digest.hexdigest(), files


def check_committed_bytes(root, commit, files):
    require(re.fullmatch(r"[0-9a-f]{40}", commit), "source commit must be full SHA-1 identity")
    requests = [commit + ":" + item["path"] for item in files]
    result = subprocess.run(["git", "cat-file", "--batch"], cwd=root,
        input=("\n".join(requests) + "\n").encode(), capture_output=True, check=True)
    raw, cursor = result.stdout, 0
    for item in files:
        end = raw.index(b"\n", cursor)
        header = raw[cursor:end].decode().split()
        require(len(header) == 3 and header[1] == "blob", "uncommitted source path: " + item["path"])
        size = int(header[2])
        cursor = end + 1
        body = raw[cursor:cursor + size]
        require(len(body) == size and raw[cursor + size:cursor + size + 1] == b"\n",
                "invalid git cat-file response")
        require(sha(body) == item["sha256"], "bytes differ from source commit: " + item["path"])
        cursor += size + 1
    require(cursor == len(raw), "unexpected trailing git cat-file response")


def audit_host_logs(root, archive, groups, python_program, cargo_program):
    summaries, identities, preflight = [], set(), None
    for group in groups[:3]:
        name = group["name"]
        stem, program, args = HOST_GROUPS[name]
        evidence = Path(group["evidence"])
        if not evidence.is_absolute():
            evidence = root / evidence
        require(evidence == root / "build/logs" / (stem + ".stdout.txt"), "wrong host log path")
        names = {stem + suffix for suffix in HOST_SUFFIXES}
        expected = group.get("log_sha256")
        require(hash_map_valid(expected, names), "missing gate-time host log SHA-256 map")
        for filename in names:
            path, archived = root / "build/logs" / filename, archive / filename
            info = regular(path, root / "build/logs", single_link=True)
            regular(archived, archive, single_link=True)
            identity = (info.st_dev, info.st_ino)
            require(identity not in identities, "reused host log inode")
            identities.add(identity)
            require(filehash(path) == filehash(archived) == expected[filename],
                    "changed/aliased host logs: " + filename)
        meta = load(root / "build/logs" / (stem + ".json"))
        expected_cmd = [python_program if program == "python" else cargo_program, *args]
        require(meta["command"] == expected_cmd and meta["cwd"] == str(root), "host command/cwd mismatch")
        require(type(meta["returncode"]) is int and meta["returncode"] == 0
                and meta.get("timed_out", False) is False, "failed/timed-out host gate")
        require(isinstance(meta["duration_s"], (int, float))
                and math.isfinite(meta["duration_s"]) and meta["duration_s"] >= 0, "host duration")
        stdout = (root / "build/logs" / (stem + ".stdout.txt")).read_text(encoding="utf-8")
        stderr = (root / "build/logs" / (stem + ".stderr.txt")).read_text(encoding="utf-8")
        counts = None
        if stem == "m6c-preflight":
            preflight = json.loads(stdout, object_pairs_hook=unique_json_pairs)
            require(preflight["workspace"] == str(root) and preflight["baseline_tools_present"] is True
                    and preflight["missing"] == [] and preflight["missing_rust_targets"] == [],
                    "strict preflight inventory failed")
            for tool in ("git", "rustc", "cargo", "rustup", "qemu-system-x86_64"):
                info = preflight["tools"][tool]
                require(info["available"] is True and info["probe_exit"] == 0 and "probe_error" not in info,
                        "required preflight tool probe failed: " + tool)
            require(preflight["tools"]["cargo"]["path"] == cargo_program, "cargo command differs from inventory")
            require({"x86_64-unknown-none", "x86_64-unknown-uefi"}.issubset(
                    preflight["installed_rust_targets"]), "missing native Rust targets")
        elif stem == "m6c-harness-tests":
            counts = re.findall(r"(?m)^Ran (\d+) tests? in ", stdout + "\n" + stderr)
            require(len(counts) == 1 and int(counts[0]) > 0 and stderr.rstrip().endswith("\nOK"),
                    "Python host-test log lacks real positive count/OK")
            counts = {"python_tests": int(counts[0])}
        else:
            counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed;", stdout)
            require(counts and sum(int(passed) for passed, _ in counts) > 0
                    and all(int(failed) == 0 for _, failed in counts), "Rust host-test result missing/failing")
            counts = {"rust_tests": sum(int(passed) for passed, _ in counts)}
        summaries.append({"name": name, "command": meta["command"], "returncode": 0,
                          "gate_time_log_sha256": expected, "test_counts": counts})
    return summaries, preflight


def qemu_command(root, directory, qemu, volume):
    rel = directory.relative_to(root).as_posix()
    args = [qemu, "-machine", "pc-q35-10.0", "-accel", "tcg,thread=single", "-cpu", "qemu64",
        "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
        "-serial", "stdio", "-nic", "none", "-no-reboot",
        "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
        "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
        "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
    if volume is not None:
        image_rel = volume.relative_to(root).as_posix()
        args += ["-drive", f"if=none,id=m6c-test,format=raw,cache=writeback,readonly=off,file={image_rel}",
            "-device", "virtio-blk-pci,drive=m6c-test,id=m6cblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-M6C-TEST"]
    share = Path(qemu).parent / "share"
    if share.is_dir():
        args += ["-L", str(share)]
    return args


def verify_oracle(report, state, observed, initial, after, meta, case):
    oracle = report["disk_oracle"]
    for key, expected in (("identity", meta["identity"]), ("seed", meta["seed"]),
        ("variant", meta["variant"]), ("sha256_initial", sha(initial)),
        ("sha256_after", sha(after)), ("state", summary_state(state)), ("observed", observed)):
        require(oracle[key] == expected, f"claimed oracle differs from independent evidence: {key}")
    keys = {"outside_journal_unchanged", "case_matches_volume_variant", "fresh_mutator_before_image",
        "journal_valid_or_explicit_rejection_case", "guest_matches_independent_disk_state",
        "read_only_stage_unchanged", "record_payloads_and_names_match_native_scenario"}
    if state is not None:
        if case in FAULTS:
            keys |= {"fault_prefix_matches_publication_boundary",
                     "fault_records_are_only_confirmed_put_and_optional_rename"}
        else:
            keys.add("expected_final_state")
    if case in FAULTS:
        keys.add("uncertainty_has_no_success_ack_or_replay")
    if observed:
        keys |= {"exact_mutation_ack_count", "exact_catalog_reply_count",
                 "domain_revisions_exclude_foreign_mutations"}
        if case in ("normal", "errors", "full"):
            keys.add("all_negative_requests_without_io")
    require(oracle["checks"] == {key: True for key in keys} and oracle["passed"] is True,
            "omitted/failing/unexpected saved oracle checks")


def journal_sectors(raw):
    return [{"slot": slot, "hex": raw[LOW + slot*SECTOR:LOW + (slot+1)*SECTOR].hex()}
            for slot in range(SLOTS) if any(raw[LOW + slot*SECTOR:LOW + (slot+1)*SECTOR])]


def reconstruct_export(export):
    require(export["schema"] == 2 and export["image_bytes"] == SIZE
            and export["journal_first_sector"] == FIRST
            and export["journal_sector_count"] == SLOTS and export["sector_bytes"] == SECTOR,
            "unexpected journal export schema/layout")
    require(len(export["volumes"]) == 16, "export must contain exactly 16 unique volumes")
    results, identities = [], set()
    for item in export["volumes"]:
        require(item["identity"] not in identities, "duplicate exported volume identity")
        identities.add(item["identity"])
        raw = bytearray(initial_image(item["seed"], item["variant"]))
        require(journal_sectors(raw) == item["initial_nonempty_journal"], "initial sectors differ from seeded fixture")
        require(sha(raw) == item["initial_sha256"], "reconstructed initial hash mismatch")
        raw[LOW:HIGH] = bytes(HIGH - LOW)
        slots = []
        for record in item["final_nonempty_journal"]:
            slot = record["slot"]
            require(type(slot) is int and 0 <= slot < SLOTS and slot not in slots, "invalid exported sector slot")
            encoded = record["hex"]
            require(isinstance(encoded, str) and re.fullmatch(r"[0-9a-f]{1024}", encoded), "noncanonical sector hex")
            data = bytes.fromhex(encoded)
            require(any(data), "empty sector in nonempty export")
            slots.append(slot)
            raw[LOW + slot*SECTOR:LOW + (slot+1)*SECTOR] = data
        require(slots == sorted(slots), "unordered exported sectors")
        require(sha(raw) == item["final_sha256"], "reconstructed final hash mismatch")
        if item["variant"] in ("legacy", "corrupt"):
            try:
                decode_image(raw)
            except AssertionError:
                require(item["state"] is None, "invalid exported state for rejected journal")
            else:
                raise AssertionError("legacy/corrupt exported journal unexpectedly valid")
        else:
            state, records = decode_image(raw)
            require(summary_state(state, True) == item["state"], "exported decoded state mismatch")
            if item["origin_case"] in FAULTS:
                fault_disk_check(item["origin_case"], state, records)
        results.append({"identity": item["identity"], "initial_sha256": item["initial_sha256"],
                        "final_sha256": item["final_sha256"], "passed": True})
    return {"mode": "text-only seeded image reconstruction", "passed": True,
            "volumes": len(results), "checks": results,
            "boundary": "Reconstructs recorded bytes and hashes; does not prove a new native execution."}


def audit(args):
    root, out = args.repo.resolve(strict=True), args.out.resolve()
    require(out != root and not out.is_relative_to(root), "audit output must be outside repository")
    out.mkdir(parents=True, exist_ok=True)
    archive = Path(args.archive)
    archive = (archive if archive.is_absolute() else root / archive).resolve(strict=True)
    require(archive.is_relative_to(root / "docs/verification"), "archive outside verification tree")
    suite_path = root / "build/m6c-test-results.json"
    suite, capture, archived_runs = load(suite_path), load(archive / "capture.json"), load(archive / "guest-runs.json")
    wanted = coverage()
    require(len(wanted) == 35, "internal coverage definition")
    require(suite["phase"] == "M6c" and suite["status"] == "completed" and suite["passed"] is True,
            "native gate is incomplete or failed; no passing audit claim permitted")
    require(suite == load(archive / "suite.json"), "archived/live suite differ")
    require(suite["source_fingerprint_version"] == 3 and suite["source_sha256"] == args.source_sha256,
            "suite source identity differs from frozen source")
    groups = suite["results"]
    require([group["name"] for group in groups] == [name for name, _, _ in wanted],
            "missing/duplicate/reordered gate groups")
    require(all(group["passed"] is True for group in groups), "one or more failed gate groups")
    require(capture["passed"] is True and capture["groups"] == 35 and capture["vm_invocations"] == 31
            and capture["source_sha256"] == args.source_sha256, "capture metadata mismatch")
    require(hash_map_valid(capture["files"], ARCHIVE_FILES), "capture hash map missing/extra required files")
    require({p.name for p in archive.iterdir()} == ARCHIVE_FILES | {"capture.json"},
            "unexpected/missing capture archive members")
    archive_checks = []
    for name, expected in capture["files"].items():
        path = archive / name
        regular(path, archive, single_link=True)
        actual = filehash(path)
        require(actual == expected, "archive hash mismatch: " + name)
        archive_checks.append({"file": name, "sha256": actual})
    actual_source, source_files = source_fingerprint(root)
    require(actual_source == args.source_sha256, "current source fingerprint differs from gate")
    test_files = [{"path": p.relative_to(root).as_posix(), "sha256": filehash(p)}
                  for p in sorted((root / "tests").glob("test_m6c_*.py"))]
    require(suite["host_test_sha256"] == {item["path"]: item["sha256"] for item in test_files},
            "host test files changed or omitted from gate snapshot")
    check_committed_bytes(root, args.source_commit, source_files + test_files)
    cargo = args.cargo_program or str(root / ".tools/nieuw-os/cargo/bin/cargo")
    host, preflight = audit_host_logs(root, archive, groups, args.python_program, cargo)
    qemu = preflight["tools"]["qemu-system-x86_64"]["path"]
    qemu_hash = filehash(Path(qemu))
    require(len(archived_runs) == 31, "wrong archived VM count")
    stage_data, volumes, origins = {}, {}, {}
    summaries, cold, crashes, faults, pids, inventory, base_counts = [], [], [], [], [], [], collections.Counter()
    directories, archive_index, artifacts_count = set(), 0, 0
    for group, (name, case, parent_name) in zip(groups, wanted):
        if case is None:
            continue
        directory = Path(group["evidence"])
        require(directory.is_absolute() and directory.resolve(strict=True) == directory
                and directory.is_relative_to(root / "build/run") and directory.is_dir(), "VM directory escapes run tree")
        require(directory not in directories, "VM directory reused across required cases")
        directories.add(directory)
        report = load(directory / "result.json")
        require(report == archived_runs[archive_index], "archived guest report differs from actual result.json: " + name)
        archive_index += 1
        require(report["run_directory"] == str(directory), "report bound to different run directory")
        require(report["passed"] is True and report["timed_out"] is False
                and report["source_sha256"] == args.source_sha256
                and report["source_fingerprint_version"] == 3
                and report["base_commit"] == args.source_commit, "run failed or wrong source/commit: " + name)
        base_counts[report["base_commit"]] += 1
        serial_file, error_file = directory / "serial.txt", directory / "stderr.txt"
        regular(serial_file, directory)
        regular(error_file, directory)
        serial_bytes, error_bytes = serial_file.read_bytes(), error_file.read_bytes()
        serial, stderr = serial_bytes.decode("utf-8"), error_bytes.decode("utf-8")
        lines = serial.splitlines()
        require(serial == report["stdout"] and stderr == report["stderr"] and not stderr,
                "raw serial/stderr mismatch or nonempty QEMU stderr: " + name)
        require(not any(re.search(r"M[1-9][A-Z]?:FAIL", line) for line in lines), "guest failure marker: " + name)
        require(all(lines.count(marker) == 1 for marker in BOOT), "missing/duplicate boot marker: " + name)
        expected_artifacts = M1_ARTIFACTS if case == "m1" else M6C_ARTIFACTS
        require(hash_map_valid(report["artifact_sha256"], expected_artifacts), "incomplete/flexible artifact set: " + name)
        for rel, expected in report["artifact_sha256"].items():
            path = directory / rel
            regular(path, directory)
            require(filehash(path) == expected, "raw artifact hash mismatch: " + name + "/" + rel)
            artifacts_count += 1
        all_files = {}
        for path in sorted(directory.rglob("*")):
            if path.is_file():
                regular(path, directory)
                all_files[path.relative_to(directory).as_posix()] = filehash(path)
        inventory.append({"run": name, "files": all_files})
        if case == "m1":
            require(report.get("case") is None and "volume_path" not in report, "M1 was not ordinary boot")
            require(report["command"] == qemu_command(root, directory, qemu, None), "unexpected M1 QEMU argv")
            require(report["raw_qemu_exit"] == 33 and report["expected_exit"] == 33,
                    "M1 raw exit mismatch")
            require([line for line in lines if line.startswith(BOOT + ("M1:PASS",))] == [*BOOT, "M1:PASS"]
                    and not any(re.match(r"^M(?:[2-9]|6[A-Z]):", line) for line in lines), "M1 marker order/phase")
            require(hash_map_valid(report["expected_artifact_sha256"], {"esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI"})
                    and report["artifact_binding_passed"] is True
                    and all(report["artifact_sha256"][key] == val
                            for key, val in report["expected_artifact_sha256"].items()), "M1 build artifact binding")
            summaries.append({"name": name, "case": case, "raw_exit": 33,
                "serial_sha256": sha(serial_bytes), "artifacts": len(expected_artifacts), "passed": True})
            continue
        require(report["case"] == case and report["label"] == name, "wrong native case or label")
        require(report["observed_source_sha256"] == args.source_sha256 and report["source_unchanged"] is True
                and report["execution_error"] is None and report["oracle_error"] is None,
                "source changed or execution/oracle error")
        require(report["qemu_sha256"] == qemu_hash and report["serial_sha256"] == sha(serial_bytes)
                and report["stderr_sha256"] == sha(error_bytes), "raw launcher/output hash mismatch")
        build = load(directory / "build-manifest.json")
        feature = "milestone-m6c" if case == "normal" else "test-m6c-" + case
        require(build["kernel_feature"] == feature and build["loader_feature"] is None
                and build["source_fingerprint_version"] == 3 and build["source_sha256"] == args.source_sha256,
                "build feature/source mismatch")
        require(build["kernel"]["sha256"] == report["artifact_sha256"]["esp/kernel.elf"]
                and build["loader"]["sha256"] == report["artifact_sha256"]["esp/EFI/BOOT/BOOTX64.EFI"],
                "build manifest does not bind actual ESP binaries")
        meta = load(directory / "volume-manifest.json")
        variant = case if case in ("corrupt", "legacy") else "normal"
        initial = initial_image(meta["seed"], variant)
        require(meta == {"scope": "fresh synthetic M6c volume", "identity": meta["identity"],
            "seed": meta["seed"], "variant": variant, "bytes": SIZE, "bdf": "00:05.0", "initial_sha256": sha(initial)}
            and isinstance(meta["identity"], str) and re.fullmatch(r"[0-9a-f]{32}", meta["identity"]),
            "synthetic volume provenance mismatch")
        before = (directory / "test-disk.before.img").read_bytes()
        after = (directory / "test-disk.after.img").read_bytes()
        volume_path = Path(report["volume_path"])
        info = regular(volume_path, root / "build/run", single_link=True)
        require(volume_path.name == "test-disk.img" and info.st_size == SIZE
                and volume_path.read_bytes() == after, "actual volume not exact final snapshot")
        require(load(volume_path.parent / "volume.json") == meta, "volume identity changed")
        require(report["command"] == qemu_command(root, directory, qemu, volume_path), "QEMU profile/volume binding mismatch")
        require(len(before) == len(after) == SIZE and before[:LOW] == after[:LOW] == initial[:LOW]
                and before[HIGH:] == after[HIGH:] == initial[HIGH:], "outside-journal canaries changed")
        if case != "recover":
            require(before == initial and report["previous_stage"] is None, "mutator/rejection stage was not fresh")
            require(meta["identity"] not in origins, "fresh case reused earlier volume")
            origins[meta["identity"]] = case
        if case in ("recover", "corrupt", "legacy"):
            require(before == after, "read-only/recovery stage wrote actual volume bytes")
        if case in ("corrupt", "legacy"):
            try:
                decode_image(after)
            except AssertionError:
                state, records = None, []
            else:
                raise AssertionError("legacy/corrupt volume unexpectedly decoded as M6c")
        else:
            state, records = decode_image(after)
            for record in records:
                if record["operation"] != 3:
                    require(record["name"] == label(record["object"], record["revision"]), "fixture label differs")
                if record["operation"] == 2:
                    require(record["payload"] == value(record["object"], record["revision"]), "fixture PUT content differs")
            if case in ("normal", "errors"):
                require(state["sequence"] == 10, "normal/error history length")
                for i, obj in enumerate(state["objects"]):
                    require(obj["revision"] == (3 if i % 2 == 0 else 2) and obj["present"] == (i % 2 == 0),
                            "normal/error final object state")
                    require([r["operation"] for r in records if r["object"] == i]
                            == ([2, 4, 2] if i % 2 == 0 else [2, 3]), "normal/error operation history")
            if case == "full":
                require(state["sequence"] == 32 and all(r["operation"] == 2 for r in records)
                        and all(obj["revision"] == (16 if i % 2 == 0 else 0)
                            and obj["present"] == (i % 2 == 0) for i, obj in enumerate(state["objects"])),
                        "full-journal native state/history")
            if case in FAULTS:
                fault_disk_check(case, state, records)
        verify_fault_markers(lines, case)
        observed = verify_state_markers(lines, state, case)
        require(controlled_markers(lines) == expected_markers(case), "native marker order/duplicates/false success: " + name)
        require(report["required_markers"] == expected_markers(case)
                and report["outcome_marker_passed"] is True, "reported marker contract mismatch")
        verify_oracle(report, state, observed, initial, after, meta, case)
        crash = report["host_crash"]
        pid = crash["process_id"]
        require(type(pid) is int and pid > 0 and pid not in pids, "missing/reused native QEMU process ID")
        pids.append(pid)
        if case in CUTS:
            point = CUTS[case]
            require(report["raw_qemu_exit"] == -9 and crash["real_process_returncode"] == -9
                    and crash["termination_code_expected"] == -9 and crash["kill_requested"] is True
                    and crash["was_running_before_kill"] is True and crash["passed"] is True
                    and crash["required_live_hold_s"] == 0.05 and crash["requested_cut"] == point
                    and crash["cut_lines"] == [f"M6C:CUT:{point}"], "not an actual externally terminated live child")
            require(state["sequence"] == point and state["objects"] == [
                {"revision": point, "present": True, "name": label(0, point), "payload": value(0, 1)}]
                + [empty_object() for _ in range(3)], "wrong before/after-rename crash image")
            crashes.append({"run": name, "process_id": pid, "raw_exit": -9,
                            "sequence": point, "image_sha256": sha(after)})
        else:
            require(report["raw_qemu_exit"] == 33 and report["expected_exit"] == 33
                    and crash["real_process_returncode"] == 33 and crash["kill_requested"] is False
                    and crash["requested_cut"] is None and crash["cut_lines"] == [], "normal native run exit/kill mismatch")
        if case == "recover":
            require(parent_name in stage_data, "cold recovery lacks exact earlier audited parent")
            parent = stage_data[parent_name]
            require(report["previous_stage"] == str(parent["directory"])
                    and parent["volume"] == str(volume_path) and parent["identity"] == meta["identity"]
                    and parent["after"] == before == after and parent["pid"] != pid,
                    "cold VM does not bind exact previous volume/bytes/distinct process")
            require(group["recovery_checks"] == {key: True for key in (
                "same_volume", "same_identity", "separate_vm", "linked_previous", "exact_previous_bytes",
                "no_writes", "frozen_source")}, "cold-recovery check map missing/failing")
            cold.append({"run": name, "previous_run": parent_name, "same_identity": meta["identity"],
                "image_sha256": sha(after), "sequence": state["sequence"], "process_ids": [parent["pid"], pid]})
        if case in FAULTS:
            require(isinstance(report["fault_injection_scope"], str)
                    and "not a physical device failure" in report["fault_injection_scope"], "fault scope misrepresented")
            faults.append({"case": case, "live_sequence": 1, "ack_count": 1, "unknown_replies": 11,
                "successful_catalog_replies": 0, "post_unknown_calls_publications_completions": [0, 0, 0],
                "physical_journal_sequence": state["sequence"],
                "journal_request_ids": [r["request_id"] for r in records],
                "quarantined": case == "reset-timeout", "image_sha256": sha(after)})
        elif report["fault_injection_scope"] is not None:
            raise AssertionError("nonfault case claims fault injection")
        stage_data[name] = {"directory": directory, "after": after, "volume": str(volume_path),
            "identity": meta["identity"], "pid": pid, "state": state, "report": report}
        entry = {"identity": meta["identity"], "origin_case": origins[meta["identity"]],
            "seed": meta["seed"], "variant": variant, "initial_sha256": sha(initial), "final_sha256": sha(after),
            "initial_nonempty_journal": journal_sectors(initial), "final_nonempty_journal": journal_sectors(after),
            "state": summary_state(state, True)}
        if meta["identity"] in volumes:
            require(volumes[meta["identity"]] == entry, "cold reconstructed volume differs from parent")
        volumes[meta["identity"]] = entry
        summaries.append({"name": name, "case": case, "raw_exit": report["raw_qemu_exit"],
            "process_id": pid, "artifacts": len(expected_artifacts), "serial_sha256": sha(serial_bytes),
            "volume_identity": meta["identity"], "image_after_sha256": sha(after),
            "journal_sequence": None if state is None else state["sequence"], "passed": True})
    require(len(directories) == len(summaries) == 31 and len(pids) == 30 and len(cold) == 14
            and len(crashes) == 2 and len(faults) == 8 and len(volumes) == 16,
            "final VM/process/cold/crash/fault/volume coverage mismatch")
    a, b = stage_data["normal-a"], stage_data["normal-b"]
    require(all(a["report"]["artifact_sha256"][key] == b["report"]["artifact_sha256"][key]
                for key in ("esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI")), "two canaries not same binaries")
    av, bv = volumes[a["identity"]], volumes[b["identity"]]
    require(av["identity"] != bv["identity"] and av["seed"] != bv["seed"]
            and av["initial_sha256"] != bv["initial_sha256"] and a["state"] == b["state"],
            "two-image seed/identity/state check failed")
    comparison_path = root / "build/logs/m6c-two-image-check.json"
    comparison_group = next(group for group in groups if group["name"] == "same-binary-two-canary-images")
    require(comparison_group["evidence"] == "build/logs/m6c-two-image-check.json", "two-image comparison wrong path")
    require(filehash(comparison_path) == filehash(archive / comparison_path.name), "two-image archived bytes differ")
    comparison = load(comparison_path)
    require(comparison["passed"] is True and comparison["checks"] == {key: True for key in (
        "same_kernel", "same_loader", "different_canary_images", "same_explicit_fixture_state")}
        and comparison["run_directories"] == [str(a["directory"]), str(b["directory"])], "comparison report not bound to normal runs")
    latest = {}
    for item in stage_data.values():
        latest[item["volume"]] = str(item["directory"])
    for volume_path, last_directory in latest.items():
        require(load(Path(volume_path).parent / "last-stage.json") == {"run_directory": last_directory},
                "volume last-stage pointer differs from final audited invocation")
    export = {"schema": 2, "scope": "actual disposable native volumes; independent byte decoding and text reconstruction",
        "source_commit": args.source_commit, "source_sha256": args.source_sha256,
        "image_bytes": SIZE, "journal_first_sector": FIRST, "journal_sector_count": SLOTS, "sector_bytes": SECTOR,
        "initial_algorithm": "random.Random(seed).randbytes(65536); zero bytes0:1024; write NOS-M6C-TEST-VOLUME plus NUL at0; zero bytes2048:18432; overlay initial_nonempty_journal at(4+slot)*512",
        "final_algorithm": "reconstruct initial; zero journal; overlay final_nonempty_journal; compare final_sha256",
        "python_version": sys.version, "volumes": list(volumes.values())}
    reconstruction = reconstruct_export(export)
    # Freeze again after all reads, including every recorded artifact's raw bytes.
    require(source_fingerprint(root)[0] == args.source_sha256, "source changed during independent audit")
    require(load(suite_path) == suite and load(archive / "capture.json") == capture
            and load(archive / "guest-runs.json") == archived_runs, "suite/archive changed during independent audit")
    for item in inventory:
        directory = next(Path(g["evidence"]) for g in groups if g["name"] == item["run"])
        require(all(filehash(directory / rel) == expected for rel, expected in item["files"].items()),
                "raw run artifact changed during independent audit: " + item["run"])
    outputs = {"m6d-actual-journal-records.json": export, "m6d-all-artifact-hashes.json": inventory,
        "m6d-audited-source-hashes.json": {"source": source_files, "host_tests": test_files},
        "m6d-image-reconstruction.json": reconstruction}
    for filename, data in outputs.items():
        save(out / filename, data)
    report = {"task": "R09 independent M6d final native audit", "passed": True,
        "audited_utc": datetime.now(timezone.utc).isoformat(), "source_commit": args.source_commit,
        "source_fingerprint_version": 3, "source_sha256": args.source_sha256,
        "source_files_checked_against_commit": len(source_files), "host_test_files_checked_against_commit": len(test_files),
        "groups": 35, "vm_invocations": 31, "m6c_invocations": 30, "m1_invocations": 1,
        "completed_exit33": 29, "external_kill_exit_minus9": 2, "cold_recoveries": 14,
        "fault_cases": 8, "unique_volumes": 16, "recorded_artifact_hashes_verified": artifacts_count,
        "all_actual_run_files_hashed": sum(len(item["files"]) for item in inventory),
        "archive_hashes_verified": len(archive_checks),
        "valid_v2_records_across_unique_volumes": sum(v["state"]["sequence"] for v in volumes.values() if v["state"]),
        "nonempty_records_exported": sum(len(v["final_nonempty_journal"]) for v in volumes.values()),
        "base_commit_metadata_counts": dict(base_counts), "host_groups": host, "runs": summaries,
        "cold_pairs": cold, "external_cuts": crashes, "uncertainty_cases": faults,
        "archive_files": archive_checks, "qemu_launcher_sha256": qemu_hash,
        "evidence_files": {name: filehash(out / name) for name in outputs},
        "suite_sha256": filehash(suite_path), "capture_sha256": filehash(archive / "capture.json"),
        "audit_script_sha256": filehash(Path(__file__)), "findings": [],
        "limitations": [
            "Read-only reinspection of existing native run evidence; this audit starts no new VM and is not an independent rerun.",
            "Error/status and malformed used-length injections modify a local completion snapshot after a real used-ring advance; they are not physical device failures.",
            "Timeout tests hide completion observation until the real deadline; a published write may already have reached the image. Live ACK state and journal sequence are checked separately.",
            "Reset-timeout validates bounded quarantine and no reuse in the observed run; it does not prove real-hardware reset behavior.",
            "Cold recovery is an OS/VM restart on the exact existing image, not automatic in-process service restart.",
            "External QEMU process termination is not a physical power-loss test. FNV is diagnostic, not cryptographic authentication.",
            "Guest no-replay and no-extra-I/O counters rely on the source-bound native assertions and raw emitted markers; this is not formal proof or hardware tracing.",
            "The QEMU launcher, guest binaries and firmware artifacts are hashed; all host shared libraries and the entire host environment are not independently attested.",
            "Initial failed preflight inventory is preserved separately and is not counted as a native VM success or failure in this completed gate.",
            "M6b regression, other milestone reruns and the 100-point OS readiness score are outside this audit.",
            "AI code/evidence review is not a human audit or formal verification.",
        ]}
    save(out / "m6d-final-native-audit.json", report)
    return {key: val for key, val in report.items() if key not in (
        "host_groups", "runs", "cold_pairs", "uncertainty_cases", "archive_files", "limitations")}


def self_test():
    """Host-only hostile-fixture tests of this audit's parser/marker checks."""
    checks = []

    def rejects(name, action):
        try:
            action()
        except AssertionError:
            checks.append(name)
        else:
            raise AssertionError("self-test accepted hostile fixture: " + name)

    def seal(record):
        record[504:] = bytes(8)
        struct.pack_into("<Q", record, 504, fnv(record))
        return bytes(record)

    def record(seq, request, op, revision, payload, name):
        raw = bytearray(SECTOR)
        raw[:8] = b"NOSM6C02"
        struct.pack_into("<HHIQQQQHHH", raw, 8, 2, 64, 0, seq, request, 0, revision,
                         op, len(payload), len(name))
        raw[64:64+len(payload)] = payload
        raw[192:192+len(name)] = name
        return seal(raw)

    def image(records):
        raw = bytearray(initial_image(1234, "normal"))
        for slot, data in enumerate(records):
            raw[LOW+slot*SECTOR:LOW+(slot+1)*SECTOR] = data
        return bytes(raw)

    put = record(1, 10, 2, 1, value(0, 1), label(0, 1))
    rename = record(2, 11, 4, 2, value(0, 1), label(0, 2))
    original = image([put, rename])
    state, records = decode_image(original)
    fault_disk_check("write-error", state, records)
    fault_disk_check("bad-descriptor", *decode_image(image([put])))
    checks += ["valid exact PUT10/RENAME11 accepted", "valid prepublication PUT10 prefix accepted"]
    tampered = bytearray(rename)
    struct.pack_into("<Q", tampered, 24, 111)
    rejects("resealed wrong request identity", lambda: fault_disk_check(
        "write-error", *decode_image(image([put, seal(tampered)]))))
    rejects("prepublication extra rename", lambda: fault_disk_check("bad-descriptor", state, records))
    bad_content = record(2, 11, 4, 2, value(0, 2), label(0, 2))
    rejects("rename changes payload with valid checksum", lambda: decode_image(image([put, bad_content])))
    bad_padding = bytearray(rename)
    bad_padding[54] = 1
    rejects("resealed noncanonical padding", lambda: decode_image(image([put, seal(bad_padding)])))
    rejects("record after empty slot", lambda: decode_image(image([bytes(SECTOR), rename])))
    wrong_checksum = bytearray(original)
    wrong_checksum[LOW+64] ^= 1
    rejects("raw checksum corruption", lambda: decode_image(wrong_checksum))
    rejects("legacy rejection", lambda: decode_image(initial_image(1234, "legacy")))
    rejects("corrupt rejection", lambda: decode_image(initial_image(1234, "corrupt")))
    case = "write-error"
    markers = expected_markers(case) + ["M6C:UNKNOWN_REPLIES:11", "M6C:POST_UNKNOWN:BLOCKCALLS:0",
        "M6C:POST_UNKNOWN:PUBLICATIONS:0", "M6C:POST_UNKNOWN:COMPLETIONS:0", "M6C:LIVE_SEQUENCE:1",
        "M6C:ACKS:1", "M6C:REJECTED_NO_IO:22", "M6C:CATALOG_REPLIES:0"]
    verify_fault_markers(markers, case)
    verify_state_markers(markers, state, case)
    checks.append("valid Unknown marker fixture accepted (host-only)")
    rejects("duplicate Unknown outcome", lambda: verify_fault_markers(markers + ["M6C:OUTCOME:UNKNOWN"], case))
    rejects("replay publication", lambda: verify_fault_markers([
        line.replace("PUBLICATIONS:0", "PUBLICATIONS:1") for line in markers], case))
    rejects("false success ACK", lambda: verify_state_markers([
        line.replace("M6C:ACKS:1", "M6C:ACKS:2") for line in markers], state, case))
    rejects("stale full-state response", lambda: verify_state_markers(markers + ["M6C:SEQUENCE:1"], state, case))
    rejects("unexpected uncertainty in recovery", lambda: verify_fault_markers(markers, "recover"))
    require(len(coverage()) == 35 and sum(case is not None for _, case, _ in coverage()) == 31,
            "self-test coverage total")
    return {"mode": "audit-script host-only self-test", "passed": True, "checks": len(checks),
            "details": checks, "boundary": "These are fabricated host test inputs, not guest execution evidence."}


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", type=Path)
    parser.add_argument("--archive", default="docs/verification/m6d")
    parser.add_argument("--out", type=Path, default=Path("/tmp/r09-m6d-audit"))
    parser.add_argument("--source-commit", default=SOURCE_COMMIT)
    parser.add_argument("--source-sha256", default=SOURCE_SHA256)
    parser.add_argument("--python-program", default=sys.executable)
    parser.add_argument("--cargo-program")
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument("--self-test", action="store_true")
    modes.add_argument("--reconstruct", type=Path)
    args = parser.parse_args()
    try:
        if args.self_test:
            result = self_test()
        elif args.reconstruct is not None:
            result = reconstruct_export(load(args.reconstruct))
        else:
            require(args.repo is not None, "--repo is required for native evidence audit")
            result = audit(args)
    except Exception as error:
        result = {"task": "R09 independent M6d audit", "passed": False,
                  "error_type": type(error).__name__, "error": str(error)}
        if not args.self_test and args.reconstruct is None and args.repo is not None:
            root, out = args.repo.resolve(), args.out.resolve()
            if out != root and not out.is_relative_to(root):
                out.mkdir(parents=True, exist_ok=True)
                save(out / "m6d-final-native-audit.json", result)
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 1
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
