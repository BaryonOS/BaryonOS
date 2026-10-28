#!/usr/bin/env python3
"""M6a QEMU runner: fresh guarded writable volumes and real host crash cuts."""
from __future__ import annotations
import argparse
import json
import os
from datetime import datetime, timezone
from pathlib import Path
import queue
import shutil
import subprocess
import threading
import time
import uuid
from common import ROOT, BUILD, environment, firmware, qemu_machine, save_json, sha256, source_hash
from build import M6A_CASES
from m6_disk import Volume, create_volume, scan, verify_volume
from run import is_success, FAIL_MARKERS

FAILURES = FAIL_MARKERS + ("M6A:FAIL",)
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")
UNKNOWN = {"write-error", "flush-error", "write-timeout", "flush-timeout", "reset-timeout",
           "bad-descriptor", "used-write", "used-flush"}
NO_DATA = UNKNOWN | {"no-flush", "readonly", "corrupt-all", "no-device"}
WINDOWS_CRASH_EXIT = 0xE06A0001


def kill_for_crash(process):
    """Terminate only our Popen child, retaining a distinguishable raw status."""
    if os.name == "nt":
        import ctypes
        from ctypes import wintypes
        terminate = ctypes.WinDLL("kernel32", use_last_error=True).TerminateProcess
        terminate.argtypes = (wintypes.HANDLE, wintypes.UINT)
        terminate.restype = wintypes.BOOL
        if not terminate(int(process._handle), WINDOWS_CRASH_EXIT):
            error = ctypes.get_last_error()
            # Windows reports ACCESS_DENIED for an already terminating process;
            # GetExitCodeProcess may briefly still report STILL_ACTIVE. Observe
            # its natural exit instead of claiming that our kill succeeded.
            if error == 5:
                try:
                    process.wait(timeout=0.25)
                    return False
                except subprocess.TimeoutExpired:
                    pass
            if process.poll() is not None:
                return False
            raise ctypes.WinError(error)
    else:
        process.kill()
    return True


def case_markers(case):
    name = "NORMAL" if case == "normal" else case.upper().replace("-", "_")
    return BOOT + (f"M6A:CASE:{name}:PASS", "M6A:SURVIVOR_PROGRESS:PASS",
                   "M6A:RESOURCES:PASS", "M6A:PASS", "M1:PASS")


def unique_markers(stdout, expected):
    prefixes = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "M6A:CASE:",
                "M6A:SURVIVOR_PROGRESS", "M6A:RESOURCES", "M6A:PASS", "M1:PASS", "M6A:CUT:")
    controlled = [line for line in stdout.splitlines() if any(line.startswith(p) for p in prefixes)]
    return controlled == list(expected)


def normal_success(rc, stdout, timed_out, case):
    markers = case_markers(case)
    return (unique_markers(stdout, markers) and is_success(rc, stdout, timed_out,
            required_markers=markers, forbidden_markers=FAILURES))


def oracle_policy(case, before):
    cut = int(case[-1]) if case.startswith("vm-cut-") else None
    service_cut = int(case[-1]) if case.startswith("service-cut-") else None
    allowed = {2}
    if case == "full": allowed = {4}
    elif case in ("torn-tail", "no-flush", "readonly", "no-device", "bad-descriptor"): allowed = {1}
    elif case == "corrupt-all": allowed = {None}
    elif case in UNKNOWN: allowed = {1, 2}
    elif case == "recover":
        selected = scan(before); allowed = {selected["generation"] if selected else None}
    elif (point := cut or service_cut) is not None:
        allowed = {1} if point == 1 else {2} if point >= 5 else {1, 2}
    return {"allowed_generations": allowed, "data_required": case not in NO_DATA and cut is None,
            "unchanged": case in ("recover", "torn-tail", "corrupt-all", "no-flush", "readonly", "no-device", "bad-descriptor") or cut == 1 or service_cut == 1,
            "before": before,
            "allow_torn": case == "recover" or case in UNKNOWN or (cut or service_cut or 0) in (2, 3, 4),
            "require_full_prefix": case == "full"}


def previous_stage(volume):
    """Recovery is evidence about an actual previous run, never fresh format."""
    volume.guard()
    pointer = json.loads((volume.directory / "last-stage.json").read_text(encoding="utf-8"))
    previous = Path(pointer["run_directory"]).resolve(strict=True)
    previous.relative_to((BUILD / "run").resolve())
    report = json.loads((previous / "result.json").read_text(encoding="utf-8"))
    image = volume.path.resolve(strict=True)
    if (report.get("passed") is not True or report.get("case") == "recover"
            or Path(report["volume_path"]).resolve(strict=True) != image
            or report["disk_oracle"]["identity"] != volume.identity
            or report["artifact_sha256"]["test-disk.after.img"] != sha256(image)
            or (previous / "test-disk.after.img").read_bytes() != image.read_bytes()):
        raise ValueError("Recovery must use the exact successful previous stage's image")
    return previous


def execute(args, env, *, cut=None, timeout=60, capture_directory=None):
    """Consume both real pipes, killing only the live child after an exact cut.

    Unit tests can use a bounded child process; that is host-harness evidence,
    never a claim that those strings came from a guest.
    """
    if not 0 < timeout <= 60 or cut is not None and not 1 <= cut <= 6:
        raise ValueError("Invalid bounded M6a run")
    process = subprocess.Popen(args, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout, stderr, events = [], [], queue.Queue()

    def reader(pipe, destination, notify):
        try:
            for line in iter(pipe.readline, b""):
                destination.append(line)
                if notify:
                    events.put(line)
        finally:
            pipe.close()

    threads = [threading.Thread(target=reader, args=(process.stdout, stdout, True), daemon=True),
               threading.Thread(target=reader, args=(process.stderr, stderr, False), daemon=True)]
    for thread in threads: thread.start()
    deadline = time.monotonic() + timeout
    timed_out = killed = running_before_kill = False
    witnessed = []
    while process.poll() is None:
        left = deadline - time.monotonic()
        if left <= 0:
            timed_out = True
            process.kill()
            break
        try:
            line = events.get(timeout=min(left, 0.05)).decode("utf-8", "replace").rstrip("\r\n")
        except queue.Empty:
            continue
        if line.startswith("M6A:CUT:"):
            witnessed.append(line)
        if cut is not None and line == f"M6A:CUT:{cut}":
            # A marker followed by a clean/self exit is not a crash. Require a
            # bounded live hold before terminating the actual child ourselves.
            try:
                process.wait(timeout=0.05)
            except subprocess.TimeoutExpired:
                running_before_kill = process.poll() is None
            if running_before_kill:
                try:
                    killed = kill_for_crash(process)
                except Exception:
                    if process.poll() is None:
                        process.kill()
                    process.wait(timeout=10)
                    for thread in threads: thread.join(timeout=10)
                    raise
            break
    process.wait(timeout=10)
    for thread in threads: thread.join(timeout=10)
    if any(thread.is_alive() for thread in threads):
        raise RuntimeError("M6a QEMU output pipe did not close")
    out_bytes, err_bytes = b"".join(stdout), b"".join(stderr)
    if capture_directory is not None:
        capture_directory.resolve(strict=True).relative_to((BUILD / "run").resolve())
        (capture_directory / "serial.txt").write_bytes(out_bytes)
        (capture_directory / "stderr.txt").write_bytes(err_bytes)
    out = out_bytes.decode("utf-8", "replace")
    err = err_bytes.decode("utf-8", "replace")
    # Count all lines, including bytes drained after process exit. A process that
    # prints a marker and exits itself is never an externally crashed QEMU.
    all_cuts = [line for line in out.splitlines() if line.startswith("M6A:CUT:")]
    crash = {"requested_cut": cut, "cut_lines": all_cuts, "kill_requested": killed,
             "process_id": process.pid,
             "was_running_before_kill": running_before_kill,
             "required_live_hold_s": 0.05,
             "termination_code_expected": WINDOWS_CRASH_EXIT if os.name == "nt" else -9,
             "real_process_returncode": process.returncode}
    if cut is not None:
        crash["passed"] = (not timed_out and killed and running_before_kill
            and process.returncode == crash["termination_code_expected"]
            and all_cuts == [f"M6A:CUT:{cut}"]
            and unique_markers(out, BOOT + (f"M6A:CUT:{cut}",))
            and is_success(process.returncode, out, required_markers=BOOT + (f"M6A:CUT:{cut}",),
                           expected_exit=process.returncode,
                           forbidden_markers=FAILURES + ("M6A:PASS", "M1:PASS")))
    return process.returncode, out, err, timed_out, crash


def run_m6(case="normal", *, volume: Volume | None = None, label=None, timeout=60):
    if not BUILD.resolve().is_relative_to(ROOT.resolve()):
        raise ValueError("NOS_BUILD_DIR must be inside the checkout for QEMU runs")
    machine = qemu_machine()
    if case not in ("normal",) + M6A_CASES:
        raise ValueError("Unknown M6a case")
    if volume is None:
        if case == "recover":
            raise ValueError("Recovery requires an earlier actual run and its Volume")
        volume = create_volume(case, variant=case if case in ("torn-tail", "corrupt-all") else "normal",
                               scratch=case == "device-rights")
    if not isinstance(volume, Volume):
        raise ValueError("Only this harness's synthetic Volume objects are accepted")
    image = volume.guard()
    previous = previous_stage(volume) if case == "recover" else None
    if case != "recover" and (volume.directory / "last-stage.json").exists():
        raise ValueError("Refuse to replay a mutator stage on an already used volume")
    before = image.read_bytes()
    if label is None: label = case
    import re
    if not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,60}", label):
        raise ValueError("Invalid M6a run label")
    run_dir = BUILD / "run" / f"m6-{label}-{uuid.uuid4().hex[:10]}"
    run_dir.mkdir(parents=True)
    (run_dir / "test-disk.before.img").write_bytes(before)
    source = (BUILD / "esp").resolve(strict=True)
    source.relative_to(BUILD.resolve())
    shutil.copytree(source, run_dir / "esp")
    manifest = json.loads((BUILD / "manifest.json").read_text(encoding="utf-8"))
    expected_feature = "milestone-m6a" if case == "normal" else f"test-m6a-{case}"
    if (manifest.get("kernel_feature") != expected_feature or manifest.get("loader_feature") is not None
            or manifest["source_sha256"] != source_hash()
            or manifest["kernel"]["sha256"] != sha256(run_dir / "esp/kernel.elf")
            or manifest["loader"]["sha256"] != sha256(run_dir / "esp/EFI/BOOT/BOOTX64.EFI")):
        raise ValueError("Build the exact M6a case on the current source before running it")
    save_json(run_dir / "build-manifest.json", manifest)
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu: raise RuntimeError("Missing QEMU")
    code, variables = firmware()
    for src, name in [(code, "OVMF_CODE.fd"), (variables, "OVMF_VARS.template.fd"), (variables, "OVMF_VARS.fd")]:
        shutil.copy2(src, run_dir / name)
    rel = run_dir.relative_to(ROOT).as_posix()
    image_rel = image.relative_to(ROOT).as_posix()
    args = [qemu, "-machine", machine, "-accel", "tcg,thread=single", "-cpu", "qemu64",
            "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
            "-serial", "stdio", "-nic", "none", "-no-reboot",
            "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
            "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
            "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
    if case != "no-device":
        readonly = "on" if case == "readonly" else "off"
        args += ["-drive", f"if=none,id=m6-test,format=raw,cache=writeback,readonly={readonly},file={image_rel}",
                 "-device", "virtio-blk-pci,drive=m6-test,id=m6blk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-M6A-TEST"]
    if (Path(qemu).parent / "share").is_dir(): args += ["-L", str(Path(qemu).parent / "share")]
    version = subprocess.check_output([qemu, "--version"], env=env, text=True).strip()
    cut = int(case[-1]) if case.startswith("vm-cut-") else None
    started = datetime.now(timezone.utc).isoformat(); clock = time.monotonic()
    rc, out, err, timed_out, crash = execute(args, env, cut=cut, timeout=timeout, capture_directory=run_dir)
    after = image.read_bytes()
    (run_dir / "test-disk.after.img").write_bytes(after)
    oracle = verify_volume(volume, out, **oracle_policy(case, before))
    markers = BOOT + (f"M6A:CUT:{cut}",) if cut else case_markers(case)
    guest_pass = crash["passed"] if cut else normal_success(rc, out, timed_out, case)
    unknown_lines = [line for line in out.splitlines() if line.startswith("M6A:OUTCOME")]
    unknown_pass = unknown_lines == (["M6A:OUTCOME:UNKNOWN"] if case in UNKNOWN else [])
    artifacts = {p.relative_to(run_dir).as_posix(): sha256(p) for p in (run_dir / "esp").rglob("*") if p.is_file()}
    artifacts.update({name: sha256(run_dir/name) for name in ("OVMF_CODE.fd", "OVMF_VARS.template.fd",
                                                            "test-disk.before.img", "test-disk.after.img", "build-manifest.json")})
    report = {"label": label, "case": case, "started_utc": started, "duration_s": time.monotonic()-clock,
              "timeout_s": timeout, "timed_out": timed_out, "raw_qemu_exit": rc,
              "expected_exit": "real host termination after exact cut" if cut else 33,
              "required_markers": list(markers),
              "forbidden_markers": list(FAILURES + (("M6A:PASS", "M1:PASS") if cut else ())),
              "passed": bool(guest_pass and oracle["passed"] and unknown_pass),
              "command": args, "qemu_version": version, "profile": f"M6a {machine}/qemu64/TCG/1CPU/256MiB; synthetic writeback raw volume",
              "base_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
              "source_fingerprint_version": 3, "source_sha256": source_hash(), "artifact_sha256": artifacts,
              "volume_path": str(image), "disk_oracle": oracle, "host_crash": crash,
              "serial_sha256": sha256(run_dir / "serial.txt"), "stderr_sha256": sha256(run_dir / "stderr.txt"),
              "previous_stage": str(previous) if previous else None,
              "unknown_marker_passed": unknown_pass, "run_directory": str(run_dir), "stdout": out, "stderr": err}
    save_json(run_dir / "result.json", report)
    save_json(volume.directory / "last-stage.json", {"run_directory": str(run_dir)})
    print(out, end=""); print(err, end="")
    print(f"M6a raw={rc}; timeout={timed_out}; passed={report['passed']}; {run_dir}")
    return report, volume


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=("normal",)+tuple(c for c in M6A_CASES if c != "recover"), default="normal")
    parser.add_argument("--smoke", action="store_true")
    args = parser.parse_args()
    raise SystemExit(0 if run_m6(args.case)[0]["passed"] else 1)
