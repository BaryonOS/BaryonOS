#!/usr/bin/env python3
"""Read-only A04 M6b regression audit; standard library only, no project imports.

Requires the captured archive, its referenced actual build/run artifacts, and
the exact source commit. --root can relocate the recorded repository tree.
All output is explicitly selected with --output-prefix. This runs no guest.
"""
import argparse, ast, datetime, hashlib, io, json, pathlib, random, re, stat, struct, subprocess, sys, tarfile

def sha(data): return hashlib.sha256(data).hexdigest()
def load(path): return json.loads(path.read_text())
def require(condition, message):
    if not condition: raise AssertionError(message)

def fnv(data):
    h = 0xcbf29ce484222325
    for x in data: h = ((h ^ x) * 0x100000001b3) & 0xffffffffffffffff
    return h
def value(obj, rev): return bytes((49+29*obj+7*rev+3*i)%256 for i in range(128))
def parse(image):
    require(len(image)==65536, 'volume size')
    objects = [dict(revision=0,present=False,length=0,digest=f'{fnv(b""):016x}') for _ in range(4)]
    seq = 0; gap = False; decoded = []
    for slot in range(32):
        b = image[(4+slot)*512:(5+slot)*512]
        if b==bytes(512): gap=True; continue
        require(not gap, 'occupied record after empty slot')
        require(b[:8]==b'NOSM6B01', 'magic')
        version,header,flags,number,request,obj,rev,op,size = struct.unpack_from('<HHIQQQQHH',b,8)
        require((version,header,flags)==(1,64,0), 'record header')
        require(number==slot+1 and request>0 and obj<4 and op in (2,3) and size<=128, 'record fields')
        require(not any(b[52:64]) and not any(b[64+size:504]), 'canonical reserved bytes')
        require(int.from_bytes(b[504:],'little')==fnv(b[:504]+bytes(8)), 'FNV diagnostic checksum')
        old=objects[obj]
        require(rev==old['revision']+1, 'object revision continuity')
        require(op!=3 or old['present'] and size==0, 'DELETE predecessor')
        payload=b[64:64+size]
        require(payload==(value(obj,rev) if op==2 else b''), 'independent deterministic fixture bytes')
        objects[obj]=dict(revision=rev,present=op==2,length=size,digest=f'{fnv(payload):016x}')
        seq=number; decoded.append(dict(sequence=seq,object=obj,revision=rev,operation=op))
    return dict(sequence=seq,objects=objects), decoded
def marker(lines,key):
    vals=[line[len(key)+1:] for line in lines if line.startswith(key+':')]
    require(len(vals)==1,'exact unique marker '+key)
    return vals[0]
def ordered(lines,required):
    positions=[]
    for x in required:
        require(lines.count(x)==1,'unique control marker '+x)
        positions.append(lines.index(x))
    require(positions==sorted(positions),'ordered markers')

RAW_INVENTORY_PATH = pathlib.Path(__file__).with_name('a04_m6b_raw_hashes.json')
RAW_INVENTORY_SHA256 = 'e1aac2a56d69ef5e13eefdfff1e7ee91c2371b480ab33ddfe8772b500cff0753'


def decode_raw_inventory(data):
    """Read the fixed historical inventory, never an audit's generated output."""
    require(sha(data) == RAW_INVENTORY_SHA256, 'pinned historical raw inventory hash')
    inventory = json.loads(data)
    require(isinstance(inventory, dict) and len(inventory) == 164,
            'complete historical 164-file raw inventory')
    return inventory


def validate_raw_inventory(actual, expected):
    """Reject missing, extra, renamed or changed raw artifacts."""
    require(set(actual) == set(expected), 'exact raw-file inventory')
    require(actual == expected, 'exact raw-file SHA-256 values')


def validate_output_prefix(prefix):
    """An audit must never replace its pinned historical input manifest."""
    protected = RAW_INVENTORY_PATH.resolve(strict=True)
    for suffix in ('_audit.json', '_sectors.json', '_source_hashes.json', '_raw_hashes.json'):
        target = pathlib.Path(str(prefix) + suffix)
        require(target.resolve() != protected and
                not (target.exists() and target.samefile(protected)),
                'audit output cannot overwrite historical raw inventory')


def validate_qemu(record, relative_run, program, recorded_digest, actual_digest,
                  *, image_relative=None, has_share=False):
    """Validate the full fixed M1/M6b VM profile and executable binding."""
    is_m1 = record.get('case', 'm1') == 'm1'
    require(is_m1 == (image_relative is None), 'QEMU disk contract matches case')
    require(isinstance(recorded_digest, str) and re.fullmatch(r'[0-9a-f]{64}', recorded_digest),
            'recorded QEMU SHA-256')
    require(actual_digest == recorded_digest, 'actual QEMU executable hash')
    if not is_m1 or 'qemu_sha256' in record:
        require(record.get('qemu_sha256') == recorded_digest, 'record QEMU executable hash')
    expected = [program, '-machine', 'pc-q35-10.0', '-accel', 'tcg,thread=single',
        '-cpu', 'qemu64', '-smp', '1', '-m', '256M', '-nodefaults', '-display', 'none',
        '-monitor', 'none', '-serial', 'stdio', '-nic', 'none', '-no-reboot',
        '-drive', f'if=pflash,unit=0,format=raw,readonly=on,file={relative_run}/OVMF_CODE.fd',
        '-drive', f'if=pflash,unit=1,format=raw,file={relative_run}/OVMF_VARS.fd',
        '-drive', f'if=virtio,index=0,format=raw,readonly=on,file=fat:{relative_run}/esp',
        '-device', 'isa-debug-exit,iobase=0xf4,iosize=0x04']
    if not is_m1:
        expected += ['-drive', f'if=none,id=m6b-test,format=raw,cache=writeback,readonly=off,file={image_relative}',
            '-device', 'virtio-blk-pci,drive=m6b-test,id=m6bblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-M6B-TEST']
    if has_share:
        expected += ['-L', str(pathlib.PurePosixPath(program).parent/'share')]
    require(record['command'] == expected, 'exact isolated QEMU command')


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=pathlib.Path, default=pathlib.Path.cwd())
    parser.add_argument('--archive', default='docs/verification/m6d-m6b-regression')
    parser.add_argument('--commit', default='88e6c04be218b8b09ea4f0a226f6b9719da706a6')
    parser.add_argument('--fingerprint', default='8ca6d5d5291fde62f91fb47dca7df4d058424604449fb32b9e5899b1d4fbef93')
    parser.add_argument('--output-prefix', type=pathlib.Path, default=pathlib.Path('/tmp/a04_m6b'))
    args = parser.parse_args(argv)
    ROOT = args.root.resolve(strict=True)
    EVIDENCE = (ROOT/args.archive).resolve(strict=True)
    COMMIT, FROZEN, PREFIX = args.commit, args.fingerprint, str(args.output_prefix)
    DEST = pathlib.Path(PREFIX+'_audit.json')
    if not (EVIDENCE/'capture.json').is_file():
        raise SystemExit('Capture does not exist; no completed audit can be claimed.')
    validate_output_prefix(args.output_prefix)
    def local_path(value):
        original = pathlib.Path(value)
        relative = original.relative_to(RECORDED_ROOT) if original.is_absolute() else original
        require('..' not in relative.parts, 'recorded path escapes repository')
        return ROOT/relative

    RECORDED_ROOT = pathlib.Path(load(EVIDENCE/'m6b-preflight.json')['cwd'])
    suite_path=ROOT/'build/m6b-test-results.json'; suite=load(suite_path)
    require(suite['status']=='completed' and suite['passed'] is True,'completed suite')
    require(suite['source_sha256']==FROZEN and suite['source_fingerprint_version']==3,'suite fingerprint')
    archive=tarfile.open(fileobj=io.BytesIO(subprocess.check_output(['git','archive',COMMIT],cwd=ROOT)))
    files={m.name:archive.extractfile(m).read() for m in archive.getmembers() if m.isfile()}
    h=hashlib.sha256(b'nieuw-os source inputs v3\n'); source_inputs={}
    for folder in ('boot','kernel','crates','drivers','services','userspace','tools','.cargo'):
        for name in sorted((n for n in files if n.startswith(folder+'/')), key=lambda n: pathlib.PurePosixPath(n).parts):
            if {'__pycache__','verification','target','build','.git'}.intersection(pathlib.PurePosixPath(name).parts[1:]): continue
            h.update(name.encode()); h.update(files[name]); source_inputs[name]=sha(files[name])
    for name in ('Cargo.toml','Cargo.lock','rust-toolchain.toml'):
        if name in files: h.update(name.encode()); h.update(files[name]); source_inputs[name]=sha(files[name])
    require(h.hexdigest()==FROZEN,'git archive source fingerprint')
    for name, digest in source_inputs.items():
        require(sha((ROOT/name).read_bytes())==digest, 'current source equals exact commit: '+name)
    require(set(suite['host_test_sha256'])=={n for n in files if re.fullmatch(r'tests/test_m6b_[^/]+\.py',n)}, 'complete host-test source set')
    for name,digest in suite['host_test_sha256'].items():
        require(sha(files[name])==digest and sha((ROOT/name).read_bytes())==digest,'host test binding')

    names=['strict-development-inventory','object-core-and-wire-host-tests','external-oracle-and-crash-harness-tests',
           'normal-a','normal-a-cold-recovery','normal-b','normal-b-cold-recovery','same-binary-two-canary-images',
           'errors','errors-cold-recovery','full','full-cold-recovery','corrupt','vm-cut-before-ack',
           'vm-cut-before-ack-cold-recovery','vm-cut-after-flush','vm-cut-after-flush-cold-recovery','m1-regression-after-m6b']
    require([g['name'] for g in suite['results']]==names,'exact 18-group contract')
    require(all(g['passed'] is True for g in suite['results']),'all groups passed')
    capture=load(EVIDENCE/'capture.json')
    require(capture['passed'] is True and capture['source_sha256']==FROZEN and capture['groups']==18 and capture['vm_invocations']==14,'capture contract')
    require(load(EVIDENCE/'suite.json')==suite,'captured suite equals actual suite')
    for name,digest in capture['files'].items(): require(sha((EVIDENCE/name).read_bytes())==digest,'capture file hash '+name)

    recorded_python=load(EVIDENCE/'m6b-preflight.json')['command'][0]
    require(pathlib.Path(recorded_python).name.startswith('python'), 'recorded Python executable')
    host_specs=[('m6b-preflight',[recorded_python,'-B','tools/preflight.py','--strict']),
     ('m6b-host-tests',[str(RECORDED_ROOT/'.tools/nieuw-os/cargo/bin/cargo'),'+1.90.0','test','--locked','--offline','-p','object-core']),
     ('m6b-harness-tests',[recorded_python,'-B','-m','unittest','discover','-s','tests','-p','test_m6b_*.py'])]
    host=[]; identities=set()
    for stem,argv in host_specs:
        meta=load(ROOT/'build/logs'/f'{stem}.json')
        require(meta['command']==argv and meta['cwd']==str(RECORDED_ROOT) and type(meta['returncode']) is int and meta['returncode']==0 and not meta.get('timed_out',False),'host command '+stem)
        digests={}
        for suffix in ('.json','.stdout.txt','.stderr.txt'):
            path=ROOT/'build/logs'/f'{stem}{suffix}'; info=path.stat(); ident=(info.st_dev,info.st_ino)
            require(path.is_file() and not path.is_symlink() and ident not in identities,'distinct physical host logs')
            identities.add(ident); digests[suffix]=sha(path.read_bytes())
            require((EVIDENCE/path.name).read_bytes()==path.read_bytes(), 'captured actual host bytes: '+path.name)
        host.append(dict(stem=stem,command=argv,returncode=0,sha256=digests))
    rust=(ROOT/'build/logs/m6b-host-tests.stdout.txt').read_text()
    rust_count=sum(map(int,re.findall(r'test result: ok\. (\d+) passed;',rust)))
    python=(ROOT/'build/logs/m6b-harness-tests.stderr.txt').read_text()
    python_count=int(re.search(r'Ran (\d+) tests',python).group(1))
    expected_python_count=sum(
        sum(isinstance(method, (ast.FunctionDef, ast.AsyncFunctionDef)) and method.name.startswith('test_')
            for method in definition.body)
        for name in suite['host_test_sha256']
        for definition in ast.parse(files[name]).body if isinstance(definition, ast.ClassDef))
    require(rust_count==22 and python_count==expected_python_count and '\nOK\n' in python,'actual host test counts')

    # M1 did not record its own executable hash. Bind its command to the same
    # executable and recorded hash as all 13 M6b runs, then hash that actual file.
    # This cannot recreate a missing gate-time M1 executable measurement.
    archived_runs = load(EVIDENCE/'guest-runs.json')
    qemu_bindings = {(r['command'][0], r['qemu_sha256']) for r in archived_runs
                     if r.get('case', 'm1') != 'm1'}
    require(len(qemu_bindings) == 1, 'one recorded M6b QEMU executable and hash')
    qemu_program, qemu_digest = next(iter(qemu_bindings))
    qemu = local_path(qemu_program)
    actual_qemu_digest = sha(qemu.read_bytes())
    expected_raw_inventory = decode_raw_inventory(RAW_INVENTORY_PATH.read_bytes())

    m1set={'esp/kernel.elf','esp/EFI/BOOT/BOOTX64.EFI','OVMF_CODE.fd','OVMF_VARS.template.fd'}
    m6set=m1set|{'OVMF_VARS.fd','test-disk.before.img','test-disk.after.img','build-manifest.json','volume-manifest.json'}
    seen=set(); volumes=set(); vm=[]; by_name={}; originals=[]; hash_count=0; cold_count=0; kill_count=0
    sector_records={}; raw_hashes={}
    for group in suite['results']:
        directory=local_path(group['evidence'])
        if not directory.is_dir(): continue
        require(directory.resolve()==directory and directory not in seen,'unique canonical VM dir')
        directory.relative_to(ROOT/'build/run'); seen.add(directory)
        r=load(directory/'result.json'); originals.append(r); by_name[group['name']]=r
        require(local_path(r['run_directory'])==directory and r['passed'] is True and not r['timed_out'],'report identity/result')
        require(r['base_commit']==COMMIT and r['source_sha256']==FROZEN and r['source_fingerprint_version']==3,'exact source checkpoint')
        lines=r['stdout'].splitlines()
        require(not any(line.startswith(('M6B:OUTCOME','M6C:','M6D:')) for line in lines), 'no different phase or unknown outcome')
        for rawfile in sorted(directory.rglob('*')):
            if rawfile.is_file():
                require(not rawfile.is_symlink(), 'raw artifact cannot be a symlink')
                raw_hashes[rawfile.relative_to(ROOT).as_posix()]=sha(rawfile.read_bytes())
        require((directory/'serial.txt').read_bytes().decode('utf-8','replace')==r['stdout'],'raw stdout')
        require((directory/'stderr.txt').read_bytes().decode('utf-8','replace')==r['stderr'],'raw stderr')
        case=r.get('case','m1'); expected=m1set if case=='m1' else m6set
        require(set(r['artifact_sha256'])==expected,'mandatory artifact set')
        for name,digest in r['artifact_sha256'].items():
            p=directory/name; require(p.is_file() and not p.is_symlink() and sha(p.read_bytes())==digest,'actual artifact '+name); hash_count+=1
        entry=dict(group=group['name'],case=case,run_directory=str(directory),raw_qemu_exit=r['raw_qemu_exit'],
          result_sha256=sha((directory/'result.json').read_bytes()),serial_sha256=sha((directory/'serial.txt').read_bytes()),
          artifact_hashes=len(expected))
        require(not any(':FAIL' in line for line in lines),'no failure marker')
        image_relative = None if case == 'm1' else local_path(r['volume_path']).relative_to(ROOT).as_posix()
        validate_qemu(r, directory.relative_to(ROOT).as_posix(), qemu_program,
                      qemu_digest, actual_qemu_digest, image_relative=image_relative,
                      has_share=(qemu.parent/'share').is_dir())
        if case=='m1':
            require(r['raw_qemu_exit']==33 and not any(line.startswith(('M2:','M3:','M4:','M5:','M6A:','M6B:','M6C:')) for line in lines),'ordinary M1 only')
            ordered(lines,['BOOT:EBS_OK','KERNEL:ENTRY','BOOTINFO:OK','M1:PASS'])
            require(r['artifact_binding_passed'] is True and r['expected_artifact_sha256']=={k:r['artifact_sha256'][k] for k in ('esp/kernel.elf','esp/EFI/BOOT/BOOTX64.EFI')},'M1 expected build binding')
            entry['kernel_sha256']=r['artifact_sha256']['esp/kernel.elf']; vm.append(entry); continue
        require(r['observed_source_sha256']==FROZEN and r['source_unchanged'] is True and not r['execution_error'] and not r['oracle_error'],'unchanged executed source')
        require(sha((directory/'serial.txt').read_bytes())==r['serial_sha256'] and sha((directory/'stderr.txt').read_bytes())==r['stderr_sha256'],'raw hashes')
        manifest=load(directory/'build-manifest.json')
        feature='milestone-m6b' if case=='normal' else 'test-m6b-'+case
        require(manifest['kernel_feature']==feature and manifest['loader_feature'] is None and manifest['source_sha256']==FROZEN,'exact native feature')
        require(manifest['kernel']['sha256']==r['artifact_sha256']['esp/kernel.elf'] and manifest['loader']['sha256']==r['artifact_sha256']['esp/EFI/BOOT/BOOTX64.EFI'],'manifest actual binaries')
        image=local_path(r['volume_path']); image.relative_to(ROOT/'build/run'); info=image.stat()
        require(image.resolve()==image and not image.is_symlink() and stat.S_ISREG(info.st_mode) and info.st_nlink==1 and info.st_size==65536,'isolated actual volume')
        volumes.add(image); meta=load(image.parent/'volume.json')
        require(meta==load(directory/'volume-manifest.json'),'volume identity')
        require(type(meta['seed']) is int and 0<=meta['seed']<2**64 and meta['variant'] in ('normal','corrupt'), 'seed/variant contract')
        require(meta['scope']=='fresh synthetic M6b volume' and meta['bytes']==65536 and meta['bdf']=='00:05.0','synthetic disk authority')
        initial=bytearray(random.Random(meta['seed']).randbytes(65536)); initial[:1024]=bytes(1024)
        label=b'NOS-M6B-TEST-VOLUME\0'; initial[:len(label)]=label; initial[2048:18432]=bytes(16384)
        if meta['variant']=='corrupt': initial[2048]=128
        require(meta['initial_sha256']==sha(initial) and meta['identity']==r['disk_oracle']['identity'],'generated initial image')
        before=(directory/'test-disk.before.img').read_bytes(); after=(directory/'test-disk.after.img').read_bytes()
        require(image.read_bytes()==after and before[:2048]==after[:2048]==initial[:2048] and before[18432:]==after[18432:]==initial[18432:],'actual image and outside-journal canaries')
        if case=='recover':
            cold_count+=1; parent_name=group['name'].removesuffix('-cold-recovery'); parent=by_name[parent_name]
            require(r['previous_stage']==parent['run_directory'] and r['volume_path']==parent['volume_path'],'linked distinct cold VM')
            require(before==after==(local_path(parent['run_directory'])/'test-disk.after.img').read_bytes(),'exact unchanged cold image')
        else: require(before==bytes(initial) and r['previous_stage'] is None,'fresh mutator initial image')
        if case=='corrupt':
            require(before==after,'corrupt image not changed')
            try: parse(after)
            except AssertionError: pass
            else: raise AssertionError('corrupt image accepted by independent parser')
            state=None; decoded=[]
        else: state,decoded=parse(after)
        require(state==r['disk_oracle']['state'],'independent state equals recorded oracle')
        require(r['disk_oracle']['passed'] is True and all(v is True for v in r['disk_oracle']['checks'].values()), 'all recorded disk checks passed')
        encoded={str(slot):after[(4+slot)*512:(5+slot)*512].hex() for slot in range(32)
                 if after[(4+slot)*512:(5+slot)*512]!=bytes(512)}
        restored=bytearray(initial); restored[2048:18432]=bytes(16384)
        for slot, hexdata in encoded.items(): restored[(4+int(slot))*512:(5+int(slot))*512]=bytes.fromhex(hexdata)
        require(bytes(restored)==after, 'durable sector/seed exact reconstruction')
        volume_record=dict(metadata=meta,initial_sha256=sha(initial),final_sha256=sha(after),journal_sector_hex=encoded,independent_state=state)
        previous_record=sector_records.get(meta['identity'])
        require(previous_record is None or previous_record==volume_record, 'same volume reconstruction across cold boots')
        sector_records[meta['identity']]=volume_record
        if case in ('normal','errors'):
            require(state['sequence']==8,'normal sequence')
            for i,o in enumerate(state['objects']): require(o['revision']==2 and o['present']==(i%2==0),'normal CRUD/tombstone state')
        if case=='full':
            require(state['sequence']==32,'full sequence')
            for i,o in enumerate(state['objects']): require(o['revision']==(16 if i%2==0 else 0) and o['present']==(i%2==0),'full state')
        if case in ('vm-cut-before-ack','vm-cut-after-flush'):
            cut=1 if case=='vm-cut-before-ack' else 2; crash=r['host_crash']; kill_count+=1
            require(r['raw_qemu_exit']==-9 and crash['kill_requested'] is True and crash['was_running_before_kill'] is True and crash['real_process_returncode']==-9 and crash['termination_code_expected']==-9 and crash['required_live_hold_s']==0.05 and crash['requested_cut']==cut and crash['cut_lines']==[f'M6B:CUT:{cut}'],'actual live child SIGKILL record')
            require('M6B:PASS' not in lines and 'M1:PASS' not in lines,'cut is no normal PASS')
            ordered(lines,['BOOT:EBS_OK','KERNEL:ENTRY','BOOTINFO:OK',f'M6B:CUT:{cut}'])
            require(state['sequence']==cut-1,'cut disk state')
            if cut==1: require(after==before,'pre-publication cut no writes')
            else: require(decoded[0]['object'] in (0,2) and decoded[0]['revision']==1,'post-flush record')
        else:
            require(r['raw_qemu_exit']==33,'normal raw exit')
            ordered(lines,['BOOT:EBS_OK','KERNEL:ENTRY','BOOTINFO:OK',f'M6B:CASE:{case.upper()}:PASS','M6B:SURVIVOR_PROGRESS:PASS','M6B:RESOURCES:PASS','M6B:PASS','M1:PASS'])
            if state is not None:
                require(int(marker(lines,'M6B:SEQUENCE'))==state['sequence'],'guest sequence')
                for i,o in enumerate(state['objects']):
                    for name in ('revision','present','length','digest'):
                        observed=marker(lines,f'M6B:OBJECT{i}:{name.upper()}')
                        want=o[name] if name=='digest' else str(int(o[name]))
                        require(observed==want,'guest object vs actual disk')
                require(int(marker(lines,'M6B:ACKS'))==(32 if case=='full' else 8 if case in ('normal','errors') else 0),'exact acknowledged mutations')
                if case=='errors': require(marker(lines,'M6B:REJECTED_NO_IO')=='34','all 34 rejected requests without I/O')
        entry.update(volume_identity=meta['identity'],disk_sha256=sha(after),sequence=None if state is None else state['sequence'],previous_stage=r['previous_stage'])
        vm.append(entry)
    require(len(vm)==14 and hash_count==121 and len(volumes)==7 and cold_count==6 and kill_count==2,'complete actual coverage counts')
    validate_raw_inventory(raw_hashes, expected_raw_inventory)
    require(load(EVIDENCE/'guest-runs.json')==originals,'captured raw reports equal originals')
    a,b=by_name['normal-a'],by_name['normal-b']
    require(all(a['artifact_sha256'][key]==b['artifact_sha256'][key] for key in ('esp/kernel.elf','esp/EFI/BOOT/BOOTX64.EFI')),'normal same binary')
    require(a['artifact_sha256']['test-disk.before.img']!=b['artifact_sha256']['test-disk.before.img'],'normal different canary images')
    # Gate/capture wrapper consoles are not inputs to this audit. All actual run,
    # host-command, archived-byte and artifact checks above are reexecuted directly.
    report=dict(schema_version=1,audit='A04 independent M6b regression evidence audit',reviewer='Separate AI agent; not a human audit or formal verification',
     recorded_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),status='passed_actual_evidence_audit',checkpoint_commit=COMMIT,
     source_fingerprint_version=3,source_sha256=FROZEN,suite_sha256=sha(suite_path.read_bytes()),
     audit_script_sha256=sha(pathlib.Path(__file__).read_bytes()),audit_script_runtime_path=str(pathlib.Path(__file__)),
     method=['Independent standard-library-only Python audit; no project journal/parser/capture imports.',
     'Reconstructed source fingerprint from exact git archive, and checked root host-test hashes.',
     'Checked exact coverage, original raw bytes, all required artifacts, manifests, capture hashes and host commands.',
     'Decoded actual journal records independently: headers, FNV, sequence, revisions, DELETE predecessor, zero padding, fixture payloads.',
     'Reconstructed all seeded volumes; checked canaries, current files, six cold recovery links and two recorded live-child SIGKILL cuts.',
     'Checked final ordinary M1 raw output, copied build binding, exact QEMU argv and the common M6b executable hash.',
     'Required the pinned 164-file raw inventory, including M1 writable firmware state, with exact paths and SHA-256 values.'],
     observed=dict(groups=18,vm_invocations=14,m6b_vm_invocations=13,m1_vm_invocations=1,artifact_hashes=121,volumes=7,
     cold_recoveries=6,actual_sigkill_records=2,rust_host_tests=rust_count,python_harness_tests=python_count),
     host_commands=host,vm_evidence=vm,source_input_count=len(source_inputs),raw_file_count=len(raw_hashes),
     archive=str(EVIDENCE.relative_to(ROOT)),recorded_repository_root=str(RECORDED_ROOT),python_version=sys.version,
     durable_journal_sector_count=sum(len(v['journal_sector_hex']) for v in sector_records.values()),
     limits=['This audit executes no new guests and rechecks the recorded actual completed gate.',
     'Only fixed synthetic QEMU/TCG fixtures are covered; no physical power-failure or production-storage claim.',
     'FNV is a diagnostic checksum, not encryption/authentication; native NAS and I/O-fault Unknown injection are outside this gate.',
     'The audit reads recorded execution evidence; it cannot independently witness past child processes or defend against a host fabricating all inputs.',
     'M6b host logs are compared against captured bytes; this historical gate does not itself preserve gate-time host-log SHA hashes.',
     'M1 did not record a gate-time QEMU executable hash; this audit binds its exact executable path to the M6b hash and current binary bytes.',
     'No human/formal audit or protection from a malicious host fabricating all evidence is claimed.'])
    for suffix, data in (
        ('_sectors.json',dict(schema_version=1,format='M6b synthetic seed plus nonempty journal sectors',
                             checkpoint_commit=COMMIT,source_sha256=FROZEN,volumes=sector_records)),
        ('_source_hashes.json',source_inputs),
        ('_raw_hashes.json',raw_hashes)):
        target=pathlib.Path(PREFIX+suffix)
        target.write_text(json.dumps(data,indent=2)+'\n')
    report['supporting_files']={suffix:sha(pathlib.Path(PREFIX+suffix).read_bytes())
        for suffix in ('_sectors.json','_source_hashes.json','_raw_hashes.json')}
    DEST.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(status=report['status'],observed=report['observed'],report=str(DEST),sha256=sha(DEST.read_bytes()))))


if __name__ == "__main__":
    main()
