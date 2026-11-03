#!/usr/bin/env python3
"""Run and validate the native caller-to-qsim userspace job interface."""
from __future__ import annotations
import importlib.util
import json
from pathlib import Path
import re
from common import ROOT, BUILD, save_json, sha256, source_hash, without_acpi_diagnostics
from run import run_guest

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS")
CASES = (
    "QSIMJOB:BELL:PASS", "QSIMJOB:SIX_QUBIT:PASS", "QSIMJOB:CANCEL:PASS",
    "QSIMJOB:WORK_BUDGET:PASS", "QSIMJOB:RESULT_PAGING:PASS", "QSIMJOB:IPC_AUTHORITY:PASS",
)
END = ("QSIMJOB:CLEANUP:PASS", "QSIMJOB:PASS", "M1:PASS")
REQUIRED = BOOT + ("QSIMJOB:MANIFEST:PASS",) + CASES + END
FORBIDDEN = ("FAIL", "PANIC")

def load_json(path: Path):
    return json.loads(path.read_text())

def builder():
    path = ROOT / "experiments/quantum/native/job/build.py"
    spec = importlib.util.spec_from_file_location("qsim_job_builder", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module

def number(line: str, label: str) -> int:
    match = re.fullmatch(re.escape(label) + r"([0-9]{20})", line)
    if not match:
        raise ValueError(f"Malformed numeric field: {label}")
    return int(match.group(1))

def validate_sequence(stdout: str) -> dict:
    if any(word in stdout for word in FORBIDDEN):
        raise ValueError("Native qsim job failure marker")
    stdout = without_acpi_diagnostics(stdout)
    lines = stdout.splitlines()
    start = lines.index(BOOT[0])
    observed = lines[start:]
    if len(observed) != 24 or tuple(observed[:8]) != BOOT + ("QSIMJOB:MANIFEST:PASS",):
        raise ValueError("Unexpected qsim job boot/manifest sequence")
    data = {
        "service_pid": number(observed[8], "QSIMJOB:SERVICE_PID:"),
        "client_pid": number(observed[9], "QSIMJOB:CLIENT_PID:"),
        "max_work": number(observed[10], "QSIMJOB:SERVICE_MAX_WORK:"),
        "peak_arena": number(observed[11], "QSIMJOB:SERVICE_PEAK_ARENA:"),
        "timer_irqs": number(observed[12], "QSIMJOB:TIMER_IRQS:"),
        "syscalls": number(observed[13], "QSIMJOB:SYSCALL_CALLS:"),
    }
    if tuple(observed[14:20]) != CASES:
        raise ValueError("Missing/reordered qsim job cases")
    data["cleanup_frames"] = number(observed[20], "QSIMJOB:CLEANUP_FRAMES:")
    if tuple(observed[21:]) != END:
        raise ValueError("Missing qsim job cleanup/end markers")
    if (data["service_pid"], data["client_pid"]) != (1, 2):
        raise ValueError("Unexpected process identities")
    if min(data["max_work"], data["peak_arena"], data["timer_irqs"], data["syscalls"], data["cleanup_frames"]) == 0:
        raise ValueError("Missing runtime/resource evidence")
    if data["peak_arena"] > 65536:
        raise ValueError("Arena quota exceeded")
    return data

def run() -> dict:
    frozen = source_hash()
    manifest = load_json(BUILD / "manifest.json")
    if manifest.get("kernel_feature") != "native-qsim-job" or manifest.get("source_sha256") != frozen:
        raise ValueError("Build the current native-qsim-job profile first")
    artifact = load_json(BUILD / "users/qsim-job/build-report.json")
    module = builder()
    elf = BUILD / "users/qsim-job/qsim-job-native.elf"
    if artifact.get("passed") is not True or artifact.get("source_sha256") != module.sources():
        raise ValueError("Stale qsim job artifact report")
    if sha256(elf) != artifact.get("binary_sha256") or elf.read_bytes() not in (BUILD / "esp/kernel.elf").read_bytes():
        raise ValueError("Selected qsim job ELF is not bound into the kernel")
    guest = run_guest(
        "native-qsim-job", esp=BUILD / "esp", timeout=60, required_markers=REQUIRED,
        expected_exit=33, forbidden_markers=FORBIDDEN,
        expected_kernel_sha256=manifest["kernel"]["sha256"],
        expected_loader_sha256=manifest["loader"]["sha256"],
    )
    report = {
        "schema_version": 1,
        "scope": "native qsim job interface: reference circuits, capability IPC, cancellation, work budget, paged results and cleanup",
        "source_fingerprint_version": 3,
        "source_sha256": frozen,
        "passed": False,
        "guest": guest,
    }
    try:
        if guest.get("passed") is not True or guest.get("raw_qemu_exit") != 33:
            raise ValueError("Guest result not accepted")
        report["observed"] = validate_sequence(guest["stdout"])
        if source_hash() != frozen:
            raise ValueError("Source changed during qsim job run")
        report["passed"] = True
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(BUILD / "native-qsim-job-result.json", report)
    print(f"Native qsim job accepted={report['passed']}")
    if "error" in report:
        print(report["error"])
    return report

if __name__ == "__main__":
    raise SystemExit(0 if run()["passed"] else 1)
