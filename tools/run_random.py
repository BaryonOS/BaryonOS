#!/usr/bin/env python3
"""Recheck actual RANDOM IPC alongside the original native system/disk chain.

Each case uses run_system's new disposable normal disk. This is a bounded
integration gate, not entropy-quality, persisted-key, TLS or release acceptance.
"""
from __future__ import annotations

import argparse
import hashlib
import math
from pathlib import Path
import re
import shutil
import subprocess

from common import ROOT, BUILD, TARGET, environment, firmware, qemu_machine, sha256, source_hash
from build_users import inspect_user
from run_fpu import private_path, publish_json, load_json
from run_system import MAX_TIMEOUT, OPAQUE_OBJECTS, check_disk, check_image, evaluate, run_system
import volume_disk

FEATURE = {"raw": "native-system-random", "sdk": "test-system-random-sdk",
           "unknown": "test-system-random-unknown", "wrong-caller": "test-system-random-wrong-caller"}
CASES = tuple(FEATURE)
LIMIT = 64 * 1024 * 1024
FIELDS = ("LEGACY_STATUS", "TERMINAL_TICKS", "STATUS_BEGIN_TICKS", "STATUS_END_TICKS",
          "LEGACY_POLLS_AFTER", "PROBE_PID", "KEY_PID", "APP_PID", "EXPECTED_CALLER_PID", "CASE",
          "SUCCESSES", "BYTES", "VARIABLES", "LEGACY_DENIED", "INVALID_DENIED", "QUOTA_DENIED",
          "CLOCK_READS", "ENQUEUED", "REBINDS", "STALE_DENIED", "REPLAY_DENIED", "REVOKED_DENIED",
          "SDK_BYTES", "SDK_SENDS", "SDK_CLOSES", "CANCELLED", "UNKNOWN", "NO_RETRY", "PID_DENIED",
          "START_TICKS", "END_TICKS", "REPLIES", "CAP_DENIED", "SDK_POSITIVES", "SERVICE_CALLS",
          "SERVICE_BYTES", "SEEDS", "INGRESS_SEEN", "BAD_CALLER", "INVALID_OPCODE", "RETIRED",
          "PENDING", "INVALID_REQUEST", "REPLY_ENQUEUED", "REPLY_CLOSED", "REPLY_TIMEOUT")
APP_FIELDS = ("SUCCESSES", "BYTES", "VARIABLES", "LEGACY_DENIED", "INVALID_DENIED", "QUOTA_DENIED",
              "ENQUEUED", "REBINDS", "STALE_DENIED", "REPLAY_DENIED", "REVOKED_DENIED", "SDK_BYTES",
              "SDK_SENDS", "SDK_CLOSES", "CANCELLED", "UNKNOWN", "NO_RETRY", "PID_DENIED", "REPLIES",
              "CAP_DENIED", "SDK_POSITIVES")
SERVICE_FIELDS = ("SERVICE_CALLS", "SERVICE_BYTES", "SEEDS", "INGRESS_SEEN", "BAD_CALLER",
                  "INVALID_OPCODE", "RETIRED", "PENDING", "INVALID_REQUEST", "REPLY_ENQUEUED",
                  "REPLY_CLOSED", "REPLY_TIMEOUT")
PROGRAMS = {"system_init": ("system-system", "system-init"),
            "system_app": ("system-system", "system-app"),
            "system_worker": ("system-system", "system-worker"),
            "keyring_service": ("system-keyring", "keyring-service"),
            "volume_service": ("system-volume", "volume-service"),
            "random_probe": ("random-probe-app", "random-probe")}


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def hash_value(value) -> str:
    if not isinstance(value, str) or re.fullmatch(r"[0-9a-f]{64}", value) is None:
        raise ValueError("missing or malformed SHA256")
    return value


def read(path: Path, *, cargo=False) -> bytes:
    path = private_path(path, regular=True, cargo_output=cargo)
    if path.stat().st_size > LIMIT:
        raise ValueError("unbounded RANDOM evidence")
    data = path.read_bytes()
    if len(data) > LIMIT:
        raise ValueError("RANDOM evidence grew while reading")
    return data


def validate_sequence(serial: str, case: str, raw_exit: int | None, timed_out: bool) -> dict:
    if case not in FEATURE or not isinstance(serial, str) or len(serial) > 128 * 1024:
        raise ValueError("unknown case or unbounded serial")
    if type(raw_exit) is not int or timed_out is not False:
        raise ValueError("a timeout is not a caller rejection")
    base = evaluate("normal", serial, raw_exit, timed_out)
    lines = serial.splitlines()
    observed = [line for line in lines if line.startswith("SYSTEM:RANDOM")]
    expected = ["SYSTEM:RANDOM_" + name + ":" for name in FIELDS]
    if len(observed) != len(FIELDS) + 1 or observed[-1] != "SYSTEM:RANDOM:PASS":
        raise ValueError("missing, unknown or duplicated RANDOM evidence")
    values = {}
    for prefix, line, name in zip(expected, observed, FIELDS):
        match = re.fullmatch(re.escape(prefix) + r"([0-9]{20})", line)
        if match is None or int(match[1]) > (1 << 64) - 1:
            raise ValueError("malformed, reordered or overflowing RANDOM counter")
        values[name] = int(match[1])
    if not (lines.index("SYSTEM:MANIFEST:PASS") < lines.index(observed[0])
            < lines.index(observed[-1]) < lines.index("SYSTEM:BROKERS:PASS")):
        raise ValueError("RANDOM observation is outside the live system boundary")
    if (values["CASE"] != CASES.index(case) or values["LEGACY_STATUS"] != 1
            or values["CLOCK_READS"] < 2 or values["LEGACY_POLLS_AFTER"] < 2
            or not values["START_TICKS"] <= values["TERMINAL_TICKS"] <= values["STATUS_BEGIN_TICKS"]
                   <= values["STATUS_END_TICKS"] <= values["END_TICKS"]
            or values["END_TICKS"] - values["START_TICKS"] > MAX_TIMEOUT * 100):
        raise ValueError("missing bounded clock or actual legacy RPC after RNG handling")
    identities = [values[name] for name in ("PROBE_PID", "KEY_PID", "APP_PID")]
    expected_caller = values["APP_PID" if case == "wrong-caller" else "PROBE_PID"]
    if 0 in identities or len(set(identities)) != 3 or values["EXPECTED_CALLER_PID"] != expected_caller:
        raise ValueError("RANDOM evidence belongs to different principals")
    app = dict.fromkeys(APP_FIELDS, 0)
    if case == "raw":
        app.update(SUCCESSES=89, BYTES=4737, VARIABLES=5, LEGACY_DENIED=9, INVALID_DENIED=9,
                   QUOTA_DENIED=1, ENQUEUED=110, REBINDS=1, STALE_DENIED=1, REPLAY_DENIED=1,
                   REVOKED_DENIED=1, REPLIES=109, CAP_DENIED=6)
        service = [90, 4769, 3, 110, 0, 9, 1, 0, 9, 109, 1, 0]
    elif case in ("sdk", "unknown"):
        cancel = int(case == "sdk")
        calls, count, output, positive = (5, 257, 225, 4) if cancel else (1, 64, 0, 0)
        app.update(ENQUEUED=calls, SDK_BYTES=output, SDK_SENDS=calls, SDK_CLOSES=2,
                   CANCELLED=cancel, UNKNOWN=1-cancel, NO_RETRY=1, SDK_POSITIVES=positive)
        closed = values["REPLY_CLOSED"]
        if closed not in (0, 1):
            raise ValueError("invalid cancellation reply race")
        service = [calls, count, 1, calls, 0, 0, closed, 0, 0, calls-closed, closed, 0]
    else:
        app.update(ENQUEUED=1, PID_DENIED=1)
        service = [0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0]
        if values["END_TICKS"] - values["STATUS_END_TICKS"] < 10:
            raise ValueError("wrong caller was not observed without replies after actual rejection")
    if any(values[name] != value for name, value in app.items()):
        raise ValueError("wrong RANDOM caller output, denial, delivery or cancellation counters")
    if [values[name] for name in SERVICE_FIELDS] != service:
        raise ValueError("missing actual service consume/reject/terminal output facts")
    if base["ENTROPY_BYTES"] != 1024 + 256 * values["SEEDS"]:
        raise ValueError("broker entropy does not match original keys plus actual fresh CSPRNG seeds")
    return {**base, **{"RANDOM_" + key: value for key, value in values.items()}}


def validate_manifest(manifest: dict, frozen: str, case: str) -> None:
    hash_value(frozen)
    if (type(manifest.get("schema_version")) is not int or manifest["schema_version"] != 1
            or manifest.get("source_fingerprint_version") != 3
            or manifest.get("source_sha256") != frozen or manifest.get("kernel_feature") != FEATURE[case]
            or manifest.get("loader_feature") is not None):
        raise ValueError("wrong or stale RANDOM build profile")
    for name in ("kernel", "loader"):
        hash_value(manifest[name]["sha256"])
    if set(manifest.get("system", {})) != set(PROGRAMS):
        raise ValueError("the actual six system programs are required")
    for details in manifest["system"].values():
        hash_value(details["sha256"])
        if not isinstance(details.get("elf"), dict):
            raise ValueError("missing native ELF inspection")
    if manifest.get("random_probe") != manifest["system"]["random_probe"]:
        raise ValueError("conflicting standalone/system RANDOM artifact entries")



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


def validate_artifacts(report: dict, frozen: str, case: str, *, qemu: str,
                       qemu_sha256: str, firmware_hashes: dict) -> dict:
    """Recheck bytes, not a cached passed flag or copied JSON claim."""
    if (case not in FEATURE or type(report.get("schema_version")) is not int or report["schema_version"] != 1
            or report.get("passed") is not True or report.get("case") != "normal"
            or report.get("random_case") != case or type(report.get("random_gate")) is not bool
            or report.get("kernel_feature") != FEATURE[case]
            or report.get("source_fingerprint_version") != 3 or report.get("source_sha256") != frozen
            or type(report.get("timeout_s")) is not int or not 0 < report["timeout_s"] <= MAX_TIMEOUT
            or type(report.get("duration_s")) not in (int, float)
            or not math.isfinite(report["duration_s"]) or not 0 <= report["duration_s"] <= report["timeout_s"] + 5
            or not isinstance(report.get("source_commit"), str)
            or re.fullmatch(r"[0-9a-f]{40}", report["source_commit"]) is None):
        raise ValueError("RANDOM guest report source/profile/exit is not bound")
    if hash_value(report.get("random_qemu_sha256")) != hash_value(qemu_sha256):
        raise ValueError("RANDOM emulator differs from its frozen executable")
    manifest = report["manifest"]
    validate_manifest(manifest, frozen, case)
    if load_json(private_path(BUILD / "manifest.json", regular=True)) != manifest:
        raise ValueError("RANDOM result is not the current inspected build")
    run = private_path(Path(report["run_directory"])).resolve(strict=True)
    volume = private_path(Path(report["volume_directory"])).resolve(strict=True)
    parent = (BUILD / "run").resolve(strict=True)
    if run.parent != parent or volume.parent != parent or run == volume:
        raise ValueError("RANDOM artifacts escaped separate generated directories")
    if (re.fullmatch(r"system-normal-[0-9a-f]{32}", run.name) is None
            or re.fullmatch(r"system-image-[0-9a-f]{32}", volume.name) is None):
        raise ValueError("RANDOM run is not a fresh normal system image")
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
        raise ValueError("different QEMU/firmware/disk/entropy profile or reused snapshot")
    if report.get("firmware_sha256") != firmware_hashes or digest(read(run / "OVMF_CODE.fd")) != firmware_hashes["code"]:
        raise ValueError("firmware differs from the selected pre-boot inputs")
    read(run / "OVMF_VARS.fd")  # Writable UEFI state is not falsely compared to its pre-boot hash.
    actual_esp = set()
    for path in (run / "esp").rglob("*"):
        private_path(path)
        if path.is_file():
            actual_esp.add(path.relative_to(run).as_posix())
    expected = {"esp/kernel.elf": manifest["kernel"]["sha256"],
                "esp/EFI/BOOT/BOOTX64.EFI": manifest["loader"]["sha256"]}
    if actual_esp != set(expected) or report.get("esp_sha256") != expected:
        raise ValueError("unexpected or unbound files in actual guest ESP")
    for name, expected_hash in expected.items():
        if digest(read(run / name)) != expected_hash or digest(read(BUILD / name)) != expected_hash:
            raise ValueError("actual guest ESP differs from built kernel/loader")
    kernel = read(run / "esp/kernel.elf")
    for name, (directory, filename) in PROGRAMS.items():
        path = TARGET / directory / "x86_64-unknown-none/release" / filename
        binary = read(path, cargo=True)  # Read-only Cargo hardlinks, never write through them.
        if (digest(binary) != manifest["system"][name]["sha256"]
                or inspect_user(path) != manifest["system"][name]["elf"] or binary not in kernel):
            raise ValueError("native program hash, ELF inspection or kernel embedding mismatch")
    initial = load_json(private_path(volume / "volume.json", regular=True))
    empty = volume_disk.empty_image_bytes()
    if (initial.get("variant") != "empty" or initial.get("sha256") != digest(empty)
            or report.get("before_sha256") != digest(empty)):
        raise ValueError("RANDOM needs a new empty disposable image, not a previous snapshot")
    image, snapshot = volume / "volume.img", run / "volume.after.img"
    check_image(image)
    check_image(snapshot)
    data = read(image)
    if digest(data) != hash_value(report.get("after_sha256")) or read(snapshot) != data:
        raise ValueError("RANDOM post-run disk snapshot changed")
    oracle = volume_disk.analyse(data, OPAQUE_OBJECTS)
    check_disk(oracle)
    if oracle["objects"][0]["request_id"] != 3:
        raise ValueError("the original document-session abort/save transaction changed")
    if report.get("oracle") != oracle:
        raise ValueError("reported disk oracle differs from actual committed bytes")
    serial = read(run / "serial.txt").decode("utf-8", "replace")
    observed = validate_sequence(serial, case, report.get("raw_qemu_exit"), report.get("timed_out"))
    if report.get("evidence") != observed:
        raise ValueError("stored counters differ from actual serial bytes")
    return observed


def _run_random(case: str, *, timeout: int = MAX_TIMEOUT) -> tuple[dict, Path]:
    if case not in FEATURE or type(timeout) is not int or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("unknown RANDOM case or unbounded timeout")
    private_path(BUILD)
    private_path(BUILD / "run")
    private_path(BUILD / "random-report.json")
    frozen = source_hash()
    manifest = load_json(private_path(BUILD / "manifest.json", regular=True))
    validate_manifest(manifest, frozen, case)
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
    # Shared runner admits these four profiles ONLY as normal, each with a new disk.
    report, volume = run_system("normal", timeout=timeout, kernel_feature=FEATURE[case])
    report.update(random_case=case, random_gate=False, random_qemu_sha256=qemu_hash)
    try:
        validate_artifacts(report, frozen, case, qemu=qemu, qemu_sha256=qemu_hash,
                           firmware_hashes=firmware_hashes)
        if (source_hash() != frozen or sha256(Path(qemu)) != qemu_hash
                or {"code": sha256(code), "vars_template": sha256(variables)} != firmware_hashes):
            raise ValueError("source, QEMU or firmware changed during RANDOM gate")
        report["random_gate"] = True
    except (ValueError, KeyError, TypeError, OSError) as error:
        report.update(passed=False, random_error=f"{type(error).__name__}: {error}")
    report["status"] = "completed" if report["random_gate"] else "failed"
    run = private_path(Path(report["run_directory"])).resolve(strict=True)
    publish_json(run / "result.json", report)
    publish_json(volume / "last-system-stage.json", report)
    publish_json(BUILD / "random-report.json", report)
    print(f"RANDOM {case}: passed={report['passed']} gate={report['random_gate']} {run}")
    if "random_error" in report:
        print(report["random_error"])
    return report, volume


def run_random(case: str, *, timeout: int = MAX_TIMEOUT) -> tuple[dict, Path]:
    # The current index describes THIS attempt. Preserve historical per-run
    # results, but never leave its earlier PASS visible after a new failure.
    # Unsafe report destinations are rejected before any attempted publication.
    private_path(BUILD)
    index = private_path(BUILD / "random-report.json")
    attempt = {"schema_version": 1, "status": "starting", "passed": False,
               "random_gate": False, "random_case": case if isinstance(case, str) else None,
               "timeout_s": timeout if type(timeout) is int else None}
    publish_json(index, attempt)
    try:
        return _run_random(case, timeout=timeout)
    except Exception as error:
        attempt.update(status="failed", error=f"{type(error).__name__}: {error}")
        publish_json(index, attempt)
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=CASES, required=True)
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    options = parser.parse_args()
    raise SystemExit(0 if run_random(options.case, timeout=options.timeout)[0]["random_gate"] else 1)
