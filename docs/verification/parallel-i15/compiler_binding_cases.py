"""I15 controlled mutations of copied real G11 rustc records; starts no VM.

Arguments: path containing the gate's tools/ directory, immutable G11 root.
Original G11 records and binaries are only read. Mutations live in a temporary
directory and are intentionally labelled host negative fixtures.
"""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import shutil
import sys
import tempfile

gate_root, root = (Path(value).resolve(strict=True) for value in sys.argv[1:3])
sys.path.insert(0, str(gate_root / 'tools'))
import test_document_session as gate

build = root / 'build/parallel-g11-document'
logs = [row[1] for row in gate.LIBRARIES.values()] + ['build-users-m6c_probe']
originals = {name: (build / 'logs' / (name + '.json')).read_bytes() for name in logs}
cases = []


def dependency(crate, mode, target='catalog_core'):
    name = 'build-users-m6c_probe' if crate == 'bin' else ('build-sdk-runtime' if crate == 'nieuw_os_sdk_runtime' else 'build-document-library-' + crate)
    data = json.loads(originals[name])
    command = data['command']
    expected = target + '=' + str(build / ('users/lib' + target + '.rlib'))
    if mode == 'foreign':
        command[command.index(expected)] = target + '=/foreign/unbound-library.rlib'
    elif mode == 'extra':
        command.extend(['--extern', 'foreign=/foreign/unselected-library.rlib'])
    elif mode == 'missing':
        index = command.index(expected)
        assert command[index - 1] == '--extern'
        del command[index - 1:index + 1]
    elif mode == 'duplicate':
        command.extend(['--extern', expected])
    cases.append((f'{crate}-{mode}-{target}', name, data))


dependency('nieuw_os_sdk_runtime', 'extra')
dependency('catalog_core', 'extra')
dependency('nieuw_os_catalog_client', 'foreign')
dependency('nieuw_os_document_session', 'foreign', 'nieuw_os_catalog_client')
dependency('bin', 'foreign')
dependency('nieuw_os_catalog_client', 'missing', 'nieuw_os_sdk_runtime')
dependency('nieuw_os_document_session', 'duplicate')
name = 'build-document-library-nieuw_os_catalog_client'
abi = {'panic': ('abort', 'unwind'), 'code-model': ('small', 'large'),
       'relocation-model': ('static', 'pic'), 'no-redzone': ('yes', 'no')}
for key, (selected, changed) in abi.items():
    for position in ('before', 'after'):
        for kind, value in (('duplicate', selected), ('conflict', changed)):
            data = json.loads(originals[name])
            extra = ['-C', key + '=' + value]
            if position == 'before':
                data['command'][2:2] = extra
            else:
                data['command'].extend(extra)
            cases.append((f'{position}-{kind}-{key}', name, data))
for index, extra in enumerate((['-Cno-redzone=no'], ['-C=no-redzone=no'],
                                ['--codegen', 'no-redzone=no'], ['--codegen=no-redzone=no'],
                                ['-C', 'no_redzone=no'], ['-C', 'no_redzone=yes'])):
    data = json.loads(originals[name])
    data['command'].extend(extra)
    cases.append((f'alternate-codegen-spelling-{index}', name, data))

result = {'scope': 'Host negative fixtures made from real G11 compiler records; no compilation or VM',
          'gate_sha256': hashlib.sha256((gate_root / 'tools/test_document_session.py').read_bytes()).hexdigest(),
          'original_record_sha256': {name: hashlib.sha256(raw).hexdigest() for name, raw in originals.items()},
          'negative_cases': [], 'native_vm_invocations': 0}
with tempfile.TemporaryDirectory(prefix='nieuw-os-i15-records-') as temporary:
    snapshot = Path(temporary)
    (snapshot / 'logs').mkdir()
    for name in logs:
        for suffix in ('.json', '.stdout.txt', '.stderr.txt'):
            shutil.copyfile(build / 'logs' / (name + suffix), snapshot / 'logs' / (name + suffix))
    gate.compiler_binding(snapshot, build, root)
    result['original_actual_records_accepted'] = True
    for label, name, metadata in cases:
        path = snapshot / 'logs' / (name + '.json')
        encoded = (json.dumps(metadata) + '\n').encode()
        path.write_bytes(encoded)
        row = {'name': label, 'record': name, 'negative_fixture_sha256': hashlib.sha256(encoded).hexdigest()}
        try:
            gate.compiler_binding(snapshot, build, root)
        except ValueError as error:
            row.update(rejected=True, reason=str(error))
        else:
            row['rejected'] = False
        result['negative_cases'].append(row)
        path.write_bytes(originals[name])
result['original_records_unchanged'] = all((build / 'logs' / (name + '.json')).read_bytes() == raw for name, raw in originals.items())
result['rejected_count'] = sum(row['rejected'] for row in result['negative_cases'])
result['negative_case_count'] = len(result['negative_cases'])
print(json.dumps(result, indent=2))
