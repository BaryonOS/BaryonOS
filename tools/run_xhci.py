#!/usr/bin/env python3
"""Verify the isolated native xHCI command-ring and VT-d DMA containment gate."""
from __future__ import annotations

import argparse
import json

from common import BUILD, save_json, sha256, source_hash
from run import is_success, run_guest

FEATURES = {"noop": "native-xhci-probe", "fault": "test-xhci-dma-fault", "no-dmar": "test-xhci-no-dmar"}
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS")
START = ("XHCI:BEGIN", "XHCI:PAGING:PASS", "XHCI:DOMAIN:PASS", "XHCI:BUS_MASTER:CONTAINED")
END = ("XHCI:GUARD:UNCHANGED", "XHCI:QUIESCE:PASS", "XHCI:PAGES:RETAINED", "XHCI:PASS")


def markers(case: str) -> tuple[str, ...]:
    if case not in FEATURES:
        raise ValueError("Unknown xHCI gate case")
    if case == "no-dmar":
        return ("XHCI:BEGIN", "XHCI:NO_DMAR:BUS_MASTER_BLOCKED", "XHCI:PASS")
    middle = "XHCI:NOOP_COMPLETION:PASS" if case == "noop" else "XHCI:DMA_FAULT:WRITE_DENIED"
    return START + (middle,) + END


def evaluate(case: str, stdout: str, raw_exit: int | None, timed_out: bool) -> dict:
    expected = markers(case)
    if not is_success(raw_exit, stdout, timed_out, required_markers=BOOT + expected,
                      forbidden_markers=("FAIL", "PANIC")):
        raise ValueError("xHCI native boot/result/exit assertions failed")
    lines = stdout.splitlines()
    if any(lines.count(marker) != 1 for marker in BOOT):
        raise ValueError("Repeated or absent native boot marker")
    observed = tuple(line for line in lines if line.startswith("XHCI:"))
    if observed != expected:
        raise ValueError("Wrong case, duplicate, unknown or reordered xHCI evidence")
    if case == "no-dmar" and "ACPI:UNAVAILABLE:NO_DMAR" not in lines:
        raise ValueError("Missing-DMAR case did not demonstrate absent firmware support")
    return {"markers": list(expected), "dma": case != "no-dmar", "usb_enumeration": False}


def run_case(case: str) -> dict:
    destination = BUILD / f"xhci-{case}-report.json"
    destination.unlink(missing_ok=True)
    frozen = source_hash()
    manifest_path = BUILD / "manifest.json"
    manifest_hash = sha256(manifest_path)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if (manifest.get("kernel_feature") != FEATURES[case]
            or manifest.get("loader_feature") is not None
            or manifest.get("source_fingerprint_version") != 3
            or manifest.get("source_sha256") != frozen):
        raise ValueError(f"Build the current {FEATURES[case]} profile first")
    report = {"schema_version": 1, "case": case, "passed": False,
              "source_sha256": frozen, "source_fingerprint_version": 3,
              "manifest_sha256": manifest_hash,
              "scope": "QEMU native pinned DMA/VT-d/xHCI command completion; no HID, IRQ or physical acceptance"}
    save_json(destination, report)
    guest = run_guest("xhci-" + case, timeout=60, required_markers=BOOT + markers(case),
                      forbidden_markers=("FAIL", "PANIC"), xhci_case=case,
                      expected_kernel_sha256=manifest["kernel"]["sha256"],
                      expected_loader_sha256=manifest["loader"]["sha256"])
    report["guest"] = guest
    save_json(destination, report)
    report["observed"] = evaluate(case, guest["stdout"], guest["raw_qemu_exit"], guest["timed_out"])
    if (not guest.get("passed") or not guest.get("artifact_binding_passed")
            or guest.get("source_sha256") != frozen or source_hash() != frozen
            or sha256(manifest_path) != manifest_hash):
        raise ValueError("xHCI source, manifest or executed artifact changed during verification")
    report["passed"] = True
    save_json(destination, report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=FEATURES, required=True)
    args = parser.parse_args()
    run_case(args.case)
