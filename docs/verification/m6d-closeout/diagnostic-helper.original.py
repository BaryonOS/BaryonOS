"""Diagnostic observation only: return Path.stat results unchanged to guard."""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import time
import traceback

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from common import save_json, source_hash
from m6c_disk import Volume
from run_m6c import run_m6c


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--label", required=True)
    args = parser.parse_args()
    events = []
    original_stat = Path.stat
    original_guard = Volume.guard
    guard_number = 0

    def traced_stat(path, *positional, **keywords):
        observed = original_stat(path, *positional, **keywords)
        if path.name == "test-disk.img" and path.parent.name.startswith("m6c-volume-"):
            events.append({
                "event": "Path.stat", "time_ns": time.time_ns(), "path": str(path),
                "guard_number": guard_number, "kwargs": keywords,
                "stat": {key: getattr(observed, key, None) for key in (
                    "st_mode", "st_nlink", "st_size", "st_ino", "st_dev",
                    "st_file_attributes", "st_reparse_tag", "st_mtime_ns", "st_ctime_ns")},
                "regular": stat.S_ISREG(observed.st_mode),
                "stack": [f"{frame.filename}:{frame.lineno}:{frame.name}"
                          for frame in traceback.extract_stack(limit=7)[:-1]],
            })
        return observed

    def traced_guard(volume):
        nonlocal guard_number
        guard_number += 1
        number = guard_number
        events.append({"event": "guard-enter", "guard_number": number,
                       "time_ns": time.time_ns(), "path": str(volume.path)})
        try:
            value = original_guard(volume)
        except Exception as error:
            events.append({"event": "guard-rejected", "guard_number": number,
                           "time_ns": time.time_ns(), "error": f"{type(error).__name__}: {error}"})
            links = subprocess.run(["fsutil", "hardlink", "list", str(volume.path)],
                                   capture_output=True, text=True, timeout=10)
            events.append({"event": "hardlink-list-after-rejection", "guard_number": number,
                           "time_ns": time.time_ns(), "argv": links.args,
                           "returncode": links.returncode, "stdout": links.stdout, "stderr": links.stderr})
            raise
        events.append({"event": "guard-return", "guard_number": number, "time_ns": time.time_ns()})
        return value

    started = datetime.now(timezone.utc).isoformat()
    source = source_hash()
    result = None
    exception = None
    Path.stat = traced_stat
    Volume.guard = traced_guard
    try:
        result, volume = run_m6c("normal", label=args.label)
    except Exception as error:
        exception = f"{type(error).__name__}: {error}"
        traceback.print_exc()
    finally:
        Path.stat = original_stat
        Volume.guard = original_guard
    report = {
        "method": "Unchanged Path.stat return object; original Volume.guard called without retries or relaxed checks.",
        "argv": [sys.executable, *sys.argv], "cwd": str(Path.cwd()),
        "started_utc": started, "finished_utc": datetime.now(timezone.utc).isoformat(),
        "source_sha256": source, "observed_source_sha256": source_hash(),
        "exception": exception, "events": events,
        "run_directory": result and result["run_directory"],
        "passed": result and result["passed"],
        "raw_qemu_exit": result and result["raw_qemu_exit"],
        "oracle_error": result and result["oracle_error"],
    }
    destination = ROOT / "build/logs" / f"{args.label}-guard-trace.json"
    save_json(destination, report)
    print(json.dumps({"trace": str(destination), "passed": report["passed"],
                      "guard_calls": guard_number,
                      "rejected": [event for event in events if event["event"] == "guard-rejected"]}))
    return 0 if result and result["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
