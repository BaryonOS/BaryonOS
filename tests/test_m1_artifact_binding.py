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
import test as m1_gate


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
        self.manifest = {'kernel': {'sha256': digest(self.kernel)},
                         'loader': {'sha256': digest(self.loader)}}
        for owner, name, value in ((runner, 'ROOT', self.root),
                                   (runner, 'BUILD', self.build),
                                   (m1_gate, 'BUILD', self.build)):
            patcher = patch.object(owner, name, value)
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


    def test_private_candidate_survives_later_public_esp_replacement(self):
        private = m1_gate.snapshot_build(self.manifest, 'normal')
        (self.esp/'kernel.elf').write_bytes(b'later unrelated kernel')
        (self.esp/'EFI/BOOT/BOOTX64.EFI').write_bytes(b'later unrelated loader')
        self.assertEqual((private/'kernel.elf').read_bytes(), self.kernel)
        self.assertEqual((private/'EFI/BOOT/BOOTX64.EFI').read_bytes(), self.loader)
        self.assertNotEqual(private, self.esp)
        self.assertTrue(private.is_relative_to(self.build))
        with patch.object(runner.subprocess, 'run', side_effect=self.fake_process), \
                patch.object(runner.subprocess, 'check_output', return_value='synthetic version'), \
                redirect_stdout(io.StringIO()):
            report = runner.run_guest('private-candidate', esp=private, **self.expected)
        copied = Path(report['run_directory'])/'esp'
        self.assertEqual((copied/'kernel.elf').read_bytes(), self.kernel)
        self.assertEqual((copied/'EFI/BOOT/BOOTX64.EFI').read_bytes(), self.loader)
        self.assertTrue(report['artifact_binding_passed'])
        self.assertFalse(report['passed'])  # No guest or fabricated guest success.

    def test_bad_candidate_is_rejected_before_host_tests_or_guest(self):
        for relative in ('kernel.elf', 'EFI/BOOT/BOOTX64.EFI'):
            (self.esp/'kernel.elf').write_bytes(self.kernel)
            (self.esp/'EFI/BOOT/BOOTX64.EFI').write_bytes(self.loader)
            (self.esp/relative).write_bytes(b'stale artifact before candidate copy')
            with self.subTest(relative=relative), \
                    patch.object(m1_gate, 'build', return_value=self.manifest), \
                    patch.object(m1_gate, 'run_logged') as host, \
                    patch.object(m1_gate, 'run_guest') as guest, \
                    self.assertRaisesRegex(ValueError, 'M1 candidate artifact SHA256 mismatch'):
                m1_gate.main()
            host.assert_not_called()
            guest.assert_not_called()

    def test_gate_uses_private_builds_for_cold_boots_and_negative_fixtures(self):
        # Only an orchestration test with labelled synthetic return values.
        # It runs no compiler/VM and stores its mock suite only in TemporaryDirectory.
        builds, calls, candidates = {}, [], {}

        def fake_build(**kwargs):
            name = kwargs.get('label', 'normal')
            kernel = b'synthetic kernel '+name.encode()
            loader = b'synthetic loader '+name.encode()
            (self.esp/'kernel.elf').write_bytes(kernel)
            (self.esp/'EFI/BOOT/BOOTX64.EFI').write_bytes(loader)
            manifest = {'kernel': {'sha256': digest(kernel)}, 'loader': {'sha256': digest(loader)}}
            builds[name] = (kernel, loader, manifest)
            return manifest

        def replace_public():
            (self.esp/'kernel.elf').write_bytes(b'unrelated public kernel')
            (self.esp/'EFI/BOOT/BOOTX64.EFI').write_bytes(b'unrelated public loader')

        def fake_host(*args, **kwargs):
            self.assertEqual(len(list(self.build.glob('m1-candidate-normal-*/esp'))), 1)
            replace_public()

        def fake_guest(label, *, esp, **bindings):
            calls.append(label)
            self.assertNotEqual(esp, self.esp)
            name = 'normal' if label.startswith('cold-') or label in ('missing', 'corrupt') else label
            kernel, loader, manifest = builds[name]
            self.assertEqual((esp/'EFI/BOOT/BOOTX64.EFI').read_bytes(), loader)
            if label in ('missing', 'corrupt'):
                if label == 'missing':
                    self.assertFalse((esp/'kernel.elf').exists())
                else:
                    self.assertEqual((esp/'kernel.elf').read_bytes(), b'\0'+kernel[1:])
                response = dict(raw_qemu_exit=35, stdout='M1:FAIL:synthetic-fixture', timed_out=False)
            else:
                self.assertEqual((esp/'kernel.elf').read_bytes(), kernel)
                self.assertEqual(bindings, m1_gate.expected_artifacts(manifest))
                if name in candidates:
                    self.assertEqual(esp, candidates[name])
                candidates[name] = esp
                response = dict(passed=True, stdout='BOOT:EBS_RETRY' if name == 'ebs-retry' else '')
                if name == 'panic':
                    response = dict(passed=False, raw_qemu_exit=35,
                                    stdout='KERNEL:ENTRY\nM1:FAIL:KERNEL_PANIC', timed_out=False)
                elif name == 'hang':
                    response = dict(passed=False, raw_qemu_exit=None, stdout='KERNEL:ENTRY', timed_out=True)
            replace_public()  # Repeated replacement must not affect the next cold boot or negative fixture.
            return dict(response, run_directory='synthetic-no-guest/'+label)

        with patch.object(m1_gate, 'build', side_effect=fake_build) as compiler, \
                patch.object(m1_gate, 'run_logged', side_effect=fake_host), \
                patch.object(m1_gate, 'run_guest', side_effect=fake_guest), \
                redirect_stdout(io.StringIO()):
            m1_gate.main()
        self.assertEqual(calls, ['cold-1', 'cold-2', 'cold-3', 'missing', 'corrupt', 'ebs-retry', 'panic', 'hang'])
        self.assertEqual(compiler.call_count, 5)  # Four candidates, then normal restoration.
        self.assertEqual(len(set(candidates.values())), 4)


if __name__ == '__main__':
    unittest.main()
