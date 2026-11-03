import hashlib, json, os, pathlib, re, shutil, subprocess, sys, time

BASE=pathlib.Path(os.environ.get('BTRFS_BASE', '/tmp/nieuw-os-storage-btrfs-20261102')).resolve()
OUT=pathlib.Path(os.environ.get('BTRFS_OUT', '/mnt/c/Users/simon/Documents/Codex/2026-11-02/native-storage-gate/btrfs'))
LOGS=OUT/'logs'
tools=json.loads((OUT/'toolchain.json').read_text()); btrfs=tools['btrfs']
records=json.loads((OUT/'commands.json').read_text())
actions=[]

def safe(path):
    path=path.resolve()
    if not path.is_relative_to(BASE) or path==BASE: raise RuntimeError('Outside owned experiment directory')
    return path

def run(name,argv,timeout=180):
    start=time.time()
    cp=subprocess.run(argv,cwd=BASE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=timeout)
    (LOGS/(name+'.stdout.txt')).write_bytes(cp.stdout)
    (LOGS/(name+'.stderr.txt')).write_bytes(cp.stderr)
    records.append({'name':name,'argv':argv,'cwd':str(BASE),'exit_code':cp.returncode,'elapsed_seconds':round(time.time()-start,3),'stdout':str(LOGS/(name+'.stdout.txt')),'stderr':str(LOGS/(name+'.stderr.txt'))})
    (OUT/'commands.json').write_text(json.dumps(records,indent=2)+'\n')
    print(json.dumps({'name':name,'exit_code':cp.returncode,'stdout_excerpt':cp.stdout.decode(errors='replace')[:1200],'stderr_excerpt':cp.stderr.decode(errors='replace')[:1800]}),flush=True)
    return cp

def digest(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
    return h.hexdigest()

def corpus_manifest(root):
    result={}
    for p in sorted(root.rglob('*')):
        if p.is_symlink():result[str(p.relative_to(root))]={'type':'symlink','target':os.readlink(p)}
        elif p.is_file():result[str(p.relative_to(root))]={'type':'file','bytes':p.stat().st_size,'sha256':digest(p)}
    return result

def restore(name,image,extra=None):
    dest=safe(BASE/('restore-'+name)); dest.mkdir(exist_ok=False)
    cp=run(name+'-restore',[btrfs,'restore','-S','-x','-m']+(extra or [])+[str(image),str(dest)])
    got=corpus_manifest(dest)
    result={'exit_code':cp.returncode,'matches':got==expected,'item_count':len(got),'missing':sorted(set(expected)-set(got)),'extra':sorted(set(got)-set(expected)),'changed':[p for p in expected if p in got and got[p]!=expected[p]],'manifest':got}
    (OUT/(name+'-restore-result.json')).write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n')
    return result

expected=json.loads((OUT/'expected-corpus.json').read_text())
baseline=safe(BASE/'baseline.img'); baseline_before=digest(baseline)
results={'baseline_restore_initial_failure':'Original target directory did not exist; restore encountered symlink first. Preserved baseline-restore log. Corrected by explicitly creating a new empty destination.','baseline_restore':restore('baseline-precreated',baseline)}
run('dependency-installed-versions',['dpkg-query','-W','-f=${Package}\t${Version}\n','libblkid1','libc6','libcom-err2','libext2fs2t64','liblzo2-2','libudev1','libuuid1','libzstd1','zlib1g'])

# Resolve payload's inode and logical extent from the actual filesystem tree.
fs=(LOGS/'baseline-dump-fs.stdout.txt').read_text()
items=re.split(r'(?=\titem \d+ key)',fs)
inode=None
for item in items:
    m=re.search(r'key \((\d+) INODE_REF \d+\)',item)
    if m and re.search(r'name: payload\.bin(?:\n|$)',item):inode=int(m.group(1))
if inode is None:raise RuntimeError('Payload inode not found')
extent=next(i for i in items if re.search(rf'key \({inode} EXTENT_DATA 0\)',i))
match=re.search(r'extent data disk byte (\d+) nr (\d+)',extent)
logical,length=map(int,match.groups())
if 'compression 0 (none)' not in extent:raise RuntimeError('Unexpected compressed extent')
chunk=(LOGS/'baseline-dump-chunk.stdout.txt').read_text()
blocks=re.split(r'(?=\titem \d+ key)',chunk)
physical=None
for item in blocks:
    m=re.search(r'CHUNK_ITEM (\d+)\)',item)
    if not m or 'type DATA|single' not in item:continue
    chunk_start=int(m.group(1));chunk_length=int(re.search(r'length (\d+)',item).group(1))
    if chunk_start<=logical<chunk_start+chunk_length:
        stripe=int(re.search(r'stripe 0 devid 1 offset (\d+)',item).group(1))
        physical=stripe+logical-chunk_start
if physical is None:raise RuntimeError('Data physical mapping not found')
payload=(BASE/'seed/documents/payload.bin').read_bytes()
with baseline.open('rb') as f:f.seek(physical);mapped=f.read(length)
if mapped!=payload:raise RuntimeError('Mapped on-disk payload differs from seed')
mapping={'inode':inode,'logical':logical,'length':length,'physical':physical,'uncompressed_payload_matches_entire_seed':True,'payload_sha256':hashlib.sha256(payload).hexdigest()}
(OUT/'payload-mapping.json').write_text(json.dumps(mapping,indent=2)+'\n')

images={}
for name in ['primary-superblock','payload-bitflip','truncated']:
    image=safe(BASE/(name+'.img'))
    if image.exists():raise RuntimeError('Refusing overwrite of fault image')
    shutil.copyfile(baseline,image)
    before=digest(image)
    if before!=baseline_before:raise RuntimeError('Clone hash differs')
    action={'image':str(image),'source':str(baseline),'source_sha256':before}
    with image.open('r+b') as f:
        if name=='primary-superblock':
            f.seek(65536);original=f.read(4096);f.seek(65536);f.write(bytes(4096))
            action.update({'kind':'zero-primary-superblock','offset':65536,'bytes':4096,'original_region_sha256':hashlib.sha256(original).hexdigest()})
        elif name=='payload-bitflip':
            offset=physical+12345;f.seek(offset);old=f.read(1);new=bytes([old[0]^1]);f.seek(offset);f.write(new)
            action.update({'kind':'single-bit-flip','offset':offset,'file_offset':12345,'old_byte':old[0],'new_byte':new[0]})
        else:
            f.truncate(32*1024*1024)
            action.update({'kind':'truncate','old_length':256*1024*1024,'new_length':32*1024*1024})
        f.flush();os.fsync(f.fileno())
    action['after_mutation_sha256']=digest(image);actions.append(action)
    check=run(name+'-check',[btrfs,'check','--readonly','--check-data-csum',str(image)])
    result={'check_exit_code':check.returncode,'restore':restore(name,image)}
    if name=='primary-superblock':
        alternate=run(name+'-check-super1',[btrfs,'check','--readonly','--check-data-csum','--super','1',str(image)])
        result['check_super1_exit_code']=alternate.returncode
        result['restore_super1']=restore(name+'-super1',image,['--super','1'])
    result['image_sha256_after_readonly_operations']=digest(image)
    result['source_unchanged_by_check_restore']=result['image_sha256_after_readonly_operations']==action['after_mutation_sha256']
    results[name]=result
    images[name]={'path':str(image),'bytes':image.stat().st_size,'sha256':digest(image)}

results['baseline_unchanged']=digest(baseline)==baseline_before
results['images']=images
results['toolchain']=tools
results['mutations']=actions
results['scope']={'filesystem':'Btrfs','tool_version':'6.17.1','package_version':'6.17.1-1build1','image_bytes':256*1024*1024,'data_profile':'single','metadata_profile':'DUP','mounted':False,'kernel_driver_exercised':False,'native_nieuw_os':False,'hardware_power_loss_tested':False,'self_healing_tested':False,'snapshots_tested':False,'encryption_tested':False,'performance_benchmark':False}
results['evidence_type']='offline host-tool recovery and intentional image-corruption experiment'
results['sources']=['https://btrfs.readthedocs.io/en/latest/mkfs.btrfs.html','https://btrfs.readthedocs.io/en/latest/btrfs-check.html','https://btrfs.readthedocs.io/en/latest/btrfs-restore.html']
(OUT/'evidence.json').write_text(json.dumps(results,indent=2,ensure_ascii=False)+'\n')
shutil.copyfile(tools['deb'],OUT/pathlib.Path(tools['deb']).name)
man={str(p.relative_to(OUT)):digest(p) for p in sorted(OUT.rglob('*')) if p.is_file() and p.name!='sha256-manifest.json'}
(OUT/'sha256-manifest.json').write_text(json.dumps(man,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({'baseline_restore':results['baseline_restore']['matches'],'baseline_unchanged':results['baseline_unchanged'],'cases':{n:{'check_exit':results[n]['check_exit_code'],'restore_matches':results[n]['restore']['matches'],'restore_exit':results[n]['restore']['exit_code'],'changed_files':results[n]['restore']['changed'],'source_unchanged':results[n]['source_unchanged_by_check_restore']} for n in images}}),flush=True)
