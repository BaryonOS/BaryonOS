#!/usr/bin/env python3
"""M6b QEMU runner: fresh guarded writable volumes and real host crash cuts."""
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
from build import M6B_CASES, M6B_FAULT_CASES, M6B_TIMEOUT_CASES
from m6b_disk import Volume, create_volume, verify_volume
from run import is_success, FAIL_MARKERS

FAILURES = FAIL_MARKERS + ("M6A:FAIL", "M6B:FAIL")
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")
CUTS = {"vm-cut-before-ack": 1, "vm-cut-after-flush": 2}
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
                    process.wait(timeout=10)
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
    fault = ()
    if case in M6B_TIMEOUT_CASES:
        operation = 'FLUSH' if case.startswith('flush') else 'WRITE'
        fault = (f'M6B:INJECT:{operation}_NOTIFY_SUPPRESSED',)
        if case.endswith('poll-limit'):
            fault += ('M6B:CANCEL:BLOCK_CAP_CLOSED', 'M6B:POLL_LIMIT:CANCELLED')
        else:
            fault += (f'M6B:DEADLINE:{operation}_TIMEOUT',)
        fault += ('M6B:RESET:REQUESTED',)
        fault += (('M6B:INJECT:RESET_ACK_MASKED', 'M6B:RESET:QUARANTINED', 'M6B:POOL_PINNED:PASS')
                  if case == 'reset-timeout' else
                  ('M6B:RESET:ACKNOWLEDGED', 'M6B:POOL_REUSE_AFTER_ACK:PASS'))
    elif case in M6B_FAULT_CASES:
        fault = (f"M6B:INJECT:{name.removesuffix('_ERROR')}_IOERR_SNAPSHOT",)
    if case in M6B_FAULT_CASES:
        fault += ("M6B:OUTCOME:UNKNOWN", "M6B:NO_REPLAY:PASS", "M6B:NO_STALE_READ:PASS")
    return BOOT + fault + (f"M6B:CASE:{name}:PASS", "M6B:SURVIVOR_PROGRESS:PASS",
                   "M6B:RESOURCES:PASS", "M6B:PASS", "M1:PASS")


def unique_markers(stdout, expected):
    prefixes = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "M6B:CASE:",
                "M6B:SURVIVOR_PROGRESS", "M6B:RESOURCES", "M6B:PASS", "M1:PASS", "M6B:CUT:", "M6A:",
                "M6B:INJECT", "M6B:OUTCOME", "M6B:NO_REPLAY", "M6B:NO_STALE_READ",
                "M6B:DEADLINE", "M6B:CANCEL", "M6B:POLL_LIMIT", "M6B:RESET:", "M6B:POOL_")
    controlled = [line for line in stdout.splitlines()
                  if any(line.startswith(p) for p in prefixes)
                  or line.startswith("M6B:") and ":PASS" in line]
    return controlled == list(expected)


def normal_success(rc, stdout, timed_out, case):
    if case not in ("normal", "recover", "errors", "full", "corrupt") + M6B_FAULT_CASES:
        return False
    markers = case_markers(case)
    return (unique_markers(stdout, markers) and is_success(rc, stdout, timed_out,
            required_markers=markers, forbidden_markers=FAILURES))


def previous_stage(volume, *, expected_source=None):
    """Bind cold recovery to a successful real stage and its exact current image."""
    volume.guard()
    frozen = source_hash() if expected_source is None else expected_source
    pointer = json.loads((volume.directory / "last-stage.json").read_text(encoding="utf-8"))
    previous = Path(pointer["run_directory"]).resolve(strict=True)
    previous.relative_to((BUILD / "run").resolve())
    report = json.loads((previous / "result.json").read_text(encoding="utf-8"))
    image = volume.path.resolve(strict=True)
    saved_after = previous / "test-disk.after.img"
    actual_hash = sha256(image)
    if (report.get("passed") is not True or report.get("case") not in ("normal",) + M6B_CASES
            or report.get("case") == "recover"
            or report.get("source_fingerprint_version") != 3
            or report.get("source_sha256") != frozen
            or Path(report["run_directory"]).resolve(strict=True) != previous
            or Path(report["volume_path"]).resolve(strict=True) != image
            or report["disk_oracle"]["identity"] != volume.identity
            or report["artifact_sha256"]["test-disk.after.img"] != actual_hash
            or sha256(saved_after) != actual_hash
            or saved_after.read_bytes() != image.read_bytes()):
        raise ValueError("Recovery requires the exact successful previous image on the same source")
    return previous


def execute(args, env, *, cut=None, timeout=60, capture_directory=None):
    """Consume both real pipes, killing only the live child after an exact cut.

    Unit tests can use a bounded child process; that is host-harness evidence,
    never a claim that those strings came from a guest.
    """
    if not 0 < timeout <= 60 or cut is not None and cut not in (1, 2):
        raise ValueError("Invalid bounded M6b run")
    if capture_directory is not None:
        capture_directory.resolve(strict=True).relative_to((BUILD / "run").resolve())
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
        if cut is not None and line == f"M6B:CUT:{cut}":
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
                    if capture_directory is not None:
                        (capture_directory / "serial.txt").write_bytes(b"".join(stdout))
                        (capture_directory / "stderr.txt").write_bytes(b"".join(stderr))
                    raise
            break
    process.wait(timeout=10)
    for thread in threads: thread.join(timeout=10)
    if any(thread.is_alive() for thread in threads):
        raise RuntimeError("M6b QEMU output pipe did not close")
    out_bytes, err_bytes = b"".join(stdout), b"".join(stderr)
    if capture_directory is not None:
        capture_directory.resolve(strict=True).relative_to((BUILD / "run").resolve())
        (capture_directory / "serial.txt").write_bytes(out_bytes)
        (capture_directory / "stderr.txt").write_bytes(err_bytes)
    out = out_bytes.decode("utf-8", "replace")
    err = err_bytes.decode("utf-8", "replace")
    # Count all lines, including bytes drained after process exit. A process that
    # prints a marker and exits itself is never an externally crashed QEMU.
    all_cuts = [line for line in out.splitlines() if line.startswith("M6B:CUT:")]
    crash = {"requested_cut": cut, "cut_lines": all_cuts, "kill_requested": killed,
             "process_id": process.pid,
             "was_running_before_kill": running_before_kill,
             "required_live_hold_s": 0.05,
             "termination_code_expected": WINDOWS_CRASH_EXIT if os.name == "nt" else -9,
             "real_process_returncode": process.returncode}
    if cut is not None:
        crash["passed"] = (not timed_out and killed and running_before_kill
            and process.returncode == crash["termination_code_expected"]
            and all_cuts == [f"M6B:CUT:{cut}"]
            and unique_markers(out, BOOT + (f"M6B:CUT:{cut}",))
            and is_success(process.returncode, out, required_markers=BOOT + (f"M6B:CUT:{cut}",),
                           expected_exit=process.returncode,
                           forbidden_markers=FAILURES + ("M6B:PASS", "M1:PASS")))
    return process.returncode, out, err, timed_out, crash


def run_m6b(case="normal", *, volume: Volume | None = None, label=None, timeout=60):
    if not BUILD.resolve().is_relative_to(ROOT.resolve()):
        raise ValueError("NOS_BUILD_DIR must be inside the checkout for QEMU runs")
    if case not in ("normal",) + M6B_CASES:
        raise ValueError("Unknown M6b case")
    if not 0 < timeout <= 60:
        raise ValueError("Invalid bounded M6b run")
    if label is None:
        label = case
    import re
    if not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,60}", label):
        raise ValueError("Invalid M6b run label")
    if volume is None:
        if case == "recover":
            raise ValueError("Recovery requires an earlier actual run and its Volume")
        volume = create_volume(label, variant="corrupt" if case == "corrupt" else "normal")
    if not isinstance(volume, Volume):
        raise ValueError("Only this harness's synthetic Volume objects are accepted")
    image = volume.guard()
    machine = qemu_machine()
    frozen = source_hash()
    previous = previous_stage(volume, expected_source=frozen) if case == "recover" else None
    if case != "recover" and (volume.directory / "last-stage.json").exists():
        raise ValueError("Refuse to replay a mutator stage on an already used volume")
    before = image.read_bytes()
    run_dir = BUILD / "run" / f"m6b-{label}-{uuid.uuid4().hex[:10]}"
    run_dir.mkdir(parents=True)
    (run_dir / "test-disk.before.img").write_bytes(before)
    shutil.copy2(volume.directory / "volume.json", run_dir / "volume-manifest.json")
    source = (BUILD / "esp").resolve(strict=True)
    source.relative_to(BUILD.resolve())
    shutil.copytree(source, run_dir / "esp")
    manifest = json.loads((BUILD / "manifest.json").read_text(encoding="utf-8"))
    expected_feature = "milestone-m6b" if case == "normal" else f"test-m6b-{case}"
    if (manifest.get("kernel_feature") != expected_feature
            or manifest.get("loader_feature") is not None
            or manifest.get("source_fingerprint_version") != 3
            or manifest.get("source_sha256") != frozen
            or manifest["kernel"]["sha256"] != sha256(run_dir / "esp/kernel.elf")
            or manifest["loader"]["sha256"] != sha256(run_dir / "esp/EFI/BOOT/BOOTX64.EFI")):
        raise ValueError("Build the exact M6b case on the current source before running it")
    save_json(run_dir / "build-manifest.json", manifest)
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("Missing QEMU")
    code, variables = firmware()
    for src, name in [(code, "OVMF_CODE.fd"), (variables, "OVMF_VARS.template.fd"),
                      (variables, "OVMF_VARS.fd")]:
        shutil.copy2(src, run_dir / name)
    rel = run_dir.relative_to(ROOT).as_posix()
    image_rel = image.relative_to(ROOT).as_posix()
    args = [qemu, "-machine", machine, "-accel", "tcg,thread=single", "-cpu", "qemu64",
            "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
            "-serial", "stdio", "-nic", "none", "-no-reboot",
            "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
            "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
            "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
            "-drive", f"if=none,id=m6b-test,format=raw,cache=writeback,readonly=off,file={image_rel}",
            "-device", "virtio-blk-pci,drive=m6b-test,id=m6bblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-M6B-TEST"]
    if (Path(qemu).parent / "share").is_dir():
        args += ["-L", str(Path(qemu).parent / "share")]
    version = subprocess.check_output([qemu, "--version"], env=env, text=True, timeout=10).strip()
    qemu_hash = sha256(Path(qemu))
    base_commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT,
                                          env=env, text=True, timeout=10).strip()
    cut = CUTS.get(case)
    started = datetime.now(timezone.utc).isoformat()
    clock = time.monotonic()
    execution_error = None
    try:
        rc, out, err, timed_out, crash = execute(args, env, cut=cut, timeout=timeout,
                                                capture_directory=run_dir)
    except Exception as error:
        # No synthetic exit status or guest output. An exceptional attempt is
        # retained as failed evidence, including any raw bytes already captured.
        execution_error = f"{type(error).__name__}: {error}"
        rc, timed_out = None, None
        captured = []
        for name in ("serial.txt", "stderr.txt"):
            path = run_dir / name
            if not path.exists():
                path.write_bytes(b"")
            captured.append(path.read_bytes().decode("utf-8", "replace"))
        out, err = captured
        crash = {"requested_cut": cut, "passed": False, "execution_error": execution_error}
    after = image.read_bytes()
    (run_dir / "test-disk.after.img").write_bytes(after)
    oracle_error = None
    try:
        oracle = verify_volume(volume, out, case=case, before=before)
    except Exception as error:
        import hashlib
        oracle_error = f"{type(error).__name__}: {error}"
        oracle = {"passed": False, "checks": {"oracle_completed": False},
                  "identity": volume.identity, "seed": volume.seed, "variant": volume.variant,
                  "sha256_initial": hashlib.sha256(volume.initial).hexdigest(),
                  "sha256_after": sha256(image), "state": None, "observed": None,
                  "error": oracle_error}
    observed_source = source_hash()
    source_unchanged = observed_source == frozen
    markers = BOOT + (f"M6B:CUT:{cut}",) if cut is not None else case_markers(case)
    guest_pass = crash.get("passed", False) if cut is not None else normal_success(rc, out, timed_out, case)
    outcome_lines = [line for line in out.splitlines() if line.startswith("M6B:OUTCOME")]
    outcome_pass = outcome_lines == (["M6B:OUTCOME:UNKNOWN"] if case in M6B_FAULT_CASES else [])
    artifacts = {p.relative_to(run_dir).as_posix(): sha256(p)
                 for p in (run_dir / "esp").rglob("*") if p.is_file()}
    artifacts.update({name: sha256(run_dir / name) for name in (
        "OVMF_CODE.fd", "OVMF_VARS.template.fd", "OVMF_VARS.fd", "test-disk.before.img",
        "test-disk.after.img", "build-manifest.json", "volume-manifest.json")})
    report = {
        "label": label, "case": case, "started_utc": started, "duration_s": time.monotonic() - clock,
        "timeout_s": timeout, "timed_out": timed_out, "raw_qemu_exit": rc,
        "expected_exit": "real host termination after exact cut" if cut is not None else 33,
        "required_markers": list(markers),
        "forbidden_markers": list(FAILURES + (("M6B:PASS", "M1:PASS") if cut is not None else ())),
        "passed": bool(execution_error is None and oracle_error is None and source_unchanged
                       and guest_pass and oracle["passed"] and outcome_pass),
        "command": args, "qemu_version": version, "qemu_sha256": qemu_hash,
        "profile": f"M6b {machine}/qemu64/TCG/1CPU/256MiB; synthetic writeback raw volume",
        "firmware_source": [str(code), str(variables)], "base_commit": base_commit,
        "source_fingerprint_version": 3, "source_sha256": frozen,
        "observed_source_sha256": observed_source, "source_unchanged": source_unchanged,
        "artifact_sha256": artifacts, "volume_path": str(image), "disk_oracle": oracle,
        "host_crash": crash, "serial_sha256": sha256(run_dir / "serial.txt"),
        "stderr_sha256": sha256(run_dir / "stderr.txt"),
        "previous_stage": str(previous) if previous else None, "outcome_marker_passed": outcome_pass,
        "execution_error": execution_error, "oracle_error": oracle_error,
        "run_directory": str(run_dir), "stdout": out, "stderr": err,
    }
    save_json(run_dir / "result.json", report)
    save_json(volume.directory / "last-stage.json", {"run_directory": str(run_dir)})
    print(out, end="")
    print(err, end="")
    if execution_error or oracle_error:
        print(f"M6b execution/oracle error: {execution_error or oracle_error}")
    print(f"M6b raw={rc}; timeout={timed_out}; passed={report['passed']}; {run_dir}")
    return report, volume


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=("normal",) + tuple(c for c in M6B_CASES if c != "recover"),
                        default="normal")
    parser.add_argument("--smoke", action="store_true")
    args = parser.parse_args()
    raise SystemExit(0 if run_m6b(args.case)[0]["passed"] else 1)

