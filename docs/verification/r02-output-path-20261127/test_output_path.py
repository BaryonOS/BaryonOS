#!/usr/bin/env python3
"""Exercise the diagnostic runner's output location without a QEMU build."""
import os
from pathlib import Path
import subprocess
import tempfile

runner = Path(__file__).resolve().parents[1] / 'r02-repaint-tick-split-20261127' / 'run.sh'
with tempfile.TemporaryDirectory(prefix='r02 output ') as tmp:
    root = Path(tmp)
    home = root / 'home with spaces'
    for path in ('projects/BaryonOS-scratch/r/diag-tick/src', 'projects/BaryonOS-worktrees/r-tick'):
        (home / path).mkdir(parents=True)
    bindir = root / 'bin'
    bindir.mkdir()
    for name in ('git', 'python3'):
        stub = bindir / name
        stub.write_text('#!/bin/sh\nprintf "stub %s\\n" "$*"\n')
        stub.chmod(0o755)
    env = dict(os.environ, HOME=str(home), PATH=str(bindir) + os.pathsep + os.environ['PATH'],
               R02_OUTPUT_DIR='relative evidence/output', BUILD_ONLY='1')
    result = subprocess.run(['bash', str(runner)], cwd=root, env=env, capture_output=True, text=True)
    assert result.returncode == 0, (result.returncode, result.stdout, result.stderr)
    output = root / env['R02_OUTPUT_DIR']
    for name in ('meta.txt', 'build-D.log', 'build-M.log'):
        assert (output / name).is_file(), name
    assert 'build-only' in (output / 'meta.txt').read_text()
print('relative output path and spaces: PASS')
