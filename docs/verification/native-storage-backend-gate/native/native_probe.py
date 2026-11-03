"""Read-only foreign-format admission probes on the unchanged Nieuw OS kernel.

All generated data stays under this script's directory. Foreign images are
copied, never mounted or altered. Only explicitly generated NOSVOL02 controls
are passed writable to QEMU. This does not implement a filesystem driver.
"""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
SOURCE = Path('C:/Users/simon/OneDrive/Documenten/GitHub/nieuw-os')
EXPECTED_HEAD = 'e08fb0e48c1bb0781e98357f588c2f73b6ae8500'
os.environ['PYTHONDONTWRITEBYTECODE'] = '1'
sys.dont_write_bytecode = True
METADATA_TIMEOUT = 15


def load_source(source):
    # Resolve configuration before loading helpers from the selected checkout.
    global SOURCE, environment, firmware, source_hash, volume_disk
    global expected_markers, controlled, numbers, check_state
    SOURCE = Path(source).resolve(strict=True)
    source_git_state()
    sys.path.insert(0, str(SOURCE / 'tools'))
    from common import environment, firmware, source_hash
    import volume_disk
    from run_volume import expected_markers, controlled, numbers, check_state


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def save(path, value):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')


def local(path):
    path = Path(path).resolve()
    path.relative_to(ROOT)
    return path


def source_git_state():
    head = subprocess.check_output(['git', '-C', str(SOURCE), 'rev-parse', 'HEAD'], text=True, timeout=METADATA_TIMEOUT).strip()
    if head != EXPECTED_HEAD:
        raise RuntimeError(f'source changed: {head}')
    status = subprocess.check_output(
        ['git', '-C', str(SOURCE), 'status', '--short'], text=True, timeout=METADATA_TIMEOUT).splitlines()
    if status:
        raise RuntimeError(f'source worktree is not clean: {status}')
    return head, status


def source_state():
    head, status = source_git_state()
    return {'head': head, 'source_sha256': source_hash(), 'git_status': status}


def build_environment():
    env = environment()
    # Prevent common.environment() in child scripts from replacing our private
    # Cargo cache with the machine-wide project cache. Rustup remains read-only.
    env.update(CARGO_HOME=str(ROOT / 'cargo-cache'),
               LOCALAPPDATA=str(ROOT / 'process-local-appdata'),
               PYTHONDONTWRITEBYTECODE='1', RUSTUP_AUTO_INSTALL='0')
    return env


def fetch():
    env = build_environment()
    destination = ROOT / 'dependency-fetch'
    destination.mkdir(parents=True, exist_ok=True)
    for label, manifest in [('root', SOURCE / 'Cargo.toml'), ('sdk', SOURCE / 'sdk/Cargo.toml')]:
        lock = manifest.parent / 'Cargo.lock'
        before = sha(lock)
        command = [shutil.which('cargo', path=env['PATH']), '+1.90.0', 'fetch', '--locked', '--manifest-path', str(manifest)]
        result = subprocess.run(command, cwd=SOURCE, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=300)
        (destination / f'{label}.stdout.txt').write_text(result.stdout, encoding='utf-8')
        (destination / f'{label}.stderr.txt').write_text(result.stderr, encoding='utf-8')
        save(destination / f'{label}.json', {'command': command, 'returncode': result.returncode,
            'lock_sha256_before': before, 'lock_sha256_after': sha(lock), 'cargo_home': env['CARGO_HOME']})
        print(f'fetch {label}: exit={result.returncode}, unchanged_lock={before == sha(lock)}', flush=True)
        if result.returncode or before != sha(lock):
            raise RuntimeError('locked dependency fetch failed or lock changed')
        metadata_command = [command[0], '+1.90.0', 'metadata', '--offline', '--locked', '--format-version', '1', '--manifest-path', str(manifest)]
        metadata = subprocess.run(metadata_command, cwd=SOURCE, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=60)
        if metadata.returncode == 0:
            packages = json.loads(metadata.stdout)['packages']
            save(destination / f'{label}-license-provenance.json', [{'name': p['name'], 'version': p['version'],
                 'license': p.get('license'), 'source': p.get('source'), 'manifest_path': p['manifest_path']}
                 for p in packages if p.get('source')])


def build(profile):
    destination = local(ROOT / 'builds' / profile)
    destination.mkdir(parents=True, exist_ok=True)
    before_state = source_state()
    env = build_environment()
    env.update(NOS_BUILD_DIR=str(destination), NOS_TARGET_DIR=str(ROOT / 'target'),
               CARGO_TARGET_DIR=str(ROOT / 'target'), PYTHONDONTWRITEBYTECODE='1',
               CARGO_NET_OFFLINE='true', RUSTUP_AUTO_INSTALL='0')
    command = [sys.executable, '-B', str(SOURCE / 'tools/build.py'), '--kernel-feature', profile]
    started = time.monotonic()
    result = subprocess.run(command, cwd=SOURCE, env=env, capture_output=True,
                            text=True, encoding='utf-8', errors='replace', timeout=600)
    (destination / 'build.stdout.txt').write_text(result.stdout, encoding='utf-8')
    (destination / 'build.stderr.txt').write_text(result.stderr, encoding='utf-8')
    after_state = source_state()
    report = {'command': command, 'cwd': str(SOURCE), 'returncode': result.returncode,
              'duration_s': time.monotonic()-started, 'source_before': before_state,
              'source': after_state, 'source_unchanged': before_state == after_state,
              'isolated_build': str(destination), 'isolated_target': str(ROOT / 'target')}
    save(destination / 'build-invocation.json', report)
    print(json.dumps(report), flush=True)
    if result.returncode or not report['source_unchanged']:
        print(result.stderr[-5000:])
        raise RuntimeError('native build failed or source changed; inspect complete saved logs')


def prepare(profile, label):
    if not label or any(ch not in 'abcdefghijklmnopqrstuvwxyz0123456789-_' for ch in label):
        raise ValueError('invalid label')
    directory = local(ROOT / 'runs' / label)
    directory.mkdir(parents=True, exist_ok=False)
    built = ROOT / 'builds' / profile
    manifest = json.loads((built / 'manifest.json').read_text(encoding='utf-8'))
    state = source_state()
    if manifest.get('kernel_feature') != profile or manifest.get('source_sha256') != state['source_sha256']:
        raise ValueError('build does not match current unchanged source/profile')
    shutil.copytree(built / 'esp', directory / 'esp')
    if sha(directory / 'esp/kernel.elf') != manifest['kernel']['sha256']:
        raise ValueError('copied kernel differs from build manifest')
    if sha(directory / 'esp/EFI/BOOT/BOOTX64.EFI') != manifest['loader']['sha256']:
        raise ValueError('copied loader differs from build manifest')
    code, variables = firmware()
    shutil.copy2(code, directory / 'OVMF_CODE.fd')
    shutil.copy2(variables, directory / 'OVMF_VARS.fd')
    save(directory / 'build-manifest.json', manifest)
    return directory, manifest, state


def run(label, image, *, profile='test-volume-recover', readonly=True, control=None, timeout=120):
    unresolved = Path(image)
    if unresolved.is_symlink():
        raise ValueError('requires a single-link regular image file')
    original = unresolved.resolve(strict=True)
    if not original.is_file() or original.stat().st_nlink != 1:
        raise ValueError('requires a single-link regular image file')
    directory, manifest, state = prepare(profile, label)
    original_before = sha(original)
    target = directory / 'data.img'
    shutil.copyfile(original, target)
    before = sha(target)
    if before != original_before:
        raise ValueError('copy hash differs')
    with target.open('rb') as f:
        f.seek(4*512)
        first_table = f.read(34*512)
        second_table = f.read(34*512)
    if len(first_table) != 34*512 or len(second_table) != 34*512:
        raise ValueError('short read of native table regions')
    env = environment()
    qemu = shutil.which('qemu-system-x86_64', path=env['PATH'])
    if not qemu:
        raise RuntimeError('QEMU unavailable')
    args = [qemu, '-machine', 'pc-q35-10.0', '-accel', 'tcg,thread=single', '-cpu', 'qemu64',
            '-smp', '1', '-m', '256M', '-nodefaults', '-display', 'none', '-monitor', 'none',
            '-serial', 'stdio', '-nic', 'none', '-no-reboot',
            '-drive', 'if=pflash,unit=0,format=raw,readonly=on,file=OVMF_CODE.fd',
            '-drive', 'if=pflash,unit=1,format=raw,file=OVMF_VARS.fd',
            '-drive', 'if=virtio,index=0,format=raw,readonly=on,file=fat:esp',
            '-device', 'isa-debug-exit,iobase=0xf4,iosize=0x04',
            '-drive', f'if=none,id=volume,format=raw,cache=writeback,readonly={"on" if readonly else "off"},file=data.img',
            '-device', 'virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-M6E-TEST']
    if not readonly and control != 'normal':
        raise ValueError('only generated normal control may be writable')
    start = time.monotonic()
    started = datetime.now(timezone.utc).isoformat()
    timed_out = False
    try:
        proc = subprocess.run(args, cwd=directory, env=env, capture_output=True, timeout=timeout)
        code, out, err = proc.returncode, proc.stdout, proc.stderr
    except subprocess.TimeoutExpired as exc:
        timed_out = True
        code, out, err = None, exc.stdout or b'', exc.stderr or b''
    (directory / 'serial.txt').write_bytes(out)
    (directory / 'stderr.txt').write_bytes(err)
    serial = out.decode('utf-8', 'replace')
    after, original_after = sha(target), sha(original)
    report = {'schema_version': 1, 'kind': 'native-control' if control else 'native-foreign-admission-probe',
              'label': label, 'started_utc': started, 'duration_s': time.monotonic()-start,
              'timeout_s': timeout, 'timed_out': timed_out, 'raw_qemu_exit': code,
              'command': args, 'cwd': str(directory), 'qemu_version': subprocess.check_output([qemu, '--version'], text=True, timeout=METADATA_TIMEOUT).strip(),
              'source': state, 'source_unchanged': state == source_state(), 'profile': profile,
              'data_disk_read_only': readonly, 'source_image': str(original),
              'source_image_size': original.stat().st_size, 'source_image_sha256_before': original_before,
              'source_image_sha256_after': original_after, 'probe_image_sha256_before': before,
              'probe_image_sha256_after': after, 'source_image_unchanged': original_before == original_after,
              'probe_image_unchanged': before == after,
              'kernel_sha256': sha(directory / 'esp/kernel.elf'),
              'loader_sha256': sha(directory / 'esp/EFI/BOOT/BOOTX64.EFI'),
              'firmware_code_sha256': sha(directory / 'OVMF_CODE.fd'),
              'native_scope_first_sector': 4, 'native_scope_sector_count': 580,
              'first_table_all_zero': not any(first_table), 'second_table_all_zero': not any(second_table),
              'boot_markers_observed': [m for m in ('BOOT:EBS_OK','KERNEL:ENTRY','BOOTINFO:OK','VOLUME:MANIFEST:PASS') if m in serial],
              'native_candidate_filesystem_support_demonstrated': False,
              'native_foreign_content_recovery_demonstrated': False,
              'serial_path': str(directory / 'serial.txt'), 'stderr_path': str(directory / 'stderr.txt')}
    if control:
        try:
            if timed_out or code != 33:
                raise ValueError(f'control exit {code}, timeout {timed_out}')
            if controlled(serial) != list(expected_markers(control)):
                raise ValueError('control marker sequence mismatch')
            oracle = volume_disk.analyse(target.read_bytes())
            check_state(control, oracle, numbers(serial))
            if readonly and before != after:
                raise ValueError('readonly control mutated')
            report['control_passed'] = True
            report['oracle_generation'] = oracle['generation']
            report['oracle_digests'] = oracle['digests']
        except Exception as exc:
            report['control_passed'] = False
            report['control_error'] = str(exc)
    else:
        if 'VOLUME:FAIL:SERVICE_STARTUP' in serial and 'VOLUME:SERVICE_ERROR:' in serial:
            report['observation'] = 'native NOSVOL02 service rejected candidate during startup; no Btrfs/ZFS parser was invoked'
        elif 'VOLUME:FAIL:APP_RESULT' in serial:
            report['observation'] = 'native recovery fixture reached application checks and failed; foreign content was not recovered; startup admission requires joint source and header analysis'
        else:
            report['observation'] = 'unclassified or incomplete native run; inspect serial and stderr'
    save(directory / 'result.json', report)
    print(json.dumps(report, ensure_ascii=False), flush=True)
    return report, target


def btrfs_admission_finding():
    directory = ROOT / 'runs/btrfs-readonly'
    try:
        report = json.loads((directory / 'result.json').read_text(encoding='utf-8'))
        serial = (directory / 'serial.txt').read_text(encoding='utf-8')
    except (OSError, ValueError):
        return None
    if not isinstance(report, dict):
        return None
    source = report.get('source')
    if (not isinstance(source, dict) or source.get('head') != EXPECTED_HEAD
            or source.get('git_status') != [] or report.get('source_unchanged') is not True
            or report.get('timed_out') is not False or report.get('raw_qemu_exit') != 35
            or report.get('kind') != 'native-foreign-admission-probe'
            or report.get('label') != 'btrfs-readonly' or report.get('profile') != 'test-volume-recover'
            or report.get('data_disk_read_only') is not True
            or report.get('source_image_unchanged') is not True
            or report.get('probe_image_unchanged') is not True
            or report.get('first_table_all_zero') is not True
            or report.get('second_table_all_zero') is not True):
        return None
    hashes = [report.get(key) for key in ('source_image_sha256_before', 'source_image_sha256_after',
                                          'probe_image_sha256_before', 'probe_image_sha256_after')]
    if not all(isinstance(value, str) and re.fullmatch(r'[0-9a-f]{64}', value) for value in hashes) or len(set(hashes)) != 1:
        return None
    steps = list(re.finditer(r'^VOLUME:APP_STEP:([^\r\n]*)', serial, re.MULTILINE))
    if not steps or not re.fullmatch(r'0*20[ \t]*', steps[-1].group(1)):
        return None
    if not re.search(r'^VOLUME:FAIL:APP_RESULT[ \t]*$', serial[steps[-1].end():], re.MULTILINE):
        return None
    step = int(steps[-1].group(1).strip().lstrip('0'))
    return {'candidate': 'btrfs-readonly', 'observation': f'Two zero-filled table regions were accepted by the existing NOSVOL02 recovery logic as an empty table; application failed at first expected document STAT (step {step}).',
        'evidence_level': 'Observed native app failure and header bytes, combined with unchanged recovery source; no Btrfs parser or contents were read.',
        'source_references': ['kernel/src/m2/m3/volume.rs:19-20', 'services/volume/src/table.rs:828-846', 'sdk/examples/volume-docs/src/service.rs:151-157', 'sdk/examples/volume-docs/src/app.rs:153-156', 'sdk/examples/volume-docs/src/app.rs:649-651'],
        'impact_boundary': 'Trusted synthetic fixture scope; no product autoformatter or physical-disk write was exercised. Must not reuse this blank-volume admission as an external-disk mount policy.'}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=['fetch','build','control','foreign','summarize'])
    parser.add_argument('--source', type=Path, default=SOURCE, help='checkout containing the source tools')
    parser.add_argument('--profile', choices=['native-volume','test-volume-recover'])
    parser.add_argument('--label')
    parser.add_argument('--image', type=Path)
    parser.add_argument('--timeout', type=int, default=120)
    args = parser.parse_args()
    load_source(args.source)
    ROOT.mkdir(parents=True, exist_ok=True)
    if args.action == 'fetch':
        fetch()
    elif args.action == 'build':
        build(args.profile)
    elif args.action == 'control':
        seed_dir = local(ROOT / 'control-seed')
        seed_dir.mkdir(parents=True, exist_ok=False)
        volume_disk.create_image(seed_dir / 'volume.img', 'empty')
        normal, image = run('control-normal', seed_dir / 'volume.img', profile='native-volume', readonly=False, control='normal', timeout=args.timeout)
        if not normal['control_passed']:
            raise RuntimeError('normal control failed')
        recovered, _ = run('control-recover', image, control='recover', timeout=args.timeout)
        if not recovered['control_passed']:
            raise RuntimeError('recovery control failed')
    elif args.action == 'foreign':
        run(args.label, args.image, timeout=args.timeout)
    else:
        reports = [json.loads(p.read_text(encoding='utf-8')) for p in sorted((ROOT / 'runs').glob('*/result.json'))]
        for report in reports:
            report['native_candidate_filesystem_support_demonstrated'] = report.pop('native_filesystem_support_demonstrated', False)
        save(ROOT / 'N1-result.json', {'schema_version': 1, 'source': source_state(),
            'native_candidate_filesystem_support_demonstrated': False, 'native_foreign_content_recovery_demonstrated': False,
            'native_backend_adoption_gate_passed': False,
            'native_existing_volume_controls_passed': all(r.get('control_passed') is True for r in reports if r['kind'] == 'native-control') and len([r for r in reports if r['kind'] == 'native-control']) == 2,
            'foreign_format_safe_rejection_gate_passed': False,
            'admission_finding': btrfs_admission_finding(),
            'openzfs_boundary': 'One read-only member of a two-file mirror, created with real OpenZFS ztest/libzpool and checked by zdb by N3. DMU test objectsets, not a ZPL user-file corpus. No native pool import, multiple-vdev operation or ZFS parsing was demonstrated.',
            'runs': reports, 'scope': 'Unchanged own kernel; disposable regular files; foreign disks read-only; no filesystem driver implemented.'})


if __name__ == '__main__':
    main()
