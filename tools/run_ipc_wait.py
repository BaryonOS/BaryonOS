#!/usr/bin/env python3
"""Build and run real CPL3 IPC queue/deadline/idle acceptance in disposable QEMU."""
from __future__ import annotations

import json
import re

from build import build
from common import BUILD, save_json, source_hash
from run import run_guest

FEATURE = "native-ipc-wait"
MARKERS = (
    "IPCWAIT:MANIFEST:PASS",
    "IPCWAIT:NATIVE:PASS",
    "IPCWAIT:CLEANUP:PASS",
    "IPCWAIT:PASS",
    "M1:PASS",
)


def evaluate(stdout: str) -> dict[str, int]:
    result = {}
    for name in ("BLOCKED", "WAKE_READY", "WAKE_TIMEOUT", "WAKE_ERROR", "IDLE_IRQS"):
        matches = re.findall(rf"^IPCWAIT:{name}:([0-9]{{20}})$", stdout, re.M)
        if len(matches) != 1:
            raise ValueError(f"expected exactly one native {name} counter")
        result[name] = int(matches[0])
    for name in ("BLOCKED", "WAKE_READY", "WAKE_TIMEOUT", "IDLE_IRQS"):
        if result[name] == 0:
            raise ValueError(f"native {name} was not exercised")
    for marker in MARKERS:
        if stdout.splitlines().count(marker) != 1:
            raise ValueError(f"expected exactly one {marker}")
    return result


def run() -> dict:
    destination = BUILD / "ipc-wait-report.json"
    frozen = source_hash()
    report = {
        "schema_version": 1,
        "passed": False,
        "feature": FEATURE,
        "source_sha256": frozen,
        "scope": "two CPL3 processes, bounded queue waits and PIT idle; not GUI or release acceptance",
    }
    save_json(destination, report)
    try:
        manifest = build(None, FEATURE, FEATURE)
        if (manifest.get("kernel_feature") != FEATURE
                or manifest.get("source_sha256") != frozen or source_hash() != frozen):
            raise ValueError("native wait build profile/source changed")
        guest = run_guest(
            FEATURE, required_markers=MARKERS,
            forbidden_markers=("IPCWAIT:FAIL:", "PANIC", "M3:FAIL:"),
            expected_kernel_sha256=manifest["kernel"]["sha256"],
            expected_loader_sha256=manifest["loader"]["sha256"],
        )
        report["guest"] = guest
        if not guest["passed"]:
            raise ValueError("native wait guest failed; inspect retained serial/stderr")
        report["observed"] = evaluate(guest["stdout"])
        if source_hash() != frozen:
            raise ValueError("source changed during native wait execution")
        report["passed"] = True
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
        save_json(destination, report)
        raise
    save_json(destination, report)
    return report


if __name__ == "__main__":
    print(json.dumps(run(), sort_keys=True))
