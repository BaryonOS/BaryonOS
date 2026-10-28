#!/usr/bin/env python3
"""Run an isolated, networkless QEMU guest and retain raw evidence."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
from pathlib import Path
import shutil
import subprocess
import time
import uuid
from common import ROOT, BUILD, LOCAL, environment, firmware, qemu_machine, save_json, sha256, source_hash
from gate_contract import (
    FAIL_MARKERS as BASE_FAIL_MARKERS,
    MILESTONE_MARKERS as BASE_MILESTONE_MARKERS,
    M1_MARKERS,
    guest_expectation,
)

MARKERS = M1_MARKERS
FAIL_MARKERS = BASE_FAIL_MARKERS + ("M5:FAIL",)
LATER_MILESTONE_PREFIXES = ("M2:", "M3:", "M4:", "M5:", "M6A:", "M6B:", "M6C:", "QSIM:")
MILESTONE_MARKERS = {
    **BASE_MILESTONE_MARKERS,
    "m5": MARKERS[:3] + ("M5:CASE:READ:PASS", "M5:SURVIVOR_PROGRESS:PASS",
                         "M5:RESOURCES:PASS", "M5:PASS", "M1:PASS"),
}


def is_success(returncode, stdout, timed_out=False, *, required_markers=MARKERS,
               expected_exit=33, forbidden_markers=FAIL_MARKERS):
    required_markers = tuple(required_markers)
    m1_only = ("M1:PASS" in required_markers
               and not any(marker.startswith(LATER_MILESTONE_PREFIXES) for marker in required_markers))
    if m1_only and any(line.startswith(LATER_MILESTONE_PREFIXES) for line in stdout.splitlines()):
        return False
    lines = iter(stdout.splitlines())
    for marker in required_markers:
        if not any(line == marker for line in lines):
            return False
    return (returncode == expected_exit and not timed_out
            and not any(marker in stdout for marker in forbidden_markers))


def run_guest(label="smoke", *, esp=None, timeout=60, required_markers=MARKERS,
              expected_exit=33, forbidden_markers=FAIL_MARKERS, m5_case=None, m5_seed=None,
              expected_kernel_sha256=None, expected_loader_sha256=None):
    if not BUILD.resolve().is_relative_to(ROOT.resolve()):
        raise ValueError("NOS_BUILD_DIR must be inside the checkout for QEMU runs")
    if not 0 < timeout <= 60:
        raise ValueError("guest timeout must be in (0,60]")
    machine = qemu_machine()
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    code, variables = firmware()
    source = Path(esp or BUILD / "esp").resolve(strict=True)
    if not source.is_relative_to(BUILD.resolve()):
        raise ValueError("ESP must be a generated directory under build/")
    run_dir = BUILD / "run" / f"{label}-{uuid.uuid4().hex[:10]}"
    run_dir.mkdir(parents=True)
    disk_path, disk_info = None, None
    if m5_case is not None:
        from build import M5_CASES
        from m5_disk import create_disk
        if m5_case not in ("read",) + M5_CASES:
            raise ValueError("Unknown M5 case")
        disk_path, disk_info = create_disk(run_dir, m5_seed)
    shutil.copytree(source, run_dir / "esp")
    expected_artifacts = {}
    for name, expected in (("esp/kernel.elf", expected_kernel_sha256),
                           ("esp/EFI/BOOT/BOOTX64.EFI", expected_loader_sha256)):
        if expected is not None:
            actual = sha256(run_dir / name)
            if actual != expected:
                raise ValueError(f"Snapshotted artifact SHA256 mismatch: {name}; expected={expected}; actual={actual}")
            expected_artifacts[name] = expected
    shutil.copy2(variables, run_dir / "OVMF_VARS.template.fd")
    shutil.copy2(run_dir / "OVMF_VARS.template.fd", run_dir / "OVMF_VARS.fd")
    shutil.copy2(code, run_dir / "OVMF_CODE.fd")
    # Use paths relative to cwd: drive-letter colons and commas are not passed
    # through QEMU's file-option parser, and the FAT tree cannot change mid-run.
    run_rel = run_dir.relative_to(ROOT).as_posix()
    args = [qemu, "-machine", machine, "-accel", "tcg,thread=single", "-cpu", "qemu64", "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "stdio", "-nic", "none", "-no-reboot",
            "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={run_rel}/OVMF_CODE.fd",
            "-drive", f"if=pflash,unit=1,format=raw,file={run_rel}/OVMF_VARS.fd",
            "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{run_rel}/esp",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
    if m5_case is not None and m5_case != "no-device":
        args += ["-drive", f"if=none,id=m5-test,format=raw,readonly=on,file={run_rel}/test-disk.img",
                 "-device", "virtio-blk-pci,drive=m5-test,id=m5blk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-M5-TEST"]
    share = Path(qemu).parent / "share"
    if share.is_dir():
        args += ["-L", str(share)]
    version = subprocess.check_output([qemu, "--version"], env=env, text=True).strip()
    start = time.monotonic()
    started = datetime.now(timezone.utc).isoformat()
    timed_out = False
    try:
        result = subprocess.run(args, cwd=ROOT, env=env, capture_output=True, timeout=timeout)
        rc, out, err = result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired as exc:
        timed_out, rc, out, err = True, None, exc.stdout or b"", exc.stderr or b""
    stdout, stderr = out.decode("utf-8", "replace"), err.decode("utf-8", "replace")
    (run_dir / "serial.txt").write_bytes(out)
    (run_dir / "stderr.txt").write_bytes(err)
    artifacts = {path.relative_to(run_dir).as_posix(): sha256(path) for path in (run_dir / "esp").rglob("*") if path.is_file()}
    artifacts.update({"OVMF_CODE.fd": sha256(run_dir / "OVMF_CODE.fd"), "OVMF_VARS.template.fd": sha256(run_dir / "OVMF_VARS.template.fd")})
    artifact_binding_passed = all(artifacts.get(name) == expected for name, expected in expected_artifacts.items())
    disk_result = None
    if disk_info is not None:
        from m5_disk import verify_disk
        disk_result = verify_disk(disk_path, disk_info, stdout,
                                  data_required=m5_case not in ("no-device", "reset-timeout"))
        artifacts["test-disk.img"] = sha256(disk_path)
    git = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True)
    report = {"label": label, "started_utc": started, "duration_s": time.monotonic()-start, "timeout_s": timeout, "timed_out": timed_out, "raw_qemu_exit": rc, "expected_exit": expected_exit, "required_markers": list(required_markers), "forbidden_markers": list(forbidden_markers), "passed": is_success(rc, stdout, timed_out, required_markers=required_markers, expected_exit=expected_exit, forbidden_markers=forbidden_markers), "command": args, "qemu_version": version, "profile": f"development-package; {machine}/qemu64/TCG/1CPU/256MiB", "base_commit": git.stdout.strip() if git.returncode == 0 else None, "source_fingerprint_version": 3, "source_sha256": source_hash(), "artifact_sha256": artifacts, "firmware_source": [str(code), str(variables)], "run_directory": str(run_dir), "stdout": stdout, "stderr": stderr}
    report.update(expected_artifact_sha256=expected_artifacts, artifact_binding_passed=artifact_binding_passed)
    report['passed'] = report['passed'] and artifact_binding_passed
    if disk_result is not None:
        report.update(m5_case=m5_case, disk_oracle=disk_result)
        report["passed"] = report["passed"] and disk_result["passed"]
        report["profile"] += "; readonly synthetic virtio-blk00:05.0" if m5_case != "no-device" else "; M5 target absent"
    save_json(run_dir / "result.json", report)
    print(stdout, end="")
    print(stderr, end="")
    print(f"QEMU raw={rc}; expected={expected_exit}; timeout={timed_out}; expectation_pass={report['passed']}; {run_dir}")
    return report


def run_gate_case(phase, group, *, label=None):
    expectation = guest_expectation(phase, group)
    if expectation is None:
        raise ValueError(f"Unknown guest gate case: {phase}/{group}")
    required_markers, expected_exit, forbidden_markers = expectation
    return run_guest(label or group, required_markers=required_markers,
                     expected_exit=expected_exit, forbidden_markers=forbidden_markers)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--smoke", action="store_true")
    parser.add_argument("--milestone", choices=list(MILESTONE_MARKERS), default="m1")
    args = parser.parse_args()
    raise SystemExit(0 if run_guest(required_markers=MILESTONE_MARKERS[args.milestone], m5_case="read" if args.milestone == "m5" else None)["passed"] else 1)
