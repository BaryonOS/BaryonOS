#!/usr/bin/env python3
"""Run one SMP AP-startup/worker case in an isolated, networkless QEMU guest."""
from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import json
import math
import re
import shutil
import subprocess
import time
import uuid

from common import BUILD, ROOT, environment, firmware, qemu_machine, save_json, sha256, source_hash

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS")
FAILURES = ("PANIC", "FAIL")
MAX_TIMEOUT = 180


def expected_markers(cpus: int, profile: str = "native-smp") -> tuple[str, ...]:
    if type(cpus) is not int or cpus not in (2, 4):
        raise ValueError("SMP QEMU profiles are exactly 2 or 4 CPUs")
    if profile not in ("native-smp", "native-smp-m3"):
        raise ValueError("unknown SMP kernel profile")
    markers = BOOT + ("SMP:AP:ENTRY",) * (cpus - 1) + tuple(
        f"SMP:CPU:{index}:APIC:" for index in range(cpus)
    ) + (
        f"SMP:CPUS:{cpus}",
        "SMP:RENDEZVOUS:PASS",
        "SMP:WORKERS:PASS",
        "SMP:PASS",
    )
    return markers + (("M3:SMP:SCHEDULER:PASS",) if profile == "native-smp-m3" else ())


def evaluate(cpus: int, stdout: str, returncode: int | None, timed_out: bool) -> dict:
    """Validate ordered guest evidence, distinct logical/APIC identities and exit."""
    expected_markers(cpus)
    if not isinstance(stdout, str) or len(stdout) > 128 * 1024:
        raise ValueError("invalid or oversized guest transcript")
    lines = stdout.splitlines()
    if any(marker in line for marker in FAILURES for line in lines):
        raise ValueError("failure marker present")
    cursor = 0
    for marker in BOOT:
        try:
            cursor = lines.index(marker, cursor) + 1
        except ValueError as error:
            raise ValueError(f"missing or out-of-order marker: {marker}") from error
    cpu_lines = [line for line in lines if line.startswith("SMP:CPU:")]
    cpu_pattern = re.compile(r"SMP:CPU:([0-9]+):APIC:([0-9]+):ONLINE")
    parsed = []
    for line in cpu_lines:
        match = cpu_pattern.fullmatch(line)
        if match is None:
            raise ValueError("malformed CPU identity marker")
        logical_id, apic_id = int(match.group(1)), int(match.group(2))
        if str(logical_id) != match.group(1) or str(apic_id) != match.group(2):
            raise ValueError("noncanonical CPU identity marker")
        parsed.append((logical_id, apic_id))
    if (len(parsed) != cpus
            or [index for index, _ in parsed] != list(range(cpus))
            or len({apic_id for _, apic_id in parsed}) != cpus):
        raise ValueError("CPU inventory is incomplete or contains duplicate APIC IDs")
    entries = ("SMP:AP:ENTRY",) * (cpus - 1)
    expected = BOOT + entries + tuple(cpu_lines) + (
        f"SMP:CPUS:{cpus}", "SMP:RENDEZVOUS:PASS", "SMP:WORKERS:PASS", "SMP:PASS",
    )
    observed = tuple(line for line in lines if line in BOOT or line.startswith("SMP:"))
    if observed != expected:
        raise ValueError(f"evidence sequence mismatch: {observed}")
    if returncode != 33 or timed_out:
        raise ValueError(f"raw QEMU exit {returncode}, timeout {timed_out}")
    return {
        "cpu_count": cpus,
        "logical_apic_ids": parsed,
        "rendezvous_generations": 64,
        "workers": 64,
    }


def evaluate_m3(cpus: int, stdout: str, returncode: int | None, timed_out: bool) -> dict:
    expected_markers(cpus, "native-smp-m3")
    if not isinstance(stdout, str) or len(stdout) > 128 * 1024:
        raise ValueError("invalid or oversized guest transcript")
    lines = stdout.splitlines()
    if any(marker in line for marker in FAILURES for line in lines):
        raise ValueError("failure marker present")
    boot_cursor = 0
    for marker in BOOT:
        try:
            boot_cursor = lines.index(marker, boot_cursor) + 1
        except ValueError as error:
            raise ValueError(f"missing or out-of-order marker: {marker}") from error

    cpu_lines = [line for line in lines if line.startswith("SMP:CPU:")]
    cpu_pattern = re.compile(r"SMP:CPU:([0-9]+):APIC:([0-9]+):ONLINE")
    parsed = []
    for line in cpu_lines:
        match = cpu_pattern.fullmatch(line)
        if match is None:
            raise ValueError("malformed CPU identity marker")
        logical_id, apic_id = int(match.group(1)), int(match.group(2))
        if str(logical_id) != match.group(1) or str(apic_id) != match.group(2):
            raise ValueError("noncanonical CPU identity marker")
        parsed.append((logical_id, apic_id))
    if (len(parsed) != cpus
            or [index for index, _ in parsed] != list(range(cpus))
            or len({apic_id for _, apic_id in parsed}) != cpus):
        raise ValueError("CPU inventory is incomplete or contains duplicate APIC IDs")

    expected_smp = BOOT + ("SMP:AP:ENTRY",) * (cpus - 1) + tuple(cpu_lines) + (
        f"SMP:CPUS:{cpus}", "SMP:RENDEZVOUS:PASS", "SMP:WORKERS:PASS", "SMP:PASS",
    )
    observed_smp = tuple(line for line in lines if line in BOOT or line.startswith("SMP:"))
    if observed_smp != expected_smp:
        raise ValueError(f"SMP evidence sequence mismatch: {observed_smp}")

    cpu_markers = [line for line in lines if line.startswith("M3:SMP:CPU:")]
    schedule_markers = [line for line in lines if line.startswith("M3:SMP:SCHEDULES:")]
    number = re.compile(r"M3:SMP:CPU:([0-9]{20})")
    schedules = re.compile(r"M3:SMP:SCHEDULES:([0-9]{20})")
    cpu_ids = []
    schedule_counts = []
    for line in cpu_markers:
        match = number.fullmatch(line)
        if match is None:
            raise ValueError("malformed M3 CPU marker")
        cpu_ids.append(int(match.group(1)))
    for line in schedule_markers:
        match = schedules.fullmatch(line)
        if match is None:
            raise ValueError("malformed M3 scheduler count")
        schedule_counts.append(int(match.group(1)))
    if cpu_ids != list(range(cpus)) or len(schedule_counts) != cpus:
        raise ValueError("M3 scheduling evidence is incomplete or out of order")
    if any(count == 0 for count in schedule_counts):
        raise ValueError("an online CPU never selected a user process")
    expected_m3 = tuple(
        marker
        for cpu, count in zip(cpu_ids, schedule_counts, strict=True)
        for marker in (f"M3:SMP:CPU:{cpu:020d}", f"M3:SMP:SCHEDULES:{count:020d}")
    ) + ("M3:SMP:SCHEDULER:PASS",)
    observed_m3 = tuple(
        line for line in lines
        if line.startswith(("M3:SMP:CPU:", "M3:SMP:SCHEDULES:"))
        or line == "M3:SMP:SCHEDULER:PASS"
    )
    if observed_m3 != expected_m3:
        raise ValueError("M3 scheduler evidence is malformed or out of order")
    scheduler_passes = [index for index, line in enumerate(lines)
                        if line == "M3:SMP:SCHEDULER:PASS"]
    if (len(scheduler_passes) != 1
            or scheduler_passes[0] <= max(
                index for index, line in enumerate(lines)
                if line.startswith("M3:SMP:SCHEDULES:")
            )):
        raise ValueError("missing or out-of-order SMP scheduler pass marker")
    if returncode != 33 or timed_out:
        raise ValueError(f"raw QEMU exit {returncode}, timeout {timed_out}")
    return {
        "cpu_count": cpus,
        "logical_apic_ids": parsed,
        "per_cpu_process_selections": schedule_counts,
    }


def run_smp(cpus: int, *, timeout: int = MAX_TIMEOUT, profile: str = "native-smp") -> dict:
    expected_markers(cpus, profile)
    if type(timeout) is not int or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("timeout must be in (0, 180]")
    root = ROOT.resolve(strict=True)
    build = BUILD.resolve(strict=True)
    if not build.is_relative_to(root):
        raise ValueError("NOS_BUILD_DIR must be inside the checkout")
    frozen = source_hash()
    manifest_path = build / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if (manifest.get("kernel_feature") != profile
            or manifest.get("loader_feature") is not None
            or manifest.get("source_fingerprint_version") != 3
            or manifest.get("source_sha256") != frozen):
        raise ValueError(f"build {profile} from the current source first")
    expected_artifacts = {
        "kernel.elf": manifest.get("kernel", {}).get("sha256"),
        "EFI/BOOT/BOOTX64.EFI": manifest.get("loader", {}).get("sha256"),
    }
    for relative, expected in expected_artifacts.items():
        artifact = esp_path = BUILD / "esp" / relative
        if (not isinstance(expected, str) or not re.fullmatch(r"[0-9a-f]{64}", expected)
                or artifact.is_symlink() or not artifact.is_file() or sha256(artifact) != expected):
            raise ValueError(f"{profile} build artifact mismatch: {relative}")

    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    code, variables = firmware()
    esp = build / "esp"
    if esp.is_symlink() or not esp.is_dir() or esp.resolve(strict=True).parent != build:
        raise ValueError("expected generated ESP directory under build/")
    run_root = build / "run"
    if run_root.is_symlink():
        raise ValueError("refusing linked QEMU run directory")
    run_root.mkdir(exist_ok=True)
    run_dir = run_root / f"smp-{cpus}-{uuid.uuid4().hex[:10]}"
    run_dir.mkdir()
    shutil.copytree(esp, run_dir / "esp")
    shutil.copy2(variables, run_dir / "OVMF_VARS.fd")
    shutil.copy2(code, run_dir / "OVMF_CODE.fd")
    run_rel = run_dir.relative_to(root).as_posix()
    machine = qemu_machine()
    command = [
        qemu, "-machine", machine, "-accel", "tcg,thread=multi", "-cpu", "qemu64",
        "-smp", str(cpus), "-m", "256M", "-nodefaults", "-display", "none",
        "-monitor", "none", "-serial", "stdio", "-nic", "none", "-no-reboot",
        "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={run_rel}/OVMF_CODE.fd",
        "-drive", f"if=pflash,unit=1,format=raw,file={run_rel}/OVMF_VARS.fd",
        "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{run_rel}/esp",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
    ]
    share = Path(qemu).parent / "share"
    if share.is_dir():
        command.extend(["-L", str(share)])
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    timed_out = False
    try:
        result = subprocess.run(command, cwd=root, env=env, capture_output=True, timeout=timeout)
        raw_exit, out, err = result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired as error:
        timed_out, raw_exit = True, None
        out, err = error.stdout or b"", error.stderr or b""
    duration = time.monotonic() - start
    stdout, stderr = out.decode("utf-8", "replace"), err.decode("utf-8", "replace")
    (run_dir / "serial.txt").write_bytes(out)
    (run_dir / "stderr.txt").write_bytes(err)
    version = subprocess.check_output([qemu, "--version"], env=env, text=True).strip()
    report = {
        "schema_version": 1,
        "cpus": cpus,
        "started_utc": started,
        "duration_s": duration,
        "timeout_s": timeout,
        "timed_out": timed_out,
        "raw_qemu_exit": raw_exit,
        "expected_exit": 33,
        "required_markers": [*BOOT, *(("SMP:AP:ENTRY",) * (cpus - 1)),
                             *(f"SMP:CPU:{i}:APIC:<id>:ONLINE" for i in range(cpus)),
                             f"SMP:CPUS:{cpus}", "SMP:RENDEZVOUS:PASS",
                             "SMP:WORKERS:PASS", "SMP:PASS",
                             *(("M3:SMP:SCHEDULER:PASS",) if profile == "native-smp-m3" else ())],
        "command": command,
        "qemu_version": version,
        "profile": f"{machine}/qemu64/TCG-multi/{cpus}CPU/256MiB",
        "source_fingerprint_version": 3,
        "source_sha256": frozen,
        "kernel_sha256": manifest["kernel"]["sha256"],
        "firmware_source": [str(code), str(variables)],
        "run_directory": str(run_dir),
        "stdout": stdout,
        "stderr": stderr,
        "passed": False,
        "kernel_feature": profile,
    }
    try:
        report["evidence"] = (
            evaluate_m3(cpus, stdout, raw_exit, timed_out)
            if profile == "native-smp-m3"
            else evaluate(cpus, stdout, raw_exit, timed_out)
        )
        report["passed"] = source_hash() == frozen
        if not report["passed"]:
            raise ValueError("source changed during QEMU run")
    except (ValueError, TypeError) as error:
        report["error"] = f"{type(error).__name__}: {error}"
    report["duration_s"] = duration if math.isfinite(duration) else None
    save_json(run_dir / "result.json", report)
    print(f"{profile} {cpus} CPU: passed={report['passed']} raw={raw_exit} "
          f"duration={duration:.1f}s {run_dir}")
    if "error" in report:
        print(report["error"])
    return report
