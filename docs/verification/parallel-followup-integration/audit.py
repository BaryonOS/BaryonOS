#!/usr/bin/env python3
"""I12: read-only crosscheck of six preserved follow-up VMs; starts no VM."""
from __future__ import annotations

import datetime
import hashlib
import json
from pathlib import Path
import random
import re
import struct
import subprocess
import sys

from PIL import Image, ImageChops

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
DOCS = ROOT / 'docs/verification'
HEAD = 'dbd35b04d742b18d66b40fdd65d8a9b900656cac'
TREE = '421bb96102779d01d9cb4e6231d686aea417d63a'
SOURCE = 'c5f3d9316433ba09ba4b886985ffa8b28643d67c72a9ebb0e2b0136362dc7a9f'
BOOT = ['BOOT:EBS_OK', 'KERNEL:ENTRY', 'BOOTINFO:OK', 'KS-01:PASS', 'KS-02:PASS', 'KS-03:PASS']
FILES, VMS = {}, []


def need(condition, label):
    if not condition:
        raise ValueError(label)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def read(path, expected=None):
    path = Path(path).resolve(strict=True)
    raw = path.read_bytes()
    item = {'sha256': sha(raw), 'bytes': len(raw)}
    if str(path) in FILES:
        need(FILES[str(path)] == item, f'Input changed while checking: {path}')
    if expected is not None:
        need(item['sha256'] == expected, f'Hash mismatch: {path}')
    FILES[str(path)] = item
    return raw


def load(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            need(key not in result, f'Duplicate JSON key: {path}/{key}')
            result[key] = value
        return result
    return json.loads(read(path), object_pairs_hook=unique)


def same(left, right):
    need(read(left) == read(right), f'Archive differs from original: {left} / {right}')


def source(item, head_key='base_commit'):
    need(item[head_key] == HEAD and item['source_sha256'] == SOURCE, 'Mixed source or commit')
    if 'source_fingerprint_version' in item:
        need(item['source_fingerprint_version'] == 3, 'Wrong source fingerprint version')


def snapshot():
    sys.path.insert(0, str(ROOT / 'tools'))
    from common import source_hash
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=ROOT, text=True).strip()
    fingerprint = source_hash()
    need((head, tree, fingerprint) == (HEAD, TREE, SOURCE), 'Checkout departed from frozen source')
    return {'head': head, 'tree': tree, 'source_fingerprint_version': 3, 'source_sha256': fingerprint}


def vm(archive, label, feature, sdk=False, graphics=False):
    result = load(archive)
    source(result)
    run = Path(result['run_directory']).resolve(strict=True)
    same(archive, run / 'result.json')
    need(type(result['raw_qemu_exit']) is int and result['raw_qemu_exit'] == 33, label + ': raw exit')
    need(result['passed'] is True and 0 < result['duration_s'] < 60, label + ': completed VM')
    if graphics:
        need(result.get('error') is None and result['checks']['real_exit33'] is True, 'G0 VM failed')
    else:
        need(result['timed_out'] is False and result['expected_exit'] == 33, label + ': timeout')
    serial = read(run / 'serial.txt')
    need(serial.decode('utf-8', 'replace') == result['stdout'], label + ': reported serial bytes')
    stderr = read(run / 'stderr.txt')
    need(not stderr, label + ': unexpected QEMU stderr')
    if not graphics:
        need(stderr.decode('utf-8', 'replace') == result['stderr'], label + ': stderr report')
    if 'serial_sha256' in result:
        need(sha(serial) == result['serial_sha256'] and sha(stderr) == result['stderr_sha256'], 'Raw log hashes')
    lines = serial.decode().splitlines()
    need(lines.count('BOOT:EBS_OK') == 1 and not any('FAIL' in x or 'PANIC' in x for x in lines), label + ': serial failure')
    lines = lines[lines.index('BOOT:EBS_OK'):]
    need(lines[:6] == BOOT and lines[-1] == 'M1:PASS', label + ': boot/end order')
    artifacts = result['artifacts' if graphics else 'artifact_sha256']
    for name, expected in artifacts.items():
        need((run / name).resolve().is_relative_to(run), label + ': artifact path escaped')
        read(run / name, expected)
    command = result['command']
    for flag, value in [('-machine', 'pc-q35-7.2'), ('-accel', 'tcg,thread=single'), ('-cpu', 'qemu64'), ('-smp', '1'), ('-m', '256M'), ('-nic', 'none')]:
        need(command.count(flag) == 1 and command[command.index(flag) + 1] == value, label + ': profile ' + flag)
    need('isa-debug-exit,iobase=0xf4,iosize=0x04' in command, label + ': exit device')
    qemu_hash = result.get('qemu_sha256', result.get('qemu_entry_sha256'))
    if qemu_hash:
        read(Path(command[0]), qemu_hash)
    base = Path(result.get('cwd', ROOT))
    for name in ('OVMF_CODE.fd', 'OVMF_VARS.fd', 'esp'):
        need(any('file=' in arg and Path(base / arg.split('file=', 1)[1].removeprefix('fat:')).resolve() == run / name for arg in command), label + ': actual drive ' + name)
    manifest_path = run / 'build-manifest.json'
    if not manifest_path.exists():
        manifest_path = archive.parent / 'build-manifest.json'
    manifest = load(manifest_path)
    need(manifest['source_sha256'] == SOURCE and manifest['source_fingerprint_version'] == 3, label + ': manifest source')
    need(manifest['kernel_feature'] == feature and manifest['sdk_runtime'] is sdk, label + ': feature')
    for key, name in [('kernel', 'esp/kernel.elf'), ('loader', 'esp/EFI/BOOT/BOOTX64.EFI')]:
        need(manifest[key]['sha256'] == artifacts[name], label + ': manifest artifact')
    VMS.append({'label': label, 'run_directory': str(run), 'raw_exit': 33,
                'timeout': False, 'timeout_evidence': 'raw33/error=null/complete markers' if graphics else 'explicit timed_out=false',
                'duration_s': result['duration_s'], 'source_commit': HEAD, 'source_sha256': SOURCE,
                'kernel_sha256': artifacts['esp/kernel.elf'], 'loader_sha256': artifacts['esp/EFI/BOOT/BOOTX64.EFI'],
                'serial_sha256': sha(serial), 'serial_bytes': len(serial), 'artifact_count': len(artifacts)})
    return result, run, lines, manifest


def g0():
    bundle = DOCS / 'parallel-followup-g0'
    report, run, lines, _ = vm(bundle / 'result.json', 'B08-G0', 'graphics-g0', graphics=True)
    expected = BOOT + ['G0:FRAMEBUFFER:800:600:800:1', 'G0:MAPPINGS:PASS', 'G0:READY',
                       'G0:INPUT:1', 'G0:INPUT:2', 'G0:INPUT:1', 'G0:INPUT:2', 'G0:PASS', 'M1:PASS']
    need(lines == expected, 'G0 exact serial sequence')
    capture = load(bundle / 'capture.json')
    source(capture, 'source_commit')
    for name, value in capture['input_sha256'].items():
        read(run / name, value)
    for name, value in capture['files'].items():
        read(bundle / name, value)
    for name in ('serial.txt', 'stderr.txt', 'result.json', 'build-manifest.json', 'qmp-trace.json'):
        same(bundle / name, run / name)
    images = []
    decoded = []
    for item in load(bundle / 'image-roundtrip.json')['images']:
        ppm = read(run / (item['name'] + '.ppm'), item['ppm_sha256'])
        png = bundle / (item['name'] + '.png')
        read(png, item['png_sha256'])
        with Image.open(png) as image:
            image.load()
            need(image.mode == 'RGB' and image.size == (800, 600), 'G0 PNG dimensions/mode')
            reconstructed = bytes.fromhex(item['original_header_hex']) + image.tobytes()
            need(reconstructed == ppm, 'PNG does not reconstruct original PPM bytes')
            decoded.append(image.copy())
        images.append({'name': item['name'], 'ppm_sha256': sha(ppm), 'png_sha256': item['png_sha256'], 'exact_reconstruction': True})
    need(len(images) == 5 and decoded[1].tobytes() == decoded[3].tobytes(), 'G0 backspace restoration')
    bounds = [list(ImageChops.difference(a, b).getbbox()) for a, b in zip(decoded, decoded[1:])]
    need(bounds == report['step_changed_bounds'], 'G0 actual pixel delta mismatch')
    trace = load(run / 'qmp-trace.json')
    need(len(trace) == 11, 'G0 QMP exchange count')
    keys = []
    for index, entry in enumerate(trace, 1):
        need(entry['request']['id'] == entry['response']['id'] == index and entry['response'].get('return') == {}, 'G0 QMP reply correlation')
        if entry['request']['execute'] == 'send-key':
            keys.append(entry['request']['arguments']['keys'][0]['data'])
    need(keys == ['a', 'b', 'backspace', 'c', 'esc'], 'G0 actual QMP key sequence')
    return {'images': images, 'step_changed_bounds': bounds, 'qmp_exchanges': len(trace), 'backspace_exact': True}


def qsim():
    bundle = DOCS / 'parallel-followup-qsim'
    report, run, lines, manifest = vm(bundle / 'result.json', 'I11-qsim', 'native-qsim')
    acceptance = load(bundle / 'acceptance.json')
    need(acceptance['guest'] == report and all(value is True for value in acceptance['checks'].values()), 'Qsim bound acceptance')
    need(len(lines) == 28 and lines[:8] == BOOT + ['M2:PASS', 'QSIM:MANIFEST:PASS'], 'Qsim marker prefix/count')
    cases = ['BELL4096', 'SIX_QUBIT', 'CANCEL', 'DEADLINE', 'RESOURCE_LIMITS', 'PROCESS_ISOLATION']
    need(lines[16:22] == ['QSIM:' + name + ':PASS' for name in cases], 'Qsim cases')
    need(lines[25:] == ['QSIM:CLEANUP:PASS', 'QSIM:PASS', 'M1:PASS'], 'Qsim ending')
    for slot in range(2):
        for j, (name, wanted) in enumerate([('PID', slot + 1), ('CHECKPOINTS', 253), ('USEFUL_ARENA', 7216), ('MAX_WORK_GAP', 448)]):
            need(lines[8 + slot * 4 + j] == f'QSIM:PROCESS_{name}:{wanted:020}', 'Qsim process field')
    for at, name in enumerate(['TIMER_IRQS', 'SYSCALL_CALLS', 'CLEANUP_FRAMES'], 22):
        need(re.fullmatch('QSIM:' + name + ':[0-9]{20}', lines[at]) is not None and 0 < int(lines[at].split(':')[-1]) < 1 << 64, 'Qsim positive native counter')
    inputs = Path(acceptance['inputs_directory'])
    for name, expected in acceptance['input_artifact_sha256'].items():
        read(inputs / name, expected)
    same(bundle / 'build-manifest.json', inputs / 'build-manifest.json')
    same(bundle / 'quantum-build-report.json', inputs / 'build-report.json')
    scalar = load(inputs / 'build-report.json')
    elf = read(inputs / 'qsim-native-probe.elf', scalar['binary_sha256'])
    need(elf in read(run / 'esp/kernel.elf'), 'Qsim exact ELF embedding')
    read(inputs / 'disassembly.stdout.txt', scalar['disassembly_sha256'])
    need(manifest['qsim_user'] == scalar and scalar['scalar_gate'] == {'decoded_instructions': 4096, 'int80_sites': 4, 'extended_registers_found': False}, 'Qsim scalar report binding')
    for name, expected in scalar['source_sha256'].items():
        read(ROOT / name, expected)
    for name in ('serial.txt', 'stderr.txt', 'result.json'):
        same(bundle / name, run / name)
    for name, item in load(bundle / 'artifact-index.json')['files'].items():
        need(len(read(bundle / name, item['sha256'])) == item['bytes'], 'Qsim archive length')
    return {'strict_lines': len(lines), 'user_elf_sha256': sha(elf), 'source_inputs_checked': len(scalar['source_sha256']), 'embedded_exactly': True}


def fnv(raw):
    value = 0xcbf29ce484222325
    for byte in raw:
        value = ((value ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{value:016x}'


def disk_state(raw, prefix):
    """Bounded direct decoder for the two normal journal fixtures in this task."""
    named = prefix == 'M6C'
    state = {'sequence': 0, 'objects': [dict(revision=0, present=False, length=0, digest=fnv(b'')) for _ in range(4)]}
    if named:
        for obj in state['objects']:
            obj.update(name_length=0, name_digest=fnv(b''))
    payloads, ended = [b''] * 4, False
    for slot in range(32):
        record = raw[(4 + slot) * 512:(5 + slot) * 512]
        if record == bytes(512):
            ended = True
            continue
        need(not ended, prefix + ': occupied record after journal end')
        version, header, flags, seq, request, obj, revision, op, length = struct.unpack_from('<HHIQQQQHH', record, 8)
        name_length = struct.unpack_from('<H', record, 52)[0] if named else 0
        need(record[:8] == (b'NOSM6C02' if named else b'NOSM6B01') and (version, header, flags) == (2 if named else 1, 64, 0), 'Journal version/header')
        need(seq == slot + 1 and request > 0 and obj < 4 and length <= 128 and name_length <= 32, 'Journal bounds/order')
        need(op in ((2, 3, 4) if named else (2, 3)), 'Journal operation')
        need(not any(record[54 if named else 52:64]) and not any(record[64 + length:192]) and not any(record[192 + name_length:504]), 'Journal canonical padding')
        need(int(fnv(record[:504] + bytes(8)), 16) == struct.unpack_from('<Q', record, 504)[0], 'Journal checksum')
        payload, name = record[64:64 + length], record[192:192 + name_length]
        prior = state['objects'][obj]
        need(revision == prior['revision'] + 1 and (op not in (3, 4) or prior['present']), 'Journal object revision')
        need(op != 4 or payload == payloads[obj], 'Rename changed contents')
        need(op != 3 or not (payload or name), 'Deleted document retains content/name')
        if named and op != 3:
            text = name.decode('utf-8', 'strict')
            need(bool(text) and text not in ('.', '..') and all(ord(c) >= 32 and not 127 <= ord(c) <= 159 and c not in '/\\' and not 0x202a <= ord(c) <= 0x202e and not 0x2066 <= ord(c) <= 0x2069 for c in text), 'Journal document name')
        current = dict(revision=revision, present=op != 3, length=len(payload), digest=fnv(payload))
        if named:
            current.update(name_length=len(name), name_digest=fnv(name))
        state['objects'][obj], payloads[obj], state['sequence'] = current, payload, seq
    return state


def storage(prefix, bundle):
    sdk = prefix == 'M6C'
    records = []
    for case in ('normal', 'recover'):
        archive = bundle / (case + '-result.json') if sdk else bundle / case / 'result.json'
        feature = ('milestone-' if case == 'normal' else 'test-') + prefix.lower() + ('' if case == 'normal' else '-recover')
        result, run, lines, manifest = vm(archive, prefix + '-' + case, feature, sdk=sdk)
        for name in ('serial.txt', 'stderr.txt', 'build-manifest.json', 'volume-manifest.json'):
            same(bundle / (case + '-' + name) if sdk else bundle / case / name, run / name)
        before, after = read(run / 'test-disk.before.img'), read(run / 'test-disk.after.img')
        need(len(before) == len(after) == 65536, prefix + ': volume extent')
        state = disk_state(after, prefix)
        need(state == result['disk_oracle']['state'] == result['disk_oracle']['observed']['state'], prefix + ': disk versus reported state')
        acks, rejected = ((10 if sdk else 8), 6) if case == 'normal' else (0, 2)
        expected = BOOT + ['M2:PASS'] + (['M6C:READY'] if sdk else []) + [f'{prefix}:SEQUENCE:{state["sequence"]}']
        for number, obj in enumerate(state['objects']):
            fields = [('REVISION', obj['revision']), ('PRESENT', int(obj['present'])), ('LENGTH', obj['length'])]
            if sdk:
                fields.extend([('NAME_LENGTH', obj['name_length']), ('NAME_DIGEST', obj['name_digest'])])
            fields.append(('DIGEST', obj['digest']))
            expected.extend(f'{prefix}:OBJECT{number}:{key}:{value}' for key, value in fields)
        expected.extend([f'{prefix}:ACKS:{acks}', f'{prefix}:REJECTED_NO_IO:{rejected}'])
        if sdk:
            expected.extend([f'M6C:CATALOG_REPLIES:{10 if case == "normal" else 2}', 'M6C:DOMAIN0:REVISION:5', 'M6C:DOMAIN1:REVISION:5'])
            user_manifest = load(bundle / (case + '-sdk-user-manifest.json'))
            build = ROOT / 'build/parallel-followup-sdk-m6c/sdk-artifacts' / case
            same(bundle / (case + '-sdk-user-manifest.json'), build / 'manifest-m6c.json')
            elf = read(build / 'm6c_probe.elf', user_manifest['binary_sha256'])
            need(user_manifest['sdk_runtime'] is True and elf in read(run / 'esp/kernel.elf'), 'SDK exact user-ELF embedding')
            for name, value in user_manifest['module_sha256'].items():
                read(ROOT / name, value)
        expected.extend([f'{prefix}:CASE:{case.upper()}:PASS', f'{prefix}:SURVIVOR_PROGRESS:PASS', f'{prefix}:RESOURCES:PASS', f'{prefix}:PASS', 'M1:PASS'])
        need(lines == expected, prefix + ': exact serial state/order')
        records.append((result, run, before, after, state))
    normal, cold = records
    need(cold[0]['previous_stage'] == str(normal[1]) and cold[0]['volume_path'] == normal[0]['volume_path'], prefix + ': cold run link')
    need(normal[3] == cold[2] == cold[3] == read(Path(normal[0]['volume_path'])), prefix + ': cold disk bytes changed')
    volume = load(normal[1] / 'volume-manifest.json')
    need(volume == load(cold[1] / 'volume-manifest.json') and volume['variant'] == 'normal', prefix + ': volume identity')
    initial = bytearray(random.Random(volume['seed']).randbytes(65536))
    initial[:1024] = bytes(1024)
    label = b'NOS-' + prefix.encode() + b'-TEST-VOLUME\0'
    initial[:len(label)] = label
    initial[2048:18432] = bytes(16384)
    need(bytes(initial) == normal[2] and sha(initial) == volume['initial_sha256'], prefix + ': independently reconstructed initial image')
    need(normal[3][:2048] == initial[:2048] and normal[3][18432:] == initial[18432:], prefix + ': outside-journal canaries')
    return {'prefix': prefix, 'identity': volume['identity'], 'initial_sha256': sha(normal[2]), 'saved_sha256': sha(normal[3]),
            'sequence': normal[4]['sequence'], 'normal_acks': 10 if sdk else 8, 'cold_acks': 0,
            'cold_bytes_exact': True, 'initial_seed_and_canaries_valid': True, 'journal_state_matches_raw_serial': True}


def main():
    output = {'task': 'I12', 'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'scope': 'Independent artifact/serial/disk/pixel crosscheck of existing six VM results; no new VM or suite',
              'passed': False, 'new_vm_invocations': 0}
    try:
        output['before'] = snapshot()
        output['g0'] = g0()
        output['qsim'] = qsim()
        output['storage'] = [storage('M6B', DOCS / 'parallel-followup-root'), storage('M6C', DOCS / 'parallel-followup-sdk-m6c')]
        sdk_index = load(DOCS / 'parallel-followup-sdk-m6c/archive-index.json')
        source(sdk_index, 'source_commit')
        for name, expected in sdk_index['files_sha256'].items():
            read(DOCS / 'parallel-followup-sdk-m6c' / name, expected)
        execution = load(DOCS / 'parallel-followup-root/execution.json')
        source(execution['source'], 'commit')
        for step in execution['steps']:
            source(step['before'], 'commit')
            source(step['after'], 'commit')
            need(step['exit_code'] == 0, 'ROOT execution step failed')
        directed = load(DOCS / 'parallel-followup-sdk-m6c/directed-result.json')
        for check in directed['freeze_checks']:
            source(check, 'head')
        for prefix in ('g0', 'qsim', 'root', 'sdk-m6c'):
            bundle = DOCS / ('parallel-followup-' + prefix)
            for path in sorted(bundle.rglob('*')):
                if path.is_file():
                    read(path)
        need(len(VMS) == len({v['run_directory'] for v in VMS}) == 6, 'Expected six unique actual VM directories')
        for path, expected in list(FILES.items()):
            read(path, expected['sha256'])
        output['after'] = snapshot()
        need(output['after'] == output['before'], 'Source changed during artifact audit')
        output['passed'] = True
    except Exception as error:
        output['error'] = type(error).__name__ + ': ' + str(error)
    output.update(vm_count=len(VMS), vms=VMS, unique_files_rehashed=len(FILES), files=FILES,
                  finished_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    (HERE / 'audit.json').write_text(json.dumps(output, indent=2) + '\n')
    print(json.dumps({key: value for key, value in output.items() if key not in ('files', 'vms')}, indent=2))
    return 0 if output['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
