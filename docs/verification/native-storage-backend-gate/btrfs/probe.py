import hashlib, json, os, pathlib, shutil, subprocess, sys, time

BASE = pathlib.Path('/tmp/nieuw-os-storage-btrfs-20261102')
OUT = pathlib.Path('/mnt/c/Users/simon/Documents/Codex/2026-11-02/native-storage-gate/btrfs')
BASE.mkdir(exist_ok=True)
OUT.mkdir(exist_ok=True)
LOGS = OUT / 'logs'
LOGS.mkdir(exist_ok=True)
records = []

def run(name, argv, env=None, timeout=180):
    start=time.time()
    cp=subprocess.run(argv,cwd=BASE,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=timeout)
    (LOGS/(name+'.stdout.txt')).write_bytes(cp.stdout)
    (LOGS/(name+'.stderr.txt')).write_bytes(cp.stderr)
    rec={'name':name,'argv':argv,'cwd':str(BASE),'exit_code':cp.returncode,'elapsed_seconds':round(time.time()-start,3),'stdout':str(LOGS/(name+'.stdout.txt')),'stderr':str(LOGS/(name+'.stderr.txt'))}
    records.append(rec)
    (OUT/'commands.json').write_text(json.dumps(records,indent=2)+'\n')
    print(json.dumps({'name':name,'exit_code':cp.returncode,'stdout_excerpt':cp.stdout.decode(errors='replace')[:4000],'stderr_excerpt':cp.stderr.decode(errors='replace')[:1000]}),flush=True)
    return cp

run('uname',['uname','-a'])
run('os-release',['cat','/etc/os-release'])
run('package-policy',['apt-cache','policy','btrfs-progs'])
run('package-metadata',['apt-cache','show','btrfs-progs'])
run('package-dependencies',['apt-cache','depends','btrfs-progs'])
# Use the recorded package identity, even when another .deb already exists.
pin=json.loads(pathlib.Path(__file__).with_name('toolchain.json').read_text())
package_name=pathlib.Path(pin['deb']).name
deb=BASE/package_name
if not deb.exists():
    version=package_name.removeprefix('btrfs-progs_').removesuffix('_amd64.deb')
    cp=run('download',['apt','download','btrfs-progs='+version])
    if cp.returncode: sys.exit(cp.returncode)
if not deb.is_file() or hashlib.sha256(deb.read_bytes()).hexdigest()!=pin['deb_sha256']:
    raise RuntimeError('Pinned btrfs-progs package missing or SHA256 mismatch')
run('package-fields',['dpkg-deb','--field',str(deb)])
root=BASE/'toolroot'
if run('extract',['dpkg-deb','--extract',str(deb),str(root)]).returncode:
    raise RuntimeError('Package extraction failed')
btrfs=next(p for p in root.rglob('btrfs') if p.is_file())
mkfs=next(p for p in root.rglob('mkfs.btrfs') if p.is_file())
run('ldd-btrfs',['ldd',str(btrfs)])
run('ldd-mkfs',['ldd',str(mkfs)])
run('btrfs-version',[str(btrfs),'version'])
run('mkfs-version',[str(mkfs),'--version'])
run('mkfs-help',[str(mkfs),'--help'])
run('check-help',[str(btrfs),'check','--help'])
run('restore-help',[str(btrfs),'restore','--help'])
run('dump-tree-help',[str(btrfs),'inspect-internal','dump-tree','--help'])
manifest={'btrfs':str(btrfs),'mkfs':str(mkfs),'deb':str(deb),'deb_sha256':hashlib.sha256(deb.read_bytes()).hexdigest()}
(OUT/'toolchain.json').write_text(json.dumps(manifest,indent=2)+'\n')
