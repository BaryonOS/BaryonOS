#!/usr/bin/env python3
"""H05 step 2 (ADR-076) and H05-T4 on one frozen source: each power or S3
case on a fresh disk, then a read-only cold recovery of exactly that image
(data survived S5, the S3 cycle, the rejected wake or the refused poweroff).

Without arguments every case runs; `--case` selects some (repeatable)."""
from __future__ import annotations

import argparse
import sys

from build import build
from common import BUILD, save_json, source_hash
from run_system import FEATURE, POWER_FEATURE, S3_FEATURE, run_system

CASES = {**POWER_FEATURE, **S3_FEATURE}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", action="append", choices=tuple(CASES))
    selected = parser.parse_args(argv).case or list(CASES)
    frozen = source_hash()
    summary = dict(schema_version=1, source_fingerprint_version=3, source_sha256=frozen,
                   status="running", passed=False, cases=list(dict.fromkeys(selected)), stages=[])
    path = BUILD / "power-system-gate.json"
    save_json(path, summary)
    try:
        for case, feature in ((case, CASES[case]) for case in summary["cases"]):
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
    print(f"H05 power gate passed={summary['passed']} stages={len(summary['stages'])} {path}")
    if "error" in summary:
        print(summary["error"])
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
