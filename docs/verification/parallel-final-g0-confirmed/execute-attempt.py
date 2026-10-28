from pathlib import Path
import sys,subprocess,json,shutil,hashlib,time
from PIL import Image
sys.path.insert(0,str(Path.cwd()/'tools'))
from common import ROOT,BUILD,source_hash,save_json,sha256
from run_graphics import run_graphics,ppm,frame_checks
expected='307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60';head='3e41bbf93822f7044c6020259f4a752925e1966d'
assert source_hash()==expected
assert subprocess.check_output(['git','rev-parse','HEAD']).decode().strip()==head
out=ROOT/'docs/verification/parallel-final-g0-confirmed';out.mkdir(exist_ok=False)
rom=Path('/tmp/nieuw-os-par-tools/qemu/usr/share/seabios/vgabios-stdvga.bin');pin=sha256(rom);records=[]
for transport in ('pipe','unix'):
 r=run_graphics(qmp_transport=transport,run_root=Path('/tmp/nieuw-os-g0-confirmed-'+transport),vga_rom=rom,vga_rom_sha256=pin)
 assert r['passed'] and r['raw_qemu_exit']==33 and r['error'] is None
 folder=out/transport;folder.mkdir();run=Path(r['run_directory'])
 for rel,digest in r['artifacts'].items():assert sha256(run/rel)==digest,(rel,transport)
 for name in ('result.json','serial.txt','stderr.txt','qmp-trace.json','build-manifest.json'):shutil.copy2(run/name,folder/name)
 image_records=[]
 for name in ('initial','a','ab','after-backspace','typed'):
  original=(run/(name+'.ppm')).read_bytes();width,height,pixels=ppm(original)
  png=folder/(name+'.png');Image.frombytes('RGB',(width,height),pixels).save(png)
  restored=original[:len(original)-len(pixels)]+Image.open(png).convert('RGB').tobytes();assert restored==original
  image_records.append(dict(name=name,width=width,height=height,original_header_hex=original[:len(original)-len(pixels)].hex(),ppm_sha256=hashlib.sha256(original).hexdigest(),png_sha256=sha256(png)))
 save_json(folder/'image-roundtrip.json',dict(scope='Lossless PNG encoding of the real QEMU PPM pixels; original PPM bytes reconstruct exactly from header plus decoded RGB pixels.',images=image_records))
 records.append(dict(transport=transport,run_directory=str(run),raw_exit=r['raw_qemu_exit'],artifact_sha256=r['artifacts'],passed=True,source_sha256=r['source_sha256']))
# Check both complete artifact sets again after both VMs have ended.
for r in records:
 for rel,digest in r['artifact_sha256'].items():assert sha256(Path(r['run_directory'])/rel)==digest,(rel,r['transport'])
assert source_hash()==expected
save_json(out/'acceptance.json',dict(source_commit=head,source_sha256=expected,source_fingerprint_version=3,passed=True,scope='Two new actual native G0 runs with separate private /tmp run directories; the earlier mutated run is preserved and is not accepted as final evidence.',native_vm_count=2,post_run_rehash_passed=True,results=records))
print('Confirmed both native G0 transports and complete post-run rehash',flush=True)
