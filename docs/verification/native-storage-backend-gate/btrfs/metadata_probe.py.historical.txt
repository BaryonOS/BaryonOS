import hashlib,json,os,pathlib,stat,subprocess,time
BASE=pathlib.Path('/tmp/nieuw-os-storage-btrfs-20261102/metadata')
OUT=pathlib.Path('/mnt/c/Users/simon/Documents/Codex/2026-11-02/native-storage-gate/btrfs/metadata')
if BASE.exists() or OUT.exists():raise RuntimeError('Refusing overwrite of existing metadata experiment')
BASE.mkdir();OUT.mkdir();(OUT/'logs').mkdir()
toolchain=json.loads((OUT.parent/'toolchain.json').read_text())
btrfs=toolchain['btrfs'];mkfs=toolchain['mkfs'];commands=[]

def sha(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
    return h.hexdigest()

def run(name,cmd):
    t=time.monotonic();p=subprocess.run(cmd,cwd=BASE,capture_output=True,timeout=180)
    (OUT/'logs'/f'{name}.stdout.txt').write_bytes(p.stdout)
    (OUT/'logs'/f'{name}.stderr.txt').write_bytes(p.stderr)
    commands.append({'name':name,'command':cmd,'cwd':str(BASE),'exit_code':p.returncode,'elapsed_seconds':time.monotonic()-t})
    (OUT/'commands.json').write_text(json.dumps(commands,indent=2)+'\n')
    print(json.dumps({'name':name,'exit_code':p.returncode,'stderr':p.stderr.decode(errors='replace')[:1200]}),flush=True)
    return p

def describe(root):
    rows={}
    for p in sorted(root.rglob('*')):
        s=p.lstat();kind='symlink' if stat.S_ISLNK(s.st_mode) else 'directory' if stat.S_ISDIR(s.st_mode) else 'file'
        row={'type':kind,'mode':oct(stat.S_IMODE(s.st_mode)),'mtime_ns':s.st_mtime_ns,'uid':s.st_uid,'gid':s.st_gid,'inode':s.st_ino,'nlink':s.st_nlink,'st_blocks_512':s.st_blocks,'size':s.st_size,'xattrs':{k:os.getxattr(p,k,follow_symlinks=False).hex() for k in sorted(os.listxattr(p,follow_symlinks=False))}}
        if kind=='file':row['sha256']=sha(p)
        if kind=='symlink':row['target']=os.readlink(p)
        rows[str(p.relative_to(root))]=row
    return rows

seed=BASE/'seed';seed.mkdir();(seed/'empty-dir').mkdir();(seed/'nested').mkdir()
(seed/'mode-file.txt').write_bytes(b'controlled mode time xattr\n')
(seed/'hard-a.bin').write_bytes(hashlib.shake_256(b'metadata-hardlinks-v1').digest(8192))
os.link(seed/'hard-a.bin',seed/'hard-b.bin')
with (seed/'sparse.bin').open('wb') as f:
    f.write(b'A'*4096);f.seek(8*1024*1024-4096);f.write(b'Z'*4096)
(seed/'nested'/'child.txt').write_bytes(b'nested child\n')
os.symlink('mode-file.txt',seed/'symbolic-link')
for p in sorted(seed.rglob('*')):
    if not p.is_symlink():os.chmod(p,0o700 if p.is_dir() else 0o640)
    os.utime(p,ns=(1577836800123456789,1577836800123456789),follow_symlinks=False)
os.chmod(seed/'mode-file.txt',0o600)
os.chmod(seed/'hard-a.bin',0o644)
os.setxattr(seed/'mode-file.txt','user.nieuwos.test',b'file-xattr-value-v1')
os.setxattr(seed/'empty-dir','user.nieuwos.test',b'empty-dir-xattr-value-v1')
# Reapply times after corpus construction so directory mtimes are controlled.
for p in seed.rglob('*'):os.utime(p,ns=(1577836800123456789,1577836800123456789),follow_symlinks=False)
expected=describe(seed);(OUT/'seed-metadata.json').write_text(json.dumps(expected,indent=2)+'\n')
image=BASE/'metadata.img'
with image.open('xb') as f:f.truncate(256*1024*1024)
mk=run('mkfs',[mkfs,'--rootdir',str(seed),'-d','single','-m','dup','--label','metadata-probe',str(image)])
if mk.returncode:raise RuntimeError('mkfs failed')
before=sha(image)
check=run('check',[btrfs,'check','--readonly','--check-data-csum',str(image)])
tree=run('dump-fs',[btrfs,'inspect-internal','dump-tree','--noscan','-t','fs',str(image)])
dest=BASE/'restored';dest.mkdir()
rest=run('restore',[btrfs,'restore','-m','-x','-S',str(image),str(dest)])
got=describe(dest);(OUT/'restored-metadata.json').write_text(json.dumps(got,indent=2)+'\n')
fields=['type','mode','mtime_ns','uid','gid','xattrs','sha256','target']
diff={}
for path in sorted(set(expected)|set(got)):
    if path not in expected or path not in got:diff[path]={'missing_or_extra':path not in got};continue
    d={f:{'expected':expected[path].get(f),'observed':got[path].get(f)} for f in fields if expected[path].get(f)!=got[path].get(f)}
    if expected[path]['type']=='file' and expected[path]['size']!=got[path]['size']:d['size']={'expected':expected[path]['size'],'observed':got[path]['size']}
    if d:diff[path]=d
hard_expected=expected['hard-a.bin']['inode']==expected['hard-b.bin']['inode']
hard_observed=got.get('hard-a.bin',{}).get('inode')==got.get('hard-b.bin',{}).get('inode') if all(k in got for k in ['hard-a.bin','hard-b.bin']) else False
report={'kind':'offline metadata roundtrip via mkfs --rootdir and btrfs restore','toolchain':toolchain,'mkfs_exit':mk.returncode,'check_exit':check.returncode,'restore_exit':rest.returncode,'image_sha256_before':before,'image_sha256_after':sha(image),'image_unchanged':before==sha(image),'paths_match':set(expected)==set(got),'entries':len(expected),'compared_fields':fields+['regular file size'],'differences':diff,'hardlink_pair':{'seed_same_inode':hard_expected,'restored_same_inode':hard_observed,'seed_nlink':expected['hard-a.bin']['nlink'],'restored_nlink':got.get('hard-a.bin',{}).get('nlink')},'sparse_file':{'seed_size':expected['sparse.bin']['size'],'restored_size':got.get('sparse.bin',{}).get('size'),'seed_allocated_512b_blocks':expected['sparse.bin']['st_blocks_512'],'restored_allocated_512b_blocks':got.get('sparse.bin',{}).get('st_blocks_512'),'content_matches':expected['sparse.bin']['sha256']==got.get('sparse.bin',{}).get('sha256'),'allocation_comparison_scope':'seed and restored files are both on same WSL /tmp backing filesystem; st_blocks is allocation observation, not a general filesystem performance score'},'not_tested':['ACLs','uid/gid remapping across users/devices','creation time','ctime/atime preservation','security xattrs','Windows ADS','resource forks','native Nieuw OS','kernel mounts','hardware power failure'],'scope':{'mounts':False,'kernel_driver':False,'native':False,'physical_device':False}}
(OUT/'metadata-evidence.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report),flush=True)
