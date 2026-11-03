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
assert set(seed)==set(imported)==set(restored)
comparisons={}
for path in seed:
    comparisons[path]={'imported_inode':imported[path]['inode'],'seed_to_import_differences':{k:{'seed':seed[path][k],'image':imported[path][k]} for k in ['type','mode','mtime_ns','uid','gid','xattrs'] if seed[path][k]!=imported[path][k]},'import_to_restore_differences':{k:{'image':imported[path][k],'restored':restored[path][k]} for k in ['type','mode','mtime_ns','uid','gid','xattrs'] if imported[path][k]!=restored[path][k]}}
assert imported['hard-a.bin']['inode']==imported['hard-b.bin']['inode'] and imported['hard-a.bin']['nlink']==2
assert imported['sparse.bin']['allocated_nbytes']==8388608
assert imported['empty-dir']['mode']=='0o700' and imported['empty-dir']['xattrs']==seed['empty-dir']['xattrs']
assert all(row['mtime_ns']==1577836800000000000 for row in imported.values())
report['import_stage']={'source':'logs/dump-fs.stdout.txt','method':'Parsed inode items, inode references and xattrs from full host dump-tree; all 8 path mappings found','manifest':imported,'comparisons':comparisons,'claims':{'fractional_mtime_lost_during_import':True,'hardlink_topology_preserved_during_import':True,'sparse_holes_materialized_during_import':True,'empty_directory_mode_and_xattr_preserved_during_import':True}}
report['restore_stage']={'hardlink_topology_lost':True,'empty_directory_mode_mtime_xattr_lost':True,'regular_file_mode_and_xattr_preserved':True,'all_regular_file_bytes_and_symlink_target_preserved':all(seed[p].get('sha256')==restored[p].get('sha256') and seed[p].get('target')==restored[p].get('target') for p in seed)}
report['metadata_roundtrip_complete']=False
report['interpretation']='The selected mkfs --rootdir and btrfs restore tool path is not a metadata-complete backup/restore workflow; this does not demonstrate an inability of the Btrfs disk format or mounted kernel driver to preserve these properties.'
(OUT/'metadata-evidence.json').write_text(json.dumps(report,indent=2)+'\n')
(OUT/'README.md').write_text('''# Btrfs metadataherstel — aparte aanvullende hostproef

Resultaat: inhoudherstel slaagt; deze mkfs --rootdir + btrfs restore-route bewaart niet alle metadata. Alle vier toolcommando's hebben exitcode 0 en de bronimage blijft bytegelijk. Baseline en N1-fixture zijn niet gewijzigd.

| Eigenschap | Na mkfs --rootdir, gecontroleerd met dump-tree | Na restore -m -x -S |
|---|---|---|
| Bestandsbytes, grootte, symlinktarget | Contentchecksums geverifieerd; volledige bytesvergelijking na restore | Alle inhoud gelijk |
| Gecontroleerde modes en user.* file-xattr | Behouden | Normale files behouden |
| mtime inclusief 123456789 nanoseconden | Fractie verloren; alle mtimes hele seconden | Hele seconden behouden behalve lege map |
| Lege map mode0700, oude mtime, user.* xattr | Alle drie aanwezig, tijd tot hele seconden | Mode0755, actuele mtime, xattr ontbreekt |
| Hardlinkpaar | Eén inode, linkcount2, twee namen | Twee losse inodes, linkcount1 |
| Sparse file8MiB, slechts8KiB oorspronkelijk gealloceerd | Alle8MiB gealloceerd in acht extents | Alle8MiB gealloceerd; inhoud gelijk |
| uid/gid | Gelijk aan huidige WSL-gebruiker1000 | Gelijk; remapping naar andere gebruikers niet getest |

Er zijn8 padvermeldingen (inclusief2 directories en1 symlink). Mode, mtime-nanoseconden, uid/gid, user.xattrs, filetype, reguliere grootte/SHA256 en symlinktarget zijn vergeleken. Hardlinktopologie is apart vergeleken op gelijke inode+linkcount, zonder inodegetallen tussen verschillende filesystems gelijk te eisen. Sparse-allocatie is geobserveerd met st_blocks op dezelfde WSL-/tmp-backingstore en Btrfs nbytes/extents; geen performanceclaim.

Deze verliezen zijn aan import of restore toegeschreven door de tussenliggende Btrfs-tree te inspecteren. Dit bewijst beperkingen van deze toolroute, niet dat het Btrfs-diskformaat de eigenschappen niet kan opslaan.

Niet getest: ACLs, security.xattrs, creation/ctime/atime, rootdirectory-metadata, uid/gid-remapping, Windows ADS, resource forks, mounted driver, native Nieuw OS, encryptie, snapshots of fysieke stroomuitval.

metadata-evidence.json bevat de3 stadia en verschillen; logs/ bevat volledige stdout/stderr. De image blijft in /tmp/nieuw-os-storage-btrfs-20261102/metadata/metadata.img. De bovenliggende metadata_probe.py en metadata_review.py beschrijven corpus, commando's en vergelijking. Geen herstel-fallback of herhaalde succesvolle substituutrun gebruikt.
''',encoding='utf-8')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
manifest={str(p.relative_to(OUT)):sha(p) for p in OUT.rglob('*') if p.is_file() and p.name!='sha256-manifest.json'}
manifest['../metadata_probe.py']=sha(OUT.parent/'metadata_probe.py');manifest['../metadata_review.py']=sha(OUT.parent/'metadata_review.py')
(OUT/'sha256-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
assert all(sha(OUT/p)==h for p,h in manifest.items())
print(json.dumps({'files_verified':len(manifest),'metadata_complete':False,'stage_attribution_assertions_passed':True}))
