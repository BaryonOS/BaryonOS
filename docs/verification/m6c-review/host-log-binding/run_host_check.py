"""Record one targeted Python host-test run; does not run a native gate."""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

root = Path(__file__).resolve().parent
source = root/'candidate'
label, pattern = sys.argv[1:]
if not re.fullmatch(r'[a-z0-9-]+', label) or not re.fullmatch(r'test_[a-z0-9_*]+\.py', pattern):
    raise ValueError('Unexpected check label or test pattern')


def hashes():
    return {path.relative_to(source).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for folder in ('tools', 'tests') for path in sorted((source/folder).rglob('*.py'))}


before = hashes()
command = [sys.executable, '-B', '-m', 'unittest', 'discover', '-s', 'tests', '-p', pattern, '-v']
started = datetime.now(timezone.utc).isoformat()
start = time.monotonic()
result = subprocess.run(command, cwd=source,
    env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1', TMPDIR=str(root/'tmp')),
    capture_output=True, timeout=60)
duration = time.monotonic()-start
after = hashes()
stdout = root/'records'/(label+'.stdout.txt')
stderr = root/'records'/(label+'.stderr.txt')
stdout.write_bytes(result.stdout)
stderr.write_bytes(result.stderr)
record = {'command': command, 'cwd': str(source), 'started_utc': started,
          'duration_s': duration, 'returncode': result.returncode,
          'baseline_commit': 'b3d9d682f3f043ef7926f112b659ad77a5f00e1a',
          'scope': 'Python-hosttests met synthetische gastrecords en commandodoubles; geen native build of VM-run.',
          'source_sha256_before': before, 'source_sha256_after': after,
          'source_unchanged': before == after, 'stdout': str(stdout), 'stderr': str(stderr),
          'stdout_sha256': hashlib.sha256(result.stdout).hexdigest(),
          'stderr_sha256': hashlib.sha256(result.stderr).hexdigest()}
(root/'records'/(label+'.json')).write_text(json.dumps(record, indent=2)+'\n')
print(json.dumps({'label': label, 'returncode': result.returncode,
                  'source_unchanged': before == after,
                  'stderr_tail': result.stderr.decode().splitlines()[-8:]}, indent=2))
