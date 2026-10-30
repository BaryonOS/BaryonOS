#!/usr/bin/env python3
"""Verify the native ring-3 UTC clock API against a controlled disposable VM RTC."""
from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import json
import re
from build import build
from common import BUILD, save_json, source_hash
from run_system import run_system

FEATURE = "native-system-clock"
RTC_BASE = "2026-10-30T12:00:00"


def evaluate(stdout: str, base: str) -> dict[str, int]:
    found = {}
    for label in ("UTC_FIRST", "UTC_SECOND", "MONOTONIC_BEFORE", "MONOTONIC_AFTER"):
        matches = re.findall(rf"^SYSTEM:CLOCK:{label}:([0-9]{{20}})$", stdout, re.M)
        if len(matches) != 1:
            raise ValueError(f"expected exactly one {label} clock observation")
        found[label] = int(matches[0])
    if stdout.splitlines().count("SYSTEM:CLOCK:INVALID_ARGS:PASS") != 1:
        raise ValueError("invalid-argument syscall case did not pass exactly once")
    first, second = found["UTC_FIRST"], found["UTC_SECOND"]
    before, after = found["MONOTONIC_BEFORE"], found["MONOTONIC_AFTER"]
    expected = int(datetime.strptime(base, "%Y-%m-%dT%H:%M:%S")
                    .replace(tzinfo=timezone.utc).timestamp())
    if not expected <= first <= second <= expected + 60:
        raise ValueError("ring-3 UTC readings disagree with the controlled guest RTC")
    if after < before:
        raise ValueError("boot-monotonic ticks regressed around UTC reads")
    return found


def run() -> dict:
    destination = BUILD / "system-clock-report.json"
    destination.unlink(missing_ok=True)
    frozen = source_hash()
    report = {
        "schema_version": 1,
        "passed": False,
        "feature": FEATURE,
        "rtc_base_utc": RTC_BASE,
        "source_sha256": frozen,
        "scope": "ring-3 RTC syscall and SDK reading type; unauthenticated, not certificate time",
    }
    save_json(destination, report)
    try:
        manifest = build(None, FEATURE, FEATURE)
        if source_hash() != frozen or manifest.get("kernel_feature") != FEATURE:
            raise ValueError("build source changed or did not produce the native clock profile")
        guest, volume = run_system(
            "normal",
            kernel_feature=FEATURE,
            rtc_base=RTC_BASE,
        )
        report["guest"] = {
            key: guest[key]
            for key in ("passed", "run_directory", "raw_qemu_exit", "timed_out",
                        "kernel_feature", "manifest", "firmware_sha256")
        }
        report["volume_directory"] = str(volume)
        if not guest.get("passed"):
            raise ValueError(guest.get("error", "native system guest failed"))
        stdout = (Path(guest["run_directory"]) / "serial.txt").read_text(encoding="utf-8")
        report["observed"] = evaluate(stdout, RTC_BASE)
        report["passed"] = source_hash() == frozen
        if not report["passed"]:
            raise ValueError("source changed during the native wall-clock run")
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
        save_json(destination, report)
        raise
    save_json(destination, report)
    return report


if __name__ == "__main__":
    print(json.dumps(run(), sort_keys=True))
