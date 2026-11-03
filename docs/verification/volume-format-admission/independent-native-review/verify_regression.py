"""V4 read-only verification of the completed nine-VM regression receipt."""
from pathlib import Path
import re
import verify_receipts as v

PLAN = [('normal',None),('recover','normal'),('cut-data',None),('recover-cut','cut-data'),
        ('cut-table',None),('recover-cut','cut-table'),('cut-txn',None),('recover-cut','cut-txn'),('corrupt',None)]
FEATURES = {'normal':'native-volume','recover':'test-volume-recover','cut-data':'test-volume-cut-data',
            'cut-table':'test-volume-cut-table','cut-txn':'test-volume-cut-txn',
            'recover-cut':'test-volume-recover-cut','corrupt':'test-volume-corrupt'}
CUTS = {'cut-data':'VOLUME:CUT:DATA','cut-table':'VOLUME:CUT:TABLE','cut-txn':'VOLUME:CUT:TXN'}
HOST_CARGO = [
    ['cargo','+1.90.0','test','--locked','-p','volume-core'],
    ['cargo','+1.90.0','test','--locked','--manifest-path','sdk/Cargo.toml','-p','nieuw-os-sdk-runtime'],
]
HOST_PYTHON_ARGS = ['-B','-m','unittest','discover','-s','tests','-p','test_volume_*.py']


def logged_text_matches(path, text):
    raw = Path(path).read_bytes()
    return raw in (text.encode('utf-8'), text.replace('\n','\r\n').encode('utf-8'))


def verify_host_checks(records):
    v.need(len(records) == 3 and all(type(h['exit']) is int and h['exit'] == 0 for h in records), 'host checks failed/absent')
    host=[]
    for index,h in enumerate(records):
        command=h['command']
        if index < len(HOST_CARGO):
            v.need(command == HOST_CARGO[index], 'host cargo suite mismatch')
        else:
            v.need(isinstance(command,list) and len(command) == 1+len(HOST_PYTHON_ARGS)
                   and isinstance(command[0],str)
                   and re.fullmatch(r'python(?:\d+(?:\.\d+)*)?(?:\.exe)?', command[0].replace('\\','/').rsplit('/',1)[-1], re.IGNORECASE)
                   and command[1:] == HOST_PYTHON_ARGS, 'host Python suite mismatch')
        totals=[line for line in (h['stdout']+h['stderr']).splitlines()
                if line.startswith(('test result:','Ran ','OK','FAILED'))]
        v.need(totals == h['totals'], 'host test totals mismatch')
        v.need(totals and not any('FAILED' in l or re.search(r'\b[1-9]\d* failed\b',l) for l in totals), 'host test failure in complete logs')
        if index < len(HOST_CARGO):
            results=[re.fullmatch(r'test result: ok\. (\d+) passed; 0 failed; \d+ ignored; \d+ measured; \d+ filtered out; finished in .+', line) for line in totals]
            v.need(all(results) and sum(int(result[1]) for result in results) > 0, 'host cargo test count absent/zero')
        else:
            count = re.fullmatch(r'Ran ([1-9]\d*) tests? in .+', totals[0]) if len(totals) == 2 else None
            result = re.fullmatch(r'OK(?: \(skipped=(\d+)\))?', totals[1]) if len(totals) == 2 else None
            v.need(count and result and int(result[1] or 0) < int(count[1]), 'host Python test count absent/zero')
        host.append({'command':command,'exit':h['exit'],'totals_reconstructed':totals})
    return host


def verify_cut(case, result):
    v.need(result['host_kill'] is True and result['running_before_kill'] is True
           and type(result['raw_qemu_exit']) is int and result['raw_qemu_exit'] == 1,
           f'{case}: live host kill/exit not recorded')


def verify():
    root=v.NATIVE/'regression'
    invocation=v.read(root/'invocation.json')
    v.need(invocation.get('passed') is True and invocation.get('returncode') == 0, 'regression invocation incomplete/failed')
    v.need(invocation.get('stage_count') == 9 and invocation.get('source_unchanged') is True, 'regression invocation scope/source')
    source=v.verify_source(invocation['source'])
    gate_path=root/'build/volume-gate.json'
    gate=v.read(gate_path)
    v.need(Path(invocation['gate_path']).resolve() == gate_path.resolve(), 'wrong gate path')
    v.need(gate.get('passed') is True and gate.get('status') == 'completed', 'gate incomplete/failed')
    v.need(gate['source_sha256'] == source['source_sha256'], 'gate source fingerprint')
    v.need([(s['case'],s['parent']) for s in gate['stages']] == PLAN, 'stage sequence mismatch')
    for name, digest in gate['test_inputs'].items():
        v.need(v.sha(v.SOURCE/name) == digest, f'host test input differs: {name}')
    expected_tests = {p.relative_to(v.SOURCE).as_posix() for p in (v.SOURCE/'tests').rglob('test_volume_*.py')}
    expected_tests |= {p.relative_to(v.SOURCE).as_posix() for p in (v.SOURCE/'tests').rglob('__init__.py')}
    v.need(set(gate['test_inputs']) == expected_tests, 'missing/extra host test fingerprints')
    host=verify_host_checks(gate['host'])
    expected_initial=v.sha(v.NATIVE/'seeds/own-empty.img')
    snapshots={}
    checked=[]
    for index,stage in enumerate(gate['stages']):
        case,parent=PLAN[index]
        directory=Path(stage['run_directory']).resolve(strict=True)
        directory.relative_to((root/'build/run').resolve())
        r=v.read(directory/'result.json')
        v.need(stage['passed'] is True and r['passed'] is True and not r['timed_out'], f'{case}: failed/incomplete')
        v.need(r['case']==case and r['parent_case']==parent and r['kernel_feature']==FEATURES[case], f'{case}: case/profile')
        v.need(r['source_sha256']==source['source_sha256'],f'{case}: source')
        for field in ('volume_identity','before_sha256','after_sha256','raw_qemu_exit'):
            v.need(r[field]==stage[field],f'{case}: report versus stage {field}')
        manifest_path=root/'build/logs'/f'build-{FEATURES[case]}-manifest.json'
        manifest=v.read(manifest_path)
        v.need(manifest['source_sha256']==source['source_sha256'] and manifest['kernel_feature']==FEATURES[case],f'{case}: build manifest binding')
        kernel=v.sha(directory/'esp/kernel.elf')
        loader=v.sha(directory/'esp/EFI/BOOT/BOOTX64.EFI')
        v.need(kernel==r['kernel_sha256']==manifest['kernel']['sha256'],f'{case}: kernel hash')
        v.need(loader==manifest['loader']['sha256'],f'{case}: loader hash')
        v.need(v.sha(directory/'OVMF_CODE.fd')==v.sha(v.NATIVE/'runs/own-normal/OVMF_CODE.fd'),f'{case}: firmware differs')
        snapshot=v.sha(directory/'volume.after.img')
        volume=Path(r['volume_directory']).resolve(strict=True)
        volume.relative_to((root/'build/run').resolve())
        v.need(snapshot==r['after_sha256']==v.sha(volume/'volume.img'),f'{case}: preserved image bytes differ')
        info=v.read(volume/'volume.json')
        v.need(info['identity']==r['volume_identity'],f'{case}: volume identity')
        if parent:
            p=snapshots[parent]
            v.need(r['before_sha256']==r['after_sha256']==p['after_sha256'],f'{case}/{parent}: cold bytes mismatch')
            v.need(volume==Path(p['volume_directory']).resolve() and r['volume_identity']==p['volume_identity'],f'{case}/{parent}: cold volume identity mismatch')
        else:
            v.need(info['sha256']==r['before_sha256'],f'{case}: initial metadata/hash mismatch')
            if case!='corrupt':
                v.need(r['before_sha256']==expected_initial,f'{case}: initial image differs from preserved canonical seed')
            else:
                v.need(r['before_sha256']==snapshot,f'{case}: read-only corrupt image changed')
        v.need(logged_text_matches(directory/'serial.txt',r['stdout']),f'{case}: serial file/report mismatch')
        v.need(logged_text_matches(directory/'stderr.txt',r['stderr']),f'{case}: stderr file/report mismatch')
        lines=r['stdout'].splitlines()
        expected=v.BOOT.copy()
        if case in CUTS:
            expected += [CUTS[case]]
            verify_cut(case,r)
        else:
            name={'normal':'NORMAL','recover':'RECOVER','recover-cut':'RECOVER_CUT','corrupt':'CORRUPT'}[case]
            expected += ['VOLUME:READY',f'VOLUME:CASE:{name}:PASS','VOLUME:RESOURCES:PASS','VOLUME:PASS','M1:PASS']
            v.need(r['raw_qemu_exit']==33 and not r['host_kill'],f'{case}: completion/exit')
        controlled=[l for l in lines if l in v.BOOT+['VOLUME:READY','VOLUME:RESOURCES:PASS','VOLUME:PASS','M1:PASS'] or l.startswith(('VOLUME:CASE:','VOLUME:CUT:'))]
        v.need(controlled==expected,f'{case}: ordered markers')
        v.need(not any('FAIL' in l or 'PANIC' in l for l in lines),f'{case}: failure marker')
        numeric={}
        for line in lines:
            m=re.fullmatch(r'(VOLUME:[A-Z_:]+:)([0-9]{20})',line)
            if m:
                v.need(m[1] not in numeric,f'{case}: duplicate numeric evidence')
                numeric[m[1]]=int(m[2])
        v.need(numeric==r['evidence'],f'{case}: numeric serial evidence')
        if parent:
            p=snapshots[parent]
            v.need(r['oracle']['generation']==p['oracle']['generation'] and r['oracle']['digests']==p['oracle']['digests'],f'{case}/{parent}: recovered oracle state differs')
        if case in ('normal','recover'):
            v.need(r['oracle']['generation']==84 and r['oracle']['digests']==[5586531168511146659,16427685867521082448],f'{case}: normal oracle values')
        elif case!='corrupt':
            v.need(r['oracle']['generation']==(3 if 'cut-txn' in (case,parent) else 1),f'{case}: pre-cut generation')
        snapshots[case]=r
        checked.append({'case':case,'parent':parent,'passed':True,'raw_qemu_exit':r['raw_qemu_exit'],
            'after_sha256_recomputed':snapshot,'kernel_sha256_recomputed':kernel,'loader_sha256_recomputed':loader,
            'serial_sha256_recomputed':v.sha(directory/'serial.txt'),'manifest_sha256':v.sha(manifest_path),
            'host_kill_recorded':r['host_kill']})
    v.need('M6e gate passed=True stages=9' in (root/'stdout.txt').read_text(encoding='utf-8',errors='replace'), 'top-level regression stdout completion absent')
    return {'passed':True,'invocation_sha256':v.sha(root/'invocation.json'),'gate_sha256':v.sha(gate_path),
            'source':source,'stage_count':len(checked),'cold_pairs_verified':4,'host_checks':host,'stages':checked,
            'verifier_sha256':v.sha(__file__),'helper_sha256':v.sha(v.__file__),
            'trace_recount_scope':'10-case matrix only; the standard nine-VM regression does not enable read/write tracing',
            'native_btrfs_or_openzfs_support_verified':False}


if __name__=='__main__':
    try:
        v.save_receipt('regression-review.json',verify())
    except Exception as exc:
        v.save_receipt('regression-review-failure.json',{'passed':False,'error':str(exc),'verifier_sha256':v.sha(__file__)})
        raise
