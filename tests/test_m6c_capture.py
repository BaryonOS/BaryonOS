"""Portable synthetic capture-validator regressions; no guest execution/evidence.

Requirements: bind distinct host requirements to their actual command contracts,
and require every fixed guest artifact before validating any claimed hashes.
All inputs live in a fresh temporary directory; no previous VM run is needed.
"""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import capture_m6c as capture


HOST_CONTRACTS = {
    'strict-development-inventory': ('m6c-preflight', 'python',
        ['-B', 'tools/preflight.py', '--strict']),
    'catalog-core-and-wire-host-tests': ('m6c-host-tests', 'cargo',
        ['+1.90.0', 'test', '--locked', '--offline', '-p', 'catalog-core']),
    'external-oracle-and-crash-harness-tests': ('m6c-harness-tests', 'python',
        ['-B', '-m', 'unittest', 'discover', '-s', 'tests', '-p', 'test_m6c_*.py']),
}
M1_KEYS = {'esp/EFI/BOOT/BOOTX64.EFI', 'esp/kernel.elf', 'OVMF_CODE.fd', 'OVMF_VARS.template.fd'}
M6C_KEYS = M1_KEYS | {'OVMF_VARS.fd', 'test-disk.before.img', 'test-disk.after.img',
                      'build-manifest.json', 'volume-manifest.json'}


class HostEvidenceBinding(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.build = self.root/'build'
        self.logs = self.build/'logs'
        self.logs.mkdir(parents=True)
        self.executables = {'python': str(self.root/'tool-bin/python'),
                            'cargo': str(self.root/'tool-bin/cargo')}
        for name, value in (('ROOT', self.root), ('BUILD', self.build)):
            patcher = patch.object(capture, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)
        # These are command-resolution doubles, not installed test tools. No
        # subprocess is executed and the fixture does not need Rust or QEMU.
        patcher = patch.object(capture, 'environment', return_value={'PATH': 'synthetic-tool-path'})
        patcher.start()
        self.addCleanup(patcher.stop)
        patcher = patch.object(capture.shutil, 'which', side_effect=lambda name, path:
            self.executables['python' if name == sys.executable else 'cargo'])
        patcher.start()
        self.addCleanup(patcher.stop)
        for name in HOST_CONTRACTS:
            self.write_host(name)

    def files(self, name):
        stem = HOST_CONTRACTS[name][0]
        return tuple(self.logs/(stem+suffix) for suffix in ('.stdout.txt', '.json', '.stderr.txt'))

    def write_host(self, name, metadata=None):
        _, program, arguments = HOST_CONTRACTS[name]
        stdout, meta, stderr = self.files(name)
        stdout.write_text('Synthetic host-validator fixture; not executed test output.\n', encoding='utf-8')
        stderr.write_text('', encoding='utf-8')
        if metadata is None:
            metadata = {'command': [self.executables[program], *arguments],
                        'cwd': str(self.root), 'returncode': 0}
        meta.write_text(json.dumps(metadata), encoding='utf-8')

    def test_exact_three_separate_host_contracts_are_accepted(self):
        seen = set()
        files = []
        for name in HOST_CONTRACTS:
            actual = capture.check_host_group(name, self.files(name)[0], seen)
            self.assertEqual(actual, self.files(name))
            files.extend(actual)
        self.assertEqual(len(files), 9)
        self.assertEqual(len(set(files)), 9)

    def test_preflight_cannot_replace_rust_or_python_test_log_paths(self):
        preflight = self.files('strict-development-inventory')[0]
        for name in ('catalog-core-and-wire-host-tests', 'external-oracle-and-crash-harness-tests'):
            with self.subTest(group=name), self.assertRaisesRegex(ValueError, 'fixed exact log path'):
                capture.check_host_group(name, preflight)

    def test_renaming_preflight_metadata_does_not_satisfy_test_commands(self):
        preflight = json.loads(self.files('strict-development-inventory')[1].read_text())
        for name in ('catalog-core-and-wire-host-tests', 'external-oracle-and-crash-harness-tests'):
            self.write_host(name, preflight)
            with self.subTest(group=name), self.assertRaisesRegex(ValueError, 'command/working-directory/exit'):
                capture.check_host_group(name, self.files(name)[0])

    def test_exact_argv_cwd_exit_and_timeout_are_required(self):
        name = 'catalog-core-and-wire-host-tests'
        original = json.loads(self.files(name)[1].read_text())
        mutations = [
            ('different-executable', {'command': ['other/cargo', *original['command'][1:]]}),
            ('missing-lock', {'command': [arg for arg in original['command'] if arg != '--locked']}),
            ('wrong-package', {'command': [*original['command'][:-1], 'boot-protocol']}),
            ('different-cwd', {'cwd': str(self.root/'elsewhere')}),
            ('nonzero-exit', {'returncode': 1}),
            ('missing-exit', {'returncode': None}),
            ('boolean-exit', {'returncode': False}),
            ('float-exit', {'returncode': 0.0}),
            ('timed-out', {'timed_out': True}),
        ]
        for label, change in mutations:
            self.write_host(name, {**original, **change})
            with self.subTest(mutation=label), self.assertRaisesRegex(ValueError, 'command/working-directory/exit'):
                capture.check_host_group(name, self.files(name)[0])

    def test_one_log_trio_cannot_count_twice(self):
        name = 'strict-development-inventory'
        seen = set()
        capture.check_host_group(name, self.files(name)[0], seen)
        with self.assertRaisesRegex(ValueError, 'reuse the same physical log'):
            capture.check_host_group(name, self.files(name)[0], seen)

    def test_missing_member_of_host_log_trio_is_rejected(self):
        name = 'catalog-core-and-wire-host-tests'
        self.files(name)[2].unlink()
        with self.assertRaisesRegex(ValueError, 'separate regular files'):
            capture.check_host_group(name, self.files(name)[0])

    def test_capture_rejects_substitution_before_touching_existing_archive(self):
        destination = self.root/'docs/verification/m6c'
        destination.mkdir(parents=True)
        preserved = destination/'preserved.txt'
        preserved.write_bytes(b'existing archive remains unchanged')
        groups = [{'name': name, 'passed': True, 'evidence': 'unused-later-fixture'}
                  for name, _, _ in capture.required_groups()]
        groups[0]['evidence'] = str(self.files('strict-development-inventory')[0])
        groups[1]['evidence'] = groups[0]['evidence']
        suite = {'status': 'completed', 'passed': True, 'source_sha256': 'synthetic-source',
                 'host_test_sha256': {}, 'results': groups}
        (self.build/'m6c-test-results.json').write_text(json.dumps(suite), encoding='utf-8')
        with patch.object(capture, 'source_hash', return_value='synthetic-source'), \
                patch.object(capture, 'test_sources', return_value={}), \
                patch.object(capture, '_replace_archive') as replace, \
                self.assertRaisesRegex(ValueError, 'fixed exact log path'):
            capture.capture()
        replace.assert_not_called()
        self.assertEqual(preserved.read_bytes(), b'existing archive remains unchanged')
        self.assertEqual(list(destination.iterdir()), [preserved])


class RequiredArtifactBinding(unittest.TestCase):
    def test_complete_fixed_m6c_and_m1_key_sets_are_accepted(self):
        self.assertEqual(len(M6C_KEYS), 9)
        self.assertEqual(len(M1_KEYS), 4)
        for phase, names in (('m6c', M6C_KEYS), ('m1', M1_KEYS)):
            capture.require_artifacts({'artifact_sha256': dict.fromkeys(names, '0'*64)}, phase)

    def test_each_missing_artifact_is_rejected_before_raw_hash_validation(self):
        for check, names in ((capture.check_run, M6C_KEYS), (capture.check_m1_run, M1_KEYS)):
            for missing in names:
                with self.subTest(check=check.__name__, missing=missing), \
                        patch.object(capture, 'raw_checks') as raw, \
                        self.assertRaisesRegex(ValueError, 'artifact set'):
                    check(Path('synthetic-run'), {'artifact_sha256': dict.fromkeys(names-{missing}, '0'*64)}, 'source')
                raw.assert_not_called()

    def test_empty_missing_extra_and_wrong_phase_artifact_maps_are_rejected(self):
        for phase, names in (('m6c', M6C_KEYS), ('m1', M1_KEYS)):
            wrong_phase = M1_KEYS if phase == 'm6c' else M6C_KEYS
            records = [
                {}, {'artifact_sha256': {}}, {'artifact_sha256': []},
                {'artifact_sha256': dict.fromkeys(names | {'unapproved.bin'}, '0'*64)},
                {'artifact_sha256': dict.fromkeys(wrong_phase, '0'*64)},
            ]
            for report in records:
                with self.subTest(phase=phase, report=report), self.assertRaisesRegex(ValueError, 'artifact set'):
                    capture.require_artifacts(report, phase)


if __name__ == '__main__':
    unittest.main()
