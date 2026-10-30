#!/usr/bin/env python3
"""Verify real read-only CMOS UTC through the own-kernel native-rtc profile."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import json
import re
from common import BUILD, save_json, sha256, source_hash
from run import run_guest, is_success

CASES = {
    "epoch": "1970-01-01T00:00:00",
    "leap": "2000-02-29T12:34:56",
    "current": "2026-10-30T12:00:00",
    "y2038": "2038-01-19T03:14:08",
    "century": "2100-03-01T00:00:00",
}
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK",
        "KS-01:PASS", "KS-02:PASS", "KS-03:PASS")
END = ("RTC:QUALITY:UNAUTHENTICATED", "RTC:NEGATIVE:PASS",
       "RTC:RESTORE:PASS", "RTC:PASS", "M2:PASS", "M1:PASS")
REQUIRED = BOOT + END
FORBIDDEN = ("FAIL", "PANIC")


def evaluate(stdout: str, raw_exit: int | None, timed_out: bool, base: str) -> dict:
    if not is_success(raw_exit, stdout, timed_out, required_markers=REQUIRED,
                      forbidden_markers=FORBIDDEN):
        raise ValueError("Native RTC boot/negative/restore/exit assertions failed")
    lines = stdout.splitlines()
    observed = lines[lines.index(BOOT[0]):]
    if tuple(observed[:len(BOOT)]) != BOOT or tuple(observed[len(BOOT) + 2:]) != END:
        raise ValueError("Unexpected RTC sequence or duplicate output")
    utc_match = re.fullmatch(r"RTC:UTC:(-?[0-9]+)", observed[len(BOOT)])
    date_match = re.fullmatch(r"RTC:DATE:([0-9]+(?:,[0-9]+){5})", observed[len(BOOT) + 1])
    if utc_match is None or date_match is None:
        raise ValueError("Malformed native calendar output")
    utc = int(utc_match[1])
    date = datetime(*map(int, date_match[1].split(",")), tzinfo=timezone.utc)
    if int(date.timestamp()) != utc:
        raise ValueError("Independent calendar oracle disagrees with kernel conversion")
    start = int(datetime.strptime(base, "%Y-%m-%dT%H:%M:%S")
                .replace(tzinfo=timezone.utc).timestamp())
    if not start <= utc <= start + 60:
        raise ValueError("RTC does not match the controlled VM calendar within 60 seconds")
    return {"unix_utc": utc, "date_utc": date.isoformat(),
            "base_utc": base, "quality": "UnauthenticatedRtc"}


def run(cases: tuple[str, ...]) -> dict:
    destination = BUILD / "rtc-report.json"
    destination.unlink(missing_ok=True)
    frozen = source_hash()
    manifest_path = BUILD / "manifest.json"
    manifest_hash = sha256(manifest_path)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if (manifest.get("kernel_feature") != "native-rtc"
            or manifest.get("loader_feature") is not None
            or manifest.get("source_fingerprint_version") != 3
            or manifest.get("source_sha256") != frozen):
        raise ValueError("Build the current native-rtc profile first")
    report = {"schema_version": 1, "passed": False, "source_sha256": frozen,
              "source_fingerprint_version": 3, "manifest_sha256": manifest_hash,
              "scope": "IF0 own-kernel CMOS reads; not a userspace TimeProvider or authenticated time",
              "cases": []}
    save_json(destination, report)
    for case in cases:
        if source_hash() != frozen or sha256(manifest_path) != manifest_hash:
            raise ValueError("RTC source or manifest changed before a run")
        guest = run_guest(
            "rtc-" + case, timeout=60, required_markers=REQUIRED,
            forbidden_markers=FORBIDDEN, rtc_base=CASES[case],
            expected_kernel_sha256=manifest["kernel"]["sha256"],
            expected_loader_sha256=manifest["loader"]["sha256"],
        )
        item = {"case": case, "guest": guest, "passed": False}
        report["cases"].append(item)
        save_json(destination, report)
        item["observed"] = evaluate(guest["stdout"], guest["raw_qemu_exit"],
                                    guest["timed_out"], CASES[case])
        if (guest.get("passed") is not True
                or guest.get("source_sha256") != frozen
                or source_hash() != frozen or sha256(manifest_path) != manifest_hash):
            raise ValueError("RTC source, artifacts or manifest changed during a run")
        item["passed"] = True
        save_json(destination, report)
    report["passed"] = bool(cases) and all(item["passed"] for item in report["cases"])
    save_json(destination, report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=("all", *CASES), default="all")
    args = parser.parse_args()
    selected = tuple(CASES) if args.case == "all" else (args.case,)
    raise SystemExit(0 if run(selected)["passed"] else 1)
