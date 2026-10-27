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
from common import ROOT, BUILD, LOCAL, environment, firmware, save_json, sha256, source_hash

MARKERS = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "M1:PASS")
FAIL_MARKERS = ("M1:FAIL", "M2:FAIL", "M3:FAIL", "M4:FAIL")


def is_success(returncode, stdout, timed_out=False, *, required_markers=MARKERS,
               expected_exit=33, forbidden_markers=FAIL_MARKERS):
    lines = iter(stdout.splitlines())
    for marker in required_markers:
        if not any(line == marker for line in lines):
            return False
    return (returncode == expected_exit and not timed_out
            and not any(marker in stdout for marker in forbidden_markers))


def run_guest(label="smoke", *, esp=None, timeout=60, required_markers=MARKERS,
              expected_exit=33, forbidden_markers=FAIL_MARKERS):
    if not 0 < timeout <= 60:
        raise ValueError("guest timeout must be in (0,60]")
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
    shutil.copytree(source, run_dir / "esp")
    shutil.copy2(variables, run_dir / "OVMF_VARS.template.fd")
    shutil.copy2(run_dir / "OVMF_VARS.template.fd", run_dir / "OVMF_VARS.fd")
    shutil.copy2(code, run_dir / "OVMF_CODE.fd")
    # Use paths relative to cwd: drive-letter colons and commas are not passed
    # through QEMU's file-option parser, and the FAT tree cannot change mid-run.
    run_rel = run_dir.relative_to(ROOT).as_posix()
    args = [qemu, "-machine", "pc-q35-10.0", "-accel", "tcg,thread=single", "-cpu", "qemu64", "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "stdio", "-nic", "none", "-no-reboot",
            "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={run_rel}/OVMF_CODE.fd",
            "-drive", f"if=pflash,unit=1,format=raw,file={run_rel}/OVMF_VARS.fd",
            "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{run_rel}/esp",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
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
    git = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True)
    report = {"label": label, "started_utc": started, "duration_s": time.monotonic()-start, "timeout_s": timeout, "timed_out": timed_out, "raw_qemu_exit": rc, "expected_exit": expected_exit, "required_markers": list(required_markers), "forbidden_markers": list(forbidden_markers), "passed": is_success(rc, stdout, timed_out, required_markers=required_markers, expected_exit=expected_exit, forbidden_markers=forbidden_markers), "command": args, "qemu_version": version, "profile": "development-package; pc-q35-10.0/qemu64/TCG/1CPU/256MiB", "base_commit": git.stdout.strip() if git.returncode == 0 else None, "source_sha256": source_hash(), "artifact_sha256": artifacts, "firmware_source": [str(code), str(variables)], "run_directory": str(run_dir), "stdout": stdout, "stderr": stderr}
    save_json(run_dir / "result.json", report)
    print(stdout, end="")
    print(stderr, end="")
    print(f"QEMU raw={rc}; expected={expected_exit}; timeout={timed_out}; expectation_pass={report['passed']}; {run_dir}")
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--smoke", action="store_true")
    args = parser.parse_args()
    raise SystemExit(0 if run_guest()["passed"] else 1)
