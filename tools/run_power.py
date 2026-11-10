#!/usr/bin/env python3
"""Verify the own-kernel ACPI S5 poweroff (native-acpi-s5) in the fixed QEMU profile.

Oracle: QEMU exits 0 only on guest poweroff, because the run overrides
-no-reboot with `-action reboot=reset` (a reset re-runs firmware and prints a
second EBS marker). The expected PM1a port is the pinned q35 ICH9 PMIO
base 0x600 + 4, independent of the kernel's own table parsing.
"""
from __future__ import annotations
import argparse
import json
from common import BUILD, save_json, sha256, source_hash, without_acpi_diagnostics
from run import run_guest

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK",
        "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS", "M1:PASS")
CASES = {
    # kernel feature -> (expected raw exit, exact serial tail after BOOT)
    "native-acpi-s5": (0, ("POWER:S5:PM1A=0x0604:SLP_TYP=0:SCI_EN=1", "POWER:S5:WRITE")),
    "test-acpi-s5-corrupt-dsdt": (33, ("POWER:S5:REJECTED:InvalidChecksum",)),
}


def evaluate(stdout: str, raw_exit: int | None, timed_out: bool, feature: str) -> list[str]:
    expected_exit, tail = CASES[feature]
    if timed_out or raw_exit != expected_exit:
        raise ValueError(f"QEMU exit {raw_exit} (timed_out={timed_out}), expected {expected_exit}")
    lines = without_acpi_diagnostics(stdout).splitlines()
    if lines.count(BOOT[0]) != 1:
        raise ValueError("Missing or repeated EBS handoff (reset or reboot)")
    start = lines.index(BOOT[0])
    if any("POWER:" in line for line in lines[:start]):
        raise ValueError("POWER marker before EBS")
    if tuple(lines[start:]) != BOOT + tail:
        raise ValueError(f"Unexpected S5 serial sequence: {lines[start:]}")
    return list(tail)


def run(repeat: int) -> dict:
    manifest_path = BUILD / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    feature = manifest.get("kernel_feature")
    frozen, manifest_hash = source_hash(), sha256(manifest_path)
    if (feature not in CASES or manifest.get("loader_feature") is not None
            or manifest.get("source_fingerprint_version") != 3
            or manifest.get("source_sha256") != frozen):
        raise ValueError("Build the current native-acpi-s5 or test-acpi-s5-* profile first")
    destination = BUILD / "power-report.json"
    report = {"schema_version": 1, "passed": False, "kernel_feature": feature,
              "source_sha256": frozen, "source_fingerprint_version": 3,
              "manifest_sha256": manifest_hash,
              "scope": "QEMU q35 ACPI S5 via own FADT/DSDT parse and PM1 write; "
                       "no sleep/wake, SMI_CMD ACPI enable, thermal, battery or physical hardware",
              "runs": []}
    save_json(destination, report)
    for index in range(repeat):
        guest = run_guest(
            f"power-{feature}-{index}", timeout=60, required_markers=BOOT,
            expected_exit=CASES[feature][0], reboot_on_reset=True,
            expected_kernel_sha256=manifest["kernel"]["sha256"],
            expected_loader_sha256=manifest["loader"]["sha256"],
        )
        item = {"guest": guest, "passed": False}
        report["runs"].append(item)
        save_json(destination, report)
        item["observed"] = evaluate(guest["stdout"], guest["raw_qemu_exit"],
                                    guest["timed_out"], feature)
        if (guest.get("passed") is not True
                or guest.get("artifact_binding_passed") is not True
                or "reboot=reset" not in guest["command"]
                or source_hash() != frozen or sha256(manifest_path) != manifest_hash):
            raise ValueError("S5 source, artifacts, manifest or reset oracle changed during a run")
        item["passed"] = True
        save_json(destination, report)
    report["passed"] = repeat > 0 and all(item["passed"] for item in report["runs"])
    save_json(destination, report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repeat", type=int, default=5)
    args = parser.parse_args()
    raise SystemExit(0 if run(args.repeat)["passed"] else 1)
