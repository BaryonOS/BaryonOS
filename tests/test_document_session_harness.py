"""Host-only negative tests of DocumentSession evidence binding; no VM logs.

All generated crate files, compiler metadata, ELF bytes and volumes are local
test fixtures. They exercise the harness boundary, not native OS execution.
"""
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import test_document_session as gate

HEAD, SOURCE = '1' * 40, '2' * 64


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value) + '\n')


def host_elf():
    """A minimal inspected ELF fixture; never executed or described as native."""
    data = bytearray(8192 + 32)
    data[:16] = b'\x7fELF\x02\x01\x01' + bytes(9)
    struct.pack_into('<HHIQQQIHHHHHH', data, 16, 2, 62, 1, 0x40000000, 64, 0, 0, 64, 56, 2, 0, 0, 0)
    struct.pack_into('<IIQQQQQQ', data, 64, 1, 5, 4096, 0x40000000, 0x40000000, 16, 16, 4096)
    struct.pack_into('<IIQQQQQQ', data, 120, 1, 6, 8192, 0x40010000, 0x40010000, 32, 32, 4096)
    data[4096:4112] = b'HOST ELF FIXTURE!'
    return bytes(data)


class CompiledBindingTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='document-session-host-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.build = self.root / 'build/private'
        self.build.mkdir(parents=True)
        for name in ('userspace/m6c_probe.rs', 'userspace/m6_runtime.rs', 'userspace/sdk_probe_runtime.rs',
                     'userspace/document_session_probe.rs', 'userspace/m6c_linker.ld', 'sdk/Cargo.toml', 'sdk/Cargo.lock'):
            self.put(self.root / name, ('host source fixture: ' + name).encode())
        for package in gate.PACKAGES:
            for name in ('Cargo.toml', 'src/lib.rs'):
                self.put(self.root / package / name, ('host crate fixture: ' + package + '/' + name).encode())
        self.put(self.root / 'services/volume/src/upload.rs', b'host nested volume source fixture')
        for name in gate.LIBRARY_PATHS:
            self.put(self.build / name, ('!<arch>\nhost rlib fixture: ' + name).encode())
        self.put(self.build / 'users/m6c_probe.elf', host_elf())
        self.put(self.build / 'esp/kernel.elf', b'host kernel binding fixture\0' + host_elf() + b'\0end')
        self.put(self.build / 'esp/EFI/BOOT/BOOTX64.EFI', b'host loader hash fixture')
        self.user = {'document_session': True, 'sdk_runtime': True,
                     'source_sha256': gate.sha256(self.root / 'userspace/m6c_probe.rs'),
                     'linker_sha256': gate.sha256(self.root / 'userspace/m6c_linker.ld'),
                     'module_sha256': gate.selected_sources(self.root),
                     'library_sha256': {name: gate.sha256(self.build / name) for name in gate.LIBRARY_PATHS},
                     'binary_sha256': gate.sha256(self.build / 'users/m6c_probe.elf'),
                     **gate.inspect_user(self.build / 'users/m6c_probe.elf')}
        self.manifest = {'document_session': True, 'sdk_runtime': True, 'kernel_feature': 'milestone-m6c',
                         'loader_feature': None, 'source_fingerprint_version': 3, 'source_sha256': SOURCE,
                         'kernel': {'sha256': gate.sha256(self.build / 'esp/kernel.elf')},
                         'loader': {'sha256': gate.sha256(self.build / 'esp/EFI/BOOT/BOOTX64.EFI')}}
        self.sync()
        self.compilers = {}
        rows = [(crate, source, log, 'rlib', f'users/lib{crate}.rlib') for crate, (source, log) in gate.LIBRARIES.items()]
        rows.append(('os_user_m6c_probe', 'userspace/m6c_probe.rs', 'build-users-m6c_probe', 'bin', 'users/m6c_probe.elf'))
        for crate, source, log, kind, output in rows:
            command = ['/host-fixture/rustc', '+1.90.0', f'--crate-name={crate}', f'--crate-type={kind}',
                       '--target=x86_64-unknown-none', '-o', str(self.build / output), str(self.root / source)]
            for value in ('panic=abort', 'code-model=small', 'relocation-model=static', 'no-redzone=yes'):
                command.extend(['-C', value])
            if kind == 'bin':
                command.extend(['--cfg=nos_sdk_runtime', '--cfg=nos_document_session'])
            for name in gate.DEPENDENCIES[crate]:
                command.extend(['--extern', f'{name}={self.build / ("users/lib" + name + ".rlib")}'])
            metadata = {'command': command, 'cwd': str(self.root), 'returncode': 0}
            self.compilers[log] = metadata
            write_json(self.build / 'logs' / (log + '.json'), metadata)
            for suffix in ('.stdout.txt', '.stderr.txt'):
                self.put(self.build / 'logs' / (log + suffix), b'')

    def put(self, path, raw):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(raw)

    def sync(self):
        write_json(self.build / 'manifest.json', self.manifest)
        write_json(self.build / 'users/manifest-m6c.json', self.user)

    def validate(self, case='normal'):
        return gate.validate_snapshot(self.build, original_build=self.build, root=self.root,
                                      frozen=SOURCE, case=case, kernel=self.build / 'esp/kernel.elf',
                                      loader=self.build / 'esp/EFI/BOOT/BOOTX64.EFI')

    def test_complete_host_binding_for_both_selected_profiles(self):
        self.assertTrue(self.validate()['actual_user_elf_embedded'])
        self.manifest['kernel_feature'] = 'test-m6c-recover'
        self.sync()
        self.assertEqual(self.validate('recover')['case'], 'recover')

    def test_document_and_sdk_flags_must_be_explicit_true_in_both_manifests(self):
        original = copy.deepcopy((self.manifest, self.user))
        for which in (0, 1):
            for field in ('document_session', 'sdk_runtime'):
                for value in (None, False, 1, 'true'):
                    with self.subTest(manifest=which, field=field, value=value):
                        self.manifest, self.user = copy.deepcopy(original)
                        target = (self.manifest, self.user)[which]
                        if value is None:
                            target.pop(field)
                        else:
                            target[field] = value
                        self.sync()
                        with self.assertRaisesRegex(ValueError, 'explicitly select'):
                            self.validate()

    def test_missing_selected_library_source_is_rejected(self):
        self.user['module_sha256'].pop('sdk/document-session/src/lib.rs')
        self.sync()
        with self.assertRaisesRegex(ValueError, 'selected library/probe source'):
            self.validate()

    def test_swapped_or_changed_source_hashes_are_rejected(self):
        modules = self.user['module_sha256']
        a, b = 'sdk/document-session/src/lib.rs', 'sdk/catalog-client/src/lib.rs'
        modules[a], modules[b] = modules[b], modules[a]
        self.sync()
        with self.assertRaisesRegex(ValueError, 'selected library/probe source'):
            self.validate()

    def test_missing_rlib_manifest_entry_and_swapped_rlib_bytes_are_rejected(self):
        original = self.user['library_sha256'].copy()
        self.user['library_sha256'].pop('users/libcatalog_core.rlib')
        self.sync()
        with self.assertRaisesRegex(ValueError, 'native library artifact'):
            self.validate()
        self.user['library_sha256'] = original
        self.sync()
        self.put(self.build / 'users/libcatalog_core.rlib', (self.build / 'users/libnieuw_os_document_session.rlib').read_bytes())
        with self.assertRaisesRegex(ValueError, 'Native library bytes differ'):
            self.validate()

    def test_elf_swap_fails_even_with_relabelled_user_hash(self):
        elf = bytearray(host_elf())
        elf[-1] = 7
        self.put(self.build / 'users/m6c_probe.elf', bytes(elf))
        self.user['binary_sha256'] = gate.sha256(self.build / 'users/m6c_probe.elf')
        self.sync()
        with self.assertRaisesRegex(ValueError, 'does not embed'):
            self.validate()

    def test_invalid_elf_cannot_be_accepted_by_relabelling_both_hashes(self):
        self.put(self.build / 'users/m6c_probe.elf', b'not an ELF')
        self.put(self.build / 'esp/kernel.elf', b'not an ELF')
        self.user['binary_sha256'] = gate.sha256(self.build / 'users/m6c_probe.elf')
        self.manifest['kernel']['sha256'] = gate.sha256(self.build / 'esp/kernel.elf')
        self.sync()
        with self.assertRaisesRegex(ValueError, 'not ELF64'):
            self.validate()

    def test_compiler_must_select_document_session_and_all_five_actual_libraries(self):
        log = 'build-users-m6c_probe'
        original = copy.deepcopy(self.compilers[log])
        for change in ('cfg', 'library'):
            metadata = copy.deepcopy(original)
            if change == 'cfg':
                metadata['command'].remove('--cfg=nos_document_session')
            else:
                metadata['command'][-1] = 'nieuw_os_document_session=/foreign/other.rlib'
            write_json(self.build / 'logs' / (log + '.json'), metadata)
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.validate()

    def test_library_compiler_entry_source_cannot_be_swapped(self):
        log = 'build-document-library-nieuw_os_document_session'
        metadata = self.compilers[log]
        command = metadata['command']
        command[command.index(str(self.root / 'sdk/document-session/src/lib.rs'))] = str(self.root / 'sdk/catalog-client/src/lib.rs')
        write_json(self.build / 'logs' / (log + '.json'), metadata)
        with self.assertRaisesRegex(ValueError, 'Compiler entry source mismatch'):
            self.validate()

    def test_volume_sources_and_actual_rlib_cannot_be_omitted(self):
        for name in ('services/volume/src/lib.rs', 'services/volume/src/upload.rs'):
            digest = self.user['module_sha256'].pop(name)
            self.sync()
            with self.subTest(source=name), self.assertRaisesRegex(ValueError, 'selected library/probe source'):
                self.validate()
            self.user['module_sha256'][name] = digest
        self.user['library_sha256'].pop('users/libvolume_core.rlib')
        self.sync()
        with self.assertRaisesRegex(ValueError, 'native library artifact'):
            self.validate()

    def test_document_library_must_link_the_selected_volume_library(self):
        log = 'build-document-library-nieuw_os_document_session'
        metadata = self.compilers[log]
        argument = f'volume_core={self.build / "users/libvolume_core.rlib"}'
        position = metadata['command'].index(argument)
        del metadata['command'][position - 1:position + 1]
        write_json(self.build / 'logs' / (log + '.json'), metadata)
        with self.assertRaisesRegex(ValueError, 'dependency set mismatch'):
            self.validate()

    def test_each_rlib_requires_its_exact_dependency_set(self):
        for crate, (_, log) in gate.LIBRARIES.items():
            original = copy.deepcopy(self.compilers[log])
            mutations = []
            extra = copy.deepcopy(original)
            extra['command'].extend(['--extern', 'foreign=/foreign/unselected.rlib'])
            mutations.append(('extra', extra))
            if gate.DEPENDENCIES[crate]:
                dependency = gate.DEPENDENCIES[crate][0]
                argument = f'{dependency}={self.build / ("users/lib" + dependency + ".rlib")}'
                for change in ('foreign', 'missing', 'duplicate'):
                    metadata = copy.deepcopy(original)
                    position = metadata['command'].index(argument)
                    if change == 'foreign':
                        metadata['command'][position] = dependency + '=/foreign/unbound.rlib'
                    elif change == 'missing':
                        del metadata['command'][position - 1:position + 1]
                    else:
                        metadata['command'].extend(['--extern', argument])
                    mutations.append((change, metadata))
            for change, metadata in mutations:
                write_json(self.build / 'logs' / (log + '.json'), metadata)
                with self.subTest(crate=crate, change=change), self.assertRaisesRegex(ValueError, 'dependency set mismatch'):
                    self.validate()
                write_json(self.build / 'logs' / (log + '.json'), original)

    def test_abi_key_duplicates_or_conflicts_before_and_after_are_rejected(self):
        log = 'build-document-library-nieuw_os_catalog_client'
        original = copy.deepcopy(self.compilers[log])
        conflicts = {'panic': 'unwind', 'code-model': 'large', 'relocation-model': 'pic', 'no-redzone': 'no'}
        for key, selected in gate.ABI_OPTIONS.items():
            for position in ('before', 'after'):
                for value in (selected, conflicts[key]):
                    metadata = copy.deepcopy(original)
                    extra = ['-C', key + '=' + value]
                    if position == 'before':
                        metadata['command'][2:2] = extra
                    else:
                        metadata['command'].extend(extra)
                    write_json(self.build / 'logs' / (log + '.json'), metadata)
                    with self.subTest(key=key, position=position, value=value), self.assertRaisesRegex(ValueError, 'exactly one ' + key):
                        self.validate()

    def test_attached_long_and_underscore_codegen_spellings_cannot_hide_duplicates(self):
        log = 'build-document-library-nieuw_os_catalog_client'
        original = copy.deepcopy(self.compilers[log])
        alternatives = (['-Cno-redzone=no'], ['-C=no-redzone=no'], ['--codegen', 'no-redzone=no'],
                        ['--codegen=no-redzone=no'], ['-C', 'no_redzone=no'], ['-C', 'no_redzone=yes'])
        for extra in alternatives:
            metadata = copy.deepcopy(original)
            metadata['command'].extend(extra)
            write_json(self.build / 'logs' / (log + '.json'), metadata)
            with self.subTest(extra=extra), self.assertRaisesRegex(ValueError, 'exactly one no-redzone'):
                self.validate()

    def test_wrong_build_source_or_profile_stops_binding(self):
        for field, value in [('source_sha256', '9' * 64), ('kernel_feature', 'milestone-m6b'), ('loader_feature', 'graphics-g0')]:
            previous = self.manifest[field]
            self.manifest[field] = value
            self.sync()
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.validate()
            self.manifest[field] = previous

    def test_symlinked_library_is_rejected_even_if_contents_match(self):
        library = self.build / 'users/libcatalog_core.rlib'
        foreign = self.root / 'outside-library.rlib'
        foreign.write_bytes(library.read_bytes())
        library.unlink()
        library.symlink_to(foreign)
        with self.assertRaisesRegex(ValueError, 'nonregular artifact'):
            self.validate()

    def test_duplicate_or_oversized_json_is_rejected(self):
        path = self.build / 'host-fixture.json'
        path.write_text('{"document_session":true,"document_session":false}')
        with self.assertRaisesRegex(ValueError, 'Duplicate'):
            gate.read_json(path)
        path.write_bytes(b' ' * (gate.MAX_JSON + 1))
        with self.assertRaisesRegex(ValueError, 'Oversized'):
            gate.read_json(path)


class ColdLinkTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='document-session-cold-host-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        a, b = self.root / 'normal', self.root / 'recover'
        a.mkdir()
        b.mkdir()
        raw = b'host cold-volume fixture'.ljust(65536, b'\0')
        for path in (a / 'test-disk.after.img', b / 'test-disk.before.img', b / 'test-disk.after.img', self.root / 'volume.img'):
            path.write_bytes(raw)
        common = {'base_commit': HEAD, 'source_fingerprint_version': 3, 'source_sha256': SOURCE,
                  'observed_source_sha256': SOURCE, 'source_unchanged': True,
                  'volume_path': str(self.root / 'volume.img')}
        self.normal = dict(common, case='normal', run_directory=str(a), previous_stage=None,
                           disk_oracle={'identity': 'host-fixture', 'state': {'sequence': 10}, 'observed': {'acks': 10}})
        self.recover = dict(common, case='recover', run_directory=str(b), previous_stage=str(a),
                            disk_oracle={'identity': 'host-fixture', 'state': {'sequence': 10}, 'observed': {'acks': 0}})

    def validate(self):
        return gate.cold_link(self.normal, self.recover, head=HEAD, frozen=SOURCE)

    def test_exact_cold_link_host_fixture(self):
        self.assertTrue(self.validate()['recovery_read_only'])

    def test_wrong_previous_stage_or_reused_invocation_is_rejected(self):
        self.recover['previous_stage'] = str(self.root / 'unrelated-normal')
        with self.assertRaisesRegex(ValueError, 'previous stage'):
            self.validate()
        self.recover['previous_stage'] = self.normal['run_directory']
        self.recover['run_directory'] = self.normal['run_directory']
        with self.assertRaisesRegex(ValueError, 'separate normal/recover'):
            self.validate()

    def test_changed_cold_image_and_new_recovery_ack_are_rejected(self):
        after = Path(self.recover['run_directory']) / 'test-disk.after.img'
        original = after.read_bytes()
        after.write_bytes(original[:-1] + b'x')
        with self.assertRaisesRegex(ValueError, 'exact unchanged saved image'):
            self.validate()
        after.write_bytes(original)
        self.recover['disk_oracle']['observed']['acks'] = 1
        with self.assertRaisesRegex(ValueError, 'sequence/ACK/state'):
            self.validate()

    def test_source_or_head_change_between_runs_is_rejected(self):
        original = self.recover.copy()
        for field, value in [('base_commit', '3' * 40), ('source_sha256', '4' * 64), ('observed_source_sha256', '5' * 64), ('source_unchanged', False)]:
            self.recover = dict(original, **{field: value})
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, 'frozen source/HEAD'):
                self.validate()

    def test_wrong_volume_identity_or_path_is_rejected(self):
        self.recover['disk_oracle']['identity'] = 'another-volume'
        with self.assertRaisesRegex(ValueError, 'identity changed'):
            self.validate()
        self.recover['disk_oracle']['identity'] = 'host-fixture'
        self.recover['volume_path'] = str(self.root / 'other.img')
        with self.assertRaisesRegex(ValueError, 'previous stage or volume'):
            self.validate()


class LaunchBoundaryTests(unittest.TestCase):
    def test_source_change_after_build_stops_before_first_vm(self):
        with tempfile.TemporaryDirectory(prefix='document-session-freeze-host-') as temporary:
            root = Path(temporary)
            build = root / 'build/private'
            target = root / 'native-target'
            environ = {'NOS_DOCUMENT_SESSION': '1', 'NOS_SDK_RUNTIME': '1', 'NOS_BUILD_DIR': str(build),
                       'NOS_TARGET_DIR': str(target), 'CARGO_TARGET_DIR': str(target)}
            with mock.patch.multiple(gate, ROOT=root, BUILD=build, TARGET=target), mock.patch.dict(os.environ, environ), \
                    mock.patch.object(gate.subprocess, 'check_output', return_value=HEAD), \
                    mock.patch.object(gate, 'source_hash', side_effect=[SOURCE, SOURCE, '3' * 64]), \
                    mock.patch.object(gate, 'build') as compile_step, \
                    mock.patch.object(gate, 'run_m6c') as native_step, mock.patch('builtins.print'):
                report = gate.run_gate(HEAD, SOURCE)
            self.assertFalse(report['passed'])
            self.assertEqual(report['status'], 'failed')
            self.assertEqual(report['real_vm_invocations'], 0)
            self.assertIn('Source or HEAD changed: after-normal-build', report['error'])
            compile_step.assert_called_once()
            native_step.assert_not_called()

    def test_explicit_double_opt_in(self):
        gate.opt_in({'NOS_DOCUMENT_SESSION': '1', 'NOS_SDK_RUNTIME': '1'})
        for values in ({}, {'NOS_SDK_RUNTIME': '1'}, {'NOS_DOCUMENT_SESSION': '1'},
                       {'NOS_DOCUMENT_SESSION': '0', 'NOS_SDK_RUNTIME': '1'},
                       {'NOS_DOCUMENT_SESSION': True, 'NOS_SDK_RUNTIME': '1'}):
            with self.subTest(values=values), self.assertRaisesRegex(ValueError, 'explicitly'):
                gate.opt_in(values)

    def test_existing_results_are_preserved_before_any_build_or_vm(self):
        with tempfile.TemporaryDirectory(prefix='document-session-root-host-') as temporary:
            root = Path(temporary)
            build = root / 'build/private'
            target = root / 'native-target'
            build.mkdir(parents=True)
            old = build / 'prior-result.json'
            old.write_bytes(b'preserve this earlier result')
            with mock.patch.multiple(gate, ROOT=root, BUILD=build, TARGET=target), mock.patch.dict(os.environ,
                    {'NOS_BUILD_DIR': str(build), 'NOS_TARGET_DIR': str(target), 'CARGO_TARGET_DIR': str(target)}):
                with self.assertRaisesRegex(ValueError, 'Preserve existing results'):
                    gate.fresh_roots()
            self.assertEqual(old.read_bytes(), b'preserve this earlier result')


if __name__ == '__main__':
    unittest.main()
