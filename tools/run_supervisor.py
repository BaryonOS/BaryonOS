#!/usr/bin/env python3
"""K05 QEMU runner: one native-supervisor case per disposable guest, no disk.
The kernel prints evidence numbers and PASS markers; this host side checks the
marker sequence, the raw exit code and the case-specific numbers independently."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import re
import shutil
import subprocess
import time
import uuid
from common import ROOT, BUILD, environment, firmware, qemu_machine, save_json, sha256, source_hash

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")
FAILURES = ("PANIC", "FAIL")
FEATURE = {"normal": "native-supervisor", "crash": "test-supervisor-crash",
           "denied": "test-supervisor-denied", "parent-exit": "test-supervisor-parent-exit",
           "quota": "test-supervisor-quota", "wait": "test-supervisor-wait"}
# K06 5d-2 §4: the same six cases in native-smp-m3 on four vCPUs (multi-thread TCG).
SMP_FEATURE = {case: "native-smp-supervisor" if case == "normal" else f"test-smp-supervisor-{case}"
               for case in FEATURE}
SMP_CPUS = 4
CASE_MARKER = {"normal": "NORMAL", "crash": "CRASH", "denied": "DENIED",
               "parent-exit": "PARENT_EXIT", "quota": "QUOTA", "wait": "WAIT"}
MAX_TIMEOUT = 180

# Host-side expectations per case, checked on the kernel's evidence numbers.
EXPECT = {
    "normal": {"SPAWNS": 3, "KILLS": 2, "SPAWN_DENIED": 0, "SPAWN_QUOTA": 0, "ORPHANS_REAPED": 0,
               "CHILD_EXITS": 1, "CHILD_FAULTS": 0, "RESTARTS": 1, "CLIENT_REPLIES": 24,
               "WORKER_INSTANCES": 2, "SPARSE_SPAWNS": 1, "SPARSE_CAP_CHECKS": 3},
    "crash": {"SPAWNS": 3, "KILLS": 1, "ORPHANS_REAPED": 0, "CHILD_EXITS": 1, "CHILD_FAULTS": 1,
              "RESTARTS": 1, "CLIENT_REPLIES": 24, "WORKER_INSTANCES": 2},
    "denied": {"SPAWNS": 2, "KILLS": 1, "ORPHANS_REAPED": 0, "CHILD_FAULTS": 0, "CHILD_EXITS": 1,
               "DENIED_MATCHED": 21, "DENIED_TOTAL": 21},
    "parent-exit": {"SPAWNS": 2, "KILLS": 0, "ORPHANS_REAPED": 2, "CHILD_EXITS": 0, "CHILD_FAULTS": 0},
    "quota": {"SPAWNS": 15, "SPAWN_QUOTA": 1, "KILLS": 15, "MAX_LIVE": 16, "ORPHANS_REAPED": 0},
    "wait": {"SPAWNS": 2, "KILLS": 2, "KILL_ERRORS": 2, "ORPHANS_REAPED": 0,
             "CHILD_EXITS": 0, "CHILD_FAULTS": 0, "RESTARTS": 1, "WORKER_INSTANCES": 2,
             "DENIED_MATCHED": 4, "DENIED_TOTAL": 4, "WAITING_KILLS": 2, "WAITING_REUSES": 1,
             "WAIT_CALLS": 4, "WAIT_BLOCKED": 4, "WAIT_TIMEOUTS": 2},
}
for case in EXPECT:
    if case != "normal":
        EXPECT[case].update(SPARSE_SPAWNS=0, SPARSE_CAP_CHECKS=0)

AT_LEAST = {"normal": {"CLIENT_CLOSED": 1}, "crash": {"CLIENT_CLOSED": 1},
            "denied": {"SPAWN_DENIED": 11, "KILL_ERRORS": 3}, "parent-exit": {"CLIENT_REPLIES": 4},
            "quota": {}, "wait": {"WAIT_IDLE_IRQS": 41}}


def numbers(stdout: str) -> dict[str, int]:
    found = {}
    for line in stdout.splitlines():
        match = re.fullmatch(r"SUPERVISOR:([A-Z_]+):([0-9]{20})", line)
        if match:
            if match.group(1) in found:
                raise ValueError(f"duplicate evidence {match.group(1)}")
            found[match.group(1)] = int(match.group(2))
    return found


def expected_markers(case: str, smp: bool = False) -> tuple[str, ...]:
    online = (f"SMP:ONLINE:{SMP_CPUS}/{SMP_CPUS}",) if smp else ()
    k05 = ("SMP:K05:PASS",) if smp else ()
    return BOOT + online + ("SUPERVISOR:MANIFEST:PASS", "SUPERVISOR:READY",
                            f"SUPERVISOR:CASE:{CASE_MARKER[case]}:PASS", *k05,
                            "SUPERVISOR:RESOURCES:PASS", "SUPERVISOR:PASS", "M1:PASS")


def controlled(stdout: str, smp: bool = False) -> list[str]:
    keep = BOOT + ("SUPERVISOR:MANIFEST:PASS", "SUPERVISOR:READY", "SUPERVISOR:RESOURCES:PASS",
                   "SUPERVISOR:PASS", "M1:PASS")
    if smp:
        keep += ("SMP:K05:PASS",)
    return [line for line in stdout.splitlines() if line in keep or line.startswith("SUPERVISOR:CASE:")
            or (smp and line.startswith("SMP:ONLINE:"))]


def check_numbers(case: str, found: dict[str, int]) -> None:
    for key, value in EXPECT[case].items():
        if found.get(key) != value:
            raise ValueError(f"{key}: expected {value}, found {found.get(key)}")
    for key, minimum in AT_LEAST[case].items():
        if found.get(key, -1) < minimum:
            raise ValueError(f"{key}: expected at least {minimum}, found {found.get(key)}")
    if "TICKS" not in found or found["TICKS"] >= 6000:
        raise ValueError("missing or exhausted tick budget")
    if case == "wait" and found["WAIT_IDLE_IRQS"] > found["TICKS"]:
        raise ValueError("wait IRQ count exceeds actual monotonic ticks")


def evaluate(case: str, stdout: str, rc, timed_out: bool, smp: bool = False) -> dict[str, int]:
    """Pure verdict over a transcript; raises ValueError with the first defect."""
    if controlled(stdout, smp) != list(expected_markers(case, smp)):
        raise ValueError(f"marker sequence mismatch: {controlled(stdout, smp)}")
    if any(marker in line for line in stdout.splitlines() for marker in FAILURES):
        raise ValueError("failure marker present")
    if rc != 33 or timed_out:
        raise ValueError(f"raw exit {rc}, timeout {timed_out}")
    found = numbers(stdout)
    check_numbers(case, found)
    homes = found.get("SMP_HOMES")
    if smp != (homes is not None):
        raise ValueError("SMP_HOMES evidence must exist exactly on SMP")
    # The supervisor is the first process, so its home is CPU 0 (lowest index);
    # at least one spawned service must run on an AP (K06 5d-2 section 4).
    if smp and not (homes < 1 << SMP_CPUS and homes & ~1):
        raise ValueError(f"services not spread over CPUs: SMP_HOMES={homes}")
    return found


def run_supervisor(case: str, *, timeout=MAX_TIMEOUT, smp: bool = False):
    if case not in FEATURE or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("unknown K05 case or invalid timeout")
    feature = (SMP_FEATURE if smp else FEATURE)[case]
    frozen = source_hash()
    manifest = json.loads((BUILD / "manifest.json").read_text())
    if manifest.get("kernel_feature") != feature or manifest.get("source_sha256") != frozen:
        raise ValueError(f"build {feature} from the current source first")
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    code, variables = firmware()
    run_dir = BUILD / "run" / f"supervisor-{'smp-' if smp else ''}{case}-{uuid.uuid4().hex[:10]}"
    run_dir.mkdir(parents=True)
    shutil.copytree(BUILD / "esp", run_dir / "esp")
    shutil.copy2(variables, run_dir / "OVMF_VARS.fd")
    shutil.copy2(code, run_dir / "OVMF_CODE.fd")
    rel = run_dir.relative_to(ROOT).as_posix()
    args = [qemu, "-machine", qemu_machine(), "-accel", f"tcg,thread={'multi' if smp else 'single'}",
            "-cpu", "qemu64", "-smp", str(SMP_CPUS if smp else 1), "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
            "-serial", "stdio", "-nic", "none", "-no-reboot",
            "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
            "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
            "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    timed_out = False
    try:
        result = subprocess.run(args, cwd=ROOT, env=env, capture_output=True, timeout=timeout)
        rc, out, err = result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired as exc:
        timed_out, rc, out, err = True, None, exc.stdout or b"", exc.stderr or b""
    duration = time.monotonic() - start
    stdout, stderr = out.decode("utf-8", "replace"), err.decode("utf-8", "replace")
    (run_dir / "serial.txt").write_text(stdout, encoding="utf-8")
    (run_dir / "stderr.txt").write_text(stderr, encoding="utf-8")
    report = dict(schema_version=1, case=case, kernel_feature=feature, smp=smp, started_utc=started,
                  duration_s=duration, timeout_s=timeout, timed_out=timed_out, raw_qemu_exit=rc,
                  command=args, qemu_version=subprocess.check_output([qemu, "--version"], text=True).strip(),
                  source_fingerprint_version=3, source_sha256=frozen, run_directory=str(run_dir),
                  kernel_sha256=manifest["kernel"]["sha256"], supervisor=manifest.get("supervisor"),
                  esp_sha256={p.relative_to(run_dir).as_posix(): sha256(p) for p in (run_dir / "esp").rglob("*") if p.is_file()},
                  stdout=stdout, stderr=stderr, passed=False)
    try:
        report["evidence"] = evaluate(case, stdout, rc, timed_out, smp)
        report["passed"] = source_hash() == frozen
        if not report["passed"]:
            raise ValueError("source changed during run")
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(run_dir / "result.json", report)
    print(f"K05 {'SMP ' if smp else ''}{case}: passed={report['passed']} raw={rc} duration={duration:.1f}s {run_dir}")
    if "error" in report:
        print(report["error"])
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=sorted(FEATURE), required=True)
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    parser.add_argument("--smp", action="store_true", help=f"K06 5d-2: {SMP_CPUS} vCPUs, native-smp-m3")
    args = parser.parse_args()
    raise SystemExit(0 if run_supervisor(args.case, timeout=args.timeout, smp=args.smp)["passed"] else 1)
