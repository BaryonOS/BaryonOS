#!/usr/bin/env python3
"""Run an isolated, networkless QEMU guest and retain raw evidence."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
from pathlib import Path
import shutil
import stat
import subprocess
import time
import uuid
from common import ROOT, BUILD, environment, firmware, qemu_machine, save_json, sha256, source_hash
from gate_contract import (
    FAIL_MARKERS as BASE_FAIL_MARKERS,
    MILESTONE_MARKERS as BASE_MILESTONE_MARKERS,
    M1_MARKERS,
    guest_expectation,
)

MARKERS = M1_MARKERS
FAIL_MARKERS = BASE_FAIL_MARKERS + ("M5:FAIL",)
LATER_MILESTONE_PREFIXES = ("M2:", "M3:", "M4:", "M5:", "M6A:", "M6B:", "M6C:", "QSIM:")
MILESTONE_MARKERS = {
    **BASE_MILESTONE_MARKERS,
    "m5": MARKERS[:3] + ("M5:CASE:READ:PASS", "M5:SURVIVOR_PROGRESS:PASS",
                         "M5:RESOURCES:PASS", "M5:PASS", "M1:PASS"),
}
# The longest admitted native profile is the specified 600-second SMP soak;
# its runner reserves 60 seconds for measured boot and teardown. Callers still
# have to opt in explicitly because the ordinary default remains 60 seconds.
MAX_GUEST_TIMEOUT_S = 660


def is_success(returncode, stdout, timed_out=False, *, required_markers=MARKERS,
               expected_exit=33, forbidden_markers=FAIL_MARKERS):
    required_markers = tuple(required_markers)
    m1_only = ("M1:PASS" in required_markers
               and not any(marker.startswith(LATER_MILESTONE_PREFIXES) for marker in required_markers))
    if m1_only and any(line.startswith(LATER_MILESTONE_PREFIXES) for line in stdout.splitlines()):
        return False
    lines = iter(stdout.splitlines())
    for marker in required_markers:
        if not any(line == marker for line in lines):
            return False
    return (returncode == expected_exit and not timed_out
            and not any(marker in stdout for marker in forbidden_markers))


def run_guest(label="smoke", *, esp=None, timeout=60, required_markers=MARKERS,
              expected_exit=33, forbidden_markers=FAIL_MARKERS, m5_case=None, m5_seed=None,
              expected_kernel_sha256=None, expected_loader_sha256=None, memory_mib=256,
              rtc_base=None, tpm_socket=None, cpus=1, tcg_threads="single",
              acpi_tables=(), reboot_on_reset=False, xhci_case=None,
              hid_input_probe=False, vga_rom=None, usb_hid_case=None):
    # Defaults keep every existing profile byte-identical (-smp 1, single TCG).
    if type(cpus) is not int or not 1 <= cpus <= 16:
        raise ValueError("cpus must be an explicit integer in [1,16]")
    if tcg_threads not in ("single", "multi"):
        raise ValueError("tcg_threads must be single or multi")
    if xhci_case is not None and (
            xhci_case not in ("noop", "fault", "no-dmar") or cpus != 1
            or tcg_threads != "single" or memory_mib != 256
            or m5_case is not None or rtc_base is not None or tpm_socket is not None
            or acpi_tables or reboot_on_reset or hid_input_probe or usb_hid_case is not None):
        raise ValueError("xHCI DMA cases require the isolated single-CPU fixture")
    if type(hid_input_probe) is not bool:
        raise ValueError("HID input fixture selection must be boolean")
    if hid_input_probe and (cpus != 1 or tcg_threads != "single" or memory_mib != 256
            or m5_case is not None or rtc_base is not None or tpm_socket is not None
            or acpi_tables or reboot_on_reset or xhci_case is not None or vga_rom is None
            or usb_hid_case is not None):
        raise ValueError("HID input requires the isolated desktop fixture and explicit VGA ROM")
    if not hid_input_probe and vga_rom is not None:
        raise ValueError("VGA ROM belongs only to the HID input fixture")
    if usb_hid_case is not None and (
            usb_hid_case not in ("reports", "readonly") or cpus != 1
            or tcg_threads != "single" or memory_mib != 256 or m5_case is not None
            or rtc_base is not None or tpm_socket is not None or acpi_tables
            or reboot_on_reset or xhci_case is not None or hid_input_probe):
        raise ValueError("USB HID requires the isolated single-CPU xHCI fixture")
    rtc_arguments = []
    if rtc_base is not None:
        # A controlled UTC calendar, never a host wall-clock fallback.
        parsed = datetime.strptime(rtc_base, "%Y-%m-%dT%H:%M:%S")
        if parsed.strftime("%Y-%m-%dT%H:%M:%S") != rtc_base:
            raise ValueError("RTC base must be a canonical UTC calendar")
        rtc_arguments = ["-rtc", f"base={rtc_base},clock=vm,driftfix=none"]
    if not BUILD.resolve().is_relative_to(ROOT.resolve()):
        raise ValueError("NOS_BUILD_DIR must be inside the checkout for QEMU runs")
    if not 0 < timeout <= MAX_GUEST_TIMEOUT_S:
        raise ValueError(f"guest timeout must be in (0,{MAX_GUEST_TIMEOUT_S}]")
    if type(memory_mib) is not int or memory_mib not in (128, 256):
        raise ValueError("Choose the explicit 128 MiB OOM or 256 MiB regression profile")
    machine = qemu_machine()
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
    hid_volume = None
    if hid_input_probe:
        from volume_disk import create_image
        rom = Path(vga_rom)
        info = rom.lstat()
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or not 0 < info.st_size <= 1 << 20:
            raise ValueError("HID VGA ROM must be a bounded regular file")
        shutil.copy2(rom, run_dir / "VGA.rom")
        hid_volume = run_dir / "volume.img"
        create_image(hid_volume, "empty")
        hid_volume_initial = sha256(hid_volume)
    disk_path, disk_info = None, None
    if m5_case is not None:
        from build import M5_CASES
        from m5_disk import create_disk
        if m5_case not in ("read",) + M5_CASES:
            raise ValueError("Unknown M5 case")
        disk_path, disk_info = create_disk(run_dir, m5_seed)
    shutil.copytree(source, run_dir / "esp")
    expected_artifacts = {}
    for name, expected in (("esp/kernel.elf", expected_kernel_sha256),
                           ("esp/EFI/BOOT/BOOTX64.EFI", expected_loader_sha256)):
        if expected is not None:
            actual = sha256(run_dir / name)
            if actual != expected:
                raise ValueError(f"Snapshotted artifact SHA256 mismatch: {name}; expected={expected}; actual={actual}")
            expected_artifacts[name] = expected
    shutil.copy2(variables, run_dir / "OVMF_VARS.template.fd")
    shutil.copy2(run_dir / "OVMF_VARS.template.fd", run_dir / "OVMF_VARS.fd")
    shutil.copy2(code, run_dir / "OVMF_CODE.fd")
    # Use paths relative to cwd: drive-letter colons and commas are not passed
    # through QEMU's file-option parser, and the FAT tree cannot change mid-run.
    run_rel = run_dir.relative_to(ROOT).as_posix()
    args = [qemu, "-machine", machine, "-accel", f"tcg,thread={tcg_threads}", "-cpu", "qemu64", "-smp", str(cpus), "-m", f"{memory_mib}M", "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "stdio", "-nic", "none", "-no-reboot",
            "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={run_rel}/OVMF_CODE.fd",
            "-drive", f"if=pflash,unit=1,format=raw,file={run_rel}/OVMF_VARS.fd",
            "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{run_rel}/esp",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
    args += rtc_arguments
    if xhci_case is not None:
        if xhci_case != "no-dmar":
            args += ["-device", "intel-iommu,caching-mode=on,intremap=off,device-iotlb=off,aw-bits=39"]
        args += ["-device", "qemu-xhci,id=h02-xhci,bus=pcie.0,addr=0x5,p2=1,p3=1"]
    if usb_hid_case is not None:
        args += ["-device", "intel-iommu,caching-mode=on,intremap=off,device-iotlb=off,aw-bits=39",
                 "-device", "qemu-xhci,id=xhci,bus=pcie.0,addr=0x5,p2=2,p3=0",
                 "-device", "usb-kbd,id=hid-kbd,bus=xhci.0,port=1,usb_version=1",
                 "-device", "usb-mouse,id=hid-mouse,bus=xhci.0,port=2,usb_version=1"]
    if hid_input_probe:
        args += ["-device", f"VGA,vgamem_mb=16,romfile={run_rel}/VGA.rom",
                 "-object", "rng-random,id=rng0,filename=/dev/urandom",
                 "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000",
                 "-drive", f"if=none,id=volume,format=raw,cache=writeback,file={run_rel}/volume.img",
                 "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST"]
    if reboot_on_reset:
        # Overrides -no-reboot: a reset re-runs firmware (second EBS marker)
        # instead of exiting 0, so raw exit 0 can only mean guest poweroff.
        args += ["-action", "reboot=reset"]
    for table in acpi_tables:
        # Generated firmware-table fixtures only, never host ACPI tables.
        table = Path(table).resolve(strict=True)
        if not table.is_relative_to(BUILD.resolve()) or not table.is_file():
            raise ValueError("ACPI table fixtures must be generated files under this build root")
        args += ["-acpitable", f"file={table.relative_to(ROOT).as_posix()}"]
    if tpm_socket is not None:
        control = Path(tpm_socket).resolve(strict=True)
        if not control.is_relative_to(BUILD.resolve()) or not control.is_socket():
            raise ValueError("TPM control must be a generated Unix socket under this build root")
        args += ["-chardev", f"socket,id=tpmsock,path={control.relative_to(ROOT).as_posix()}",
                 "-tpmdev", "emulator,id=tpm0,chardev=tpmsock",
                 "-device", "tpm-crb,tpmdev=tpm0"]
    if m5_case is not None and m5_case != "no-device":
        args += ["-drive", f"if=none,id=m5-test,format=raw,readonly=on,file={run_rel}/test-disk.img",
                 "-device", "virtio-blk-pci,drive=m5-test,id=m5blk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-M5-TEST"]
    share = Path(qemu).parent / "share"
    if share.is_dir():
        args += ["-L", str(share)]
    version = subprocess.check_output([qemu, "--version"], env=env, text=True).strip()
    start = time.monotonic()
    started = datetime.now(timezone.utc).isoformat()
    timed_out = False
    usb_control = None
    try:
        if usb_hid_case is not None:
            from run_usb_hid import controlled_guest
            usb_control = controlled_guest(args, cwd=ROOT, env=env, timeout=timeout,
                                           run_dir=run_dir, case=usb_hid_case)
            rc, out, err = (usb_control["returncode"], usb_control["stdout"], usb_control["stderr"])
            timed_out = usb_control["timed_out"]
        else:
            result = subprocess.run(args, cwd=ROOT, env=env, capture_output=True, timeout=timeout)
            rc, out, err = result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired as exc:
        timed_out, rc, out, err = True, None, exc.stdout or b"", exc.stderr or b""
    stdout, stderr = out.decode("utf-8", "replace"), err.decode("utf-8", "replace")
    (run_dir / "serial.txt").write_bytes(out)
    (run_dir / "stderr.txt").write_bytes(err)
    artifacts = {path.relative_to(run_dir).as_posix(): sha256(path) for path in (run_dir / "esp").rglob("*") if path.is_file()}
    artifacts.update({"OVMF_CODE.fd": sha256(run_dir / "OVMF_CODE.fd"), "OVMF_VARS.template.fd": sha256(run_dir / "OVMF_VARS.template.fd")})
    if hid_input_probe:
        artifacts["VGA.rom"] = sha256(run_dir / "VGA.rom")
    artifact_binding_passed = all(artifacts.get(name) == expected for name, expected in expected_artifacts.items())
    disk_result = None
    if disk_info is not None:
        from m5_disk import verify_disk
        disk_result = verify_disk(disk_path, disk_info, stdout,
                                  data_required=m5_case not in ("no-device", "reset-timeout"))
        artifacts["test-disk.img"] = sha256(disk_path)
    git = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True)
    report = {"label": label, "started_utc": started, "duration_s": time.monotonic()-start, "timeout_s": timeout, "timed_out": timed_out, "raw_qemu_exit": rc, "expected_exit": expected_exit, "required_markers": list(required_markers), "forbidden_markers": list(forbidden_markers), "passed": is_success(rc, stdout, timed_out, required_markers=required_markers, expected_exit=expected_exit, forbidden_markers=forbidden_markers), "command": args, "qemu_version": version, "profile": f"development-package; {machine}/qemu64/TCG{'' if tcg_threads == 'single' else '-MT'}/{cpus}CPU/{memory_mib}MiB", "base_commit": git.stdout.strip() if git.returncode == 0 else None, "source_fingerprint_version": 3, "source_sha256": source_hash(), "artifact_sha256": artifacts, "firmware_source": [str(code), str(variables)], "run_directory": str(run_dir), "stdout": stdout, "stderr": stderr}
    report.update(expected_artifact_sha256=expected_artifacts, artifact_binding_passed=artifact_binding_passed)
    if xhci_case is not None:
        report.update(xhci_case=xhci_case, dma_scope="isolated QEMU requester 00:05.0; no USB device attached")
    if hid_input_probe:
        report.update(hid_input_probe=True, input_scope="synthetic HID through native service IPC; no USB device attached",
                      volume_initial_sha256=hid_volume_initial, volume_final_sha256=sha256(hid_volume))
    if usb_hid_case is not None:
        report.update(usb_hid_case=usb_hid_case, usb_control=usb_control and {
            "trace": usb_control["trace"], "error": usb_control["error"]})
        report["passed"] = report["passed"] and usb_control is not None and usb_control["error"] is None
    report['passed'] = report['passed'] and artifact_binding_passed
    if disk_result is not None:
        report.update(m5_case=m5_case, disk_oracle=disk_result)
        report["passed"] = report["passed"] and disk_result["passed"]
        report["profile"] += "; readonly synthetic virtio-blk00:05.0" if m5_case != "no-device" else "; M5 target absent"
    save_json(run_dir / "result.json", report)
    print(stdout, end="")
    print(stderr, end="")
    print(f"QEMU raw={rc}; expected={expected_exit}; timeout={timed_out}; expectation_pass={report['passed']}; {run_dir}")
    return report


def run_gate_case(phase, group, *, label=None):
    expectation = guest_expectation(phase, group)
    if expectation is None:
        raise ValueError(f"Unknown guest gate case: {phase}/{group}")
    required_markers, expected_exit, forbidden_markers = expectation
    return run_guest(label or group, required_markers=required_markers,
                     expected_exit=expected_exit, forbidden_markers=forbidden_markers)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--smoke", action="store_true")
    parser.add_argument("--milestone", choices=list(MILESTONE_MARKERS), default="m1")
    args = parser.parse_args()
    raise SystemExit(0 if run_guest(required_markers=MILESTONE_MARKERS[args.milestone], m5_case="read" if args.milestone == "m5" else None)["passed"] else 1)
