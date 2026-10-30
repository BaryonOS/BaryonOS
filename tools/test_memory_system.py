#!/usr/bin/env python3
"""Build and execute all five native heap cases on one frozen committed source."""
from __future__ import annotations

import subprocess
import sys

from common import ROOT, BUILD, TARGET, run_logged, sha256, source_hash
from run_memory import CASES, FEATURES
from run_fpu import private_path, publish_json, load_json

SOURCE_PATHS = ("boot", "kernel", "crates", "drivers", "services", "userspace", "sdk",
                "experiments/quantum/qsim-core", "experiments/quantum/native",
                "experiments/quantum/job-protocol", "tools", ".cargo", "Cargo.toml",
                "Cargo.lock", "rust-toolchain.toml", "experiments/quantum/Cargo.toml")


def main() -> int:
    build = private_path(BUILD)
    target = private_path(TARGET)
    if any(path.exists() and not path.is_dir() for path in (build, target)):
        raise ValueError("Memory build/target must be private directories")
    destination = private_path(build / "memory-gate.json")
    build.mkdir(parents=True, exist_ok=True)
    pending = subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all", "--",
                                       *SOURCE_PATHS], cwd=ROOT)
    if pending:
        raise ValueError("Commit the exact memory source before starting the matrix")
    frozen = source_hash()
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    report = {"schema_version": 1, "source_commit": commit, "source_fingerprint_version": 3,
              "source_sha256": frozen, "passed": False, "status": "running", "stages": [],
              "scope": "five real private-heap guests; no full product or physical-hardware acceptance"}
    publish_json(destination, report)
    try:
        run_logged([sys.executable, "-B", "-m", "unittest", "discover", "-s", "tools", "-p", "test_memory.py"],
                   "memory-host-evidence")
        for case in CASES:
            case_build = private_path(build / case)
            case_target = private_path(target / case)
            extra = {"NOS_BUILD_DIR": str(case_build), "NOS_TARGET_DIR": str(case_target),
                     "CARGO_TARGET_DIR": str(private_path(case_target / "host"))}
            run_logged([sys.executable, "-B", "tools/build.py", "--kernel-feature", FEATURES[case]],
                       "memory-build-" + case, timeout=180, extra_env=extra)
            run_logged([sys.executable, "-B", "tools/run_memory.py", "--case", case],
                       "memory-run-" + case, timeout=90, extra_env=extra)
            result_path = private_path(case_build / "memory-report.json", regular=True)
            result = load_json(result_path)
            if (result.get("passed") is not True or result.get("case") != case
                    or result.get("source_sha256") != frozen or result["guest"].get("base_commit") != commit):
                raise ValueError("Memory stage source/commit/case mismatch")
            report["stages"].append({"case": case, "passed": True,
                                     "report": str(result_path), "report_sha256": sha256(result_path),
                                     "kernel_sha256": result["build_manifest"]["kernel"]["sha256"],
                                     "run_directory": result["guest"]["run_directory"],
                                     "duration_s": result["guest"]["duration_s"]})
            publish_json(destination, report)
        if source_hash() != frozen:
            raise ValueError("Memory source changed during the five guests")
        report.update(passed=True, status="completed")
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
    publish_json(destination, report)
    print(f"Native memory gate passed={report['passed']} stages={len(report['stages'])} {destination}")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
