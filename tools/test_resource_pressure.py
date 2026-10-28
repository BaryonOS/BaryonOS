#!/usr/bin/env python3
"""K07: source-bound native resource-pressure and recovery gate."""
from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

from common import BUILD, ROOT, TARGET, save_json, source_hash

def run(command: list[str], *, env: dict[str, str]) -> None:
    result = subprocess.run(command, cwd=ROOT, env=env, text=True, timeout=300)
    if result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {' '.join(command)}")

def load(path: Path) -> dict:
    data = json.loads(path.read_text())
    if not isinstance(data, dict):
        raise ValueError(f"invalid report: {path}")
    return data

def child_env(build: Path, target: Path) -> dict[str, str]:
    env = dict(os.environ)
    env["NOS_BUILD_DIR"] = str(build)
    env["NOS_TARGET_DIR"] = str(target)
    env["CARGO_TARGET_DIR"] = str(target)
    return env

def main() -> int:
    frozen = source_hash()
    root = BUILD / "resource-pressure"
    if root.exists():
        raise ValueError("resource-pressure output already exists; use a fresh NOS_BUILD_DIR")
    root.mkdir(parents=True)
    report = {
        "schema_version": 1,
        "criterion": "K07",
        "source_fingerprint_version": 3,
        "source_sha256": frozen,
        "passed": False,
        "checks": {},
    }
    try:
        # Handle slots, derivation depth and Message320 queue capacity. The guest
        # marker is emitted only after the kernel asserts exact quota errors,
        # reuse, 16 sends/receives and invariant-preserving cleanup.
        m4_build = root / "m4"
        m4_target = TARGET / "resource-pressure-m4"
        env = child_env(m4_build, m4_target)
        run([sys.executable, "-B", "tools/build.py", "--kernel-feature", "test-m4-quotas"], env=env)
        code = (
            "from common import BUILD,save_json;"
            "from run import run_gate_case;"
            "r=run_gate_case('m4','test-m4-quotas');"
            "save_json(BUILD/'k07-m4.json',r);"
            "raise SystemExit(0 if r.get('passed') is True else 1)"
        )
        pyenv = dict(env)
        pyenv["PYTHONPATH"] = str(ROOT / "tools")
        run([sys.executable, "-B", "-c", code], env=pyenv)
        m4 = load(m4_build / "k07-m4.json")
        serial = m4.get("stdout", "")
        required_m4 = (
            "M4:CASE:QUOTAS:PASS",
            "M4:SURVIVOR_PROGRESS:PASS",
            "M4:CLEANUP:PASS",
            "KS-15:PASS",
            "M4:PASS",
        )
        if m4.get("passed") is not True or any(marker not in serial for marker in required_m4):
            raise ValueError("M4 quota/cleanup evidence incomplete")
        report["checks"]["ipc_handle_quota_and_recovery"] = True
        report["m4"] = {
            "raw_qemu_exit": m4.get("raw_qemu_exit"),
            "run_directory": m4.get("run_directory"),
            "required_markers": list(required_m4),
        }

        # Two isolated native qsim processes exercise the private 64-KiB arena,
        # actual ResourceExhausted, cancellation/deadline checkpoints, timer
        # preemption, progress in both processes and exact frame cleanup.
        q_build = root / "qsim"
        q_target = TARGET / "resource-pressure-qsim"
        env = child_env(q_build, q_target)
        run([sys.executable, "-B", "tools/build.py", "--kernel-feature", "native-qsim"], env=env)
        run([sys.executable, "-B", "tools/run_qsim.py"], env=env)
        qsim = load(q_build / "native-qsim-result.json")
        observed = qsim.get("observed", {})
        guest = qsim.get("guest", {})
        stdout = guest.get("stdout", "")
        required_q = (
            "QSIM:RESOURCE_LIMITS:PASS",
            "QSIM:PROCESS_ISOLATION:PASS",
            "QSIM:CANCEL:PASS",
            "QSIM:DEADLINE:PASS",
            "QSIM:CLEANUP:PASS",
            "QSIM:PASS",
        )
        processes = observed.get("processes", [])
        if (
            qsim.get("passed") is not True
            or len(processes) != 2
            or any(item.get("checkpoints", 0) <= 0 for item in processes)
            or observed.get("timer_irqs", 0) <= 0
            or observed.get("cleanup_frames", 0) <= 0
            or any(marker not in stdout for marker in required_q)
        ):
            raise ValueError("qsim memory/CPU/resource evidence incomplete")
        report["checks"]["memory_cpu_budget_and_recovery"] = True
        report["qsim"] = {
            "raw_qemu_exit": guest.get("raw_qemu_exit"),
            "run_directory": guest.get("run_directory"),
            "processes": processes,
            "timer_irqs": observed.get("timer_irqs"),
            "cleanup_frames": observed.get("cleanup_frames"),
            "required_markers": list(required_q),
        }

        if source_hash() != frozen:
            raise ValueError("source changed during K07 gate")
        report["checks"]["source_unchanged"] = True
        report["passed"] = all(report["checks"].values())
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    finally:
        save_json(root / "result.json", report)
    print(f"K07 resource-pressure accepted={report['passed']}; {root / 'result.json'}")
    if "error" in report:
        print(report["error"])
    return 0 if report["passed"] else 1

if __name__ == "__main__":
    raise SystemExit(main())
