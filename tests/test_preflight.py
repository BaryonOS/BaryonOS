"""Inventory policy with synthetic compiler/filesystem probes; no build evidence."""
from contextlib import redirect_stdout
import io
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import preflight


def archive_bytes(payload=b'synthetic archive member'):
    header = (b'metadata/'.ljust(16) + b'0'.ljust(12) + b'0'.ljust(6)
              + b'0'.ljust(6) + b'100644'.ljust(8)
              + str(len(payload)).encode().ljust(10) + b'`\n')
    return b'!<arch>\n' + header + payload + (b'\n' if len(payload) % 2 else b'')


class RequiredToolProbes(unittest.TestCase):
    def inventory(self, tool=None, strict=True, *, tool_name='cargo', mutate=None,
                  compiler='rustc 1.90.0 (synthetic)', query_fault=None,
                  rustup=None, wrong_target=False):
        if rustup is None:
            rustup = {'available': False, 'path': None, 'version': None}
        self.commands = []
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / 'sysroot'
            root.mkdir()
            for target in preflight.REQUIRED_TARGETS:
                directory = root / 'lib/rustlib' / target / 'lib'
                directory.mkdir(parents=True)
                for library in preflight.REQUIRED_RLIBS:
                    (directory / ('lib' + library + '-fixture.rlib')).write_bytes(archive_bytes())
            if mutate:
                mutate(root)
            code = Path(temporary) / 'OVMF_CODE.fd'
            code.write_bytes(b'synthetic firmware placeholder')

            def probe(name):
                if name == 'rustup':
                    return rustup
                if name == tool_name and tool is not None:
                    return tool
                return {'available': True, 'probe_exit': 0, 'path': 'synthetic-' + name,
                        'version': compiler if name == 'rustc' else name + ' synthetic'}

            def run(command, **_kwargs):
                self.commands.append(command)
                if command[0] == 'synthetic-rustup':
                    self.assertEqual(command[1:], ['target', 'list', '--installed', '--toolchain', '1.90.0'])
                    return subprocess.CompletedProcess(command, 0, '\n'.join(preflight.REQUIRED_TARGETS) + '\n', '')
                self.assertEqual(command[:2], ['synthetic-rustc', '+1.90.0'])
                target = command[-1] if '--target' in command else 'sysroot'
                if query_fault and query_fault[0] == target:
                    failure = query_fault[1]
                    if isinstance(failure, BaseException):
                        raise failure
                    return subprocess.CompletedProcess(command, failure, '', 'synthetic query error')
                result = root if target == 'sysroot' else root / 'lib/rustlib' / target / 'lib'
                if wrong_target and target == preflight.REQUIRED_TARGETS[0]:
                    result = root / 'lib/rustlib' / preflight.REQUIRED_TARGETS[1] / 'lib'
                return subprocess.CompletedProcess(command, 0, str(result) + '\n', '')

            command = ['preflight.py', '--firmware', str(code)] + (['--strict'] if strict else [])
            output = io.StringIO()
            with patch.object(sys, 'argv', command), patch.object(preflight, 'probe', side_effect=probe), \
                    patch.object(preflight.subprocess, 'run', side_effect=run), redirect_stdout(output):
                status = preflight.main()
            report = json.loads(output.getvalue())
        return status, report

    def test_missing_or_failed_probe_fails_strict_inventory(self):
        for tool in ({'available': False}, {'available': True, 'probe_exit': -6},
                     {'available': True, 'probe_exit': 1}):
            with self.subTest(tool=tool):
                status, report = self.inventory(tool)
                self.assertEqual(status, 2)
                self.assertEqual(report['missing'], ['cargo'])
                self.assertFalse(report['baseline_tools_present'])
                self.assertEqual(report['missing_rust_targets'], [])

    def test_zero_and_default_zero_probe_exit_succeed(self):
        for tool in ({'available': True, 'probe_exit': 0}, {'available': True}):
            with self.subTest(tool=tool):
                status, report = self.inventory(tool)
                self.assertEqual(status, 0)
                self.assertEqual(report['missing'], [])
                self.assertTrue(report['baseline_tools_present'])

    def test_probe_exceptions_fail_strict_inventory(self):
        for error in (OSError('synthetic execution failure'),
                      subprocess.TimeoutExpired('synthetic-cargo', 10)):
            with self.subTest(error=type(error).__name__):
                with patch.object(preflight.shutil, 'which', return_value='synthetic-cargo'), \
                        patch.object(preflight.subprocess, 'run', side_effect=error):
                    tool = preflight.probe('cargo')
                self.assertTrue(tool['available'])
                self.assertEqual(tool['probe_error'], type(error).__name__)
                self.assertNotIn('probe_exit', tool)
                status, report = self.inventory(tool)
                self.assertEqual(status, 2)
                self.assertEqual(report['missing'], ['cargo'])
                self.assertFalse(report['baseline_tools_present'])

    def test_non_strict_inventory_reports_probe_exceptions_without_failing(self):
        for error in ('OSError', 'TimeoutExpired'):
            status, report = self.inventory({'available': True, 'probe_error': error}, strict=False)
            self.assertEqual(status, 0)
            self.assertEqual(report['missing'], ['cargo'])
            self.assertFalse(report['baseline_tools_present'])

    def test_non_strict_inventory_reports_crash_without_failing(self):
        status, report = self.inventory({'available': True, 'probe_exit': -6}, strict=False)
        self.assertEqual(status, 0)
        self.assertEqual(report['missing'], ['cargo'])
        self.assertFalse(report['baseline_tools_present'])

    def test_standalone_compiler_without_rustup_requires_real_target_archives(self):
        status, report = self.inventory()
        self.assertEqual(status, 0)
        self.assertFalse(report['tools']['rustup']['available'])
        self.assertEqual(report['installed_rust_targets'], list(preflight.REQUIRED_TARGETS))
        self.assertEqual(report['rust_toolchain']['rustup_reported_targets'], [])
        for target in preflight.REQUIRED_TARGETS:
            self.assertIn(['synthetic-rustc', '+1.90.0', '--print', 'target-libdir', '--target', target], self.commands)
            self.assertTrue(report['rust_toolchain']['targets'][target]['available'])

    def test_manager_query_is_pinned_and_never_substitutes_for_target_files(self):
        manager = {'available': True, 'path': 'synthetic-rustup', 'probe_exit': 0}
        status, report = self.inventory(rustup=manager)
        self.assertEqual(status, 0)
        self.assertEqual(report['rust_toolchain']['rustup_reported_targets'], list(preflight.REQUIRED_TARGETS))
        target = preflight.REQUIRED_TARGETS[0]
        status, report = self.inventory(rustup=manager,
            mutate=lambda root: shutil.rmtree(root / 'lib/rustlib' / target))
        self.assertEqual(status, 2)
        self.assertIn(target, report['rust_toolchain']['rustup_reported_targets'])
        self.assertEqual(report['missing_rust_targets'], [target])

    def test_missing_or_failed_optional_manager_does_not_hide_its_status(self):
        for manager in ({'available': False}, {'available': True, 'probe_exit': -6},
                        {'available': True, 'probe_error': 'TimeoutExpired'}):
            status, report = self.inventory(rustup=manager)
            self.assertEqual(status, 0)
            self.assertEqual(report['tools']['rustup'], manager)

    def test_wrong_selected_compiler_version_is_rejected(self):
        for version in ('rustc 1.89.0 (old)', 'rustc 1.90.0-nightly (other)', 'rustc 1.91.0 (new)', None):
            status, report = self.inventory(compiler=version)
            self.assertEqual(status, 2)
            self.assertFalse(report['rust_toolchain']['compiler_valid'])
            self.assertEqual(report['missing_rust_targets'], list(preflight.REQUIRED_TARGETS))

    def test_every_required_target_query_failure_fails_strict(self):
        for target in ('sysroot', *preflight.REQUIRED_TARGETS):
            for fault in (1, OSError('missing compiler'), subprocess.TimeoutExpired('rustc', 10)):
                with self.subTest(target=target, fault=type(fault).__name__):
                    status, report = self.inventory(query_fault=(target, fault))
                    self.assertEqual(status, 2)
                    self.assertFalse(report['baseline_tools_present'])
                    self.assertEqual(report['missing_rust_targets'],
                                     list(preflight.REQUIRED_TARGETS) if target == 'sysroot' else [target])

    def test_missing_target_directory_or_any_required_library_fails_strict(self):
        for target in preflight.REQUIRED_TARGETS:
            status, report = self.inventory(mutate=lambda root: shutil.rmtree(root / 'lib/rustlib' / target))
            self.assertEqual((status, report['missing_rust_targets']), (2, [target]))
            for library in preflight.REQUIRED_RLIBS:
                def remove(root):
                    (root / 'lib/rustlib' / target / 'lib' / ('lib' + library + '-fixture.rlib')).unlink()
                status, report = self.inventory(mutate=remove)
                self.assertEqual((status, report['missing_rust_targets']), (2, [target]))

    def test_empty_nonarchive_and_truncated_rlibs_are_rejected(self):
        target = preflight.REQUIRED_TARGETS[0]
        for content in (b'', b'not a library', b'!<arch>\n', archive_bytes()[:-8]):
            def corrupt(root):
                (root / 'lib/rustlib' / target / 'lib/libcore-fixture.rlib').write_bytes(content)
            status, report = self.inventory(mutate=corrupt)
            self.assertEqual((status, report['missing_rust_targets']), (2, [target]))

    def test_wrong_target_directory_and_symlinks_outside_sysroot_are_rejected(self):
        target = preflight.REQUIRED_TARGETS[0]
        status, report = self.inventory(wrong_target=True)
        self.assertEqual((status, report['missing_rust_targets']), (2, [target]))
        for link_directory in (False, True):
            def outside(root):
                directory = root / 'lib/rustlib' / target / 'lib'
                external = root.parent / 'external'
                external.mkdir()
                (external / 'libcore-fixture.rlib').write_bytes(archive_bytes())
                if link_directory:
                    shutil.rmtree(directory)
                    directory.symlink_to(external, target_is_directory=True)
                else:
                    (directory / 'libcore-fixture.rlib').unlink()
                    (directory / 'libcore-fixture.rlib').symlink_to(external / 'libcore-fixture.rlib')
            status, report = self.inventory(mutate=outside)
            self.assertEqual((status, report['missing_rust_targets']), (2, [target]))

    def test_rustc_and_cargo_version_probes_select_the_pinned_toolchain(self):
        for name in ('rustc', 'cargo'):
            with patch.object(preflight.shutil, 'which', return_value='synthetic-' + name), \
                    patch.object(preflight.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, name + ' 1.90.0\n', '')) as run:
                tool = preflight.probe(name)
            self.assertEqual(run.call_args.args[0], ['synthetic-' + name, '+1.90.0', '--version'])
            self.assertEqual(tool['selected_toolchain'], '1.90.0')


if __name__ == '__main__':
    unittest.main()
