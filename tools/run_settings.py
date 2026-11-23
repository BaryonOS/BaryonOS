#!/usr/bin/env python3
"""U05-N1 native settings persistence runner: six cold QEMU boots on one
disposable volume image chain, each checked by an independent host oracle.

1 settings, blank image -> both domains initialize and set: gen 2 = S2.
2 settings-recover, read-only mount -> gen 2 = S2, image SHA-256 unchanged.
3 settings -> recover, verify S2 and revert after the restart: gen 3 = S1.
4 settings-recover, domain 0's newest slot damaged -> fallback to the other
  slot (gen 2 = S2; gen 1 was overwritten by step 3's revert).
5 settings-recover, both domain-0 slots damaged -> Corrupt refusal, no writes.
6 settings-recover, domain 0's older slot replaced by a schema-2 record with a
  valid SHA-256 -> UnsupportedVersion refusal although the other slot is valid.

Damage edits settings bytes and then repairs the volume's FNV sums, so the
refusal comes from settings-core's SHA-256/version checks, not the volume.
Domain 1 is never damaged; it must recover its own value in every step.
Evidence is native QEMU for a bounded per-domain settings path, not U05
acceptance; SHA-256 here is integrity only (no authentication/rollback).
"""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import time
import uuid
from common import BUILD, ROOT, environment, firmware, qemu_accel, qemu_machine, save_json, sha256, source_hash
from run_volume import BOOT, FAILURES, controlled, execute, numbers
import volume_disk

FEATURE = {"settings": "test-volume-settings", "settings-recover": "test-volume-settings-recover"}
MARKER = {"settings": "SETTINGS", "settings-recover": "SETTINGS_RECOVER"}
SLOT_LEN, BODY_LEN, MAGIC, SCHEMA = 96, 64, b"NOSSET01", 1
DOMAINS = 2
# Settings slot i of domain d is volume object d * PER_DOMAIN + i.
SLOT_OBJECTS = {d: (d * volume_disk.PER_DOMAIN, d * volume_disk.PER_DOMAIN + 1) for d in range(DOMAINS)}
OPAQUE = frozenset(o for pair in SLOT_OBJECTS.values() for o in pair)
SLOT_NAMES = ("settings-a", "settings-b")
# Report word 496 (sdk/examples/settings-volume/src/lib.rs `code`).
CORRUPT, UNSUPPORTED = 5, 6


def settings_value(theme: int, motion: bool, scale: int) -> int:
    """Report word 488: the canonical 8-byte settings encoding, little-endian."""
    return int.from_bytes(bytes([theme, int(motion)]) + struct.pack("<H", scale) + bytes(4), "little")


# Fixture values of the app (first/second per domain), written out independently.
S1 = {0: settings_value(1, False, 100), 1: settings_value(0, False, 125)}
S2 = {0: settings_value(2, True, 150), 1: settings_value(1, True, 200)}
# step -> (case, image source, expected (generation, value, status) per domain)
STEPS = {
    1: ("settings", "fresh", {0: (2, S2[0], 0), 1: (2, S2[1], 0)}),
    2: ("settings-recover", "previous", {0: (2, S2[0], 0), 1: (2, S2[1], 0)}),
    3: ("settings", "previous", {0: (3, S1[0], 0), 1: (3, S1[1], 0)}),
    4: ("settings-recover", "damage-newest", {0: (2, S2[0], 0), 1: (3, S1[1], 0)}),
    5: ("settings-recover", "damage-both", {0: (0, 0, CORRUPT), 1: (3, S1[1], 0)}),
    6: ("settings-recover", "schema-2", {0: (0, 0, UNSUPPORTED), 1: (3, S1[1], 0)}),
}
MAX_TIMEOUT = 600


# --- independent NOSSET01 oracle -------------------------------------------------

def decode_settings(raw: bytes):
    """Canonical settings value or None (strict theme/flag/scale/padding)."""
    theme, motion, scale = raw[0], raw[1], struct.unpack_from("<H", raw, 2)[0]
    if theme > 2 or motion > 1 or any(raw[4:8]) or not 75 <= scale <= 300:
        return None
    return int.from_bytes(raw[:8], "little")


def decode_record(raw: bytes):
    """("blank",), ("bad",), ("unsupported",) or ("good", generation, current, previous)."""
    if len(raw) != SLOT_LEN:
        raise ValueError("settings slot must be 96 bytes")
    if not any(raw):
        return ("blank",)
    if raw[:8] != MAGIC or raw[BODY_LEN:] != hashlib.sha256(raw[:BODY_LEN]).digest():
        return ("bad",)
    version, flag = struct.unpack_from("<IB", raw, 8)
    if version != SCHEMA:
        return ("unsupported",)
    generation = struct.unpack_from("<Q", raw, 16)[0]
    if generation == 0 or flag > 1 or any(raw[13:16]) or any(raw[40:BODY_LEN]):
        return ("bad",)
    current = decode_settings(raw[24:32])
    if current is None:
        return ("bad",)
    if flag:
        previous = decode_settings(raw[32:40])
        if previous is None:
            return ("bad",)
    else:
        if any(raw[32:40]):
            return ("bad",)
        previous = None
    return ("good", generation, current, previous)


def encode_record(generation: int, current: int, previous: int | None, schema: int = SCHEMA) -> bytes:
    body = bytearray(BODY_LEN)
    body[:8] = MAGIC
    struct.pack_into("<IB", body, 8, schema, int(previous is not None))
    struct.pack_into("<QQQ", body, 16, generation, current, previous or 0)
    return bytes(body) + hashlib.sha256(body).digest()


def expected_recovery(slots):
    """settings-core recover() on two decoded slots -> (generation, value, status)."""
    records = [decode_record(raw) for raw in slots]
    if any(r[0] == "unsupported" for r in records):
        return (0, 0, UNSUPPORTED)
    good = [r for r in records if r[0] == "good"]
    if not good:
        return (0, 0, 3 if all(r[0] == "blank" for r in records) else CORRUPT)
    if len(good) == 2 and good[0][1] == good[1][1]:
        return (0, 0, CORRUPT)
    best = max(good, key=lambda r: r[1])
    return (best[1], best[2], 0)


def slot_bytes(oracle, obj: int) -> bytes:
    """Current content of one settings object; absent objects read as blank."""
    entry = oracle["objects"][obj]
    if not entry["present"]:
        return bytes(SLOT_LEN)
    data = bytes.fromhex(entry["versions"][0]["content_hex"])
    if len(data) != SLOT_LEN or entry["versions"][0]["name"] != SLOT_NAMES[obj % volume_disk.PER_DOMAIN]:
        raise ValueError(f"settings object {obj} has an unexpected length or name")
    return data


def domain_slots(image: bytes, domain: int):
    oracle = volume_disk.analyse(image, OPAQUE)
    return [slot_bytes(oracle, obj) for obj in SLOT_OBJECTS[domain]], oracle


# --- host damage helpers (repair the volume's FNV sums) --------------------------

def rewrite_object(image: bytes, obj: int, content: bytes) -> bytes:
    """Replace the current content of `obj` (same length) and repair every
    volume checksum that covers its chunk, in both table copies."""
    disk = volume_disk
    volume_disk.analyse(image, OPAQUE)
    data = bytearray(image)
    volume = image[disk.FIRST * disk.SECTOR:(disk.FIRST + disk.VOLUME_SECTORS) * disk.SECTOR]
    version = disk.choose(volume)["entries"][obj]["current"]
    if version["length"] != len(content) or len(version["chunks"]) != 1:
        raise ValueError("rewrite must keep the single-chunk object length")
    chunk = version["chunks"][0]
    start = (disk.FIRST + disk.DATA_START + chunk * disk.CHUNK_SECTORS) * disk.SECTOR
    data[start:start + len(content)] = content
    checksum = disk.fnv(content)
    for slot in range(2):
        sectors, _ = disk.slot_states(volume, slot)
        try:
            table = disk.decode_table(slot, sectors)
        except ValueError:
            continue  # an invalid copy references nothing the service trusts
        touched = False
        for entry in table["entries"]:
            for candidate in disk.retained(entry):
                if candidate["chunks"] == [chunk]:
                    candidate["content_sum"], candidate["sums"] = checksum, [checksum]
                    touched = True
        if touched:
            for index, sector in enumerate(disk.encode_table(table["generation"], table["request_id"],
                                                             table["entries"], table["collections"],
                                                             table["clocks"])):
                offset = (disk.FIRST + slot * disk.TABLE_SECTORS + index) * disk.SECTOR
                data[offset:offset + disk.SECTOR] = sector
    result = bytes(data)
    oracle = volume_disk.analyse(result, OPAQUE)
    if slot_bytes(oracle, obj) != content:
        raise ValueError("rewritten content not read back by the volume oracle")
    return result


def damaged(raw: bytes) -> bytes:
    """Flip one generation bit; the SHA-256 no longer matches."""
    out = bytearray(raw)
    out[16] ^= 0x01
    return bytes(out)


def newest_slot(slots) -> int:
    records = [decode_record(raw) for raw in slots]
    generations = [r[1] if r[0] == "good" else -1 for r in records]
    return generations.index(max(generations))


def derive(image: bytes, source: str) -> bytes:
    """Step 4-6 inputs from step 3's image; only domain 0 is changed."""
    slots, _ = domain_slots(image, 0)
    objects = SLOT_OBJECTS[0]
    newest = newest_slot(slots)
    if source == "damage-newest":
        return rewrite_object(image, objects[newest], damaged(slots[newest]))
    if source == "damage-both":
        image = rewrite_object(image, objects[0], damaged(slots[0]))
        return rewrite_object(image, objects[1], damaged(slots[1]))
    if source == "schema-2":
        older = 1 - newest
        good = decode_record(slots[older])
        return rewrite_object(image, objects[older], encode_record(good[1], good[2], good[3], schema=2))
    raise ValueError(f"unknown image source {source}")


# --- runner -------------------------------------------------------------------

def require_wiring() -> None:
    """Refuse clearly when the shared kernel/build wiring is absent."""
    cargo = (ROOT / "kernel/Cargo.toml").read_text(encoding="utf-8")
    volume = (ROOT / "kernel/src/m2/m3/volume.rs").read_text(encoding="utf-8")
    build = (ROOT / "tools/build.py").read_text(encoding="utf-8")
    missing = [what for what, ok in (
        ("kernel/Cargo.toml features test-volume-settings[-recover]",
         all(f'{f} = ["native-volume"]' in cargo for f in FEATURE.values())),
        ("kernel/src/m2/m3/volume.rs case()/read_only()/marker() for cases 23 and 24",
         "VOLUME:CASE:SETTINGS:PASS" in volume and "VOLUME:CASE:SETTINGS_RECOVER:PASS" in volume),
        ("tools/build.py VOLUME_SETTINGS_CASES and the settings-app override",
         "VOLUME_SETTINGS_CASES" in build and "settings-volume/build.py" in build)) if not ok]
    if missing:
        raise SystemExit("run_settings: missing shared edit(s): " + "; ".join(missing)
                         + " (see sdk/examples/settings-volume/README.md)")


def check_build(build_dir: Path, case: str, frozen: str) -> dict:
    manifest_path = build_dir / "manifest.json"
    if not manifest_path.is_file():
        raise SystemExit(f"run_settings: no build in {build_dir}; run NOS_BUILD_DIR={build_dir} "
                         f"python3 -B tools/build.py --kernel-feature {FEATURE[case]}")
    manifest = json.loads(manifest_path.read_text())
    if manifest.get("kernel_feature") != FEATURE[case] or manifest.get("source_sha256") != frozen:
        raise SystemExit(f"run_settings: {build_dir} is not a {FEATURE[case]} build of the current source")
    return manifest


def boot(build_dir: Path, run_dir: Path, image: Path, readonly: bool, timeout: int):
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    code, variables = firmware()
    shutil.copytree(build_dir / "esp", run_dir / "esp")
    shutil.copy2(variables, run_dir / "OVMF_VARS.fd")
    shutil.copy2(code, run_dir / "OVMF_CODE.fd")
    rel = Path(os.path.relpath(run_dir, ROOT)).as_posix()
    image_rel = Path(os.path.relpath(image, ROOT)).as_posix()
    args = [qemu, "-machine", qemu_machine(), "-accel", qemu_accel(), "-cpu", "qemu64",
            "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
            "-serial", "stdio", "-nic", "none", "-no-reboot",
            "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
            "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
            "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
            "-drive", f"if=none,id=volume,format=raw,cache=writeback,readonly={'on' if readonly else 'off'},"
                      f"file={image_rel}",
            "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,"
                       "queue-size=8,serial=NOS-M6E-TEST"]
    rc, out, err, timed_out, _, _ = execute(args, env, timeout=timeout)
    version = subprocess.check_output([qemu, "--version"], text=True).strip()
    return args, version, rc, out, err, timed_out


def reported(evidence) -> dict:
    words = ("RECOVERED_REVISION", "RECOVERED_COLLECTION", "RECOVERED_SECOND")
    return {d: tuple(evidence[f"VOLUME:{'AB'[d]}:{word}:"] for word in words) for d in range(DOMAINS)}


def check_records(step: int, image: bytes) -> dict:
    """On-disk records after the writable steps, decoded by the oracle."""
    states = {}
    for domain in range(DOMAINS):
        slots, oracle = domain_slots(image, domain)
        records = [decode_record(raw) for raw in slots]
        if step == 1:
            want = [("good", 1, S1[domain], None), ("good", 2, S2[domain], S1[domain])]
        elif step == 3:
            want = [("good", 3, S1[domain], S2[domain]), ("good", 2, S2[domain], S1[domain])]
        else:
            want = records
        if records != want:
            raise ValueError(f"domain {domain} records {records} != {want}")
        states[domain] = [list(r) for r in records]
    untouched = [o for o in oracle["objects"] if o["object"] not in OPAQUE]
    if any(o["present"] or o["revision"] for o in untouched) or any(
            c["present"] or c["revision"] for c in oracle["collections"]):
        raise ValueError("guest touched an object or collection outside the settings slots")
    if step in (1, 3):
        commits = 2 if step == 1 else 3
        if (oracle["generation"], oracle["clocks"]) != (2 * commits, [commits, commits]):
            raise ValueError(f"unexpected volume generation/clocks {oracle['generation']} {oracle['clocks']}")
    states["generation"] = oracle["generation"]
    return states


def run_step(step: int, build_dir: Path, chain: Path, previous: Path | None, frozen: str, timeout: int):
    case, source, want = STEPS[step]
    manifest = check_build(build_dir, case, frozen)
    run_dir = build_dir / "run" / f"settings-{step}-{uuid.uuid4().hex[:10]}"
    run_dir.mkdir(parents=True)
    image = chain / f"step{step}.img"
    if source == "fresh":
        volume_disk.create_image(image, "empty")
    elif source == "previous":
        shutil.copyfile(previous, image)
    else:
        image.write_bytes(derive(previous.read_bytes(), source))
    before = sha256(image)
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    readonly = case == "settings-recover"
    args, version, rc, out, err, timed_out = boot(build_dir, run_dir, image, readonly, timeout)
    duration = time.monotonic() - start
    (run_dir / "serial.txt").write_text(out, encoding="utf-8")
    (run_dir / "stderr.txt").write_text(err, encoding="utf-8")
    after = sha256(image)
    report = dict(schema_version=1, step=step, case=case, image_source=source, kernel_feature=FEATURE[case],
                  started_utc=started, duration_s=duration, raw_qemu_exit=rc, timed_out=timed_out,
                  command=args, qemu_version=version, source_sha256=frozen,
                  kernel_sha256=manifest["kernel"]["sha256"], app=manifest.get("volume_docs"),
                  before_sha256=before, after_sha256=after, run_directory=str(run_dir),
                  expected={str(d): list(v) for d, v in want.items()}, passed=False)
    try:
        markers = BOOT + ("VOLUME:MANIFEST:PASS", "VOLUME:READY", f"VOLUME:CASE:{MARKER[case]}:PASS",
                          "VOLUME:RESOURCES:PASS", "VOLUME:PASS", "M1:PASS")
        if controlled(out) != list(markers):
            raise ValueError(f"marker sequence mismatch: {controlled(out)}")
        if any(marker in line for line in out.splitlines() for marker in FAILURES):
            raise ValueError("failure marker present")
        if rc != 33 or timed_out:
            raise ValueError(f"raw exit {rc}, timeout {timed_out}")
        if readonly and after != before:
            raise ValueError("read-only step changed the image")
        evidence = numbers(out)
        got = reported(evidence)
        if got != want:
            raise ValueError(f"report words {got} != expected {want}")
        raw = image.read_bytes()
        oracle_view = {d: expected_recovery(domain_slots(raw, d)[0]) for d in range(DOMAINS)}
        if oracle_view != want:
            raise ValueError(f"host oracle {oracle_view} disagrees with expected {want}")
        report.update(evidence=evidence, reported={str(d): list(v) for d, v in got.items()},
                      records=check_records(step, raw), passed=source_hash() == frozen)
        if not report["passed"]:
            raise ValueError("source changed during run")
    except Exception as error:  # recorded in the result, then fail closed
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(run_dir / "result.json", report)
    print(f"U05-N1 step {step} {case} ({source}): passed={report['passed']} raw={rc} "
          f"duration={duration:.1f}s before={before[:16]} after={after[:16]} {run_dir}", flush=True)
    if "error" in report:
        print(report["error"], flush=True)
    return report, image


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--settings-build", type=Path, default=BUILD,
                        help="NOS_BUILD_DIR of a test-volume-settings build")
    parser.add_argument("--recover-build", type=Path, default=BUILD,
                        help="NOS_BUILD_DIR of a test-volume-settings-recover build")
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    args = parser.parse_args()
    if not 0 < args.timeout <= MAX_TIMEOUT:
        raise SystemExit("invalid timeout")
    require_wiring()
    frozen = source_hash()
    builds = {"settings": args.settings_build.resolve(), "settings-recover": args.recover_build.resolve()}
    for case, path in builds.items():
        check_build(path, case, frozen)
    chain = builds["settings"] / "run" / f"settings-chain-{uuid.uuid4().hex[:10]}"
    chain.mkdir(parents=True)
    results, previous, step3 = [], None, None
    for step, (case, source, _) in STEPS.items():
        parent = step3 if source not in ("fresh", "previous") else previous
        report, image = run_step(step, builds[case], chain, parent, frozen, args.timeout)
        results.append({key: report.get(key) for key in (
            "step", "case", "image_source", "passed", "raw_qemu_exit", "before_sha256", "after_sha256",
            "reported", "expected", "run_directory", "error")})
        if not report["passed"]:
            break
        previous = image
        if step == 3:
            step3 = image
    passed = len(results) == len(STEPS) and all(r["passed"] for r in results) and source_hash() == frozen
    save_json(chain / "summary.json", dict(schema_version=1, source_sha256=frozen, passed=passed,
                                           steps=results, builds={k: str(v) for k, v in builds.items()}))
    print(f"U05-N1 settings sequence: passed={passed} {chain / 'summary.json'}")
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
