#!/usr/bin/env python3
"""H06-T3 native HW-INV inventory in QEMU, checked against independent oracles.

Boots the paired native-hw-inventory kernel and hw-inventory loader. The kernel
reports `HWINV:` markers and then halts without the isa-debug-exit port, so the
runner waits for HWINV:PASS, confirms QEMU is still running (no exit path was
taken), reads QEMU's own PCI view through QMP `query-pci` and quits.

Oracles per case: the CPUID vendor and hypervisor bit the runner configured,
the presence of the emulated VT-d unit (DMAR with one DRHD) and the kernel's
own ACPI marker, and set equality of the PCI functions with `query-pci`
(address, vendor, device, class, subclass). The converted inventory must agree
with the log under `hw_evidence.pci_log_problem` and must be rejected by
`hw_evidence` as a virtual capture. QEMU evidence only; not physical hardware.

H06-T5 terminal fixtures boot the `plain` configuration with one injected
failure each (kernel panic, M2 kernel exception, HWINV:FAIL, missing kernel
file). Each must print its markers, be recognised as a protocol failure and
leave QEMU running: no terminal path may write the isa-debug-exit port.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import uuid

from common import ROOT, BUILD, environment, firmware, qemu_machine, save_json, sha256, source_hash
from hw_evidence import EvidenceError, _inventory, pci_log_problem
from hw_inventory import FAILURES, InventoryLogError, hwinv_lines, inventory_from_log
from run_graphics import Qmp

IOMMU = "intel-iommu,caching-mode=on,intremap=off,device-iotlb=off,aw-bits=39"
CASES = {
    # No VT-d unit, hypervisor bit cleared, AMD vendor string.
    "plain": dict(cpu="qemu64,vendor=AuthenticAMD,-hypervisor", vendor="AuthenticAMD", hypervisor="0",
                  devices=[], dmar="HWINV:ACPI:DMAR=ABSENT:DRHD=0", acpi="ACPI:UNAVAILABLE:NO_DMAR"),
    # One VT-d unit, hypervisor bit set, Intel vendor string, and an xHCI
    # function behind a PCIe root port (bus 1, bridge header type 1).
    "iommu": dict(cpu="qemu64,vendor=GenuineIntel", vendor="GenuineIntel", hypervisor="1",
                  devices=["-device", IOMMU,
                           "-device", "pcie-root-port,id=rp1,bus=pcie.0,chassis=1,addr=0x6",
                           "-device", "qemu-xhci,id=inv-xhci,bus=rp1"],
                  dmar="HWINV:ACPI:DMAR=PRESENT:DRHD=1", acpi="ACPI:DISCOVERY:MCFG_DMAR"),
}
# Fixture -> (kernel feature of its build, ordered marker prefixes, ESP file to remove).
FIXTURES = {
    "panic": ("test-hwinv-panic", ("BOOTINFO:OK", "M2:PANIC_LINE:", "M1:FAIL:KERNEL_PANIC"), None),
    "m2-fault": ("test-hwinv-m2-fault", ("M2:FAULT:KERNEL_UD:PASS", "M2:FAIL:KERNEL_EXCEPTION"), None),
    "hwinv-fail": ("test-hwinv-fail", ("M2:PASS", "HWINV:BEGIN", "HWINV:FAIL:FIXTURE"), None),
    # Loader M1:FAIL before ExitBootServices, on the unmodified production build.
    "missing-kernel": ("native-hw-inventory", ("M1:FAIL:KERNEL_OPEN",), "kernel.elf"),
}
REQUIRED = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "M2:PASS", "HWINV:BEGIN", "HWINV:PASS")
TIMEOUT_S = 120  # protocol HW-INV step S4


def qmp_functions(buses: list) -> set[tuple[str, str, str, str, str]]:
    """(address, vendor, device, class, subclass) of every function in a query-pci reply."""
    found = set()

    def walk(devices: list) -> None:
        for entry in devices:
            klass = entry["class_info"]["class"]
            address = f"0000:{entry['bus']:02x}:{entry['slot']:02x}.{entry['function']:x}"
            found.add((address, f"{entry['id']['vendor']:04x}", f"{entry['id']['device']:04x}",
                       f"{klass >> 8:02x}", f"{klass & 0xff:02x}"))
            bridge = entry.get("pci_bridge")
            if bridge is not None:
                walk(bridge.get("devices", []))

    for bus in buses:
        walk(bus["devices"])
    return found


def evaluate(text: str, case: str, qmp_pci: set | None, alive_at_pass: bool) -> tuple[dict, dict | None, str | None]:
    """Return (checks, inventory, error) for one captured serial log."""
    spec = CASES[case]
    lines = [line for line in text.replace("\r", "").splitlines()]
    checks: dict[str, bool] = {}
    positions = [next((i for i, line in enumerate(lines) if line == marker), -1) for marker in REQUIRED]
    checks["required_markers_in_order"] = -1 not in positions and positions == sorted(positions)
    checks["single_ebs"] = sum("BOOT:EBS_OK" in line for line in lines) == 1
    checks["no_failure_marker"] = not any(marker in line for line in lines for marker in FAILURES)
    checks["kernel_acpi_marker"] = spec["acpi"] in lines
    checks["halted_without_exit_port"] = alive_at_pass
    try:
        inventory = inventory_from_log(text)
    except InventoryLogError as error:
        return checks | {"inventory_converted": False}, None, str(error)
    checks["inventory_converted"] = True
    block = hwinv_lines(text)
    checks["cpuid_matches_configuration"] = block[1] == (
        f"HWINV:CPUID:VENDOR={spec['vendor']}:HYPERVISOR={spec['hypervisor']}")
    checks["secure_boot_not_claimed"] = inventory["secure_boot"] in ("disabled", "unknown")
    checks["dmar_matches_configuration"] = block[3] == spec["dmar"]
    checks["log_binding"] = pci_log_problem(lines, inventory["pci"], required=True) is None
    ours = {(f["address"], f["vendor"], f["device"], f["class"], f["subclass"]) for f in inventory["pci"]}
    checks["pci_equals_qmp_query_pci"] = qmp_pci is not None and ours == qmp_pci
    try:
        _inventory(json.dumps(inventory).encode())
        checks["rejected_as_virtual_capture"] = False
    except EvidenceError as error:
        checks["rejected_as_virtual_capture"] = str(error).startswith("virtual capture")
    return checks, inventory, None


def evaluate_fixture(text: str, fixture: str, alive: bool) -> dict:
    """Checks for one terminal-fixture log: markers in order, failure seen, no exit port."""
    lines = text.replace("\r", "").splitlines()
    positions = [next((i for i, line in enumerate(lines) if line.startswith(marker)), -1)
                 for marker in FIXTURES[fixture][1]]
    try:
        inventory_from_log(text)
        rejected = False
    except InventoryLogError as error:
        rejected = str(error).startswith("log reports ")
    return {"fixture_markers_in_order": -1 not in positions and positions == sorted(positions),
            "no_hwinv_pass": "HWINV:PASS" not in lines,
            "protocol_failure_recognised": rejected,
            "halted_without_exit_port": alive}


def run_case(case: str, *, build_directory: Path = BUILD, run_root: Path | None = None,
             machine: str | None = None) -> dict:
    if case not in CASES and case not in FIXTURES:
        raise ValueError("Unknown HW-INV case")
    feature, _, removed = FIXTURES.get(case, ("native-hw-inventory", (), None))
    machine = qemu_machine(machine)
    build_directory = Path(build_directory).resolve(strict=True)
    run_root = (build_directory / "run") if run_root is None else Path(run_root).resolve()
    frozen = source_hash()
    manifest = json.loads((build_directory / "manifest.json").read_text())
    if (manifest.get("kernel_feature") != feature
            or manifest.get("loader_feature") != "hw-inventory"
            or manifest.get("source_sha256") != frozen):
        raise ValueError(f"Build the paired {feature} profile on the current source first")
    directory = run_root / f"hwinv-{case}-{uuid.uuid4().hex[:10]}"
    if len(os.fsencode(directory / "qmp.sock")) >= 104:
        raise ValueError("Private QMP socket path too long; choose a short --run-root")
    directory.mkdir(parents=True)
    shutil.copytree(build_directory / "esp", directory / "esp")
    if (sha256(directory / "esp/kernel.elf") != manifest["kernel"]["sha256"]
            or sha256(directory / "esp/EFI/BOOT/BOOTX64.EFI") != manifest["loader"]["sha256"]):
        raise ValueError("HW-INV build artifacts changed")
    if removed is not None:
        (directory / "esp" / removed).unlink()
    save_json(directory / "build-manifest.json", manifest)
    code, variables = firmware()
    for source, name in ((code, "OVMF_CODE.fd"), (variables, "OVMF_VARS.fd")):
        shutil.copy2(source, directory / name)
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if qemu is None:
        raise RuntimeError("Missing QEMU")
    spec = CASES.get(case, CASES["plain"])
    arguments = [qemu, "-machine", machine, "-accel", "tcg,thread=single", "-cpu", spec["cpu"],
                 "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
                 "-serial", "stdio", "-nic", "none", "-no-reboot", *spec["devices"],
                 "-qmp", "unix:qmp.sock,server=on,wait=off",
                 "-drive", "if=pflash,unit=0,format=raw,readonly=on,file=OVMF_CODE.fd",
                 "-drive", "if=pflash,unit=1,format=raw,file=OVMF_VARS.fd",
                 "-drive", "if=virtio,index=0,format=raw,readonly=on,file=fat:esp",
                 # Present only so an unexpected exit-port write ends QEMU and fails the case.
                 "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
    start = time.monotonic()
    deadline = start + TIMEOUT_S
    failure = None
    qmp_pci = None
    alive_at_pass = False
    trace: list = []
    rc = None
    with (directory / "serial.txt").open("wb") as stdout, (directory / "stderr.txt").open("wb") as stderr:
        process = subprocess.Popen(arguments, cwd=directory, env=env, stdout=stdout,
                                   stdin=subprocess.DEVNULL, stderr=stderr, bufsize=0)
        control = None
        try:
            control = Qmp(directory / "qmp.sock", deadline, process)
            while True:
                lines = (directory / "serial.txt").read_bytes().decode("utf-8", "replace").splitlines()
                if any(line.rstrip("\r") == "HWINV:PASS" or any(m in line for m in FAILURES) for line in lines):
                    break
                if time.monotonic() >= deadline or process.poll() is not None:
                    raise RuntimeError("HW-INV did not reach HWINV:PASS or a failure marker")
                time.sleep(0.05)
            time.sleep(1.0)  # a halted guest stays up; an exit-port write would end QEMU here
            alive_at_pass = process.poll() is None
            if alive_at_pass:
                if case in CASES:
                    qmp_pci = qmp_functions(control.command("query-pci"))
                control.command("quit")
            rc = process.wait(timeout=max(0.1, deadline - time.monotonic()))
        except Exception as error:  # recorded in the report; the case fails
            failure = f"{type(error).__name__}: {error}"
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=10)
            rc = process.returncode if rc is None else rc
            if control is not None:
                trace = control.trace
                control.close()
    text = (directory / "serial.txt").read_bytes().decode("utf-8", "replace")
    if case in CASES:
        checks, inventory, error = evaluate(text, case, qmp_pci, alive_at_pass)
    else:
        checks, inventory, error = evaluate_fixture(text, case, alive_at_pass), None, None
    checks["source_unchanged"] = source_hash() == frozen
    checks["clean_exit_after_quit"] = rc == 0  # QMP quit ends QEMU with 0; crash/signal/exit port do not
    if inventory is not None:
        save_json(directory / "inventory.json", inventory)
    save_json(directory / "qmp-trace.json", trace)
    report = dict(
        scope="QEMU HW-INV inventory by the own IF0 kernel; not physical hardware or a hardware record",
        case=case, kernel_feature=feature, removed_esp_file=removed, passed=failure is None and error is None and all(checks.values()), checks=checks,
        error=failure, inventory_error=error, qmp_pci=sorted(qmp_pci) if qmp_pci is not None else None,
        command=arguments, machine=machine, raw_qemu_exit=rc, duration_s=time.monotonic() - start,
        source_fingerprint_version=3, source_sha256=frozen,
        base_commit=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        qemu_version=subprocess.check_output([qemu, "--version"], env=env, text=True).strip(),
        firmware=dict(code=str(code), code_sha256=sha256(code)), run_directory=str(directory), stdout=text,
        artifacts={p.relative_to(directory).as_posix(): sha256(p)
                   for p in directory.rglob("*") if p.is_file() and "esp" not in p.parts})
    save_json(directory / "result.json", report)
    save_json(build_directory / f"hw-inventory-{case}-result.json", report)
    print(text, end="")
    print(f"HW-INV {case}: passed={report['passed']} raw={rc}; {directory}")
    for name, ok in checks.items():
        if not ok:
            print("  failed check:", name)
    if failure or error:
        print(" ", failure or error)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=("all", *CASES, *FIXTURES), default="all",
                        help="all = the inventory cases; each fixture needs its own build")
    parser.add_argument("--run-root", type=Path, default=None)
    parser.add_argument("--machine", default=None)
    args = parser.parse_args()
    selected = tuple(CASES) if args.case == "all" else (args.case,)
    reports = [run_case(case, run_root=args.run_root, machine=args.machine) for case in selected]
    raise SystemExit(0 if all(report["passed"] for report in reports) else 1)
