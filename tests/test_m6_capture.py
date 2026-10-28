"""Synthetic M6a host-log routing tests; no native build or guest execution."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import capture_m6 as capture
import test_m6 as gate


class HostLogRouting(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.build = self.root / 'build'
        self.logs = self.build / 'logs'
        self.logs.mkdir(parents=True)
        self.stem = 'm6a-user-artifact-tests'
        for name, value in (('ROOT', self.root), ('BUILD', self.build)):
            patcher = patch.object(capture, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)

    def write_fixture(self, suffix, returncode=0):
        """Exercise the actual host-file branch; guest coverage is a double."""
        (self.logs / (self.stem + '.stdout.txt')).write_text('')
        (self.logs / (self.stem + '.stderr.txt')).write_text('Synthetic unittest result fixture.\n')
        (self.logs / (self.stem + '.json')).write_text(json.dumps({'returncode': returncode}))
        suite = {'status': 'completed', 'passed': True, 'source_sha256': 'synthetic-source',
                 'results': [{'name': 'm6-user-artifact-tests', 'passed': True,
                              'evidence': 'build/logs/' + self.stem + suffix,
                              'requirements': ['M6A-USER-ELF']}]}
        (self.build / 'm6-test-results.json').write_text(json.dumps(suite))

    def run_capture(self):
        with patch.object(capture, 'source_hash', return_value='synthetic-source'), \
                patch.object(capture, 'validate_coverage'):
            capture.capture()

    def test_stdout_and_stderr_evidence_capture_the_complete_host_log_trio(self):
        for suffix in ('.stdout.txt', '.stderr.txt'):
            with self.subTest(evidence=suffix):
                self.write_fixture(suffix)
                destination = self.root / 'docs/verification/m6a'
                if destination.exists():
                    for path in destination.iterdir():
                        path.unlink()
                self.run_capture()
                for member in ('.stdout.txt', '.json', '.stderr.txt'):
                    original = (self.logs / (self.stem + member)).read_text().rstrip()
                    expected = original + '\n' if original else ''
                    self.assertEqual((destination / (self.stem + member)).read_text(), expected)

    def test_stdout_and_stderr_evidence_reject_failed_host_commands(self):
        for suffix in ('.stdout.txt', '.stderr.txt'):
            with self.subTest(evidence=suffix):
                self.write_fixture(suffix, returncode=1)
                with self.assertRaisesRegex(ValueError, 'Failed host command'):
                    self.run_capture()

    def test_stderr_evidence_requires_metadata_and_stdout_members(self):
        for missing in ('.json', '.stdout.txt'):
            with self.subTest(missing=missing):
                self.write_fixture('.stderr.txt')
                (self.logs / (self.stem + missing)).unlink()
                with self.assertRaises(FileNotFoundError):
                    self.run_capture()


class UserArtifactEvidenceProducer(unittest.TestCase):
    def test_user_artifact_group_points_to_the_result_on_stderr(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            build = root / 'build'
            logs = build / 'logs'
            logs.mkdir(parents=True)

            def command_double(args, stem):
                (logs / (stem + '.stdout.txt')).write_text('')
                (logs / (stem + '.stderr.txt')).write_text('Synthetic unittest result fixture.\n')

            stop = RuntimeError('synthetic stop before any guest execution')
            with patch.object(gate, 'ROOT', root), patch.object(gate, 'BUILD', build), \
                    patch.object(gate, 'source_hash', return_value='synthetic-source'), \
                    patch.object(gate, 'run_logged', side_effect=command_double), \
                    patch.object(gate, 'build'), \
                    patch.object(gate, 'run_m6', side_effect=stop), \
                    self.assertRaises(RuntimeError) as raised:
                gate.test_m6()
            self.assertIs(raised.exception, stop)
            suite = json.loads((build / 'm6-test-results.json').read_text())
            group = next(group for group in suite['results'] if group['name'] == 'm6-user-artifact-tests')
            self.assertEqual(group['requirements'], ['M6A-USER-ELF'])
            self.assertEqual(group['evidence'], 'build/logs/m6a-user-artifact-tests.stderr.txt')
            self.assertEqual((root / group['evidence']).read_text(), 'Synthetic unittest result fixture.\n')


if __name__ == '__main__':
    unittest.main()
