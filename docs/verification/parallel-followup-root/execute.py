import contextlib, hashlib, json, os, pathlib, shutil, subprocess, sys, time
from datetime import datetime, timezone

ROOT = pathlib.Path('/workspace/scratch/debaa3b6a81c/nieuw-os')
sys.path.insert(0, str(ROOT/'tools'))
from common import BUILD, source_hash, sha256
from build import build
from run_m6b import run_m6b
from capture_m6b import check_run

freeze = json.loads(pathlib.Path('/tmp/nieuw-os-followup-freeze.json').read_text())
OUT = ROOT/'docs/verification/parallel-followup-root'
OUT.mkdir(parents=True, exist_ok=False)
shutil.copyfile(__file__, OUT/'execute.py')
records = []
def bound():
    head = subprocess.check_output(['git','rev-parse','HEAD'], cwd=ROOT).decode().strip()
    actual = source_hash()
    assert head == freeze['commit'] and actual == freeze['source_sha256'], (head, actual)
    return {'commit':head, 'source_sha256':actual}

def save():
    (OUT/'execution.json').write_text(json.dumps({'source':freeze,'steps':records},indent=2)+'\n')

def call(name, command, operation):
    before = bound(); start = time.monotonic()
    record = dict(name=name, command=command, started_utc=datetime.now(timezone.utc).isoformat(), before=before)
    try:
        with (OUT/(name+'.stdout.txt')).open('w') as out, (OUT/(name+'.stderr.txt')).open('w') as err:
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                value = operation(out, err)
        record['exit_code']=0
    except BaseException as error:
        record.update(exit_code=1,error=repr(error)); raise
    finally:
        record.update(duration_s=time.monotonic()-start,after=bound()); records.append(record); save()
        print(name,record['exit_code'],flush=True)
    return value

def subprocess_step(name, command):
    def perform(out,err):
        subprocess.run(command,cwd=ROOT,stdout=out,stderr=err,check=True,timeout=90)
    return call(name,command,perform)

bound()
subprocess_step('host-graphics',['python','-B','-m','unittest','discover','-s','tests','-p','test_graphics*.py'])
subprocess_step('host-preflight',['python','-B','-m','unittest','discover','-s','tests','-p','test_preflight.py'])
subprocess_step('host-build-paths',['python','-B','-m','unittest','discover','-s','tests','-p','test_build_paths.py'])
subprocess_step('host-source-binding',['python','-B','-m','unittest','discover','-s','tests','-p','test_common.py'])
subprocess_step('actual-strict-preflight',['python','-B','tools/preflight.py','--strict'])
call('build-normal',"build(kernel_feature='milestone-m6b', label='followup-root-normal')",lambda out,err:build(kernel_feature='milestone-m6b',label='followup-root-normal'))
normal, volume = call('vm-normal',"run_m6b('normal', label='followup-root-normal')",lambda out,err:run_m6b('normal',label='followup-root-normal'))
assert normal['passed'] and normal['raw_qemu_exit']==33
check_run(pathlib.Path(normal['run_directory']),normal,freeze['source_sha256'])
call('build-recover',"build(kernel_feature='test-m6b-recover', label='followup-root-recover')",lambda out,err:build(kernel_feature='test-m6b-recover',label='followup-root-recover'))
recovered, _ = call('vm-recover',"run_m6b('recover', volume=normal_volume, label='followup-root-cold')",lambda out,err:run_m6b('recover',volume=volume,label='followup-root-cold'))
assert recovered['passed'] and recovered['raw_qemu_exit']==33
hashes=[]
for report in (normal,recovered):
    directory=pathlib.Path(report['run_directory'])
    check_run(directory,report,freeze['source_sha256'])
    assert report['base_commit']==freeze['commit']
    target=OUT/report['case']; target.mkdir()
    for name in ('result.json','build-manifest.json','serial.txt','stderr.txt','volume-manifest.json'):
        shutil.copyfile(directory/name,target/name)
        assert (target/name).read_bytes()==(directory/name).read_bytes()
    for name,expected in report['artifact_sha256'].items():
        actual=sha256(directory/name)
        assert actual==expected, (directory,name)
        hashes.append(dict(path=str(directory/name),sha256=actual))
checks=dict(two_distinct_actual_vms=normal['run_directory']!=recovered['run_directory'],same_volume=normal['volume_path']==recovered['volume_path'],linked_previous=recovered['previous_stage']==normal['run_directory'],same_identity=normal['disk_oracle']['identity']==recovered['disk_oracle']['identity'],exact_saved_image=normal['artifact_sha256']['test-disk.after.img']==recovered['artifact_sha256']['test-disk.before.img'],recovery_read_only=recovered['artifact_sha256']['test-disk.before.img']==recovered['artifact_sha256']['test-disk.after.img'],source_unchanged=bound()=={'commit':freeze['commit'],'source_sha256':freeze['source_sha256']})
assert all(checks.values())
verification=dict(scope='Two actual current-source M6b VMs plus focused host/tooling checks; not a rerun of the full 32-group gate',source=freeze,passed=True,checks=checks,artifact_checks=hashes,raw_exit_codes=[normal['raw_qemu_exit'],recovered['raw_qemu_exit']],run_directories=[normal['run_directory'],recovered['run_directory']])
(OUT/'verification.json').write_text(json.dumps(verification,indent=2)+'\n')
print(json.dumps({'passed':True,'actual_vms':2,'artifact_checks':len(hashes),'source':freeze}))
