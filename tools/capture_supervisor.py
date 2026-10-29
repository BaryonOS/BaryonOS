#!/usr/bin/env python3
"""Copy compact, hash-bound K05 gate evidence into docs/verification."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import shutil
from common import BUILD, ROOT


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", type=Path, default=ROOT / "docs/verification/k05")
    args = parser.parse_args()
    destination = args.destination.resolve()
    destination.relative_to((ROOT / "docs/verification").resolve())
    if destination.exists():
        raise FileExistsError(f"{destination} exists; use a new destination")
    summary_path = BUILD / "supervisor-gate.json"
    summary = json.loads(summary_path.read_text())
    if summary.get("status") != "completed" or summary.get("passed") is not True:
        raise ValueError("only a completed, passing K05 gate can be captured")
    destination.mkdir(parents=True)
    shutil.copy2(summary_path, destination / "supervisor-gate.json")
    for index, stage in enumerate(summary["stages"]):
        run = Path(stage["run_directory"]).resolve(strict=True)
        run.relative_to((BUILD / "run").resolve())
        report = json.loads((run / "result.json").read_text())
        if report.get("passed") is not True or report["source_sha256"] != summary["source_sha256"]:
            raise ValueError(f"stage {index} is not a passing run of the frozen source")
        target = destination / f"{index}-{stage['case']}"
        target.mkdir()
        for name in ("result.json", "serial.txt", "stderr.txt"):
            shutil.copy2(run / name, target / name)
    sums = [f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.relative_to(destination).as_posix()}"
            for path in sorted(destination.rglob("*")) if path.is_file()]
    (destination / "SHA256SUMS").write_text("\n".join(sums) + "\n", encoding="utf-8")
    print(f"captured {len(summary['stages'])} stages into {destination}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
