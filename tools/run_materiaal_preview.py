#!/usr/bin/env python3
"""Boot non-probe Materiaal, optionally checking real input against native frames."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import time
import uuid

from common import BUILD, ROOT, environment, firmware, qemu_machine, sha256, source_hash
from package_windows_qemu import APP_NAMES, load_json, validate_build
from run_graphics import QmpPipe, ppm
from run_system import check_image
from run_fpu import private_path, publish, publish_json
import run_materiaal_probe as probe
import volume_disk

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")
READY = (
    "MATERIAAL:MANIFEST:PASS",
    "MATERIAAL:SOURCE:READY",
    "MATERIAAL:POLICY:READY",
    "MATERIAAL:COMPOSITOR:READY",
    "MATERIAAL:FOCUS:READY",
)
WIDTH, HEIGHT = 800, 600
SERIAL_LIMIT = 256 * 1024


def verify_ready_transcript(raw: bytes) -> list[str]:
    if not isinstance(raw, bytes) or len(raw) > SERIAL_LIMIT:
        raise ValueError("Oversized or non-byte native serial transcript")
    lines = raw.decode("utf-8", "strict").splitlines()
    if any("FAIL" in line or "PANIC" in line for line in lines):
        raise ValueError("Native guest reported a failure")
    observed = [line for line in lines if line in (*BOOT, *READY)]
    if (observed[:len(BOOT)] != list(BOOT)
            or observed[len(BOOT):len(BOOT) + 1] != [READY[0]]
            or len(observed) != len(BOOT) + len(READY)
            or any(observed.count(marker) != 1 for marker in READY)):
        raise ValueError(f"Native readiness sequence differs: {observed}")
    return observed


def verify_frame(raw: bytes, expected_size: tuple[int, int] = (WIDTH, HEIGHT)) -> dict:
    width, height, pixels = ppm(raw)
    if (width, height) != expected_size:
        raise ValueError(f"Wrong native framebuffer size: {width}x{height}")
    colors = {pixels[index:index + 3] for index in range(0, len(pixels), 3)}
    if len(colors) < 8:
        raise ValueError("Framebuffer is blank or lacks a visible native scene")
    return {"width": width, "height": height, "distinct_colors": len(colors)}


def read_serial(path: Path) -> bytes:
    if not path.exists():
        return b""
    details = path.lstat()
    if not stat.S_ISREG(details.st_mode) or details.st_size > SERIAL_LIMIT:
        raise ValueError("Native serial transcript is missing or oversized")
    return path.read_bytes()


def reference_binding(report: dict, reference_manifest: dict, manifest: dict,
                      frozen: str, machine: str) -> None:
    if (report.get("passed") is not True or report.get("case") != "normal"
            or report.get("source_sha256") != frozen
            or reference_manifest.get("source_sha256") != frozen
            or manifest.get("source_sha256") != frozen
            or type(report.get("source_fingerprint_version")) is not int
            or report["source_fingerprint_version"] != 3
            or report.get("machine") != machine
            or reference_manifest.get("kernel_feature") != probe.FEATURES["normal"]
            or manifest.get("kernel_feature") != "native-system-materiaal"):
        raise ValueError("Reference is not a same-source normal native probe")
    current_apps = manifest["materiaal"]["apps"]
    reference_apps = reference_manifest["materiaal"]["apps"]
    if (len(current_apps) != len(APP_NAMES) or len(reference_apps) != len(APP_NAMES) + 1
            or any(current.get("name") != name or reference != current
                   for name, current, reference in
                   zip(APP_NAMES, current_apps, reference_apps[:len(APP_NAMES)], strict=True))
            or reference_apps[-1].get("name") != "desktop-controller"):
        raise ValueError("Reference differs from the seven interactive app identities")
    if manifest["loader"]["sha256"] != reference_manifest["loader"]["sha256"]:
        raise ValueError("Reference loader differs from the interactive loader")


def load_reference(directory: Path, manifest: dict, frozen: str, machine: str,
                   firmware_hashes: dict, qemu_hash: str) -> dict:
    directory = private_path(directory).resolve(strict=True)
    report_path = private_path(directory / "result.json", regular=True)
    report_hash = sha256(report_path)
    report = load_json(report_path, "native reference report", 16 * 1024 * 1024)
    reference_manifest = load_json(
        private_path(directory / "inputs/manifest.json", regular=True), "native reference manifest"
    )
    reference_binding(report, reference_manifest, manifest, frozen, machine)
    # Recompute the original eight-process proof before borrowing its images.
    # This never interprets the interactive guest as a diagnostic/controller run.
    probe.replay(report, directory, frozen)
    expected_firmware = {
        "OVMF_CODE.fd": firmware_hashes["ovmf_code_sha256"],
        "OVMF_VARS.template.fd": firmware_hashes["ovmf_vars_template_sha256"],
        "VGA.rom": firmware_hashes["seabios_vga_sha256"],
    }
    if (any(report["frozen_artifacts"].get(name) != digest
            for name, digest in expected_firmware.items())
            or report.get("qemu_entry_sha256") != qemu_hash):
        raise ValueError("Reference firmware or QEMU identity differs")
    frames = {name: probe.frame_bytes(directory, name) for name in probe.PHASES["normal"]}
    probe.validate_frames(frames, "normal")
    if sha256(report_path) != report_hash:
        raise ValueError("Reference report changed during validation")
    return {
        "directory": directory,
        "report_sha256": report_hash,
        "frames": frames,
        "source_sha256": frozen,
        "apps": manifest["materiaal"]["apps"],
        "qemu_implementation_sha256": report["qemu_implementation_sha256"],
    }


def interactive_groups(directory: Path) -> list:
    groups = probe.qmp_groups(directory, "normal")
    expected_close = [{
        "execute": "send-key",
        "arguments": {"keys": [{"type": "qcode", "data": "f12"}], "hold-time": 30},
    }]
    if (tuple(name for name, _ in groups) != (*probe.PHASES["normal"], "close")
            or groups[-1][1] != expected_close):
        raise ValueError("Diagnostic QMP groups changed; review interactive input contract")
    if any(key.get("data") == "f12"
           for _, group in groups[:-1] for request in group
           if request["execute"] == "send-key"
           for key in request["arguments"]["keys"]):
        raise ValueError("F12 is forbidden anywhere in the interactive input chain")
    return groups[:-1]


def matching_reference_frame(raw: bytes, reference: bytes) -> bool:
    if ppm(raw)[:2] != (WIDTH, HEIGHT) or ppm(reference)[:2] != (WIDTH, HEIGHT):
        raise ValueError("Wrong interactive/reference framebuffer geometry")
    # No masks: cursor, caret, chrome and every app pixel must match.
    return raw == reference


def run_interactive(control, process, directory: Path, reference: dict, deadline: float) -> dict:
    probe.validate_frames(reference["frames"], "normal")
    frames = {}
    stages = []
    for name, group in interactive_groups(directory):
        for request in group:
            if request["execute"] == "screendump":
                while True:
                    if process.poll() is not None or time.monotonic() >= deadline:
                        raise TimeoutError(f"Interactive native frame did not match reference: {name}")
                    verify_ready_transcript(read_serial(directory / "serial.txt"))
                    control.command(request["execute"], request["arguments"])
                    raw = probe.frame_bytes(directory, name)
                    if matching_reference_frame(raw, reference["frames"][name]):
                        frames[name] = raw
                        stages.append({
                            "phase": name,
                            "screenshot_sha256": hashlib.sha256(raw).hexdigest(),
                            "qmp_index": len(control.trace),
                        })
                        break
                    time.sleep(min(0.08, max(0, deadline - time.monotonic())))
            else:
                if not stages or time.monotonic() >= deadline or process.poll() is not None:
                    raise RuntimeError("Interactive QMP input before ready or after guest exit/deadline")
                control.command(request["execute"], request["arguments"])
                time.sleep(min(0.08, max(0, deadline - time.monotonic())))
    pixels = probe.validate_frames(frames, "normal")
    return {"stages": stages, "frames": pixels, "pixel_masks": []}


def inspect_stopped_disk(process, directory: Path) -> dict:
    raw_exit = process.poll()
    if type(raw_exit) is not int or raw_exit != 0:
        raise ValueError("Independent disk inspection requires an exited QEMU with raw exit 0")
    check_image(directory / "volume.img")
    after = (directory / "volume.img").read_bytes()
    oracle = probe.disk_oracle(after)
    publish(directory / "volume.after.img", after)
    return {
        "disk": oracle,
        "before_sha256": sha256(directory / "volume.before.img"),
        "after_sha256": sha256(directory / "volume.after.img"),
    }


def run_preview(
    *,
    build_directory: Path = BUILD,
    bios_rom: Path,
    vga_rom: Path,
    timeout: int = 45,
    machine: str | None = None,
    interactive_reference_run: Path | None = None,
    session_fixture_directory: Path | None = None,
) -> dict:
    report_path = private_path(Path(build_directory) / "preview-result.json")
    if report_path.exists():
        report_path.unlink()
    if os.name == "nt":
        raise RuntimeError("This evidence runner uses Linux QMP pipes; use the Windows bundle launcher.")
    if interactive_reference_run is not None and session_fixture_directory is not None:
        raise ValueError("Choose one native walkthrough, not both")
    maximum = 240 if interactive_reference_run is not None or session_fixture_directory is not None else 60
    if type(timeout) is not int or not 5 <= timeout <= maximum:
        raise ValueError(f"Preview timeout must be between 5 and {maximum} seconds")
    machine = qemu_machine(machine)
    build, manifest = validate_build(Path(build_directory))
    frozen = source_hash()
    if manifest["source_sha256"] != frozen:
        raise ValueError("Build source changed before preview")
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if qemu is None:
        raise RuntimeError("Missing QEMU")
    qemu_version = subprocess.check_output(
        [qemu, "--version"], env=env, text=True, timeout=10
    ).strip()
    code, variables = firmware()
    run_root = private_path(build / "preview-run")
    run_root.mkdir(parents=True, exist_ok=True)
    directory = private_path(run_root / ("materiaal-" + uuid.uuid4().hex[:10]))
    directory.mkdir()

    shutil.copytree(build / "esp", directory / "esp")
    for relative, expected in (
        ("esp/kernel.elf", manifest["kernel"]["sha256"]),
        ("esp/EFI/BOOT/BOOTX64.EFI", manifest["loader"]["sha256"]),
    ):
        if sha256(directory / relative) != expected:
            raise ValueError(f"Build artifact changed while preparing preview: {relative}")
    for source, name in (
        (code, "OVMF_CODE.fd"),
        (variables, "OVMF_VARS.template.fd"),
        (variables, "OVMF_VARS.fd"),
        (bios_rom, "bios-256k.bin"),
        (vga_rom, "VGA.rom"),
    ):
        source = Path(source)
        details = source.lstat()
        limit = 32 * 1024 * 1024 if name.startswith("OVMF_") else 1024 * 1024
        if not stat.S_ISREG(details.st_mode) or details.st_nlink != 1 or not 0 < details.st_size <= limit:
            raise ValueError(f"Invalid native preview input: {name}")
        publish(directory / name, source.read_bytes())
    if (directory / "OVMF_VARS.fd").read_bytes() != (directory / "OVMF_VARS.template.fd").read_bytes():
        raise ValueError("Initial writable firmware differs from its frozen template")
    volume_disk.create_image(directory / "volume.img", "empty")
    check_image(directory / "volume.img")
    if interactive_reference_run is not None:
        publish(directory / "volume.before.img", (directory / "volume.img").read_bytes())

    command = [
        qemu, "-L", ".", "-machine", machine, "-accel", "tcg,thread=single",
        "-cpu", "qemu64", "-smp", "1", "-m", "256M", "-nodefaults",
        "-display", "none", "-monitor", "none", "-serial", "file:serial.txt",
        "-nic", "none", "-no-reboot",
        "-device", "VGA,vgamem_mb=16,romfile=VGA.rom",
        "-qmp", "stdio",
        "-drive", "if=pflash,unit=0,format=raw,readonly=on,file=OVMF_CODE.fd",
        "-drive", "if=pflash,unit=1,format=raw,file=OVMF_VARS.fd",
        "-drive", "if=virtio,index=0,format=raw,readonly=on,file=fat:esp",
        "-object", "rng-random,id=rng0,filename=/dev/urandom",
        "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000",
        "-drive", "if=none,id=volume,format=raw,cache=writeback,readonly=off,file=volume.img",
        "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST",
    ]
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    deadline = start + timeout
    report = {
        "schema_version": 1,
        "passed": False,
        "scope": (
            "Native boot/readiness and first rendered frame only; no manual UX, "
            "full desktop, release, or Windows-host acceptance."
        ),
        "source_commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True, timeout=10
        ).strip(),
        "source_sha256": frozen,
        "source_fingerprint_version": 3,
        "kernel_sha256": manifest["kernel"]["sha256"],
        "loader_sha256": manifest["loader"]["sha256"],
        "kernel_feature": manifest["kernel_feature"],
        "machine": machine,
        "qemu_version": qemu_version,
        "qemu_sha256": sha256(Path(qemu)),
        "firmware": {
            "ovmf_code_sha256": sha256(directory / "OVMF_CODE.fd"),
            "ovmf_vars_template_sha256": sha256(directory / "OVMF_VARS.template.fd"),
            "seabios_bios_sha256": sha256(directory / "bios-256k.bin"),
            "seabios_vga_sha256": sha256(directory / "VGA.rom"),
        },
        "command": command,
        "run_directory": str(directory),
        "started_utc": started,
    }
    reference = None
    session_fixture = None
    if interactive_reference_run is not None:
        report["scope"] = (
            "Interactive seven-process native input/save/list/reopen against exact native "
            "reference pixels and post-quit disk bytes; no controller/snapshot/cleanup, "
            "cold recovery, manual UX, full release or Windows-host acceptance."
        )
    if session_fixture_directory is not None:
        report["scope"] = (
            "Opt-in native guest accounts/login/desktop/search/Materiaal route through QMP; "
            "all app-owned pixels match a same-source host-rendered Controller/Frame oracle. "
            "No real account authentication, secure lock/key release, manual UX, release, "
            "or physical/Windows-host acceptance."
        )
    publish_json(report_path, report)

    process = control = None
    try:
        if session_fixture_directory is not None:
            import run_session_preview as session_preview
            session_fixture = session_preview.load_fixture(
                session_fixture_directory, manifest, frozen,
            )
            report["session_oracle"] = {
                "directory": str(session_fixture["directory"]),
                "metadata_sha256": session_fixture["metadata_sha256"],
                **session_fixture["metadata"],
            }
            publish_json(report_path, report)
        if interactive_reference_run is not None:
            reference = load_reference(
                interactive_reference_run, manifest, frozen, machine,
                report["firmware"], report["qemu_sha256"],
            )
            report["interactive_reference"] = {
                "run_directory": str(reference["directory"]),
                "report_sha256": reference["report_sha256"],
                "source_sha256": reference["source_sha256"],
                "apps": reference["apps"],
                "qemu_implementation_sha256": reference["qemu_implementation_sha256"],
                "frames_sha256": {
                    name: hashlib.sha256(raw).hexdigest()
                    for name, raw in reference["frames"].items()
                },
            }
            publish_json(report_path, report)
        with (directory / "stderr.txt").open("xb") as stderr:
            process = subprocess.Popen(
                command,
                cwd=directory,
                env=env,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=stderr,
                bufsize=0,
            )
            control = QmpPipe(process, deadline)
            if reference is not None or session_fixture is not None:
                implementation = Path(os.readlink(f"/proc/{process.pid}/exe"))
                report["qemu_implementation_sha256"] = sha256(implementation)
                if reference is not None and report["qemu_implementation_sha256"] != reference["qemu_implementation_sha256"]:
                    raise ValueError("Interactive QEMU implementation differs from native reference")
            expected = (*BOOT, *READY)
            while True:
                serial = read_serial(directory / "serial.txt")
                lines = serial.decode("utf-8", "strict").splitlines()
                if any("FAIL" in line or "PANIC" in line for line in lines):
                    raise ValueError("Native guest reported a failure")
                if all(marker in lines for marker in expected):
                    break
                if process.poll() is not None or time.monotonic() >= deadline:
                    raise TimeoutError("Native Materiaal session did not reach all readiness gates")
                time.sleep(0.02)
            observed = verify_ready_transcript(serial)
            if session_fixture is not None:
                report["session_walkthrough"] = session_preview.run_walkthrough(
                    control, process, directory, session_fixture, deadline,
                )
                screenshot = directory / "session-material.ppm"
            elif reference is not None:
                report["interactive"] = run_interactive(
                    control, process, directory, reference, deadline
                )
                screenshot = directory / "reopened.ppm"
            else:
                screenshot = directory / "preview.ppm"
                control.command("screendump", {"filename": str(screenshot)})
            frame = verify_frame(screenshot.read_bytes())
            serial = read_serial(directory / "serial.txt")
            observed = verify_ready_transcript(serial)
            serial_hash = sha256(directory / "serial.txt")
            control.command("quit")
            raw_exit = process.wait(timeout=max(1, deadline - time.monotonic()))
            if time.monotonic() >= deadline:
                raise TimeoutError("Native preview exceeded its shared VM deadline")
        if raw_exit != 0:
            raise RuntimeError(f"QEMU preview exited unexpectedly: {raw_exit}")
        observed = verify_ready_transcript(read_serial(directory / "serial.txt"))
        serial_hash = sha256(directory / "serial.txt")
        if source_hash() != frozen:
            raise ValueError("Source changed during native preview")
        if session_fixture is not None:
            confirmed = session_preview.load_fixture(
                session_fixture["directory"], manifest, frozen,
            )
            if (confirmed["metadata_sha256"] != session_fixture["metadata_sha256"]
                    or confirmed["frames"] != session_fixture["frames"]
                    or sha256(implementation) != report["qemu_implementation_sha256"]
                    or sha256(Path(qemu)) != report["qemu_sha256"]
                    or source_hash() != frozen):
                raise ValueError("Session oracle, source or QEMU identity changed during run")
        if reference is not None:
            report["interactive"].update(inspect_stopped_disk(process, directory))
            if (sha256(implementation) != report["qemu_implementation_sha256"]
                    or sha256(Path(qemu)) != report["qemu_sha256"]):
                raise ValueError("Interactive QEMU identity changed during run")
            confirmed = load_reference(
                reference["directory"], manifest, frozen, machine,
                report["firmware"], report["qemu_sha256"],
            )
            if (confirmed["report_sha256"] != reference["report_sha256"]
                    or confirmed["frames"] != reference["frames"]):
                raise ValueError("Reference changed during interactive run")
            if source_hash() != frozen:
                raise ValueError("Source changed during final reference validation")
        report.update(
            passed=True,
            raw_qemu_exit=raw_exit,
            duration_s=time.monotonic() - start,
            readiness_markers=observed,
            qmp_trace=control.trace if control is not None else [],
            serial_sha256=serial_hash,
            screenshot_sha256=sha256(screenshot),
            screenshot_size=frame,
            volume_sha256=sha256(directory / "volume.img"),
        )
    except Exception as error:
        report.update(
            error=f"{type(error).__name__}: {error}",
            raw_qemu_exit=process.returncode if process is not None else None,
            duration_s=time.monotonic() - start,
            qmp_trace=control.trace if control is not None else [],
        )
        publish_json(directory / "result.json", report)
        publish_json(report_path, report)
        raise
    finally:
        if control is not None:
            control.close()
        if process is not None and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)

    publish_json(directory / "result.json", report)
    publish_json(report_path, report)
    print(json.dumps(report, indent=2))
    return report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build-dir", type=Path, default=BUILD)
    parser.add_argument("--bios-rom", type=Path, required=True)
    parser.add_argument("--vga-rom", type=Path, required=True)
    parser.add_argument("--timeout", type=int, default=45)
    parser.add_argument("--machine")
    parser.add_argument(
        "--interactive-reference-run", type=Path,
        help="Replay a completed same-source normal native probe and require its exact phase pixels",
    )
    args = parser.parse_args()
    run_preview(
        build_directory=args.build_dir,
        bios_rom=args.bios_rom,
        vga_rom=args.vga_rom,
        timeout=args.timeout,
        machine=args.machine,
        interactive_reference_run=args.interactive_reference_run,
    )


if __name__ == "__main__":
    main()
