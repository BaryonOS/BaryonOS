#!/usr/bin/env python3
"""Run the A03 task-grant broker with and without the fixture grant."""
from __future__ import annotations
import argparse
import json
import re
from common import BUILD, save_json, source_hash, with_acpi_markers, without_acpi_diagnostics
from run import run_guest

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS")
NORMAL = ("TASKGRANT:OWNER_READ:PASS", "TASKGRANT:OWNER_LIMITS:PASS",
          "TASKGRANT:HELPER_DENIED:PASS", "TASKGRANT:CLOSE_REVOKES:PASS")
DENIED = ("TASKGRANT:DENIED:PASS", "TASKGRANT:HELPER_DENIED:PASS")
END = ("TASKGRANT:CLEANUP:PASS", "TASKGRANT:PASS", "M1:PASS")
OWNER = "TASKGRANT:OWNER_COUNTER:"
HELPER = "TASKGRANT:HELPER_COUNTER:"

def number(line: str, label: str) -> int:
    match = re.fullmatch(re.escape(label) + r"([0-9]{20})", line)
    if not match:
        raise ValueError(f"malformed numeric evidence: {label}")
    return int(match.group(1))

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=("normal", "denied"), required=True)
    args = parser.parse_args()
    feature = "native-task-grant" if args.case == "normal" else "test-task-grant-denied"
    frozen = source_hash()
    manifest = json.loads((BUILD / "manifest.json").read_text())
    if manifest.get("kernel_feature") != feature or manifest.get("source_sha256") != frozen:
        raise ValueError("build the selected current task-grant profile first")
    expected_cases = NORMAL if args.case == "normal" else DENIED
    required = BOOT + ("TASKGRANT:MANIFEST:PASS",) + expected_cases + END
    report = {
        "schema_version": 1,
        "case": args.case,
        "source_fingerprint_version": 3,
        "source_sha256": frozen,
        "passed": False,
        "error": "run did not complete",
    }
    out = BUILD / f"native-task-grant-{args.case}.json"
    # Replace any earlier success before the guest can fail to start.
    save_json(out, report)
    try:
        guest = run_guest(
            "native-task-grant-" + args.case,
            esp=BUILD / "esp",
            timeout=60,
            required_markers=with_acpi_markers(required),
            expected_exit=33,
            forbidden_markers=("FAIL", "PANIC"),
            expected_kernel_sha256=manifest["kernel"]["sha256"],
            expected_loader_sha256=manifest["loader"]["sha256"],
        )
        report["guest"] = guest
        if guest.get("passed") is not True or guest.get("raw_qemu_exit") != 33:
            raise ValueError("guest did not pass")
        lines = without_acpi_diagnostics(guest["stdout"]).splitlines()
        start = lines.index(BOOT[0])
        tail = lines[start:]
        owner_counter = number(next(line for line in tail if line.startswith(OWNER)), OWNER)
        helper_counter = number(next(line for line in tail if line.startswith(HELPER)), HELPER)
        structural = tuple(
            line for line in tail if not line.startswith(OWNER) and not line.startswith(HELPER)
        )
        if structural != required:
            raise ValueError("task-grant evidence sequence mismatch")
        # Counters are scheduler progress only. Authorization outcomes are
        # proven inside the kernel observer via the broker's exact decision log.
        if source_hash() != frozen:
            raise ValueError("source changed during task-grant run")
        report.update(passed=True, owner_counter=owner_counter, helper_counter=helper_counter)
        del report["error"]
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(out, report)
    print(f"Native task grant {args.case} accepted={report['passed']}")
    if "error" in report:
        print(report["error"])
    return 0 if report["passed"] else 1

if __name__ == "__main__":
    raise SystemExit(main())
