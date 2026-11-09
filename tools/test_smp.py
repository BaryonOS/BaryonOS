#!/usr/bin/env python3
"""Run host evidence tests and native 1/2/4-CPU SMP scheduling/IPC probes."""
from __future__ import annotations

from datetime import datetime, timezone
import os
from pathlib import Path
import shutil
import subprocess
import sys
from zoneinfo import ZoneInfo

from build import build
from common import BUILD, ROOT, save_json, source_hash
from run import MILESTONE_MARKERS, run_gate_case, run_guest
from run_smp_legacy import run_smp


def main() -> int:
    source = source_hash()
    now = datetime.now(timezone.utc)
    local_date = now.astimezone(ZoneInfo("Europe/Amsterdam")).date().isoformat()
    stamp = now.strftime("%Y%m%dT%H%M%SZ")
    receipt = ROOT / "docs/verification/sm-smp" / f"smp-rendezvous-{stamp}.json"
    summary = {
        "schema_version": 1,
        "task": "SYSTEM-MATURITY-SMP AP bring-up, M3 scheduling, process exit, and cross-CPU M4 IPC/revoke",
        "source_fingerprint_version": 3,
        "source_sha256": source,
        "source_commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
        ).strip(),
        "branch": subprocess.check_output(
            ["git", "branch", "--show-current"], cwd=ROOT, text=True
        ).strip(),
        "invocation": {
            "command": [sys.executable, "-B", "tools/test_smp.py"],
            "environment": {
                name: os.environ.get(name)
                for name in (
                    "LD_LIBRARY_PATH",
                    "OS_OVMF_CODE",
                    "OS_OVMF_VARS",
                )
            },
            "tool_paths": {
                name: shutil.which(command)
                for name, command in (
                    ("cargo", "cargo"),
                    ("rustc", "rustc"),
                    ("cc", "cc"),
                    ("qemu", "qemu-system-x86_64"),
                )
            },
        },
        "date_europe_amsterdam": local_date,
        "status": "running",
        "passed": False,
        "host": None,
        "stages": [],
        "acceptance_boundary": (
            "AP startup, bounded rendezvous, per-CPU M3 process scheduling, and successful "
            "process-exit cleanup on an AP plus cross-CPU M4 ring-3 IPC and revoked-capability "
            "rejection, including M4 quiescence before cleanup and repeated lifecycle resets; "
            "not full K06 remote TLB or general race acceptance."
        ),
    }
    receipt.parent.mkdir(parents=True, exist_ok=True)
    save_json(receipt, summary)
    command = [
        sys.executable, "-B", "-m", "unittest", "discover",
        "-s", "tools/tests", "-p", "test_smp_host.py",
    ]
    env = dict(os.environ, PYTHONPATH=str(ROOT / "tools"))
    try:
        host = subprocess.run(
            command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=600
        )
    except subprocess.TimeoutExpired:
        host_passed = False
        summary["host"] = {
            "command": command, "exit": None, "timeout_s": 600,
            "error": "host unittest process timed out",
        }
        summary["status"] = "failed"
        save_json(receipt, summary)
    else:
        host_passed = host.returncode == 0
        summary["host"] = {
            "command": command,
            "exit": host.returncode,
            "stdout": host.stdout,
            "stderr": host.stderr,
        }
        save_json(receipt, summary)
        if not host_passed:
            summary["status"] = "failed"
            save_json(receipt, summary)
            return host.returncode

    stages = (
        ("single-m2", "milestone-m2", 1, "m2"),
        ("two-m2", "native-smp-legacy", 2, "smp"),
        ("four-m2", "native-smp-legacy", 4, "smp"),
        ("m3-two", "native-smp-m3", 2, "m3-smp"),
        ("m3-exit-two", "native-smp-m3-exit", 2, "m3-exit-smp"),
        ("m3-lock-contention-two", "native-smp-m3-lock-contention", 2, "m3-smp"),
        ("m3-tlb-reuse-two", "native-smp-m3-tlb-reuse", 2, "m3-tlb-smp"),
        ("m3-x87-two", "native-smp-m3,test-m3-x87", 2, "native-smp-m3-x87"),
        ("m3-unexpected-ud-two", "native-smp-m3,test-m3-ud", 2, "native-smp-m3-unexpected-ud"),
        ("m3-single-regression", "milestone-m3", 1, "m3"),
        ("m3-four", "native-smp-m3", 4, "m3-smp"),
        ("m4-two", "native-smp-m4", 2, "m4-smp"),
        ("m4-two-repeat", "native-smp-m4", 2, "m4-smp"),
        ("m4-lifecycle-two", "native-smp-m4,test-m4-lifecycle", 2, "m4-lifecycle-smp"),
        ("m4-revoke-two", "native-smp-m4-revoke", 2, "m4-revoke-smp"),
    )
    for label, feature, cpus, runner in stages:
        try:
            build(None, feature, f"smp-{label}")
            if runner == "m2":
                report = run_guest(
                    label="smp-single",
                    required_markers=MILESTONE_MARKERS["m2"],
                    memory_mib=256,
                )
            elif runner == "smp":
                report = run_smp(cpus, profile="native-smp")
            elif runner == "m3":
                report = run_gate_case(
                    "m3",
                    "private-address-spaces-preemption-fairness-cleanup",
                    label="m3-single-smp-regression",
                )
            elif runner == "m4-lifecycle-smp":
                report = run_smp(cpus, profile="native-smp-m4-lifecycle")
            elif runner in ("native-smp-m3-x87", "native-smp-m3-unexpected-ud"):
                report = run_smp(cpus, profile=runner)
            else:
                report = run_smp(cpus, profile=feature)
            summary["stages"].append({
                "profile": label,
                "kernel_feature": feature,
                "cpu_count": cpus,
                "runner": runner,
                "passed": report["passed"],
                "raw_qemu_exit": report.get("raw_qemu_exit"),
                "duration_s": report.get("duration_s"),
                "run_directory": report.get("run_directory"),
                "evidence": report.get("evidence"),
                "error": report.get("error"),
            })
            stage_passed = report["passed"] and source_hash() == source
            # Only the completed, fully verified matrix may publish passed=True.
            save_json(receipt, summary)
            if not stage_passed:
                summary["status"] = "failed"
                save_json(receipt, summary)
                return 1
        except Exception as error:
            summary["status"] = "failed"
            summary["passed"] = False
            summary["failed_stage"] = label
            summary["error"] = f"{type(error).__name__}: {error}"
            save_json(receipt, summary)
            raise

    summary["status"] = "completed" if host_passed else "failed"
    summary["passed"] = (
        host_passed and source_hash() == source and len(summary["stages"]) == len(stages)
    )
    save_json(receipt, summary)
    print(f"SMP gates passed={summary['passed']} stages={len(summary['stages'])} {receipt}")
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
