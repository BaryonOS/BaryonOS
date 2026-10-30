#!/usr/bin/env python3
"""Run exactly two opt-in native DocumentSession cases: edit/save, cold reopen.

This directed gate reuses the existing build, M6c runner and complete per-run
capture validator. It is not the full M6c acceptance suite or a GUI test.
"""
from __future__ import annotations

import argparse
from contextlib import redirect_stderr, redirect_stdout
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time

from common import ROOT, BUILD, TARGET, sha256, source_hash
from build import build
from build_users import inspect_user
from capture_m6c import check_run
from run_m6c import run_m6c

MAX_JSON = 2 * 1024 * 1024
MAX_ARTIFACT = 16 * 1024 * 1024
MAX_LOG = 2 * 1024 * 1024
PACKAGES = ('sdk/runtime', 'sdk/catalog-client', 'sdk/document-session', 'services/catalog', 'services/volume')
LIBRARIES = {
    'nieuw_os_sdk_runtime': ('sdk/runtime/src/lib.rs', 'build-sdk-runtime'),
    'catalog_core': ('services/catalog/src/lib.rs', 'build-document-library-catalog_core'),
    'volume_core': ('services/volume/src/lib.rs', 'build-document-library-volume_core'),
    'nieuw_os_catalog_client': ('sdk/catalog-client/src/lib.rs', 'build-document-library-nieuw_os_catalog_client'),
    'nieuw_os_document_session': ('sdk/document-session/src/lib.rs', 'build-document-library-nieuw_os_document_session'),
}
LIBRARY_PATHS = frozenset(f'users/lib{crate}.rlib' for crate in LIBRARIES)
DEPENDENCIES = {
    'nieuw_os_sdk_runtime': (),
    'catalog_core': (),
    'volume_core': ('catalog_core',),
    'nieuw_os_catalog_client': ('nieuw_os_sdk_runtime', 'catalog_core'),
    'nieuw_os_document_session': ('nieuw_os_sdk_runtime', 'catalog_core', 'nieuw_os_catalog_client', 'volume_core'),
    'os_user_m6c_probe': tuple(LIBRARIES),
}
ABI_OPTIONS = {'panic': 'abort', 'code-model': 'small', 'relocation-model': 'static', 'no-redzone': 'yes'}
FEATURES = {'normal': 'milestone-m6c', 'recover': 'test-m6c-recover'}
REQUESTS_PER_CLIENT = {'normal': 16, 'recover': 3}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def opt_in(environ):
    require(environ.get('NOS_DOCUMENT_SESSION') == '1' and environ.get('NOS_SDK_RUNTIME') == '1',
            'Set NOS_DOCUMENT_SESSION=1 and NOS_SDK_RUNTIME=1 explicitly')


def regular(path, root, limit=MAX_ARTIFACT):
    path = Path(path)
    require(not path.is_symlink() and path.is_file(), f'Missing/nonregular artifact: {path}')
    require(path.resolve().is_relative_to(Path(root).resolve()), f'Artifact escaped its root: {path}')
    require(path.stat().st_size <= limit, f'Oversized artifact: {path}')
    return path


def read_json(path):
    path = Path(path)
    regular(path, path.parent, MAX_JSON)
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, 'Duplicate evidence key')
            result[key] = value
        return result
    value = json.loads(path.read_bytes(), object_pairs_hook=unique)
    require(isinstance(value, dict), 'Evidence JSON must contain an object')
    return value


def save_report(path, report):
    encoded = json.dumps(report, indent=2, ensure_ascii=False).encode('utf-8') + b'\n'
    require(len(encoded) <= MAX_JSON, 'Gate report exceeds its bounded size')
    path.write_bytes(encoded)


def selected_sources(root):
    names = {'userspace/m6c_probe.rs', 'userspace/m6_runtime.rs', 'userspace/sdk_probe_runtime.rs',
             'userspace/document_session_probe.rs', 'sdk/Cargo.toml', 'sdk/Cargo.lock'}
    for package in PACKAGES:
        names.add(package + '/Cargo.toml')
        require((root / package / 'src/lib.rs').is_file(), f'Missing selected library source: {package}')
        names.update(path.relative_to(root).as_posix() for path in (root / package / 'src').rglob('*.rs'))
    for name in names:
        regular(root / name, root)
    return {name: sha256(root / name) for name in sorted(names)}


def option_values(argv, option):
    values = []
    for index, argument in enumerate(argv):
        if argument == option:
            require(index + 1 < len(argv), 'Truncated compiler option')
            values.append(argv[index + 1])
        elif argument.startswith(option + '='):
            values.append(argument[len(option) + 1:])
    return values


def abi_settings(argv):
    """Count every spelling of rustc -C/--codegen; later values cannot hide."""
    settings = {}
    index = 0
    while index < len(argv):
        argument = argv[index]
        index += 1
        if argument in ('-C', '--codegen'):
            require(index < len(argv), 'Truncated compiler codegen option')
            value = argv[index]
            index += 1
        elif argument.startswith('-C='):
            value = argument[3:]
        elif argument.startswith('-C') and len(argument) > 2:
            value = argument[2:]
        elif argument.startswith('--codegen='):
            value = argument[len('--codegen='):]
        else:
            continue
        key, separator, selected = value.partition('=')
        key = key.replace('_', '-')
        if key in ABI_OPTIONS:
            settings.setdefault(key, []).append(selected if separator else None)
    return settings


def compiler_binding(snapshot, original_build, root):
    """Bind successful recorded rustc invocations to the selected crate inputs."""
    commands = {}
    rows = [(crate, source, log, 'rlib', f'users/lib{crate}.rlib')
            for crate, (source, log) in LIBRARIES.items()]
    rows.append(('os_user_m6c_probe', 'userspace/m6c_probe.rs', 'build-users-m6c_probe', 'bin', 'users/m6c_probe.elf'))
    for crate, source, log, kind, output in rows:
        metadata = read_json(snapshot / 'logs' / (log + '.json'))
        command = metadata.get('command')
        require(isinstance(command, list) and all(isinstance(arg, str) for arg in command), 'Invalid compiler command')
        require(command and Path(command[0]).name in ('rustc', 'rustc.exe') and command[1:2] == ['+1.90.0'], 'Pinned rustc launcher missing')
        require(type(metadata.get('returncode')) is int and metadata['returncode'] == 0
                and metadata.get('timed_out', False) is False and metadata.get('cwd') == str(root), 'Failed or foreign compile evidence')
        for option, expected in (('--crate-name', crate), ('--crate-type', kind), ('--target', 'x86_64-unknown-none'), ('-o', str(original_build / output))):
            require(option_values(command, option) == [expected], 'Compiler output/profile mismatch: ' + crate)
        require(str(root / source) in command, 'Compiler entry source mismatch: ' + crate)
        settings = abi_settings(command)
        for key, expected in ABI_OPTIONS.items():
            require(settings.get(key) == [expected], 'Native ABI requires exactly one ' + key + '=' + expected)
        expected = {f'{name}={original_build / ("users/lib" + name + ".rlib")}' for name in DEPENDENCIES[crate]}
        actual = option_values(command, '--extern')
        require(len(actual) == len(expected) and set(actual) == expected,
                'Exact selected compiler dependency set mismatch: ' + crate)
        if kind == 'bin':
            require(set(option_values(command, '--cfg')) == {'nos_sdk_runtime', 'nos_document_session'}, 'DocumentSession compiler selection missing')
        for suffix in ('.stdout.txt', '.stderr.txt'):
            regular(snapshot / 'logs' / (log + suffix), snapshot, MAX_LOG)
        commands[log] = sha256(snapshot / 'logs' / (log + '.json'))
    return commands


def validate_snapshot(snapshot, *, original_build, root, frozen, case, kernel, loader):
    """Rehash actual binaries, sources and compile inputs; no guest is simulated."""
    require(case in FEATURES, 'Only normal and recovery are part of this gate')
    manifest = read_json(snapshot / 'manifest.json')
    user = read_json(snapshot / 'users/manifest-m6c.json')
    for evidence in (manifest, user):
        require(evidence.get('document_session') is True and evidence.get('sdk_runtime') is True,
                'Both build and user manifests must explicitly select DocumentSession and SDK')
    require(manifest.get('kernel_feature') == FEATURES[case] and manifest.get('loader_feature') is None,
            'Wrong exclusive native DocumentSession profile')
    require(manifest.get('source_fingerprint_version') == 3 and manifest.get('source_sha256') == frozen,
            'Build belongs to a different source')
    require(user.get('source_sha256') == sha256(root / 'userspace/m6c_probe.rs')
            and user.get('linker_sha256') == sha256(root / 'userspace/m6c_linker.ld'), 'Probe entry source/linker mismatch')
    sources = selected_sources(root)
    require(user.get('module_sha256') == sources, 'Missing, extra or changed selected library/probe source')
    libraries = user.get('library_sha256')
    require(isinstance(libraries, dict) and set(libraries) == LIBRARY_PATHS, 'Missing or unexpected native library artifact')
    for name, expected in libraries.items():
        path = regular(snapshot / name, snapshot)
        require(isinstance(expected, str) and re.fullmatch('[0-9a-f]{64}', expected)
                and sha256(path) == expected, 'Native library bytes differ: ' + name)
    elf = regular(snapshot / 'users/m6c_probe.elf', snapshot)
    require(sha256(elf) == user.get('binary_sha256'), 'User ELF bytes differ from the selected artifact')
    inspected = inspect_user(elf)
    require(all(user.get(key) == value for key, value in inspected.items()), 'User ELF layout differs from manifest')
    regular(kernel, kernel.parent)
    regular(loader, loader.parent)
    require(sha256(kernel) == manifest['kernel']['sha256'] and sha256(loader) == manifest['loader']['sha256'], 'Actual kernel/loader mismatch')
    require(elf.read_bytes() in kernel.read_bytes(), 'Actual kernel does not embed the selected DocumentSession ELF')
    commands = compiler_binding(snapshot, original_build, root)
    return {'case': case, 'document_session': True, 'sdk_runtime': True,
            'user_elf_sha256': user['binary_sha256'], 'actual_user_elf_embedded': True,
            'kernel_sha256': sha256(kernel), 'loader_sha256': sha256(loader),
            'selected_source_sha256': sources, 'library_sha256': libraries, 'compiler_command_sha256': commands}


def snapshot_build(case, root, output, frozen):
    destination = output / 'artifacts' / case
    destination.mkdir(parents=True)
    names = {'manifest.json', 'users/manifest-m6c.json', 'users/m6c_probe.elf'} | set(LIBRARY_PATHS)
    names.update('logs/' + log + suffix for log in [row[1] for row in LIBRARIES.values()] + ['build-users-m6c_probe']
                 for suffix in ('.json', '.stdout.txt', '.stderr.txt'))
    for name in sorted(names):
        original = regular(BUILD / name, BUILD, MAX_LOG if name.startswith('logs/') else MAX_ARTIFACT)
        target = destination / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(original, target)
        require(sha256(original) == sha256(target), 'Build input changed while snapshotting')
    binding = validate_snapshot(destination, original_build=BUILD, root=root, frozen=frozen, case=case,
                                kernel=BUILD / 'esp/kernel.elf', loader=BUILD / 'esp/EFI/BOOT/BOOTX64.EFI')
    hashes = {name: sha256(destination / name) for name in sorted(names)}
    return destination, binding, hashes


def validate_source(report, head, frozen):
    require(report.get('base_commit') == head and report.get('source_fingerprint_version') == 3
            and report.get('source_sha256') == report.get('observed_source_sha256') == frozen
            and report.get('source_unchanged') is True, 'Run does not bind the frozen source/HEAD')


def validate_native(report, snapshot, *, head, frozen, case):
    validate_source(report, head, frozen)
    require(report.get('case') == case and type(report.get('raw_qemu_exit')) is int
            and report['raw_qemu_exit'] == 33 and report.get('timed_out') is False
            and report.get('passed') is True, 'Native DocumentSession run failed')
    directory = Path(report['run_directory']).resolve(strict=True)
    require(directory.is_relative_to((BUILD / 'run').resolve()), 'Guest evidence left its private build root')
    require(read_json(directory / 'result.json') == report, 'Returned report differs from preserved VM result')
    check_run(directory, report, frozen)  # Existing complete validator, unchanged.
    require((directory / 'build-manifest.json').read_bytes() == (snapshot / 'manifest.json').read_bytes(),
            'Guest used another build manifest')
    binding = validate_snapshot(snapshot, original_build=BUILD, root=ROOT, frozen=frozen, case=case,
                                kernel=directory / 'esp/kernel.elf', loader=directory / 'esp/EFI/BOOT/BOOTX64.EFI')
    require(sha256(Path(report['command'][0])) == report['qemu_sha256'], 'QEMU launcher changed')
    observed = report['disk_oracle']['observed']
    require(observed['state']['sequence'] == 10 and observed['acks'] == (10 if case == 'normal' else 0)
            and observed['domain_revisions'] == [5, 5], 'Wrong committed DocumentSession outcome')
    binding['requests_per_client_required_by_kernel_observer'] = REQUESTS_PER_CLIENT[case]
    binding['request_count_evidence'] = 'Existing exact per-client observer assertion before M6C:PASS; not an additional serial counter'
    binding['observer_source_sha256'] = sha256(ROOT / 'kernel/src/m2/m3/m6c.rs')
    return binding


def cold_link(normal, recovered, *, head, frozen):
    for report in (normal, recovered):
        validate_source(report, head, frozen)
    a, b = Path(normal['run_directory']), Path(recovered['run_directory'])
    require(normal.get('case') == 'normal' and recovered.get('case') == 'recover' and a.resolve() != b.resolve(), 'Expected two separate normal/recover invocations')
    require(recovered.get('previous_stage') == str(a) and normal['volume_path'] == recovered['volume_path'], 'Wrong previous stage or volume in cold link')
    require(normal['disk_oracle']['identity'] == recovered['disk_oracle']['identity'], 'Cold volume identity changed')
    after = regular(a / 'test-disk.after.img', a, 65536).read_bytes()
    before = regular(b / 'test-disk.before.img', b, 65536).read_bytes()
    cold = regular(b / 'test-disk.after.img', b, 65536).read_bytes()
    actual = Path(normal['volume_path'])
    require(len(after) == 65536 and after == before == cold == regular(actual, actual.parent, 65536).read_bytes(),
            'Cold recovery did not read the exact unchanged saved image')
    require(normal['disk_oracle']['observed']['acks'] == 10 and recovered['disk_oracle']['observed']['acks'] == 0
            and normal['disk_oracle']['state'] == recovered['disk_oracle']['state']
            and recovered['disk_oracle']['state']['sequence'] == 10, 'Cold sequence/ACK/state mismatch')
    return {'same_exact_saved_image': True, 'recovery_read_only': True, 'sequence': 10,
            'normal_acks': 10, 'recovery_acks': 0, 'saved_image_sha256': sha256(a / 'test-disk.after.img')}


def fresh_roots():
    require(os.environ.get('NOS_BUILD_DIR') and os.environ.get('NOS_TARGET_DIR')
            and os.environ.get('CARGO_TARGET_DIR'), 'Set isolated build and both target paths explicitly')
    require(BUILD.is_relative_to(ROOT / 'build') and BUILD != ROOT / 'build', 'Choose a private build subdirectory inside the checkout')
    require(Path(os.environ['CARGO_TARGET_DIR']).resolve() == TARGET, 'Cargo and native target directories must match')
    require(not BUILD.exists() or BUILD.is_dir() and not any(BUILD.iterdir()), 'Preserve existing results: use an empty private build directory')
    BUILD.mkdir(parents=True, exist_ok=True)


def run_gate(expected_head=None, expected_source=None):
    opt_in(os.environ)
    require((expected_head is None) == (expected_source is None), 'Provide both expected HEAD and source, or neither')
    if expected_head is not None:
        require(re.fullmatch('[0-9a-f]{40}', expected_head) and re.fullmatch('[0-9a-f]{64}', expected_source), 'Require full frozen HEAD/source hashes')
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True, timeout=10).strip()
    frozen = source_hash()
    require(expected_head is None or (head, frozen) == (expected_head, expected_source), 'Initial source differs from requested freeze')
    fresh_roots()
    output = BUILD / 'document-session-gate'
    output.mkdir()
    result_path = output / 'result.json'
    report = {'schema_version': 1, 'status': 'running', 'passed': False,
              'scope': 'Two real native DocumentSession normal/cold-recovery cases; no GUI or full M6c suite',
              'source_commit': head, 'source_fingerprint_version': 3, 'source_sha256': frozen,
              'started_utc': datetime.now(timezone.utc).isoformat(), 'gate_sha256': sha256(Path(__file__)),
              'environment': {key: os.environ.get(key) for key in ('NOS_DOCUMENT_SESSION', 'NOS_SDK_RUNTIME', 'NOS_BUILD_DIR', 'NOS_TARGET_DIR', 'CARGO_TARGET_DIR', 'CARGO_BUILD_JOBS', 'NOS_QEMU_MACHINE')},
              'freeze_checks': [], 'phases': [], 'runs': [], 'bindings': [], 'snapshots': [], 'real_vm_invocations': 0}
    save_report(result_path, report)

    def freeze(stage):
        observed_head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True, timeout=10).strip()
        observed = source_hash()
        row = {'stage': stage, 'head': observed_head, 'source_sha256': observed,
               'passed': observed_head == head and observed == frozen}
        report['freeze_checks'].append(row)
        save_report(result_path, report)
        require(row['passed'], 'Source or HEAD changed: ' + stage)

    def phase(name, function):
        started = time.monotonic()
        out, err = output / (name + '.stdout.txt'), output / (name + '.stderr.txt')
        result = None
        error = None
        try:
            with out.open('w', encoding='utf-8', newline='') as stdout, err.open('w', encoding='utf-8', newline='') as stderr:
                with redirect_stdout(stdout), redirect_stderr(stderr):
                    result = function()
            return result
        except Exception as exception:
            error = f'{type(exception).__name__}: {exception}'
            raise
        finally:
            row = {'name': name, 'duration_s': time.monotonic() - started, 'completed': error is None, 'error': error}
            row['logs_sha256'] = {path.name: sha256(regular(path, output, MAX_LOG)) for path in (out, err)}
            report['phases'].append(row)
            save_report(result_path, report)

    try:
        volume = None
        for case in ('normal', 'recover'):
            freeze('before-' + case + '-build')
            phase('build-' + case, lambda: build(kernel_feature=FEATURES[case], label='document-session-' + case))
            freeze('after-' + case + '-build')
            snapshot, binding, hashes = snapshot_build(case, ROOT, output, frozen)
            report['snapshots'].append({'case': case, 'directory': str(snapshot), 'files_sha256': hashes})
            freeze('before-' + case + '-vm')
            result, volume = phase('vm-' + case, lambda: run_m6c(case, volume=volume, label='document-session-' + case))
            report['runs'].append(result)
            report['real_vm_invocations'] = len(report['runs'])
            freeze('after-' + case + '-vm')
            report['bindings'].append(validate_native(result, snapshot, head=head, frozen=frozen, case=case))
        require(len(report['runs']) == 2, 'Exactly two real VM results required')
        require(report['bindings'][0]['user_elf_sha256'] == report['bindings'][1]['user_elf_sha256']
                and report['bindings'][0]['library_sha256'] == report['bindings'][1]['library_sha256'],
                'DocumentSession app or native libraries changed between the two cases')
        report['same_user_and_library_artifacts'] = True
        report['cold_link'] = cold_link(*report['runs'], head=head, frozen=frozen)
        # Revalidate the normal evidence too after the recovery build and VM.
        for result, snapshot in zip(report['runs'], report['snapshots']):
            for name, expected in snapshot['files_sha256'].items():
                require(sha256(regular(Path(snapshot['directory']) / name, Path(snapshot['directory']))) == expected, 'Saved build snapshot changed')
            validate_native(result, Path(snapshot['directory']), head=head, frozen=frozen, case=result['case'])
        freeze('after-complete-recheck')
        report.update(status='completed', passed=True)
    except Exception as error:
        report.update(status='failed', error=f'{type(error).__name__}: {error}')
    finally:
        report['finished_utc'] = datetime.now(timezone.utc).isoformat()
        save_report(result_path, report)
    print(f"DocumentSession accepted={report['passed']}; actual VM results={len(report['runs'])}; {result_path}")
    if 'error' in report:
        print(report['error'])
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--expected-head')
    parser.add_argument('--expected-source')
    args = parser.parse_args()
    raise SystemExit(0 if run_gate(args.expected_head, args.expected_source)['passed'] else 1)
