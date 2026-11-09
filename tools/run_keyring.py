#!/usr/bin/env python3
"""B01 QEMU runner: one native-keyring case per disposable guest. The normal
and stuck/restart cases attach a virtio-rng-pci device fed by the host's urandom;
the timeout case uses a silent local EGD backend and no-device boots without it.
The kernel prints evidence numbers and PASS
markers; this host side checks the marker sequence, raw exit and numbers."""
from __future__ import annotations
import argparse
from contextlib import nullcontext
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import shutil
import socket
import subprocess
import tempfile
import threading
import time
import uuid
from common import ROOT, BUILD, environment, firmware, qemu_machine, save_json, sha256, source_hash

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")
FAILURES = ("PANIC", "FAIL")
FEATURE = {"normal": "native-keyring", "no-device": "test-keyring-no-device",
           "stuck": "test-keyring-stuck", "timeout": "test-keyring-timeout",
           "restart": "test-keyring-restart"}
CASE_MARKER = {case: case.upper().replace("-", "_") for case in FEATURE}
MAX_TIMEOUT = 180
RNG_DEVICE = ["-object", "rng-random,id=rng0,filename=/dev/urandom",
              "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000"]

EXPECT = {
    "normal": {"SERVICE_A_STATE": 1, "SERVICE_B_STATE": 1, "SERVICE_A_REASON": 0,
               "SERVICE_B_REASON": 0, "OFFLINE_ERRORS": 0, "APP_QUOTA_DENIED": 1,
               "APP_RECOVERY_EXPORT_DENIED": 1, "APP_RECOVERY_RESTORE_DENIED": 1,
               "APP_NOT_READY": 0, "SERVICE_B_KEYS": 0},
    "no-device": {"SERVICE_A_STATE": 2, "SERVICE_B_STATE": 2, "SERVICE_A_REASON": 1,
                  "SERVICE_B_REASON": 1, "DELIVERED_BYTES": 0, "APP_GENERATED": 0,
                  "APP_SEALED": 0, "SERVICE_A_KEYS": 0, "SERVICE_B_KEYS": 0},
    "stuck": {"SERVICE_A_STATE": 2, "SERVICE_B_STATE": 2, "SERVICE_A_REASON": 2,
              "SERVICE_B_REASON": 2, "OFFLINE_ERRORS": 0, "APP_GENERATED": 0,
              "APP_SEALED": 0, "SERVICE_A_KEYS": 0, "SERVICE_B_KEYS": 0},
    "timeout": {"SERVICE_A_STATE": 2, "SERVICE_B_STATE": 2, "SERVICE_A_REASON": 1,
                "SERVICE_B_REASON": 1, "DELIVERED_BYTES": 0, "APP_GENERATED": 0,
                "APP_SEALED": 0, "SERVICE_A_KEYS": 0, "SERVICE_B_KEYS": 0,
                "SERVICE_A_ENTROPY_BYTES": 0, "SERVICE_B_ENTROPY_BYTES": 0,
                "TIMEOUTS": 1, "RESET_ACKS": 1, "QUARANTINED": 0, "RESTARTED_BYTES": 0},
}
EXPECT["restart"] = {**EXPECT["normal"], "TIMEOUTS": 0, "RESET_ACKS": 1,
                     "QUARANTINED": 0, "RESTARTED_BYTES": 32}
AT_LEAST = {
    "normal": {"DELIVERED_BYTES": 2048, "SERVICE_A_ENTROPY_BYTES": 1024,
               "SERVICE_B_ENTROPY_BYTES": 1024, "APP_GENERATED": 17, "APP_SEALED": 1,
               "APP_UNSEALED": 1, "APP_TAMPER_DENIED": 4, "APP_CROSS_DENIED": 3,
               "APP_REVOKED_DENIED": 1, "APP_RECOVERY_EXPORT_DENIED": 1,
               "APP_RECOVERY_RESTORE_DENIED": 1, "APP_ENTROPY_DENIED": 1,
               "DENIED": 1},
    "no-device": {"OFFLINE_ERRORS": 2, "REQUESTS": 2, "APP_NOT_READY": 2, "APP_ENTROPY_DENIED": 1},
    "stuck": {"DELIVERED_BYTES": 256, "APP_NOT_READY": 2, "APP_ENTROPY_DENIED": 1},
    "timeout": {"OFFLINE_ERRORS": 1, "REQUESTS": 2, "APP_NOT_READY": 2,
                "APP_ENTROPY_DENIED": 1, "DENIED": 1},
}
AT_LEAST["restart"] = dict(AT_LEAST["normal"])


class SilentEgd:
    """Local EGD backend that withholds entropy after the real kernel manifest.

    Unlike replacing completed bytes, withholding the backend response leaves
    the real virtio used ring untouched and exercises the driver's pending I/O.
    The bounded request transcript is control data, never entropy or key bytes.
    """
    def __init__(self, serial_path: Path | None = None):
        self.serial_path = serial_path
        self.requests: list[int] = []
        self.firmware_requests: list[int] = []
        self.error: str | None = None
        self._stop = threading.Event()
        self.observed = threading.Event()
        self._connection = None

    def __enter__(self):
        self._directory = tempfile.TemporaryDirectory(prefix="nieuw-os-rng-")
        self.path = Path(self._directory.name) / "egd.sock"
        self._listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self._listener.bind(str(self.path))
        self._listener.listen(1)
        self._listener.settimeout(0.1)
        self._thread = threading.Thread(target=self._serve, daemon=True)
        self._thread.start()
        return self

    def _serve(self):
        try:
            while not self._stop.is_set():
                try:
                    self._connection, _ = self._listener.accept()
                    break
                except socket.timeout:
                    continue
            if self._connection is None:
                return
            with self._connection as connection:
                connection.settimeout(0.1)
                pending = b""
                while not self._stop.is_set():
                    try:
                        data = connection.recv(256)
                    except socket.timeout:
                        continue
                    if not data:
                        break
                    pending += data
                    while len(pending) >= 2:
                        command, length = pending[:2]
                        pending = pending[2:]
                        if command != 2 or length == 0 or len(self.requests) + len(self.firmware_requests) >= 1024:
                            raise ValueError("unexpected or unbounded EGD request")
                        # OVMF itself uses virtio-rng before entering the
                        # kernel. QEMU's file chardev writes serial output
                        # synchronously; check it in the request handler,
                        # not in a racing observer thread. The kernel emits
                        # MANIFEST before scheduling its first RNG request.
                        manifest = (self.serial_path is None or
                                    self.serial_path.exists() and
                                    b"KEYRING:MANIFEST:PASS\n" in self.serial_path.read_bytes())
                        if manifest:
                            self.requests.append(length)
                            self.observed.set()
                        else:
                            self.firmware_requests.append(length)
                            connection.sendall(os.urandom(length))
                if pending:
                    raise ValueError("truncated EGD request")
        except Exception as error:
            self.error = f"{type(error).__name__}: {error}"

    def __exit__(self, *_):
        self._stop.set()
        self._thread.join(timeout=2)
        self._listener.close()
        self._directory.cleanup()
        if self._thread.is_alive():
            raise RuntimeError("silent EGD worker did not stop")

    def evidence(self) -> dict:
        return {"backend": "silent-local-egd", "requested_lengths": list(self.requests),
                "firmware_requested_lengths": list(self.firmware_requests),
                "firmware_response_bytes": sum(self.firmware_requests),
                "response_bytes": 0, "error": self.error}


def numbers(stdout: str) -> dict[str, int]:
    found = {}
    for line in stdout.splitlines():
        match = re.fullmatch(r"KEYRING:([A-Z_]+):([0-9]{20})", line)
        if match:
            if match.group(1) in found:
                raise ValueError(f"duplicate evidence {match.group(1)}")
            found[match.group(1)] = int(match.group(2))
    return found


def expected_markers(case: str) -> tuple[str, ...]:
    return BOOT + ("KEYRING:MANIFEST:PASS", "KEYRING:READY", f"KEYRING:CASE:{CASE_MARKER[case]}:PASS",
                   "KEYRING:RESOURCES:PASS", "KEYRING:PASS", "M1:PASS")


def controlled(stdout: str) -> list[str]:
    keep = BOOT + ("KEYRING:MANIFEST:PASS", "KEYRING:READY", "KEYRING:RESOURCES:PASS", "KEYRING:PASS", "M1:PASS")
    return [line for line in stdout.splitlines() if line in keep or line.startswith("KEYRING:CASE:")]


def check_numbers(case: str, found: dict[str, int]) -> None:
    for key, value in EXPECT[case].items():
        if found.get(key) != value:
            raise ValueError(f"{key}: expected {value}, found {found.get(key)}")
    for key, minimum in AT_LEAST[case].items():
        if found.get(key, -1) < minimum:
            raise ValueError(f"{key}: expected at least {minimum}, found {found.get(key)}")
    if "TICKS" not in found or found["TICKS"] >= 6000:
        raise ValueError("missing or exhausted tick budget")


def evaluate(case: str, stdout: str, rc, timed_out: bool) -> dict[str, int]:
    if controlled(stdout) != list(expected_markers(case)):
        raise ValueError(f"marker sequence mismatch: {controlled(stdout)}")
    if any(marker in line for line in stdout.splitlines() for marker in FAILURES):
        raise ValueError("failure marker present")
    if rc != 33 or timed_out:
        raise ValueError(f"raw exit {rc}, timeout {timed_out}")
    found = numbers(stdout)
    check_numbers(case, found)
    return found


def qemu_arguments(case: str, qemu: str, rel: str, rng_socket: Path | None = None) -> list[str]:
    args = [qemu, "-machine", qemu_machine(), "-accel", "tcg,thread=single", "-cpu", "qemu64",
            "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
            "-serial", "stdio", "-nic", "none", "-no-reboot",
            "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
            "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
            "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
    if case == "timeout":
        if rng_socket is None:
            raise ValueError("timeout requires the silent EGD fixture")
        args += ["-chardev", f"socket,id=rng-channel,path={rng_socket}",
                 "-object", "rng-egd,id=rng0,chardev=rng-channel", *RNG_DEVICE[2:]]
        args[args.index("-serial") + 1] = f"file:{rel}/serial.raw"
    elif case != "no-device":
        args += RNG_DEVICE
    return args


def run_keyring(case: str, *, timeout=MAX_TIMEOUT):
    if case not in FEATURE or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("unknown B01 case or invalid timeout")
    frozen = source_hash()
    manifest = json.loads((BUILD / "manifest.json").read_text())
    if manifest.get("kernel_feature") != FEATURE[case] or manifest.get("source_sha256") != frozen:
        raise ValueError(f"build {FEATURE[case]} from the current source first")
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    code, variables = firmware()
    run_dir = BUILD / "run" / f"keyring-{case}-{uuid.uuid4().hex[:10]}"
    run_dir.mkdir(parents=True)
    shutil.copytree(BUILD / "esp", run_dir / "esp")
    shutil.copy2(variables, run_dir / "OVMF_VARS.fd")
    shutil.copy2(code, run_dir / "OVMF_CODE.fd")
    fixture = SilentEgd(run_dir / "serial.raw") if case == "timeout" else None
    with fixture if fixture is not None else nullcontext():
        args = qemu_arguments(case, qemu, run_dir.relative_to(ROOT).as_posix(),
                              fixture.path if fixture else None)
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
    if fixture is not None and (run_dir / "serial.raw").exists():
        stdout = (run_dir / "serial.raw").read_text(encoding="utf-8", errors="replace")
    (run_dir / "serial.txt").write_text(stdout, encoding="utf-8")
    (run_dir / "stderr.txt").write_text(stderr, encoding="utf-8")
    report = dict(schema_version=1, case=case, kernel_feature=FEATURE[case], started_utc=started,
                  duration_s=duration, timeout_s=timeout, timed_out=timed_out, raw_qemu_exit=rc,
                  command=args, qemu_version=subprocess.check_output([qemu, "--version"], text=True).strip(),
                  source_fingerprint_version=3, source_sha256=frozen, run_directory=str(run_dir),
                  kernel_sha256=manifest["kernel"]["sha256"], keyring=manifest.get("keyring"),
                  esp_sha256={p.relative_to(run_dir).as_posix(): sha256(p) for p in (run_dir / "esp").rglob("*") if p.is_file()},
                  stdout=stdout, stderr=stderr, passed=False)
    if fixture is not None:
        report["fault_fixture"] = fixture.evidence()
    try:
        report["evidence"] = evaluate(case, stdout, rc, timed_out)
        if fixture is not None:
            evidence = report["fault_fixture"]
            if evidence["error"] is not None or not evidence["requested_lengths"]:
                raise ValueError("silent EGD did not observe a real pending request")
        report["passed"] = source_hash() == frozen
        if not report["passed"]:
            raise ValueError("source changed during run")
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(run_dir / "result.json", report)
    print(f"B01 {case}: passed={report['passed']} raw={rc} duration={duration:.1f}s {run_dir}")
    if "error" in report:
        print(report["error"])
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=sorted(FEATURE), required=True)
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    args = parser.parse_args()
    raise SystemExit(0 if run_keyring(args.case, timeout=args.timeout)["passed"] else 1)
