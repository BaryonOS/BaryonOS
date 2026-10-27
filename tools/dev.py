#!/usr/bin/env python3
"""Run a development command in the isolated local tool environment."""
import subprocess
import sys
import shutil
from common import ROOT, environment

if __name__ == "__main__":
    if len(sys.argv) < 2:
        raise SystemExit("Usage: python tools/dev.py COMMAND [ARGS...]")
    env = environment()
    command = shutil.which(sys.argv[1], path=env["PATH"]) or sys.argv[1]
    raise SystemExit(subprocess.call([command, *sys.argv[2:]], cwd=ROOT, env=env))
