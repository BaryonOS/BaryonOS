"""Classify differences in the two preserved PE loaders without changing them."""
from __future__ import annotations

import datetime
import hashlib
import json
from pathlib import Path
import struct
import subprocess

ROOT = Path('/workspace/scratch/debaa3b6a81c/nieuw-os')
BUILD = ROOT / 'build/parallel-followup-qsim'
PATHS = {
    'previous': ROOT / 'build/parallel-final-qsim/esp/EFI/BOOT/BOOTX64.EFI',
    'current': BUILD / 'esp/EFI/BOOT/BOOTX64.EFI',
}


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def inspect(raw):
    pe = struct.unpack_from('<I', raw, 0x3c)[0]
    assert raw[:2] == b'MZ' and raw[pe:pe + 4] == b'PE\0\0'
    count = struct.unpack_from('<H', raw, pe + 6)[0]
    optional_size = struct.unpack_from('<H', raw, pe + 20)[0]
    optional = pe + 24
    assert struct.unpack_from('<H', raw, optional)[0] == 0x20b
    table = optional + optional_size
    sections = []
    for index in range(count):
        offset = table + index * 40
        name = raw[offset:offset + 8].split(b'\0', 1)[0].decode('ascii')
        virtual_size, address, size, pointer = struct.unpack_from('<IIII', raw, offset + 8)
        flags = struct.unpack_from('<I', raw, offset + 36)[0]
        sections.append({'name': name, 'virtual_size': virtual_size, 'rva': address,
                         'raw_size': size, 'raw_offset': pointer, 'flags': flags,
                         'raw_sha256': digest(raw[pointer:pointer + size])})
    debug_rva, debug_size = struct.unpack_from('<II', raw, optional + 112 + 6 * 8)
    assert debug_size == 28
    mapped = [s['raw_offset'] + debug_rva - s['rva'] for s in sections
              if s['rva'] <= debug_rva and debug_rva + debug_size <= s['rva'] + s['raw_size']]
    assert len(mapped) == 1
    debug = mapped[0]
    characteristics, timestamp, major, minor, kind, size, address, pointer = struct.unpack_from('<IIHHIIII', raw, debug)
    assert kind == 2 and raw[pointer:pointer + 4] == b'RSDS'
    metadata = [
        {'kind': 'COFF TimeDateStamp', 'offset': pe + 8, 'size': 4},
        {'kind': 'IMAGE_DEBUG_DIRECTORY TimeDateStamp', 'offset': debug + 4, 'size': 4},
        {'kind': 'CodeView RSDS GUID', 'offset': pointer + 4, 'size': 16},
    ]
    normalized = bytearray(raw)
    for item in metadata:
        item['hex'] = raw[item['offset']:item['offset'] + item['size']].hex()
        normalized[item['offset']:item['offset'] + item['size']] = bytes(item['size'])
    coff_time = struct.unpack_from('<I', raw, pe + 8)[0]
    return {
        'bytes': len(raw), 'sha256': digest(raw), 'pe_header_offset': pe,
        'coff_timestamp': coff_time,
        'coff_timestamp_utc': datetime.datetime.fromtimestamp(coff_time, datetime.timezone.utc).isoformat(),
        'debug_timestamp': timestamp, 'debug_type': kind,
        'codeview_signature': raw[pointer:pointer + 4].decode('ascii'),
        'codeview_guid_hex': raw[pointer + 4:pointer + 20].hex(),
        'codeview_age': struct.unpack_from('<I', raw, pointer + 20)[0],
        'codeview_pdb': raw[pointer + 24:pointer + size].split(b'\0', 1)[0].decode('ascii'),
        'metadata': metadata, 'sections': sections,
        'in_memory_metadata_zeroed_sha256': digest(normalized),
    }


raws = {key: path.read_bytes() for key, path in PATHS.items()}
parsed = {key: inspect(raw) for key, raw in raws.items()}
assert len(raws['previous']) == len(raws['current'])
offsets = [index for index, pair in enumerate(zip(raws['previous'], raws['current'])) if pair[0] != pair[1]]
assert [(i['offset'], i['size']) for i in parsed['previous']['metadata']] == [(i['offset'], i['size']) for i in parsed['current']['metadata']]
allowed = {i for item in parsed['current']['metadata'] for i in range(item['offset'], item['offset'] + item['size'])}
assert set(offsets) <= allowed
assert parsed['previous']['in_memory_metadata_zeroed_sha256'] == parsed['current']['in_memory_metadata_zeroed_sha256']
assert parsed['previous']['codeview_pdb'] == parsed['current']['codeview_pdb']
assert parsed['previous']['codeview_age'] == parsed['current']['codeview_age']
executables = {}
for old, new in zip(parsed['previous']['sections'], parsed['current']['sections']):
    assert old['name'] == new['name'] and old['flags'] == new['flags']
    if old['flags'] & 0x20000000:
        assert old['raw_sha256'] == new['raw_sha256']
        executables[old['name']] = old['raw_sha256']
commands = {}
for name, path in PATHS.items():
    command = ['objdump', '-x', str(path)]
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=True)
    commands[name] = {'command': command, 'returncode': result.returncode,
                      'stdout': result.stdout, 'stderr': result.stderr}
report = {'scope': 'direct comparison of preserved PE artifacts; no build or VM; no file normalization',
          'previous_source_commit': '3e41bbf93822f7044c6020259f4a752925e1966d',
          'current_source_commit': 'dbd35b04d742b18d66b40fdd65d8a9b900656cac',
          'paths': {key: str(path) for key, path in PATHS.items()},
          'parsed': parsed, 'changed_byte_count': len(offsets), 'changed_offsets': offsets,
          'changes_only_coff_debug_timestamps_and_codeview_guid': True,
          'all_other_file_bytes_identical': True, 'executable_sections_sha256': executables,
          'artifacts_unchanged_after_inspection': all(digest(path.read_bytes()) == parsed[key]['sha256'] for key, path in PATHS.items()),
          'independent_objdump_headers': commands,
          'explanation': 'Only the build-time COFF/debug timestamps and CodeView RSDS build identifier differ. The executable .text section and every other byte outside those metadata fields are identical.'}
assert report['artifacts_unchanged_after_inspection']
(BUILD / 'loader-comparison.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({key: report[key] for key in ('changed_byte_count', 'changed_offsets', 'changes_only_coff_debug_timestamps_and_codeview_guid', 'all_other_file_bytes_identical', 'artifacts_unchanged_after_inspection')}, indent=2))
