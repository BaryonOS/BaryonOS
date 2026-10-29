#!/usr/bin/env python3
"""M6e QEMU runner: fresh guarded volume images, real host kills and a cold
read-only recovery of exactly the preserved image, checked by the host oracle."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import queue
import re
import shutil
import subprocess
import threading
import time
import uuid
from common import ROOT, BUILD, environment, firmware, qemu_machine, save_json, sha256, source_hash
from build import VOLUME_FEATURES
import volume_disk

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")
FAILURES = ("PANIC", "FAIL", "M5:FAIL")
FEATURE = {"normal": "native-volume", "recover": "test-volume-recover",
           "cut-data": "test-volume-cut-data", "cut-table": "test-volume-cut-table",
           "cut-txn": "test-volume-cut-txn", "recover-cut": "test-volume-recover-cut",
           "corrupt": "test-volume-corrupt"}
CUT_MARKER = {"cut-data": "VOLUME:CUT:DATA", "cut-table": "VOLUME:CUT:TABLE", "cut-txn": "VOLUME:CUT:TXN"}
CASE_MARKER = {"normal": "NORMAL", "recover": "RECOVER", "recover-cut": "RECOVER_CUT", "corrupt": "CORRUPT"}
READ_ONLY = ("recover", "recover-cut", "corrupt")
MAX_TIMEOUT = 600

# Expected final state of the normal fixture: object -> (revision, content
# revision, length, name, collection, content revisions of the history).
FINAL = {
    1: (3, 1, 129, "Notitie ✓", 2, []), 2: (2, 1, 0, "Leeg", 1, []),
    3: (42, 38, 5 * 4096, "Concept", 0, []), 4: (4, 1, 5000, "Herstart", 0, []),
    5: (1, 1, 65536, "Archief", 0, []),
    8: (2, 1, 4 * 4096, "Foto-index", 1, []), 9: (1, 1, 300, "Lijst", 0, []),
    10: (21, 20, 4096, "Logboek", 0, []), 11: (1, 1, 65536, "Groot 1", 0, []),
    12: (1, 1, 65536, "Groot 2", 0, []),
}
TOMBSTONES = {0: 4}
# Present collections per (domain, index) -> name; all others never existed.
COLLECTIONS = {(0, 1): "Werk", (0, 2): "Privé", (1, 1): "Fotos"}
# 57 commits by A and 27 by B; A holds 24 chunks and B 38 (history included).
GENERATION, CHUNKS_USED = 84, 62
DOMAIN_CLOCKS = [57, 27]


def numbers(stdout: str) -> dict[str, int]:
    found = {}
    for line in stdout.splitlines():
        match = re.fullmatch(r"(VOLUME:[A-Z_:]+:)([0-9]{20})", line)
        if match:
            if match.group(1) in found:
                raise ValueError(f"duplicate evidence {match.group(1)}")
            found[match.group(1)] = int(match.group(2))
    return found


def expected_markers(case: str) -> tuple[str, ...]:
    if case in CUT_MARKER:
        return BOOT + ("VOLUME:MANIFEST:PASS", CUT_MARKER[case])
    return BOOT + ("VOLUME:MANIFEST:PASS", "VOLUME:READY", f"VOLUME:CASE:{CASE_MARKER[case]}:PASS",
                   "VOLUME:RESOURCES:PASS", "VOLUME:PASS", "M1:PASS")


def controlled(stdout: str) -> list[str]:
    keep = BOOT + ("VOLUME:MANIFEST:PASS", "VOLUME:READY", "VOLUME:RESOURCES:PASS", "VOLUME:PASS", "M1:PASS")
    return [line for line in stdout.splitlines()
            if line in keep or line.startswith(("VOLUME:CASE:", "VOLUME:CUT:"))]


def execute(args, env, *, cut_marker=None, timeout=MAX_TIMEOUT):
    process = subprocess.Popen(args, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    out, err, lines = [], [], queue.Queue()

    def reader(pipe, sink, notify):
        for raw in iter(pipe.readline, b""):
            sink.append(raw)
            if notify:
                lines.put(raw.decode("utf-8", "replace").rstrip("\r\n"))
        pipe.close()

    threads = [threading.Thread(target=reader, args=(process.stdout, out, True), daemon=True),
               threading.Thread(target=reader, args=(process.stderr, err, False), daemon=True)]
    for thread in threads:
        thread.start()
    deadline = time.monotonic() + timeout
    timed_out = killed = running_before_kill = False
    while True:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            timed_out = True
            process.kill()
            break
        try:
            line = lines.get(timeout=min(remaining, 0.5))
        except queue.Empty:
            if process.poll() is not None and lines.empty():
                break
            continue
        if cut_marker is not None and line == cut_marker:
            running_before_kill = process.poll() is None
            if running_before_kill:
                process.kill()
                killed = True
            break
    process.wait()
    for thread in threads:
        thread.join(timeout=10)
    return (process.returncode, b"".join(out).decode("utf-8", "replace"),
            b"".join(err).decode("utf-8", "replace"), timed_out, killed, running_before_kill)


def run_volume(case: str, volume_dir: Path | None = None, *, timeout=MAX_TIMEOUT):
    if case not in FEATURE or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("unknown M6e case or invalid timeout")
    frozen = source_hash()
    manifest = json.loads((BUILD / "manifest.json").read_text())
    if manifest.get("kernel_feature") != FEATURE[case] or manifest.get("source_sha256") != frozen:
        raise ValueError(f"build {FEATURE[case]} from the current source first")
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    code, variables = firmware()
    run_dir = BUILD / "run" / f"volume-{case}-{uuid.uuid4().hex[:10]}"
    run_dir.mkdir(parents=True)
    shutil.copytree(BUILD / "esp", run_dir / "esp")
    shutil.copy2(variables, run_dir / "OVMF_VARS.fd")
    shutil.copy2(code, run_dir / "OVMF_CODE.fd")
    if case in ("normal", "cut-data", "cut-table", "cut-txn", "corrupt"):
        if volume_dir is not None:
            raise ValueError("writable and corrupt cases always start from a fresh image")
        volume_dir = BUILD / "run" / f"volume-image-{uuid.uuid4().hex[:10]}"
        volume_dir.mkdir(parents=True)
        info = volume_disk.create_image(volume_dir / "volume.img", "corrupt" if case == "corrupt" else "empty")
    else:
        if volume_dir is None:
            raise ValueError("recovery requires the preserved image of a previous run")
        volume_dir = volume_dir.resolve(strict=True)
        volume_dir.relative_to((BUILD / "run").resolve())
        info = json.loads((volume_dir / "volume.json").read_text())
        last = json.loads((volume_dir / "last-stage.json").read_text())
        if (last.get("passed") is not True or last.get("after_sha256") != sha256(volume_dir / "volume.img")
                or last.get("source_sha256") != frozen):
            raise ValueError("recovery requires the exact image left by a passing previous stage")
    parent = None
    if case not in ("normal", "cut-data", "cut-table", "cut-txn", "corrupt"):
        parent = last["case"]
        if (case, parent) not in (("recover", "normal"), ("recover-cut", "cut-data"),
                                  ("recover-cut", "cut-table"), ("recover-cut", "cut-txn")):
            raise ValueError(f"{case} cannot follow {parent}")
    image = volume_dir / "volume.img"
    if image.stat().st_nlink != 1:
        raise ValueError("volume image must have exactly one link")
    before = sha256(image)
    rel = run_dir.relative_to(ROOT).as_posix()
    image_rel = image.relative_to(ROOT).as_posix()
    readonly = "on" if case in READ_ONLY else "off"
    args = [qemu, "-machine", qemu_machine(), "-accel", "tcg,thread=single", "-cpu", "qemu64",
            "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
            "-serial", "stdio", "-nic", "none", "-no-reboot",
            "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
            "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
            "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
            "-drive", f"if=none,id=volume,format=raw,cache=writeback,readonly={readonly},file={image_rel}",
            "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-M6E-TEST"]
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    rc, out, err, timed_out, killed, running = execute(
        args, env, cut_marker=CUT_MARKER.get(case), timeout=timeout)
    duration = time.monotonic() - start
    (run_dir / "serial.txt").write_text(out, encoding="utf-8")
    (run_dir / "stderr.txt").write_text(err, encoding="utf-8")
    after = sha256(image)
    shutil.copy2(image, run_dir / "volume.after.img")
    report = dict(schema_version=1, case=case, kernel_feature=FEATURE[case], started_utc=started,
                  duration_s=duration, timeout_s=timeout, timed_out=timed_out, raw_qemu_exit=rc,
                  host_kill=killed, running_before_kill=running, command=args,
                  qemu_version=subprocess.check_output([qemu, "--version"], text=True).strip(),
                  source_fingerprint_version=3, source_sha256=frozen, volume_identity=info["identity"],
                  parent_case=parent, volume_directory=str(volume_dir), before_sha256=before, after_sha256=after,
                  run_directory=str(run_dir), kernel_sha256=manifest["kernel"]["sha256"],
                  volume_docs=manifest.get("volume_docs"), stdout=out, stderr=err, passed=False)
    try:
        expected = expected_markers(case)
        if controlled(out) != list(expected):
            raise ValueError(f"marker sequence mismatch: {controlled(out)}")
        if any(marker in line for line in out.splitlines() for marker in FAILURES):
            raise ValueError("failure marker present")
        if case in CUT_MARKER:
            if not (killed and running and not timed_out):
                raise ValueError("cut was not followed by a real host kill of a live guest")
        elif rc != 33 or timed_out:
            raise ValueError(f"raw exit {rc}, timeout {timed_out}")
        if case in READ_ONLY and after != before:
            raise ValueError("read-only case changed the image")
        evidence = numbers(out)
        oracle = None
        if case == "corrupt":
            try:
                volume_disk.analyse(image.read_bytes())
                raise ValueError("oracle accepted the corrupt fixture")
            except ValueError as error:
                if "accepted" in str(error):
                    raise
                oracle = {"refused": str(error)}
        else:
            oracle = volume_disk.analyse(image.read_bytes())
            check_state(case, oracle, evidence, parent)
        report.update(evidence=evidence, oracle=oracle, passed=source_hash() == frozen)
        if not report["passed"]:
            raise ValueError("source changed during run")
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(run_dir / "result.json", report)
    save_json(volume_dir / "last-stage.json", dict(case=case, passed=report["passed"],
              run_directory=str(run_dir), after_sha256=after, source_sha256=frozen))
    print(f"M6e {case}: passed={report['passed']} raw={rc} duration={duration:.1f}s {run_dir}")
    if "error" in report:
        print(report["error"])
    return report, volume_dir


def current(item):
    """The present version of an oracle object (first retained version)."""
    if not item["present"] or not item["versions"]:
        raise ValueError(f"object {item['object']} has no present version")
    return item["versions"][0]


def check_collections(oracle, expected):
    for item in oracle["collections"]:
        name = expected.get((item["domain"], item["index"]))
        if name is None:
            if item["present"] or item["revision"]:
                raise ValueError(f"unexpected collection {item}")
        elif not item["present"] or item["name"] != name:
            raise ValueError(f"collection mismatch {item}")


def check_document(item, revision, content_revision, length, name, collection, history):
    version = current(item)
    found = (item["revision"], version["content_revision"], version["length"], version["name"],
             item["collection"], [v["content_revision"] for v in item["versions"][1:]])
    if found != (revision, content_revision, length, name, collection, history):
        raise ValueError(f"oracle object {item['object']} mismatch: {found}")


def torn_sectors(oracle, in_table: bool) -> int:
    """Sectors of the interrupted generation g + 1 in the other table copy.

    A kill inside the table write must leave a real torn attempt there: a
    non-empty, incomplete prefix of g + 1 sectors followed by the previous
    copy (g - 1, or never-written sectors when g == 1). A kill during the
    content writes must leave that copy untouched.
    """
    g = oracle["generation"]
    other = oracle["slots"][(g + 1) % 2]
    count = sum(1 for sector in other if sector == g + 1)
    if not in_table:
        if count or any(sector != g - 1 for sector in other):
            raise ValueError(f"content cut touched the other table copy: {other}")
        return 0
    prefix = other[:count] == [g + 1] * count
    if not (0 < count < len(other) and prefix and all(s == g - 1 for s in other[count:])):
        raise ValueError(f"no torn table write in the other copy: {other}")
    return count


def check_state(case: str, oracle, evidence, parent: str | None = None):
    objects = oracle["objects"]
    if case in ("normal", "recover"):
        for obj, item in enumerate(objects):
            if obj in FINAL:
                check_document(item, *FINAL[obj])
            elif obj in TOMBSTONES:
                if item["present"] or item["revision"] != TOMBSTONES[obj] or item["versions"]:
                    raise ValueError(f"oracle tombstone {obj} mismatch")
            elif item["present"] or item["revision"] != 0:
                raise ValueError(f"unexpected object {obj}")
        check_collections(oracle, COLLECTIONS)
        if (oracle["generation"], oracle["chunks_used"]) != (GENERATION, CHUNKS_USED):
            raise ValueError(f"oracle generation/usage {oracle['generation']}/{oracle['chunks_used']}")
        if oracle["clocks"] != DOMAIN_CLOCKS:
            raise ValueError(f"oracle domain clocks {oracle['clocks']}")
        if [evidence["VOLUME:A:DIGEST:"], evidence["VOLUME:B:DIGEST:"]] != oracle["digests"]:
            raise ValueError("guest-reported digests differ from the disk oracle")
        if evidence["VOLUME:SERVICE:GENERATION:"] != GENERATION:
            raise ValueError("service generation differs")
        if case == "normal" and not (evidence["VOLUME:SERVICE:COMMITS:"] == GENERATION
                                     and evidence["VOLUME:SERVICE:CHUNK_ALLOCATIONS:"] > 64
                                     and evidence["VOLUME:SERVICE:TRANSACTIONS:"] == 1
                                     and evidence["VOLUME:SERVICE:RESTORES:"] == 2):
            raise ValueError("space reuse, transaction or restore evidence missing")
    elif case in ("cut-data", "cut-table", "cut-txn", "recover-cut"):
        txn = "cut-txn" in (case, parent)
        check_document(objects[0], 1, 1, 9000, "Voor de crash", 0, [])
        if txn:
            check_document(objects[1], 1, 1, 300, "Tweede", 0, [])
        rest = objects[2:] if txn else objects[1:]
        if any(item["present"] or item["revision"] for item in rest):
            raise ValueError("unexpected objects after cut")
        check_collections(oracle, {(0, 1): "Werk"} if txn else {})
        if oracle["generation"] != (3 if txn else 1):
            raise ValueError(f"interrupted commit must leave the previous generation: {oracle['generation']}")
        oracle["torn_sectors"] = torn_sectors(oracle, "cut-data" not in (case, parent))
        if case == "recover-cut":
            if (evidence["VOLUME:A:RECOVERED_REVISION:"], evidence["VOLUME:A:RECOVERED_COLLECTION:"],
                    evidence["VOLUME:A:RECOVERED_SECOND:"]) != (1, 0, 1 if txn else 0):
                raise ValueError("guest recovered a different state than the oracle")
            if [evidence["VOLUME:A:DIGEST:"], evidence["VOLUME:B:DIGEST:"]] != oracle["digests"]:
                raise ValueError("guest-reported digests differ from the disk oracle")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=sorted(FEATURE), required=True)
    parser.add_argument("--volume", type=Path)
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    args = parser.parse_args()
    raise SystemExit(0 if run_volume(args.case, args.volume, timeout=args.timeout)[0]["passed"] else 1)
