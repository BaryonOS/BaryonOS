#!/usr/bin/env python3
"""Copy compact, hash-bound M6e gate evidence (no disk images) into docs/verification."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import shutil
from common import BUILD, ROOT, sha256


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", type=Path, default=ROOT / "docs/verification/m6e")
    args = parser.parse_args()
    destination = args.destination.resolve()
    destination.relative_to((ROOT / "docs/verification").resolve())
    if destination.exists():
        raise FileExistsError(f"{destination} exists; use a new destination")
    summary_path = BUILD / "volume-gate.json"
    summary = json.loads(summary_path.read_text())
    if summary.get("status") != "completed" or summary.get("passed") is not True:
        raise ValueError("only a completed, passing M6e gate can be captured")
    destination.mkdir(parents=True)
    shutil.copy2(summary_path, destination / "volume-gate.json")
    sums = []
    for index, stage in enumerate(summary["stages"]):
        run = Path(stage["run_directory"]).resolve(strict=True)
        run.relative_to((BUILD / "run").resolve())
        report = json.loads((run / "result.json").read_text())
        if report.get("passed") is not True or report["source_sha256"] != summary["source_sha256"]:
            raise ValueError(f"stage {index} is not a passing run of the frozen source")
        if sha256(run / "volume.after.img") != report["after_sha256"]:
            raise ValueError(f"stage {index} image hash no longer matches its report")
        target = destination / f"{index}-{stage['case']}"
        target.mkdir()
        for name in ("result.json", "serial.txt", "stderr.txt"):
            shutil.copy2(run / name, target / name)
    for path in sorted(destination.rglob("*")):
        if path.is_file():
            sums.append(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.relative_to(destination).as_posix()}")
    (destination / "SHA256SUMS").write_text("\n".join(sums) + "\n", encoding="utf-8")
    print(f"captured {len(summary['stages'])} stages into {destination}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
