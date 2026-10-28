"""Inventory exit policy with synthetic tool probes; no build or VM evidence."""
from contextlib import redirect_stdout
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import preflight


class RequiredToolProbes(unittest.TestCase):
    def inventory(self, rustup, strict=True):
        def probe(name):
            return rustup if name == 'rustup' else {'available': True, 'probe_exit': 0}

        with tempfile.TemporaryDirectory() as temporary:
            code = Path(temporary)/'OVMF_CODE.fd'
            code.write_bytes(b'synthetic firmware placeholder')
            command = ['preflight.py', '--firmware', str(code)]
            if strict:
                command.append('--strict')
            output = io.StringIO()
            targets = subprocess.CompletedProcess([], 0,
                'x86_64-unknown-uefi\nx86_64-unknown-none\n', '')
            with patch.object(sys, 'argv', command), \
                    patch.object(preflight, 'probe', side_effect=probe), \
                    patch.object(preflight.subprocess, 'run', return_value=targets), \
                    redirect_stdout(output):
                status = preflight.main()
        return status, json.loads(output.getvalue())

    def test_missing_or_failed_probe_fails_strict_inventory(self):
        for tool in ({'available': False},
                     {'available': True, 'path': 'synthetic-rustup', 'probe_exit': -6},
                     {'available': True, 'path': 'synthetic-rustup', 'probe_exit': 1}):
            with self.subTest(tool=tool):
                status, report = self.inventory(tool)
                self.assertEqual(status, 2)
                self.assertEqual(report['missing'], ['rustup'])
                self.assertFalse(report['baseline_tools_present'])
                expected_targets = [] if tool['available'] else [
                    'x86_64-unknown-uefi', 'x86_64-unknown-none']
                self.assertEqual(report['missing_rust_targets'], expected_targets)

    def test_zero_and_default_zero_probe_exit_succeed(self):
        for tool in ({'available': True, 'path': 'synthetic-rustup', 'probe_exit': 0},
                     {'available': True, 'path': 'synthetic-rustup'}):
            with self.subTest(tool=tool):
                status, report = self.inventory(tool)
                self.assertEqual(status, 0)
                self.assertEqual(report['missing'], [])
                self.assertTrue(report['baseline_tools_present'])

    def test_non_strict_inventory_reports_crash_without_failing(self):
        status, report = self.inventory(
            {'available': True, 'path': 'synthetic-rustup', 'probe_exit': -6}, strict=False)
        self.assertEqual(status, 0)
        self.assertEqual(report['missing'], ['rustup'])
        self.assertFalse(report['baseline_tools_present'])


if __name__ == '__main__':
    unittest.main()
