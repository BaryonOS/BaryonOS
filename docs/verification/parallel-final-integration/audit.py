from pathlib import Path
import hashlib,json,collections,sys
sys.path.insert(0,'tools');from common import ROOT,sha256,source_hash,save_json
EXPECTED='307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60'
v=ROOT/'docs/verification';out=v/'parallel-final-integration';out.mkdir(exist_ok=True)
reports=[]
for track in ('m6b','sdk-m6c'):
 for r in json.loads((v/('parallel-final-'+track)/'guest-runs.json').read_text()):reports.append((track,r))
for track in ('g0','qsim'):
 base = v/'parallel-final-g0-confirmed/pipe' if track=='g0' else v/('parallel-final-'+track)
 reports.append((track,json.loads((base/'result.json').read_text())))
for p in (v/'parallel-final-m3/runs').rglob('result.json'):reports.append(('m3',json.loads(p.read_text())))
seen=set();checked=0;summary=[];errors=[]
for track,r in reports:
 path=Path(r['run_directory']);label=r.get('label',path.name)
 if str(path) in seen:errors.append('duplicate VM '+str(path))
 seen.add(str(path))
 if r.get('source_sha256')!=EXPECTED:errors.append('wrong source '+label)
 if track=='g0':
  expected_checks={'real_key_changes_visible_pixels','only_input_row_changes','backspace_restores_previous_frame_exactly','nonempty_native_scene','real_exit33','post_ebs_kernel_m2','exact_g0_sequence','no_guest_failure','source_unchanged','normal_end_marker'}
  ok=(r.get('passed') is True and r.get('error') is None and r.get('raw_qemu_exit')==33 and set(r.get('checks',{}))==expected_checks and all(x is True for x in r['checks'].values()) and 0<r.get('duration_s',0)<60)
 else:ok=r.get('passed') is True and r.get('timed_out') is False
 if not ok:errors.append('failed or timeout '+label)
 live=json.loads((path/'result.json').read_text())
 if r!=live:errors.append('modified archive '+label)
 for rel,expected in r.get('artifacts' if track=='g0' else 'artifact_sha256',{}).items():
  artifact=path/rel
  if not artifact.is_file() or sha256(artifact)!=expected:errors.append('artifact '+label+':'+rel)
  checked+=1
 summary.append(dict(track=track,label=label,run_directory=str(path),raw_exit=r['raw_qemu_exit'],source_sha256=r['source_sha256'],artifact_count=len(r.get('artifacts' if track=='g0' else 'artifact_sha256',{}))))
assert len(reports)==80,(len(reports),collections.Counter(t for t,r in reports))
assert source_hash()==EXPECTED
proof=dict(source_commit='3e41bbf93822f7044c6020259f4a752925e1966d',source_sha256=EXPECTED,source_fingerprint_version=3,scope='Direct rehash of actual retained artifacts and byte-equal parsed archived/native reports; no new VM execution.',native_vms=len(reports),unique_vm_directories=len(seen),track_counts=dict(collections.Counter(t for t,r in reports)),raw_exit_counts=dict(collections.Counter(str(r['raw_qemu_exit']) for t,r in reports)),artifact_hash_checks=checked,errors=errors,passed=not errors,results=summary)
save_json(out/'artifact-audit.json',proof)
print(json.dumps({k:x for k,x in proof.items() if k!='results'},indent=2))
assert not errors
