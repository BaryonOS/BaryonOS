#!/usr/bin/env python3
"""Host artifact/ELF checks only; never executes the probe or proves ring-3 isolation."""
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
spec = importlib.util.spec_from_file_location('probe_build_inspector', ROOT / 'tools/build_users.py')
inspector = importlib.util.module_from_spec(spec)
spec.loader.exec_module(inspector)


class ProbeArtifact(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.raw = ELF.read_bytes()
        cls.plan = inspector.inspect_user(ELF)
        cls.phoff = struct.unpack_from('<Q', cls.raw, 32)[0]
        cls.text = next(s for s in cls.plan['segments'] if s['flags'] == 5)
        cls.data = next(s for s in cls.plan['segments'] if s['flags'] == 6)
        cls.code = cls.raw[cls.text['offset']:cls.text['offset'] + cls.text['filesz']]

    def rejected(self, mutations=(), raw=None):
        changed = bytearray(self.raw if raw is None else raw)
        for fmt, offset, value in mutations:
            struct.pack_into(fmt, changed, offset, value)
        # All temporary writes remain inside this agent's owned subtree.
        with tempfile.TemporaryDirectory(dir=Path(__file__).parent) as folder:
            path = Path(folder) / 'rejected.elf'
            path.write_bytes(changed)
            with self.assertRaises(ValueError):
                inspector.inspect_user(path)

    def test_exact_entry_segments_and_private_data_layout(self):
        self.assertEqual(self.plan['entry'], 0x40000000)
        self.assertEqual(len(self.plan['segments']), 2)
        self.assertEqual(self.text['address'], 0x40000000)
        self.assertEqual(self.data['address'], 0x40010000)
        self.assertEqual((self.data['filesz'], self.data['memsz']), (32, 32))
        self.assertEqual(self.raw[self.data['offset']:self.data['offset'] + 32], bytes(32))
        self.assertLessEqual(self.text['memsz'], 4096)

    def test_manifest_matches_actual_source_linker_and_binary(self):
        manifest = json.loads(ELF.with_name('manifest.json').read_text(encoding='utf-8'))
        for key, path in [('binary_sha256', ELF), ('source_sha256', ROOT / 'userspace/probe.rs'),
                          ('linker_sha256', ROOT / 'userspace/linker.ld')]:
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), manifest[key])

    def test_no_dynamic_relocations_tls_or_unexpected_allocatable_sections(self):
        shoff = struct.unpack_from('<Q', self.raw, 40)[0]
        shsize, shnum, shstr = struct.unpack_from('<HHH', self.raw, 58)
        self.assertEqual(shsize, 64)
        self.assertGreater(shnum, 0)
        sections = [struct.unpack_from('<IIQQQQIIQQ', self.raw, shoff + i * shsize) for i in range(shnum)]
        names = self.raw[sections[shstr][4]:sections[shstr][4] + sections[shstr][5]]
        allocated = set()
        for section in sections:
            name = names[section[0]:].split(b'\0', 1)[0].decode('ascii')
            self.assertFalse(section[2] & 0x400, f'TLS section: {name}')
            self.assertFalse(section[1] in (4, 6, 9, 11) and section[5], name)
            if section[2] & 2 and section[5]:
                allocated.add(name)
        self.assertEqual(allocated, {'.text', '.data'})

    def test_exit_machine_code_clears_every_unused_argument_including_rdx(self):
        # RAX=2; RSI,RDX,R10,R8,R9=0; int80; ud2 if exit erroneously returns.
        # RDI deliberately contains the exit code and is not cleared here.
        expected = bytes.fromhex('b8 02 00 00 00 31 f6 31 d2 45 31 d2 45 31 c0 45 31 c9 cd 80 0f 0b')
        self.assertTrue(self.code.endswith(expected), self.code[-len(expected):].hex())
        # Normal/unknown/invalid-argument prelude, yield and exit.
        self.assertEqual(self.code.count(bytes.fromhex('cd 80')), 5)

    def test_negative_syscall_and_direction_flag_prelude_is_in_actual_binary(self):
        for label, opcode in [
            ('hostile DF directly before int80', 'fd cd 80'),
            ('read and check returned DF, then clear it', '9c 5a f7 c2 00 04 00 00 fc'),
            ('unknown syscall with zero RDX, expect ENOSYS', 'b8 ff ff 00 00 31 d2 cd 80 48 83 f8 fe'),
            ('unused RSI=1, expect EINVAL', '31 c0 be 01 00 00 00 cd 80 48 83 f8 ff'),
        ]:
            with self.subTest(prelude=label):
                self.assertIn(bytes.fromhex(opcode), self.code)

    def test_required_forbidden_instruction_encodings_are_in_rx_segment(self):
        for label, opcode in [('cli', 'fa'), ('fninit', 'db e3'), ('pxor xmm0,xmm0', '66 0f ef c0'),
                              ('vxorps ymm0,ymm0,ymm0', 'c5 fc 57 c0'), ('ud2', '0f 0b')]:
            with self.subTest(instruction=label):
                self.assertIn(bytes.fromhex(opcode), self.code)

    def test_rejects_wrong_class_endian_machine_and_dynamic_type(self):
        for fmt, offset, value in [('<B', 4, 1), ('<B', 5, 2), ('<H', 16, 3), ('<H', 18, 3),
                                   ('<H', 52, 63), ('<H', 54, 55), ('<H', 56, 0), ('<H', 56, 65)]:
            with self.subTest(offset=offset, value=value):
                self.rejected([(fmt, offset, value)])

    def test_rejects_truncated_required_bytes(self):
        for size in [0, 63, self.phoff + 111, self.text['offset'],
                     self.text['offset'] + self.text['filesz'] - 1,
                     self.data['offset'] + self.data['filesz'] - 1]:
            with self.subTest(size=size):
                self.rejected(raw=self.raw[:size])

    def test_rejects_interpreter_dynamic_tls_and_unknown_program_types(self):
        for kind in [2, 3, 7, 0x12345678]:
            with self.subTest(kind=kind):
                self.rejected([('<I', self.phoff, kind)])

    def test_rejects_writable_executable_and_unknown_permission_bits(self):
        for flags in [0, 1, 2, 3, 7, 13]:
            with self.subTest(flags=flags):
                self.rejected([('<I', self.phoff + 4, flags)])

    def test_rejects_out_of_range_unaligned_and_arithmetic_extremes(self):
        for offset, value in [(8, (1 << 64) - 1), (16, 0x3ffff000), (16, 0x60000000),
                              (24, (1 << 64) - 1), (32, self.text['memsz'] + 1),
                              (40, (1 << 64) - 1), (48, 4097), (48, 2048)]:
            with self.subTest(offset=offset, value=value):
                self.rejected([('<Q', self.phoff + offset, value)])

    def test_rejects_page_overlap_even_for_disjoint_byte_ranges(self):
        self.rejected([('<Q', self.phoff + 56 + 8, self.text['offset'] + 0x200),
                       ('<Q', self.phoff + 56 + 16, 0x40000200),
                       ('<Q', self.phoff + 56 + 24, 0x40000200)])

    def test_rejects_entry_in_data_outside_image_or_rx_bss(self):
        for entry in [0, 0x40010000, 0x60000000]:
            with self.subTest(entry=entry):
                self.rejected([('<Q', 24, entry)])
        self.rejected([('<Q', self.phoff + 40, self.text['filesz'] + 1),
                       ('<Q', 24, self.text['address'] + self.text['filesz'])])


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--elf', type=Path, default=ROOT / 'build/users/probe.elf')
    arguments, rest = parser.parse_known_args()
    ELF = arguments.elf.resolve(strict=True)
    unittest.main(argv=[sys.argv[0], *rest], verbosity=2)
