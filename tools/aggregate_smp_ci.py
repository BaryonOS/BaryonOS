#!/usr/bin/env python3
"""Aggregate independently archived SMP CI shards into one 50/50 report."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
from pathlib import Path

from common import save_json, source_hash
from run_smp import CASES

REPETITIONS = 50
SOAK_CASE = "m3-soak-four"
SOAK_SHARDS = 10
SOAK_REPETITIONS = 5


def aggregate(input_dir: Path, output: Path) -> dict:
    root = input_dir.resolve(strict=True)
    reports = sorted(root.rglob("smp-report.json"))
    if not reports:
        raise ValueError("no SMP shard reports downloaded")
    frozen = source_hash()
    observed: dict[str, list[tuple[Path, dict]]] = {case: [] for case in CASES}
    for path in reports:
        if path.is_symlink() or not path.is_file() or not path.resolve().is_relative_to(root):
            raise ValueError("SMP shard report is not a confined regular file")
        report = json.loads(path.read_text(encoding="utf-8"))
        repetitions = report.get("repetitions")
        if (report.get("schema_version") != 1 or report.get("passed") is not True
                or report.get("source_fingerprint_version") != 3
                or report.get("source_sha256") != frozen
                or not isinstance(repetitions, dict) or len(repetitions) != 1):
            raise ValueError(f"invalid or failed SMP shard report: {path}")
        case = next(iter(repetitions))
        if case not in CASES or report.get("kernel_feature") != CASES[case][0]:
            raise ValueError(f"unknown case or profile mismatch: {path}")
        summary = repetitions[case]
        expected = SOAK_REPETITIONS if case == SOAK_CASE else REPETITIONS
        if (report.get("repeat") != expected or summary.get("requested") != expected
                or summary.get("attempted") != expected or summary.get("passed") != expected
                or summary.get("failed") != 0 or summary.get("complete") is not True
                or summary.get("flaky") is not False):
            raise ValueError(f"incomplete SMP shard: {path}")
        runs = report.get("cases")
        if (not isinstance(runs, list) or len(runs) != expected
                or any(run.get("case") != case or run.get("passed") is not True for run in runs)):
            raise ValueError(f"malformed SMP run evidence: {path}")
        observed[case].append((path, report))

    cases = {}
    for case, shards in observed.items():
        expected_shards = SOAK_SHARDS if case == SOAK_CASE else 1
        if len(shards) != expected_shards:
            raise ValueError(f"expected {expected_shards} reports for {case}, found {len(shards)}")
        attempted = sum(report["repetitions"][case]["attempted"] for _, report in shards)
        passed = sum(report["repetitions"][case]["passed"] for _, report in shards)
        if attempted != REPETITIONS or passed != REPETITIONS:
            raise ValueError(f"case is not 50/50: {case}")
        cases[case] = {
            "attempted": attempted,
            "passed": passed,
            "reports": [str(path.relative_to(root)) for path, _ in shards],
        }

    result = {
        "schema": 1,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "source_sha256": frozen,
        "repetitions_per_case": REPETITIONS,
        "case_count": len(CASES),
        "guest_run_count": len(CASES) * REPETITIONS,
        "passed": True,
        "cases": cases,
    }
    save_json(output, result)
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    aggregate(args.input_dir, args.output)
