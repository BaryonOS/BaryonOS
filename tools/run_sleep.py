#!/usr/bin/env python3
"""Verify the own-kernel ACPI S3 suspend/resume (native-acpi-s3) in the fixed QEMU profile.

Oracles independent of the kernel's own report:
- after each POWER:S3:ENTER marker, QMP `query-status` must report
  `suspended`, and during a fixed dwell the serial output must not grow;
- only the harness ends a sleep, with QMP `system_wakeup`; the ordered QMP
  SUSPEND/WAKEUP events are recorded and must match the cycles;
- exactly one EBS handoff: a resume never re-runs the loader, and `-no-reboot`
  makes any firmware reset during resume end QEMU with the wrong exit.

QMP uses private process pipes and one shared deadline (as run_usb_hid.py).
The scope is QEMU 8.2 q35 + OVMF under TCG; see the kernel module for what the
profile does not cover.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import subprocess
import time

from common import BUILD, save_json, sha256, source_hash, without_acpi_diagnostics
from run_graphics import QmpPipe

FEATURES = {
    "cycles": "native-acpi-s3",
    "corrupt-facs": "test-acpi-s3-corrupt-facs",
    "stale-token": "test-acpi-s3-stale-token",
}
CYCLES = 3
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS",
        "KS-03:PASS", "M2:PASS", "M1:PASS")
PLAN = re.compile(r"POWER:S3:PLAN:PM1A=0x0604:SLP_TYP=1:PM1A_STS=0x0600:FACS_VERSION=0:"
                  r"REAL=(0x[0-9a-f]+):PM32=(0x[0-9a-f]+):POLICY=Edk2Version0Extended")
RESUMED = re.compile(r"POWER:S3:RESUMED:CYCLE=(\d):ENTRY=PM32:PM1_STS=(0x[0-9a-f]{4}):WAK_STS=1:"
                     r"SCI_EN=(RESTORED|KEPT):HWSIG=SAME:FACS=DISARMED:CANARY=PASS:CR3=SAME")
ENTER = "POWER:S3:ENTER:CYCLE={}:SCI_EN=1"
STALE = "POWER:S3:WAKE:REJECTED:TOKEN:ARMED=1:PAGE=0x5a5a0001"
DWELL_S = 0.5
MAX_OUTPUT = 4 * 1024 * 1024


class EventPipe(QmpPipe):
    """QmpPipe that keeps asynchronous QMP events instead of discarding them."""

    def __init__(self, process, deadline):
        self.events = []
        super().__init__(process, deadline)

    def read(self):
        reply = super().read()
        if "event" in reply and isinstance(reply["event"], str):
            self.events.append(reply["event"])
        return reply


def expected_wakeups(case: str) -> int:
    if case not in FEATURES:
        raise ValueError("Unknown S3 fixture case")
    return {"cycles": CYCLES, "corrupt-facs": 0, "stale-token": 1}[case]


def _serial(path: Path) -> str:
    if path.stat().st_size > MAX_OUTPUT:
        raise RuntimeError("S3 fixture output exceeds its bound")
    return path.read_bytes().decode("utf-8", errors="replace")


def _status(control) -> str:
    status = control.command("query-status")
    if not isinstance(status, dict) or not isinstance(status.get("status"), str):
        raise RuntimeError("Malformed QMP query-status reply")
    return status["status"]


def controlled_guest(args: list[str], *, cwd, env, timeout, run_dir, case: str) -> dict:
    """Run the run.py-prepared command; wake each S3 entry exactly once.

    All process exits are reaped; errors and timeout cannot leave a private
    emulator alive. There is no fresh timeout for an individual phase.
    """
    wakeups = expected_wakeups(case)
    if timeout <= 0:
        raise ValueError("S3 fixture timeout must be positive")
    run_dir = Path(run_dir)
    serial_path, stderr_path = run_dir / "s3-serial.bin", run_dir / "s3-stderr.bin"
    deadline = time.monotonic() + timeout
    process = control = None
    error = None
    timed_out = False
    observations = []
    serial_rel = serial_path.resolve().relative_to(Path(cwd).resolve()).as_posix()
    if "," in serial_rel or args.count("-serial") != 1 or "-qmp" in args:
        raise ValueError("Unexpected prepared serial/QMP command")
    args[args.index("-serial") + 1] = "file:" + serial_rel
    args.extend(["-qmp", "stdio"])
    serial_path.write_bytes(b"")
    with stderr_path.open("wb") as stderr:
        try:
            process = subprocess.Popen(args, cwd=cwd, env=env, stdin=subprocess.PIPE,
                                       stdout=subprocess.PIPE, stderr=stderr)
            control = EventPipe(process, deadline)
            cycle = 0
            while process.poll() is None and time.monotonic() < deadline:
                text = _serial(serial_path)
                if "FAIL" in text or "PANIC" in text:
                    break
                if cycle < wakeups and ENTER.format(cycle + 1) in text.splitlines():
                    while _status(control) != "suspended":
                        if process.poll() is not None or time.monotonic() >= deadline:
                            raise RuntimeError(f"Guest never suspended after cycle {cycle + 1}")
                        time.sleep(0.01)
                    before = len(_serial(serial_path))
                    time.sleep(DWELL_S)
                    dwell_status = _status(control)
                    after = len(_serial(serial_path))
                    if dwell_status != "suspended" or after != before:
                        raise RuntimeError(f"Guest left S3 without a wake request (cycle {cycle + 1})")
                    control.command("system_wakeup")
                    cycle += 1
                    observations.append({"cycle": cycle, "suspended": True,
                                         "dwell_s": DWELL_S, "serial_bytes": before})
                    continue
                time.sleep(0.02)
            process.wait(timeout=max(0, deadline - time.monotonic()))
            if cycle != wakeups:
                error = f"Expected {wakeups} wake requests, issued {cycle}"
        except (TimeoutError, subprocess.TimeoutExpired) as exc:
            timed_out = True
            error = str(exc)
        except Exception as exc:
            error = f"{type(exc).__name__}: {exc}"
        finally:
            events = list(control.events) if control is not None else []
            trace = list(control.trace) if control is not None else []
            if control is not None:
                control.close()
            if process is not None:
                if process.poll() is None:
                    process.kill()
                process.wait()
    output = []
    for path in (serial_path, stderr_path):
        with path.open("rb") as stream:
            data = stream.read(MAX_OUTPUT + 1)
        if len(data) > MAX_OUTPUT:
            error = error or "S3 fixture output exceeds its bound"
        output.append(data[:MAX_OUTPUT])
    return {"returncode": process.returncode if process is not None else None,
            "stdout": output[0], "stderr": output[1], "timed_out": timed_out,
            "trace": trace, "events": events, "observations": observations, "error": error}


def evaluate(case: str, stdout: str, raw_exit: int | None, timed_out: bool,
             events: list[str]) -> dict:
    """Checks the exact serial tail, exit, single boot and QMP event order."""
    wakeups = expected_wakeups(case)
    if timed_out or raw_exit != 33:
        raise ValueError(f"QEMU exit {raw_exit} (timed_out={timed_out}), expected 33")
    lines = without_acpi_diagnostics(stdout).splitlines()
    if lines.count(BOOT[0]) != 1:
        raise ValueError("Missing or repeated EBS handoff (reset or reboot)")
    start = lines.index(BOOT[0])
    if any("POWER:" in line for line in lines[:start]):
        raise ValueError("POWER marker before EBS")
    if tuple(lines[start:start + len(BOOT)]) != BOOT:
        raise ValueError(f"Unexpected boot sequence: {lines[start:start + len(BOOT)]}")
    tail = lines[start + len(BOOT):]
    sleep_events = [event for event in events if event in ("SUSPEND", "WAKEUP")]
    if sleep_events != ["SUSPEND", "WAKEUP"] * wakeups:
        raise ValueError(f"Unexpected QMP sleep events: {sleep_events}")
    observed: dict = {"case": case, "wakeups": wakeups}
    if case == "corrupt-facs":
        if tail != ["POWER:S3:REJECTED:InvalidSignature"]:
            raise ValueError(f"Unexpected corrupt-FACS tail: {tail}")
        observed["rejection"] = tail[0]
        return observed
    if not tail or not (plan := PLAN.fullmatch(tail[0])):
        raise ValueError(f"Missing or unexpected S3 plan: {tail[:1]}")
    real, pm32 = (int(value, 16) for value in plan.groups())
    if not (0x1000 <= real < 0xa0000 and real % 0x1000 == 0 and 0 < pm32 < 1 << 32
            and pm32 % 16 == 0):
        raise ValueError("Wake entries outside the real-mode page / 32-bit image bounds")
    observed.update(real_mode_entry=hex(real), protected_entry=hex(pm32))
    if case == "stale-token":
        if tail[1:] != [ENTER.format(1), STALE]:
            raise ValueError(f"Unexpected stale-token tail: {tail[1:]}")
        observed["rejection"] = STALE
        return observed
    body = tail[1:]
    if len(body) != 2 * CYCLES + 1 or body[-1] != "POWER:S3:PASS":
        raise ValueError(f"Unexpected S3 cycle sequence: {body}")
    statuses = []
    for cycle in range(1, CYCLES + 1):
        enter, resumed = body[2 * cycle - 2], body[2 * cycle - 1]
        match = RESUMED.fullmatch(resumed)
        if enter != ENTER.format(cycle) or not match or int(match.group(1)) != cycle:
            raise ValueError(f"Unexpected S3 cycle {cycle}: {enter!r} {resumed!r}")
        status = int(match.group(2), 16)
        if not status & 0x8000:
            raise ValueError("WAK_STS missing from the reported PM1 status")
        statuses.append({"pm1_status": hex(status), "sci_en": match.group(3)})
    observed["cycles"] = statuses
    return observed


def run_case(case: str) -> dict:
    from run import run_guest
    expected_wakeups(case)
    destination = BUILD / f"sleep-{case}-report.json"
    report = {"schema_version": 1, "case": case, "passed": False,
              "scope": "QEMU 8.2 q35/OVMF TCG ACPI S3 by the own kernel: FACS-armed wake, "
                       "QMP system_wakeup, own trampoline and post-wake checks; no AML "
                       "_PTS/_WAK, devices, services, SMP, interrupts or physical hardware"}
    save_json(destination, report)
    try:
        frozen = source_hash()
        manifest_path = BUILD / "manifest.json"
        manifest_hash = sha256(manifest_path)
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        if (manifest.get("kernel_feature") != FEATURES[case]
                or manifest.get("loader_feature") is not None
                or manifest.get("source_fingerprint_version") != 3
                or manifest.get("source_sha256") != frozen):
            raise ValueError(f"Build the current {FEATURES[case]} profile first")
        report.update(source_sha256=frozen, source_fingerprint_version=3,
                      manifest_sha256=manifest_hash)
        guest = run_guest("sleep-" + case, timeout=180, required_markers=BOOT,
                          s3_case=case, expected_kernel_sha256=manifest["kernel"]["sha256"],
                          expected_loader_sha256=manifest["loader"]["sha256"])
        report["guest"] = guest
        save_json(destination, report)
        report["observed"] = evaluate(case, guest["stdout"], guest["raw_qemu_exit"],
                                      guest["timed_out"], guest["s3_control"]["events"])
        if (not guest.get("passed") or not guest.get("artifact_binding_passed")
                or guest.get("source_sha256") != frozen or source_hash() != frozen
                or sha256(manifest_path) != manifest_hash):
            raise ValueError("S3 source, manifest or executed artifact changed during verification")
        report["passed"] = True
    except Exception as exc:
        report["error"] = f"{type(exc).__name__}: {exc}"
        save_json(destination, report)
        raise
    save_json(destination, report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=FEATURES, required=True)
    raise SystemExit(0 if run_case(parser.parse_args().case)["passed"] else 1)
