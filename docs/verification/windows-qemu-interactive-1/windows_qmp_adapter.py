#!/usr/bin/env python3
"""One-shot WSL evidence harness for the real Windows QEMU executable.

Not a launcher option or an OS implementation. No network QMP listener:
the Windows process exposes QMP only through its inherited stdio pipes.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools"))
from common import sha256, source_hash
from run_graphics import QmpPipe
from run_materiaal_preview import BOOT, READY, interactive_groups, matching_reference_frame, verify_ready_transcript
import run_materiaal_probe as probe
from run_system import check_image

SOURCE = "a1998a126a3a1ccc18523925a422912d934c14bab865b3d7df346862a2c251a6"
QEMU = Path("/mnt/c/Program Files/qemu/qemu-system-x86_64.exe")
POWERSHELL = Path("/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe")


def windows(path: Path) -> str:
    path = path.resolve(strict=True)
    if not path.is_relative_to("/mnt/c"):
        raise ValueError("Evidence VM must use its own local Windows C: directory")
    return "C:\\" + str(path.relative_to("/mnt/c")).replace("/", "\\")


def read_serial(state: Path) -> bytes:
    path = state / "serial.txt"
    if not path.exists():
        return b""
    if path.is_symlink() or not path.is_file() or path.stat().st_size > probe.SERIAL_LIMIT:
        raise ValueError("Invalid/oversized Windows guest transcript")
    return path.read_bytes()


def save(path: Path, value) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n")


def run(bundle: Path, reference: Path, output: Path) -> dict:
    bundle = bundle.resolve(strict=True)
    output.mkdir(parents=True, exist_ok=False)
    state = bundle / ("proof-state-" + output.name)
    state.mkdir(exist_ok=False)
    summary_path = ROOT / "docs/verification/full-release-interactive-fixed/interactive/summary.json"
    summary = json.loads(summary_path.read_text())
    if summary["source_sha256"] != SOURCE or source_hash() != SOURCE:
        raise ValueError("Wrong frozen native source")
    manifest = json.loads((bundle / "manifest.json").read_text())
    if manifest["source_sha256"] != SOURCE or manifest["kernel_feature"] != "native-system-materiaal":
        raise ValueError("Wrong Windows bundle source/profile")
    for relative, expected in manifest["files"].items():
        path = bundle / relative
        if (path.is_symlink() or not path.is_file()
                or path.stat().st_size != expected["size_bytes"]
                or sha256(path) != expected["sha256"]):
            raise ValueError("Bundle bytes changed: " + relative)
    if (sha256(bundle / "esp/kernel.elf") != summary["kernel_sha256"]
            or sha256(bundle / "esp/EFI/BOOT/BOOTX64.EFI") != summary["loader_sha256"]):
        raise ValueError("Windows ESP differs from native accepted ESP")
    frames = {name: (reference / f"{name}.ppm").read_bytes() for name in probe.PHASES["normal"]}
    expected_frames = summary["interactive_reference"]["frames_sha256"]
    if any(hashlib.sha256(raw).hexdigest() != expected_frames[name] for name, raw in frames.items()):
        raise ValueError("Native reference frames differ from committed acceptance hashes")
    probe.validate_frames(frames, "normal")
    check = subprocess.run(
        [str(POWERSHELL), "-NoProfile", "-File", windows(bundle / "Start-NieuwOS.ps1"), "-CheckOnly"],
        capture_output=True, timeout=30,
    )
    (output / "checkonly.txt").write_bytes(check.stdout + check.stderr)
    check.check_returncode()
    for source, target in (("OVMF_VARS.template.fd", "OVMF_VARS.fd"), ("volume.empty.img", "volume.img")):
        shutil.copyfile(bundle / source, state / target)
    (output / "volume.before.img").write_bytes((state / "volume.img").read_bytes())
    # Keep the real launcher's literal command contract. Only private state
    # filenames differ, plus the explicitly documented QMP stdio endpoint.
    launcher = (bundle / "Start-NieuwOS.ps1").read_text()
    block = launcher.split("$arguments = @(", 1)[1].split("\n    )", 1)[0]
    arguments = []
    for line in block.splitlines():
        if not line.strip():
            continue
        if re.fullmatch(r'\s*"[^"]*"(?:,\s*"[^"]*")*,?\s*', line) is None:
            raise ValueError("Launcher command changed; review evidence adapter")
        arguments.extend(value.replace("state/", state.name + "/")
                         for value in re.findall(r'"([^"]*)"', line))
    if (arguments[arguments.index("-display") + 1] != "sdl"
            or arguments[arguments.index("-machine") + 1] != "pc-q35-8.2"
            or "rng-builtin,id=rng0" not in arguments
            or arguments[arguments.index("-nic") + 1] != "none"):
        raise ValueError("Wrong Windows display/machine/RNG/isolation profile")
    command = [str(QEMU), *arguments, "-qmp", "stdio"]
    started = time.monotonic()
    deadline = started + 480
    report = {
        "schema_version": 1, "passed": False,
        "scope": "Real Windows QEMU/SDL hardware inputs, six full reference frames and post-quit disk; not manual UX, cold recovery, cleanup or release acceptance",
        "source_commit": summary["source_commit"], "source_checkpoint": manifest["source_commit"],
        "source_sha256": SOURCE, "source_fingerprint_version": 3,
        "kernel_sha256": summary["kernel_sha256"], "loader_sha256": summary["loader_sha256"],
        "started_utc": datetime.now(timezone.utc).isoformat(),
        "platform": "Windows QEMU via WSL interop; SDL; pc-q35-8.2; rng-builtin; TCG; qemu64; 1CPU; 256MiB",
        "qemu_path": windows(QEMU), "qemu_sha256": sha256(QEMU),
        "qemu_version": subprocess.check_output([str(QEMU), "--version"], text=True).strip(),
        "command": command, "cwd": windows(bundle),
        "launcher_sha256": sha256(bundle / "Start-NieuwOS.ps1"),
        "adapter_sha256": sha256(Path(__file__)),
        "bundle_manifest_sha256": sha256(bundle / "manifest.json"),
        "native_summary_sha256": sha256(summary_path),
        "pixel_masks": [], "stages": [], "qmp_trace": [],
    }
    save(output / "result.json", report)
    process = control = None
    try:
        with (output / "stderr.txt").open("xb") as stderr:
            process = subprocess.Popen(
                command, cwd=bundle, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                stderr=stderr, bufsize=0,
            )
            report["interop_pid"] = process.pid
            control = QmpPipe(process, deadline)
            report["qmp_responsive"] = True
            save(output / "result.json", report)
            while True:
                serial = read_serial(state)
                lines = serial.decode("utf-8", "strict").splitlines()
                if any("FAIL" in line or "PANIC" in line for line in lines):
                    raise ValueError("Windows native guest failure")
                if all(marker in lines for marker in (*BOOT, *READY)):
                    verify_ready_transcript(serial)
                    break
                if process.poll() is not None or time.monotonic() >= deadline:
                    raise TimeoutError("Windows guest did not reach readiness")
                time.sleep(0.05)
            captured = {}
            for name, group in interactive_groups(state):
                for request in group:
                    if request["execute"] == "screendump":
                        phase_deadline = min(deadline, time.monotonic() + 60)
                        image = state / f"{name}.ppm"
                        while True:
                            verify_ready_transcript(read_serial(state))
                            if process.poll() is not None or time.monotonic() >= phase_deadline:
                                raise TimeoutError("Windows frame differs from Linux reference: " + name)
                            # Windows QEMU uses its local Windows path, not /mnt/c.
                            control.command("screendump", {"filename": state.name + "/" + image.name})
                            raw = image.read_bytes()
                            (output / image.name).write_bytes(raw)
                            if matching_reference_frame(raw, frames[name]):
                                captured[name] = raw
                                report["stages"].append({
                                    "phase": name, "sha256": hashlib.sha256(raw).hexdigest(),
                                    "full_linux_reference_match": True, "qmp_index": len(control.trace),
                                    "elapsed_s": time.monotonic() - started,
                                })
                                save(output / "result.json", report)
                                break
                            time.sleep(0.08)
                    else:
                        if not report["stages"] or process.poll() is not None or time.monotonic() >= deadline:
                            raise ValueError("Input before verified ready or after exit/deadline")
                        control.command(request["execute"], request["arguments"])
                        time.sleep(0.08)
            report["frames"] = probe.validate_frames(captured, "normal")
            control.command("quit")
            report["qmp_quit_sent"] = True
            report["raw_qemu_exit"] = process.wait(timeout=max(0.1, deadline - time.monotonic()))
        if type(process.returncode) is not int or process.returncode != 0:
            raise ValueError("Windows QEMU did not exit raw 0")
        # No disk decoding while Windows QEMU still has the disk open.
        check_image(state / "volume.img")
        after = (state / "volume.img").read_bytes()
        (output / "volume.after.img").write_bytes(after)
        report["disk"] = probe.disk_oracle(after)
        report["before_sha256"] = sha256(output / "volume.before.img")
        report["after_sha256"] = sha256(output / "volume.after.img")
        verify_ready_transcript(read_serial(state))
        if source_hash() != SOURCE or sha256(QEMU) != report["qemu_sha256"]:
            raise ValueError("Source or Windows QEMU changed")
        if any(sha256(bundle / relative) != item["sha256"] for relative, item in manifest["files"].items()):
            raise ValueError("Bundle changed during Windows run")
        report["passed"] = True
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        if process is not None and process.poll() is None and control is not None:
            try:
                control.command("quit")
                report["failure_qmp_quit_sent"] = True
                process.wait(timeout=10)
            except (OSError, RuntimeError, TimeoutError, subprocess.TimeoutExpired) as error:
                report["failure_quit_error"] = str(error)
        if process is not None and process.poll() is None:
            # Only this spawned process, never name-based process killing.
            process.terminate()
            process.wait(timeout=10)
        if control is not None:
            report["qmp_trace"] = control.trace
            control.close()
        if process is not None:
            report["raw_qemu_exit"] = process.returncode
        (output / "serial.txt").write_bytes(read_serial(state))
        report["serial_sha256"] = sha256(output / "serial.txt")
        report["duration_s"] = time.monotonic() - started
        save(output / "qmp-trace.json", report["qmp_trace"])
        report["qmp_trace_sha256"] = sha256(output / "qmp-trace.json")
        save(output / "result.json", report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, required=True)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = run(args.bundle, args.reference, args.output)
    print(json.dumps({key: value for key, value in result.items() if key != "qmp_trace"}, indent=2))
