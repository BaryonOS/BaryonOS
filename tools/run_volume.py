#!/usr/bin/env python3
"""M6e QEMU runner: fresh guarded volume images, real host kills and a cold
read-only recovery of exactly the preserved image, checked by the host oracle.
The S09 archive stages add an export into a disjoint archive range of the same
disposable image, a restore, a damaged-archive refusal and a restore into a
domain with conflicting names and collections on fresh images, an export
into a target too small for the archive that must fail without a usable archive,
and the D6(a) user export of that archive to ustar, read back with Python tarfile.
The archive-disk stages attach a test-only second virtio-blk disk at 00:06.0:
export to it, a cold restore from it alone, an absent and a too-small disk, a
grant of the wrong disk, and a power cut during an export over an older
archive followed by a cold verify. The S09 L2 install stages are read with the
v3-capable table reader (volume_disk.choose(allow_v3=True)): an INSTALL of
fixture versions and an INSTALL_COLLECTION, their cold read-only re-check, and a
power cut inside the INSTALL_COMMIT table write, after which the object must be
exactly never-used or fully installed."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import hashlib
import io
import json
from pathlib import Path
import queue
import re
import shutil
import struct
import subprocess
import tarfile
import threading
import time
import uuid
from common import ROOT, BUILD, environment, firmware, qemu_accel, qemu_machine, save_json, sha256, source_hash
import volume_archive
import volume_disk

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")
FAILURES = ("PANIC", "FAIL", "M5:FAIL")
FEATURE = {"normal": "native-volume", "recover": "test-volume-recover",
           "cut-data": "test-volume-cut-data", "cut-table": "test-volume-cut-table",
           "cut-txn": "test-volume-cut-txn", "recover-cut": "test-volume-recover-cut",
           "corrupt": "test-volume-corrupt", "foreign": "test-volume-foreign",
           "archive-export": "test-volume-archive-export",
           "archive-restore": "test-volume-archive-restore",
           "archive-corrupt": "test-volume-archive-corrupt",
           "archive-conflict": "test-volume-archive-conflict",
           "archive-full": "test-volume-archive-full",
           "archive-ustar": "test-volume-archive-ustar",
           **{case: f"test-volume-{case}" for case in (
               "archive-disk-export", "archive-disk-restore", "archive-disk-absent", "archive-disk-small",
               "archive-disk-wrong", "archive-disk-cut", "archive-disk-verify",
               "install", "install-recover", "install-cut")}}
CUT_MARKER = {"cut-data": "VOLUME:CUT:DATA", "cut-table": "VOLUME:CUT:TABLE", "cut-txn": "VOLUME:CUT:TXN",
              "archive-disk-cut": "VOLUME:CUT:ARCHIVE", "install-cut": "VOLUME:CUT:INSTALL"}
CASE_MARKER = {"normal": "NORMAL", "recover": "RECOVER", "recover-cut": "RECOVER_CUT",
               "corrupt": "CORRUPT", "foreign": "CORRUPT", "archive-export": "ARCHIVE_EXPORT",
               "archive-restore": "ARCHIVE_RESTORE", "archive-corrupt": "ARCHIVE_CORRUPT",
               "archive-conflict": "ARCHIVE_CONFLICT", "archive-full": "ARCHIVE_FULL",
               "archive-ustar": "ARCHIVE_USTAR", "archive-disk-export": "ARCHIVE_DISK_EXPORT",
               "archive-disk-restore": "ARCHIVE_DISK_RESTORE", "archive-disk-absent": "ARCHIVE_DISK_ABSENT",
               "archive-disk-small": "ARCHIVE_DISK_SMALL", "archive-disk-wrong": "ARCHIVE_DISK_WRONG",
               "archive-disk-verify": "ARCHIVE_DISK_VERIFY", "install": "INSTALL",
               "install-recover": "INSTALL_RECOVER"}
READ_ONLY = ("recover", "recover-cut", "corrupt", "foreign", "install-recover")
# Cases that always start from a fresh image; every other case needs the image
# its allowed parent left.
FRESH = ("normal", "cut-data", "cut-table", "cut-txn", "corrupt", "foreign", "install", "install-cut")
ALLOWED_PARENTS = frozenset({
    ("recover", "normal"), ("recover-cut", "cut-data"),
    ("recover-cut", "cut-table"), ("recover-cut", "cut-txn"),
    ("archive-export", "recover"), ("archive-restore", "archive-export"),
    ("archive-corrupt", "archive-export"), ("archive-conflict", "archive-export"),
    ("archive-full", "archive-export"), ("archive-ustar", "archive-export"),
    ("archive-disk-export", "archive-export"), ("archive-disk-absent", "archive-export"),
    ("archive-disk-small", "archive-export"),
    ("archive-disk-restore", "archive-disk-export"),
    ("archive-disk-wrong", "archive-disk-export"),
    ("archive-disk-cut", "archive-disk-export"),
    ("archive-disk-verify", "archive-disk-cut"),
    ("install-recover", "install"), ("recover-cut", "install-cut")})
MAX_TIMEOUT = 600
# S09 archive range; must equal ARCHIVE_FIRST_SECTOR/ARCHIVE_SECTORS in the
# kernel volume profile and ARCHIVE_SECTORS in the portal app.
ARCHIVE_FIRST, ARCHIVE_SECTORS, SECTOR = 1000, 1040, 512
ARCHIVE_SPAN = slice(ARCHIVE_FIRST * SECTOR, (ARCHIVE_FIRST + ARCHIVE_SECTORS) * SECTOR)
# A real sector follows the range (the portal's scope probe must not be refused
# by the device capacity instead), and the largest v1 archive fits it.
assert ARCHIVE_FIRST + ARCHIVE_SECTORS < volume_disk.IMAGE_SECTORS
assert volume_disk.FIRST + volume_disk.VOLUME_SECTORS <= ARCHIVE_FIRST
assert (volume_archive.HEADER + volume_archive.TRAILER + volume_archive.COLLECTIONS * volume_archive.COLLECTION_RECORD
        + volume_archive.OBJECTS * (volume_archive.OBJECT_RECORD + volume_archive.MAX_LENGTH)
        <= ARCHIVE_SECTORS * SECTOR)
RESTORED = ("archive-restore", "archive-corrupt", "archive-conflict")
# archive-full: the portal's scope is cut to FULL_SECTORS (FULL_SECTORS in the
# kernel volume profile and the portal app), below the fixture archive's length.
FULL_SECTORS = 100
assert FULL_SECTORS < ARCHIVE_SECTORS
FULL_END = ARCHIVE_SPAN.start + FULL_SECTORS * SECTOR
# archive-ustar: the portal exports into the lower half of the range and writes
# the ustar conversion of that archive into the upper half (USTAR_FIRST in the portal app).
USTAR_FIRST = ARCHIVE_SECTORS // 2
USTAR_START = ARCHIVE_SPAN.start + USTAR_FIRST * SECTOR
# Domain-0 state the portal writes before the archive-conflict restore (mirrors
# portal() in sdk/examples/volume-docs/src/app.rs): object -> (name, length),
# content = fixture(object, 1, length); collection index -> name.
CONFLICT_OBJECTS = {3: ("Bezet", 77), 6: ("Herstart", 33)}
# S09 test-only second disk (archive.img next to volume.img), attached at
# 00:06.0. The portal's scope on it starts at sector 0 and has the device-0
# scope sizes, so it ends below the disk: ARCHIVE_SECTORS of a 2048-sector
# disk, or FULL_SECTORS of a too-small disk (the driver needs >= 8 sectors).
DISK_CASES = ("archive-disk-export", "archive-disk-restore", "archive-disk-absent", "archive-disk-small",
              "archive-disk-wrong", "archive-disk-cut", "archive-disk-verify")
DISK1_READ_ONLY = ("archive-disk-restore", "archive-disk-wrong", "archive-disk-verify")
DISK1_SECTORS, SMALL_DISK_SECTORS = 2048, FULL_SECTORS + 8
assert ARCHIVE_SECTORS < DISK1_SECTORS and 8 <= FULL_SECTORS < SMALL_DISK_SECTORS
DISK1_SPAN = slice(0, ARCHIVE_SECTORS * SECTOR)
# Completed device-1 writes before the power cut (ARCHIVE_CUT_WRITES in the kernel).
ARCHIVE_CUT_WRITES = 20
# The domain-0 collection the portal adds before that export (portal() in
# sdk/examples/volume-docs/src/app.rs), so the new archive differs from sector 0.
CUT_COLLECTION = (3, "Nieuw")
CUT_COLLECTION_KEY = (0, CUT_COLLECTION[0])
CONFLICT_COLLECTIONS = {1: "Werk", 2: "Anders"}

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

# S09 L2 INSTALL fixture, mirrored by INSTALL_* in sdk/examples/volume-docs/src/app.rs
# (tests/test_volume_runner.py checks both copies agree). Domain 0 first installs
# collection INSTALL_COLLECTION = (index, name, origin), then object INSTALL_OBJECT
# with INSTALL_VERSIONS = ((name, length), ...), current first: version i has
# revision n - i, content fixture(INSTALL_OBJECT, n - i, length) and origin
# INSTALL_ORIGIN_0 - i; INSTALL_ORIGIN = (origin_revision, created, modified);
# INSTALL_META = (tags, kind, collection).
INSTALL_OBJECT = 0
INSTALL_VERSIONS = (("Versie drie", 5000), ("Versie twee", 300), ("Versie een", 0))
INSTALL_ORIGIN = (40, 7, 31)
INSTALL_ORIGIN_0 = 40
INSTALL_META = (0x5A, 2, 1)
INSTALL_COLLECTION = (1, "Import", 12)
# Domain-0 commits of that flow (INSTALL_COLLECTION, INSTALL_COMMIT): the
# generation and domain-0 clock of the installed state; one less is never-used.
INSTALL_COMMITS = 2
INSTALL_CASES = ("install", "install-recover", "install-cut")


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


def execute(args, env, *, cut_marker=None, timeout=MAX_TIMEOUT, on_line=None):
    """Run QEMU with a hard deadline. `on_line(line, process)` (optional) sees
    each complete serial line in order; an exception from it kills QEMU and is
    re-raised after the pipes are drained, so a harness never continues blind."""
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
    hook_error = None
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
        if on_line is not None:
            try:
                on_line(line, process)
            except Exception as error:  # noqa: BLE001 - re-raised below
                hook_error = error
                process.kill()
                break
        if cut_marker is not None and line == cut_marker:
            running_before_kill = process.poll() is None
            if running_before_kill:
                process.kill()
                killed = True
            break
    process.wait()
    for thread in threads:
        thread.join(timeout=10)
    stdout, stderr = b"".join(out), b"".join(err)
    if hook_error is not None:
        hook_error.partial_stdout = stdout.decode("utf-8", "replace")
        hook_error.partial_stderr = stderr.decode("utf-8", "replace")
        raise hook_error
    return (process.returncode, stdout.decode("utf-8", "replace"),
            stderr.decode("utf-8", "replace"), timed_out, killed, running_before_kill)


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
    if case in FRESH:
        if volume_dir is not None:
            raise ValueError("writable and corrupt cases always start from a fresh image")
        volume_dir = BUILD / "run" / f"volume-image-{uuid.uuid4().hex[:10]}"
        volume_dir.mkdir(parents=True)
        variant = case if case in ("corrupt", "foreign") else "empty"
        info = volume_disk.create_image(volume_dir / "volume.img", variant)
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
        if (volume_dir / "archive.img").exists() and last.get("archive_after_sha256") != sha256(
                volume_dir / "archive.img"):
            raise ValueError("the archive disk differs from the one left by the previous stage")
    parent = None
    if case not in FRESH:
        parent = last["case"]
        if (case, parent) not in ALLOWED_PARENTS:
            raise ValueError(f"{case} cannot follow {parent}")
    archive_flip = archive_reference = None
    disk1 = None
    if case in DISK_CASES:
        parent_dir = volume_dir
        volume_dir = BUILD / "run" / f"volume-image-{uuid.uuid4().hex[:10]}"
        volume_dir.mkdir(parents=True)
        if case in ("archive-disk-export", "archive-disk-absent", "archive-disk-small"):
            # Disk 0: the recovered fixture without its archive (as for archive-full);
            # disk 1: a fresh disk of canaries, or none at all.
            exported = (parent_dir / "volume.img").read_bytes()
            archive_reference = stored_archive(exported)
            fresh = outside_archive(exported)
            (volume_dir / "volume.img").write_bytes(fresh)
            info = dict(info, variant=case, sha256=hashlib.sha256(fresh).hexdigest())
            if case != "archive-disk-absent":
                disk1 = disk1_image(DISK1_SECTORS if case == "archive-disk-export" else SMALL_DISK_SECTORS)
        elif case == "archive-disk-cut":
            # Both disks of the export stage: the volume, and its archive as the older one.
            shutil.copyfile(parent_dir / "volume.img", volume_dir / "volume.img")
            disk1 = (parent_dir / "archive.img").read_bytes()
            info = dict(info, variant=case)
        else:
            # Source loss: a fresh empty disk 0 and only the parent's disk 1.
            info = volume_disk.create_image(volume_dir / "volume.img", "empty")
            disk1 = (parent_dir / "archive.img").read_bytes()
            info = dict(info, variant=case)
        if disk1 is not None:
            (volume_dir / "archive.img").write_bytes(disk1)
        (volume_dir / "volume.json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
    elif case in ("archive-full", "archive-ustar"):
        # The export stage's input again: the recovered fixture whose archive
        # range holds its canaries and no archive. The export stage's archive
        # is the reference for the bytes the too-small target may receive and
        # for the archive the ustar stage exports again.
        exported = (volume_dir / "volume.img").read_bytes()
        archive_reference = stored_archive(exported)
        fresh = outside_archive(exported)
        volume_dir = BUILD / "run" / f"volume-image-{uuid.uuid4().hex[:10]}"
        volume_dir.mkdir(parents=True)
        (volume_dir / "volume.img").write_bytes(fresh)
        info = dict(info, variant=case, sha256=hashlib.sha256(fresh).hexdigest())
        (volume_dir / "volume.json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
    elif case in RESTORED:
        # Source loss: a fresh image that holds only the exported archive range.
        exported = (volume_dir / "volume.img").read_bytes()
        volume_dir = BUILD / "run" / f"volume-image-{uuid.uuid4().hex[:10]}"
        volume_dir.mkdir(parents=True)
        info = volume_disk.create_image(volume_dir / "volume.img", "empty")
        fresh = bytearray((volume_dir / "volume.img").read_bytes())
        fresh[ARCHIVE_SPAN] = exported[ARCHIVE_SPAN]
        if case == "archive-corrupt":
            archive_flip = volume_archive.stored_length(exported[ARCHIVE_SPAN][:64]) // 2
            fresh[ARCHIVE_SPAN.start + archive_flip] ^= 0x01
        (volume_dir / "volume.img").write_bytes(bytes(fresh))
        info = dict(info, variant=case, sha256=hashlib.sha256(fresh).hexdigest(), archive_flip=archive_flip)
        (volume_dir / "volume.json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
    image = volume_dir / "volume.img"
    archive_image = volume_dir / "archive.img" if disk1 is not None else None
    for path in (image, archive_image):
        if path is not None and path.stat().st_nlink != 1:
            raise ValueError("volume and archive images must have exactly one link")
    before = sha256(image)
    before_bytes = image.read_bytes()
    archive_before = sha256(archive_image) if archive_image else None
    rel = run_dir.relative_to(ROOT).as_posix()
    image_rel = image.relative_to(ROOT).as_posix()
    readonly = "on" if case in READ_ONLY else "off"
    args = [qemu, "-machine", qemu_machine(), "-accel", qemu_accel(), "-cpu", "qemu64",
            "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
            "-serial", "stdio", "-nic", "none", "-no-reboot",
            "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
            "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
            "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
            "-drive", f"if=none,id=volume,format=raw,cache=writeback,readonly={readonly},file={image_rel}",
            "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-M6E-TEST"]
    if archive_image is not None:
        # Explicit slot 6: QEMU refuses an explicit addr that is already taken,
        # so a boot also shows the auto-assigned ESP is not on 00:06.0.
        readonly1 = "on" if case in DISK1_READ_ONLY else "off"
        archive_rel = archive_image.relative_to(ROOT).as_posix()
        drive = f"if=none,id=archive,format=raw,cache=writeback,readonly={readonly1},file={archive_rel}"
        device = ("virtio-blk-pci,drive=archive,id=arcblk,addr=0x6,disable-legacy=on,"
                  + "num-queues=1,queue-size=8,serial=NOS-S09-ARCHIVE")
        args += ["-drive", drive, "-device", device]
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    rc, out, err, timed_out, killed, running = execute(
        args, env, cut_marker=CUT_MARKER.get(case), timeout=timeout)
    duration = time.monotonic() - start
    (run_dir / "serial.txt").write_text(out, encoding="utf-8")
    (run_dir / "stderr.txt").write_text(err, encoding="utf-8")
    after = sha256(image)
    shutil.copy2(image, run_dir / "volume.after.img")
    archive_after = sha256(archive_image) if archive_image else None
    if archive_image:
        shutil.copy2(archive_image, run_dir / "archive.after.img")
    report = dict(schema_version=1, case=case, kernel_feature=FEATURE[case], started_utc=started,
                  duration_s=duration, timeout_s=timeout, timed_out=timed_out, raw_qemu_exit=rc,
                  host_kill=killed, running_before_kill=running, command=args,
                  qemu_version=subprocess.check_output([qemu, "--version"], text=True).strip(),
                  source_fingerprint_version=3, source_sha256=frozen, volume_identity=info["identity"],
                  parent_case=parent, volume_directory=str(volume_dir), before_sha256=before, after_sha256=after,
                  archive_before_sha256=archive_before, archive_after_sha256=archive_after,
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
        if case in DISK_CASES:
            oracle = check_disk(case, before_bytes, image.read_bytes(),
                                disk1, archive_image.read_bytes() if archive_image else None,
                                evidence, archive_reference)
        elif case in ("corrupt", "foreign"):
            try:
                volume_disk.analyse(image.read_bytes())
                raise ValueError(f"oracle accepted the {case} fixture")
            except ValueError as error:
                if "accepted" in str(error):
                    raise
                oracle = {"refused": str(error)}
        elif case in INSTALL_CASES or (case, parent) == ("recover-cut", "install-cut"):
            oracle = analyse_v3(image.read_bytes())
            check_install(case, oracle, evidence)
        elif case.startswith("archive-"):
            oracle = check_archive(case, before_bytes, image.read_bytes(), evidence, archive_reference)
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
              run_directory=str(run_dir), after_sha256=after, archive_after_sha256=archive_after,
              source_sha256=frozen))
    print(f"M6e {case}: passed={report['passed']} raw={rc} duration={duration:.1f}s {run_dir}")
    if "error" in report:
        print(report["error"])
    return report, volume_dir


def outside_archive(image: bytes) -> bytes:
    """The image with the archive range back at its canaries, as the volume
    oracle requires every sector outside the volume scope to be."""
    data = bytearray(image)
    data[ARCHIVE_SPAN] = b"".join(volume_disk.canary(index)
                                  for index in range(ARCHIVE_FIRST, ARCHIVE_FIRST + ARCHIVE_SECTORS))
    return bytes(data)


def stored_archive(image: bytes) -> bytes:
    return stored_archive_in(image[ARCHIVE_SPAN])


def check_archive(case: str, before: bytes, after: bytes, evidence, reference: bytes | None = None):
    """Byte-level S09 acceptance with the independent format reader and the
    volume oracle; the kernel already counted archive I/O per capability."""
    counts = [evidence[f"VOLUME:ARCHIVE:{name}:"] for name in ("READS", "WRITES", "FLUSHES")]
    if case == "archive-full":
        # Spec: a full target leaves no finalized archive and verify refuses it.
        if reference is None or len(reference) <= FULL_SECTORS * SECTOR:
            raise ValueError("the full-target reference archive fits the cut range")
        if after[:ARCHIVE_SPAN.start] != before[:ARCHIVE_SPAN.start] or after[FULL_END:] != before[FULL_END:]:
            raise ValueError("full-target export changed sectors outside the cut range")
        data = after[ARCHIVE_SPAN.start:FULL_END]
        # The same source yields the same stream: the cut range holds exactly its prefix.
        if data != reference[:len(data)]:
            raise ValueError("cut range differs from the prefix of the export stage's archive")
        if counts[0] == 0 or counts[1:] != [FULL_SECTORS, 0]:
            raise ValueError(f"archive I/O counts {counts}")
        # Neither the cut range nor the full archive range (what a later reader
        # with the normal scope would see) may parse as an archive.
        for label, candidate in (("cut range", data), ("archive range", stored_archive(after))):
            try:
                volume_archive.parse(candidate)
                raise ValueError(f"format reader accepted the unfinished {label}")
            except ValueError as error:
                if "accepted" in str(error):
                    raise
        oracle = volume_disk.analyse(outside_archive(after))
        check_state("recover", oracle, evidence)
    elif case == "archive-ustar":
        # The lower half holds the export stage's archive again, the upper half
        # its ustar conversion. The guest first offers a refused name list: it
        # must write nothing, so the stream starts at USTAR_START and the write
        # count has no spare sector.
        length = evidence["VOLUME:ARCHIVE:USTAR_BYTES:"]
        data = stored_archive(after)
        end = ARCHIVE_SPAN.start + -(-len(data) // SECTOR) * SECTOR
        stop = USTAR_START + length
        if reference is None or data != reference or end > USTAR_START or stop > ARCHIVE_SPAN.stop:
            raise ValueError("lower half differs from the export stage's archive or the halves overlap")
        if after[ARCHIVE_SPAN.start + len(data):end] != bytes(end - ARCHIVE_SPAN.start - len(data)):
            raise ValueError("archive tail padding is not zero")
        if any(after[part] != before[part] for part in (slice(ARCHIVE_SPAN.start), slice(end, USTAR_START),
                                                        slice(stop, None))):
            raise ValueError("ustar export changed sectors outside the archive and its stream")
        if counts[0] == 0 or counts[1:] != [(end - ARCHIVE_SPAN.start + length) // SECTOR, 2]:
            raise ValueError(f"archive I/O counts {counts}")
        stream = after[USTAR_START:stop]
        members = check_ustar(stream, volume_archive.parse(data))
        oracle = volume_disk.analyse(outside_archive(after))
        check_state("recover", oracle, evidence)
        oracle["ustar"] = {"length": length, "sha256": hashlib.sha256(stream).hexdigest(), "members": members}
    elif case == "archive-export":
        if after[:ARCHIVE_SPAN.start] != before[:ARCHIVE_SPAN.start]:
            raise ValueError("export changed sectors outside the archive range")
        data = stored_archive(after)
        end = ARCHIVE_SPAN.start + -(-len(data) // SECTOR) * SECTOR
        if after[ARCHIVE_SPAN.start + len(data):end] != bytes(end - ARCHIVE_SPAN.start - len(data)):
            raise ValueError("archive tail padding is not zero")
        if after[end:] != before[end:]:
            raise ValueError("export wrote past the archive length")
        if counts[1] != (end - ARCHIVE_SPAN.start) // SECTOR or counts[0] == 0 or counts[2] == 0:
            raise ValueError(f"archive I/O counts {counts}")
        oracle = volume_disk.analyse(outside_archive(after))
        check_state("recover", oracle, evidence)
        matches_volume(volume_archive.parse(data), oracle)
    else:
        if after[ARCHIVE_SPAN] != before[ARCHIVE_SPAN] or counts[0] == 0 or counts[1:] != [0, 0]:
            raise ValueError(f"archive range changed or I/O counts {counts}")
        data = stored_archive(before)
        oracle = check_restored(case, data, after, evidence)
    oracle["archive"] = {"length": len(data), "sha256": hashlib.sha256(data).hexdigest(), "io": counts}
    if reference is not None:
        oracle["archive"]["reference_length"] = len(reference)
    return oracle


def domain_generation(oracle) -> int:
    """The domain-0 generation an archive header carries, as LIST reports it:
    the sum of the domain's object and collection revisions (not the global one)."""
    return (sum(item["revision"] for item in oracle["objects"][:volume_disk.PER_DOMAIN])
            + sum(c["revision"] for c in oracle["collections"] if c["domain"] == 0))


def matches_volume(archive: dict, oracle) -> None:
    """The parsed archive equals domain 0 of the volume oracle: objects with
    their content and metadata, collections and the header generation."""
    if archive["generation"] != domain_generation(oracle):
        raise ValueError("archive generation differs from the domain generation")
    present = [item["object"] for item in oracle["objects"][:volume_disk.PER_DOMAIN] if item["present"]]
    if [item["slot"] for item in archive["objects"]] != present:
        raise ValueError("archive objects differ from the domain-0 objects")
    for source in archive["objects"]:
        item = oracle["objects"][source["slot"]]
        version = current(item)
        found = (source["name"], source["length"], source["collection"], source["revision"],
                 source["tags"], source["kind"], source["created"], source["modified"], source["content"])
        want = (version["name"], version["length"], item["collection"], item["revision"], item["tags"],
                item["kind"], item["created"], item["modified"],
                volume_disk.fixture(source["slot"], version["content_revision"], version["length"]))
        if found != want:
            raise ValueError(f"archive object {source['slot']} differs from the volume")
    names = {c["index"]: c["name"] for c in oracle["collections"] if c["domain"] == 0 and c["present"]}
    if archive["collections"] != names:
        raise ValueError("archive collections differ from the volume")


def refused(data: bytes, label: str) -> str:
    """The format reader must refuse `data`; returns its reason."""
    try:
        volume_archive.parse(data)
    except ValueError as error:
        return str(error)
    raise ValueError(f"format reader accepted the {label}")


def check_restored(case: str, data: bytes, after: bytes, evidence):
    """A restore (or a refused one) from `data` into the fresh disk 0 `after`."""
    if case in ("archive-corrupt", "archive-disk-wrong", "archive-disk-verify"):
        refused(data, "damaged archive")
        oracle = volume_disk.analyse(outside_archive(after))
        pre, expected = {}, volume_archive.expected_restore({"objects": []}, {}, {})
    else:
        archive = volume_archive.parse(data)
        pre, held = (CONFLICT_OBJECTS, CONFLICT_COLLECTIONS) if case == "archive-conflict" else ({}, {})
        # The end state follows from the archive and the pre-state by the spec rule alone.
        expected = volume_archive.expected_restore(archive, {o: name for o, (name, _) in pre.items()}, held)
        # The native fixture exercises these four rules. Two more are host tests in
        # services/volume/tests/archive.rs: "no collection for a conflicting object" cannot hold
        # together with the four here (the archive has only two collected objects), and
        # "uncollected without metadata stays at revision 1" is not exercised by this fixture.
        if case == "archive-conflict" and not all(
                expected[key] for key in ("slot_conflicts", "name_conflicts", "uncollected", "reused")):
            raise ValueError(f"conflict fixture misses a restore rule: {expected}")
        # Only restored objects carry archive bytes; the pre-state must stay a written fixture.
        oracle = volume_disk.analyse(outside_archive(after), opaque=frozenset(expected["objects"]))
    for item in oracle["objects"]:
        obj, source = item["object"], expected["objects"].get(item["object"])
        if obj in pre:
            version = current(item)
            found = (item["revision"], len(item["versions"]), version["name"], version["length"],
                     version["content_revision"], item["collection"], item["tags"], item["kind"])
            if found != (1, 1, *pre[obj], 1, 0, 0, 0):
                raise ValueError(f"restore changed pre-existing object {obj}: {found}")
        elif source is None:
            if item["present"] or item["revision"]:
                raise ValueError(f"unexpected object {obj} after {case}")
        else:
            version = current(item)
            found = (item["revision"], len(item["versions"]), version["name"], version["length"],
                     item["collection"], item["tags"], item["kind"], bytes.fromhex(version["content_hex"]))
            want = (source["revision"], 1, source["name"], source["length"], source["collection"],
                    source["tags"], source["kind"], source["content"])
            if found != want:
                raise ValueError(f"restored object {obj} differs from the archive: {found[:7]}")
    check_collections(oracle, {(0, index): name for index, name in expected["collections"].items()})
    if [evidence["VOLUME:A:DIGEST:"], evidence["VOLUME:B:DIGEST:"]] != oracle["digests"]:
        raise ValueError("guest-reported digests differ from the disk oracle")
    return oracle


def disk1_image(sectors: int) -> bytes:
    """A fresh archive disk: every sector holds its canary."""
    return b"".join(volume_disk.canary(index) for index in range(sectors))


def disk_archive(disk: bytes) -> bytes:
    """The archive stored from sector 0 of the archive disk's scope."""
    return stored_archive_in(disk[DISK1_SPAN])


def stored_archive_in(region: bytes) -> bytes:
    length = volume_archive.stored_length(region[:volume_archive.HEADER])
    if length > len(region):
        raise ValueError("archive header claims more than the archive range")
    return region[:length]


def refused_disk(disk: bytes, label: str) -> str:
    """The archive disk must not hold an archive: either its header claims
    more than the scope, or the reader refuses the stored length it claims."""
    try:
        data = disk_archive(disk)
    except ValueError as error:
        return str(error)
    return refused(data, label)


def torn_archive(before: bytes, after: bytes, limit: int, generation: int, added: tuple[int, str]) -> dict:
    """Power cut over an older archive: classify the archive disk. The older
    archive must parse. The expected new archive differs from it only by the
    portal's added collection `added` = (index, name): a new header (one more
    collection, domain generation `generation` from the disk-0 oracle, same
    objects and content bytes), the collection records with `added` inserted,
    and then the older object stream unchanged. The changed sectors must start
    at sector 0, end before `limit` (the kernel stops the guest after `limit`
    completed lane-1 writes) and, up to the last one, equal that expected
    prefix. Then the region is neither the older archive nor a complete new
    one, and only Corrupt is a correct verdict."""
    older = disk_archive(before)
    old = volume_archive.parse(older)
    if len(older) <= limit * SECTOR:
        raise ValueError("the older archive ends before the cut: no torn mix is possible")
    changed = [index for index in range(len(after) // SECTOR)
               if after[index * SECTOR:(index + 1) * SECTOR] != before[index * SECTOR:(index + 1) * SECTOR]]
    if not changed or changed[0] != 0 or changed[-1] >= limit:
        raise ValueError(f"cut did not leave a torn new header within {limit} sectors: {changed[:4]}...")
    header = after[:volume_archive.HEADER]
    count, collections = older[12], older[13]
    if (header[:8] != volume_archive.MAGIC or header[8:12] != older[8:12] or not volume_archive._sealed(header)
            or (header[12], header[13]) != (count, collections + 1)
            # Reserved [14:16] and [32:56] must stay zero as in the parsed older header.
            or header[14:16] != older[14:16] or header[24:56] != older[24:56]
            or struct.unpack_from("<Q", header, 16)[0] != generation
            or volume_archive.stored_length(header) != len(older) + volume_archive.COLLECTION_RECORD):
        raise ValueError("sector 0 does not hold the expected new archive header")
    names = {**old["collections"], added[0]: added[1]}
    records = b""
    for index in sorted(names):
        start = volume_archive.HEADER + len(records)
        record = after[start:start + volume_archive.COLLECTION_RECORD]
        name = names[index].encode()
        if (not volume_archive._sealed(record) or record[0] != index or record[1] != len(name)
                or record[8:8 + len(name)] != name or any(record[2:8]) or any(record[8 + len(name):40])):
            raise ValueError(f"new archive collection record {index} differs from the volume")
        records += record
    expected = header + records + older[volume_archive.HEADER + collections * volume_archive.COLLECTION_RECORD:]
    landed = (changed[-1] + 1) * SECTOR
    if after[:landed] != expected[:landed]:
        raise ValueError("the changed sectors are not the expected new archive prefix")
    reason = refused_disk(after, "torn archive")
    return {"classification": "corrupt", "changed_sectors": len(changed), "last_changed": changed[-1],
            "older_length": len(older), "older_sha256": hashlib.sha256(older).hexdigest(),
            "new_length": len(older) + volume_archive.COLLECTION_RECORD,
            "reason": f"new header and records over the older body; reader: {reason}"}


def cut_volume(before, after) -> None:
    """The power-cut stage's disk 0 differs from its input only by the
    portal's one new domain-0 collection: same objects, content, revisions
    and other collections, domain-1 state unchanged."""
    if (after["objects"] != before["objects"] or after["chunks_used"] != before["chunks_used"]
            or after["generation"] != before["generation"] + 1
            or after["clocks"] != [before["clocks"][0] + 1, before["clocks"][1]]
            or after["digests"][1] != before["digests"][1]):
        raise ValueError("power-cut stage changed disk 0 beyond one new collection")
    old = {(c["domain"], c["index"]): c for c in before["collections"]}
    for item in after["collections"]:
        key = (item["domain"], item["index"])
        if key == CUT_COLLECTION_KEY:
            if old[key]["present"] or (item["present"], item["revision"], item["name"]) != (True, 1, CUT_COLLECTION[1]):
                raise ValueError(f"the new collection is not {CUT_COLLECTION}: {item}")
        elif item != old[key]:
            raise ValueError(f"power-cut stage changed collection {key}")


def check_disk(case: str, before: bytes, after: bytes, disk1_before: bytes | None, disk1_after: bytes | None,
               evidence, reference: bytes | None):
    """Host acceptance of the second-disk stages: both disks byte for byte,
    the independent format reader and the volume oracle."""
    if case == "archive-disk-cut":
        # The guest stops before its summary: no evidence lines, host bytes only.
        oracle = volume_disk.analyse(after)
        cut_volume(volume_disk.analyse(before), oracle)
        oracle["archive_disk"] = torn_archive(disk1_before, disk1_after, ARCHIVE_CUT_WRITES,
                                              domain_generation(oracle), CUT_COLLECTION)
        return oracle
    counts = [evidence[f"VOLUME:ARCHIVE:{name}:"] for name in ("READS", "WRITES", "FLUSHES")]
    device = [evidence[f"VOLUME:BROKER:ARCHIVE_DISK_{name}:"] for name in ("READS", "WRITES", "FLUSHES")]
    granted = 0 if case == "archive-disk-wrong" else 1
    offline = 14 if case == "archive-disk-absent" else 0
    if (evidence["VOLUME:ARCHIVE:DEVICE:"], evidence["VOLUME:ARCHIVE:DISK_OFFLINE:"]) != (granted, offline):
        raise ValueError("archive grant on the wrong device or device-1 state")
    if evidence["VOLUME:ARCHIVE:DISK_FOUND:"] != (case != "archive-disk-absent"):
        raise ValueError("archive disk presence differs from the attached devices")
    # The portal's past-the-end probe (and, too small, its write past the scope).
    if evidence["VOLUME:ARCHIVE:SCOPE_REFUSALS:"] < (2 if case == "archive-disk-small" else 1):
        raise ValueError("scope refusals on the archive capability not observed")
    if granted and (device != counts or evidence["VOLUME:ARCHIVE:ESCAPE_DENIALS:"] != 2):
        raise ValueError(f"device-1 I/O {device} is not exactly the portal's {counts}, or escape probes missing")
    if case in ("archive-disk-export", "archive-disk-absent", "archive-disk-small"):
        # The source volume is mounted read-only: disk 0 is byte-identical.
        if after != before or (evidence["VOLUME:BROKER:VOLUME_DISK_WRITES:"], evidence["VOLUME:BROKER:VOLUME_DISK_FLUSHES:"]) != (0, 0):
            raise ValueError("export changed disk 0")
        oracle = volume_disk.analyse(after)
        check_state("recover", oracle, evidence)
    if case == "archive-disk-absent":
        if counts != [0, 0, 0] or disk1_after is not None:
            raise ValueError(f"absent archive disk saw I/O {counts}")
        return oracle
    if case == "archive-disk-export":
        data = disk_archive(disk1_after)
        end = -(-len(data) // SECTOR) * SECTOR
        if reference is None or data != reference:
            raise ValueError("archive disk differs from the archive exported to disk 0")
        if disk1_after[len(data):end] != bytes(end - len(data)) or disk1_after[end:] != disk1_before[end:]:
            raise ValueError("archive disk padding is not zero or the export wrote past the archive")
        if counts[0] == 0 or counts[1] != end // SECTOR or counts[2] == 0:
            raise ValueError(f"archive I/O counts {counts}")
        matches_volume(volume_archive.parse(data), oracle)
    elif case == "archive-disk-small":
        cut = FULL_SECTORS * SECTOR
        if reference is None or len(reference) <= cut or disk1_after[:cut] != reference[:cut]:
            raise ValueError("small disk does not hold the prefix of the export stage's archive")
        if disk1_after[cut:] != disk1_before[cut:]:
            raise ValueError("export wrote past the scope of the small disk")
        if counts[0] == 0 or counts[1:] != [FULL_SECTORS, 0]:
            raise ValueError(f"archive I/O counts {counts}")
        refused(disk1_after[:cut], "unfinished cut range")
        refused(disk1_after, "whole small disk")
        data = disk1_after[:cut]
    else:
        if disk1_after != disk1_before or counts[0] == 0 or counts[1:] != [0, 0]:
            raise ValueError(f"archive disk changed or I/O counts {counts}")
        if after[ARCHIVE_SPAN] != before[ARCHIVE_SPAN]:
            raise ValueError("the device-0 archive range changed")
        if case == "archive-disk-wrong":
            # Granted the device-0 range (canaries): Corrupt, though disk 1 holds a valid archive.
            if device != [0, 0, 0]:
                raise ValueError(f"wrong-device case touched disk 1: {device}")
            volume_archive.parse(disk_archive(disk1_after))
            data = after[ARCHIVE_SPAN]
        elif case == "archive-disk-verify":
            refused_disk(disk1_after, "torn archive")
            data = disk1_after[DISK1_SPAN]
        else:
            data = disk_archive(disk1_after)
        oracle = check_restored(case, data, after, evidence)
        if case == "archive-disk-verify":
            oracle["archive_disk"] = {"classification": "corrupt", "reason": refused_disk(disk1_after, "torn archive")}
    oracle["archive"] = {"length": len(data), "sha256": hashlib.sha256(data).hexdigest(), "io": counts,
                         "device1": device}
    return oracle


def check_ustar(stream: bytes, archive: dict) -> list[str]:
    """Read the guest's ustar stream with Python tarfile, independent of
    services/volume/src/ustar.rs, against the parsed archive it came from.
    tarfile accepts a missing end and stops quietly at damage, so the bytes are
    accounted for too: members tile the stream, then exactly two zero blocks."""
    want = [((archive["collections"][item["collection"]] + "/" if item["collection"] else "") + item["name"],
             item["content"]) for item in archive["objects"]]
    if not stream or len(stream) % SECTOR or stream[-2 * SECTOR:] != bytes(2 * SECTOR):
        raise ValueError("ustar stream does not end in two zero blocks")
    with tarfile.open(fileobj=io.BytesIO(stream), mode="r:", encoding="utf-8", errors="strict") as tar:
        members = tar.getmembers()
        if [member.name for member in members] != [path for path, _ in want]:
            raise ValueError(f"ustar members {[member.name for member in members]}")
        offset = 0
        for member, (path, content) in zip(members, want):
            header = stream[member.offset_data - SECTOR:member.offset_data]
            # A pax 'x' header with only a path record precedes exactly the non-ASCII paths.
            pax = member.offset != member.offset_data - SECTOR
            if (member.offset != offset or header[156:157] != b"0" or header[257:265] != b"ustar\x0000"
                    or pax == path.isascii() or member.pax_headers != ({"path": path} if pax else {})
                    or (member.mode, member.uid, member.gid, member.mtime) != (0o644, 0, 0, 0)):
                raise ValueError(f"ustar header of {path!r}")
            offset = member.offset_data + -(-member.size // SECTOR) * SECTOR
            if (tar.extractfile(member).read() != content
                    or stream[member.offset_data + member.size:offset] != bytes(offset - member.offset_data - member.size)):
                raise ValueError(f"ustar content of {path!r}")
        if tar.offset != offset or offset != len(stream) - 2 * SECTOR:
            raise ValueError("ustar stream has bytes between its last member and its end")
    return [member.name for member in members]


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
        slot_markers = oracle.get("slot_markers") or [False, False]
        other_marker = bool(slot_markers[(g + 1) % 2]) if len(slot_markers) > (g + 1) % 2 else False
        marker_only = g == 1 and other_marker and other[0] == -1 and all(sector == 0 for sector in other[1:])
        if count or (not marker_only and any(sector != g - 1 for sector in other)):
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


def analyse_v3(image: bytes):
    """volume_disk.analyse (frozen v2 reader) with volume_disk.choose(allow_v3=True),
    plus the table format and the v3 provenance of every object and collection
    (zero in a v2 table). Same image-size and outside-scope canary checks."""
    if len(image) != volume_disk.IMAGE_SECTORS * SECTOR:
        raise ValueError("image size changed")
    first, count = volume_disk.FIRST, volume_disk.VOLUME_SECTORS
    outside = [i for i in range(volume_disk.IMAGE_SECTORS)
               if not first <= i < first + count and image[i * SECTOR:(i + 1) * SECTOR] != volume_disk.canary(i)]
    if outside:
        raise ValueError(f"sectors outside the granted scope changed: {outside[:8]}")
    volume = image[first * SECTOR:(first + count) * SECTOR]
    table = volume_disk.choose(volume, allow_v3=True)
    objects = []
    for obj, entry in enumerate(table["entries"]):
        versions = []
        for version in volume_disk.retained(entry):
            data = volume_disk.content(volume, version)
            written = next((r for r in range(version["revision"], 0, -1)
                            if data == volume_disk.fixture(obj, r, version["length"])), None)
            if written is None:
                raise ValueError(f"object {obj} version {version['revision']} is not a written fixture")
            versions.append({"revision": version["revision"], "length": version["length"],
                             "name": version["name"].decode("utf-8"), "content_revision": written,
                             "chunks": version["chunks"]})
        objects.append({"object": obj, "revision": entry["revision"], "present": entry["present"],
                        "collection": entry["collection"], "tags": entry["tags"], "kind": entry["kind"],
                        "created": entry["created"], "modified": entry["modified"],
                        "request_id": entry["request_id"], "versions": versions,
                        "installed": entry.get("installed", 0), "origin": list(entry.get("origin", (0, 0, 0))),
                        "origins": list(entry.get("origins", [0] * (volume_disk.HISTORY + 1)))})
    collections = [{"index": i % volume_disk.COLLECTIONS + 1, "domain": i // volume_disk.COLLECTIONS,
                    "present": c["present"], "revision": c["revision"], "name": c["name"].decode("utf-8"),
                    "origin": c.get("origin", 0)}
                   for i, c in enumerate(table["collections"])]
    scans = [volume_disk.slot_states(volume, slot) for slot in range(2)]
    return {"format": table["format"], "generation": table["generation"], "clocks": table["clocks"],
            "objects": objects, "collections": collections,
            "chunks_used": sum(len(volume_disk.unique_chunks(e)) for e in table["entries"]),
            "slots": [[g if kind == "valid" else (0 if kind == "zero" else -1) for kind, g, _ in states]
                      for _, states in scans],
            "slot_markers": [volume_disk.is_format_marker(sectors[0]) for sectors, _ in scans],
            "digests": [volume_disk.domain_digest(table, d) for d in range(volume_disk.DOMAINS)]}


def install_state(oracle, installed: bool):
    """Exactly the state after the fixture INSTALL_COLLECTION, then (when
    `installed`) the fixture INSTALL; every value derives from INSTALL_*."""
    n = len(INSTALL_VERSIONS)
    commits = INSTALL_COMMITS if installed else INSTALL_COMMITS - 1
    if (oracle["format"], oracle["generation"], oracle["clocks"]) != (3, commits, [commits, 0]):
        raise ValueError(f"install table {oracle['format']}/{oracle['generation']}/{oracle['clocks']}")
    index, name, origin = INSTALL_COLLECTION
    check_collections(oracle, {(0, index): name})
    if [c["origin"] for c in oracle["collections"]] != [
            origin if (c["domain"], c["index"]) == (0, index) else 0 for c in oracle["collections"]]:
        raise ValueError("collection origins differ from the install fixture")
    clock = oracle["clocks"][0]
    tags, kind, collection = INSTALL_META
    for item in oracle["objects"]:
        if item["object"] != INSTALL_OBJECT or not installed:
            if item["present"] or item["revision"] or item["installed"] or any(item["origin"]) \
                    or any(item["origins"]):
                raise ValueError(f"object {item['object']} is not never-used")
            continue
        # Rule R: revision n, created == modified == the commit's own clock.
        found = (item["present"], item["revision"], item["installed"], item["created"], item["modified"],
                 item["collection"], item["tags"], item["kind"], tuple(item["origin"]), item["origins"])
        if found != (True, n, n, clock, clock, collection, tags, kind, INSTALL_ORIGIN,
                     [INSTALL_ORIGIN_0 - i for i in range(n)]):
            raise ValueError(f"installed object mismatch: {found}")
        versions = [(v["revision"], v["name"], v["length"], v["content_revision"]) for v in item["versions"]]
        if versions != [(n - i, vname, length, n - i) for i, (vname, length) in enumerate(INSTALL_VERSIONS)]:
            raise ValueError(f"installed versions mismatch: {versions}")
    chunks = sum(-(-length // volume_disk.CHUNK) for _, length in INSTALL_VERSIONS) if installed else 0
    if oracle["chunks_used"] != chunks:
        raise ValueError(f"install chunk usage {oracle['chunks_used']}")


def check_install(case: str, oracle, evidence):
    """install and install-recover: the installed state, its previous table
    copy complete. install-cut: never-used with a torn INSTALL_COMMIT copy.
    recover-cut after install-cut: exactly one of the two."""
    if case == "recover-cut":
        installed = oracle["generation"] == INSTALL_COMMITS
    else:
        installed = case != "install-cut"
    install_state(oracle, installed)
    g = oracle["generation"]
    if installed:
        if oracle["slots"][(g + 1) % 2] != [g - 1] * volume_disk.TABLE_SECTORS:
            raise ValueError("the previous table copy of the install is not complete")
    else:
        oracle["torn_sectors"] = torn_sectors(oracle, True)
    oracle["install_outcome"] = "installed" if installed else "never-used"
    if case == "install-cut":
        return
    item = oracle["objects"][INSTALL_OBJECT]
    commits = INSTALL_COMMITS if case == "install" else 0
    want = {"VOLUME:A:RECOVERED_REVISION:": item["revision"],
            "VOLUME:A:RECOVERED_COLLECTION:": item["collection"],
            "VOLUME:A:RECOVERED_SECOND:": item["installed"],
            "VOLUME:SERVICE:GENERATION:": g,
            # install: exactly the two publishes; the Stale and Range refusals wrote none.
            "VOLUME:SERVICE:COMMITS:": commits,
            "VOLUME:SERVICE:TABLE_WRITES:": commits,
            # Broker view of the table region: the one-sector format marker of the
            # fresh image plus one full table copy per publish, nothing else.
            "VOLUME:BROKER:TABLE_SECTOR_WRITES:": 1 + commits * volume_disk.TABLE_SECTORS if commits else 0}
    if {key: evidence[key] for key in want} != want:
        raise ValueError(f"guest install evidence differs from the oracle: {want}")
    if [evidence["VOLUME:A:DIGEST:"], evidence["VOLUME:B:DIGEST:"]] != oracle["digests"]:
        raise ValueError("guest-reported digests differ from the disk oracle")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=sorted(FEATURE), required=True)
    parser.add_argument("--volume", type=Path)
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    args = parser.parse_args()
    raise SystemExit(0 if run_volume(args.case, args.volume, timeout=args.timeout)[0]["passed"] else 1)
