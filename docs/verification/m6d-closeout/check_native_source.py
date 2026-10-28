"""Read-only binding of the reviewed native files; no build or guest execution."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[3]
expected = {
    'kernel/src/m2/m3/m6c.rs': '6a4eed0fde2d2741e26274df0a01c7cf067b96e98959b58df526dcd9e319c3f9',
    'userspace/m6c_probe.rs': '1ba38587737ce9768ec4e54e174cafd5cb2b5df425685030bbcdb9c3da0337b2',
}
refs = ('b739183c48a06cb7798d0be8cc683430152533ac',
        '88e6c04be218b8b09ea4f0a226f6b9719da706a6',
        '7484d87ec639fd8a29be327cad47c2f97739199d',
        '326d2f6efa20411d5bb349b8275bc3e741be7ae0', 'HEAD')
checks = []
for ref in refs:
    for name, digest in expected.items():
        command = ['git', 'show', f'{ref}:{name}']
        result = subprocess.run(command, cwd=root, capture_output=True, check=True)
        actual = hashlib.sha256(result.stdout).hexdigest()
        if actual != digest:
            raise AssertionError(f'Native source changed: {ref}:{name}')
        checks.append(dict(command=command, exit_status=result.returncode, sha256=actual))
for name, digest in expected.items():
    actual = hashlib.sha256((root/name).read_bytes()).hexdigest()
    if actual != digest:
        raise AssertionError(f'Checkout bytes changed: {name}')
    checks.append(dict(checkout_path=name, sha256=actual))
print(json.dumps(dict(command=[sys.executable, '-X', 'utf8', '-B',
    'docs/verification/m6d-closeout/check_native_source.py'], cwd=str(root),
    passed=True, checks=checks,
    boundary='Source-byte binding only; no build, guest execution or human audit.'), indent=2))
