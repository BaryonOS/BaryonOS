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
           "recover-cut": "test-volume-recover-cut", "corrupt": "test-volume-corrupt"}
CUT_MARKER = {"cut-data": "VOLUME:CUT:DATA", "cut-table": "VOLUME:CUT:TABLE"}
CASE_MARKER = {"normal": "NORMAL", "recover": "RECOVER", "recover-cut": "RECOVER_CUT", "corrupt": "CORRUPT"}
READ_ONLY = ("recover", "recover-cut", "corrupt")
MAX_TIMEOUT = 600

# Expected final state of the normal fixture (object, revision, content revision, length, name).
FINAL = {
    1: (2, 1, 129, "Notitie ✓"), 2: (1, 1, 0, "Leeg"), 3: (40, 40, 5 * 4096, "Concept"),
    4: (1, 1, 5000, "Herstart"), 5: (1, 1, 65536, "Archief"),
    8: (1, 1, 4 * 4096, "Foto-index"), 9: (1, 1, 300, "Lijst"), 10: (20, 20, 4096, "Logboek"),
    11: (1, 1, 65536, "Groot 1"), 12: (1, 1, 65536, "Groot 2"),
}
TOMBSTONES = {0: 2}


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
    if case in ("normal", "cut-data", "cut-table", "corrupt"):
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
                  volume_directory=str(volume_dir), before_sha256=before, after_sha256=after,
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
            check_state(case, oracle, evidence)
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


def check_state(case: str, oracle, evidence):
    objects = oracle["objects"]
    if case in ("normal", "recover"):
        for obj, item in enumerate(objects):
            if obj in FINAL:
                revision, content_revision, length, name = FINAL[obj]
                if (item["present"], item["revision"], item.get("content_revision"), item["length"], item["name"]) \
                        != (True, revision, content_revision, length, name):
                    raise ValueError(f"oracle object {obj} mismatch: {item}")
            elif obj in TOMBSTONES:
                if item["present"] or item["revision"] != TOMBSTONES[obj]:
                    raise ValueError(f"oracle tombstone {obj} mismatch")
            elif item["present"] or item["revision"] != 0:
                raise ValueError(f"unexpected object {obj}")
        # 47 commits by A, 24 by B; A holds 24 chunks, B 38 (see FINAL).
        if oracle["generation"] != 71 or oracle["chunks_used"] != 62:
            raise ValueError(f"oracle generation/usage {oracle['generation']}/{oracle['chunks_used']}")
        if [evidence["VOLUME:A:DIGEST:"], evidence["VOLUME:B:DIGEST:"]] != oracle["digests"]:
            raise ValueError("guest-reported digests differ from the disk oracle")
        if evidence["VOLUME:SERVICE:GENERATION:"] != 71:
            raise ValueError("service generation differs")
        if case == "normal" and not (evidence["VOLUME:SERVICE:COMMITS:"] == 71
                                     and evidence["VOLUME:SERVICE:CHUNK_ALLOCATIONS:"] > 64):
            raise ValueError("space reuse evidence missing")
    elif case in ("cut-data", "cut-table", "recover-cut"):
        first = objects[0]
        if (first["present"], first["revision"], first["length"], first.get("content_revision")) != (True, 1, 9000, 1):
            raise ValueError(f"interrupted second commit must leave revision 1 intact: {first}")
        if any(item["present"] or item["revision"] for item in objects[1:]):
            raise ValueError("unexpected objects after cut")
        if case == "recover-cut":
            if evidence["VOLUME:A:RECOVERED_REVISION:"] != 1:
                raise ValueError("guest recovered a different revision than the oracle")
            if [evidence["VOLUME:A:DIGEST:"], evidence["VOLUME:B:DIGEST:"]] != oracle["digests"]:
                raise ValueError("guest-reported digests differ from the disk oracle")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=sorted(FEATURE), required=True)
    parser.add_argument("--volume", type=Path)
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    args = parser.parse_args()
    raise SystemExit(0 if run_volume(args.case, args.volume, timeout=args.timeout)[0]["passed"] else 1)
