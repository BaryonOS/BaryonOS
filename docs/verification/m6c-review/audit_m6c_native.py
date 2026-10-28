#!/usr/bin/env python3
"""Read-only independent audit; imports no Nieuw OS parser, harness or source helper."""
import collections, hashlib, json, random, re, struct, subprocess, sys, unicodedata
from pathlib import Path
ROOT=Path('/workspace/scratch/244976efa240/nieuw-os')
OUT=Path('/workspace/scratch/244976efa240/review')
SOURCE='4468b3c18bd7fa57bf3631d634d52ee4c5fe9a76da9f125714d6db95e4407345'
COMMIT='bc4cd104e86e2c047b727ae96e07a03fbc810c3c'
BASE='bb22fdbe15919edc23baa158a46236195ce314b2'
SIZE=65536;LOW=4*512;HIGH=36*512

def sha(data):return hashlib.sha256(data).hexdigest()
def filehash(path):return sha(path.read_bytes())
def load(path):return json.loads(path.read_text())
def require(ok,message):
 if not ok:raise AssertionError(message)
def fnv(data):
 x=0xcbf29ce484222325
 for b in data:x=((x^b)*0x100000001b3)&((1<<64)-1)
 return x

def name_ok(data):
 try:s=data.decode('utf-8')
 except UnicodeDecodeError:return False
 return 1<=len(data)<=32 and s not in ('.','..') and all(unicodedata.category(c)!='Cc' and c not in '/\\' and not 0x202a<=ord(c)<=0x202e and not 0x2066<=ord(c)<=0x2069 for c in s)

def newobj():return {'revision':0,'present':False,'name':b'','payload':b''}
def decode_image(data):
 require(len(data)==SIZE,'wrong image length')
 objects=[newobj() for _ in range(4)];records=[];ended=False
 for slot in range(32):
  raw=data[LOW+512*slot:LOW+512*(slot+1)]
  if not any(raw):ended=True;continue
  require(not ended,'record after empty slot')
  require(raw[:8]==b'NOSM6C02','unexpected magic/legacy')
  version,header,flags=struct.unpack_from('<HHI',raw,8)
  seq,request_id,oid,rev=struct.unpack_from('<QQQQ',raw,16)
  op,length,nlen=struct.unpack_from('<HHH',raw,48)
  require((version,header,flags)==(2,64,0),'noncanonical version/header/flags')
  require(seq==slot+1 and request_id>0 and oid<4 and 1<=rev<=seq,'record index/identity/revision')
  require(op in (2,3,4) and length<=128 and nlen<=32,'record operation/length')
  require(not any(raw[54:64]+raw[64+length:192]+raw[192+nlen:504]),'record padding')
  require(struct.unpack_from('<Q',raw,504)[0]==fnv(raw[:504]+bytes(8)),'checksum mismatch')
  payload=raw[64:64+length];name=raw[192:192+nlen];old=objects[oid]
  require(rev==old['revision']+1,'object revision is not predecessor+1')
  if op==3:require(old['present'] and not name and not payload,'invalid tombstone')
  else:
   require(name_ok(name),'invalid name')
   if op==4:require(old['present'] and payload==old['payload'],'rename changes content or has no predecessor')
  objects[oid]={'revision':rev,'present':op!=3,'name':name,'payload':payload}
  records.append({'slot':slot,'sequence':seq,'request_id':request_id,'object':oid,'revision':rev,'operation':op,'name':name,'payload':payload})
 return {'sequence':len(records),'objects':objects},records

def value(oid,rev):return bytes((49+29*oid+7*rev+3*i)&255 for i in range(128))
def label(oid,rev):return (['Café A','Werk A','Café B','Werk B'] if rev==1 else ['Café hernoemd A','Werk hernoemd A','Café hernoemd B','Werk hernoemd B'])[oid].encode()
def object_summary(o):return {'revision':o['revision'],'present':o['present'],'name':o['name'].decode(),'name_length':len(o['name']),'name_digest':f"{fnv(o['name']):016x}",'length':len(o['payload']),'digest':f"{fnv(o['payload']):016x}",'payload_sha256':sha(o['payload'])}

def reconstruct_initial(seed,variant):
 data=bytearray(random.Random(seed).randbytes(SIZE));data[:1024]=bytes(1024);label_bytes=b'NOS-M6C-TEST-VOLUME\0';data[:len(label_bytes)]=label_bytes;data[LOW:HIGH]=bytes(HIGH-LOW)
 if variant=='corrupt':data[LOW]=128
 if variant=='legacy':
  r=bytearray(512);r[:8]=b'NOSM6B01';struct.pack_into('<HHIQQQQHH',r,8,1,64,0,1,1,0,1,2,128);r[64:192]=value(0,1);struct.pack_into('<Q',r,504,fnv(r));data[LOW:LOW+512]=r
 require(variant in ('normal','legacy','corrupt'),'unexpected volume variant')
 return bytes(data)

def source_fingerprint():
 h=hashlib.sha256(b'nieuw-os source inputs v3\n');files=[]
 for folder in ['boot','kernel','crates','drivers','services','userspace','tools','.cargo']:
  base=ROOT/folder
  for p in sorted(base.rglob('*')) if base.exists() else []:
   if p.is_file() and not set(p.relative_to(base).parts)&{'__pycache__','verification','target','build','.git'}:
    rel=p.relative_to(ROOT).as_posix();raw=p.read_bytes();h.update(rel.encode());h.update(raw);files.append({'path':rel,'sha256':sha(raw)})
 for name in ['Cargo.toml','Cargo.lock','rust-toolchain.toml']:
  p=ROOT/name
  if p.exists():raw=p.read_bytes();h.update(name.encode());h.update(raw);files.append({'path':name,'sha256':sha(raw)})
 return h.hexdigest(),files

def stage_state(serial,state,case):
 found={}
 for line in serial.splitlines():
  if re.match(r'^M6C:(?:SEQUENCE|OBJECT|ACKS|REJECTED_NO_IO|CATALOG_REPLIES|DOMAIN)',line):
   key,_,v=line.rpartition(':');require(key not in found,'duplicate state marker');found[key]=v
 if case in ('corrupt','legacy','vm-cut-before-rename','vm-cut-after-rename'):
  require(not found,'failure/cut has live state markers');return
 expected={'M6C:SEQUENCE':str(state['sequence'])}
 for i,o in enumerate(state['objects']):
  sm=object_summary(o)
  for field,val in [('REVISION',o['revision']),('PRESENT',int(o['present'])),('LENGTH',sm['length']),('NAME_LENGTH',sm['name_length']),('NAME_DIGEST',sm['name_digest']),('DIGEST',sm['digest'])]:expected[f'M6C:OBJECT{i}:{field}']=str(val)
 expected['M6C:ACKS']=str(32 if case=='full' else 10 if case in ('normal','errors') else 0)
 expected['M6C:CATALOG_REPLIES']=str(4 if case=='full' else 10 if case in ('normal','errors') else 2)
 expected['M6C:REJECTED_NO_IO']=str({'normal':6,'errors':80,'full':8}.get(case,sum(not o['present'] for o in state['objects'])))
 for d in range(2):expected[f'M6C:DOMAIN{d}:REVISION']=str(sum(o['revision'] for o in state['objects'][2*d:2*d+2]))
 require(found==expected,'serial state/counters differ from independently decoded image')

def audit():
 suite=load(ROOT/'build/m6c-test-results.json');archive=ROOT/'docs/verification/m6c';capture=load(archive/'capture.json');archruns=load(archive/'guest-runs.json')
 require(suite['passed'] and suite['status']=='completed' and len(suite['results'])==19,'suite not complete19')
 require(suite==load(archive/'suite.json'),'archived suite mismatch')
 require(suite['source_sha256']==SOURCE and suite['source_fingerprint_version']==3,'wrong suite source')
 archive_checks=[]
 for name,expected in capture['files'].items():
  actual=filehash(archive/name);require(actual==expected,'archive hash mismatch '+name);archive_checks.append({'file':name,'sha256':actual})
 require(capture['passed'] and capture['groups']==19 and capture['vm_invocations']==15 and capture['source_sha256']==SOURCE,'capture metadata')
 for rel,expected in suite['host_test_sha256'].items():require(filehash(ROOT/rel)==expected,'host test source differs '+rel)
 actual_source,source_files=source_fingerprint();require(actual_source==SOURCE,'current source fingerprint differs')
 changed=subprocess.run(['git','diff','--name-only',COMMIT,'--','boot','kernel','crates','drivers','services','userspace','tools','.cargo','Cargo.toml','Cargo.lock','rust-toolchain.toml'],cwd=ROOT,capture_output=True,text=True,check=True).stdout.strip()
 require(not changed,'source diff since audited commit: '+changed)
 # Bind each hashed source path to actual committed bytes, not merely Git diff output.
 for item in source_files:
  raw=subprocess.run(['git','show',COMMIT+':'+item['path']],cwd=ROOT,capture_output=True,check=True).stdout
  require(sha(raw)==item['sha256'],'uncommitted source '+item['path'])
 summaries=[];volume_data={};stage_data={};artifacts_count=0;all_file_count=0;base_counts=collections.Counter();pids=[];crashes=[];cold=[];artifact_inventory=[]
 roots=[x for x in suite['results'] if Path(x['evidence']).is_dir()]
 require(len(roots)==15 and len(archruns)==15,'VM record count')
 require(len({x['evidence'] for x in roots})==15,'duplicate VM directories')
 for group,archived in zip(roots,archruns):
  d=Path(group['evidence']);r=load(d/'result.json');case=r.get('case','m1');base_counts[r['base_commit']]+=1
  require(r==archived,'guest archive differs '+group['name'])
  require(r['passed'] and not r['timed_out'] and r['source_sha256']==SOURCE and r['source_fingerprint_version']==3,'bad run/source')
  require(r['base_commit'] in (BASE,COMMIT),'unexpected base commit')
  serial_bytes=(d/'serial.txt').read_bytes();stderr_bytes=(d/'stderr.txt').read_bytes();serial=serial_bytes.decode('utf-8');stderr=stderr_bytes.decode('utf-8');lines=serial.splitlines()
  require(serial==r['stdout'] and stderr==r['stderr'] and not stderr,'raw output mismatch/nonempty stderr')
  require(not any(re.match(r'^M[1-9][A-Z]?:FAIL',line) for line in lines),'guest failure marker')
  require(all(lines.count(m)==1 for m in ('BOOT:EBS_OK','KERNEL:ENTRY','BOOTINFO:OK')),'boot marker count')
  for rel,expected in r['artifact_sha256'].items():
   target=d/rel;require(target.is_file() and target.resolve().is_relative_to(d.resolve()),'artifact missing/escaped')
   actual=filehash(target);require(actual==expected,'artifact hash '+str(target));artifacts_count+=1
  inventory={p.relative_to(d).as_posix():filehash(p) for p in sorted(d.rglob('*')) if p.is_file()};all_file_count+=len(inventory);artifact_inventory.append({'run':group['name'],'files':inventory})
  cmd=r['command']
  for flag,value_ in [('-machine','pc-q35-10.0'),('-accel','tcg,thread=single'),('-cpu','qemu64'),('-smp','1'),('-m','256M'),('-nic','none')]:require(cmd[cmd.index(flag)+1]==value_,'QEMU profile '+flag)
  require('-nodefaults' in cmd and '-snapshot' not in cmd,'unsafe/nonpersistent QEMU profile')
  if case=='m1':
   require(r['raw_qemu_exit']==33 and lines.count('M1:PASS')==1 and not any(re.match(r'^M(?:[2-9]|6C):',x) for x in lines),'M1 phase/exit')
   require(r['artifact_binding_passed'] and all(r['artifact_sha256'][k]==v for k,v in r['expected_artifact_sha256'].items()),'M1 artifact binding')
   summaries.append({'name':group['name'],'case':case,'base_commit':r['base_commit'],'raw_exit':33,'artifacts':len(r['artifact_sha256']),'serial_sha256':sha(serial_bytes),'passed':True});continue
  require(r['observed_source_sha256']==SOURCE and r['source_unchanged'],'M6c source changed')
  require(filehash(Path(cmd[0]))==r['qemu_sha256'],'QEMU binary hash')
  require(sha(serial_bytes)==r['serial_sha256'] and sha(stderr_bytes)==r['stderr_sha256'],'recorded raw output hash')
  build=load(d/'build-manifest.json');expected_feature='milestone-m6c' if case=='normal' else 'test-m6c-'+case
  require(build['source_sha256']==SOURCE and build['kernel_feature']==expected_feature and build['loader_feature'] is None,'build feature/source')
  require(build['kernel']['sha256']==filehash(d/'esp/kernel.elf') and build['loader']['sha256']==filehash(d/'esp/EFI/BOOT/BOOTX64.EFI'),'build binary identity')
  crash=r['host_crash'];pids.append(crash['process_id']);require(crash['process_id']>0,'bad PID')
  before=(d/'test-disk.before.img').read_bytes();after=(d/'test-disk.after.img').read_bytes();v=load(d/'volume-manifest.json');initial=reconstruct_initial(v['seed'],v['variant'])
  require(v['bytes']==SIZE and v['bdf']=='00:05.0' and sha(initial)==v['initial_sha256'],'synthetic volume provenance')
  require(len(before)==len(after)==SIZE and before[:LOW]==initial[:LOW] and before[HIGH:]==initial[HIGH:] and after[:LOW]==initial[:LOW] and after[HIGH:]==initial[HIGH:],'outside scope canaries changed')
  path=Path(r['volume_path']);require(path.is_relative_to(ROOT/'build/run') and path.name=='test-disk.img' and path.read_bytes()==after,'actual final volume differs')
  require(any('id=m6c-test' in x and 'cache=writeback' in x and 'file='+str(path.relative_to(ROOT)) in x for x in cmd),'actual QEMU volume binding')
  require(load(path.parent/'volume.json')==v,'volume manifest identity differs')
  if case!='recover':require(before==initial,'mutating/rejection stage not fresh')
  if case in ('recover','legacy','corrupt'):require(before==after,'read-only stage changes bytes')
  if case in ('legacy','corrupt'):
   try:decode_image(after)
   except AssertionError:state=None;records=[]
   else:raise AssertionError('rejection volume decoded as M6c')
   require('M6C:READY' not in lines,'rejection reached READY')
  else:
   state,records=decode_image(after);require(lines.count('M6C:READY')==1,'missing/duplicate READY')
   for e in records:
    if e['operation']!=3:require(e['name']==label(e['object'],e['revision']),'actual name differs fixture')
    if e['operation']==2:require(e['payload']==value(e['object'],e['revision']),'actual content differs fixture')
   if case in ('normal','errors'):
    require(state['sequence']==10,'normal final sequence')
    for i,o in enumerate(state['objects']):
     require(o['revision']==(3 if i%2==0 else 2) and o['present']==(i%2==0),'normal object state')
     require([e['operation'] for e in records if e['object']==i]==([2,4,2] if i%2==0 else [2,3]),'normal operation history')
   if case=='full':
    require(state['sequence']==32 and all(e['operation']==2 for e in records),'full sequence/operations')
    require(all(o['revision']==(16 if i%2==0 else 0) and o['present']==(i%2==0) for i,o in enumerate(state['objects'])),'full state')
  stage_state(serial,state,case)
  controlled=[l for l in lines if l in ('BOOT:EBS_OK','KERNEL:ENTRY','BOOTINFO:OK','M6C:READY','M6C:SURVIVOR_PROGRESS:PASS','M6C:RESOURCES:PASS','M6C:PASS','M1:PASS') or l.startswith(('M6C:CASE:','M6C:CUT:'))]
  if case.startswith('vm-cut-'):
   point=1 if case=='vm-cut-before-rename' else 2
   require(controlled==['BOOT:EBS_OK','KERNEL:ENTRY','BOOTINFO:OK','M6C:READY',f'M6C:CUT:{point}'],'crash serial order/false PASS')
   require(r['raw_qemu_exit']==-9 and crash['real_process_returncode']==-9 and crash['termination_code_expected']==-9 and crash['kill_requested'] and crash['was_running_before_kill'] and crash['required_live_hold_s']>=.05 and crash['cut_lines']==[f'M6C:CUT:{point}'],'live external-kill evidence')
   require(state['sequence']==point and state['objects'][0]['revision']==point and state['objects'][0]['payload']==value(0,1) and state['objects'][0]['name']==label(0,point) and all(o==newobj() for o in state['objects'][1:]),'crash image outcome')
   crashes.append({'run':group['name'],'process_id':crash['process_id'],'raw_exit':-9,'sequence':point,'name':state['objects'][0]['name'].decode(),'payload_sha256':sha(state['objects'][0]['payload'])})
  else:
   expected=['BOOT:EBS_OK','KERNEL:ENTRY','BOOTINFO:OK']+([] if case in ('corrupt','legacy') else ['M6C:READY'])+[f'M6C:CASE:{case.upper()}:PASS','M6C:SURVIVOR_PROGRESS:PASS','M6C:RESOURCES:PASS','M6C:PASS','M1:PASS']
   require(controlled==expected and r['raw_qemu_exit']==33 and crash['real_process_returncode']==33 and not crash['kill_requested'],'normal serial/exit')
  if case=='recover':
   prev=Path(r['previous_stage']);require(prev in stage_data,'cold run not linked to earlier audited stage');p=stage_data[prev]
   require(p['volume']==r['volume_path'] and p['identity']==v['identity'] and p['after']==before==after and p['pid']!=crash['process_id'],'cold identity/byte/process mismatch')
   cold.append({'run':group['name'],'previous_run':p['name'],'same_identity':v['identity'],'same_before_after_sha256':sha(after),'distinct_process_ids':[p['pid'],crash['process_id']]})
  stage_data[d]={'name':group['name'],'after':after,'volume':r['volume_path'],'identity':v['identity'],'pid':crash['process_id']}
  entry={'identity':v['identity'],'seed':v['seed'],'variant':v['variant'],'initial_sha256':sha(initial),'final_sha256':sha(after),'initial_nonempty_journal':[{'slot':slot,'hex':initial[LOW+slot*512:LOW+(slot+1)*512].hex()} for slot in range(32) if any(initial[LOW+slot*512:LOW+(slot+1)*512])],'final_nonempty_journal':[{'slot':slot,'hex':after[LOW+slot*512:LOW+(slot+1)*512].hex()} for slot in range(32) if any(after[LOW+slot*512:LOW+(slot+1)*512])],'state':None if state is None else {'sequence':state['sequence'],'objects':[object_summary(o) for o in state['objects']]}}
  if v['identity'] in volume_data:require(volume_data[v['identity']]==entry,'cold volume export differs')
  volume_data[v['identity']]=entry
  summaries.append({'name':group['name'],'case':case,'base_commit':r['base_commit'],'raw_exit':r['raw_qemu_exit'],'process_id':crash['process_id'],'artifacts':len(r['artifact_sha256']),'serial_sha256':sha(serial_bytes),'volume_identity':v['identity'],'image_after_sha256':sha(after),'sequence':None if state is None else state['sequence'],'passed':True})
 require(len(pids)==len(set(pids))==14 and len(cold)==6 and len(crashes)==2 and len(volume_data)==8,'final VM/crash/cold/volume counts')
 a,b=[x for x in roots if x['name'] in ('normal-a','normal-b')]
 ra,rb=[load(Path(x['evidence'])/'result.json') for x in (a,b)]
 require(all(ra['artifact_sha256'][k]==rb['artifact_sha256'][k] for k in ['esp/kernel.elf','esp/EFI/BOOT/BOOTX64.EFI']),'normal runs binary mismatch')
 require(ra['disk_oracle']['identity']!=rb['disk_oracle']['identity'] and ra['disk_oracle']['seed']!=rb['disk_oracle']['seed'],'canary images not independent')
 for name in ('m6c-preflight','m6c-host-tests','m6c-harness-tests'):require(load(archive/(name+'.json'))['returncode']==0,'host gate exit')
 journal_export={'schema':1,'scope':'actual final disposable volumes, independently decoded; no binary files needed for reconstruction','image_bytes':SIZE,'journal_first_sector':4,'journal_sector_count':32,'sector_bytes':512,'initial_algorithm':'random.Random(seed).randbytes(65536); zero bytes0:1024; write b"NOS-M6C-TEST-VOLUME\\0" at0; zero bytes2048:18432; overlay initial_nonempty_journal at(4+slot)*512','final_algorithm':'take reconstructed initial image; zero entire journal; overlay final_nonempty_journal at(4+slot)*512; compare final_sha256','python_version':sys.version,'volumes':list(volume_data.values())}
 # Reconstruct every exported image from JSON-equivalent hex and seed, without retaining its original bytes.
 for v in journal_export['volumes']:
  initial=bytearray(random.Random(v['seed']).randbytes(SIZE));initial[:1024]=bytes(1024);lbl=b'NOS-M6C-TEST-VOLUME\0';initial[:len(lbl)]=lbl;initial[LOW:HIGH]=bytes(HIGH-LOW)
  for r in v['initial_nonempty_journal']:initial[LOW+r['slot']*512:LOW+(r['slot']+1)*512]=bytes.fromhex(r['hex'])
  require(sha(initial)==v['initial_sha256'],'export initial reconstruct mismatch')
  initial[LOW:HIGH]=bytes(HIGH-LOW)
  for r in v['final_nonempty_journal']:initial[LOW+r['slot']*512:LOW+(r['slot']+1)*512]=bytes.fromhex(r['hex'])
  require(sha(initial)==v['final_sha256'],'export final reconstruct mismatch')
 (OUT/'m6c-actual-journal-records.json').write_text(json.dumps(journal_export,ensure_ascii=False,indent=2)+'\n')
 (OUT/'m6c-all-artifact-hashes.json').write_text(json.dumps(artifact_inventory,indent=2)+'\n')
 (OUT/'m6c-audited-source-hashes.json').write_text(json.dumps(source_files,indent=2)+'\n')
 report={'task':'V07 independent final native audit','passed':True,'source_commit':COMMIT,'source_fingerprint_version':3,'source_sha256':SOURCE,'source_files_checked_against_commit':len(source_files),'groups':19,'vm_invocations':15,'m6c_invocations':14,'m1_invocations':1,'completed_exit33':13,'external_kill_exit_minus9':2,'cold_recoveries':6,'unique_volumes':8,'recorded_artifact_hashes_verified':artifacts_count,'all_actual_run_files_hashed':all_file_count,'archive_hashes_verified':len(archive_checks),'valid_v2_records_across_unique_volumes':sum(v['state']['sequence'] for v in volume_data.values() if v['state']),'nonempty_records_exported':sum(len(v['final_nonempty_journal']) for v in volume_data.values()),'base_commit_metadata_counts':dict(base_counts),'provenance_note':'Six early normal/error runs retain bb22fdbe as then-current Git HEAD; every run and build binds identical independently recomputed source fingerprint, whose every file is byte-equal to commit bc4cd104. Remaining nine records name bc4cd104. No missing-code inference.','runs':summaries,'cold_pairs':cold,'external_cuts':crashes,'evidence_files':{'journal_records':'m6c-actual-journal-records.json','journal_records_sha256':filehash(OUT/'m6c-actual-journal-records.json'),'artifact_inventory':'m6c-all-artifact-hashes.json','artifact_inventory_sha256':filehash(OUT/'m6c-all-artifact-hashes.json'),'source_hashes':'m6c-audited-source-hashes.json','source_hashes_sha256':filehash(OUT/'m6c-audited-source-hashes.json')},'suite_sha256':filehash(ROOT/'build/m6c-test-results.json'),'capture_sha256':filehash(archive/'capture.json'),'audit_script_sha256':filehash(Path(__file__)),'findings':[],'limitations':['Read-only reinspection of real existing run evidence; no new VM execution.','QEMU process kills are not physical power-loss tests.','FNV is diagnostic and not cryptographic authentication.','AI audit is not a human audit or formal proof.','M6b regression running separately is outside this report.']}
 (OUT/'m6c-final-native-audit.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
 print(json.dumps({k:v for k,v in report.items() if k not in ('runs','cold_pairs','evidence_files')},ensure_ascii=False,indent=2))

if __name__=='__main__':
 try:audit()
 except Exception as e:
  (OUT/'m6c-final-native-audit.json').write_text(json.dumps({'task':'V07','passed':False,'error_type':type(e).__name__,'error':str(e)},indent=2)+'\n');raise
