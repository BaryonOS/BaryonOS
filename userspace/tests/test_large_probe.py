#!/usr/bin/env python3
"""Large-image artifact regression only; no kernel execution or BSS RAM reads."""
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
spec = importlib.util.spec_from_file_location('large_build_inspector', ROOT / 'tools/build_users.py')
inspector = importlib.util.module_from_spec(spec)
spec.loader.exec_module(inspector)


class LargeArtifact(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.raw = ELF.read_bytes()
        cls.plan = inspector.inspect_user(ELF)
        cls.text, cls.data = cls.plan['segments']

    def test_exact_memory_plan_exceeds_old_64_user_mapping_limit(self):
        self.assertEqual(self.plan['entry'], 0x40000000)
        self.assertEqual(len(self.plan['segments']), 2)
        self.assertEqual((self.text['flags'], self.text['address']), (5, 0x40000000))
        self.assertEqual((self.data['flags'], self.data['address']), (6, 0x40010000))
        self.assertEqual((self.data['filesz'], self.data['memsz']), (32, 34 * 4096))
        pages = sum((s['page_end'] - s['page_start']) // 4096 for s in self.plan['segments'])
        self.assertEqual(pages, 35)
        self.assertEqual(pages + 16 + 16, 67)
        self.assertGreater(pages + 32, 64)
        self.assertLessEqual(pages + 32, 2048)

    def test_text_is_byte_identical_to_unchanged_normal_m3_probe(self):
        normal = ELF.with_name('probe.elf')
        normal_raw = normal.read_bytes()
        normal_text = inspector.inspect_user(normal)['segments'][0]
        actual = self.raw[self.text['offset']:self.text['offset'] + self.text['filesz']]
        expected = normal_raw[normal_text['offset']:normal_text['offset'] + normal_text['filesz']]
        self.assertEqual(actual, expected)

    def test_initial_four_words_match_normal_data_and_do_not_include_bss_bytes(self):
        self.assertEqual(self.raw[self.data['offset']:self.data['offset'] + 32], bytes(32))
        self.assertLess(self.data['filesz'], self.data['memsz'])
        self.assertEqual(self.data['page_end'], 0x40032000)

    def test_bss_is_noload_33_pages_and_inside_rw_segment(self):
        shoff = struct.unpack_from('<Q', self.raw, 40)[0]
        size, count, names_index = struct.unpack_from('<HHH', self.raw, 58)
        self.assertEqual(size, 64)
        sections = [struct.unpack_from('<IIQQQQIIQQ', self.raw, shoff + i * size) for i in range(count)]
        ns = sections[names_index]
        names = self.raw[ns[4]:ns[4] + ns[5]]
        found = []
        for section in sections:
            name = names[section[0]:].split(b'\0', 1)[0].decode('ascii')
            self.assertFalse(section[2] & 0x400, name)
            self.assertFalse(section[1] in (4, 6, 9, 11) and section[5], name)
            if name == '.bss':
                found.append(section)
        self.assertEqual(len(found), 1)
        bss = found[0]
        self.assertEqual((bss[1], bss[2], bss[3], bss[5]), (8, 3, 0x40011000, 33 * 4096))
        self.assertEqual(bss[3] + bss[5], self.data['address'] + self.data['memsz'])

    def test_manifest_binds_existing_probe_source_new_linker_and_actual_binary(self):
        manifest = json.loads(ELF.with_name('manifest-large.json').read_text(encoding='utf-8'))
        for key, path in [('binary_sha256', ELF), ('source_sha256', ROOT / 'userspace/probe.rs'),
                          ('linker_sha256', ROOT / 'userspace/large_linker.ld')]:
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), manifest[key])

    def test_mutated_large_segment_cannot_escape_user_image_range_or_gain_wx(self):
        phoff = struct.unpack_from('<Q', self.raw, 32)[0]
        for fmt, offset, value in [('<Q', phoff + 56 + 40, (1 << 64) - 1),
                                   ('<I', phoff + 56 + 4, 7)]:
            changed = bytearray(self.raw)
            struct.pack_into(fmt, changed, offset, value)
            with self.subTest(offset=offset), tempfile.TemporaryDirectory(dir=Path(__file__).parent) as folder:
                path = Path(folder) / 'bad-large.elf'
                path.write_bytes(changed)
                with self.assertRaises(ValueError):
                    inspector.inspect_user(path)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--elf', type=Path, default=ROOT / 'build/users/probe_large.elf')
    arguments, rest = parser.parse_known_args()
    ELF = arguments.elf.resolve(strict=True)
    unittest.main(argv=[sys.argv[0], *rest], verbosity=2)
