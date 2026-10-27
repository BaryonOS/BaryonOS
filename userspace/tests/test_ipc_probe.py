#!/usr/bin/env python3
"""Inspect the actual IPC ELF and reject malformed artifacts; no CPL3 execution."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
spec = importlib.util.spec_from_file_location('ipc_build_inspector', ROOT / 'tools/build_users.py')
inspector = importlib.util.module_from_spec(spec)
spec.loader.exec_module(inspector)


def exact_ipc(path):
    plan = inspector.inspect_user(path)
    segments = plan['segments']
    if plan['entry'] != 0x40000000 or len(segments) != 3:
        raise ValueError('IPC entry/segment count changed')
    text, data, readonly = segments
    if not (text['address'] == 0x40000000 and text['flags'] == 5 and 0 < text['memsz'] <= 65536
            and data['address'] == 0x40010000 and data['flags'] == 6
            and data['filesz'] == data['memsz'] == 12288
            and readonly['address'] == 0x40013000 and readonly['flags'] == 4
            and readonly['filesz'] == readonly['memsz'] == 4096):
        raise ValueError('IPC fixture memory layout changed')
    return plan


class IpcArtifact(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.raw = ELF.read_bytes()
        cls.plan = exact_ipc(ELF)
        cls.text, cls.data, cls.readonly = cls.plan['segments']
        cls.code = cls.raw[cls.text['offset']:cls.text['offset'] + cls.text['filesz']]
        cls.phoff = struct.unpack_from('<Q', cls.raw, 32)[0]

    def reject(self, mutations=(), raw=None):
        data = bytearray(self.raw if raw is None else raw)
        for fmt, offset, value in mutations:
            struct.pack_into(fmt, data, offset, value)
        with tempfile.TemporaryDirectory(dir=Path(__file__).parent) as folder:
            path = Path(folder) / 'bad-ipc.elf'
            path.write_bytes(data)
            with self.assertRaises(ValueError):
                exact_ipc(path)

    def test_actual_rx_rw_ro_segments_have_no_writable_executable_page(self):
        self.assertEqual([s['flags'] for s in self.plan['segments']], [5, 6, 4])
        self.assertEqual(self.data['page_end'], self.readonly['page_start'])
        self.assertLessEqual(self.text['page_end'], self.data['page_start'])

    def test_all_control_rows_results_and_message_bytes_start_zero(self):
        self.assertEqual(self.raw[self.data['offset']:self.data['offset'] + 12288], bytes(12288))
        self.assertEqual(self.raw[self.readonly['offset']:self.readonly['offset'] + 4096], bytes(4096))
        # The RO page is therefore also a canonical zero-length Message320 input.
        header = struct.unpack_from('<QQIIQ', self.raw, self.readonly['offset'])
        self.assertEqual(header, (0, 0, 0, 0, 0))

    def test_script_and_result_tables_fit_control_page_without_overlap(self):
        rows_start, rows_end = 0x100, 0x100 + 32 * 80
        results_start, results_end = 0xc00, 0xc00 + 32 * 16
        self.assertGreaterEqual(rows_start, 128)
        self.assertLessEqual(rows_end, results_start)
        self.assertLessEqual(results_end, 4096)
        self.assertEqual(struct.calcsize('<10Q'), 80)
        self.assertEqual(struct.calcsize('<2Q'), 16)

    def test_manifest_matches_actual_binary_source_and_linker(self):
        manifest = json.loads(ELF.with_name('manifest-ipc.json').read_text(encoding='utf-8'))
        for key, path in [('binary_sha256', ELF), ('source_sha256', ROOT / 'userspace/ipc_probe.rs'),
                          ('linker_sha256', ROOT / 'userspace/ipc_linker.ld')]:
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), manifest[key])

    def test_no_tls_dynamic_relocation_or_unexpected_allocatable_section(self):
        shoff = struct.unpack_from('<Q', self.raw, 40)[0]
        size, count, names_index = struct.unpack_from('<HHH', self.raw, 58)
        self.assertEqual(size, 64)
        self.assertGreater(count, 0)
        sections = [struct.unpack_from('<IIQQQQIIQQ', self.raw, shoff + i * size) for i in range(count)]
        names_section = sections[names_index]
        names = self.raw[names_section[4]:names_section[4] + names_section[5]]
        allocated = set()
        for s in sections:
            name = names[s[0]:].split(b'\0', 1)[0].decode('ascii')
            self.assertFalse(s[2] & 0x400, f'TLS: {name}')
            self.assertFalse(s[1] in (4, 6, 9, 11) and s[5], f'dynamic/relocation: {name}')
            if s[2] & 2 and s[5]:
                allocated.add(name)
        self.assertEqual(allocated, {'.text', '.data', '.rodata'})

    def test_actual_binary_contains_limits_before_script_execution(self):
        for description, code in [
            ('row count <=32', '49 83 7c 24 58 20'),
            ('repetition <=1024', '49 81 ff 00 04 00 00'),
            ('attempts <=32768', '49 81 7c 24 70 00 80 00 00'),
            ('result-index x16', '48 c1 e1 04'),
            ('accepted script flag mask', '48 b8 ff 01 00 00 ff ff 00 00'),
        ]:
            with self.subTest(limit=description):
                self.assertIn(bytes.fromhex(code), self.code)

    def test_actual_syscalls_generic_dispatch_and_failure_exit_use_int80(self):
        self.assertEqual(self.code.count(bytes.fromhex('cd 80')), 4)
        # Four static sites: abi_info, row-dispatched syscall, yield, failure exit.
        self.assertIn(bytes.fromhex('48 8b 45 00 48 3d 00 01 00 00'), self.code)
        self.assertTrue(self.code.endswith(bytes.fromhex(
            'b8 02 00 00 00 bf e4 00 00 00 31 f6 31 d2 45 31 d2 45 31 c0 45 31 c9 cd 80 0f 0b')))

    def test_actual_private_output_and_load_instructions_exist(self):
        # Store both actual result registers; load a received u64 into RDX.
        self.assertIn(bytes.fromhex('49 89 84 0c 00 0c 00 00'), self.code)
        self.assertIn(bytes.fromhex('49 89 94 0c 08 0c 00 00'), self.code)
        self.assertIn(bytes.fromhex('48 8b 17 31 c0'), self.code)
        self.assertIn(bytes.fromhex('0f 0b'), self.code)

    def test_reject_truncation_of_required_load_bytes(self):
        for size in [0, 63, self.phoff + 167, self.text['offset'] + self.text['filesz'] - 1,
                     self.data['offset'] + self.data['filesz'] - 1,
                     self.readonly['offset'] + self.readonly['filesz'] - 1]:
            with self.subTest(size=size):
                self.reject(raw=self.raw[:size])

    def test_reject_bad_headers_and_dynamic_features(self):
        for fmt, offset, value in [('<H', 16, 3), ('<H', 18, 3), ('<B', 4, 1), ('<B', 5, 2),
                                   ('<H', 56, 0), ('<H', 56, 65), ('<I', self.phoff, 2),
                                   ('<I', self.phoff, 3), ('<I', self.phoff, 7)]:
            with self.subTest(offset=offset, value=value):
                self.reject([(fmt, offset, value)])

    def test_reject_permission_escalation_and_readonly_boundary_changes(self):
        for offset, value in [(self.phoff + 4, 7), (self.phoff + 112 + 4, 6),
                              (self.phoff + 56 + 4, 4)]:
            with self.subTest(offset=offset, value=value):
                self.reject([('<I', offset, value)])
        self.reject([('<Q', self.phoff + 112 + 16, 0x40014000),
                     ('<Q', self.phoff + 112 + 24, 0x40014000)])

    def test_reject_entry_in_data_or_readonly_and_page_overlap(self):
        for entry in [0, 0x40010000, 0x40013000, 0x60000000]:
            with self.subTest(entry=entry):
                self.reject([('<Q', 24, entry)])
        self.reject([('<Q', self.phoff + 112 + 16, 0x40012000),
                     ('<Q', self.phoff + 112 + 24, 0x40012000)])

    def test_reject_size_overflow_alignment_and_control_shrink(self):
        for offset, value in [(self.phoff + 8, (1 << 64) - 1),
                              (self.phoff + 40, (1 << 64) - 1),
                              (self.phoff + 48, 4097), (self.phoff + 56 + 32, 32),
                              (self.phoff + 56 + 40, 32)]:
            with self.subTest(offset=offset, value=value):
                self.reject([('<Q', offset, value)])


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--elf', type=Path, default=ROOT / 'build/users/ipc_probe.elf')
    arguments, rest = parser.parse_known_args()
    ELF = arguments.elf.resolve(strict=True)
    unittest.main(argv=[sys.argv[0], *rest], verbosity=2)
