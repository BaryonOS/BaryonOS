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
    def inventory(self, tool=None, strict=True, *, strict_host_tests=False,
                  tool_name='cargo', mutate=None,
                  compiler='rustc 1.90.0 (synthetic)', query_fault=None,
                  rustup=None, wrong_target=False, compiler_args=None):
        if compiler_args is None:
            compiler_args = ['+1.90.0']
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
                result = {'available': True, 'probe_exit': 0, 'path': 'synthetic-' + name,
                          'version': compiler if name == 'rustc' else name + ' synthetic'}
                if name == 'rustc':
                    result['toolchain_args'] = compiler_args
                return result

            def run(command, **_kwargs):
                self.commands.append(command)
                if command[0] == 'synthetic-rustup':
                    self.assertEqual(command[1:], ['target', 'list', '--installed', '--toolchain', '1.90.0'])
                    return subprocess.CompletedProcess(command, 0, '\n'.join(preflight.REQUIRED_TARGETS) + '\n', '')
                prefix = ['synthetic-rustc', *compiler_args]
                self.assertEqual(command[:len(prefix)], prefix)
                self.assertEqual(command[len(prefix):len(prefix) + 1], ['--print'])
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

            command = ['preflight.py', '--firmware', str(code)]
            if strict:
                command.append('--strict')
            if strict_host_tests:
                command.append('--strict-host-tests')
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
                self.assertEqual(report['missing_host_test_tools'], [])
                self.assertTrue(report['baseline_tools_present'])

    @unittest.skipUnless(preflight.HOST_TEST_TOOLS, 'host C linker check applies to POSIX')
    def test_missing_host_linker_is_reported_separately_and_can_be_strict(self):
        for strict_host_tests, expected_status in ((False, 0), (True, 2)):
            with self.subTest(strict_host_tests=strict_host_tests):
                status, report = self.inventory(
                    {'available': False},
                    strict_host_tests=strict_host_tests,
                    tool_name='cc',
                )
                self.assertEqual(status, expected_status)
                self.assertEqual(report['missing'], [])
                self.assertEqual(report['missing_host_test_tools'], ['cc'])
                self.assertTrue(report['baseline_tools_present'])

    def test_missing_windows_linker_is_reported_separately_and_can_be_strict(self):
        self.assertEqual(preflight.host_test_tools('nt'), ('link.exe',))
        with patch.object(preflight, 'HOST_TEST_TOOLS', preflight.host_test_tools('nt')):
            status, report = self.inventory(
                {'available': False},
                strict_host_tests=True,
                tool_name='link.exe',
            )
        self.assertEqual(status, 2)
        self.assertEqual(report['missing'], [])
        self.assertEqual(report['missing_host_test_tools'], ['link.exe'])
        self.assertTrue(report['baseline_tools_present'])

    def test_windows_linker_probe_uses_msvc_help_switch(self):
        with patch.object(preflight.shutil, 'which', return_value='C:/VS/link.exe'), \
                patch.object(preflight.subprocess, 'run') as run:
            run.return_value = subprocess.CompletedProcess(
                ['C:/VS/link.exe', '/?'], 0, 'Microsoft Incremental Linker', '')
            result = preflight.probe('link.exe')
        self.assertTrue(result['available'])
        self.assertEqual(result['probe_exit'], 0)
        self.assertEqual(run.call_args.args[0], ['C:/VS/link.exe', '/?'])

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
        status, report = self.inventory(compiler_args=[])
        self.assertEqual(status, 0)
        self.assertFalse(report['tools']['rustup']['available'])
        self.assertEqual(report['installed_rust_targets'], list(preflight.REQUIRED_TARGETS))
        self.assertEqual(report['rust_toolchain']['rustup_reported_targets'], [])
        for target in preflight.REQUIRED_TARGETS:
            self.assertIn(['synthetic-rustc', '--print', 'target-libdir', '--target', target], self.commands)
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
        with tempfile.TemporaryDirectory() as root:
            for name in ('rustc', 'cargo'):
                def run(command, **_kwargs):
                    stdout = str(Path(root).resolve()) + '\n' if '--print' in command else name + ' 1.90.0\n'
                    return subprocess.CompletedProcess(command, 0, stdout, '')
                with patch.object(preflight.shutil, 'which', return_value='synthetic-' + name), \
                        patch.object(preflight.subprocess, 'run', side_effect=run) as calls:
                    tool = preflight.probe(name)
                self.assertEqual(calls.call_args_list[0].args[0], ['synthetic-' + name, '+1.90.0', '--version'])
                if name == 'rustc':
                    self.assertEqual(calls.call_args_list[1].args[0], ['synthetic-rustc', '+1.90.0', '--print', 'sysroot'])
                self.assertEqual(tool['selected_toolchain'], '1.90.0')
                self.assertEqual(tool['toolchain_args'], ['+1.90.0'])
                self.assertEqual(tool['invocation_mode'], 'pinned-selector')
                self.assertTrue(preflight.usable(tool))

    def test_direct_binaries_rejecting_selector_use_verified_direct_version(self):
        with tempfile.TemporaryDirectory() as root:
            for name in ('rustc', 'cargo'):
                def run(command, **_kwargs):
                    if '+1.90.0' in command:
                        return subprocess.CompletedProcess(command, 1, '', 'unsupported selector')
                    stdout = str(Path(root).resolve()) + '\n' if '--print' in command else name + ' 1.90.0 (direct)\n'
                    return subprocess.CompletedProcess(command, 0, stdout, '')
                with patch.object(preflight.shutil, 'which', return_value='synthetic-' + name), \
                        patch.object(preflight.subprocess, 'run', side_effect=run) as calls:
                    tool = preflight.probe(name)
                self.assertTrue(preflight.usable(tool))
                self.assertEqual(tool['toolchain_args'], [])
                self.assertEqual(tool['invocation_mode'], 'verified-direct')
                self.assertEqual(calls.call_args_list[1].args[0], ['synthetic-' + name, '--version'])

    def test_direct_rustc_ignoring_selector_for_version_needs_bare_path_queries(self):
        with tempfile.TemporaryDirectory() as root:
            def run(command, **_kwargs):
                if '--version' in command:
                    return subprocess.CompletedProcess(command, 0, 'rustc 1.90.0 (direct)\n', '')
                if '+1.90.0' in command:
                    return subprocess.CompletedProcess(command, 1, '', "couldn't read +1.90.0")
                return subprocess.CompletedProcess(command, 0, str(Path(root).resolve()) + '\n', '')
            with patch.object(preflight.shutil, 'which', return_value='synthetic-rustc'), \
                    patch.object(preflight.subprocess, 'run', side_effect=run):
                tool = preflight.probe('rustc')
            self.assertTrue(preflight.usable(tool))
            self.assertEqual(tool['toolchain_args'], [])
            status, report = self.inventory(tool, tool_name='rustc', compiler_args=[])
            self.assertEqual(status, 0)
            self.assertTrue(report['rust_toolchain']['compiler_valid'])
            self.assertTrue(all('+1.90.0' not in command for command in self.commands))

    def test_wrong_versions_cannot_pass_either_invocation_form(self):
        for name in ('rustc', 'cargo'):
            for version in ('1.89.0', '1.90.0-nightly', '1.91.0', '1.90.00'):
                for direct in (False, True):
                    def run(command, **_kwargs):
                        if direct and '+1.90.0' in command:
                            return subprocess.CompletedProcess(command, 1, '', 'unsupported selector')
                        return subprocess.CompletedProcess(command, 0, name + ' ' + version + '\n', '')
                    with self.subTest(name=name, version=version, direct=direct), \
                            patch.object(preflight.shutil, 'which', return_value='synthetic-' + name), \
                            patch.object(preflight.subprocess, 'run', side_effect=run):
                        tool = preflight.probe(name)
                    self.assertFalse(preflight.usable(tool))
                    self.assertEqual(tool['probe_error'], 'ToolchainVersionMismatch')
                    self.assertNotIn('toolchain_args', tool)

    def test_compiler_capability_crash_or_timeout_is_not_hidden_by_fallback(self):
        for fault in (-6, subprocess.TimeoutExpired('synthetic-rustc', 10)):
            def run(command, **_kwargs):
                if '--version' in command:
                    return subprocess.CompletedProcess(command, 0, 'rustc 1.90.0\n', '')
                if isinstance(fault, BaseException):
                    raise fault
                return subprocess.CompletedProcess(command, fault, '', 'compiler crashed')
            with patch.object(preflight.shutil, 'which', return_value='synthetic-rustc'), \
                    patch.object(preflight.subprocess, 'run', side_effect=run) as calls:
                tool = preflight.probe('rustc')
            self.assertFalse(preflight.usable(tool))
            self.assertEqual(calls.call_count, 2)
            self.assertNotIn('toolchain_args', tool)

    def test_non_rust_probe_has_no_toolchain_selector(self):
        with patch.object(preflight.shutil, 'which', return_value='synthetic-git'), \
                patch.object(preflight.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, 'git version synthetic\n', '')) as calls:
            tool = preflight.probe('git')
        self.assertEqual(calls.call_args.args[0], ['synthetic-git', '--version'])
        self.assertTrue(preflight.usable(tool))
        self.assertNotIn('selected_toolchain', tool)

    def test_cross_target_substitution_within_sysroot_is_rejected(self):
        target, other = preflight.REQUIRED_TARGETS
        for kind in ('target-directory', 'library-directory', 'rlib'):
            def substitute(root):
                own = root / 'lib/rustlib' / target
                foreign = root / 'lib/rustlib' / other
                if kind == 'target-directory':
                    shutil.rmtree(own)
                    own.symlink_to(foreign, target_is_directory=True)
                elif kind == 'library-directory':
                    shutil.rmtree(own / 'lib')
                    (own / 'lib').symlink_to(foreign / 'lib', target_is_directory=True)
                else:
                    archive = own / 'lib/libcore-fixture.rlib'
                    archive.unlink()
                    archive.symlink_to(foreign / 'lib/libcore-fixture.rlib')
            with self.subTest(kind=kind):
                status, report = self.inventory(mutate=substitute)
                self.assertEqual((status, report['missing_rust_targets']), (2, [target]))
                self.assertTrue(report['rust_toolchain']['targets'][other]['available'])

    def test_symlinks_remaining_within_the_same_target_are_valid_inventory(self):
        target = preflight.REQUIRED_TARGETS[0]
        for link_directory in (False, True):
            def relocate(root):
                own = root / 'lib/rustlib' / target
                if link_directory:
                    (own / 'lib').rename(own / 'real-libraries')
                    (own / 'lib').symlink_to(own / 'real-libraries', target_is_directory=True)
                else:
                    (own / 'real-libraries').mkdir()
                    archive = own / 'lib/libcore-fixture.rlib'
                    archive.rename(own / 'real-libraries/libcore-fixture.rlib')
                    archive.symlink_to(own / 'real-libraries/libcore-fixture.rlib')
            with self.subTest(link_directory=link_directory):
                status, report = self.inventory(mutate=relocate)
                self.assertEqual(status, 0)
                self.assertTrue(report['baseline_tools_present'])


if __name__ == '__main__':
    unittest.main()
