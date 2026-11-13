#!/usr/bin/env python3
"""Run the frozen SMP section 9 matrix locally, as native-smp.yml does, within a vCPU budget.

Layout mirrors the CI artifacts (smp-case-<case>/, smp-soak-<shard>/) so that
tools/aggregate_smp_ci.py and docs/verification/k06-smp-sched/verify_artifacts.py
check it unchanged. A failed case is recorded, never retried, and fails the matrix.
"""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import threading

from aggregate_smp_ci import SOAK_CASE, SOAK_REPETITIONS, SOAK_SHARDS, aggregate
from common import ROOT, save_json, source_hash
from run_smp import CASES

REPETITIONS = 50
# Bounded host overcommit: TCG drops PIT edges when vCPU threads exceed host cores.
VCPU_BUDGET = 24
NO_FPU_PROBE = ("native-smp-m3-unmap", "test-smp-m3-no-shootdown")


class CpuBudget:
    def __init__(self, total: int):
        self.available = total
        self.condition = threading.Condition()

    def acquire(self, amount: int):
        with self.condition:
            self.condition.wait_for(lambda: self.available >= amount)
            self.available -= amount

    def release(self, amount: int):
        with self.condition:
            self.available += amount
            self.condition.notify_all()


def invoke(command: list[str], *, build: Path, log: Path) -> int:
    environment = os.environ.copy()
    environment["NOS_BUILD_DIR"] = str(build)
    with log.open("wb") as output:
        return subprocess.run(command, cwd=ROOT, env=environment, stdout=output,
                              stderr=subprocess.STDOUT).returncode


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    if subprocess.run(["git", "diff", "--quiet", "HEAD", "--"], cwd=ROOT).returncode != 0:
        raise ValueError("tracked changes present; the matrix binds to a committed HEAD")
    output = args.output_dir.resolve()
    if output.exists():
        raise ValueError("matrix output directory must not already exist")
    logs, images = output / "logs", output / "images"
    logs.mkdir(parents=True)
    images.mkdir()

    frozen = source_hash()
    for profile in sorted({spec[0] for spec in CASES.values()}):
        build = images / profile
        if invoke([sys.executable, "-B", "tools/build.py", "--kernel-feature", profile],
                  build=build, log=logs / f"build-{profile}.log"):
            raise RuntimeError(f"build failed: {profile}; see {logs}")
        if json.loads((build / "manifest.json").read_text())["source_sha256"] != frozen:
            raise RuntimeError("source changed during matrix image construction")
        if "m3" in profile and profile not in NO_FPU_PROBE:
            if invoke([sys.executable, "-B", "userspace/tests/test_smp_fpu_probe.py",
                       "--elf", str(build / "users" / "probe.elf")],
                      build=build, log=build / "fpu-fixture-host.log"):
                raise RuntimeError(f"FPU probe host check failed: {profile}")

    tasks = []
    for case, (profile, cpus, _, _) in CASES.items():
        shards = ([(f"smp-soak-{n}", SOAK_REPETITIONS) for n in range(SOAK_SHARDS)]
                  if case == SOAK_CASE else [(f"smp-case-{case}", REPETITIONS)])
        for artifact, repeat in shards:
            build = output / artifact
            build.mkdir()
            shutil.copytree(images / profile / "esp", build / "esp")
            for name in ("manifest.json", "fpu-fixture-host.log"):
                if (images / profile / name).exists():
                    shutil.copy2(images / profile / name, build / name)
            tasks.append((artifact, case, repeat, cpus, build))
    # Long soaks first; shorter cases backfill as vCPU capacity returns.
    tasks.sort(key=lambda task: (task[1] != SOAK_CASE, -task[3], task[0]))
    budget = CpuBudget(VCPU_BUDGET)

    def execute(task):
        artifact, case, repeat, cpus, build = task
        budget.acquire(cpus)
        try:
            return artifact, invoke([sys.executable, "-B", "-u", "tools/run_smp.py", "--case", case,
                                     "--repeat", str(repeat)], build=build, log=logs / f"run-{artifact}.log")
        finally:
            budget.release(cpus)

    status = {}
    with ThreadPoolExecutor(max_workers=12) as pool:
        for future in as_completed([pool.submit(execute, task) for task in tasks]):
            artifact, code = future.result()
            status[artifact] = code
            print(f"MATRIX {artifact} rc={code}", flush=True)
            save_json(output / "status.json", status)
    if source_hash() != frozen:
        raise RuntimeError("source changed during matrix execution")
    shutil.rmtree(images)
    report = output / "smp-section9-matrix-report" / "matrix-report.json"
    try:
        aggregate(output, report)
    except ValueError as exc:
        print(f"MATRIX aggregate failed: {exc}", flush=True)
        return 1
    return 0 if all(code == 0 for code in status.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
