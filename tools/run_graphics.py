#!/usr/bin/env python3
"""G0 native hardware probe: private QEMU, QMP keys and real framebuffer bytes.

No storage volume or guest network. Screenshots come from QEMU, not a host UI.
The initial adapter is Linux/Unix-only, using a private local QMP Unix socket.
"""
from __future__ import annotations
import argparse
import json
import os
import re
from pathlib import Path
import shutil
import socket
import subprocess
import time
import uuid
from common import ROOT, BUILD, environment, firmware, save_json, sha256, source_hash


def ppm(data):
    """Decode the bounded P6 format emitted by QEMU screendump, without libraries."""
    if len(data) > 1920*1080*3 + 128:
        raise ValueError('Framebuffer screenshot exceeds G0 bound')
    magic, dimensions, maximum, pixels = data.split(b'\n', 3)
    width, height = map(int, dimensions.split())
    if (magic != b'P6' or maximum != b'255' or not 1 <= width <= 1920
            or not 1 <= height <= 1080 or len(pixels) != width*height*3):
        raise ValueError('Not a complete bounded QEMU P6 screenshot')
    return width, height, pixels


def changed_bounds(before, after):
    width, height, old = ppm(before)
    other_width, other_height, new = ppm(after)
    if (width, height) != (other_width, other_height):
        raise ValueError('Screenshot geometry changed')
    left, top, right, bottom = width, height, -1, -1
    for at in range(0, len(old), 3):
        if old[at:at+3] != new[at:at+3]:
            pixel = at//3
            x, y = pixel % width, pixel // width
            left, top, right, bottom = min(left,x), min(top,y), max(right,x), max(bottom,y)
    return None if right < 0 else (left,top,right+1,bottom+1)


def valid_sequence(stdout, screenshot_width, screenshot_height):
    lines = stdout.splitlines()
    dimensions = [line for line in lines if line.startswith('G0:FRAMEBUFFER:')]
    if len(dimensions) != 1:
        return False
    match = re.fullmatch(r'G0:FRAMEBUFFER:([0-9]{1,4}):([0-9]{1,4}):([0-9]{1,4}):([01])', dimensions[0])
    if not match:
        return False
    width, height, stride, _ = map(int, match.groups())
    if (width,height) != (screenshot_width,screenshot_height) or not width <= stride <= 4096:
        return False
    expected = ['BOOT:EBS_OK','KERNEL:ENTRY','BOOTINFO:OK','KS-01:PASS','KS-02:PASS','KS-03:PASS',
                dimensions[0], 'G0:MAPPINGS:PASS','G0:READY','G0:INPUT:1','G0:INPUT:2',
                'G0:INPUT:1','G0:INPUT:2','G0:PASS','M1:PASS']
    controlled = [line for line in lines if line.startswith(('BOOT:EBS_','KERNEL:ENTRY','BOOTINFO:',
        'KS-','G0:','M1:','M2:','M3:','M4:','M5:','M6'))]
    return controlled == expected


class Qmp:
    def __init__(self, path, deadline, process):
        self.socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        while True:
            try:
                self.socket.connect(str(path))
                break
            except (FileNotFoundError, ConnectionRefusedError):
                if time.monotonic() >= deadline or process.poll() is not None:
                    self.socket.close()
                    raise RuntimeError('QEMU did not open its private QMP socket')
                time.sleep(0.02)
        self.socket.settimeout(5)
        self.reader = self.socket.makefile('rb')
        self.sequence = 0
        self.trace = []
        if 'QMP' not in self.read():
            raise ValueError('Missing QMP greeting')
        self.command('qmp_capabilities')

    def read(self):
        line = self.reader.readline(1024*1024)
        if not line or not line.endswith(b'\n'):
            raise RuntimeError('Missing or oversized QMP response')
        return json.loads(line)

    def command(self, name, arguments=None):
        self.sequence += 1
        request = dict(execute=name, id=self.sequence)
        if arguments is not None:
            request['arguments'] = arguments
        self.socket.sendall((json.dumps(request)+'\n').encode())
        while True:
            reply = self.read()
            if reply.get('id') == self.sequence:
                self.trace.append(dict(request=request,response=reply))
                if 'error' in reply:
                    raise RuntimeError('QMP command failed: '+str(reply['error']))
                return reply.get('return')

    def close(self):
        self.reader.close()
        self.socket.close()


def run_graphics():
    if os.name == 'nt':
        raise RuntimeError('G0 QMP probe currently requires Unix sockets on Linux')
    frozen = source_hash()
    manifest = json.loads((BUILD/'manifest.json').read_text())
    if (manifest.get('kernel_feature') != 'graphics-g0' or manifest.get('loader_feature') != 'graphics-g0'
            or manifest.get('source_sha256') != frozen):
        raise ValueError('Build the paired graphics-g0 profile on the current source first')
    directory = BUILD/'run'/('graphics-g0-'+uuid.uuid4().hex[:10])
    directory.mkdir(parents=True)
    shutil.copytree(BUILD/'esp',directory/'esp')
    if (sha256(directory/'esp/kernel.elf') != manifest['kernel']['sha256']
            or sha256(directory/'esp/EFI/BOOT/BOOTX64.EFI') != manifest['loader']['sha256']):
        raise ValueError('G0 build artifacts changed')
    save_json(directory/'build-manifest.json',manifest)
    code, variables = firmware()
    for source, name in ((code,'OVMF_CODE.fd'),(variables,'OVMF_VARS.template.fd'),(variables,'OVMF_VARS.fd')):
        shutil.copy2(source,directory/name)
    env = environment()
    qemu = shutil.which('qemu-system-x86_64',path=env['PATH'])
    if qemu is None:
        raise RuntimeError('Missing QEMU')
    relative = directory.relative_to(ROOT).as_posix()
    arguments = [qemu,'-machine','pc-q35-10.0','-accel','tcg,thread=single','-cpu','qemu64',
        '-smp','1','-m','256M','-nodefaults','-display','none','-monitor','none','-serial','stdio',
        '-nic','none','-no-reboot','-device','VGA,vgamem_mb=16',
        '-qmp',f'unix:{relative}/qmp.sock,server=on,wait=off',
        '-drive',f'if=pflash,unit=0,format=raw,readonly=on,file={relative}/OVMF_CODE.fd',
        '-drive',f'if=pflash,unit=1,format=raw,file={relative}/OVMF_VARS.fd',
        '-drive',f'if=virtio,index=0,format=raw,readonly=on,file=fat:{relative}/esp',
        '-device','isa-debug-exit,iobase=0xf4,iosize=0x04']
    start = time.monotonic()
    deadline = start+60
    failure = None
    control = None
    checks = {}
    screen_size = (0,0)
    rc = None
    trace = []
    with (directory/'serial.txt').open('wb') as stdout, (directory/'stderr.txt').open('wb') as stderr:
        process = subprocess.Popen(arguments,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
        try:
            control = Qmp(directory/'qmp.sock',deadline,process)
            def wait_for(marker, count=1):
                while True:
                    text = (directory/'serial.txt').read_bytes().decode('utf-8','replace')
                    if text.splitlines().count(marker) >= count:
                        return
                    if time.monotonic() >= deadline or process.poll() is not None:
                        raise RuntimeError('G0 did not reach '+marker)
                    time.sleep(0.02)
            def screenshot(name):
                path = directory/(name+'.ppm')
                control.command('screendump',dict(filename=str(path)))
                raw = path.read_bytes()
                ppm(raw)
                return raw
            def key(name):
                control.command('send-key',{'keys':[{'type':'qcode','data':name}],'hold-time':30})
                # Let the key break reach the bounded PS/2 decoder before the
                # next synthetic make. All waits share the same 60s deadline.
                time.sleep(0.05)
            wait_for('G0:READY')
            initial = screenshot('initial')
            screen_size = ppm(initial)[:2]
            key('a'); wait_for('G0:INPUT:1')
            first = screenshot('a')
            key('b'); wait_for('G0:INPUT:2')
            key('backspace'); wait_for('G0:INPUT:1',2)
            erased = screenshot('after-backspace')
            key('c'); wait_for('G0:INPUT:2',2)
            final = screenshot('typed')
            bounds = changed_bounds(initial,final)
            checks['real_key_changes_visible_pixels'] = bounds is not None
            checks['only_input_row_changes'] = (bounds is not None and bounds[0]>=48
                and bounds[1]>=192 and bounds[2]<=80 and bounds[3]<=212)
            checks['backspace_restores_previous_frame_exactly'] = first==erased
            checks['nonempty_native_scene'] = len(set(ppm(initial)[2]))>8
            key('esc')
            rc = process.wait(timeout=max(0.1,deadline-time.monotonic()))
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
    raw = (directory/'serial.txt').read_bytes()
    text = raw.decode('utf-8','replace')
    lines = text.splitlines()
    checks['real_exit33'] = rc==33
    checks['post_ebs_kernel_m2'] = all(lines.count(marker)==1 for marker in
        ('BOOT:EBS_OK','KERNEL:ENTRY','BOOTINFO:OK','KS-01:PASS','KS-02:PASS','KS-03:PASS'))
    checks['exact_g0_sequence'] = valid_sequence(text,*screen_size)
    checks['no_guest_failure'] = not any(marker in text for marker in ('FAIL','KERNEL_PANIC','M2:PANIC_LINE:'))
    checks['source_unchanged'] = source_hash()==frozen
    checks['normal_end_marker'] = lines.count('M1:PASS')==1
    save_json(directory/'qmp-trace.json',trace)
    report = dict(scope='actual own-kernel G0 framebuffer and PS/2 RAM input; not desktop or persistence',
        passed=failure is None and all(checks.values()),checks=checks,error=failure,
        command=arguments,raw_qemu_exit=rc,duration_s=time.monotonic()-start,
        source_fingerprint_version=3,source_sha256=frozen,
        base_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
        qemu_version=subprocess.check_output([qemu,'--version'],env=env,text=True).strip(),
        qemu_entry_path=qemu,qemu_entry_sha256=sha256(Path(qemu)),
        run_directory=str(directory),stdout=text,
        artifacts={p.relative_to(directory).as_posix():sha256(p) for p in directory.rglob('*') if p.is_file()},
        changed_bounds=checks.get('only_input_row_changes') and bounds)
    save_json(directory/'result.json',report)
    save_json(BUILD/'graphics-g0-result.json',report)
    print(text,end='')
    print(f'G0 raw={rc}; passed={report["passed"]}; {directory}')
    if failure: print(failure)
    return report


if __name__=='__main__':
    argparse.ArgumentParser(description=__doc__).parse_args()
    raise SystemExit(0 if run_graphics()['passed'] else 1)
