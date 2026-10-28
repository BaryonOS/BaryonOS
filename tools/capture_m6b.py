#!/usr/bin/env python3
"""Recheck complete native M6b evidence before an atomic text-only archive."""
from __future__ import annotations
from pathlib import Path
import argparse
import shutil
import sys
import tempfile
from common import ROOT, BUILD, environment, save_json, sha256, source_hash
from capture_m6 import inside_run, read_json, raw_checks
from capture_gate import _replace_archive
from m6b_disk import Volume, synthetic_initial, verify_volume
from run_m6b import BOOT, CUTS, FAILURES, WINDOWS_CRASH_EXIT, normal_success, unique_markers
from run import is_success
from test_m6b import test_sources
from build import M6B_FAULT_CASES


# These are the fixed outputs of run_m6b.py and an ordinary run.py M1 run.
# A claimed digest map is not allowed to decide which evidence is mandatory.
M6B_ARTIFACTS = frozenset((
    'esp/EFI/BOOT/BOOTX64.EFI', 'esp/kernel.elf', 'OVMF_CODE.fd',
    'OVMF_VARS.template.fd', 'OVMF_VARS.fd', 'test-disk.before.img',
    'test-disk.after.img', 'build-manifest.json', 'volume-manifest.json',
))
M1_ARTIFACTS = frozenset((
    'esp/EFI/BOOT/BOOTX64.EFI', 'esp/kernel.elf', 'OVMF_CODE.fd',
    'OVMF_VARS.template.fd',
))
HOST_GROUPS = {
    'strict-development-inventory': ('m6b-preflight', 'python',
        ('-B', 'tools/preflight.py', '--strict')),
    'object-core-and-wire-host-tests': ('m6b-host-tests', 'cargo',
        ('+1.90.0', 'test', '--locked', '--offline', '-p', 'object-core')),
    'external-oracle-and-crash-harness-tests': ('m6b-harness-tests', 'python',
        ('-B', '-m', 'unittest', 'discover', '-s', 'tests', '-p', 'test_m6b_*.py')),
}


def require_artifacts(report, phase):
    expected = {'m1': M1_ARTIFACTS, 'm6b': M6B_ARTIFACTS}[phase]
    actual = report.get('artifact_sha256')
    if not isinstance(actual, dict) or set(actual) != expected:
        raise ValueError(f'Incomplete or unexpected {phase} artifact set')


def check_host_group(name, evidence, seen=None):
    """Bind each host requirement to its own log trio and exact executed argv."""
    if name not in HOST_GROUPS:
        raise ValueError('Unknown M6b host-test group')
    stem, program, arguments = HOST_GROUPS[name]
    logs = (BUILD/'logs').resolve(strict=True)
    logs.relative_to(BUILD.resolve(strict=True))
    evidence = Path(evidence)
    if not evidence.is_absolute():
        evidence = ROOT/evidence
    if (evidence.parent.resolve(strict=True) != logs
            or evidence.name != stem+'.stdout.txt'):
        raise ValueError('Host group requires its fixed exact log path')
    files = tuple(logs/(stem+suffix) for suffix in ('.stdout.txt', '.json', '.stderr.txt'))
    identities = set()
    for path in files:
        if path.is_symlink() or not path.is_file():
            raise ValueError('Host logs must be separate regular files')
        info = path.stat()
        identity = {('inode', info.st_dev, info.st_ino), ('path', str(path))}
        if identities.intersection(identity) or seen is not None and seen.intersection(identity):
            raise ValueError('Host groups cannot reuse the same physical log file')
        identities.update(identity)
    command = [sys.executable if program == 'python' else program, *arguments]
    command[0] = shutil.which(command[0], path=environment()['PATH']) or command[0]
    metadata = read_json(files[1])
    if (not isinstance(metadata, dict)
            or metadata.get('command') != command or metadata.get('cwd') != str(ROOT)
            or type(metadata.get('returncode')) is not int or metadata['returncode'] != 0
            or metadata.get('timed_out', False) is not False):
        raise ValueError('Host command/working-directory/exit contract mismatch')
    if seen is not None:
        seen.update(identities)
    return files


def check_m1_run(directory, report, frozen):
    require_artifacts(report, 'm1')
    raw_checks(directory, report, frozen)
    if (not is_success(report['raw_qemu_exit'], report['stdout'], report['timed_out'])
            or 'M6B:' in report['stdout'] or 'volume_path' in report):
        raise ValueError('Final M1 must be an ordinary M1 run')


def required_groups():
    groups=[(name,None,None) for name in ('strict-development-inventory',
        'object-core-and-wire-host-tests','external-oracle-and-crash-harness-tests')]
    for suffix in ('a','b'):
        name='normal-'+suffix
        groups += [(name,'normal',None),(name+'-cold-recovery','recover',name)]
    groups += [('same-binary-two-canary-images',None,None)]
    for case in ('errors','full','corrupt','vm-cut-before-ack','vm-cut-after-flush') + M6B_FAULT_CASES:
        groups += [(case,case,None)]
        if case!='corrupt': groups += [(case+'-cold-recovery','recover',case)]
    return groups+[('m1-regression-after-m6b','m1',None)]


def check_run(directory,report,frozen):
    require_artifacts(report, 'm6b')
    raw_checks(directory,report,frozen)
    case=report['case']
    image=inside_run(report['volume_path'])
    metadata=read_json(directory/'volume-manifest.json')
    if metadata!=read_json(image.parent/'volume.json'):
        raise ValueError('Volume manifest changed after run')
    volume=Volume(image.parent,metadata['seed'],synthetic_initial(metadata['seed'],metadata['variant']),
                  metadata['variant'],metadata['identity'])
    if volume.guard()!=image or volume.variant!=('corrupt' if case=='corrupt' else 'normal'):
        raise ValueError('Volume identity/variant mismatch')
    before=(directory/'test-disk.before.img').read_bytes()
    after=(directory/'test-disk.after.img').read_bytes()
    if image.read_bytes()!=after:
        raise ValueError('Actual image changed after run')
    oracle=verify_volume(volume,report['stdout'],case=case,before=before)
    if not oracle['passed'] or oracle!=report['disk_oracle']:
        raise ValueError('Independent external disk oracle failed')
    manifest=read_json(directory/'build-manifest.json')
    feature='milestone-m6b' if case=='normal' else 'test-m6b-'+case
    if (manifest.get('kernel_feature')!=feature or manifest.get('loader_feature') is not None
            or manifest['source_sha256']!=frozen
            or manifest['kernel']['sha256']!=report['artifact_sha256']['esp/kernel.elf']
            or manifest['loader']['sha256']!=report['artifact_sha256']['esp/EFI/BOOT/BOOTX64.EFI']
            or report.get('execution_error') or report.get('oracle_error')
            or report.get('observed_source_sha256')!=frozen):
        raise ValueError('Wrong feature/source/artifact or failed execution')
    outcomes = [line for line in report['stdout'].splitlines() if line.startswith('M6B:OUTCOME')]
    if outcomes != (['M6B:OUTCOME:UNKNOWN'] if case in M6B_FAULT_CASES else []):
        raise ValueError('Missing, duplicate or unexpected native Unknown outcome')
    if case in CUTS:
        point=CUTS[case]; crash=report['host_crash']; raw=report['raw_qemu_exit']
        if (report['timed_out'] or raw not in (-9,WINDOWS_CRASH_EXIT)
                or not crash['kill_requested'] or not crash['was_running_before_kill']
                or crash['real_process_returncode']!=raw or crash['termination_code_expected']!=raw
                or crash['requested_cut']!=point or crash['cut_lines']!=[f'M6B:CUT:{point}']
                or crash['required_live_hold_s']!=0.05
                or not unique_markers(report['stdout'],BOOT+(f'M6B:CUT:{point}',))
                or not is_success(raw,report['stdout'],expected_exit=raw,
                    required_markers=BOOT+(f'M6B:CUT:{point}',),forbidden_markers=FAILURES+('M6B:PASS','M1:PASS'))):
            raise ValueError('Not an actual external live-child termination')
    elif not normal_success(report['raw_qemu_exit'],report['stdout'],report['timed_out'],case):
        raise ValueError('Raw normal guest evidence failed')
    if case=='recover':
        previous=inside_run(report['previous_stage']); old=read_json(previous/'result.json')
        if (previous==directory or old.get('case') in ('recover','corrupt') or old.get('passed') is not True
                or old['source_sha256']!=frozen or old['volume_path']!=report['volume_path']
                or old['disk_oracle']['identity']!=volume.identity
                or (previous/'test-disk.after.img').read_bytes()!=before or before!=after):
            raise ValueError('Recovery is not the exact previous image in a separate invocation')
    elif before!=volume.initial:
        raise ValueError('Mutator did not start with its fresh synthetic fixture')


def capture(destination=None):
    destination = (ROOT/'docs/verification/m6b' if destination is None else ROOT/destination).resolve()
    archive_root = (ROOT/'docs/verification').resolve()
    destination.relative_to(archive_root)
    if destination == archive_root:
        raise ValueError('Archive destination must be a subdirectory of docs/verification')
    suite=read_json(BUILD/'m6b-test-results.json'); frozen=source_hash()
    if (suite.get('status')!='completed' or suite.get('passed') is not True
            or suite.get('source_sha256')!=frozen or suite.get('host_test_sha256')!=test_sources()):
        raise ValueError('Only a complete passed gate on unchanged source/tests can be captured')
    expected=required_groups(); groups=suite['results']
    if [g['name'] for g in groups]!=[name for name,_,_ in expected]:
        raise ValueError('Missing/reordered/duplicate M6b coverage')
    runs=[]; by_name={}; host_files=[]; host_seen=set()
    for group,(name,case,parent) in zip(groups,expected):
        if group.get('passed') is not True: raise ValueError('Failed group cannot be relabeled')
        evidence=Path(group['evidence'])
        if not evidence.is_absolute(): evidence=ROOT/evidence
        if case is not None:
            directory=inside_run(evidence); report=read_json(directory/'result.json')
            if case=='m1':
                check_m1_run(directory,report,frozen)
            else:
                if report.get('case')!=case: raise ValueError('Group executed a different case')
                check_run(directory,report,frozen)
                if parent and report['previous_stage']!=by_name[parent]['run_directory']:
                    raise ValueError('Recovery is attached to a different parent')
            by_name[name]=report; runs.append(report)
        else:
            if name=='same-binary-two-canary-images':
                evidence=evidence.resolve(strict=True); evidence.relative_to((BUILD/'logs').resolve())
                saved=read_json(evidence); a,b=by_name['normal-a'],by_name['normal-b']
                checks=dict(same_kernel=a['artifact_sha256']['esp/kernel.elf']==b['artifact_sha256']['esp/kernel.elf'],
                    same_loader=a['artifact_sha256']['esp/EFI/BOOT/BOOTX64.EFI']==b['artifact_sha256']['esp/EFI/BOOT/BOOTX64.EFI'],
                    different_canary_images=a['disk_oracle']['sha256_initial']!=b['disk_oracle']['sha256_initial'],
                    same_explicit_fixture_state=a['disk_oracle']['state']==b['disk_oracle']['state'])
                if not all(checks.values()) or saved['checks']!=checks or saved['passed'] is not True:
                    raise ValueError('Two-image check does not match actual runs')
                host_files.append(evidence)
            else:
                host_files.extend(check_host_group(name,evidence,host_seen))
    if len(runs)!=28 or len({r['run_directory'] for r in runs})!=28:
        raise ValueError('Gate requires 28 distinct real VM invocations')
    destination.parent.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='m6b-capture-',dir=BUILD) as temporary:
        staged=Path(temporary)/'archive'; staged.mkdir()
        for path in host_files: shutil.copy2(path,staged/path.name)
        save_json(staged/'suite.json',suite)
        save_json(staged/'guest-runs.json',runs)
        save_json(staged/'capture.json',dict(passed=True,source_sha256=frozen,groups=len(groups),
            vm_invocations=len(runs),scope='Raw output, exit, image, hash, exact coverage and recovery links rechecked.',
            files={p.name:sha256(p) for p in staged.iterdir() if p.is_file()}))
        _replace_archive(staged,destination)
    print(f'M6b capture passed: {len(groups)} groups, {len(runs)} distinct actual VM runs')


if __name__=='__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--destination', help='Archive subdirectory within docs/verification/')
    capture(parser.parse_args().destination)
