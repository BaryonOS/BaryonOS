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
from gate_contract import guest_expectation

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS")
FAILURES = ("PANIC", "FAIL")
MAX_TIMEOUT = 180


def expected_markers(cpus: int, profile: str = "native-smp") -> tuple[str, ...]:
    if type(cpus) is not int or cpus not in (2, 4):
        raise ValueError("SMP QEMU profiles are exactly 2 or 4 CPUs")
    if profile not in (
        "native-smp",
        "native-smp-m3",
        "native-smp-m3-exit",
        "native-smp-m3-tlb-reuse",
        "native-smp-m3-lock-contention",
        "native-smp-m4",
        "native-smp-m4-lifecycle",
        "native-smp-m4-revoke",
    ):
        raise ValueError("unknown SMP kernel profile")
    if profile in (
        "native-smp-m3-tlb-reuse",
        "native-smp-m3-lock-contention",
        "native-smp-m4",
        "native-smp-m4-lifecycle",
        "native-smp-m4-revoke",
    ) and cpus != 2:
        raise ValueError("native TLB-reuse and M4 SMP profiles require exactly two CPUs")
    markers = BOOT + ("SMP:AP:ENTRY",) * (cpus - 1) + tuple(
        f"SMP:CPU:{index}:APIC:" for index in range(cpus)
    ) + (
        f"SMP:CPUS:{cpus}",
        "SMP:RENDEZVOUS:PASS",
        "SMP:WORKERS:PASS",
        "SMP:PASS",
    )
    return markers + (("M3:SMP:SCHEDULER:PASS",)
                      if profile in (
                          "native-smp-m3",
                          "native-smp-m3-exit",
                          "native-smp-m3-tlb-reuse",
                          "native-smp-m3-lock-contention",
                          "native-smp-m4",
                          "native-smp-m4-lifecycle",
                          "native-smp-m4-revoke",
                      ) else ())


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


def evaluate_m4_smp(cpus: int, stdout: str, returncode: int | None, timed_out: bool) -> dict:
    expected_markers(cpus, "native-smp-m4")
    scheduler = evaluate_m3(cpus, stdout, returncode, timed_out)
    lines = stdout.splitlines()
    required, expected_exit, forbidden = guest_expectation("m4", "real-ring3-message-exchange")
    cursor = 0
    for marker in required:
        try:
            cursor = lines.index(marker, cursor) + 1
        except ValueError as error:
            raise ValueError(f"missing or out-of-order M4 marker: {marker}") from error
    if any(marker in line for marker in forbidden for line in lines):
        raise ValueError("M4 failure marker present")
    if lines.count("M4:SMP:QUIESCED:PASS") != 1:
        raise ValueError("M4 cleanup lacked a single completed CPU-quiescence handshake")
    if not (
        lines.index("M4:SMP:QUIESCED:PASS")
        < lines.index("M4:SMP:ROUND_CLEANUP:PASS")
        < lines.index("M4:CLEANUP:PASS")
    ):
        raise ValueError("M4 cleanup was reported before CPU quiescence")
    if returncode != expected_exit or timed_out:
        raise ValueError(f"raw QEMU exit {returncode}, timeout {timed_out}")

    pattern = re.compile(r"M4:SMP:(SEND|RECV):([0-9]{20})")
    events = []
    event_lines = [(index, line) for index, line in enumerate(lines)
                   if line.startswith("M4:SMP:")
                   and line not in {
                       "M4:SMP:QUIESCED:PASS",
                       "M4:SMP:ROUND_CLEANUP:PASS",
                       "M4:SMP:ROUND_RESET:PASS",
                   }]
    for line_index, line in event_lines:
        match = pattern.fullmatch(line)
        if match is None:
            raise ValueError("malformed M4 cross-CPU IPC marker")
        operation, packed = match.groups()
        identity = int(packed)
        slot, cpu = identity >> 8, identity & 0xFF
        if slot not in (0, 1, 2) or cpu >= cpus:
            raise ValueError("M4 IPC marker has an invalid process or CPU")
        events.append((operation, slot, cpu, line_index))
    expected_ops = {("SEND", 0), ("RECV", 1), ("SEND", 1), ("RECV", 2)}
    if len(events) != len(expected_ops) or {(op, slot) for op, slot, _, _ in events} != expected_ops:
        raise ValueError("M4 exchange did not complete its two send/receive pairs")
    expected_cpu = {0: 0, 1: 1, 2: 0}
    if any(cpu != expected_cpu[slot] for _, slot, cpu, _ in events):
        raise ValueError("M4 process ran outside its acceptance-profile CPU assignment")
    exchange_index = lines.index("M4:EXCHANGE:PASS")
    if any(line_index >= exchange_index for _, _, _, line_index in events):
        raise ValueError("M4 IPC evidence appeared after the exchange pass marker")

    by_operation = {(operation, slot): cpu for operation, slot, cpu, _ in events}
    crossed = []
    for sender, receiver in ((0, 1), (1, 2)):
        send_cpu = by_operation[("SEND", sender)]
        receive_cpu = by_operation[("RECV", receiver)]
        if send_cpu != receive_cpu:
            crossed.append({
                "sender_slot": sender,
                "receiver_slot": receiver,
                "sender_cpu": send_cpu,
                "receiver_cpu": receive_cpu,
            })
    if not crossed:
        raise ValueError("no successful IPC exchange crossed a CPU boundary")
    return {
        "scheduler": scheduler,
        "ipc_events": [
            {"operation": operation.lower(), "slot": slot, "cpu": cpu}
            for operation, slot, cpu, _ in events
        ],
        "cross_cpu_message_pairs": crossed,
    }


def evaluate_m4_revoke_smp(
    cpus: int, stdout: str, returncode: int | None, timed_out: bool
) -> dict:
    expected_markers(cpus, "native-smp-m4-revoke")
    scheduler = evaluate_m3(cpus, stdout, returncode, timed_out)
    lines = stdout.splitlines()
    required, expected_exit, forbidden = guest_expectation("m4", "test-m4-revoke")
    cursor = 0
    for marker in required:
        try:
            cursor = lines.index(marker, cursor) + 1
        except ValueError as error:
            raise ValueError(f"missing or out-of-order M4 revoke marker: {marker}") from error
    if any(marker in line for marker in forbidden for line in lines):
        raise ValueError("M4 revoke failure marker present")
    if lines.count("M4:SMP:QUIESCED:PASS") != 1:
        raise ValueError("M4 revoke cleanup lacked a single completed CPU-quiescence handshake")
    if not (
        lines.index("M4:SMP:QUIESCED:PASS")
        < lines.index("M4:SMP:ROUND_CLEANUP:PASS")
        < lines.index("M4:CLEANUP:PASS")
    ):
        raise ValueError("M4 revoke cleanup was reported before CPU quiescence")
    if returncode != expected_exit or timed_out:
        raise ValueError(f"raw QEMU exit {returncode}, timeout {timed_out}")

    pattern = re.compile(
        r"M4:SMP:(SEND|RECV|REVOKE|REVOKED):([0-9]{20})"
    )
    events = []
    for index, line in enumerate(lines):
        if not line.startswith("M4:SMP:") or line in {
            "M4:SMP:QUIESCED:PASS",
            "M4:SMP:ROUND_CLEANUP:PASS",
            "M4:SMP:ROUND_RESET:PASS",
        }:
            continue
        match = pattern.fullmatch(line)
        if match is None:
            raise ValueError("malformed M4 cross-CPU revoke marker")
        operation, packed = match.groups()
        identity = int(packed)
        slot, cpu = identity >> 8, identity & 0xFF
        if slot not in (0, 1, 2) or cpu >= cpus:
            raise ValueError("M4 revoke marker has an invalid process or CPU")
        if cpu != (1 if slot == 1 else 0):
            raise ValueError("M4 revoke process ran outside its acceptance-profile CPU assignment")
        events.append((operation, slot, cpu, index))

    revokes = [event for event in events if event[0] == "REVOKE" and event[1] == 0]
    denied = [event for event in events if event[0] == "REVOKED" and event[1] in (1, 2)]
    if len(revokes) != 2 or len(denied) < 2:
        raise ValueError("M4 revoke fixture lacks the expected revoke/denial operations")
    _, _, _, second_revoke_index = revokes[-1]
    crossed = next(
        (event for event in denied
         if event[3] > second_revoke_index and event[2] != revokes[-1][2]),
        None,
    )
    if crossed is None:
        raise ValueError("revoked capability use was not rejected on another CPU")
    return {
        "scheduler": scheduler,
        "events": [
            {"operation": operation.lower(), "slot": slot, "cpu": cpu}
            for operation, slot, cpu, _ in events
        ],
        "cross_cpu_revocation": {
            "revoker_cpu": revokes[-1][2],
            "receiver_slot": crossed[1],
            "receiver_cpu": crossed[2],
        },
    }


def evaluate_m4_lifecycle_smp(
    cpus: int, stdout: str, returncode: int | None, timed_out: bool
) -> dict:
    expected_markers(cpus, "native-smp-m4-lifecycle")
    scheduler = evaluate_m3(cpus, stdout, returncode, timed_out)
    lines = stdout.splitlines()
    required, expected_exit, forbidden = guest_expectation("m4", "test-m4-lifecycle")
    cursor = 0
    for marker in required:
        try:
            cursor = lines.index(marker, cursor) + 1
        except ValueError as error:
            raise ValueError(f"missing or out-of-order M4 lifecycle marker: {marker}") from error
    if any(marker in line for marker in forbidden for line in lines):
        raise ValueError("M4 lifecycle failure marker present")
    if returncode != expected_exit or timed_out:
        raise ValueError(f"raw QEMU exit {returncode}, timeout {timed_out}")
    if lines.count("M4:SMP:QUIESCED:PASS") != 3:
        raise ValueError("M4 lifecycle did not quiesce all CPUs before each cleanup")
    if lines.count("M4:SMP:ROUND_CLEANUP:PASS") != 3:
        raise ValueError("M4 lifecycle did not complete all three cleanup rounds")
    if lines.count("M4:SMP:ROUND_RESET:PASS") != 2:
        raise ValueError("M4 lifecycle did not complete both process-table resets")
    quiesced = [i for i, line in enumerate(lines) if line == "M4:SMP:QUIESCED:PASS"]
    cleanups = [i for i, line in enumerate(lines) if line == "M4:SMP:ROUND_CLEANUP:PASS"]
    resets = [i for i, line in enumerate(lines) if line == "M4:SMP:ROUND_RESET:PASS"]
    if any(not (q < c) for q, c in zip(quiesced, cleanups)):
        raise ValueError("M4 lifecycle cleanup preceded its quiescence acknowledgement")
    if any(not (cleanups[i] < resets[i] < quiesced[i + 1]) for i in range(2)):
        raise ValueError("M4 lifecycle reset was not bracketed by completed quiescence phases")
    return {
        "scheduler": scheduler,
        "quiescence_rounds": 3,
        "cleanup_rounds": 3,
        "process_table_resets": 2,
    }


def evaluate_m3_exit_smp(
    cpus: int, stdout: str, returncode: int | None, timed_out: bool
) -> dict:
    scheduler = evaluate_m3(cpus, stdout, returncode, timed_out)
    lines = stdout.splitlines()
    required = ("M3:EXIT:PASS", "M3:SURVIVOR_PROGRESS:PASS")
    cursor = 0
    for marker in required:
        try:
            cursor = lines.index(marker, cursor) + 1
        except ValueError as error:
            raise ValueError(f"missing or out-of-order M3 exit marker: {marker}") from error
    if any(marker in line for marker in FAILURES for line in lines):
        raise ValueError("M3 exit failure marker present")
    if returncode != 33 or timed_out:
        raise ValueError(f"raw QEMU exit {returncode}, timeout {timed_out}")

    pattern = re.compile(r"M3:SMP:EXIT:([0-9]{20})")
    exits = []
    for line in lines:
        if not line.startswith("M3:SMP:EXIT:"):
            continue
        match = pattern.fullmatch(line)
        if match is None:
            raise ValueError("malformed M3 process-cleanup marker")
        identity = int(match.group(1))
        slot, cpu = identity >> 8, identity & 0xFF
        if slot != 0 or cpu >= cpus:
            raise ValueError("M3 process-cleanup marker has an invalid slot or CPU")
        exits.append((slot, cpu))
    if len(exits) != 1:
        raise ValueError("M3 exit fixture did not clean up exactly one process")
    if exits[0][1] == 0:
        raise ValueError("M3 test process exited on the BSP, not an AP")
    return {
        "scheduler": scheduler,
        "process_cleanup_event": {"slot": exits[0][0], "cpu": exits[0][1]},
    }


def evaluate_m3_tlb_reuse_smp(
    cpus: int, stdout: str, returncode: int | None, timed_out: bool
) -> dict:
    expected_markers(cpus, "native-smp-m3-tlb-reuse")
    scheduler = evaluate_m3(cpus, stdout, returncode, timed_out)
    lines = stdout.splitlines()
    if lines.count("M3:EXIT:PASS") != 2:
        raise ValueError("TLB reuse fixture did not complete two process exits")
    pattern = re.compile(r"M3:SMP:EXIT:([0-9]{20})")
    exits = []
    for index, line in enumerate(lines):
        if not line.startswith("M3:SMP:EXIT:"):
            continue
        match = pattern.fullmatch(line)
        if match is None:
            raise ValueError("malformed TLB-reuse cleanup marker")
        identity = int(match.group(1))
        slot, cpu = identity >> 8, identity & 0xFF
        if (slot, cpu) != (0, 1):
            raise ValueError("TLB-reuse process did not exit twice on its AP")
        exits.append(index)
    if len(exits) != 2:
        raise ValueError("TLB-reuse fixture lacks exactly two AP cleanup events")
    try:
        reuse_index = lines.index("M3:SMP:TLB_REUSE:PASS")
    except ValueError as error:
        raise ValueError("missing stale-TLB process-reuse evidence") from error
    if reuse_index <= exits[-1]:
        raise ValueError("TLB-reuse evidence appeared before replacement cleanup")
    if any(marker in line for marker in FAILURES for line in lines):
        raise ValueError("TLB-reuse failure marker present")
    return {
        "scheduler": scheduler,
        "process_cleanup_events": [
            {"slot": 0, "cpu": 1},
            {"slot": 0, "cpu": 1},
        ],
        "stale_mapping_target_retained_until_replacement_exit": True,
    }


def evaluate_m3_lock_contention_smp(
    cpus: int, stdout: str, returncode: int | None, timed_out: bool
) -> dict:
    expected_markers(cpus, "native-smp-m3-lock-contention")
    scheduler = evaluate_m3(cpus, stdout, returncode, timed_out)
    lines = stdout.splitlines()
    pattern = re.compile(r"M3:SMP:LOCK:CONTENDED:([0-9]{20})")
    markers = [
        (index, pattern.fullmatch(line))
        for index, line in enumerate(lines)
        if line.startswith("M3:SMP:LOCK:CONTENDED:")
    ]
    if len(markers) != 1 or markers[0][1] is None:
        raise ValueError("missing or malformed M3 lock-contention evidence")
    marker_index, match = markers[0]
    assert match is not None
    contentions = int(match.group(1))
    if contentions == 0:
        raise ValueError("M3 lock-contention fixture observed no failed acquisitions")
    if marker_index <= max(
        index for index, line in enumerate(lines)
        if line.startswith("M3:SMP:SCHEDULES:")
    ) or marker_index >= lines.index("M3:SMP:SCHEDULER:PASS"):
        raise ValueError("M3 lock-contention evidence is out of order")
    if any(marker in line for marker in FAILURES for line in lines):
        raise ValueError("M3 lock-contention failure marker present")
    return {"scheduler": scheduler, "lock_contention_attempts": contentions}


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
    expected_kernel_feature = (
        "native-smp-m4,test-m4-lifecycle"
        if profile == "native-smp-m4-lifecycle"
        else profile
    )
    if (manifest.get("kernel_feature") != expected_kernel_feature
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
                             *(("M3:SMP:SCHEDULER:PASS",)
                               if profile in (
                                   "native-smp-m3",
                                   "native-smp-m3-exit",
                                   "native-smp-m3-tlb-reuse",
                                   "native-smp-m4",
                                   "native-smp-m4-lifecycle",
                                   "native-smp-m4-revoke",
                               ) else ()),
                             *(("M4:EXCHANGE:PASS", "M4:SURVIVOR_PROGRESS:PASS",
                                "M4:CLEANUP:PASS", "M4:PASS", "M1:PASS")
                               if profile == "native-smp-m4" else ()),
                             *(("M4:CASE:REVOKE:PASS", "M4:SURVIVOR_PROGRESS:PASS",
                                "M4:CLEANUP:PASS", "KS-14:PASS", "M4:PASS", "M1:PASS")
                               if profile == "native-smp-m4-revoke" else ()),
                              *(("M4:CASE:LIFECYCLE:PASS", "M4:SURVIVOR_PROGRESS:PASS",
                                "M4:CLEANUP:PASS", "KS-16:PASS", "M4:PASS", "M1:PASS")
                               if profile == "native-smp-m4-lifecycle" else ()),
                             *(("M3:EXIT:PASS", "M3:SURVIVOR_PROGRESS:PASS")
                               if profile == "native-smp-m3-exit" else ()),
                             *(("M3:EXIT:PASS", "M3:SURVIVOR_PROGRESS:PASS",
                                "M3:SMP:TLB_REUSE:PASS")
                               if profile == "native-smp-m3-tlb-reuse" else ())],
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
        report["evidence"] = {
            "native-smp-m4": lambda: evaluate_m4_smp(cpus, stdout, raw_exit, timed_out),
            "native-smp-m4-revoke": lambda: evaluate_m4_revoke_smp(
                cpus, stdout, raw_exit, timed_out
            ),
            "native-smp-m4-lifecycle": lambda: evaluate_m4_lifecycle_smp(
                cpus, stdout, raw_exit, timed_out
            ),
            "native-smp-m3-exit": lambda: evaluate_m3_exit_smp(
                cpus, stdout, raw_exit, timed_out
            ),
            "native-smp-m3-tlb-reuse": lambda: evaluate_m3_tlb_reuse_smp(
                cpus, stdout, raw_exit, timed_out
            ),
            "native-smp-m3-lock-contention": lambda: evaluate_m3_lock_contention_smp(
                cpus, stdout, raw_exit, timed_out
            ),
            "native-smp-m3": lambda: evaluate_m3(cpus, stdout, raw_exit, timed_out),
        }.get(profile, lambda: evaluate(cpus, stdout, raw_exit, timed_out))()
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
