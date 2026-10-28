#!/usr/bin/env python3
"""Run the ordinary SDK catalog app with normal or denied authority."""
from __future__ import annotations
import argparse
import json
import re
from common import BUILD, save_json, source_hash
from run import run_guest

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS")
NORMAL = ("SDKAPP:READ:PASS", "SDKAPP:SANDBOX:PASS", "SDKAPP:EXIT:PASS")
DENIED = ("SDKAPP:DENIED:PASS", "SDKAPP:ERROR_EXIT:PASS")
END = ("SDKAPP:CLEANUP:PASS", "SDKAPP:PASS", "M1:PASS")

def number(line: str, label: str) -> int:
    match = re.fullmatch(re.escape(label) + r"([0-9]{20})", line)
    if not match:
        raise ValueError(f"malformed numeric evidence: {label}")
    return int(match.group(1))

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=("normal", "denied"), required=True)
    args = parser.parse_args()
    feature = "native-sdk-app" if args.case == "normal" else "test-sdk-app-denied"
    frozen = source_hash()
    manifest = json.loads((BUILD / "manifest.json").read_text())
    if manifest.get("kernel_feature") != feature or manifest.get("source_sha256") != frozen:
        raise ValueError("build the selected current SDK app profile first")
    expected_cases = NORMAL if args.case == "normal" else DENIED
    required = BOOT + ("SDKAPP:MANIFEST:PASS",) + expected_cases + END
    guest = run_guest(
        "native-sdk-app-" + args.case,
        esp=BUILD / "esp",
        timeout=60,
        required_markers=required,
        expected_exit=33,
        forbidden_markers=("FAIL", "PANIC"),
        expected_kernel_sha256=manifest["kernel"]["sha256"],
        expected_loader_sha256=manifest["loader"]["sha256"],
    )
    report = {
        "schema_version": 1,
        "case": args.case,
        "source_fingerprint_version": 3,
        "source_sha256": frozen,
        "passed": False,
        "guest": guest,
    }
    try:
        if guest.get("passed") is not True or guest.get("raw_qemu_exit") != 33:
            raise ValueError("guest did not pass")
        lines = guest["stdout"].splitlines()
        start = lines.index(BOOT[0])
        tail = lines[start:]
        app_line = next(line for line in tail if line.startswith("SDKAPP:APP_COUNTER:"))
        service_line = next(line for line in tail if line.startswith("SDKAPP:SERVICE_COUNTER:"))
        app_counter = number(app_line, "SDKAPP:APP_COUNTER:")
        service_counter = number(service_line, "SDKAPP:SERVICE_COUNTER:")
        structural = tuple(
            line for line in tail
            if not line.startswith("SDKAPP:APP_COUNTER:")
            and not line.startswith("SDKAPP:SERVICE_COUNTER:")
        )
        if structural != required:
            raise ValueError("SDK app evidence sequence mismatch")
        # The service counter is scheduler progress only. Request/reply activity
        # is proven inside the kernel observer via the service's exact counters.
        if source_hash() != frozen:
            raise ValueError("source changed during SDK app run")
        report.update(passed=True, app_counter=app_counter, service_counter=service_counter)
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(BUILD / f"native-sdk-app-{args.case}.json", report)
    print(f"Native SDK app {args.case} accepted={report['passed']}")
    if "error" in report:
        print(report["error"])
    return 0 if report["passed"] else 1

if __name__ == "__main__":
    raise SystemExit(main())
