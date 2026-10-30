"""Focused host checks for portable B03 evidence capture."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import capture_update


class Capture(unittest.TestCase):
    def test_capture_normalizes_paths_before_hashing(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            build = root / 'build'
            run = build / 'run' / 'update-example'
            run.mkdir(parents=True)
            destination = root / 'docs' / 'verification' / 'capture'
            report = {'passed': True, 'source_sha256': 'source', 'run_directory': str(run),
                      'state_directory': str(build / 'run' / 'state'),
                      'command': ['/usr/bin/qemu-system-x86_64'], 'stdout': 'cwd ' + str(root)}
            (run / 'result.json').write_text(json.dumps(report))
            for name in ('serial.txt', 'stderr.txt'):
                (run / name).write_text('fixture')
            summary = {'status': 'completed', 'passed': True, 'source_sha256': 'source',
                       'stages': [{'stage': 'install', 'run_directory': str(run),
                                   'state_directory': str(build / 'run' / 'state')}],
                       'host': [{'stdout': 'build ' + str(root / 'target')}]}
            (build / 'update-gate.json').write_text(json.dumps(summary))
            with mock.patch.object(capture_update, 'ROOT', root), mock.patch.object(capture_update, 'BUILD', build), \
                 mock.patch.object(sys, 'argv', ['capture_update.py', '--destination', str(destination)]):
                self.assertEqual(capture_update.main(), 0)
            captured = json.loads((destination / 'update-gate.json').read_text())
            stage = json.loads((destination / '0-install' / 'result.json').read_text())
            self.assertEqual(captured['stages'][0]['run_directory'], 'build/run/update-example')
            self.assertEqual(stage['run_directory'], 'build/run/update-example')
            self.assertEqual(stage['state_directory'], 'build/run/state')
            self.assertNotIn(str(root), json.dumps([captured, stage]))
            self.assertNotIn('/usr/bin/', json.dumps([captured, stage]))
            for line in (destination / 'SHA256SUMS').read_text().splitlines():
                digest, name = line.split('  ', 1)
                self.assertEqual(digest, hashlib.sha256((destination / name).read_bytes()).hexdigest())


if __name__ == '__main__':
    unittest.main()
