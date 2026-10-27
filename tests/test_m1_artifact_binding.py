"""M1 snapshot-binding regressions with temporary bytes and subprocess doubles.

These tests do not build or run a guest. They exercise the production runner's
copy/hash/start ordering without depending on any existing build or VM files.
"""
from contextlib import redirect_stdout
import hashlib
import io
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import run as runner


def digest(value):
    return hashlib.sha256(value).hexdigest()


class SnapshotArtifactBinding(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.build = self.root/'build'
        self.esp = self.build/'esp'
        (self.esp/'EFI/BOOT').mkdir(parents=True)
        self.kernel = b'synthetic intended M1 kernel bytes'
        self.loader = b'synthetic intended loader bytes'
        (self.esp/'kernel.elf').write_bytes(self.kernel)
        (self.esp/'EFI/BOOT/BOOTX64.EFI').write_bytes(self.loader)
        self.code = self.root/'code.fd'
        self.variables = self.root/'vars.fd'
        self.code.write_bytes(b'synthetic code template')
        self.variables.write_bytes(b'synthetic vars template')
        self.qemu = str(self.root/'tool-bin/qemu-system-x86_64')
        self.expected = {'expected_kernel_sha256': digest(self.kernel),
                         'expected_loader_sha256': digest(self.loader)}
        for name, value in (('ROOT', self.root), ('BUILD', self.build)):
            patcher = patch.object(runner, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)
        for name, value in (('environment', {'PATH': 'synthetic-tool-path'}),
                            ('firmware', (self.code, self.variables)),
                            ('source_hash', 'synthetic-source')):
            patcher = patch.object(runner, name, return_value=value)
            patcher.start()
            self.addCleanup(patcher.stop)
        patcher = patch.object(runner.shutil, 'which', return_value=self.qemu)
        patcher.start()
        self.addCleanup(patcher.stop)

    def assert_mismatch_without_child(self, **arguments):
        with patch.object(runner.subprocess, 'run') as child, \
                patch.object(runner.subprocess, 'check_output') as version, \
                self.assertRaisesRegex(ValueError, 'Snapshotted artifact SHA256 mismatch'):
            runner.run_guest('snapshot-regression', **arguments)
        child.assert_not_called()
        version.assert_not_called()

    def test_stale_kernel_in_shared_esp_is_rejected_before_any_child(self):
        stale = b'synthetic old M5-no-device kernel bytes'
        (self.esp/'kernel.elf').write_bytes(stale)
        self.assert_mismatch_without_child(**self.expected)
        snapshots = list((self.build/'run').glob('*/esp/kernel.elf'))
        self.assertEqual(len(snapshots), 1)
        self.assertEqual(snapshots[0].read_bytes(), stale)

    def test_guard_reads_copied_snapshot_not_original_esp(self):
        original_copytree = runner.shutil.copytree

        def changed_copy(source, destination, *args, **kwargs):
            result = original_copytree(source, destination, *args, **kwargs)
            (Path(destination)/'kernel.elf').write_bytes(b'changed only in copied snapshot')
            return result

        with patch.object(runner.shutil, 'copytree', side_effect=changed_copy):
            self.assert_mismatch_without_child(**self.expected)
        self.assertEqual((self.esp/'kernel.elf').read_bytes(), self.kernel)

    def test_wrong_loader_snapshot_is_also_rejected_before_any_child(self):
        (self.esp/'EFI/BOOT/BOOTX64.EFI').write_bytes(b'different loader build')
        self.assert_mismatch_without_child(**self.expected)

    def fake_process(self, command, **kwargs):
        if command[0] == self.qemu:
            # Deliberately no PASS output: this double cannot supply guest proof.
            return subprocess.CompletedProcess(command, 0, b'', b'')
        if command == ['git', 'rev-parse', 'HEAD']:
            return subprocess.CompletedProcess(command, 0, 'synthetic-commit\n', '')
        raise AssertionError(f'Unexpected subprocess request: {command}')

    def test_matching_snapshot_reaches_runner_with_recorded_binding(self):
        with patch.object(runner.subprocess, 'run', side_effect=self.fake_process) as child, \
                patch.object(runner.subprocess, 'check_output', return_value='synthetic version'), \
                redirect_stdout(io.StringIO()):
            report = runner.run_guest('matching-snapshot', **self.expected)
        self.assertEqual(child.call_count, 2)
        self.assertEqual(child.call_args_list[0].args[0][0], self.qemu)
        self.assertEqual(report['expected_artifact_sha256'], {
            'esp/kernel.elf': digest(self.kernel),
            'esp/EFI/BOOT/BOOTX64.EFI': digest(self.loader),
        })
        self.assertTrue(report['artifact_binding_passed'])
        self.assertFalse(report['passed'])

    def test_intentional_unbound_missing_kernel_fixture_can_still_run(self):
        (self.esp/'kernel.elf').unlink()
        with patch.object(runner.subprocess, 'run', side_effect=self.fake_process) as child, \
                patch.object(runner.subprocess, 'check_output', return_value='synthetic version'), \
                redirect_stdout(io.StringIO()):
            report = runner.run_guest('intentional-missing-kernel')
        self.assertEqual(child.call_args_list[0].args[0][0], self.qemu)
        self.assertEqual(report['expected_artifact_sha256'], {})
        self.assertNotIn('esp/kernel.elf', report['artifact_sha256'])
        self.assertFalse(report['passed'])


if __name__ == '__main__':
    unittest.main()
