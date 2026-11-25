#!/usr/bin/env python3
"""Run and validate the opt-in native SSE2 AI job profile (`native-ai-job`)."""
from __future__ import annotations

from pathlib import Path
import re
import shutil
import uuid

from common import ROOT, BUILD, environment, qemu_machine, regular_files_below, save_json, sha256, source_hash, without_acpi_diagnostics
from build_users import inspect_user
from run import run_guest
from run_fpu import check_guest_run, load_json, private_path, publish_json

FEATURE = "native-ai-job"
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS")
END = ("AIJOB:CONTEXT_ISOLATION:PASS", "AIJOB:CLEANUP:PASS", "AIJOB:PASS", "M1:PASS")
REQUIRED = BOOT + ("AIJOB:MANIFEST:PASS", "AIJOB:SURVIVOR:PASS", "AIJOB:SLOT_REUSE:PASS") + END
FORBIDDEN = ("FAIL", "PANIC")
WORDS = 32
RUN_FIELDS = ("ITERATIONS", "START_TICKS", "END_TICKS", "MACS", "DIGEST", "ARGMAX",
              *(f"BITS{i:02d}" for i in range(WORDS)))
TOTAL_FIELDS = ("RUNS", "SLOT_REUSES", "TIMER_A", "TIMER_B", "TIMER_IRQS", "SYSCALL_CALLS", "SAVES",
                "RESTORES", "BASELINE_FRAMES", "FREE_FRAMES", "CONTEXT_BYTES_WIPED")
PIT_HZ = 100
MIN_ITERATIONS, MIN_TICKS = 2000, 50
SCOPE = ("Opt-in native-ai-job profile: one capability-free ring-3 SSE2 AI job (tiny two-layer decoder "
         "fixture, not Qwen) run twice with slot reuse beside the fpu-b register canary, using the existing "
         "eager FXSAVE64/FXRSTOR64 context. Evidence limits: QEMU TCG qemu64, one CPU, no SMP and no "
         "sleep/wake claim, no XSAVE/AVX, no physical-CPU or real-time performance claim; MACs/s are "
         "TCG guest PIT-tick figures only.")


def parse_number(line: str, label: str) -> int:
    match = re.fullmatch(re.escape(label) + r"([0-9]{20})", line)
    if not match or int(match.group(1)) > (1 << 64) - 1:
        raise ValueError(f"Malformed or overflowing native numeric field {label}")
    return int(match.group(1))


def validate_sequence(stdout: str, expected: dict) -> dict:
    """Strict serial parser: exact marker order, numeric fields and oracle equality."""
    if not isinstance(stdout, str) or len(stdout) > 128 * 1024 or any(word in stdout for word in FORBIDDEN):
        raise ValueError("Malformed or failed native serial output")
    lines = without_acpi_diagnostics(stdout).splitlines()
    if lines.count(BOOT[0]) != 1:
        raise ValueError("Missing or repeated EBS handoff")
    start = lines.index(BOOT[0])
    if any(re.search(r"(?:BOOT:|BOOTINFO:|KERNEL:|KS-|M[0-9]+:|AIJOB:)", line) for line in lines[:start]):
        raise ValueError("Unexpected marker before EBS")
    observed = iter(lines[start:])

    def take(marker: str) -> None:
        if next(observed, None) != marker:
            raise ValueError(f"Missing, repeated, reordered or unknown marker (expected {marker})")

    def fields(prefix: str, names: tuple[str, ...]) -> dict:
        values = {}
        for name in names:
            line = next(observed, None)
            if line is None:
                raise ValueError("Truncated native output")
            values[name] = parse_number(line, f"{prefix}{name}:")
        return values

    for marker in BOOT + ("AIJOB:MANIFEST:PASS",):
        take(marker)
    runs = [fields("AIJOB:RUN1:", RUN_FIELDS)]
    take("AIJOB:SURVIVOR:PASS")
    take("AIJOB:SLOT_REUSE:PASS")
    runs.append(fields("AIJOB:RUN2:", RUN_FIELDS))
    totals = fields("AIJOB:", TOTAL_FIELDS)
    for marker in END:
        take(marker)
    if next(observed, None) is not None:
        raise ValueError("Unexpected trailing native output")

    oracle_bits = expected.get("bits")
    oracle_argmax = expected.get("argmax")
    if (not isinstance(oracle_bits, list) or len(oracle_bits) != 2 * WORDS
            or not all(type(v) is int and 0 <= v < 1 << 32 for v in oracle_bits)
            or not isinstance(oracle_argmax, list) or len(oracle_argmax) != 4
            or not all(type(v) is int and 0 <= v < 256 for v in oracle_argmax)
            or type(expected.get("macs")) is not int or type(expected.get("digest")) is not int):
        raise ValueError("Malformed host oracle")
    results = []
    for number, run in enumerate(runs, 1):
        bits = []
        for i in range(WORDS):
            word = run[f"BITS{i:02d}"]
            bits += [word & 0xffff_ffff, word >> 32]
        argmax = list(run["ARGMAX"].to_bytes(8, "little")[:4])
        ticks = run["END_TICKS"] - run["START_TICKS"]
        if (run["ITERATIONS"] < MIN_ITERATIONS or run["END_TICKS"] < run["START_TICKS"] or ticks < MIN_TICKS
                or run["ARGMAX"] >> 32):
            raise ValueError(f"Run {number} is incomplete or not preempted long enough")
        if (bits != oracle_bits or run["DIGEST"] != expected["digest"] or run["MACS"] != expected["macs"]
                or argmax != oracle_argmax):
            raise ValueError(f"Run {number} differs from the host oracle")
        results.append({"run": number, "iterations": run["ITERATIONS"], "start_ticks": run["START_TICKS"],
                        "end_ticks": run["END_TICKS"], "ticks": ticks, "macs_per_iteration": run["MACS"],
                        "macs_per_second": run["MACS"] * run["ITERATIONS"] * PIT_HZ // ticks,
                        "digest": run["DIGEST"], "argmax": argmax, "bits": bits})
    if runs[0]["START_TICKS"] >= runs[1]["START_TICKS"] or runs[0]["END_TICKS"] > runs[1]["START_TICKS"]:
        raise ValueError("Runs overlap or are out of order")
    user_irqs = totals["TIMER_A"] + totals["TIMER_B"]
    if (totals["RUNS"] != 2 or totals["SLOT_REUSES"] != 1 or totals["CONTEXT_BYTES_WIPED"] != 8192
            or totals["TIMER_A"] < 2 or totals["TIMER_B"] < 2 or totals["TIMER_IRQS"] != user_irqs
            or totals["SAVES"] != user_irqs + totals["SYSCALL_CALLS"] or totals["RESTORES"] != totals["SAVES"]
            or totals["FREE_FRAMES"] == 0 or totals["FREE_FRAMES"] != totals["BASELINE_FRAMES"]):
        raise ValueError("Inconsistent native context, timer or frame evidence")
    return {"runs": results, "totals": totals}


def validate_build(build: Path, frozen: str) -> dict:
    private_path(build)
    manifest = load_json(private_path(build / "manifest.json", regular=True))
    if (manifest.get("kernel_feature") != FEATURE or manifest.get("loader_feature") is not None
            or manifest.get("source_fingerprint_version") != 3 or manifest.get("source_sha256") != frozen
            or manifest.get("sdk_runtime") is not False or manifest.get("document_session") is not False):
        raise ValueError("Build the native-ai-job profile on the current source first")
    job = load_json(private_path(build / "users/ai-job/build-report.json", regular=True))
    canary = load_json(private_path(build / "users/fpu/build-report.json", regular=True))
    if (manifest.get("ai_job") != {"job": job, "canary": canary} or job.get("passed") is not True
            or canary.get("passed") is not True or not isinstance(job.get("expected"), dict)
            or not isinstance(canary.get("apps"), list) or len(canary["apps"]) != 2):
        raise ValueError("Unbound or malformed AI job/canary build report")
    kernel_bytes = private_path(build / "esp/kernel.elf", regular=True).read_bytes()
    elf = private_path(build / "users/ai-job/ai-job-native.elf", regular=True)
    if sha256(elf) != job.get("binary_sha256") or elf.read_bytes() not in kernel_bytes:
        raise ValueError("AI job ELF hash or embedded bytes mismatch")
    app = canary["apps"][1]
    binary = private_path(build / "users/fpu/fpu-b.elf", regular=True)
    if (app.get("name") != "fpu-b" or sha256(binary) != app.get("sha256")
            or inspect_user(binary) != app.get("elf") or binary.read_bytes() not in kernel_bytes):
        raise ValueError("Canary ELF inspection/hash or embedded bytes mismatch")
    expected = {"esp/kernel.elf": manifest["kernel"]["sha256"], "esp/EFI/BOOT/BOOTX64.EFI": manifest["loader"]["sha256"]}
    for name, value in expected.items():
        if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value) or sha256(private_path(build / name, regular=True)) != value:
            raise ValueError("Kernel or loader hash mismatch")
    files = {path.relative_to(build).as_posix() for path in regular_files_below(build / "esp", validate_path=private_path)}
    if files != set(expected):
        raise ValueError("Unexpected ESP artifact")
    return {"manifest": manifest, "expected": expected, "job": job, "canary": canary}


def run_ai_job() -> dict:
    build = private_path(BUILD).resolve(strict=True)
    if not build.is_relative_to(ROOT):
        raise ValueError("NOS_BUILD_DIR must be inside the checkout")
    frozen = source_hash()
    bound = validate_build(build, frozen)
    qemu = shutil.which("qemu-system-x86_64", path=environment()["PATH"])
    if qemu is None:
        raise RuntimeError("Missing QEMU")
    qemu_hash = sha256(Path(qemu))
    private_path(build / "run")
    result_path = private_path(build / "native-ai-job-result.json")
    inputs = private_path(build / ("ai-job-inputs-" + uuid.uuid4().hex[:10]))
    inputs.mkdir()
    shutil.copyfile(build / "users/ai-job/ai-job-native.elf", inputs / "ai-job-native.elf")
    shutil.copyfile(build / "users/ai-job/build-report.json", inputs / "build-report.json")
    shutil.copyfile(build / "users/fpu/fpu-b.elf", inputs / "fpu-b.elf")
    save_json(inputs / "build-manifest.json", bound["manifest"])
    checks = {"before_launch_source_unchanged": source_hash() == frozen}
    report = {"schema_version": 1, "scope": SCOPE, "passed": False, "checks": checks,
              "pit_hz": PIT_HZ, "source_fingerprint_version": 3, "source_sha256": frozen,
              "job_binary_sha256": bound["job"]["binary_sha256"], "oracle": bound["job"]["expected"],
              "qemu_entry_path": qemu, "qemu_entry_sha256": qemu_hash, "inputs_directory": str(inputs)}
    try:
        if not all(checks.values()):
            raise ValueError("Source changed while freezing build inputs")
        guest = run_guest(FEATURE, timeout=60, required_markers=REQUIRED, expected_exit=33,
                          forbidden_markers=FORBIDDEN, expected_kernel_sha256=bound["expected"]["esp/kernel.elf"],
                          expected_loader_sha256=bound["expected"]["esp/EFI/BOOT/BOOTX64.EFI"])
        report["guest"] = guest
        report["raw_qemu_exit"] = guest.get("raw_qemu_exit")
        check_guest_run(guest, bound, frozen, qemu_machine(), qemu, build, REQUIRED)
        checks["guest_exit_command_and_artifacts"] = True
        observed = validate_sequence(guest["stdout"], bound["job"]["expected"])
        report["runs"] = observed["runs"]
        report["frames"] = {"baseline": observed["totals"]["BASELINE_FRAMES"],
                            "free_after_cleanup": observed["totals"]["FREE_FRAMES"]}
        report["totals"] = observed["totals"]
        checks["serial_sequence_strict"] = True
        checks["run1_equals_host_oracle"] = True
        checks["run2_equals_host_oracle_and_run1"] = observed["runs"][0]["bits"] == observed["runs"][1]["bits"]
        checks["context_wipe_and_entry_counts"] = True
        checks["frames_returned_to_baseline"] = True
        checks["after_run_source_unchanged"] = source_hash() == frozen
        checks["qemu_entry_unchanged"] = sha256(Path(qemu)) == qemu_hash
        checks["user_binary_snapshots"] = (sha256(inputs / "ai-job-native.elf") == bound["job"]["binary_sha256"]
                                           and sha256(inputs / "fpu-b.elf") == bound["canary"]["apps"][1]["sha256"])
        checks["build_manifest_snapshot"] = load_json(inputs / "build-manifest.json") == bound["manifest"]
        validate_build(build, frozen)
        checks["build_artifacts_unchanged"] = True
        report["passed"] = all(checks.values())
    except (ValueError, KeyError, TypeError, OSError) as error:
        report["error"] = f"{type(error).__name__}: {error}"
    publish_json(result_path, report)
    return report


if __name__ == "__main__":
    result = run_ai_job()
    print(f"native-ai-job passed={result['passed']} raw_qemu_exit={result.get('raw_qemu_exit')} "
          f"result={BUILD / 'native-ai-job-result.json'}" + (f" error={result['error']}" if "error" in result else ""))
    raise SystemExit(0 if result["passed"] else 1)
