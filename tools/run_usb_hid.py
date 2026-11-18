#!/usr/bin/env python3
"""QEMU-only native USB HID transport, directional DMA and unplug verification.

QMP uses private process pipes and one shared deadline. Input is introduced by
QEMU's documented input-send-event command, never by a guest PS/2 injection.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import time

from common import BUILD, save_json, sha256, source_hash
from run_graphics import QmpPipe

FEATURES = {"reports": "native-usb-hid-probe", "readonly": "test-usb-hid-readonly-fault"}
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS")
START = ("USB:BEGIN", "USB:PAGING:PASS", "USB:DOMAIN:PASS", "USB:BUS_MASTER:CONTAINED")
REPORTS = ("USB:ENUM:KEYBOARD", "USB:ENUM:MOUSE", "USB:PROTOCOL:PASS", "USB:READY",
           "USB:KEY_DOWN", "USB:KEY_UP", "USB:MOUSE_DOWN", "USB:MOUSE_UP", "USB:HELD_KEY",
           "USB:UNPLUG_CLEAR:PASS", "USB:STALE_TOKEN:DENIED", "USB:REPORTS:PASS")
END = ("USB:GUARD:UNCHANGED", "USB:QUIESCE:PASS", "USB:PAGES:RETAINED", "USB:PASS")
MAX_OUTPUT = 4 * 1024 * 1024


def markers(case: str) -> tuple[str, ...]:
    if case not in FEATURES:
        raise ValueError("Unknown USB HID fixture case")
    return START + (REPORTS if case == "reports" else ("USB:READONLY:WRITE_DENIED",)) + END


def _key(name: str, down: bool) -> dict:
    return {"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": name}}}


def _button(down: bool) -> dict:
    return {"type": "btn", "data": {"down": down, "button": "left"}}


def _move(axis: str, value: int) -> dict:
    return {"type": "rel", "data": {"axis": axis, "value": value}}


def _output(path: Path) -> bytes:
    if path.stat().st_size > MAX_OUTPUT:
        raise RuntimeError("USB fixture output exceeds its bound")
    return path.read_bytes()


def _wait_marker(process, path: Path, marker: str, deadline: float) -> None:
    while time.monotonic() < deadline:
        output = _output(path).decode("utf-8", errors="replace")
        if "FAIL" in output or "PANIC" in output or "USB-DIAG:REJECT:" in output:
            raise RuntimeError("Guest rejected USB fixture before " + marker)
        if marker in output.splitlines():
            return
        if process.poll() is not None:
            raise RuntimeError("Guest exited before " + marker)
        time.sleep(min(0.01, max(0, deadline - time.monotonic())))
    raise TimeoutError("USB fixture exceeded shared deadline before " + marker)


def controlled_guest(args: list[str], *, cwd, env, timeout, run_dir, case: str) -> dict:
    """Execute the run.py prepared command and preserve its actual arguments.

    There is no fresh timeout for an individual phase or cleanup. All process
    exits are reaped; errors and timeout cannot leave a private emulator alive.
    """
    markers(case)
    if timeout <= 0:
        raise ValueError("USB fixture timeout must be positive")
    run_dir = Path(run_dir)
    stdout_path, stderr_path = run_dir / "usb-stdout.bin", run_dir / "usb-stderr.bin"
    deadline = time.monotonic() + timeout
    process = control = None
    error = None
    timed_out = False
    held_keys: set[str] = set()
    held_button = False
    trace = []
    # The established pipe adapter also runs where AF_UNIX is unavailable. QMP
    # owns stdio; serial bytes are independently captured in this private run.
    serial_path = stdout_path.resolve().relative_to(Path(cwd).resolve()).as_posix()
    if "," in serial_path or args.count("-serial") != 1 or "-qmp" in args:
        raise ValueError("Unexpected prepared serial/QMP command")
    args[args.index("-serial") + 1] = "file:" + serial_path
    args.extend(["-qmp", "stdio"])
    stdout_path.write_bytes(b"")
    with stderr_path.open("wb") as stderr:
        try:
            process = subprocess.Popen(args, cwd=cwd, env=env, stdin=subprocess.PIPE,
                                       stdout=subprocess.PIPE, stderr=stderr)
            control = QmpPipe(process, deadline)
            trace = control.trace
            if case == "reports":
                _wait_marker(process, stdout_path, "USB:READY", deadline)
                # Track before sending: even an unacknowledged command may
                # have delivered input and therefore needs terminal cleanup.
                held_keys.add("a")
                control.command("input-send-event", {"events": [_key("a", True)]})
                _wait_marker(process, stdout_path, "USB:KEY_DOWN", deadline)
                control.command("input-send-event", {"events": [_key("a", False)]})
                held_keys.remove("a")
                _wait_marker(process, stdout_path, "USB:KEY_UP", deadline)
                held_button = True
                control.command("input-send-event", {"events": [_move("x", 7), _move("y", -3), _button(True)]})
                _wait_marker(process, stdout_path, "USB:MOUSE_DOWN", deadline)
                control.command("input-send-event", {"events": [_button(False)]})
                held_button = False
                _wait_marker(process, stdout_path, "USB:MOUSE_UP", deadline)
                held_keys.add("b")
                control.command("input-send-event", {"events": [_key("b", True)]})
                _wait_marker(process, stdout_path, "USB:HELD_KEY", deadline)
                control.command("device_del", {"id": "hid-kbd"})
            process.wait(timeout=max(0, deadline - time.monotonic()))
        except (TimeoutError, subprocess.TimeoutExpired) as exc:
            timed_out = True
            error = str(exc)
        except Exception as exc:
            error = f"{type(exc).__name__}: {exc}"
        finally:
            if control is not None and process is not None and process.poll() is None:
                try:
                    releases = [_key(key, False) for key in sorted(held_keys)]
                    if held_button:
                        releases.append(_button(False))
                    if releases and time.monotonic() < deadline:
                        control.command("input-send-event", {"events": releases})
                except Exception as exc:
                    error = error or f"USB input cleanup failed: {exc}"
            if control is not None:
                control.close()
            if process is not None:
                if process.poll() is None:
                    process.kill()
                process.wait()
    output = []
    for path in (stdout_path, stderr_path):
        with path.open("rb") as stream:
            data = stream.read(MAX_OUTPUT + 1)
        if len(data) > MAX_OUTPUT:
            error = error or "USB fixture output exceeds its bound"
        output.append(data[:MAX_OUTPUT])
    return {"returncode": process.returncode if process is not None else None,
            "stdout": output[0], "stderr": output[1], "timed_out": timed_out,
            "trace": trace, "error": error}


def evaluate(case: str, stdout: str, raw_exit: int | None, timed_out: bool) -> dict:
    # Local imports avoid a run.py -> helper -> run.py initialization cycle.
    from run import is_success
    expected = markers(case)
    if not is_success(raw_exit, stdout, timed_out, required_markers=BOOT + expected + ("XHCI:PASS",),
                      forbidden_markers=("FAIL", "PANIC", "USB-DIAG:REJECT:")):
        raise ValueError("USB native boot/result/exit assertions failed")
    lines = stdout.splitlines()
    if any(lines.count(marker) != 1 for marker in BOOT):
        raise ValueError("Repeated or absent native boot marker")
    controlled = tuple(line for line in lines if line.startswith(("USB:", "XHCI:")))
    if controlled != ("XHCI:BEGIN",) + expected + ("XHCI:PASS",):
        raise ValueError("Wrong case, duplicate, unknown or reordered USB evidence")
    return {"markers": list(expected), "usb_reports": case == "reports",
            "readonly_write_denied": case == "readonly", "input_policy_authority": False}


def run_case(case: str) -> dict:
    from run import run_guest
    markers(case)
    destination = BUILD / f"usb-hid-{case}-report.json"
    report = {"schema_version": 1, "case": case, "passed": False,
              "scope": "QEMU native USB2 HID completions and directional VT-d; no IPC broker, IRQ or physical acceptance"}
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
        report.update(source_sha256=frozen, source_fingerprint_version=3, manifest_sha256=manifest_hash)
        guest = run_guest("usb-hid-" + case, timeout=120, required_markers=BOOT + markers(case),
                          forbidden_markers=("FAIL", "PANIC", "USB-DIAG:REJECT:"), usb_hid_case=case,
                          expected_kernel_sha256=manifest["kernel"]["sha256"],
                          expected_loader_sha256=manifest["loader"]["sha256"])
        report["guest"] = guest
        save_json(destination, report)
        report["observed"] = evaluate(case, guest["stdout"], guest["raw_qemu_exit"], guest["timed_out"])
        if (not guest.get("passed") or not guest.get("artifact_binding_passed")
                or guest.get("source_sha256") != frozen or source_hash() != frozen
                or sha256(manifest_path) != manifest_hash):
            raise ValueError("USB source, manifest or executed artifact changed during verification")
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
    run_case(parser.parse_args().case)
