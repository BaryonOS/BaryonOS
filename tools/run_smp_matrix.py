#!/usr/bin/env python3
"""Run the frozen SMP acceptance matrix with a bounded aggregate vCPU budget."""
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
from datetime import datetime, timezone

from common import ROOT, save_json, source_hash
from run_smp import CASES

REPETITIONS = 50
VCPU_BUDGET = 24
SOAK_SHARDS = (9, 9, 8, 8, 8, 8)


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


def invoke(command: list[str], *, build: Path, log: Path) -> None:
    environment = os.environ.copy()
    environment["NOS_BUILD_DIR"] = str(build)
    with log.open("wb") as output:
        result = subprocess.run(command, cwd=ROOT, env=environment, stdout=output,
                                stderr=subprocess.STDOUT)
    if result.returncode != 0:
        raise RuntimeError(f"command failed ({result.returncode}); see {log}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    output = args.output_dir.resolve()
    if output.exists():
        raise ValueError("matrix output directory must not already exist")
    output.mkdir(parents=True)
    logs, images, runs = (output / name for name in ("logs", "images", "runs"))
    logs.mkdir(); images.mkdir(); runs.mkdir()

    frozen = source_hash()
    profiles = sorted({profile for profile, _, _, _ in CASES.values()})
    for profile in profiles:
        build = images / profile
        invoke([sys.executable, "-B", "tools/build.py", "--kernel-feature", profile],
               build=build, log=logs / f"build-{profile}.log")
        manifest = json.loads((build / "manifest.json").read_text())
        if manifest["source_sha256"] != frozen:
            raise RuntimeError("source changed during matrix image construction")

    tasks = []
    for case, (profile, cpus, _, _) in CASES.items():
        repeats = SOAK_SHARDS if case == "m3-soak-four" else (REPETITIONS,)
        for shard, repeat in enumerate(repeats):
            build = runs / f"{case}-{shard}"
            build.mkdir()
            shutil.copytree(images / profile / "esp", build / "esp")
            shutil.copy2(images / profile / "manifest.json", build / "manifest.json")
            tasks.append((case, shard, repeat, cpus, build))
    # Start the six long soaks first; the shared budget admits exactly six
    # four-vCPU guests and then backfills shorter cases as capacity returns.
    tasks.sort(key=lambda task: (task[0] != "m3-soak-four", -task[3], task[0], task[1]))
    budget = CpuBudget(VCPU_BUDGET)

    def execute(task):
        case, shard, repeat, cpus, build = task
        budget.acquire(cpus)
        try:
            invoke([sys.executable, "-B", "-u", "tools/run_smp.py", "--case", case,
                    "--repeat", str(repeat)], build=build,
                   log=logs / f"run-{case}-{shard}.log")
            return task, json.loads((build / "smp-report.json").read_text())
        finally:
            budget.release(cpus)

    results = []
    with ThreadPoolExecutor(max_workers=12) as pool:
        futures = [pool.submit(execute, task) for task in tasks]
        for future in as_completed(futures):
            task, report = future.result()
            results.append((task, report))
            print(f"MATRIX {task[0]} shard={task[1]}: {report['passed_runs']}/{report['attempted_runs']}",
                  flush=True)

    cases = {}
    for case in CASES:
        shards = [(task, report) for task, report in results if task[0] == case]
        attempted = sum(report["attempted_runs"] for _, report in shards)
        passed = sum(report["passed_runs"] for _, report in shards)
        hashes = {report["source_sha256"] for _, report in shards}
        cases[case] = {"attempted": attempted, "passed": passed,
                       "shards": [str(task[4].relative_to(output)) for task, _ in shards]}
        if attempted != REPETITIONS or passed != REPETITIONS or hashes != {frozen}:
            raise RuntimeError(f"matrix case not 50/50 on frozen source: {case}")
    if source_hash() != frozen:
        raise RuntimeError("source changed during matrix execution")
    save_json(output / "matrix-report.json", {
        "schema": 1,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "source_sha256": frozen,
        "vcpus_budget": VCPU_BUDGET,
        "repetitions_per_case": REPETITIONS,
        "passed": True,
        "cases": cases,
    })
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
