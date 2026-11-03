"""V4: independent read-only verification of native evidence artifacts.

Only receipts under this script's directory are written. No build, VM,
filesystem mount, repair command or source import is executed.
"""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import subprocess

OUT = Path(__file__).resolve().parent
BASE = OUT.parent
NATIVE = BASE / 'native'
SOURCE = BASE / 'source'
HEAD = '94f12da0b9740a1a0ced1f08b4a30787f82ae79f'
RUNNER = 'dc5707cc6972b6b666d2e9d6925a275850d5afa9b48fa9bd68ce69d2c26175a0'
BOOT = ['BOOT:EBS_OK', 'KERNEL:ENTRY', 'BOOTINFO:OK', 'VOLUME:MANIFEST:PASS']
HASHES = {}

# Fixed from the preserved independent matrix review and the original seed and
# candidate records, not from the matrix being checked. These identify the
# pre-run image for each case; own-normal is expected to change its probe.
IMAGE_HASHES = {
    'own-normal': '45e7b67b8940ac5ad287cd46071fcb67db13dcc58d70c956cf0db92ee6732ae6',
    'own-cold-recover': '19e2fdb3081ed07d54e5f42d152919606aac51ac07cd9dd079769ec011146bb3',
    'btrfs': '0f3e5d9631ac9e8747abd7d6744538772620c0d2d9c6c5f40f2ed7e864edac39',
    'zero': '30e14955ebf1352266dc2ff8067e68104607e750abb9d3b36582b8af909fcb58',
    'zero-prefix-payload-tail': '3dd443382e1edb96c2ebcd6d11c0361f249096245afda8df4d43193e8234270b',
    'openzfs-leaf': '23a409ac5e9a844056d9096ead0453112f1d1e6e45d4b7d5db084a397baea4ea',
}
EXPECTED_RUNS = {
    'own-normal': (IMAGE_HASHES['own-normal'], False),
    'own-cold-recover': (IMAGE_HASHES['own-cold-recover'], True),
    **{f'{candidate}-{mode}': (IMAGE_HASHES[candidate], mode == 'readonly')
       for candidate in ('btrfs', 'zero', 'zero-prefix-payload-tail', 'openzfs-leaf')
       for mode in ('readonly', 'writable')},
}


def sha(path):
    path = Path(path).resolve(strict=True)
    if path not in HASHES:
        with path.open('rb') as stream:
            HASHES[path] = hashlib.file_digest(stream, 'sha256').hexdigest()
    return HASHES[path]


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def need(condition, message):
    if not condition:
        raise AssertionError(message)


def source_fingerprint():
    # Independently implements the documented source-fingerprint-v3 layout;
    # deliberately does not import the runner or source's common.py.
    digest = hashlib.sha256(b'nieuw-os source inputs v3\n')
    folders = ('boot', 'kernel', 'crates', 'drivers', 'services', 'userspace', 'sdk',
               'ui/native', 'ui/surface-protocol', 'experiments/quantum/qsim-core',
               'experiments/quantum/native', 'experiments/quantum/job-protocol', 'tools', '.cargo')
    excluded = {'__pycache__','verification','target','build','.git'}
    for name in folders:
        folder = SOURCE / name
        if folder.exists():
            for path in sorted(folder.rglob('*')):
                if path.is_file() and not excluded.intersection(path.relative_to(folder).parts):
                    digest.update(path.relative_to(SOURCE).as_posix().encode())
                    digest.update(path.read_bytes())
    for name in ('Cargo.toml','Cargo.lock','rust-toolchain.toml','experiments/quantum/Cargo.toml','ui/prototype/tokens.css'):
        path = SOURCE / name
        if path.exists():
            digest.update(name.encode())
            digest.update(path.read_bytes())
    return digest.hexdigest()


def verify_source(record):
    actual_head = subprocess.check_output(['git','-C',str(SOURCE),'rev-parse','HEAD'], text=True).strip()
    need(actual_head == HEAD == record['head'], 'source HEAD mismatch')
    need(sha(NATIVE/'native_admission.py') == RUNNER == record['runner_sha256'], 'runner changed')
    actual_fingerprint = source_fingerprint()
    need(actual_fingerprint == record['source_sha256'], 'source fingerprint differs')
    for name, expected in record['volume_test_inputs'].items():
        need(sha(SOURCE/name) == expected, f'test source changed: {name}')
    return {'head': actual_head, 'source_sha256': actual_fingerprint,
            'runner_sha256': RUNNER, 'test_hashes_checked': len(record['volume_test_inputs'])}


def option_pairs(command, flag):
    return [command[i+1] for i, value in enumerate(command[:-1]) if value == flag]


def check_run_contract(item):
    label = item['label']
    need(label in EXPECTED_RUNS, f'{label}: unexpected label')
    image_hash, readonly = EXPECTED_RUNS[label]
    need(item.get('source_image_sha256_before') == image_hash, f'{label}: expected image hash')
    need(item.get('data_disk_read_only') is readonly, f'{label}: expected data drive mode')


def check_serial_digests(item, serial, label):
    digests = item.get('oracle_digests')
    need(isinstance(digests, list) and len(digests) == 2, f'{label}: expected two oracle digests')
    for prefix, digest in zip(('VOLUME:A:DIGEST:', 'VOLUME:B:DIGEST:'), digests):
        need([int(l[len(prefix):]) for l in serial if l.startswith(prefix)] == [digest], f'{label}: serial digest')


def verify_matrix():
    matrix_path = NATIVE/'matrix.json'
    matrix = read(matrix_path)
    need(matrix['passed'] is True, 'matrix not complete/passed')
    source = verify_source(matrix['source'])
    need(len(matrix['runs']) == 10, 'matrix must contain ten runs')
    need({r['label'] for r in matrix['runs']} == set(EXPECTED_RUNS), 'matrix labels mismatch')
    checked = []
    for item in matrix['runs']:
        label = item['label']
        check_run_contract(item)
        directory = NATIVE/'runs'/label
        report = read(directory/'result.json')
        need(report == item, f'{label}: individual and aggregate report differ')
        need(item['passed'] is True and not item['timed_out'], f'{label}: failed/incomplete')
        need(item['source'] == matrix['source'] and item['source_unchanged'] is True, f'{label}: source binding')
        manifest = read(directory/'build-manifest.json')
        original_manifest = read(NATIVE/'builds'/item['profile']/'manifest.json')
        need(manifest == original_manifest, f'{label}: manifest copy differs')
        need(manifest['source_sha256'] == source['source_sha256'] and manifest['kernel_feature'] == item['profile'], f'{label}: manifest source/profile mismatch')
        actual_kernel = sha(directory/'esp/kernel.elf')
        actual_loader = sha(directory/'esp/EFI/BOOT/BOOTX64.EFI')
        need(actual_kernel == item['kernel_sha256'] == manifest['kernel']['sha256'], f'{label}: kernel hash')
        need(actual_loader == item['loader_sha256'] == manifest['loader']['sha256'], f'{label}: loader hash')
        need(sha(directory/'OVMF_CODE.fd') == item['firmware_code_sha256'], f'{label}: firmware hash')
        original = Path(item['source_image'])
        actual_original, actual_probe = sha(original), sha(directory/'data.img')
        need(original.stat().st_size == item['source_image_size'], f'{label}: source size')
        need(actual_original == item['source_image_sha256_before'] == item['source_image_sha256_after'] == item['probe_image_sha256_before'], f'{label}: original/before image hashes')
        need(actual_probe == item['probe_image_sha256_after'], f'{label}: current probe/after hash')
        serial_file, trace_file = directory/'serial.txt', directory/'stderr-trace.txt'
        need(sha(serial_file) == item['serial_sha256'], f'{label}: serial hash')
        need(sha(trace_file) == item['trace_sha256'], f'{label}: trace hash')
        serial = serial_file.read_text(encoding='utf-8', errors='replace').splitlines()
        traces = trace_file.read_text(encoding='utf-8', errors='replace').splitlines()
        reads = [line for line in traces if re.match(r'^virtio_blk_handle_read\s+vdev\s+',line)]
        writes = [line for line in traces if re.match(r'^virtio_blk_handle_write\s+vdev\s+',line)]
        need(len(reads) == item['trace_read_requests'] > 0, f'{label}: trace read count')
        need(len(writes) == item['trace_write_requests'], f'{label}: trace write count')
        need(all(serial.count(marker) == 1 for marker in BOOT), f'{label}: boot markers')
        command = item['command']
        need(option_pairs(command,'-nic') == ['none'], f'{label}: networking')
        need(option_pairs(command,'-display') == ['none'], f'{label}: display')
        need(option_pairs(command,'-trace') == ['enable=virtio_blk_handle_read','enable=virtio_blk_handle_write'], f'{label}: trace flags')
        data_drives = [v for v in option_pairs(command,'-drive') if 'id=volume,' in v]
        need(len(data_drives) == 1, f'{label}: data drive count')
        readonly = item['data_disk_read_only']
        need(f'readonly={"on" if readonly else "off"}' in data_drives[0] and 'file=data.img' in data_drives[0], f'{label}: data drive mode')
        if item['control']:
            case = 'NORMAL' if item['control'] == 'normal' else 'RECOVER'
            markers = BOOT + ['VOLUME:READY',f'VOLUME:CASE:{case}:PASS','VOLUME:RESOURCES:PASS','VOLUME:PASS','M1:PASS']
            positions = [serial.index(marker) for marker in markers]
            need(positions == sorted(positions), f'{label}: ordered control markers')
            need(item['raw_qemu_exit'] == 33 and not any('FAIL' in l or 'PANIC' in l for l in serial), f'{label}: positive control exit/markers')
            need(item['oracle_generation'] == 84, f'{label}: control generation')
            check_serial_digests(item, serial, label)
            if readonly:
                need(not writes and actual_probe == actual_original, f'{label}: cold image/write count')
            else:
                need(writes and actual_probe != actual_original, f'{label}: normal control did not write')
        else:
            need(item['raw_qemu_exit'] == 35, f'{label}: rejection raw exit')
            need(serial.count('VOLUME:SERVICE_ERROR:18446744073709551598') == 1, f'{label}: Corrupt status')
            need([l for l in serial if l.startswith('VOLUME:FAIL:')] == ['VOLUME:FAIL:SERVICE_STARTUP'], f'{label}: startup failure marker')
            need(not any(l.startswith(('VOLUME:READY','VOLUME:APP_','VOLUME:CASE:','VOLUME:PASS','M1:PASS')) or 'PANIC' in l for l in serial), f'{label}: unexpected app/pass/panic')
            need(not writes and actual_original == actual_probe, f'{label}: mutation')
        checked.append({'label': label, 'passed': True, 'exit': item['raw_qemu_exit'],
            'reads_recounted': len(reads), 'writes_recounted': len(writes),
            'source_image_sha256_recomputed': actual_original, 'probe_image_sha256_recomputed': actual_probe,
            'kernel_sha256_recomputed': actual_kernel, 'loader_sha256_recomputed': actual_loader})
    normal = next(r for r in matrix['runs'] if r['label']=='own-normal')
    cold = next(r for r in matrix['runs'] if r['label']=='own-cold-recover')
    need(normal['probe_image_sha256_after'] == cold['source_image_sha256_before'] == cold['probe_image_sha256_after'], 'cold generation image mismatch')
    need(normal['oracle_digests'] == cold['oracle_digests'], 'cold digest mismatch')
    for name, expected in matrix['seed_hashes'].items():
        need(sha(NATIVE/'seeds'/name) == expected, f'seed hash {name}')
    return {'passed': True, 'matrix_sha256': sha(matrix_path), 'source': source, 'run_count':len(checked),
            'runs': checked, 'cold_image_equal': True, 'readonly_and_writable_negative_cases_each': 4,
            'zero_writes_on_all_negative_traces': True,
            'native_btrfs_or_openzfs_support_verified': False}


def save_receipt(name, body):
    OUT.mkdir(parents=True, exist_ok=True)
    value = {'schema_version':1, 'review_type':'independent AI-team artifact verification; not a human audit',
        'verified_utc':datetime.now(timezone.utc).isoformat(), 'verifier_sha256':sha(__file__),
        'no_vm_or_build_executed': True, **body,
        'hashes_recomputed':len(HASHES),
        'limitations':['Historical pre-run bytes are checked against the preserved source image plus recorded pre-run hashes; no independent observation of the original execution.',
                       'QEMU read/write traces and preserved artifacts verify this virtual-device test scope; not physical power-loss or hostile-device containment.']}
    (OUT/name).write_text(json.dumps(value,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    print(json.dumps({k:value[k] for k in ('passed','review_type','hashes_recomputed')},ensure_ascii=False))


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('action',choices=['matrix'])
    args=parser.parse_args()
    try:
        result=verify_matrix()
        save_receipt('matrix-review.json', result)
    except Exception as exc:
        save_receipt('matrix-review-failure.json',{'passed':False,'error':str(exc)})
        raise


if __name__=='__main__':
    main()
