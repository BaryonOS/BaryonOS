#!/usr/bin/env python3
"""S08 current-source native storage resilience acceptance."""
from __future__ import annotations

from datetime import datetime, timezone
import shutil
import subprocess

from build import build
from common import BUILD, ROOT, save_json, source_hash
from run_m6c import run_m6c

CASES = (
    "write-error",
    "flush-error",
    "write-timeout",
    "flush-timeout",
    "reset-timeout",
    "full",
    "corrupt",
)
RECOVERABLE = tuple(case for case in CASES if case != "corrupt")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def git_head() -> str:
    git = shutil.which("git")
    if git is None:
        raise RuntimeError("git executable not found")
    return subprocess.check_output(
        [git, "rev-parse", "HEAD"], cwd=ROOT, text=True, timeout=10
    ).strip()


def accepted_guest(report: dict, case: str, frozen: str, head: str) -> dict:
    require(report.get("passed") is True, f"{case}: guest/oracle did not pass")
    require(report.get("case") == case, f"{case}: wrong case record")
    require(
        report.get("source_sha256") == frozen
        and report.get("observed_source_sha256") == frozen
        and report.get("source_unchanged") is True,
        f"{case}: source binding failed",
    )
    require(report.get("base_commit") == head, f"{case}: HEAD binding failed")
    stdout = report.get("stdout", "")
    require("M6C:SURVIVOR_PROGRESS:PASS" in stdout, f"{case}: peer progress missing")
    require("M6C:RESOURCES:PASS" in stdout, f"{case}: resource cleanup missing")
    require(report["disk_oracle"].get("passed") is True, f"{case}: disk oracle failed")
    if case in ("write-error", "flush-error", "write-timeout", "flush-timeout", "reset-timeout"):
        require(report.get("outcome_marker_passed") is True, f"{case}: uncertainty proof failed")
        require("M6C:OUTCOME:UNKNOWN" in stdout, f"{case}: missing Unknown")
        require("M6C:NO_REPLAY:PASS" in stdout, f"{case}: replay detected")
        require("M6C:POST_UNKNOWN:BLOCKCALLS:0" in stdout, f"{case}: post-Unknown I/O")
    if case == "reset-timeout":
        require("M6C:QUARANTINE:PASS" in stdout, "reset-timeout: quarantine missing")
    return {
        "case": case,
        "run_directory": report["run_directory"],
        "raw_qemu_exit": report["raw_qemu_exit"],
        "image_after_sha256": report["artifact_sha256"]["test-disk.after.img"],
        "survivor_progress": True,
        "resource_cleanup": True,
        "no_replay": "M6C:NO_REPLAY:PASS" in stdout,
        "quarantine": "M6C:QUARANTINE:PASS" in stdout,
    }


def cold_recovery(
    previous: dict,
    volume,
    case: str,
    frozen: str,
    head: str,
    report: dict,
) -> dict:
    build(kernel_feature="test-m6c-recover", label=f"s08-{case}-recover")
    recovered, _ = run_m6c(
        "recover", volume=volume, label=f"s08-{case}-cold-recovery"
    )
    report["real_vm_invocations"] += 1
    report["last_run_directory"] = recovered.get("run_directory")
    require(recovered.get("passed") is True, f"{case}: cold recovery failed")
    # Hash equality alone misses transient writes that are later undone.
    require(
        any(
            argument.startswith("if=none,id=m6c-test,")
            and ",readonly=on," in argument
            for argument in recovered.get("command", [])
        ),
        f"{case}: recovery disk was not opened read-only",
    )
    require(recovered.get("base_commit") == head, f"{case}: recovery HEAD mismatch")
    require(
        recovered.get("source_sha256") == frozen
        and recovered.get("observed_source_sha256") == frozen
        and recovered.get("source_unchanged") is True,
        f"{case}: recovery source mismatch",
    )
    require(
        recovered.get("previous_stage") == previous["run_directory"],
        f"{case}: recovery did not bind previous VM",
    )
    require(
        recovered["volume_path"] == previous["volume_path"],
        f"{case}: recovery used another volume",
    )
    require(
        recovered["artifact_sha256"]["test-disk.before.img"]
        == previous["artifact_sha256"]["test-disk.after.img"],
        f"{case}: recovery did not start from exact saved bytes",
    )
    require(
        recovered["artifact_sha256"]["test-disk.before.img"]
        == recovered["artifact_sha256"]["test-disk.after.img"],
        f"{case}: recovery wrote to the image",
    )
    return {
        "case": case,
        "run_directory": recovered["run_directory"],
        "raw_qemu_exit": recovered["raw_qemu_exit"],
        "same_exact_image": True,
        "read_only_recovery": True,
    }


def main() -> int:
    result_path = BUILD / "s08-storage-resilience.json"
    frozen = source_hash()
    head = git_head()
    report = {
        "schema_version": 1,
        "criterion": "S08",
        "status": "running",
        "passed": False,
        "source_commit": head,
        "source_fingerprint_version": 3,
        "source_sha256": frozen,
        "started_utc": datetime.now(timezone.utc).isoformat(),
        "cases": [],
        "recoveries": [],
        "real_vm_invocations": 0,
        "claim_boundary": (
            "Native QEMU evidence for storage failure responsiveness and explicit "
            "cold recovery; no physical disk or power-loss claim."
        ),
    }
    save_json(result_path, report)
    try:
        for case in CASES:
            require(git_head() == head and source_hash() == frozen, f"{case}: source moved")
            build(kernel_feature=f"test-m6c-{case}", label=f"s08-{case}")
            guest, volume = run_m6c(case, label=f"s08-{case}")
            report["real_vm_invocations"] += 1
            report["last_run_directory"] = guest.get("run_directory")
            report["cases"].append(accepted_guest(guest, case, frozen, head))
            save_json(result_path, report)
            if case in RECOVERABLE:
                recovery = cold_recovery(
                    guest, volume, case, frozen, head, report
                )
                report["recoveries"].append(recovery)
                save_json(result_path, report)
        require(report["real_vm_invocations"] == 13, "expected exactly 13 VM invocations")
        require(len(report["cases"]) == 7 and len(report["recoveries"]) == 6, "case coverage incomplete")
        require(git_head() == head and source_hash() == frozen, "source moved after gate")
        report.update(status="completed", passed=True)
    except Exception as exc:
        report.update(status="failed", passed=False, error=f"{type(exc).__name__}: {exc}")
        raise
    finally:
        report["finished_utc"] = datetime.now(timezone.utc).isoformat()
        save_json(result_path, report)
    print(f"S08 accepted={report['passed']}; real VMs={report['real_vm_invocations']}; {result_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
