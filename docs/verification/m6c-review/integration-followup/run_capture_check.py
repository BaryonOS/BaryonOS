"""Record the single combined M6a/M6b/M6c capture integration check."""
from collections import Counter
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
CANDIDATE = ROOT / 'candidate'
RECORDS = ROOT / 'records'
record_path = RECORDS / 'combined-capture.json'
if record_path.exists():
    raise ValueError('This invocation is already recorded; do not overwrite evidence')


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inputs():
    return {path.relative_to(CANDIDATE).as_posix(): sha256(path)
            for folder in ('tools', 'tests')
            for path in sorted((CANDIDATE / folder).rglob('*'))
            if path.is_file() and '__pycache__' not in path.parts}


provenance = json.loads((ROOT / 'provenance.json').read_text())
expected = {name: value['sha256'] for name, value in provenance['baseline_host_files'].items()}
for overlay in provenance['overlays']:
    expected.update({entry['path']: entry['sha256'] for entry in overlay['files']})
before = inputs()
if before != expected:
    raise ValueError('Combined host inputs differ from the verified base plus six overlays')
command = [sys.executable, '-B', '-m', 'unittest', 'discover', '-s', 'tests',
           '-p', 'test_m6*_capture.py', '-v']
environment = {'PYTHONDONTWRITEBYTECODE': '1', 'TMPDIR': str(ROOT / 'tmp')}
started = datetime.now(timezone.utc).isoformat()
start = time.monotonic()
completed = subprocess.run(command, cwd=CANDIDATE, env=dict(os.environ, **environment),
                           capture_output=True, timeout=60)
duration = time.monotonic() - start
after = inputs()
outputs = {}
for stream in ('stdout', 'stderr'):
    path = RECORDS / ('combined-capture.' + stream + '.txt')
    data = getattr(completed, stream)
    path.write_bytes(data)
    outputs[stream] = {'path': str(path), 'bytes': len(data), 'sha256': sha256(path)}
stderr = completed.stderr.decode('utf-8', 'replace')
count = re.search(r'^Ran (\d+) tests in ', stderr, re.MULTILINE)
modules = Counter(re.findall(r'^test_\S+ \((test_m6[a-z]*_capture)\.', stderr, re.MULTILINE))
record = {'scope': provenance['scope'], 'baseline_commit': provenance['baseline_commit'],
          'overlays': provenance['overlays'], 'provenance_sha256': sha256(ROOT / 'provenance.json'),
          'command': command, 'cwd': str(CANDIDATE), 'environment_overrides': environment,
          'started_utc': started, 'duration_s': duration, 'returncode': completed.returncode,
          'python_version': sys.version, 'recorder_sha256': sha256(Path(__file__)),
          'unittest_reported_tests': int(count.group(1)) if count else None,
          'test_method_counts_by_module': dict(modules),
          'input_sha256_before': before, 'input_sha256_after': after,
          'inputs_unchanged': before == after, 'output': outputs}
record_path.write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({'returncode': record['returncode'],
                  'unittest_reported_tests': record['unittest_reported_tests'],
                  'test_method_counts_by_module': record['test_method_counts_by_module'],
                  'inputs_unchanged': record['inputs_unchanged'],
                  'stderr_tail': stderr.splitlines()[-6:]}, indent=2))
