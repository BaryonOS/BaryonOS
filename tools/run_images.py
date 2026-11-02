#!/usr/bin/env python3
"""Bind actual native PNG/JPEG probe results to the complete system/disk run.

No image decoding runs on the host. ELF FNV is parity only; SHA256 binds bytes.
This gate is not viewer, full-format, cancellation-latency or release acceptance.
"""
from __future__ import annotations

import argparse
import hashlib
import math
from pathlib import Path
import re
import shutil

from common import ROOT, BUILD, TARGET, environment, firmware, qemu_machine, sha256, source_hash
from build_users import inspect_user
from run_fpu import private_path, publish_json, load_json
from run_system import MAX_TIMEOUT, OPAQUE_OBJECTS, check_disk, check_image, evaluate, run_system
import volume_disk

FEATURE = "native-system-images"
CASE = "normal"
LIMIT = 64 * 1024 * 1024
FIELDS = ("PID", "ELF_BYTES", "ELF_FNV64", "PNG", "JPEG", "PIXELS", "NEGATIVES", "PIXEL_HASH",
          "DECODED_BYTES", "HEAP_GROW_PAGES", "ARENA_PEAK", "LIVE_ALLOCATIONS", "CANCELS",
          "START_TICKS", "END_TICKS", "EXIT_TICKS", "QUALITY_OPEN", "STACK_PAGES", "STACK_GUARD",
          "HEAP_KERNEL_PAGES")
EXACT = {"PNG": 8, "JPEG": 3, "PIXELS": 149647, "NEGATIVES": 10,
         "PIXEL_HASH": 548683896226622996, "DECODED_BYTES": 623808, "HEAP_GROW_PAGES": 1024,
         "LIVE_ALLOCATIONS": 0, "CANCELS": 1, "QUALITY_OPEN": 0, "STACK_PAGES": 16,
         "STACK_GUARD": 1, "HEAP_KERNEL_PAGES": 1040}
PROGRAMS = {"system_init": ("system-system", "system-init"),
            "system_app": ("system-system", "system-app"),
            "system_worker": ("system-system", "system-worker"),
            "keyring_service": ("system-keyring", "keyring-service"),
            "volume_service": ("system-volume", "volume-service"),
            "image_probe": ("system-image-probe", "image-probe")}


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def fnv64(data: bytes) -> int:
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return value


def hash_value(value) -> str:
    if not isinstance(value, str) or re.fullmatch(r"[0-9a-f]{64}", value) is None:
        raise ValueError("missing or malformed SHA256")
    return value


def read(path: Path, *, cargo=False) -> bytes:
    path = private_path(path, regular=True, cargo_output=cargo)
    if path.stat().st_size > LIMIT:
        raise ValueError("unbounded image evidence")
    data = path.read_bytes()
    if len(data) > LIMIT:
        raise ValueError("image evidence grew while reading")
    return data


def validate_sequence(serial: str, raw_exit: int | None, timed_out: bool, *, linked_elf: bytes) -> dict:
    if not isinstance(serial, str) or len(serial) > 128 * 1024 or not linked_elf:
        raise ValueError("missing or unbounded image evidence")
    if type(raw_exit) is not int or timed_out is not False:
        raise ValueError("timeout or missing native exit is not image completion")
    base = evaluate(CASE, serial, raw_exit, timed_out)
    lines = serial.splitlines()
    anchor_prefix = "SYSTEM:IMAGE_MANIFEST_PID:"
    anchors = [line for line in lines if line.startswith(anchor_prefix)]
    if len(anchors) != 1:
        raise ValueError("missing or duplicate actual bootstrap PID")
    match = re.fullmatch(re.escape(anchor_prefix) + r"([0-9]{20})", anchors[0])
    if match is None or not 0 < int(match[1]) < 1 << 64:
        raise ValueError("invalid actual bootstrap PID")
    manifest_pid = int(match[1])
    observed = [line for line in lines if line.startswith("SYSTEM:IMAGE") and line != anchors[0]]
    if len(observed) != len(FIELDS) + 1 or observed[-1] != "SYSTEM:IMAGES:PASS":
        raise ValueError("missing, duplicated, failed or unknown image evidence")
    values = {}
    for name, line in zip(FIELDS, observed):
        match = re.fullmatch(re.escape("SYSTEM:IMAGE_" + name + ":") + r"([0-9]{20})", line)
        if match is None or int(match[1]) > (1 << 64) - 1:
            raise ValueError("malformed, reordered or overflowing image counter")
        values[name] = int(match[1])
    if not (lines.index("BOOT:EBS_OK") < lines.index(anchors[0]) < lines.index("SYSTEM:MANIFEST:PASS")
            < lines.index(observed[0])
            < lines.index(observed[-1]) < lines.index("SYSTEM:BROKERS:PASS")):
        raise ValueError("image evidence is outside its boot/live-system boundary")
    if values["PID"] != manifest_pid:
        raise ValueError("final image observation belongs to another process")
    if any(values[name] != expected for name, expected in EXACT.items()):
        raise ValueError("wrong native image pixel, denial, heap, stack or cancellation facts")
    if not 0 < values["ARENA_PEAK"] <= 4 * 1024 * 1024:
        raise ValueError("image arena highwater is outside its actual 4MiB budget")
    if (not values["START_TICKS"] <= values["END_TICKS"] <= values["EXIT_TICKS"]
            or values["EXIT_TICKS"] - values["START_TICKS"] >= 20000):
        raise ValueError("image ticks are outside bounded native execution")
    if values["ELF_BYTES"] != len(linked_elf) or values["ELF_FNV64"] != fnv64(linked_elf):
        raise ValueError("kernel's linked image ELF differs from the actual inspected binary")
    if base["ENTROPY_BYTES"] != 1024:
        raise ValueError("image-only probe must not add key/RNG operations")
    return {"MANIFEST_PID": manifest_pid, **values}


def validate_manifest(manifest: dict, frozen: str) -> None:
    hash_value(frozen)
    if (type(manifest.get("schema_version")) is not int or manifest["schema_version"] != 1
            or manifest.get("source_fingerprint_version") != 3 or manifest.get("source_sha256") != frozen
            or manifest.get("kernel_feature") != FEATURE or manifest.get("loader_feature") is not None):
        raise ValueError("wrong or stale native image build profile")
    for name in ("kernel", "loader"):
        hash_value(manifest[name]["sha256"])
    if set(manifest.get("system", {})) != set(PROGRAMS):
        raise ValueError("exactly six inspected native system programs are required")
    for details in manifest["system"].values():
        hash_value(details["sha256"])
        if not isinstance(details.get("elf"), dict):
            raise ValueError("missing native image ELF inspection")
    if manifest.get("image_probe") != manifest["system"]["image_probe"]:
        raise ValueError("conflicting standalone/system image entries")



def preflight_esp(manifest: dict) -> None:
    """Refuse any extra/link/special source before the shared FAT copy or VM."""
    esp = private_path(BUILD / "esp")
    if not esp.is_dir():
        raise ValueError("missing real source ESP directory")
    expected = {"kernel.elf": manifest["kernel"]["sha256"],
                "EFI/BOOT/BOOTX64.EFI": manifest["loader"]["sha256"]}
    actual = set()
    for path in esp.rglob("*"):
        private_path(path)  # Includes every ancestor; never follow a source link.
        if path.is_file():
            actual.add(path.relative_to(esp).as_posix())
    if actual != set(expected):
        raise ValueError("source ESP must contain exactly kernel and loader")
    for name, expected_hash in expected.items():
        if digest(read(esp / name)) != hash_value(expected_hash):
            raise ValueError("source ESP differs from the inspected build")


def validate_artifacts(report: dict, frozen: str, *, qemu: str, qemu_sha256: str,
                       firmware_hashes: dict) -> dict:
    if (type(report.get("schema_version")) is not int or report["schema_version"] != 1
            or report.get("passed") is not True or report.get("case") != CASE
            or report.get("image_case") != CASE or type(report.get("image_gate")) is not bool
            or report.get("kernel_feature") != FEATURE or report.get("source_fingerprint_version") != 3
            or report.get("source_sha256") != frozen or type(report.get("timeout_s")) is not int
            or not 0 < report["timeout_s"] <= MAX_TIMEOUT or type(report.get("duration_s")) not in (int, float)
            or not math.isfinite(report["duration_s"]) or not 0 <= report["duration_s"] <= report["timeout_s"] + 5
            or not isinstance(report.get("source_commit"), str)
            or re.fullmatch(r"[0-9a-f]{40}", report["source_commit"]) is None):
        raise ValueError("native image guest source/profile/runtime is not bound")
    if hash_value(report.get("image_qemu_sha256")) != hash_value(qemu_sha256):
        raise ValueError("image emulator differs from its frozen executable")
    manifest = report["manifest"]
    validate_manifest(manifest, frozen)
    if load_json(private_path(BUILD / "manifest.json", regular=True)) != manifest:
        raise ValueError("image result is not the current inspected build")
    run = private_path(Path(report["run_directory"])).resolve(strict=True)
    volume = private_path(Path(report["volume_directory"])).resolve(strict=True)
    parent = (BUILD / "run").resolve(strict=True)
    if (run.parent != parent or volume.parent != parent or run == volume
            or re.fullmatch(r"system-normal-[0-9a-f]{32}", run.name) is None
            or re.fullmatch(r"system-image-[0-9a-f]{32}", volume.name) is None):
        raise ValueError("image evidence escaped separate new generated normal-run directories")
    relative, disk = run.relative_to(ROOT).as_posix(), (volume / "volume.img").relative_to(ROOT).as_posix()
    command = [qemu, "-machine", qemu_machine(), "-accel", "tcg,thread=single", "-cpu", "qemu64",
               "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
               "-serial", "stdio", "-nic", "none", "-no-reboot",
               "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={relative}/OVMF_CODE.fd",
               "-drive", f"if=pflash,unit=1,format=raw,file={relative}/OVMF_VARS.fd",
               "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{relative}/esp",
               "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
               "-drive", f"if=none,id=volume,format=raw,cache=writeback,readonly=off,file={disk}",
               "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST",
               "-object", "rng-random,id=rng0,filename=/dev/urandom",
               "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000"]
    if report.get("command") != command or not report.get("qemu_version", "").startswith("QEMU emulator version "):
        raise ValueError("changed image VM command, firmware, network or snapshot profile")
    if report.get("firmware_sha256") != firmware_hashes or digest(read(run / "OVMF_CODE.fd")) != firmware_hashes["code"]:
        raise ValueError("image firmware differs from selected preboot inputs")
    read(run / "OVMF_VARS.fd")
    actual_esp = set()
    for path in (run / "esp").rglob("*"):
        private_path(path)
        if path.is_file():
            actual_esp.add(path.relative_to(run).as_posix())
    expected = {"esp/kernel.elf": manifest["kernel"]["sha256"],
                "esp/EFI/BOOT/BOOTX64.EFI": manifest["loader"]["sha256"]}
    if actual_esp != set(expected) or report.get("esp_sha256") != expected:
        raise ValueError("unbound or extra files in actual image guest ESP")
    for name, expected_hash in expected.items():
        if digest(read(run / name)) != expected_hash or digest(read(BUILD / name)) != expected_hash:
            raise ValueError("actual image guest kernel/loader differs from build")
    kernel = read(run / "esp/kernel.elf")
    probe = None
    for name, (directory, filename) in PROGRAMS.items():
        path = TARGET / directory / "x86_64-unknown-none/release" / filename
        binary = read(path, cargo=True)
        if (digest(binary) != manifest["system"][name]["sha256"]
                or inspect_user(path) != manifest["system"][name]["elf"] or binary not in kernel):
            raise ValueError("image program hash, ELF inspection or kernel embedding mismatch")
        if name == "image_probe":
            probe = binary
    initial = load_json(private_path(volume / "volume.json", regular=True))
    empty = volume_disk.empty_image_bytes()
    if (initial.get("variant") != "empty" or initial.get("sha256") != digest(empty)
            or report.get("before_sha256") != digest(empty)):
        raise ValueError("image probe must use a fresh empty disposable disk")
    image, snapshot = volume / "volume.img", run / "volume.after.img"
    check_image(image)
    check_image(snapshot)
    data = read(image)
    if digest(data) != hash_value(report.get("after_sha256")) or read(snapshot) != data:
        raise ValueError("image probe post-run disk bytes changed")
    oracle = volume_disk.analyse(data, OPAQUE_OBJECTS)
    check_disk(oracle)
    if oracle["objects"][0]["request_id"] != 3 or report.get("oracle") != oracle:
        raise ValueError("image probe oracle/SAVE identity differs from actual original document")
    serial = read(run / "serial.txt").decode("utf-8", "replace")
    observed = validate_sequence(serial, report.get("raw_qemu_exit"), report.get("timed_out"), linked_elf=probe)
    # The generic parser currently accepts A-Z/_ names only, so ELF_FNV64 is
    # independently parsed above and stored in image_evidence, never invented.
    if report.get("evidence") != evaluate(CASE, serial, report.get("raw_qemu_exit"), report.get("timed_out")):
        raise ValueError("cached generic counters differ from actual serial")
    if "image_evidence" in report and report["image_evidence"] != observed:
        raise ValueError("cached image counters differ from actual serial")
    return observed


def _run_images(case: str = CASE, *, timeout: int = MAX_TIMEOUT) -> tuple[dict, Path]:
    if case != CASE or type(timeout) is not int or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("only bounded normal native image case is supported")
    private_path(BUILD)
    private_path(BUILD / "run")
    private_path(BUILD / "image-report.json")
    frozen = source_hash()
    manifest = load_json(private_path(BUILD / "manifest.json", regular=True))
    validate_manifest(manifest, frozen)
    preflight_esp(manifest)
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if qemu is None:
        raise RuntimeError("missing QEMU")
    qemu_hash = sha256(Path(qemu))
    code, variables = firmware()
    if not code.is_file() or not variables.is_file():
        raise ValueError("firmware must be regular selected input files")
    firmware_hashes = {"code": sha256(code), "vars_template": sha256(variables)}
    report, volume = run_system(CASE, timeout=timeout, kernel_feature=FEATURE)
    report.update(image_case=CASE, image_gate=False, image_qemu_sha256=qemu_hash)
    try:
        report["image_evidence"] = validate_artifacts(report, frozen, qemu=qemu,
                                                   qemu_sha256=qemu_hash, firmware_hashes=firmware_hashes)
        if (source_hash() != frozen or sha256(Path(qemu)) != qemu_hash
                or {"code": sha256(code), "vars_template": sha256(variables)} != firmware_hashes):
            raise ValueError("source, QEMU or firmware changed during image gate")
        report["image_gate"] = True
    except (ValueError, KeyError, TypeError, OSError) as error:
        report.update(passed=False, image_error=f"{type(error).__name__}: {error}")
    report["status"] = "completed" if report["image_gate"] else "failed"
    run = private_path(Path(report["run_directory"])).resolve(strict=True)
    publish_json(run / "result.json", report)
    publish_json(volume / "last-system-stage.json", report)
    publish_json(BUILD / "image-report.json", report)
    print(f"IMAGES {case}: passed={report['passed']} gate={report['image_gate']} {run}")
    if "image_error" in report:
        print(report["image_error"])
    return report, volume


def run_images(case: str = CASE, *, timeout: int = MAX_TIMEOUT) -> tuple[dict, Path]:
    # The current index describes THIS attempt. Preserve historical per-run
    # results, but never leave its earlier PASS visible after a new failure.
    # Unsafe report destinations are rejected before any attempted publication.
    private_path(BUILD)
    index = private_path(BUILD / "image-report.json")
    attempt = {"schema_version": 1, "status": "starting", "passed": False,
               "image_gate": False, "image_case": case if isinstance(case, str) else None,
               "timeout_s": timeout if type(timeout) is int else None}
    publish_json(index, attempt)
    try:
        return _run_images(case, timeout=timeout)
    except Exception as error:
        attempt.update(status="failed", error=f"{type(error).__name__}: {error}")
        publish_json(index, attempt)
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=(CASE,), default=CASE)
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    options = parser.parse_args()
    raise SystemExit(0 if run_images(options.case, timeout=options.timeout)[0]["image_gate"] else 1)
