import json,sys,subprocess,hashlib,shutil,time
from pathlib import Path
sys.path.insert(0,str(Path.cwd()/'tools'))
from common import ROOT,BUILD,source_hash,save_json,sha256
from build import build
from run_graphics import run_graphics
expected_commit='3e41bbf93822f7044c6020259f4a752925e1966d'
expected_source='307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60'
assert subprocess.check_output(['git','rev-parse','HEAD']).decode().strip()==expected_commit
assert source_hash()==expected_source
out=ROOT/'docs/verification/parallel-final-g0';out.mkdir(exist_ok=False)
record={'schema_version':1,'source_commit':expected_commit,'source_sha256':expected_source,'source_fingerprint_version':3,'passed':False,'scope':'Actual isolated native G0 VM plus deliberately wrong ROM hash rejected before VM launch; no native document UI claim.'}
save_json(out/'acceptance.json',record)
manifest=build(kernel_feature='graphics-g0')
rom=Path('/tmp/nieuw-os-par-tools/qemu/usr/share/seabios/vgabios-stdvga.bin');digest=sha256(rom)
try:
 run_graphics(qmp_transport='pipe',vga_rom=rom,vga_rom_sha256='0'*64)
except ValueError as exc:
 if 'explicitly pinned SHA256' not in str(exc):raise
 record['wrong_rom_pin']={'rejected':True,'error':str(exc),'expected_sha256':'0'*64,'actual_sha256':digest,'vm_started':False}
else:raise AssertionError('Wrong ROM hash accepted')
result=run_graphics(qmp_transport='pipe',vga_rom=rom,vga_rom_sha256=digest)
assert result['passed'] and result['raw_qemu_exit']==33
assert source_hash()==expected_source
assert subprocess.check_output(['git','rev-parse','HEAD']).decode().strip()==expected_commit
run=Path(result['run_directory'])
for name in ('result.json','stdout.txt','stderr.txt','serial.txt','build-manifest.json'):
 p=run/name
 if p.exists():shutil.copy2(p,out/name)
frames={p.name:sha256(p) for p in run.glob('*.ppm')}
record.update(passed=True,raw_qemu_exit=result['raw_qemu_exit'],run_directory=str(run),frame_sha256=frames,rom_sha256=digest,source_unchanged=True,head_unchanged=True,build_manifest_sha256=sha256(run/'build-manifest.json'))
save_json(out/'acceptance.json',record)
print(json.dumps(record,indent=2))
