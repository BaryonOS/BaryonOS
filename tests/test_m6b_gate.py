"""M6b cleanup contract with doubles; these tests build and boot no artifacts."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import test_m6b as gate


class FailedGateRestoration(unittest.TestCase):
    def test_restore_preserves_original_error_and_failed_suite(self):
        for restore_error in (None, RuntimeError('synthetic restore failure')):
            with self.subTest(restore_error=restore_error), tempfile.TemporaryDirectory() as temporary:
                build = Path(temporary)
                original = RuntimeError('synthetic gate failure')

                def restore():
                    # A failed result must already be durable before cleanup starts.
                    failed = json.loads((build/'m6b-test-results.json').read_text())
                    self.assertEqual(failed['status'], 'failed')
                    self.assertFalse(failed['passed'])
                    if restore_error:
                        raise restore_error

                with patch.object(gate, 'BUILD', build), \
                        patch.object(gate, 'source_hash', return_value='synthetic-source'), \
                        patch.object(gate, 'test_sources', return_value={}), \
                        patch.object(gate, 'run_logged', side_effect=original), \
                        patch.object(gate, 'build', side_effect=restore) as builder, \
                        patch.object(gate, 'run_m6b') as guest, \
                        self.assertRaises(RuntimeError) as raised:
                    gate.test_m6b()

                self.assertIs(raised.exception, original)
                builder.assert_called_once_with()
                guest.assert_not_called()
                suite = json.loads((build/'m6b-test-results.json').read_text())
                self.assertEqual(suite['error'], str(original))
                self.assertEqual(suite['status'], 'failed')
                self.assertFalse(suite['passed'])
                self.assertIn('finished_utc', suite)
                if restore_error:
                    self.assertEqual(suite['restore_error'], str(restore_error))
                else:
                    self.assertNotIn('restore_error', suite)


if __name__ == '__main__':
    unittest.main()
