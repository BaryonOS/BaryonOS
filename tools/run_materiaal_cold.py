#!/usr/bin/env python3
"""Cold reopen of the ordinary seven-process guest, with no controller or masks."""
from __future__ import annotations

import argparse
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import time
import uuid

import materiaal_cold as cold
import run_materiaal_preview as preview


def run_cold(*, build_directory: Path, parent_run: Path, reference_run: Path,
             bios_rom: Path, vga_rom: Path, machine: str, timeout: int = 180) -> dict:
    report_path = preview.private_path(build_directory / "cold-result.json")
    if report_path.exists():
        report_path.unlink()
    if os.name != "posix" or type(timeout) is not int or not 5 <= timeout <= 240:
        raise ValueError("Cold proof requires Linux and a 5..240 second deadline")
    machine = preview.qemu_machine(machine)
    build, manifest = preview.validate_build(build_directory)
    frozen = preview.source_hash()
    if manifest["source_sha256"] != frozen:
        raise ValueError("Cold build is not current source")
    env = preview.environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if qemu is None:
        raise RuntimeError("Missing QEMU")
    qemu_hash = preview.sha256(Path(qemu))
    code, variables = preview.firmware()
    root = preview.private_path(build / "cold-run")
    root.mkdir(exist_ok=True)
    directory = preview.private_path(root / ("materiaal-" + uuid.uuid4().hex[:10]))
    directory.mkdir(mode=0o700)
    report = {
        "schema_version": 1, "passed": False,
        "scope": "Ordinary seven-process cold reopen and unchanged disk; no write-denial, "
                 "cleanup, controller, manual UX, production encryption or release acceptance.",
        "stop_method": "host QMP quit, not guest clean shutdown or abrupt power loss",
        "guest_clean_shutdown": False,
        "source_sha256": frozen, "source_fingerprint_version": 3,
        "run_directory": str(directory), "machine": machine,
        "source_commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=preview.ROOT, text=True, timeout=10
        ).strip(),
        "kernel_sha256": manifest["kernel"]["sha256"],
        "loader_sha256": manifest["loader"]["sha256"],
    }
    preview.publish_json(report_path, report)
    process = control = None
    start = time.monotonic()
    deadline = start + timeout
    try:
        shutil.copytree(build / "esp", directory / "esp")
        for path, digest in (
            ("esp/kernel.elf", manifest["kernel"]["sha256"]),
            ("esp/EFI/BOOT/BOOTX64.EFI", manifest["loader"]["sha256"]),
        ):
            if preview.sha256(directory / path) != digest:
                raise ValueError("Cold executable changed while copying")
        for source, name in (
            (code, "OVMF_CODE.fd"), (variables, "OVMF_VARS.template.fd"),
            (variables, "OVMF_VARS.fd"), (bios_rom, "bios-256k.bin"), (vga_rom, "VGA.rom"),
        ):
            source = preview.private_path(Path(source), regular=True)
            maximum = 32 * 1024 * 1024 if name.startswith("OVMF_") else 1024 * 1024
            if not 0 < source.stat().st_size <= maximum:
                raise ValueError("Oversized or empty cold firmware input")
            preview.publish(directory / name, source.read_bytes())
        firmware = {
            "ovmf_code_sha256": preview.sha256(directory / "OVMF_CODE.fd"),
            "ovmf_vars_template_sha256": preview.sha256(directory / "OVMF_VARS.template.fd"),
            "seabios_bios_sha256": preview.sha256(directory / "bios-256k.bin"),
            "seabios_vga_sha256": preview.sha256(directory / "VGA.rom"),
        }
        reference = preview.load_reference(
            reference_run, manifest, frozen, machine, firmware, qemu_hash
        )
        parent = cold.replay_parent(
            parent_run, manifest, reference, frozen, machine, firmware, qemu_hash
        )
        expected = {
            "list": cold.cold_list_frame(reference["frames"]),
            "reopened": reference["frames"]["reopened"],
        }
        preview.publish(directory / "volume.img", parent["disk"])
        preview.publish(directory / "volume.before.img", parent["disk"])
        command = [
            qemu, "-L", ".", "-machine", machine, "-accel", "tcg,thread=single",
            "-cpu", "qemu64", "-smp", "1", "-m", "256M", "-nodefaults",
            "-display", "none", "-monitor", "none", "-serial", "file:serial.txt",
            "-nic", "none", "-no-reboot", "-device", "VGA,vgamem_mb=16,romfile=VGA.rom",
            "-qmp", "stdio",
            "-drive", "if=pflash,unit=0,format=raw,readonly=on,file=OVMF_CODE.fd",
            "-drive", "if=pflash,unit=1,format=raw,file=OVMF_VARS.fd",
            "-drive", "if=virtio,index=0,format=raw,readonly=on,file=fat:esp",
            "-object", "rng-random,id=rng0,filename=/dev/urandom",
            "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000",
            "-drive", "if=none,id=volume,format=raw,cache=writeback,readonly=off,file=volume.img",
            "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST",
        ]
        report.update(
            command=command, firmware=firmware, qemu_entry_sha256=qemu_hash,
            parent_report_sha256=parent["report_sha256"],
            parent_run=str(parent["directory"]), reference_report_sha256=reference["report_sha256"],
            apps=reference["apps"],
            pixel_masks=[], stages=[],
            initial_list_oracle="Layout::status Ready pixels composed with populated native list",
        )
        with (directory / "stderr.txt").open("xb") as stderr:
            process = subprocess.Popen(
                command, cwd=directory, env=env, stdin=subprocess.PIPE,
                stdout=subprocess.PIPE, stderr=stderr, bufsize=0,
            )
            control = preview.QmpPipe(process, deadline)
            implementation = Path(os.readlink(f"/proc/{process.pid}/exe"))
            implementation_hash = preview.sha256(implementation)
            if implementation_hash != reference["qemu_implementation_sha256"]:
                raise ValueError("Cold QEMU implementation differs")
            report["qemu_implementation_sha256"] = implementation_hash
            while True:
                raw = preview.read_serial(directory / "serial.txt")
                lines = raw.decode("utf-8", "strict").splitlines()
                if any("FAIL" in line or "PANIC" in line for line in lines):
                    raise ValueError("Cold guest failure")
                if all(marker in lines for marker in (*preview.BOOT, *preview.READY)):
                    preview.verify_ready_transcript(raw)
                    break
                if process.poll() is not None or time.monotonic() >= deadline:
                    raise TimeoutError("Cold guest readiness")
                time.sleep(0.02)
            for phase, group in cold.cold_groups(directory):
                for request in group:
                    if request["execute"] != "screendump":
                        if process.poll() is not None or time.monotonic() >= deadline:
                            raise TimeoutError("Cold input after guest exit/deadline")
                        control.command(request["execute"], request["arguments"])
                        time.sleep(min(0.08, max(0, deadline - time.monotonic())))
                        continue
                    while True:
                        if process.poll() is not None or time.monotonic() >= deadline:
                            raise TimeoutError("Cold exact frame " + phase)
                        preview.verify_ready_transcript(preview.read_serial(directory / "serial.txt"))
                        control.command(request["execute"], request["arguments"])
                        frame = preview.probe.frame_bytes(directory, phase)
                        if preview.matching_reference_frame(frame, expected[phase]):
                            report["stages"].append({
                                "phase": phase, "sha256": hashlib.sha256(frame).hexdigest(),
                                "screenshot_sha256": hashlib.sha256(frame).hexdigest(),
                                "qmp_index": len(control.trace),
                            })
                            break
                        time.sleep(min(0.08, max(0, deadline - time.monotonic())))
            control.command("quit")
            raw_exit = process.wait(timeout=max(1, deadline - time.monotonic()))
        if raw_exit != 0 or time.monotonic() >= deadline:
            raise ValueError("Cold QEMU did not exit raw0 within deadline")
        disk = preview.inspect_stopped_disk(process, directory)
        if (directory / "volume.after.img").read_bytes() != parent["disk"]:
            raise ValueError("Cold interaction changed writable image")
        confirmed = cold.replay_parent(
            parent_run, manifest, reference, frozen, machine, firmware, qemu_hash
        )
        checked_reference = preview.load_reference(
            reference_run, manifest, frozen, machine, firmware, qemu_hash
        )
        if (confirmed != parent or checked_reference != reference
                or preview.source_hash() != frozen or preview.sha256(Path(qemu)) != qemu_hash
                or preview.sha256(implementation) != implementation_hash):
            raise ValueError("Cold source, parent, reference or emulator changed")
        for relative, digest in (
            ("esp/kernel.elf", manifest["kernel"]["sha256"]),
            ("esp/EFI/BOOT/BOOTX64.EFI", manifest["loader"]["sha256"]),
            ("OVMF_CODE.fd", firmware["ovmf_code_sha256"]),
            ("OVMF_VARS.template.fd", firmware["ovmf_vars_template_sha256"]),
            ("bios-256k.bin", firmware["seabios_bios_sha256"]),
            ("VGA.rom", firmware["seabios_vga_sha256"]),
        ):
            if preview.sha256(preview.private_path(directory / relative, regular=True)) != digest:
                raise ValueError("Cold frozen executable or firmware changed")
        cold.replay_input_trace(control.trace, report["stages"], directory, cold=True)
        if time.monotonic() >= deadline:
            raise TimeoutError("Cold proof exceeded shared deadline during final validation")
        report.update(passed=True, raw_qemu_exit=raw_exit, volume_unchanged=True, **disk)
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        report.update(duration_s=time.monotonic() - start, qmp_trace=control.trace if control else [])
        if control is not None:
            control.close()
        if process is not None and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
        if process is not None:
            report["raw_qemu_exit"] = process.poll()
        if (directory / "serial.txt").is_file():
            report["serial_sha256"] = preview.sha256(directory / "serial.txt")
        preview.publish_json(directory / "result.json", report)
        preview.publish_json(report_path, report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("build-dir", "parent-run", "reference-run", "bios-rom", "vga-rom"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--machine", required=True)
    parser.add_argument("--timeout", type=int, default=180)
    args = parser.parse_args()
    result = run_cold(
        build_directory=args.build_dir, parent_run=args.parent_run,
        reference_run=args.reference_run, bios_rom=args.bios_rom, vga_rom=args.vga_rom,
        machine=args.machine, timeout=args.timeout,
    )
    print(f"Ordinary cold reopen: passed={result['passed']}; {result['run_directory']}")
