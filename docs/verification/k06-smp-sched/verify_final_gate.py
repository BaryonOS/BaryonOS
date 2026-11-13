#!/usr/bin/env python3
"""Read back the K06 final-gate artifact; never execute a build or a guest.

Use the tested checkout for --repo. Raw REST run/jobs responses are required;
wrapper execution and archived reports alone do not establish a green CI job.
A successful result verifies retained evidence, not missing ELF/firmware bytes.
"""
from __future__ import annotations
import argparse
from datetime import datetime
import hashlib
import json
import math
from pathlib import Path
import re
import shlex
import sys

sys.dont_write_bytecode = True

STAGES = ('host-tests', 'clippy', 'fmt', 'm2', 'k05', 'k05-smp', 'k07', 'rtc-build', 'rtc')
M2 = (
    ('own-cr3-frame-ownership-wx', 'm2', 'milestone-m2'),
    ('test-m2-ro', 'test-m2-ro', 'test-m2-ro'),
    ('test-m2-nx', 'test-m2-nx', 'test-m2-nx'),
    ('test-m2-absent', 'test-m2-absent', 'test-m2-absent'),
    ('kernel-fault-stops-controlled', 'm2-kernel-fault', 'test-m2-kernel-fault'),
    ('unusable-main-stack-double-fault-ist', 'm2-double-fault', 'test-m2-double-fault'),
    ('m1-regression', 'm1-after-m2', None),
)
LIMITS = [
    'The artifact omits ELF/EFI/firmware/QEMU bytes. Recorded hashes and build-manifest '
    'coherence are checked; binaries cannot be independently rehashed or executed.',
    'K05 raw reports have no base_commit or artifact_binding_passed field. Their '
    'source-v3 and ESP hashes are checked against source-bound build manifests; '
    'commit provenance is supplied by the required GitHub run/job records.',
    'Top-level cargo check/test have the combined gate.log and green GitHub step, '
    'not separate structured execution receipts. Host checks are not rerun by this readback.',
    'The existing serial oracles are replayed from the matching local source, not '
    'a separately implemented hardware model or a human security audit.',
    'A successful final-gate readback does not independently verify the separate '
    '74x50 matrix artifact; the aggregate job must also be successful.',
]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f'duplicate JSON key: {key}')
        result[key] = value
    return result


def read_json(path):
    return json.loads(path.read_text(encoding='utf-8'), object_pairs_hook=unique,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sha(value):
    return isinstance(value, str) and re.fullmatch('[0-9a-f]{64}', value) is not None


def stamp(value):
    parsed = datetime.fromisoformat(value.replace('Z', '+00:00'))
    require(parsed.tzinfo is not None, 'timestamp has no timezone')
    return parsed


class Readback:
    def __init__(self, root, source, commit):
        self.root = root.resolve(strict=True)
        self.source, self.commit = source, commit
        self.used = set()
        self.ids = set()
        self.guest_hashes = {}

    def file(self, relative):
        path = self.root / relative
        require(path.is_file() and not path.is_symlink(), f'missing regular artifact: {relative}')
        require(path.resolve().is_relative_to(self.root), f'escaped artifact: {relative}')
        return path

    def load(self, relative):
        return read_json(self.file(relative))

    def text(self, relative):
        return self.file(relative).read_bytes().decode('utf-8', 'replace')

    def bound(self, report):
        require(report.get('source_fingerprint_version') == 3
                and report.get('source_sha256') == self.source, 'source-v3 binding mismatch')

    def mapped(self, original):
        require(isinstance(original, str), 'missing original evidence path')
        anchor = '/k06-smp-final-gate/'
        require(original.count(anchor) == 1, f'unknown evidence prefix: {original}')
        suffix = original.split(anchor, 1)[1]
        require(not Path(suffix).is_absolute() and '..' not in Path(suffix).parts,
                'unsafe evidence path')
        return self.file(suffix)

    def manifest(self, path, feature):
        manifest = read_json(self.file(path))
        self.bound(manifest)
        require(manifest.get('kernel_feature') == feature
                and manifest.get('loader_feature') is None, f'wrong build profile: {path}')
        require(sha(manifest['kernel']['sha256']) and sha(manifest['loader']['sha256']),
                'invalid kernel/loader digest')
        return manifest

    def guest(self, original, *, expected_exit, timeout=60, k05=False, embedded=None, smp=False):
        path = self.mapped(original + '/result.json')
        require(path not in self.used, f'duplicate guest result: {path}')
        self.used.add(path)
        report = read_json(path)
        if embedded is not None:
            require(report == embedded, f'embedded guest differs from raw result: {path}')
        self.bound(report)
        require(report.get('run_directory') == original and original not in self.ids,
                'missing or duplicate original run identity')
        self.ids.add(original)
        require(report.get('passed') is True and 'error' not in report
                and report.get('timed_out') is False
                and type(report.get('raw_qemu_exit')) is int
                and report['raw_qemu_exit'] == expected_exit
                and report.get('timeout_s') == timeout, f'failed exit/watchdog: {path}')
        duration = report.get('duration_s')
        require(type(duration) in (int, float) and math.isfinite(duration) and duration >= 0,
                'invalid guest duration')
        require(self.started <= stamp(report['started_utc']) <= self.completed,
                'guest start lies outside the executed final-gate step')
        require(self.text(path.parent.relative_to(self.root) / 'serial.txt') == report['stdout']
                and self.text(path.parent.relative_to(self.root) / 'stderr.txt') == report['stderr'],
                f'raw transcript differs from JSON: {path}')
        command = report.get('command')
        require(isinstance(command, list) and all(isinstance(arg, str) for arg in command),
                'invalid QEMU argv')
        for option, expected in (('-machine', 'pc-q35-10.0'),
                                 ('-accel', 'tcg,thread=multi' if smp else 'tcg,thread=single'),
                                 ('-cpu', 'qemu64'), ('-smp', '4' if smp else '1'), ('-m', '256M'),
                                 ('-display', 'none'), ('-monitor', 'none'), ('-nic', 'none')):
            require(command.count(option) == 1 and command[command.index(option) + 1] == expected,
                    f'QEMU profile mismatch: {option}')
        require(str(report.get('qemu_version', '')).startswith('QEMU emulator version 10.2.1 '),
                'unexpected QEMU version')
        require('-nodefaults' in command and '-no-reboot' in command, 'unbounded VM defaults')
        if k05:
            artifacts = report.get('esp_sha256', {})
            require(set(artifacts) == {'esp/kernel.elf', 'esp/EFI/BOOT/BOOTX64.EFI'}
                    and all(sha(value) for value in artifacts.values()), 'K05 ESP hash set mismatch')
            require(report.get('kernel_sha256') == artifacts['esp/kernel.elf'], 'K05 kernel hash mismatch')
        else:
            require(report.get('base_commit') == self.commit
                    and report.get('artifact_binding_passed') is True
                    and report.get('expected_exit') == expected_exit, 'guest head/artifact binding mismatch')
            artifacts = report.get('artifact_sha256', {})
            require(set(artifacts) == {'esp/kernel.elf', 'esp/EFI/BOOT/BOOTX64.EFI',
                                      'OVMF_CODE.fd', 'OVMF_VARS.template.fd'}
                    and all(sha(value) for value in artifacts.values()), 'guest artifact hash set mismatch')
            require(artifacts['OVMF_CODE.fd'] == '874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7'
                    and artifacts['OVMF_VARS.template.fd'] == '5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e',
                    'pinned firmware hash mismatch')
            expected = report.get('expected_artifact_sha256')
            require(isinstance(expected, dict) and all(artifacts.get(key) == value
                    for key, value in expected.items()), 'artifact expectation mismatch')
        self.guest_hashes[str(path.relative_to(self.root))] = digest(path)
        return report, artifacts

    def bind_artifacts(self, artifacts, manifest):
        require(artifacts['esp/kernel.elf'] == manifest['kernel']['sha256']
                and artifacts['esp/EFI/BOOT/BOOTX64.EFI'] == manifest['loader']['sha256'],
                'recorded guest bytes differ from selected build manifest')

    def api(self, run, job_payload):
        require(run.get('head_sha') == self.commit and run.get('status') == 'completed'
                and run.get('conclusion') == 'success' and run.get('event') == 'workflow_dispatch',
                'workflow run is not successful on the expected head')
        if isinstance(job_payload, dict):
            jobs = job_payload.get('jobs', [])
            require(len(jobs) == job_payload.get('total_count'), 'incomplete jobs API pagination')
        else:
            require(isinstance(job_payload, list), 'jobs API must be an object or list of pages')
            jobs = [job for page in job_payload for job in page.get('jobs', [])]
            require(len(jobs) == max(page.get('total_count', 0) for page in job_payload),
                    'incomplete jobs API pages')
        require(len({job['id'] for job in jobs}) == len(jobs), 'duplicate API job IDs')
        selected = {}
        for name in ('SMP section 9 aggregate 74x50', 'SMP final repository gate'):
            matching = [job for job in jobs if job.get('name') == name]
            require(len(matching) == 1, f'missing or duplicate job: {name}')
            job = matching[0]
            require(job.get('run_id') == run.get('id') and job.get('status') == 'completed'
                    and job.get('conclusion') == 'success', f'job did not execute successfully: {name}')
            if 'head_sha' in job:
                require(job['head_sha'] == self.commit, 'job head differs from run head')
            selected[name] = job
        job = selected['SMP final repository gate']
        required = ('Checkout accepted source', 'Pinned QEMU and OVMF', 'Pinned Python',
                    'Pinned Rust', 'Run tools/gate.sh only after the complete matrix',
                    'Preserve final gate evidence')
        steps = job.get('steps', [])
        for name in required:
            matching = [step for step in steps if step.get('name') == name]
            require(len(matching) == 1 and matching[0].get('status') == 'completed'
                    and matching[0].get('conclusion') == 'success', f'final step not green: {name}')
        step = next(step for step in steps if step['name'] == required[4])
        self.started, self.completed = stamp(step['started_at']), stamp(step['completed_at'])
        aggregate = selected['SMP section 9 aggregate 74x50']
        require(stamp(aggregate['completed_at']) <= self.started <= self.completed,
                'final gate ran before aggregate completed')
        return {'run_id': run['id'], 'final_job_id': job['id'], 'aggregate_job_id': aggregate['id']}

    def gate_log(self):
        text = self.text('logs/gate.log')
        lines = text.splitlines()
        start = '🔍 [Gate] Start verificatie voor nieuw-os...'
        end = '✅ Alle uitgevoerde gates geslaagd!'
        require(lines.count(start) == lines.count(end) == 1 and lines[0] == start
                and lines[-1] == end, 'missing/duplicated/truncated gate start or success')
        require('❌' not in text and 'Source changed before regression stage' not in text
                and 'Source changed during regression stage' not in text, 'gate failure diagnostic')
        require(lines.count('Regression source_sha256=' + self.source) == 1
                and lines.count('Regression gates passed on source_sha256=' + self.source) == 1,
                'wrapper source/success marker mismatch')
        require(self.text('regressions/source.txt') == 'Regression source_sha256=' + self.source + '\n',
                'archived wrapper source binding mismatch')
        starts = list(re.finditer(r'^Regression stage=([^ ]+) command=(.*)$', text, re.M))
        ends = list(re.finditer(r'^Regression stage=([^ ]+) exit=([0-9]+) elapsed_s=([0-9]+)$', text, re.M))
        require(tuple(item[1] for item in starts) == STAGES
                and tuple(item[1] for item in ends) == STAGES, 'wrong or incomplete wrapper stages')
        native = {'m2': ['python3', '-B', 'tools/test_milestones.py', '--through', 'm2'],
                  'k05': ['python3', '-B', '-u', 'tools/test_supervisor.py'],
                  'k05-smp': ['python3', '-B', '-u', 'tools/test_supervisor.py', '--smp'],
                  'k07': ['python3', '-B', 'tools/test_resource_pressure.py'],
                  'rtc-build': ['python3', '-B', 'tools/build.py', '--kernel-feature', 'native-rtc'],
                  'rtc': ['python3', '-B', 'tools/run_rtc.py', '--case', 'all']}
        host = {'host-tests': ['cargo', '+1.90.0', 'test', '--locked', '-p', 'boot-protocol', '-p', 'smp-lock'],
                'clippy': ['cargo', '+1.90.0', 'clippy', '--locked', '-p', 'boot-protocol', '-p', 'smp-lock', '--all-targets', '--', '-D', 'warnings'],
                'fmt': ['cargo', '+1.90.0', 'fmt', '--all', '--', '--check']}
        previous = -1
        for first, last in zip(starts, ends):
            stage = first[1]
            require(previous < first.start() < last.start() and last[2] == '0', 'failed or overlapping wrapper stages')
            previous = last.end()
            argv = shlex.split(first[2])
            if stage in host:
                require(argv == host[stage], f'changed host command: {stage}')
            else:
                require(argv[0] == 'env' and argv[4:] == native[stage], f'changed native command: {stage}')
                env = dict(argument.split('=', 1) for argument in argv[1:4])
                leaf = 'rtc' if stage == 'rtc-build' else stage
                require(set(env) == {'NOS_BUILD_DIR', 'NOS_TARGET_DIR', 'CARGO_TARGET_DIR'}
                        and env['NOS_TARGET_DIR'] == env['CARGO_TARGET_DIR']
                        and env['NOS_BUILD_DIR'].endswith('/k06-smp-final-gate/regressions/' + leaf)
                        and env['NOS_TARGET_DIR'].endswith('/regressions/' + leaf), 'mixed native build directories')
            require(self.text('regressions/logs/' + stage + '.log') == text[first.end() + 1:last.start()],
                    f'wrapper stage log differs from gate stream: {stage}')
        return {first[1]: {'exit': 0, 'elapsed_s': int(last[3])} for first, last in zip(starts, ends)}

    def evidence(self, gate_contract, run_module, supervisor, rtc, qsim, supervisor_tests):
        # M2: one host group plus seven distinct native runs (8 groups total).
        suite = self.load('regressions/m2/m2-test-results.json')
        require(suite.get('passed') is True and suite.get('status') == 'completed'
                and suite.get('phase') == 'M2' and 'error' not in suite, 'M2 suite incomplete')
        groups = suite.get('results', [])
        require([group.get('name') for group in groups] == ['frame-map-protocol-host-tests'] + [x[0] for x in M2]
                and all(group.get('passed') is True for group in groups), 'M2 exact groups mismatch')
        metadata = self.load('regressions/m2/logs/m2-host-tests.json')
        require(metadata.get('returncode') == 0 and not metadata.get('timed_out')
                and metadata.get('command', [])[1:] == ['+1.90.0', 'test', '--locked', '-p', 'kernel-core', '-p', 'boot-protocol', '-p', 'elf-loader-core'], 'M2 host test metadata failed')
        for channel in ('stdout', 'stderr'):
            self.file('regressions/m2/logs/m2-host-tests.' + channel + '.txt')
        for group, (name, label, feature) in zip(groups[1:], M2):
            required, code, forbidden = gate_contract.guest_expectation('m2', name)
            original = str(Path(group['evidence']))
            report, artifacts = self.guest(original, expected_exit=code)
            require(report.get('label') == label and report.get('required_markers') == list(required)
                    and report.get('forbidden_markers') == list(forbidden), 'changed M2 guest oracle')
            require(run_module.is_success(code, report['stdout'], False, required_markers=required,
                    expected_exit=code, forbidden_markers=forbidden), f'M2 transcript rejected: {name}')
            manifest = self.manifest('regressions/m2/logs/build-' + (label if feature is not None else 'normal') + '-manifest.json', feature)
            self.bind_artifacts(artifacts, manifest)
        # K05: exact six source-bound process lifecycle cases and four host checks,
        # on one CPU and, by K06 5d-2 section 4, on four vCPUs in native-smp-m3.
        for leaf, gate, smp in (('k05', 'supervisor-gate.json', False),
                                ('k05-smp', 'supervisor-smp-gate.json', True)):
            features = supervisor.SMP_FEATURE if smp else supervisor.FEATURE
            suite = self.load(f'regressions/{leaf}/{gate}')
            self.bound(suite)
            require(suite.get('passed') is True and suite.get('status') == 'completed'
                    and suite.get('smp', False) is smp
                    and 'error' not in suite and suite.get('test_inputs') == supervisor_tests.test_inputs(),
                    'K05 suite/test-source binding mismatch')
            require(len(suite.get('host', [])) == 4 and all(item.get('exit') == 0 for item in suite['host']),
                    'K05 host checks incomplete')
            for item, expected in zip(suite['host'], supervisor_tests.HOST):
                command = item.get('command', [])
                require(command[1:] == expected[1:] and
                        (command[0] == 'cargo' if expected[0] == 'cargo' else Path(command[0]).name.startswith('python')),
                        'changed K05 host command')
                require(isinstance(item.get('stdout'), str) and isinstance(item.get('stderr'), str),
                        'K05 embedded host logs missing')
            require([item.get('case') for item in suite.get('stages', [])] == list(supervisor_tests.PLAN),
                    'K05 exact six-case plan mismatch')
            for item in suite['stages']:
                case = item['case']
                require(item.get('passed') is True and not item.get('error'), 'K05 stage failed')
                report, artifacts = self.guest(item['run_directory'], expected_exit=33, timeout=180,
                                               k05=True, smp=smp)
                require(report.get('case') == case and report.get('kernel_feature') == features[case]
                        and report.get('smp', False) is smp, 'K05 profile mismatch')
                observed = supervisor.evaluate(case, report['stdout'], report['raw_qemu_exit'],
                                               report['timed_out'], smp)
                require(observed == report.get('evidence') == item.get('evidence')
                        and report['raw_qemu_exit'] == item.get('raw_qemu_exit')
                        and report['duration_s'] == item.get('duration_s'), 'K05 raw/summary evidence mismatch')
                feature = features[case]
                manifest = self.manifest(f'regressions/{leaf}/logs/build-' + feature + '-manifest.json', feature)
                require(report.get('supervisor') == manifest.get('supervisor'), 'K05 embedded bundle metadata mismatch')
                self.bind_artifacts(artifacts, manifest)
        # K07: original one-CPU M4 quotas plus two-process qsim pressure/cleanup.
        pressure = self.load('regressions/k07/resource-pressure/result.json')
        self.bound(pressure)
        require(pressure.get('passed') is True and 'error' not in pressure
                and pressure.get('checks') == {'ipc_handle_quota_and_recovery': True,
                    'memory_cpu_budget_and_recovery': True, 'source_unchanged': True}, 'K07 incomplete checks')
        m4 = self.load('regressions/k07/resource-pressure/m4/k07-m4.json')
        raw, artifacts = self.guest(m4['run_directory'], expected_exit=33, embedded=m4)
        required, code, forbidden = gate_contract.guest_expectation('m4', 'test-m4-quotas')
        require(raw.get('required_markers') == list(required) and raw.get('forbidden_markers') == list(forbidden)
                and run_module.is_success(raw['raw_qemu_exit'], raw['stdout'], raw['timed_out'],
                    required_markers=required, expected_exit=code, forbidden_markers=forbidden), 'K07 M4 transcript rejected')
        self.bind_artifacts(artifacts, self.manifest('regressions/k07/resource-pressure/m4/manifest.json', 'test-m4-quotas'))
        require(pressure['m4']['run_directory'] == m4['run_directory']
                and pressure['m4']['raw_qemu_exit'] == 33
                and all(marker in raw['stdout'].splitlines() for marker in pressure['m4']['required_markers']), 'K07 M4 summary mismatch')
        quantum = self.load('regressions/k07/resource-pressure/qsim/native-qsim-result.json')
        self.bound(quantum)
        checks = ('before_launch_source_unchanged', 'guest_exit_sequence_profile_and_artifacts',
                  'after_run_source_unchanged', 'qemu_entry_unchanged', 'quantum_binary_snapshot',
                  'scalar_report_snapshot', 'build_artifacts_unchanged')
        require(quantum.get('passed') is True and 'error' not in quantum
                and quantum.get('checks') == dict.fromkeys(checks, True), 'qsim source/artifact checks incomplete')
        report, artifacts = self.guest(quantum['guest']['run_directory'], expected_exit=33, embedded=quantum['guest'])
        observed = qsim.validate_sequence(report['stdout'])
        require(observed == quantum.get('observed') and observed['processes'] == pressure['qsim']['processes']
                and all(observed[key] == pressure['qsim'][key] for key in ('timer_irqs', 'cleanup_frames'))
                and pressure['qsim']['raw_qemu_exit'] == 33
                and pressure['qsim']['run_directory'] == report['run_directory'], 'K07 qsim summary mismatch')
        self.bind_artifacts(artifacts, self.manifest('regressions/k07/resource-pressure/qsim/manifest.json', 'native-qsim'))
        # RTC: exact existing five controlled dates, no skipped/out-of-window case.
        suite = self.load('regressions/rtc/rtc-report.json')
        self.bound(suite)
        manifest_path = self.file('regressions/rtc/manifest.json')
        manifest = self.manifest('regressions/rtc/manifest.json', 'native-rtc')
        require(suite.get('passed') is True and suite.get('manifest_sha256') == digest(manifest_path)
                and [item.get('case') for item in suite.get('cases', [])] == list(rtc.ACCEPTANCE_CASES), 'RTC exact plan/manifest mismatch')
        for item in suite['cases']:
            case = item['case']
            require(item.get('passed') is True, 'RTC case failed')
            report, artifacts = self.guest(item['guest']['run_directory'], expected_exit=33, embedded=item['guest'])
            require(rtc.evaluate(report['stdout'], report['raw_qemu_exit'], report['timed_out'], rtc.CASES[case]) == item.get('observed'), 'RTC oracle/summary mismatch')
            self.bind_artifacts(artifacts, manifest)
        raw = {path.resolve() for path in self.root.rglob('result.json') if path.parent.parent.name == 'run'}
        require(raw == self.used and len(raw) == 20, 'missing or additional native guest evidence')
        return {'m2_groups': 8, 'm2_native_runs': 7, 'k05_cases': 6, 'k07_native_runs': 2,
                'rtc_cases': 5, 'total_native_runs': 20, 'raw_result_sha256': self.guest_hashes}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('artifact_dir', type=Path)
    parser.add_argument('--expected-commit', required=True)
    parser.add_argument('--expected-source', required=True)
    parser.add_argument('--run-json', required=True, type=Path)
    parser.add_argument('--jobs-json', required=True, type=Path)
    parser.add_argument('--repo', type=Path)
    args = parser.parse_args()
    try:
        require(re.fullmatch('[0-9a-f]{40}', args.expected_commit) is not None
                and sha(args.expected_source), 'invalid expected commit/source')
        require(args.artifact_dir.is_dir(), 'final-gate artifact directory missing')
        repo = args.repo
        if repo is None:
            repo = next((path for path in (Path.cwd(), *Path(__file__).resolve().parents)
                         if (path / 'tools/common.py').is_file()), None)
        require(repo is not None, 'pass the matching checkout as --repo')
        sys.path.insert(0, str(repo.resolve() / 'tools'))
        import common
        import gate_contract
        import run as run_module
        import run_supervisor
        import run_rtc
        import run_qsim
        import test_supervisor
        require(common.ROOT.resolve() == repo.resolve() and common.source_hash() == args.expected_source,
                'local oracle checkout does not match the expected source-v3')
        paths = list(args.artifact_dir.rglob('logs/gate.log'))
        require(len(paths) == 1, 'missing or ambiguous final gate.log')
        readback = Readback(paths[0].parent.parent, args.expected_source, args.expected_commit)
        api = readback.api(read_json(args.run_json), read_json(args.jobs_json))
        stages = readback.gate_log()
        evidence = readback.evidence(gate_contract, run_module, run_supervisor, run_rtc,
                                     run_qsim, test_supervisor)
        print(json.dumps({'passed': True, 'verification_kind': 'retained-final-gate-evidence-readback',
                          'tested_commit': args.expected_commit, 'source_sha256': args.expected_source,
                          'api': api, 'api_run_sha256': digest(args.run_json),
                          'api_jobs_sha256': digest(args.jobs_json),
                          'gate_log_sha256': digest(readback.file('logs/gate.log')),
                          'wrapper_stages': stages, 'native_evidence': evidence,
                          'limitations': LIMITS}, indent=2))
        return 0
    except (ValueError, OSError, KeyError, IndexError, TypeError, StopIteration) as error:
        print(json.dumps({'passed': False, 'error': f'{type(error).__name__}: {error}'}, indent=2))
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
