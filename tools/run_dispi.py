#!/usr/bin/env python3
"""H03 native Bochs DISPI mode-set probe: private QEMU, QMP screendumps, exact pixels.

The positive case boots the paired native-dispi-probe kernel on QEMU stdvga
(1234:1111 class 0x0300). For each allowlisted mode the runner takes a real
QMP screendump, checks its dimensions and every pixel against the host-computed
pattern, then releases the next mode with a real PS/2 `n` key. The negative
case boots with `bochs-display` (1234:1111 class 0x0380, no VGA ports) and
requires the kernel's fail-closed DISPI:REFUSED path before any mode change.
QEMU-only evidence; not physical display hardware or a display driver service.
"""
from __future__ import annotations
import argparse
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import time
import uuid
from common import ROOT, BUILD, QEMU_MACHINES, environment, firmware, qemu_machine, save_json, sha256, source_hash
from run_graphics import Qmp, ppm

MODES = ((640, 480), (800, 600), (1024, 768), (1280, 720))
GOP = (1280, 720)
CASES = {'positive': 'VGA,vgamem_mb=16', 'negative': 'bochs-display'}
PREFIXES = ('BOOT:EBS_', 'KERNEL:ENTRY', 'BOOTINFO:', 'KS-', 'DISPI', 'G0:', 'M1:', 'M2:')
# Closed grammar of kernel/src/dispi.rs `DISPI:REFUSED:<TOKEN>` lines, in bind() order.
REFUSALS = ('GOP', 'PCI_SCAN', 'DEVICE', 'PCI_BAR', 'BAR0', 'PCI_COMMAND', 'DECODE', 'ID', 'EXTENT')


def pattern(mode, x, y):
    """Host oracle of kernel/src/dispi.rs::pattern as an (r, g, b) triple."""
    return ((x*3 + mode*0x40) & 0xff, (y*5 + mode*0x25) & 0xff,
            ((x >> 8)*0x40 + (y >> 8)*0x10 + (x ^ y)) & 0xff)


def expected_pixels(mode, width, height):
    row = bytearray()
    out = bytearray()
    for y in range(height):
        row.clear()
        for x in range(width):
            row.extend(pattern(mode, x, y))
        out.extend(row)
    return bytes(out)


def check_screen(raw, mode):
    """Exact dimension and pixel comparison of one QEMU P6 screendump."""
    width, height = MODES[mode]
    try:
        got_width, got_height, pixels = ppm(raw)
    except ValueError as error:
        return dict(dimensions=False, pixels=False, error=str(error))
    result = dict(width=got_width, height=got_height,
                  dimensions=(got_width, got_height) == (width, height), pixels=False)
    if not result['dimensions']:
        return result
    expected = expected_pixels(mode, width, height)
    if pixels == expected:
        result['pixels'] = True
        return result
    mismatches = sum(1 for at in range(0, len(expected), 3) if pixels[at:at+3] != expected[at:at+3])
    first = next(at for at in range(0, len(expected), 3) if pixels[at:at+3] != expected[at:at+3])
    result.update(mismatched_pixels=mismatches,
                  first_mismatch=dict(x=(first//3) % width, y=(first//3)//width,
                                      got=list(pixels[first:first+3]),
                                      expected=list(expected[first:first+3])))
    return result


def _hex(line, label):
    match = re.fullmatch(re.escape(label)+r'([0-9a-f]{16})', line)
    return None if match is None else int(match.group(1), 16)


def validate_serial(stdout, case):
    """Exact controlled marker sequence and decoded device facts for one case."""
    lines = [line for line in stdout.replace('\r', '').splitlines()
             if line.startswith(PREFIXES)]
    head = ['BOOT:EBS_OK', 'KERNEL:ENTRY', 'BOOTINFO:OK', 'KS-01:PASS', 'KS-02:PASS', 'KS-03:PASS']
    if lines[:6] != head or len(lines) < 8:
        return False, 'boot prefix'
    base = _hex(lines[6], 'DISPI:GOP_BASE:')
    size = _hex(lines[7], 'DISPI:GOP_BYTES:')
    if base is None or size is None or size < GOP[0]*GOP[1]*4:
        return False, 'GOP descriptor lines'
    rest = lines[8:]
    refused = [line.removeprefix('DISPI:REFUSED:') for line in rest if line.startswith('DISPI:REFUSED:')]
    if any(token not in REFUSALS for token in refused):
        return False, 'unknown refusal token'
    if case == 'negative':
        if rest != ['DISPI:REFUSED:DEVICE', 'M1:FAIL:DISPI_REFUSED']:
            return False, 'negative must refuse the device before any port or mode access'
        return True, None
    if refused:
        return False, 'refused: '+refused[0]
    if len(rest) != 4 + 2*len(MODES) + 2:
        return False, 'positive line count'
    bar0 = _hex(rest[0], 'DISPI:BAR0_RAW:')
    command = _hex(rest[1], 'DISPI:COMMAND:')
    ident = _hex(rest[2], 'DISPI:ID:')
    vram = _hex(rest[3], 'DISPI:VRAM:')
    if bar0 is None or bar0 & 7 or bar0 & ~0xf != base:
        return False, 'BAR0 is not a 32-bit memory BAR at the GOP base'
    if command is None or command > 0xffff or command & 3 != 3:
        return False, 'PCI command register lacks I/O or memory decode'
    if ident is None or not 0xb0c0 <= ident <= 0xb0c5:
        return False, 'DISPI ID'
    if vram is None or any(w*h*4 > min(vram, size) for w, h in MODES):
        return False, 'VRAM or GOP extent'
    for index, (width, height) in enumerate(MODES):
        readback, marker = rest[4+2*index:6+2*index]
        match = re.fullmatch(r'DISPI:READBACK:'+':'.join([r'([0-9]{1,5})']*8), readback)
        if match is None:
            return False, 'readback format'
        values = list(map(int, match.groups()))
        if values[:5] != [width, height, 32, 0x41, width] or values[5] < height or values[6:] != [0, 0]:
            return False, f'readback mismatch for {width}x{height}'
        if marker != f'DISPI:MODE:{width}x{height}:SET':
            return False, f'mode marker for {width}x{height}'
    if rest[-2:] != ['DISPI:PASS', 'M1:PASS']:
        return False, 'end markers'
    return True, None


def run_dispi(case, *, build_directory=BUILD, run_root=None, machine=None):
    if case not in CASES:
        raise ValueError('Unknown DISPI case')
    machine = qemu_machine(machine)
    build_directory = Path(build_directory).resolve(strict=True)
    run_root = (build_directory/'run') if run_root is None else Path(run_root).resolve()
    frozen = source_hash()
    manifest = json.loads((build_directory/'manifest.json').read_text())
    if (manifest.get('kernel_feature') != 'native-dispi-probe'
            or manifest.get('loader_feature') != 'desktop-mode'
            or manifest.get('source_sha256') != frozen):
        raise ValueError('Build the paired native-dispi-probe profile on the current source first')
    directory = run_root/(f'dispi-{case}-'+uuid.uuid4().hex[:10])
    if len(os.fsencode(directory/'qmp.sock')) >= 104:
        raise ValueError('Private QMP socket path too long; choose a short --run-root')
    directory.mkdir(parents=True)
    shutil.copytree(build_directory/'esp', directory/'esp')
    if (sha256(directory/'esp/kernel.elf') != manifest['kernel']['sha256']
            or sha256(directory/'esp/EFI/BOOT/BOOTX64.EFI') != manifest['loader']['sha256']):
        raise ValueError('DISPI build artifacts changed')
    save_json(directory/'build-manifest.json', manifest)
    code, variables = firmware()
    for source, name in ((code, 'OVMF_CODE.fd'), (variables, 'OVMF_VARS.fd')):
        shutil.copy2(source, directory/name)
    env = environment()
    qemu = shutil.which('qemu-system-x86_64', path=env['PATH'])
    if qemu is None:
        raise RuntimeError('Missing QEMU')
    arguments = [qemu, '-machine', machine, '-accel', 'tcg,thread=single', '-cpu', 'qemu64',
        '-smp', '1', '-m', '256M', '-nodefaults', '-display', 'none', '-monitor', 'none',
        '-serial', 'stdio', '-nic', 'none', '-no-reboot', '-device', CASES[case],
        '-qmp', 'unix:qmp.sock,server=on,wait=off',
        '-drive', 'if=pflash,unit=0,format=raw,readonly=on,file=OVMF_CODE.fd',
        '-drive', 'if=pflash,unit=1,format=raw,file=OVMF_VARS.fd',
        '-drive', 'if=virtio,index=0,format=raw,readonly=on,file=fat:esp',
        '-device', 'isa-debug-exit,iobase=0xf4,iosize=0x04']
    start = time.monotonic()
    deadline = start+180
    failure = None
    control = None
    screens = []
    rc = None
    trace = []
    with (directory/'serial.txt').open('wb') as stdout, (directory/'stderr.txt').open('wb') as stderr:
        process = subprocess.Popen(arguments, cwd=directory, env=env, stdout=stdout,
                                   stdin=subprocess.DEVNULL, stderr=stderr, bufsize=0)
        try:
            control = Qmp(directory/'qmp.sock', deadline, process)
            def wait_for(marker):
                while True:
                    text = (directory/'serial.txt').read_bytes().decode('utf-8', 'replace')
                    if marker in text.replace('\r', '').splitlines():
                        return
                    if time.monotonic() >= deadline or process.poll() is not None:
                        raise RuntimeError('DISPI did not reach '+marker)
                    time.sleep(0.02)
            if case == 'positive':
                for index, (width, height) in enumerate(MODES):
                    wait_for(f'DISPI:MODE:{width}x{height}:SET')
                    path = directory/f'mode-{width}x{height}.ppm'
                    control.command('screendump', dict(filename=str(path)))
                    result = check_screen(path.read_bytes(), index)
                    result.update(mode=f'{width}x{height}', sha256=sha256(path))
                    screens.append(result)
                    control.command('send-key', {'keys': [{'type': 'qcode', 'data': 'n'}], 'hold-time': 30})
            rc = process.wait(timeout=max(0.1, deadline-time.monotonic()))
        except Exception as error:
            failure = f'{type(error).__name__}: {error}'
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=10)
            if rc is None:
                rc = process.returncode
            if control is not None:
                trace = control.trace
                control.close()
    text = (directory/'serial.txt').read_bytes().decode('utf-8', 'replace')
    sequence_ok, sequence_error = validate_serial(text, case)
    checks = dict(exact_serial_sequence=sequence_ok,
                  raw_exit=rc == (33 if case == 'positive' else 35),
                  source_unchanged=source_hash() == frozen)
    if case == 'positive':
        checks['every_mode_screendumped'] = len(screens) == len(MODES)
        checks['screen_dimensions'] = checks['every_mode_screendumped'] and all(s['dimensions'] for s in screens)
        checks['screen_pixels_exact'] = checks['every_mode_screendumped'] and all(s['pixels'] for s in screens)
    else:
        checks['no_mode_set'] = 'DISPI:MODE:' not in text and 'DISPI:READBACK:' not in text
    save_json(directory/'qmp-trace.json', trace)
    report = dict(scope='QEMU stdvga Bochs DISPI mode set by the own IF0 kernel; not physical hardware',
        case=case, passed=failure is None and all(checks.values()), checks=checks, error=failure,
        sequence_error=sequence_error, screens=screens, command=arguments, cwd=str(directory),
        machine=machine, raw_qemu_exit=rc, duration_s=time.monotonic()-start,
        source_fingerprint_version=3, source_sha256=frozen,
        base_commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        qemu_version=subprocess.check_output([qemu, '--version'], env=env, text=True).strip(),
        qemu_entry_path=qemu, firmware=dict(code=str(code), code_sha256=sha256(code)),
        run_directory=str(directory), stdout=text,
        artifacts={p.relative_to(directory).as_posix(): sha256(p)
                   for p in directory.rglob('*') if p.is_file() and 'esp' not in p.parts})
    save_json(directory/'result.json', report)
    save_json(build_directory/f'dispi-{case}-result.json', report)
    print(text, end='')
    print(f'DISPI {case} raw={rc}; passed={report["passed"]}; {directory}')
    if failure:
        print(failure)
    if sequence_error:
        print('sequence:', sequence_error)
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--case', choices=sorted(CASES), default='positive')
    parser.add_argument('--build-dir', type=Path, default=BUILD,
                        help='Paired native-dispi-probe build output bound to the current source')
    parser.add_argument('--run-root', type=Path, help='Private VM directory parent')
    parser.add_argument('--machine', choices=QEMU_MACHINES,
                        help='Exact machine; defaults to NOS_QEMU_MACHINE')
    args = parser.parse_args()
    raise SystemExit(0 if run_dispi(args.case, build_directory=args.build_dir,
                                    run_root=args.run_root, machine=args.machine)['passed'] else 1)
