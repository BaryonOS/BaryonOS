"""Record one bounded Python host check with its exact input bytes and output."""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
label, snapshot, *arguments = sys.argv[1:]
if not re.fullmatch(r'[a-z0-9-]+', label):
    raise ValueError('Unexpected record label')
cwd = (ROOT / snapshot).resolve()
cwd.relative_to(ROOT)
command = [sys.executable, *arguments]
inputs = ['docs/verification/m6b-review/merge-followup/verify_m5_archive.py',
          'docs/verification/regression-m6b/m5/guest-runs.json',
          'docs/verification/regression-m6b/m5/m5-two-image-check.json']


def hashes():
    paths = [path for folder in ('tools', 'tests') for path in (cwd / folder).glob('*.py')]
    paths += [cwd / path for path in inputs if (cwd / path).is_file()]
    return {path.relative_to(cwd).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(paths)}


before = hashes()
started = datetime.now(timezone.utc).isoformat()
start = time.monotonic()
environment = {'PYTHONDONTWRITEBYTECODE': '1', 'TMPDIR': str(ROOT / 'tmp')}
completed = subprocess.run(command, cwd=cwd, env=dict(os.environ, **environment),
                           capture_output=True, timeout=60)
duration = time.monotonic() - start
after = hashes()
files = {}
for stream in ('stdout', 'stderr'):
    data = getattr(completed, stream)
    path = ROOT / 'records' / (label + '.' + stream + '.txt')
    path.write_bytes(data)
    files[stream] = {'path': str(path), 'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data)}
record = {'baseline_commit': '3f308b07e0bab445d1ea926561eab8d8e01c86a4',
          'command': command, 'cwd': str(cwd), 'environment_overrides': environment,
          'started_utc': started, 'duration_s': duration, 'returncode': completed.returncode,
          'scope': 'Python host checks and archive reconstruction only; synthetic negative fixture where identified. No native build or VM run.',
          'recorder_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
          'input_sha256_before': before, 'input_sha256_after': after,
          'inputs_unchanged': before == after, 'output': files}
(ROOT / 'records' / (label + '.json')).write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({'label': label, 'returncode': completed.returncode,
                  'inputs_unchanged': before == after,
                  'stderr_tail': completed.stderr.decode('utf-8', 'replace').splitlines()[-8:]}, indent=2))
