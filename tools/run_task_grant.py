#!/usr/bin/env python3
"""Run the A03 task-grant broker with and without the fixture grant.

Cases `volume`/`volume-denied` (A03-T2c-2) run the broker as sole client of a
read-only volume domain 0 over a fresh `seeded-read` image attached writable;
the image must be byte-identical to a fresh seed afterwards."""
from __future__ import annotations
import argparse
import hashlib
import json
import re
import tempfile
from pathlib import Path
from common import BUILD, save_json, source_hash, with_acpi_markers, without_acpi_diagnostics
from run import run_guest
import volume_disk

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS")
NORMAL = ("TASKGRANT:OWNER_READ:PASS", "TASKGRANT:OWNER_LIMITS:PASS",
          "TASKGRANT:HELPER_DENIED:PASS", "TASKGRANT:CLOSE_REVOKES:PASS",
          "TASKGRANT:FORGED_RETIRE_DENIED:PASS", "TASKGRANT:EXIT_RETIRES:PASS")
DENIED = ("TASKGRANT:DENIED:PASS", "TASKGRANT:HELPER_DENIED:PASS")
END = ("TASKGRANT:CLEANUP:PASS", "TASKGRANT:PASS", "M1:PASS")
VOLUME_START = ("TASKGRANT:VOLUME_READ_ONLY:PASS", "TASKGRANT:NO_VOLUME_HANDLE:PASS")
VOLUME_NORMAL = ("TASKGRANT:VOLUME_READ_OK:PASS", "TASKGRANT:OWNER_READ:PASS",
                 "TASKGRANT:OWNER_LIMITS:PASS", "TASKGRANT:UNGRANTED_OBJECT_DENIED:PASS",
                 "TASKGRANT:FORGED_ACTOR_DENIED:PASS", "TASKGRANT:HELPER_DENIED:PASS",
                 "TASKGRANT:CLOSE_REVOKES:PASS", "TASKGRANT:FORGED_RETIRE_DENIED:PASS",
                 "TASKGRANT:EXIT_RETIRES:PASS", "TASKGRANT:READ_AFTER_RETIREMENT_DENIED:PASS")
FEATURES = {"normal": "native-task-grant", "denied": "test-task-grant-denied",
            "volume": "native-task-grant-volume", "volume-denied": "test-task-grant-volume-denied"}
NUMERIC = re.compile(r"TASKGRANT:[A-Z0-9_]+:[0-9]{20}")
OWNER = "TASKGRANT:OWNER_COUNTER:"
HELPER = "TASKGRANT:HELPER_COUNTER:"

def number(line: str, label: str) -> int:
    match = re.fullmatch(re.escape(label) + r"([0-9]{20})", line)
    if not match:
        raise ValueError(f"malformed numeric evidence: {label}")
    return int(match.group(1))

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=tuple(FEATURES), required=True)
    args = parser.parse_args()
    feature = FEATURES[args.case]
    volume = args.case.startswith("volume")
    frozen = source_hash()
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
    manifest = json.loads((BUILD / "manifest.json").read_text())
    if manifest.get("kernel_feature") != feature or manifest.get("source_sha256") != frozen:
        raise ValueError("build the selected current task-grant profile first")
    if volume:
        cases = VOLUME_NORMAL if args.case == "volume" else DENIED
        case_marker = "TASKGRANT:CASE:VOLUME:PASS" if args.case == "volume" else "TASKGRANT:CASE:VOLUME_DENIED:PASS"
        required = (BOOT + VOLUME_START + ("TASKGRANT:MANIFEST:PASS",) + cases
                    + ("TASKGRANT:DENIED_NO_VOLUME_REQUEST:PASS", "TASKGRANT:CLEANUP:PASS", case_marker)
                    + END[1:])
    else:
        expected_cases = NORMAL if args.case == "normal" else DENIED
        required = BOOT + ("TASKGRANT:MANIFEST:PASS",) + expected_cases + END
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
            volume_variant="seeded-read" if volume else None,
        )
        report["guest"] = guest
        if guest.get("passed") is not True or guest.get("raw_qemu_exit") != 33:
            raise ValueError("guest did not pass")
        lines = without_acpi_diagnostics(guest["stdout"]).splitlines()
        start = lines.index(BOOT[0])
        tail = lines[start:]
        numbers = {}
        for line in tail:
            if NUMERIC.fullmatch(line):
                label = line[:-20]
                if label in numbers:
                    raise ValueError(f"duplicate numeric evidence: {label}")
                numbers[label] = number(line, label)
        structural = tuple(line for line in tail if not NUMERIC.fullmatch(line))
        if structural != required:
            raise ValueError("task-grant evidence sequence mismatch")
        if volume:
            # Fresh seeds are byte-identical, so equality proves objects 0/1
            # (and everything else on the disk) were never written.
            with tempfile.TemporaryDirectory(dir=BUILD) as scratch:
                fresh = Path(scratch) / "fresh.img"
                volume_disk.create_image(fresh, "seeded-read")
                fresh_sha = hashlib.sha256(fresh.read_bytes()).hexdigest()
            if not (guest["volume_initial_sha256"] == guest["volume_final_sha256"] == fresh_sha):
                raise ValueError("seeded volume image changed or differs from a fresh seed")
            report.update(numbers=numbers, volume_sha256=fresh_sha)
        else:
            owner_counter = numbers[OWNER]
            helper_counter = numbers[HELPER]
        # Counters are scheduler progress only. Authorization outcomes are
        # proven inside the kernel observer via the broker's exact decision log.
        if source_hash() != frozen:
            raise ValueError("source changed during task-grant run")
        if volume:
            report.update(passed=True)
        else:
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
