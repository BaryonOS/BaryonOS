from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys
import time

root = Path('/workspace/baryon-h02-completion')
sys.path.insert(0, str(root / 'tools'))
from common import source_hash

expected = '2a08f4a33eff63b354821f6892e1be28f7602c9b'
out = root / 'build/h02-combined-2a08f4a3'
out.mkdir(parents=True, exist_ok=True)
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
assert head == expected
fingerprint = source_hash()
report = {'source_commit': head, 'source_fingerprint_version': 3,
          'source_sha256': fingerprint, 'checks': [], 'status': 'running',
          'scope': 'Six existing H02 native cases; no integrated USB-to-UI or physical hardware acceptance'}
path = out / 'checks.json'

def save():
    path.write_text(json.dumps(report, indent=2) + '\n')

save()
cases = [
    ('noop', 'native-xhci-probe', 'tools/run_xhci.py', ['--case', 'noop']),
    ('fault', 'test-xhci-dma-fault', 'tools/run_xhci.py', ['--case', 'fault']),
    ('no-dmar', 'test-xhci-no-dmar', 'tools/run_xhci.py', ['--case', 'no-dmar']),
    ('usb-reports', 'native-usb-hid-probe', 'tools/run_usb_hid.py', ['--case', 'reports']),
    ('usb-readonly', 'test-usb-hid-readonly-fault', 'tools/run_usb_hid.py', ['--case', 'readonly']),
    ('hid-ipc', 'native-hid-input-probe', 'tools/run_hid_input.py',
     ['--vga-rom', '/workspace/.nieuw-os-env/sysroot/usr/share/seabios/vgabios-stdvga.bin']),
]
for name, feature, runner, args in cases:
    env = os.environ.copy()
    env['NOS_QEMU_MACHINE'] = 'pc-q35-8.2'
    env['NOS_BUILD_DIR'] = str(out / name)
    env['NOS_TARGET_DIR'] = str(root / 'target/h02-combined-native')
    env['CARGO_TARGET_DIR'] = str(root / 'target/h02-combined-native')
    for phase, command in [('build', [sys.executable, '-B', 'tools/build.py', '--kernel-feature', feature]),
                           ('run', [sys.executable, '-B', runner, *args])]:
        started = time.monotonic()
        result = subprocess.run(command, cwd=root, env=env, stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT, timeout=240)
        log = out / f'{name}-{phase}.log'
        log.write_bytes(result.stdout)
        check = {'name': name + '-' + phase, 'command': command,
                 'environment': {k: env[k] for k in ['NOS_QEMU_MACHINE', 'NOS_BUILD_DIR', 'NOS_TARGET_DIR', 'CARGO_TARGET_DIR']},
                 'exit_code': result.returncode, 'duration_seconds': round(time.monotonic()-started,3),
                 'log': str(log.relative_to(root)), 'log_sha256': hashlib.sha256(result.stdout).hexdigest()}
        report['checks'].append(check)
        save()
        print(json.dumps(check), flush=True)
        if result.returncode != 0:
            break
report['observed_source_sha256'] = source_hash()
report['source_unchanged'] = report['observed_source_sha256'] == fingerprint
report['status'] = 'completed'
report['passed'] = len(report['checks']) == 12 and all(c['exit_code'] == 0 for c in report['checks']) and report['source_unchanged']
save()
raise SystemExit(0 if report['passed'] else 1)
