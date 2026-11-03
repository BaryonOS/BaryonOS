from pathlib import Path
import datetime, hashlib, json, re
base = Path('/mnt/c/Users/simon/Documents/Codex/2026-11-02/native-storage-gate/openzfs')
logs = base/'logs'
def txt(name): return (logs/name).read_text(errors='replace')
def digest(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda:f.read(1024*1024),b''): h.update(chunk)
    return h.hexdigest()
def code(name): return int(txt(name+'-exit.txt').strip())
def metrics(name):
    t=txt(name+'-time.txt')
    def extract(pattern):
        m=re.search(pattern,t); return m.group(1).strip() if m else None
    return {'elapsed':extract(r'Elapsed \(wall clock\) time \(h:mm:ss or m:ss\): (.+)'), 'max_rss_kib':int(extract(r'Maximum resident set size \(kbytes\): (\d+)')), 'raw_time_log':'logs/'+name+'-time.txt'}
packages=[]
metadata=txt('package-metadata.txt')
for block in metadata.strip().split('\n\n'):
    fields=dict(re.findall(r'^([A-Za-z0-9-]+): (.*)$',block,re.M))
    if 'Filename' not in fields: continue
    path=base/'packages'/Path(fields['Filename']).name
    actual=digest(path)
    packages.append({'package':fields['Package'],'version':fields['Version'],'architecture':fields['Architecture'],'origin':fields.get('Origin'),'upstream_filename':fields['Filename'],'local_path':str(path.relative_to(base)), 'sha256':actual,'apt_metadata_sha256':fields['SHA256'],'matches_apt_metadata':actual==fields['SHA256']})
assert len(packages)==6 and all(p['matches_apt_metadata'] for p in packages)
features=[]
for name,count in re.findall(r'Verified (\S+) feature refcount of (\d+) is correct',txt('fresh-zdb.txt')):
    features.append({'feature':name,'refcount':int(count)})
datasets=re.findall(r'^Dataset (.+)$',txt('fresh-zdb.txt'),re.M)
images=[]
for p in sorted((base/'images').glob('nieuwos_n3.*a')):
    images.append({'path':str(p.relative_to(base)),'bytes':p.stat().st_size,'sha256':digest(p),'type':'file-vdev mirror member; independent degraded import not tested'})
source_paths={'ztest-2.4.1.c':'cmd/ztest.c','zdb-2.4.1.c':'cmd/zdb/zdb.c','libzpool-kernel.c':'lib/libzpool/kernel.c','libzpool-util.c':'lib/libzpool/util.c','zutil-import.c':'lib/libzutil/zutil_import.c'}
sources=[{'local_path':'sources/'+name,'url':'https://raw.githubusercontent.com/openzfs/zfs/zfs-2.4.1/'+path,'sha256':digest(base/'sources'/name),'limit':'Pinned upstream tag used for interpretation; Ubuntu package may contain distro patches.'} for name,path in source_paths.items()]
evidence={
 'schema_version':1,'task':'N3 OpenZFS proven-engine host userspace recovery gate',
 'generated_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'overall_decision':'NOT_ACCEPTED_FOR_NIEUW_OS_NATIVE_BACKEND_OR_FILE_RECOVERY',
 'summary_nl':'Echte OpenZFS poolaanmaak en checksumcontrole in verse userspaceprocessen zijn aangetoond; gewone bestandsrecovery en eigen-kernelintegratie niet. Crashrun heeft een onopgelost allocatieverschil en voldoet niet aan de strenge no-leak oracle.',
 'scope':{'execution':'WSL Ubuntu 26.04.1, existing Linux host userspace, amd64','engine':'Ubuntu OpenZFS 2.4.1-1ubuntu5.1 libzpool + ztest + zdb','installation':'apt download and dpkg-deb -x only; no package install, kernel module, mount, loop device, physical drive, repository edit or filesystem driver','allowed_writes':['/tmp/nieuw-os-storage-zfs-20261102','C:/Users/simon/Documents/Codex/2026-11-02/native-storage-gate/openzfs'],'runtime_exception':'ztest creates hardcoded /tmp/ztest.data.XXXXXX with mkstemp and immediately unlinks it; parent explicitly authorized this ephemeral test-harness file. No engine modification.'},
 'preflight_observations':{'source':'Initial tool preflight observed in conversation; raw preflight transcript was not separately retained. Executed binary dependency output retained in logs/*-ldd.txt.','compiler_available':False,'make_available':False,'initial_zfs_tools_available':False,'host_ram_approx_gib':30,'logical_cpu_count':32,'swap_gib':8},
 'resource_bounds':{'virtual_memory_per_process_kib':8388608,'aggregate_rss_hard_limit':None,'ulimit_c':0,'ulimit_f':4194304,'MALLOC_ARENA_MAX':'2','ZFS_STACK_SIZE':'131072','test_worker_threads':2,'ztest_run_seconds_argument':15,'ztest_pass_seconds_argument':5,'timeouts_seconds':{'create':90,'create_crash':120,'fresh_checks':60,'export_and_object_dump':30,'blockpointer_listing':40},'timeout_kill_grace_seconds':5,'note':'All long probes wrapped in timeout; maxima are observed time(1) process-tree RSS, not hardware performance/RAM sizing recommendations.'},
 'packages':packages,'source_interpretation':sources,'license':{'engine':'CDDL-1.0 with component-specific notices; see complete distro copyright file','artifact':'licenses/ubuntu-zfs-copyright','note':'No license compatibility or kernelport conclusion inferred solely from package execution.'},
 'baseline':{'pool':'nieuwos_n3','pool_guid':'9169139500157372876','label_version':5000,'topology':'one top-level mirror with two regular file leaves, 256 MiB each','ashift':9,'best_uberblock_txg':526,'immutable_windows_copies':images,'on_disk_read_features_from_label':['com.delphix:hole_birth','com.delphix:embedded_data','com.klarasystems:vdev_zaps_v2'],'feature_refcounts_observed':features,'dataset_manifest':datasets,'content_limit':'ztest OTHER/DMU objectsets; empty ZPL root. No named user-file payload oracle or filesystem tree.','all_source_and_copy_image_hashes':'logs/final-images-sha256.txt'},
 'gates':[
 {'id':'create','status':'PASS_HOST_POOL_OBJECT_TEST_ONLY','exit_code':code('create-bounded'),'metrics':metrics('create-bounded'),'completed_passes':2,'killed_passes':0,'fault_inject_calls':0,'log':'logs/create-bounded.txt','script':'03-create.sh'},
 {'id':'fresh_import_check','status':'PASS_HOST_POOL_OBJECT_CONSISTENCY_ONLY','exit_code':code('fresh-zdb'),'metrics':metrics('fresh-zdb'),'block_count':1141,'no_leaks_message':True,'checksums':'all metadata and data, -bcc','import_evidence':'spa_import and LOADED debug messages; separate process after ztest ended','all_reconstruction_Y':True,'log':'logs/fresh-zdb.txt','script':'04-baseline-freshcheck.sh'},
 {'id':'fresh_check_without_Y','status':'PASS_HOST_POOL_OBJECT_CONSISTENCY_ONLY','exit_code':code('baseline-allblocks'),'block_count':1141,'no_leaks_message':True,'all_reconstruction_Y':False,'log':'logs/baseline-allblocks.txt','script':'09-crash-freshcheck-blocks.sh'},
 {'id':'ztest_E_reimport','status':'FAIL_HARNESS_HOST_DEPENDENCY','exit_code':code('reuse-kill'),'metrics':metrics('reuse-kill'),'log':'logs/reuse-kill.txt','script':'05-reuse-kill.sh','finding':'No pools found then assertion. Upstream libzpool/util.c pool_active opens ZFS_DEV and calls ioctl; missing /dev/zfs causes error. zdb can_be_active=true bypasses this host-active check. No module installed and no shim/patch used.','certainty':'Source-based explanation consistent with observed module-free host; no syscall trace was recorded.'},
 {'id':'process_crash','status':'NOT_ACCEPTED_STRICT_NO_LEAK_ORACLE','exit_code':code('crash-create'),'metrics':metrics('crash-create'),'actual_SIGKILL_count':3,'completed_passes':0,'fault_inject_calls':0,'physical_power_loss_test':False,'log':'logs/crash-create.txt','script':'07-crash-create.sh'},
 {'id':'post_crash_fresh_check','status':'NOT_ACCEPTED_UNRESOLVED_ALLOCATION_DIFFERENCE','exit_code':code('crash-fresh-zdb'),'metrics':metrics('crash-fresh-zdb'),'block_count':519,'reported_checksum_errors':0,'reported_leaked_bytes':9216,'traversal_allocated_bytes':1346560,'space_map_allocated_bytes':1355776,'all_reconstruction_Y':False,'log':'logs/crash-fresh-zdb.txt','script':'09-crash-freshcheck-blocks.sh','interpretation':'Exit 0 does not satisfy our stricter no-leak output oracle. Cause remains unresolved; not a production OpenZFS defect claim.','preserved_pool_archive':'images/crash-pool.tar.gz'},
 {'id':'raw_DMU_backup_stream','status':'FAIL_ABORT_EMPTY_OUTPUT','exit_code':code('backup-stream'),'output_bytes':(base/'exports/ds0-raw.zfs').stat().st_size,'log':'logs/backup-stream.txt','script':'08-backup-stream.sh','finding':'dsl_crypto_populate_key_nvlist assertion; no usable backup stream and no successful receive/restore.'},
 {'id':'verbose_DMU_object_dump','status':'FAIL_ASSERTION','exit_code':code('ds0-objects'),'log':'logs/ds0-objects.txt','script':'08-backup-stream.sh','finding':'arc_buf_try_copy_decompressed_data assertion; no payload extraction claim.'},
 {'id':'corrupt_block_mirror_recovery','status':'NOT_TESTED','reason':'No external corruption injected; ztest_fault_inject actual count zero.'},
 {'id':'persistent_self_healing','status':'NOT_TESTED','reason':'No corrupted-copy pre/post proof and no repair claim.'},
 {'id':'named_file_export_restore','status':'NOT_TESTED','reason':'No normal ZPL file tree; DMU/pool verification does not establish file recovery.'},
 {'id':'own_kernel_integration','status':'NOT_TESTED_BY_N3','reason':'Linux dynamic ELF binaries require glibc, ELF interpreter, threads and host syscalls; dependency evidence in ldd logs. No native source build/port attempted; separate native agent tests only format recognition.'}
 ],
 'failed_setup_attempts':[{'script':'03-create-initial-help-mismatch.sh','log':'logs/create.txt','exit_code':code('create'),'finding':'-C off rejected; help syntax differs from actual required property=value.'},{'script':'03-create-invalid-dedup.sh','log':'logs/create-retry.txt','exit_code':code('create-retry'),'finding':'dedup=off not accepted for vdev class state.'},{'script':'03-create-unsupported-tunable.sh','log':'logs/create-run.txt','exit_code':code('create-run'),'finding':'zfs_arc_max/min unavailable as tunables in this package.'},{'script':'03-create-low-as.sh','log':'logs/create-final.txt','exit_code':code('create-final'),'finding':'4 GiB virtual-address cap caused pthread_create EAGAIN; subsequent bounded 8 GiB run succeeded.'}],
 'not_established':['Physical device power-loss safety','Persisted user-file ACK/durability contract','Full-disk and SSD-wear behavior','Lost-key recovery','Recovery on a second physical device','Reliable ordinary file export or import','Stable production RAM/performance figures','Native compatibility with nieuw-os kernel'],
 'official_documentation':['https://openzfs.github.io/openzfs-docs/man/v2.4/1/ztest.1.html','https://openzfs.github.io/openzfs-docs/man/v2.4/8/zdb.8.html'],
 'raw_log_hash_manifest':'artifact-sha256.json'
}
(base/'evidence.json').write_text(json.dumps(evidence,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({'packages_verified':len(packages),'images':images,'gates':[{k:g[k] for k in ('id','status','exit_code') if k in g} for g in evidence['gates']]},indent=2))