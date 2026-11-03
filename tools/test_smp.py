#!/usr/bin/env python3
"""Run host evidence tests and native 1/2/4-CPU QEMU SMP rendezvous probes."""
from __future__ import annotations

from datetime import datetime, timezone
import os
from pathlib import Path
import subprocess
import sys
from zoneinfo import ZoneInfo

from build import build
from common import BUILD, ROOT, save_json, source_hash
from run import MILESTONE_MARKERS, run_guest
from run_smp import run_smp


def main() -> int:
    source = source_hash()
    now = datetime.now(timezone.utc)
    local_date = now.astimezone(ZoneInfo("Europe/Amsterdam")).date().isoformat()
    stamp = now.strftime("%Y%m%dT%H%M%SZ")
    receipt = ROOT / "docs/verification/sm-smp" / f"smp-rendezvous-{stamp}.json"
    summary = {
        "schema_version": 1,
        "task": "SYSTEM-MATURITY-SMP AP bring-up and rendezvous checkpoint",
        "source_fingerprint_version": 3,
        "source_sha256": source,
        "source_commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
        ).strip(),
        "branch": subprocess.check_output(
            ["git", "branch", "--show-current"], cwd=ROOT, text=True
        ).strip(),
        "date_europe_amsterdam": local_date,
        "status": "running",
        "passed": False,
        "host": None,
        "stages": [],
        "acceptance_boundary": (
            "AP startup and bounded all-CPU atomic rendezvous/worker smoke only; not full "
            "K06 scheduling, IPC, revoke, exit, cross-CPU TLB, or race acceptance."
        ),
    }
    receipt.parent.mkdir(parents=True, exist_ok=True)
    save_json(receipt, summary)
    command = [
        sys.executable, "-B", "-m", "unittest", "discover",
        "-s", "tools/tests", "-p", "test_smp.py",
    ]
    env = dict(os.environ, PYTHONPATH=str(ROOT / "tools"))
    host = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    summary["host"] = {
        "command": command,
        "exit": host.returncode,
        "stdout": host.stdout,
        "stderr": host.stderr,
    }
    save_json(receipt, summary)
    if host.returncode:
        summary["status"] = "failed"
        save_json(receipt, summary)
        return host.returncode

    stages = (
        ("single", "milestone-m2", "single"),
        ("two", "native-smp", 2),
        ("four", "native-smp", 4),
    )
    for label, feature, cpus in stages:
        try:
            build(None, feature, f"smp-{label}")
            if cpus == "single":
                report = run_guest(
                    label="smp-single",
                    required_markers=MILESTONE_MARKERS["m2"],
                    memory_mib=256,
                )
            else:
                report = run_smp(cpus)
            summary["stages"].append({
                "profile": label,
                "kernel_feature": feature,
                "passed": report["passed"],
                "raw_qemu_exit": report.get("raw_qemu_exit"),
                "duration_s": report.get("duration_s"),
                "run_directory": report.get("run_directory"),
                "evidence": report.get("evidence"),
                "error": report.get("error"),
            })
            summary["passed"] = report["passed"] and source_hash() == source
            save_json(receipt, summary)
            if not summary["passed"]:
                summary["status"] = "failed"
                save_json(receipt, summary)
                return 1
        except Exception as error:
            summary["status"] = "failed"
            summary["error"] = f"{type(error).__name__}: {error}"
            save_json(receipt, summary)
            raise

    summary["status"] = "completed"
    summary["passed"] = source_hash() == source and len(summary["stages"]) == 3
    save_json(receipt, summary)
    print(f"SMP rendezvous passed={summary['passed']} stages={len(summary['stages'])} {receipt}")
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
