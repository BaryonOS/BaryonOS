#!/usr/bin/env python3
"""M5 artifact/negative ELF checks; does not execute service or device I/O."""
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
spec = importlib.util.spec_from_file_location('m5_build_inspector', ROOT / 'tools/build_users.py')
inspector = importlib.util.module_from_spec(spec)
spec.loader.exec_module(inspector)


def exact_m5(path):
    plan = inspector.inspect_user(path)
    if plan['entry'] != 0x40000000 or len(plan['segments']) != 3:
        raise ValueError('M5 entry/segment count')
    text, data, readonly = plan['segments']
    if not (text['address'] == 0x40000000 and text['flags'] == 5 and 0 < text['memsz'] <= 65536
            and data['address'] == 0x40010000 and data['flags'] == 6
            and data['memsz'] == 12288 and data['filesz'] == 12288
            and readonly['address'] == 0x40013000 and readonly['flags'] == 4
            and 0 < readonly['memsz'] <= 4096):
        raise ValueError('M5 private memory ABI')
    return plan


class M5Artifact(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.raw = ELF.read_bytes()
        cls.plan = exact_m5(ELF)
        cls.text, cls.data, cls.readonly = cls.plan['segments']
        cls.phoff = struct.unpack_from('<Q', cls.raw, 32)[0]

    def reject(self, mutations=(), raw=None):
        data = bytearray(self.raw if raw is None else raw)
        for fmt, offset, value in mutations:
            struct.pack_into(fmt, data, offset, value)
        with tempfile.TemporaryDirectory(dir=Path(__file__).parent) as folder:
            path = Path(folder) / 'bad-m5.elf'
            path.write_bytes(data)
            with self.assertRaises(ValueError):
                exact_m5(path)

    def sections(self):
        at = struct.unpack_from('<Q', self.raw, 40)[0]
        size, count, name_index = struct.unpack_from('<HHH', self.raw, 58)
        self.assertEqual(size, 64)
        sections = [struct.unpack_from('<IIQQQQIIQQ', self.raw, at + i * size) for i in range(count)]
        names_head = sections[name_index]
        names = self.raw[names_head[4]:names_head[4] + names_head[5]]
        return [(names[s[0]:].split(b'\0', 1)[0].decode('ascii'), s) for s in sections], sections

    def test_fixed_rx_rw_ro_memory_and_mapping_budget(self):
        self.assertEqual([s['flags'] for s in self.plan['segments']], [5, 6, 4])
        self.assertEqual(self.data['page_end'], self.readonly['page_start'])
        pages = sum((s['page_end'] - s['page_start']) // 4096 for s in self.plan['segments'])
        self.assertLessEqual(pages + 32, 2048)

    def test_initial_startup_and_two_message_pages_are_zero(self):
        offset = self.data['offset']
        self.assertEqual(self.raw[offset:offset + 1024], bytes(1024))
        self.assertEqual(self.raw[offset + 4096:offset + 12288], bytes(8192))

    def test_symbols_bind_startup_message_output_and_entry_addresses(self):
        named, sections = self.sections()
        symbols = {}
        for _, section in named:
            if section[1] != 2:
                continue
            strings_head = sections[section[6]]
            strings = self.raw[strings_head[4]:strings_head[4] + strings_head[5]]
            self.assertEqual(section[9], 24)
            for at in range(section[4], section[4] + section[5], 24):
                name, _, _, _, value, _ = struct.unpack_from('<IBBHQQ', self.raw, at)
                label = strings[name:].split(b'\0', 1)[0].decode('ascii')
                symbols[label] = value
        for name, address in [('_start', 0x40000000), ('m5_startup', 0x40010000),
                              ('m5_message', 0x40011000), ('m5_output', 0x40012000)]:
            self.assertEqual(symbols[name], address)

    def test_no_tls_dynamic_relocations_or_unexpected_allocatable_sections(self):
        named, _ = self.sections()
        for name, section in named:
            self.assertFalse(section[2] & 0x400, name)
            self.assertFalse(section[1] in (4, 6, 9, 11) and section[5], name)
            if section[2] & 2 and section[5]:
                self.assertIn(name, {'.text', '.data', '.buffers', '.rodata', '.bss'})

    def test_manifest_binds_actual_main_linker_and_binary(self):
        manifest = json.loads(ELF.with_name('manifest-m5.json').read_text(encoding='utf-8'))
        for key, path in [('binary_sha256', ELF), ('source_sha256', ROOT / 'userspace/m5_probe.rs'),
                          ('linker_sha256', ROOT / 'userspace/m5_linker.ld')]:
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), manifest[key])
        modules = ['userspace/m5_probe.rs', 'userspace/m5_runtime.rs',
                   'services/block/src/wire.rs', 'services/block/src/user_service.rs']
        self.assertEqual(set(manifest['module_sha256']), set(modules))
        for name in modules:
            with self.subTest(module=name):
                self.assertEqual(hashlib.sha256((ROOT / name).read_bytes()).hexdigest(),
                                 manifest['module_sha256'][name])

    def test_static_int80_boundary_is_in_actual_text(self):
        text = self.raw[self.text['offset']:self.text['offset'] + self.text['filesz']]
        self.assertIn(b'\xcd\x80', text)
        # Presence is artifact evidence only, never proof of an executed syscall.

    def test_reject_truncated_required_segment_bytes(self):
        sizes = [0, 63, self.phoff + 167]
        sizes += [s['offset'] + s['filesz'] - 1 for s in self.plan['segments']]
        for size in sizes:
            with self.subTest(size=size):
                self.reject(raw=self.raw[:size])

    def test_reject_wrong_header_dynamic_image_and_tls(self):
        for fmt, offset, value in [('<H', 16, 3), ('<H', 18, 3), ('<B', 4, 1), ('<B', 5, 2),
                                   ('<H', 56, 0), ('<H', 56, 65), ('<I', self.phoff, 2),
                                   ('<I', self.phoff, 3), ('<I', self.phoff, 7)]:
            with self.subTest(offset=offset, value=value):
                self.reject([(fmt, offset, value)])

    def test_reject_write_execute_and_changed_buffer_or_ro_permissions(self):
        for offset, value in [(self.phoff + 4, 7), (self.phoff + 56 + 4, 4),
                              (self.phoff + 112 + 4, 6)]:
            self.reject([('<I', offset, value)])
        self.reject([('<Q', self.phoff + 56 + 40, 8192)])

    def test_reject_entry_in_nonexecutable_data_or_alias_outside_image(self):
        for entry in [0, 0x40010000, 0x40013000, 0x0001000040000000]:
            self.reject([('<Q', 24, entry)])

    def test_reject_segment_overflow_overlap_and_bad_alignment(self):
        self.reject([('<Q', self.phoff + 40, (1 << 64) - 1)])
        self.reject([('<Q', self.phoff + 48, 4097)])
        self.reject([('<Q', self.phoff + 112 + 16, 0x40012000),
                     ('<Q', self.phoff + 112 + 24, 0x40012000)])


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--elf', type=Path, default=ROOT / 'build/users/m5_probe.elf')
    arguments, rest = parser.parse_known_args()
    ELF = arguments.elf.resolve(strict=True)
    unittest.main(argv=[sys.argv[0], *rest], verbosity=2)
