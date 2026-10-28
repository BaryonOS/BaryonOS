"""I11: one source-frozen qsim build and VM, isolated from other evidence."""
from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

ROOT = Path('/workspace/scratch/debaa3b6a81c/nieuw-os')
BUILD = ROOT / 'build/parallel-followup-qsim'
TARGET = Path('/tmp/nieuw-os-followup-qsim-target')
OLD = ROOT / 'build/parallel-final-qsim'
OLD_PROOF = ROOT / 'docs/verification/parallel-final-qsim'


def stamp():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--head', required=True)
    parser.add_argument('--source', required=True)
    args = parser.parse_args()
    if not re.fullmatch('[0-9a-f]{40}', args.head) or not re.fullmatch('[0-9a-f]{64}', args.source):
        parser.error('Require the full parent-approved frozen commit and source fingerprint')
    os.environ.update({
        'NOS_BUILD_DIR': str(BUILD), 'NOS_TARGET_DIR': str(TARGET),
        'CARGO_TARGET_DIR': str(TARGET), 'CARGO_BUILD_JOBS': '2',
        'CARGO_HOME': '/tmp/nieuw-os-par-tools/cargo-home',
        'CARGO_NET_OFFLINE': 'true', 'NOS_QEMU_MACHINE': 'pc-q35-7.2',
        'NOS_SDK_RUNTIME': '0', 'PYTHONDONTWRITEBYTECODE': '1',
    })
    sys.path.insert(0, str(ROOT / 'tools'))
    from common import environment, firmware, source_hash
    env = environment()

    def snapshot():
        return {
            'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
            'tree': subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=ROOT, text=True).strip(),
            'source_fingerprint_version': 3, 'source_sha256': source_hash(),
        }

    if BUILD.is_symlink() or TARGET.is_symlink():
        raise RuntimeError('I11 build and target roots must not be symlinks')
    BUILD.mkdir(parents=True, exist_ok=True)
    if (BUILD / 'i11-run-checks.json').exists() or (BUILD / 'manifest.json').exists():
        raise RuntimeError('Refusing to overwrite earlier I11 execution')
    report = {
        'task': 'I11', 'scope': 'one current native qsim smoke; no M3 regression rerun',
        'expected_head': args.head, 'expected_source_sha256': args.source,
        'started_utc': stamp(), 'passed': False, 'commands': [],
        'environment': {key: env.get(key) for key in (
            'NOS_BUILD_DIR', 'NOS_TARGET_DIR', 'CARGO_TARGET_DIR', 'CARGO_HOME',
            'CARGO_BUILD_JOBS', 'CARGO_NET_OFFLINE', 'NOS_QEMU_MACHINE', 'NOS_SDK_RUNTIME',
            'OS_OVMF_CODE', 'OS_OVMF_VARS', 'PYTHONDONTWRITEBYTECODE')},
    }
    try:
        report['before'] = snapshot()
        if report['before']['head'] != args.head or report['before']['source_sha256'] != args.source:
            raise RuntimeError('Current checkout does not match the explicitly approved freeze')
        old_manifest = json.loads((OLD_PROOF / 'build-manifest.json').read_text())
        old_scalar = json.loads((OLD_PROOF / 'quantum-build-report.json').read_text())
        old_expected = {
            'esp/kernel.elf': old_manifest['kernel']['sha256'],
            'esp/EFI/BOOT/BOOTX64.EFI': old_manifest['loader']['sha256'],
            'users/qsim/qsim-native-probe.elf': old_scalar['binary_sha256'],
        }
        old_actual = {name: digest(OLD / name) for name in old_expected}
        if old_actual != old_expected:
            raise RuntimeError('Previous immutable qsim artifact differs from its archived evidence')
        report['previous_artifacts'] = old_actual
        report['previous_source_sha256'] = old_manifest['source_sha256']
        report['qsim_inputs_vs_previous'] = {
            name: {'previous': expected, 'current': digest(ROOT / name)}
            for name, expected in old_scalar['source_sha256'].items()
        }
        report['qsim_inputs_unchanged'] = all(
            item['previous'] == item['current'] for item in report['qsim_inputs_vs_previous'].values())
        report['tools'] = {}
        for name in ('rustc', 'cargo', 'qemu-system-x86_64', 'objdump', 'nm'):
            path = Path(shutil.which(name, path=env['PATH']))
            version = subprocess.run([str(path), '--version'], cwd=ROOT, env=env, capture_output=True, text=True, timeout=20, check=True)
            report['tools'][name] = {'path': str(path), 'sha256': digest(path), 'version': version.stdout.strip()}
        report['firmware'] = {str(path): digest(path) for path in firmware()}
        save(BUILD / 'i11-run-checks.json', report)
        for label, command in (
            ('build', ['python3', '-B', 'tools/build.py', '--kernel-feature', 'native-qsim']),
            ('accept', ['python3', '-B', 'tools/run_qsim.py']),
        ):
            before = snapshot()
            if before['head'] != args.head or before['source_sha256'] != args.source:
                raise RuntimeError('Source or HEAD changed before ' + label)
            print(stamp() + ' I11 ' + label + ' start', flush=True)
            started = time.monotonic()
            out = BUILD / (label + '.stdout.txt')
            err = BUILD / (label + '.stderr.txt')
            with out.open('wb') as stdout, err.open('wb') as stderr:
                result = subprocess.run(command, cwd=ROOT, env=env, stdout=stdout, stderr=stderr, timeout=240)
            report['commands'].append({
                'label': label, 'command': command, 'cwd': str(ROOT),
                'returncode': result.returncode, 'duration_s': time.monotonic() - started,
                'stdout': str(out), 'stderr': str(err),
                'stdout_sha256': digest(out), 'stderr_sha256': digest(err),
            })
            save(BUILD / 'i11-run-checks.json', report)
            print(stamp() + ' I11 ' + label + ' exit ' + str(result.returncode), flush=True)
            if result.returncode:
                raise RuntimeError(label + ' failed; raw logs retained in own build root')
        acceptance = json.loads((BUILD / 'native-qsim-result.json').read_text())
        report['after'] = snapshot()
        report['source_and_head_unchanged'] = report['after'] == report['before']
        report['current_artifacts'] = {name: digest(BUILD / name) for name in old_expected}
        report['artifact_comparison'] = {
            name: {'previous_sha256': old_actual[name], 'current_sha256': report['current_artifacts'][name],
                   'identical_bytes': old_actual[name] == report['current_artifacts'][name]}
            for name in old_expected
        }
        report['old_artifacts_still_unchanged'] = {name: digest(OLD / name) for name in old_expected} == old_actual
        report['acceptance_path'] = str(BUILD / 'native-qsim-result.json')
        report['acceptance_sha256'] = digest(BUILD / 'native-qsim-result.json')
        report['raw_qemu_exit'] = acceptance.get('guest', {}).get('raw_qemu_exit')
        report['acceptance_checks'] = acceptance.get('checks')
        report['observed'] = acceptance.get('observed')
        report['vm_count'] = len(list((BUILD / 'run').glob('*/result.json')))
        report['passed'] = bool(acceptance.get('passed') is True and report['source_and_head_unchanged']
                              and report['old_artifacts_still_unchanged'] and report['vm_count'] == 1)
        if not report['passed']:
            raise RuntimeError('I11 acceptance or frozen-source checks did not pass')
    except Exception as error:
        report['error'] = type(error).__name__ + ': ' + str(error)
        raise
    finally:
        report['finished_utc'] = stamp()
        save(BUILD / 'i11-run-checks.json', report)
    print(json.dumps({'passed': report['passed'], 'head': args.head,
                      'source': args.source, 'raw_qemu_exit': report['raw_qemu_exit'],
                      'artifact_comparison': report['artifact_comparison'],
                      'observed': report['observed']}, indent=2), flush=True)


if __name__ == '__main__':
    main()
