#!/usr/bin/env python3
"""Copy compact, hash-bound B03 gate evidence into docs/verification."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import shutil
from common import BUILD, ROOT
from update_paths import portable


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", type=Path, default=ROOT / "docs/verification/b03")
    args = parser.parse_args()
    destination = args.destination.resolve()
    destination.relative_to((ROOT / "docs/verification").resolve())
    if destination.exists():
        raise FileExistsError(f"{destination} exists; use a new destination")
    summary_path = BUILD / "update-gate.json"
    summary = json.loads(summary_path.read_text())
    if summary.get("status") != "completed" or summary.get("passed") is not True:
        raise ValueError("only a completed, passing B03 gate can be captured")
    destination.mkdir(parents=True)
    for index, stage in enumerate(summary["stages"]):
        run = (ROOT / stage["run_directory"]).resolve(strict=True)
        run.relative_to((BUILD / "run").resolve())
        report = json.loads((run / "result.json").read_text())
        if report.get("passed") is not True or report["source_sha256"] != summary["source_sha256"]:
            raise ValueError(f"stage {index} is not a passing run of the frozen source")
        relative_run = run.relative_to(ROOT).as_posix()
        stage["run_directory"] = relative_run
        report["run_directory"] = relative_run
        target = destination / f"{index}-{stage['stage']}"
        target.mkdir()
        (target / "result.json").write_text(json.dumps(portable(report), indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        for name in ("serial.txt", "stderr.txt"):
            shutil.copy2(run / name, target / name)
    (destination / "update-gate.json").write_text(json.dumps(portable(summary), indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    sums = [f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.relative_to(destination).as_posix()}"
            for path in sorted(destination.rglob("*")) if path.is_file()]
    (destination / "SHA256SUMS").write_text("\n".join(sums) + "\n", encoding="utf-8")
    print(f"captured {len(summary['stages'])} stages into {destination}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
