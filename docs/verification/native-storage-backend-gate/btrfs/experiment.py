import hashlib, json, os, pathlib, re, shutil, subprocess, sys, time

BASE=pathlib.Path(os.environ.get('BTRFS_BASE', '/tmp/nieuw-os-storage-btrfs-20261102'))
OUT=pathlib.Path(os.environ.get('BTRFS_OUT', '/mnt/c/Users/simon/Documents/Codex/2026-11-02/native-storage-gate/btrfs'))
LOGS=OUT/'logs'
tools=json.loads((OUT/'toolchain.json').read_text())
records=json.loads((OUT/'commands.json').read_text())
btrfs=tools['btrfs']; mkfs=tools['mkfs']

def run(name,argv,timeout=180):
    start=time.time()
    cp=subprocess.run(argv,cwd=BASE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=timeout)
    (LOGS/(name+'.stdout.txt')).write_bytes(cp.stdout)
    (LOGS/(name+'.stderr.txt')).write_bytes(cp.stderr)
    records.append({'name':name,'argv':argv,'cwd':str(BASE),'exit_code':cp.returncode,'elapsed_seconds':round(time.time()-start,3),'stdout':str(LOGS/(name+'.stdout.txt')),'stderr':str(LOGS/(name+'.stderr.txt'))})
    (OUT/'commands.json').write_text(json.dumps(records,indent=2)+'\n')
    print(json.dumps({'name':name,'exit_code':cp.returncode,'stdout_excerpt':cp.stdout.decode(errors='replace')[:2000],'stderr_excerpt':cp.stderr.decode(errors='replace')[:1800]}),flush=True)
    return cp

def digest(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
    return h.hexdigest()

def corpus_manifest(root):
    result={}
    for p in sorted(root.rglob('*')):
        if p.is_symlink(): result[str(p.relative_to(root))]={'type':'symlink','target':os.readlink(p)}
        elif p.is_file(): result[str(p.relative_to(root))]={'type':'file','bytes':p.stat().st_size,'sha256':digest(p)}
    return result

seed=BASE/'seed'
if seed.exists(): raise RuntimeError('Refusing to overwrite an existing seed experiment')
seed.mkdir(); (seed/'documents').mkdir(); (seed/'small').mkdir()
(seed/'documents'/'payload.bin').write_bytes(hashlib.shake_256(b'nieuw-os-btrfs-payload-v1').digest(1024*1024))
(seed/'documents'/'zero.dat').write_bytes(bytes(128*1024))
(seed/'documents'/'rapport-één.txt').write_text('Dit is synthetische opslagtestdata.\n',encoding='utf-8')
(seed/'empty').touch()
for i in range(128): (seed/'small'/f'file-{i:04}.txt').write_text(f'Deterministic item {i}\n')
os.symlink('documents/rapport-één.txt',seed/'document-link')
expected=corpus_manifest(seed)
(OUT/'expected-corpus.json').write_text(json.dumps(expected,indent=2,ensure_ascii=False)+'\n')
image=BASE/'baseline.img'
with image.open('xb') as f:f.truncate(256*1024*1024)
run('mkfs-baseline',[mkfs,'--rootdir',str(seed),'--label','nieuw-os-probe','-d','single','-m','dup',str(image)])
run('baseline-check',[btrfs,'check','--readonly','--check-data-csum',str(image)])
run('baseline-restore',[btrfs,'restore','-S','-x','-m',str(image),str(BASE/'restore-baseline')])
run('baseline-dump-fs',[btrfs,'inspect-internal','dump-tree','--noscan','-t','fs',str(image)])
run('baseline-dump-chunk',[btrfs,'inspect-internal','dump-tree','--noscan','-t','chunk',str(image)])
run('baseline-dump-super',[btrfs,'inspect-internal','dump-super','-a',str(image)])
observed=corpus_manifest(BASE/'restore-baseline')
report={'seed_count':len(expected),'baseline_sha256':digest(image),'baseline_restore_matches':observed==expected,'baseline_observed':observed}
(OUT/'baseline-result.json').write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({'baseline_restore_matches':observed==expected,'seed_count':len(expected)}),flush=True)
