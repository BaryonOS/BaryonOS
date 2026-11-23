#!/usr/bin/env python3
"""H05 step 2 (ADR-076) and H05-T4 on one frozen source: each power or S3
case on a fresh disk, then a read-only cold recovery of exactly that image
(data survived S5, the S3 cycle, the rejected wake or the refused poweroff).

Without arguments exactly the three H05 POWER cases run and report to
`power-system-gate.json` (the native-power.yml step). The S3 cases run only
when selected, with `--s3` (all of them) or a repeatable `--case`, and report
to `s3-system-gate.json`, so both gates can share one NOS_BUILD_DIR. One
selection holds either POWER or S3 cases, never both."""
from __future__ import annotations

import argparse
import sys

from build import build
from common import BUILD, save_json, source_hash
from run_system import FEATURE, POWER_FEATURE, S3_FEATURE, run_system

CASES = {**POWER_FEATURE, **S3_FEATURE}
REPORTS = {"power": "power-system-gate.json", "s3": "s3-system-gate.json"}


def select(argv: list[str] | None = None) -> tuple[list[str], str]:
    """Selected cases (in order, without duplicates) and their report name."""
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--case", action="append", choices=tuple(CASES))
    group.add_argument("--s3", action="store_true", help="run every S3 case")
    args = parser.parse_args(argv)
    cases = list(S3_FEATURE) if args.s3 else list(dict.fromkeys(args.case or POWER_FEATURE))
    families = {"s3" if case in S3_FEATURE else "power" for case in cases}
    if len(families) != 1:
        parser.error("select either POWER or S3 cases; their gate reports are separate")
    return cases, REPORTS[families.pop()]


def main(argv: list[str] | None = None) -> int:
    selected, report = select(argv)
    frozen = source_hash()
    summary = dict(schema_version=1, source_fingerprint_version=3, source_sha256=frozen,
                   status="running", passed=False, cases=selected, stages=[])
    path = BUILD / report
    save_json(path, summary)
    try:
        for case, feature in ((case, CASES[case]) for case in selected):
            image = None
            for stage, stage_feature in ((case, feature), ("recover", FEATURE["recover"])):
                build(None, stage_feature, stage_feature)
                result, image = run_system(stage, image)
                summary["stages"].append({"parent": case, **{key: result[key] for key in
                                          ("case", "passed", "raw_qemu_exit", "run_directory",
                                           "before_sha256", "after_sha256", "duration_s")}})
                save_json(path, summary)
                if not result["passed"]:
                    raise RuntimeError(result.get("error", f"{case}/{stage} failed"))
        if source_hash() != frozen:
            raise ValueError("source changed during gate")
        summary.update(status="completed", passed=True)
    except Exception as error:
        summary.update(status="failed", error=f"{type(error).__name__}: {error}")
    save_json(path, summary)
    label = "H05-T4 S3" if report == REPORTS["s3"] else "H05 power"
    print(f"{label} gate passed={summary['passed']} stages={len(summary['stages'])} {path}")
    if "error" in summary:
        print(summary["error"])
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
