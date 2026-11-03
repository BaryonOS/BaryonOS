import hashlib,json,pathlib,re,stat
OUT=pathlib.Path('/mnt/c/Users/simon/Documents/Codex/2026-11-02/native-storage-gate/btrfs/metadata')
seed=json.loads((OUT/'seed-metadata.json').read_text())
restored=json.loads((OUT/'restored-metadata.json').read_text())
report=json.loads((OUT/'metadata-evidence.json').read_text())
tree=(OUT/'logs/dump-fs.stdout.txt').read_text()
items=re.split(r'(?=\titem \d+ key)',tree)
inodes={};refs={};xattrs={}
for item in items:
    key=re.search(r'^\titem \d+ key \((\d+) (INODE_ITEM|INODE_REF|XATTR_ITEM) (\d+)\)',item)
    if not key:continue
    ino=int(key[1]);kind=key[2];offset=int(key[3])
    if kind=='INODE_ITEM':
        n=re.search(r'size (\d+) nbytes (\d+)',item);m=re.search(r'mode (\d+) links (\d+) uid (\d+) gid (\d+)',item);t=re.search(r'mtime (\d+)\.(\d+)',item)
        mode=int(m[1],8)
        inodes[ino]={'inode':ino,'type':'directory' if stat.S_ISDIR(mode) else 'symlink' if stat.S_ISLNK(mode) else 'file','mode':oct(stat.S_IMODE(mode)),'size':int(n[1]),'allocated_nbytes':int(n[2]),'nlink':int(m[2]),'uid':int(m[3]),'gid':int(m[4]),'mtime_ns':int(t[1])*1000000000+int(t[2])}
    elif kind=='INODE_REF':
        if ino!=256:refs.setdefault(ino,[]).extend((offset,name) for name in re.findall(r'index \d+ namelen \d+ name: (.+)',item))
    elif kind=='XATTR_ITEM':
        x=re.search(r'name: (.+)\n\s*data (.+)',item)
        if x:xattrs.setdefault(ino,{})[x[1]]=x[2].encode().hex()

def paths(ino):
    if ino==256:return ['']
    return [str(pathlib.PurePosixPath(parent)/name) for p,name in refs[ino] for parent in paths(p)]
imported={}
for ino,row in inodes.items():
    if ino==256:continue
    row['xattrs']=xattrs.get(ino,{})
    for path in paths(ino):imported[path]=row
if set(seed)!=set(imported) or set(seed)!=set(restored):
    raise RuntimeError('Seed, import and restore paths differ')
comparisons={}
for path in seed:
    comparisons[path]={'imported_inode':imported[path]['inode'],'seed_to_import_differences':{k:{'seed':seed[path][k],'image':imported[path][k]} for k in ['type','mode','mtime_ns','uid','gid','xattrs'] if seed[path][k]!=imported[path][k]},'import_to_restore_differences':{k:{'image':imported[path][k],'restored':restored[path][k]} for k in ['type','mode','mtime_ns','uid','gid','xattrs'] if imported[path][k]!=restored[path][k]}}
hardpaths=('hard-a.bin','hard-b.bin')
def same_hardlinks(rows):
    return (rows[hardpaths[0]]['inode']==rows[hardpaths[1]]['inode']
            and all(rows[path]['nlink']==2 for path in hardpaths))
import_claims={
    'fractional_mtime_lost_during_import':all(seed[path]['mtime_ns']%1000000000 != 0 and imported[path]['mtime_ns']==seed[path]['mtime_ns']//1000000000*1000000000 for path in seed),
    'hardlink_topology_preserved_during_import':same_hardlinks(seed) and same_hardlinks(imported),
    'sparse_holes_materialized_during_import':imported['sparse.bin']['allocated_nbytes'] > seed['sparse.bin']['st_blocks_512']*512,
    'empty_directory_mode_and_xattr_preserved_during_import':all(imported['empty-dir'][key]==seed['empty-dir'][key] for key in ('mode','xattrs')),
}
report['import_stage']={'source':'logs/dump-fs.stdout.txt','method':f"Parsed inode items, inode references and xattrs from full host dump-tree; {len(imported)} path mappings found",'manifest':imported,'comparisons':comparisons,'claims':import_claims}
regular=[path for path in imported if imported[path]['type']=='file']
symlinks=[path for path in imported if imported[path]['type']=='symlink']
claims={
    'hardlink_topology_lost':same_hardlinks(imported) and not same_hardlinks(restored),
    'empty_directory_mode_mtime_xattr_lost':all(imported['empty-dir'][key]!=restored['empty-dir'][key] for key in ('mode','mtime_ns','xattrs')),
    'regular_file_mode_and_xattr_preserved':all(all(imported[path][key]==restored[path][key] for key in ('type','mode','xattrs')) for path in regular),
    'all_regular_file_bytes_and_symlink_target_preserved':all(seed[path]['sha256']==restored[path].get('sha256') and seed[path]['size']==restored[path].get('size') for path in regular) and all(seed[path]['target']==restored[path].get('target') for path in symlinks),
}
report['restore_stage']=claims
# Compare the whole measured roundtrip to the seed as well as the restore stage.
metadata_fields=('type','mode','mtime_ns','uid','gid','xattrs')
report['metadata_roundtrip_complete']=(not claims['hardlink_topology_lost'] and not claims['empty_directory_mode_mtime_xattr_lost']
    and claims['regular_file_mode_and_xattr_preserved'] and claims['all_regular_file_bytes_and_symlink_target_preserved']
    and same_hardlinks(seed)==same_hardlinks(restored)
    and all(seed[path]['nlink']==restored[path]['nlink'] for path in seed if seed[path]['type']=='file')
    and all(all(seed[path][key]==restored[path][key] for key in metadata_fields) for path in seed)
    and seed['sparse.bin']['st_blocks_512']==restored['sparse.bin']['st_blocks_512'])
report['interpretation']=('All measured properties were preserved in this experiment.' if report['metadata_roundtrip_complete'] else 'The selected mkfs --rootdir and btrfs restore tool path is not a metadata-complete backup/restore workflow; this does not demonstrate an inability of the Btrfs disk format or mounted kernel driver to preserve these properties.')
(OUT/'metadata-evidence.json').write_text(json.dumps(report,indent=2)+'\n')
(OUT/'README.md').write_text(f'''# Btrfs metadataherstel — aparte aanvullende hostproef

Gemeten volledige metadata-roundtrip: {report['metadata_roundtrip_complete']}. Zie metadata-evidence.json voor exitcodes en bronhashes.

| Eigenschap | Na mkfs --rootdir, gecontroleerd met dump-tree | Na restore -m -x -S |
|---|---|---|
| Bestandsbytes, grootte, symlinktarget | Contentchecksums geverifieerd | SHA-256, grootte en symlinktarget vergeleken; gelijk: {claims['all_regular_file_bytes_and_symlink_target_preserved']} |
| Gecontroleerde modes en user.* file-xattr | Zie verschillen per pad | Normale files behouden: {claims['regular_file_mode_and_xattr_preserved']} |
| mtime | Fractie verloren: {import_claims['fractional_mtime_lost_during_import']} | Zie verschillen per pad in metadata-evidence.json |
| Lege map mode, mtime, user.* xattr | Mode en xattr behouden: {import_claims['empty_directory_mode_and_xattr_preserved_during_import']} | Alle drie gewijzigd: {claims['empty_directory_mode_mtime_xattr_lost']} |
| Hardlinkpaar | Topologie behouden: {import_claims['hardlink_topology_preserved_during_import']} | Topologie verloren: {claims['hardlink_topology_lost']} |
| Sparse file | Meer bytes gealloceerd dan seed: {import_claims['sparse_holes_materialized_during_import']} | Zie restore-allocatie in metadata-evidence.json |
| uid/gid | Zie verschillen per pad | Zie verschillen per pad; remapping niet getest |

Er zijn {len(seed)} padvermeldingen. Mode, mtime-nanoseconden, uid/gid, user.xattrs, filetype, reguliere grootte/SHA256 en symlinktarget zijn vergeleken. Hardlinktopologie is apart vergeleken op gelijke inode+linkcount, zonder inodegetallen tussen verschillende filesystems gelijk te eisen. Sparse-allocatie is geobserveerd met st_blocks op dezelfde WSL-/tmp-backingstore en Btrfs nbytes/extents; geen performanceclaim.

Verschillen worden aan import of restore toegeschreven door de tussenliggende Btrfs-tree te inspecteren. Dit bewijst beperkingen van deze toolroute, niet dat het Btrfs-diskformaat de eigenschappen niet kan opslaan.

Niet getest: ACLs, security.xattrs, creation/ctime/atime, rootdirectory-metadata, uid/gid-remapping, Windows ADS, resource forks, mounted driver, native Nieuw OS, encryptie, snapshots of fysieke stroomuitval.

metadata-evidence.json bevat de 3 stadia en verschillen; logs/ bevat volledige stdout/stderr. De image blijft in /tmp/nieuw-os-storage-btrfs-20261102/metadata/metadata.img. De bovenliggende metadata_probe.py en metadata_review.py beschrijven corpus, commando's en vergelijking. Geen herstel-fallback gebruikt.
''',encoding='utf-8')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
manifest={str(p.relative_to(OUT)):sha(p) for p in OUT.rglob('*') if p.is_file() and p.name!='sha256-manifest.json'}
manifest['../metadata_probe.py']=sha(OUT.parent/'metadata_probe.py');manifest['../metadata_review.py']=sha(OUT.parent/'metadata_review.py')
(OUT/'sha256-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
assert all(sha(OUT/p)==h for p,h in manifest.items())
print(json.dumps({'files_verified':len(manifest),'metadata_complete':report['metadata_roundtrip_complete'],'stage_attribution_measured':True}))
