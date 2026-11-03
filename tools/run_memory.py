#!/usr/bin/env python3
"""Validate real own-heap growth, CPU faults and physical OOM on isolated QEMU."""
from __future__ import annotations

import argparse
import math
from pathlib import Path
import re
import shutil
import sys

from common import ROOT, BUILD, TARGET, environment, qemu_machine, run_logged, sha256, source_hash
from build_users import inspect_user
from run import run_guest
from run_fpu import private_path, publish, publish_json, load_json

CASES = ("normal", "nx", "guard", "peer", "oom")
FEATURES = {case: "native-memory" + ("" if case == "normal" else "-" + case) for case in CASES}
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS")
FIELDS = ("CLEAN_STARTS", "GROWN_PAGES", "ZERO_BYTES", "ARG_DENIED", "QUOTA_DENIED",
          "PRIVATE_CHECKS", "COLLECTION_RUNS", "ALLOC_OOM_DENIED", "FAULTS", "EXITS",
          "SLOT_REUSES", "OOM_FAILURES", "OOM_RECOVERIES", "RETRY_ZERO_BYTES",
          "PRESSURE_NORMAL", "PRESSURE_ELEVATED", "PRESSURE_CRITICAL",
          "RECLAIM_OPPORTUNITIES", "RECLAIMED_PROCESSES", "RECLAIMED_FRAMES",
          "PRESSURE_RECOVERED",
          "TIMER_IRQS", "LIVE_PROCESSES", "FRAMES_RECOVERED", "KERNEL_TABLES_RETAINED",
          "CLEANUP_ZERO_BYTES")
FORBIDDEN = (":FAIL", "PANIC")


def endings(case: str) -> tuple[str, ...]:
    if case not in CASES:
        raise ValueError("Unknown native memory case")
    return (("MEMORY:GROWTH:PASS", "MEMORY:ISOLATION:PASS", "MEMORY:PRESSURE:PASS")
            + (("MEMORY:RECLAIM:PASS",) if case == "oom" else ())
            + (("MEMORY:OOM:PASS",) if case == "oom" else ())
            + ("MEMORY:CLEANUP:PASS", f"MEMORY:CASE:{case.upper()}:PASS", "M1:PASS"))


def required(case: str) -> tuple[str, ...]:
    return BOOT + ("MEMORY:MANIFEST:PASS",) + endings(case)


def build_users() -> tuple[dict, dict]:
    """Common builder hook. Inspect and snapshot the exact linked user ELF."""
    private_path(BUILD)
    private_path(TARGET)
    output = private_path(BUILD / "users/memory")
    output.mkdir(parents=True, exist_ok=True)
    for name in ("memory-probe.elf", "build-report.json"):
        private_path(output / name)
    run_logged([sys.executable, "-B", str(ROOT / "sdk/examples/memory/build.py")],
               "build-memory-user", extra_env={"CARGO_TARGET_DIR": str(TARGET / "memory-app")})
    source = private_path(TARGET / "memory-app/x86_64-unknown-none/release/memory-probe",
                          regular=True, cargo_output=True)
    binary = output / "memory-probe.elf"
    publish(binary, source.read_bytes())
    artifact = {"passed": True, "native_runtime_performed": False,
                "sha256": sha256(binary), "elf": inspect_user(binary)}
    publish_json(output / "build-report.json", artifact)
    return {"NOS_MEMORY_ELF": str(binary.resolve())}, artifact


def validate_sequence(stdout: str, case: str, image_pages: int) -> dict:
    end = endings(case)
    if (not isinstance(stdout, str) or len(stdout) > 128 * 1024
            or any(word in stdout for word in FORBIDDEN)
            or type(image_pages) is not int or not 1 <= image_pages < 1500):
        raise ValueError("Invalid memory evidence or image geometry")
    lines = stdout.splitlines()
    if lines.count(BOOT[0]) != 1:
        raise ValueError("Missing or repeated boot handoff")
    start = lines.index(BOOT[0])
    if any(re.search(r"(?:BOOT:|BOOTINFO:|KERNEL:|KS-|M[0-9]+:|MEMORY:)", line) for line in lines[:start]):
        raise ValueError("Native marker before boot handoff")
    lines = lines[start:]
    prefix = BOOT + ("MEMORY:MANIFEST:PASS",)
    if (len(lines) != len(prefix) + len(FIELDS) + len(end)
            or tuple(lines[:len(prefix)]) != prefix or tuple(lines[-len(end):]) != end):
        raise ValueError("Unknown, missing, duplicated or reordered memory evidence")
    numbers = {}
    for name, line in zip(FIELDS, lines[len(prefix):]):
        match = re.fullmatch(re.escape("MEMORY:" + name + ":") + r"([0-9]{20})", line)
        if match is None or int(match.group(1)) > (1 << 64) - 1:
            raise ValueError("Malformed memory counter")
        numbers[name] = int(match.group(1))
    lives = 16 if case == "oom" else 3
    fixed = {"CLEAN_STARTS": lives, "ARG_DENIED": 7 * lives, "PRIVATE_CHECKS": lives,
             "COLLECTION_RUNS": lives, "ALLOC_OOM_DENIED": lives,
             "FAULTS": int(case in ("nx", "guard", "peer")),
             "EXITS": lives - int(case in ("nx", "guard", "peer")),
             "SLOT_REUSES": int(case != "oom"), "LIVE_PROCESSES": 0, "FRAMES_RECOVERED": 1,
             "PRESSURE_NORMAL": 1, "PRESSURE_ELEVATED": int(case == "oom"),
             "PRESSURE_CRITICAL": int(case == "oom"),
             "RECLAIM_OPPORTUNITIES": int(case == "oom"),
             "RECLAIMED_PROCESSES": 15 if case == "oom" else 0,
             "RECLAIMED_FRAMES": int(case == "oom"),
             "PRESSURE_RECOVERED": int(case == "oom"),
             "OOM_RECOVERIES": int(case == "oom"), "RETRY_ZERO_BYTES": 4096 * int(case == "oom")}
    if any(numbers[name] != value for name, value in fixed.items()):
        raise ValueError("Wrong memory lifecycle/refusal/cleanup counters")
    if case == "oom":
        if (numbers["PRESSURE_ELEVATED"] != 1
                or numbers["PRESSURE_CRITICAL"] != 1
                or numbers["RECLAIM_OPPORTUNITIES"] != 1
                or numbers["RECLAIMED_PROCESSES"] != 15
                or numbers["RECLAIMED_FRAMES"] == 0
                or numbers["PRESSURE_RECOVERED"] != 1):
            raise ValueError("Physical pressure was not recovered by bounded cleanup")
    elif any(numbers[name] != 0 for name in (
            "PRESSURE_ELEVATED", "PRESSURE_CRITICAL", "RECLAIM_OPPORTUNITIES",
            "RECLAIMED_PROCESSES", "RECLAIMED_FRAMES", "PRESSURE_RECOVERED")):
        raise ValueError("Unexpected pressure/reclaim evidence in non-OOM profile")
    if (numbers["TIMER_IRQS"] < 2 or numbers["ZERO_BYTES"] != numbers["GROWN_PAGES"] * 4096
            or numbers["KERNEL_TABLES_RETAINED"] > 256
            or numbers["CLEANUP_ZERO_BYTES"] != numbers["ZERO_BYTES"] + lives * 65536):
        raise ValueError("Incomplete memory/zeroing/preemption evidence")
    maximum = 2048 - (image_pages + 16 + 16)  # Program, initial heap and user stack.
    if case == "oom":
        if (not 1 <= numbers["OOM_FAILURES"] <= 16
                or numbers["QUOTA_DENIED"] + numbers["OOM_FAILURES"] != 16
                or not 16 * 64 <= numbers["GROWN_PAGES"] <= 16 * maximum):
            raise ValueError("Physical OOM/recovery was not established")
    elif (numbers["QUOTA_DENIED"] != 6 or numbers["OOM_FAILURES"] != 0
          or numbers["GROWN_PAGES"] != lives * maximum or maximum < 514):
        raise ValueError("Full process quota or cross-page-table growth was not established")
    return numbers


def validate_build(build: Path, frozen: str, case: str) -> dict:
    manifest = load_json(private_path(build / "manifest.json", regular=True))
    if (manifest.get("kernel_feature") != FEATURES[case] or manifest.get("loader_feature") is not None
            or manifest.get("source_fingerprint_version") != 3 or manifest.get("source_sha256") != frozen
            or manifest.get("sdk_runtime") is not False or manifest.get("document_session") is not False):
        raise ValueError("Build the selected native memory profile on the current source")
    artifact = load_json(private_path(build / "users/memory/build-report.json", regular=True))
    if (manifest.get("memory") != artifact or artifact.get("passed") is not True
            or artifact.get("native_runtime_performed") is not False):
        raise ValueError("Unbound memory app report")
    binary = private_path(build / "users/memory/memory-probe.elf", regular=True)
    kernel = private_path(build / "esp/kernel.elf", regular=True)
    if (sha256(binary) != artifact.get("sha256") or inspect_user(binary) != artifact.get("elf")
            or binary.read_bytes() not in kernel.read_bytes()):
        raise ValueError("Memory user ELF inspection/hash/embedding mismatch")
    expected = {"esp/kernel.elf": manifest["kernel"]["sha256"],
                "esp/EFI/BOOT/BOOTX64.EFI": manifest["loader"]["sha256"]}
    actual = set()
    for path in (build / "esp").rglob("*"):
        private_path(path)
        if path.is_file():
            actual.add(path.relative_to(build).as_posix())
    if actual != set(expected):
        raise ValueError("Unexpected memory ESP file")
    for name, value in expected.items():
        if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value) or sha256(private_path(build / name, regular=True)) != value:
            raise ValueError("Memory kernel/loader mismatch")
    image_pages = sum((s["page_end"] - s["page_start"]) // 4096 for s in artifact["elf"]["segments"])
    return {"manifest": manifest, "artifact": artifact, "expected": expected, "image_pages": image_pages}


def validate_guest(guest: dict, bound: dict, frozen: str, case: str, build: Path, qemu: str) -> dict:
    ram = 128 if case == "oom" else 256
    duration = guest.get("duration_s")
    if (type(guest.get("raw_qemu_exit")) is not int or guest["raw_qemu_exit"] != 33
            or guest.get("timed_out") is not False or guest.get("passed") is not True
            or guest.get("timeout_s") != 60 or type(guest.get("timeout_s")) is not int
            or type(duration) not in (int, float) or not math.isfinite(duration) or not 0 <= duration <= 65
            or not isinstance(guest.get("qemu_version"), str)
            or not guest["qemu_version"].startswith("QEMU emulator version ")
            or guest.get("expected_exit") != 33 or guest.get("artifact_binding_passed") is not True
            or guest.get("required_markers") != list(required(case))
            or guest.get("forbidden_markers") != list(FORBIDDEN)
            or guest.get("source_fingerprint_version") != 3 or guest.get("source_sha256") != frozen):
        raise ValueError("Memory guest exit/source/expectation mismatch")
    directory = private_path(Path(guest["run_directory"])).resolve(strict=True)
    if not directory.is_relative_to((build / "run").resolve()) or not directory.is_relative_to(ROOT):
        raise ValueError("Memory guest evidence escaped private build")
    relative = directory.relative_to(ROOT).as_posix()
    machine = qemu_machine()
    command = [qemu, "-machine", machine, "-accel", "tcg,thread=single", "-cpu", "qemu64",
               "-smp", "1", "-m", f"{ram}M", "-nodefaults", "-display", "none", "-monitor", "none",
               "-serial", "stdio", "-nic", "none", "-no-reboot",
               "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={relative}/OVMF_CODE.fd",
               "-drive", f"if=pflash,unit=1,format=raw,file={relative}/OVMF_VARS.fd",
               "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{relative}/esp",
               "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
    if (Path(qemu).parent / "share").is_dir():
        command += ["-L", str(Path(qemu).parent / "share")]
    if (guest.get("command") != command
            or guest.get("profile") != f"development-package; {machine}/qemu64/TCG/1CPU/{ram}MiB"):
        raise ValueError("Memory CPU/RAM/network/command mismatch")
    hashes = guest.get("artifact_sha256", {})
    if (set(hashes) != set(bound["expected"]) | {"OVMF_CODE.fd", "OVMF_VARS.template.fd"}
            or guest.get("expected_artifact_sha256") != bound["expected"]):
        raise ValueError("Unexpected memory snapshot set")
    actual_esp = set()
    for path in (directory / "esp").rglob("*"):
        private_path(path)
        if path.is_file():
            actual_esp.add(path.relative_to(directory).as_posix())
    if actual_esp != set(bound["expected"]):
        raise ValueError("Unexpected file in actual memory guest ESP")
    for name, value in hashes.items():
        if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value) or sha256(private_path(directory / name, regular=True)) != value:
            raise ValueError("Changed memory guest snapshot")
    if any(hashes[name] != value for name, value in bound["expected"].items()):
        raise ValueError("Memory guest image differs from inspected build")
    if (private_path(directory / "serial.txt", regular=True).read_bytes().decode("utf-8", "replace")
            != guest.get("stdout")):
        raise ValueError("Memory report differs from actual serial bytes")
    return validate_sequence(guest["stdout"], case, bound["image_pages"])


def run_memory(case: str = "normal") -> dict:
    if case not in CASES:
        raise ValueError("Unknown memory case")
    build = private_path(BUILD).resolve(strict=True)
    frozen = source_hash()
    bound = validate_build(build, frozen, case)
    private_path(build / "run")
    private_path(build / "memory-report.json")
    qemu = shutil.which("qemu-system-x86_64", path=environment()["PATH"])
    if qemu is None:
        raise RuntimeError("Missing QEMU")
    qemu_hash = sha256(Path(qemu))
    report = {"schema_version": 1, "case": case, "passed": False,
              "scope": "own RW/NX heap and process isolation on one QEMU CPU; 128 MiB physical OOM is a negative profile, not the product profile",
              "source_fingerprint_version": 3, "source_sha256": frozen,
              "qemu_entry_sha256": qemu_hash, "build_manifest": bound["manifest"]}
    try:
        guest = run_guest("native-memory-" + case, timeout=60, required_markers=required(case),
                          forbidden_markers=FORBIDDEN, expected_exit=33,
                          expected_kernel_sha256=bound["expected"]["esp/kernel.elf"],
                          expected_loader_sha256=bound["expected"]["esp/EFI/BOOT/BOOTX64.EFI"],
                          memory_mib=128 if case == "oom" else 256)
        report["guest"] = guest
        report["observed"] = validate_guest(guest, bound, frozen, case, build, qemu)
        validate_build(build, frozen, case)
        report["checks"] = {"source_unchanged": source_hash() == frozen,
                            "qemu_unchanged": sha256(Path(qemu)) == qemu_hash,
                            "strict_guest_and_image_binding": True}
        report["passed"] = all(report["checks"].values())
    except (ValueError, KeyError, TypeError, OSError) as error:
        report["error"] = f"{type(error).__name__}: {error}"
    publish_json(build / "memory-report.json", report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=CASES, default="normal")
    raise SystemExit(0 if run_memory(parser.parse_args().case)["passed"] else 1)
