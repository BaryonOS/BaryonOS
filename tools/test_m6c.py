#!/usr/bin/env python3
"""M6c native gate: two domains, CRUD, tombstones and actual cold-VM recovery."""
from __future__ import annotations
from datetime import datetime, timezone
import json
import sys
from common import ROOT, BUILD, run_logged, save_json, sha256, source_hash
from build import build
from run import run_guest
from run_m6c import run_m6c


def test_sources():
    return {p.relative_to(ROOT).as_posix(): sha256(p)
            for p in sorted((ROOT/'tests').glob('test_m6c_*.py'))}


def test_m6c():
    path = BUILD/'m6c-test-results.json'
    frozen = source_hash()
    tests = test_sources()
    suite = dict(phase='M6c',status='running',passed=False,
                 source_fingerprint_version=3, source_sha256=frozen,
                 host_test_sha256=tests, started_utc=datetime.now(timezone.utc).isoformat(),results=[])
    save_json(path,suite)

    def record(name, passed, evidence, requirements, **details):
        suite['results'].append(dict(name=name,passed=bool(passed),evidence=str(evidence),
                                    requirements=requirements,**details))
        save_json(path,suite)
        if not passed: raise RuntimeError(f'M6c gate failed: {name}')

    def guest(case, label, volume=None):
        result, volume = run_m6c(case,label=label,volume=volume)
        record(label,result['passed'] and result['source_sha256']==frozen,
               result['run_directory'],['M6C-'+case.upper(),'M6C-EXTERNAL-DISK-ORACLE'])
        return result,volume

    def recovery(previous,volume,label):
        build(kernel_feature='test-m6c-recover',label='m6c-recover')
        result,_ = run_m6c('recover',volume=volume,label=label)
        checks = dict(same_volume=previous['volume_path']==result['volume_path'],
            same_identity=previous['disk_oracle']['identity']==result['disk_oracle']['identity'],
            separate_vm=previous['run_directory']!=result['run_directory'],
            linked_previous=result['previous_stage']==previous['run_directory'],
            exact_previous_bytes=result['artifact_sha256']['test-disk.before.img']==previous['artifact_sha256']['test-disk.after.img'],
            no_writes=result['artifact_sha256']['test-disk.before.img']==result['artifact_sha256']['test-disk.after.img'],
            frozen_source=result['source_sha256']==previous['source_sha256']==frozen)
        record(label,result['passed'] and all(checks.values()),result['run_directory'],
               ['M6C-COLD-RECOVERY','M6C-NO-REPLAY','M6C-EXACT-SAME-IMAGE'],recovery_checks=checks)

    try:
        run_logged([sys.executable,'-B','tools/preflight.py','--strict'],'m6c-preflight')
        record('strict-development-inventory',True,'build/logs/m6c-preflight.stdout.txt',['M6C-TOOLS'])
        run_logged(['cargo','+1.90.0','test','--locked','--offline','-p','catalog-core'],'m6c-host-tests')
        record('catalog-core-and-wire-host-tests',True,'build/logs/m6c-host-tests.stdout.txt',
               ['M6C-REVISIONS','M6C-TOMBSTONES','M6C-BOUNDED-RECOVERY'])
        run_logged([sys.executable,'-B','-m','unittest','discover','-s','tests','-p','test_m6c_*.py'],
                   'm6c-harness-tests')
        record('external-oracle-and-crash-harness-tests',True,'build/logs/m6c-harness-tests.stdout.txt',
               ['M6C-EVIDENCE','M6C-IMAGE-BOUNDARY'])
        normals=[]
        for suffix in ('a','b'):
            build(kernel_feature='milestone-m6c',label='m6c-normal')
            result,volume=guest('normal',f'normal-{suffix}')
            normals.append(result)
            recovery(result,volume,f'normal-{suffix}-cold-recovery')
        a,b=normals
        checks=dict(same_kernel=a['artifact_sha256']['esp/kernel.elf']==b['artifact_sha256']['esp/kernel.elf'],
                    same_loader=a['artifact_sha256']['esp/EFI/BOOT/BOOTX64.EFI']==b['artifact_sha256']['esp/EFI/BOOT/BOOTX64.EFI'],
                    different_canary_images=a['disk_oracle']['sha256_initial']!=b['disk_oracle']['sha256_initial'],
                    same_explicit_fixture_state=a['disk_oracle']['state']==b['disk_oracle']['state'])
        comparison=dict(checks=checks,passed=all(checks.values()),
                        scope='Canary seed variation; document bytes are explicitly deterministic.',
                        run_directories=[a['run_directory'],b['run_directory']])
        save_json(BUILD/'logs/m6c-two-image-check.json',comparison)
        record('same-binary-two-canary-images',comparison['passed'],'build/logs/m6c-two-image-check.json',
               ['M6C-EXTERNAL-DISK-ORACLE'])
        for case in ('errors','full','corrupt','legacy','vm-cut-before-rename','vm-cut-after-rename'):
            build(kernel_feature='test-m6c-'+case,label='m6c-'+case)
            result,volume=guest(case,case)
            if case not in ('corrupt','legacy'): recovery(result,volume,case+'-cold-recovery')
        baseline=build()
        regression=run_guest('m1-after-m6c',
            expected_kernel_sha256=baseline['kernel']['sha256'],
            expected_loader_sha256=baseline['loader']['sha256'])
        record('m1-regression-after-m6c',regression['passed'] and regression['source_sha256']==frozen,
               regression['run_directory'],['M1-BOOT'])
        if source_hash()!=frozen or test_sources()!=tests:
            raise RuntimeError('Source or host tests changed during M6c gate')
        suite.update(status='completed',passed=True)
    except Exception as exc:
        suite.update(status='failed',passed=False,error=str(exc))
        raise
    finally:
        suite['finished_utc']=datetime.now(timezone.utc).isoformat()
        save_json(path,suite)
    print(f'M6c gate passed: {len(suite["results"])} groups; broader regression separate')


if __name__=='__main__': test_m6c()
