#!/usr/bin/env python3
"""Capture one command's complete stdout, stderr, and raw exit status."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--log", type=Path, required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command:
        parser.error("a command is required")
    draft = Path(__file__).resolve().parents[1]
    log = args.log if args.log.is_absolute() else draft / args.log
    log = log.resolve()
    if not log.is_relative_to(draft):
        parser.error("log must remain inside the draft")
    result = subprocess.run(command, capture_output=True)
    record = {
        "command": command,
        "exit_code": result.returncode,
        "stdout_utf8": result.stdout.decode("utf-8", "backslashreplace"),
        "stderr_utf8": result.stderr.decode("utf-8", "backslashreplace"),
    }
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps({
        "log": str(log),
        "exit_code": result.returncode,
        "stdout_bytes": len(result.stdout),
        "stderr_bytes": len(result.stderr),
    }, sort_keys=True))
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
