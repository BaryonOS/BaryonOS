#!/usr/bin/env python3
"""Boot the non-probe Materiaal session and verify its first native frame."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import time
import uuid

from common import BUILD, ROOT, environment, firmware, qemu_machine, sha256, source_hash
from package_windows_qemu import validate_build
from run_graphics import QmpPipe, ppm
from run_system import check_image
from run_fpu import private_path, publish, publish_json
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


def run_preview(
    *,
    build_directory: Path = BUILD,
    bios_rom: Path,
    vga_rom: Path,
    timeout: int = 45,
    machine: str | None = None,
) -> dict:
    if os.name == "nt":
        raise RuntimeError("This evidence runner uses Linux QMP pipes; use the Windows bundle launcher.")
    if type(timeout) is not int or not 5 <= timeout <= 60:
        raise ValueError("Preview timeout must be between 5 and 60 seconds")
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
    volume_disk.create_image(directory / "volume.img", "empty")
    check_image(directory / "volume.img")

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
    report_path = build / "preview-result.json"
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
        "kernel_sha256": manifest["kernel"]["sha256"],
        "loader_sha256": manifest["loader"]["sha256"],
        "kernel_feature": manifest["kernel_feature"],
        "machine": machine,
        "qemu_version": qemu_version,
        "qemu_sha256": sha256(Path(qemu)),
        "firmware": {
            "ovmf_code_sha256": sha256(code),
            "ovmf_vars_template_sha256": sha256(variables),
            "seabios_bios_sha256": sha256(bios_rom),
            "seabios_vga_sha256": sha256(vga_rom),
        },
        "command": command,
        "run_directory": str(directory),
        "started_utc": started,
    }
    publish_json(report_path, report)

    process = control = None
    try:
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
            screenshot = directory / "preview.ppm"
            control.command("screendump", {"filename": str(screenshot)})
            frame = verify_frame(screenshot.read_bytes())
            serial = read_serial(directory / "serial.txt")
            observed = verify_ready_transcript(serial)
            serial_hash = sha256(directory / "serial.txt")
            control.command("quit")
            raw_exit = process.wait(timeout=max(1, deadline - time.monotonic()))
        if raw_exit != 0:
            raise RuntimeError(f"QEMU preview exited unexpectedly: {raw_exit}")
        if source_hash() != frozen:
            raise ValueError("Source changed during native preview")
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
    args = parser.parse_args()
    run_preview(
        build_directory=args.build_dir,
        bios_rom=args.bios_rom,
        vga_rom=args.vga_rom,
        timeout=args.timeout,
        machine=args.machine,
    )


if __name__ == "__main__":
    main()
