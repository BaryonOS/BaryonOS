"""Run the unchanged gate and capture after an explicit root source freeze."""
from datetime import datetime, timezone
from pathlib import Path
import argparse
import json
import os
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / "tools"))
from common import BUILD, sha256, source_hash


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--expected-source", required=True)
    parser.add_argument("--expected-commit", required=True)
    args = parser.parse_args()
    if BUILD != OUT or Path.cwd() != ROOT:
        raise RuntimeError("Wrong repository or isolated build directory")
    actual_commit = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    if actual_commit != args.expected_commit or source_hash() != args.expected_source:
        raise RuntimeError("Root source freeze no longer matches before launch")
    destination = ROOT / "docs/verification/parallel-final-m6b"
    if destination.exists():
        raise RuntimeError("Refuse to replace an earlier final M6b archive")
    report = {
        "scope": "Complete M6b native suite plus unchanged capture validator",
        "status": "running", "passed": False,
        "source_commit": actual_commit, "source_sha256": args.expected_source,
        "source_fingerprint_version": 3,
        "started_utc": datetime.now(timezone.utc).isoformat(),
        "free_bytes_before": shutil.disk_usage(ROOT).free,
        "environment": {name: os.environ.get(name) for name in (
            "NOS_BUILD_DIR", "NOS_TARGET_DIR", "CARGO_TARGET_DIR", "CARGO_BUILD_JOBS",
            "CARGO_HOME", "CARGO_NET_OFFLINE", "NOS_QEMU_MACHINE", "NOS_SDK_RUNTIME",
            "OS_OVMF_CODE", "OS_OVMF_VARS")},
        "commands": [],
    }

    def save():
        (OUT / "execution.json").write_text(json.dumps(report, indent=2) + "\n")

    try:
        save()
        for name, command in (
            ("gate", [sys.executable, "-u", "-B", "tools/test_m6b.py"]),
            ("capture", [sys.executable, "-u", "-B", "tools/capture_m6b.py",
                         "--destination", "docs/verification/parallel-final-m6b"]),
        ):
            if source_hash() != args.expected_source:
                raise RuntimeError("Source changed before " + name)
            started = time.monotonic()
            print("Starting", name, flush=True)
            with (OUT / (name + ".stdout.txt")).open("wb") as stdout, (OUT / (name + ".stderr.txt")).open("wb") as stderr:
                run = subprocess.run(command, cwd=ROOT, stdout=stdout, stderr=stderr, timeout=7200)
            report["commands"].append({"name": name, "command": command,
                "returncode": run.returncode, "duration_s": time.monotonic() - started,
                "stdout_sha256": sha256(OUT / (name + ".stdout.txt")),
                "stderr_sha256": sha256(OUT / (name + ".stderr.txt"))})
            save()
            print(name, "exit", run.returncode, flush=True)
            if run.returncode:
                raise RuntimeError(name + " failed")
        if source_hash() != args.expected_source:
            raise RuntimeError("Source changed after capture")
        report.update(status="completed", passed=True)
    except Exception as error:
        report.update(status="failed", passed=False, error=str(error))
        raise
    finally:
        report["finished_utc"] = datetime.now(timezone.utc).isoformat()
        report["free_bytes_after"] = shutil.disk_usage(ROOT).free
        report["observed_source_sha256"] = source_hash()
        save()
    for name in ("preparation.json", "execution.json", "execute_gate.py",
                 "gate.stdout.txt", "gate.stderr.txt", "capture.stdout.txt", "capture.stderr.txt"):
        shutil.copy2(OUT / name, destination / name)
    print(json.dumps({"passed": report["passed"], "archive": str(destination),
                      "source_commit": actual_commit, "source_sha256": args.expected_source}), flush=True)


if __name__ == "__main__":
    main()
