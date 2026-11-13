#!/usr/bin/env python3
"""Host-only SMP canary artifact and mutation checks; no CPL3/QEMU execution.

R3-3: a timer may interrupt any instruction. DATA+128 may advertise a ready
canary only after initialization and successful GPR/x87/SSE comparisons.
The small decoder below accepts only the reviewed canary instruction sequence;
it is intentionally not a general x86 decoder or proof of FPU isolation.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import struct
import sys
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
spec = importlib.util.spec_from_file_location('smp_fpu_build_inspector', ROOT / 'tools/build_users.py')
inspector = importlib.util.module_from_spec(spec)
spec.loader.exec_module(inspector)

READY = bytes.fromhex('49 c7 84 24 80 00 00 00 01 00 00 00')
STAGE4 = bytes.fromhex('49 c7 84 24 a0 00 00 00 04 00 00 00')
STAGE5 = bytes.fromhex('49 c7 84 24 a0 00 00 00 05 00 00 00')


def inspect_canary(code: bytes) -> dict:
    """Validate the actual RX sequence, its branch targets and ready store."""
    if code.count(STAGE4) != 1 or code.count(READY) != 1:
        raise ValueError('missing or duplicate canary entry/readiness store')
    cursor = code.index(STAGE4)
    instructions = []
    offsets = {}

    def consume(name, encoded):
        nonlocal cursor
        expected = bytes.fromhex(encoded) if isinstance(encoded, str) else encoded
        if code[cursor:cursor + len(expected)] != expected:
            raise ValueError(f'{name}: unexpected canary instruction at RX+{cursor:#x}')
        offsets[name] = cursor
        instructions.append((name, cursor, cursor + len(expected)))
        cursor += len(expected)

    def jne(name):
        nonlocal cursor
        at = cursor
        if code[at:at + 1] == b'\x75' and at + 2 <= len(code):
            end = at + 2
            target = end + struct.unpack_from('<b', code, at + 1)[0]
        elif code[at:at + 2] == b'\x0f\x85' and at + 6 <= len(code):
            end = at + 6
            target = end + struct.unpack_from('<i', code, at + 2)[0]
        else:
            raise ValueError(f'{name}: missing conditional failure/initialization branch')
        if not 0 <= target < len(code):
            raise ValueError(f'{name}: branch escapes RX segment')
        offsets[name] = at
        instructions.append((name, at, end))
        cursor = end
        return target

    consume('entry', STAGE4)
    abi_targets = []
    for register, encoded in (
        ('rbx', '48 81 fb df 9b 57 13'),
        ('rbp', '48 81 fd e0 ac 68 24'),
        ('r14', '49 81 fe 44 33 22 11'),
        ('r15', '49 81 ff 88 77 66 55'),
    ):
        consume('compare_' + register, encoded)
        abi_targets.append(jne('reject_' + register))
    if len(set(abi_targets)) != 1 or code[abi_targets[0]:abi_targets[0] + 6] != bytes.fromhex('fc bf e0 00 00 00'):
        raise ValueError('GPR mismatch does not reach the existing ABI failure exit')
    consume('stage5', STAGE5)
    consume('ready_test', '49 83 bc 24 80 00 00 00 00')
    initialized_target = jne('skip_initialized')
    consume('fninit', 'db e3')
    consume('load_st0', '41 df 6c 24 10')
    consume('load_xmm0', '66 49 0f 6e c5')
    consume('ones_xmm15', '66 45 0f 76 ff')
    consume('complement_xmm15', '66 44 0f ef f8')
    if initialized_target != cursor:
        raise ValueError('initialized branch bypasses the repeated FPU checks')
    consume('read_xmm0', '66 48 0f 7e c0')
    consume('compare_xmm0', '4c 39 e8')
    corrupt_targets = [jne('reject_xmm0')]
    consume('read_xmm15', '66 4c 0f 7e f8')
    consume('invert_xmm15', '48 f7 d0')
    consume('compare_xmm15', '4c 39 e8')
    corrupt_targets.append(jne('reject_xmm15'))
    consume('duplicate_st0', 'd9 c0')
    consume('convert_pop_st0', 'df 7c 24 f8')
    consume('compare_st0', '4c 39 6c 24 f8')
    corrupt_targets.append(jne('reject_st0'))
    if len(set(corrupt_targets)) != 1 or code[corrupt_targets[0]:corrupt_targets[0] + 3] != bytes.fromhex('cc 0f 0b'):
        raise ValueError('FPU mismatch does not reach int3/ud2 without readiness')
    consume('publish_ready', READY)
    consume('ipc_mode', '49 83 3c 24 14')
    return {'offsets': offsets, 'instructions': instructions,
            'fpu_failure': corrupt_targets[0], 'abi_failure': abi_targets[0]}


class SmpFpuProbeArtifact(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.raw = ELF.read_bytes()
        plan = inspector.inspect_user(ELF)
        cls.text = next(segment for segment in plan['segments'] if segment['flags'] == 5)
        cls.code = cls.raw[cls.text['offset']:cls.text['offset'] + cls.text['filesz']]
        cls.evidence = inspect_canary(cls.code)

    def test_actual_elf_is_bound_to_current_probe_source_and_binary(self):
        manifest = json.loads(ELF.with_name('manifest.json').read_text(encoding='utf-8'))
        for key, path in (('binary_sha256', ELF), ('source_sha256', ROOT / 'userspace/probe.rs')):
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), manifest[key])

    def test_ready_follows_complete_initialization_and_successful_checks(self):
        at = self.evidence['offsets']
        self.assertLess(at['reject_r15'], at['fninit'])
        self.assertLess(at['fninit'], at['load_st0'])
        self.assertLess(at['complement_xmm15'], at['reject_xmm0'])
        self.assertLess(at['reject_xmm0'], at['reject_xmm15'])
        self.assertLess(at['reject_xmm15'], at['compare_st0'])
        self.assertLess(at['reject_st0'], at['publish_ready'])

    def test_every_first_pass_interrupt_boundary_before_readiness_stays_unready(self):
        # This bounded model uses instruction boundaries decoded from the real
        # ELF. It models only a first, successful initialization/check pass,
        # with a timer interruption before/after each instruction; it does not
        # claim to execute x86 or prove the kernel's save/restore implementation.
        required = {'reject_rbx', 'reject_rbp', 'reject_r14', 'reject_r15',
                    'fninit', 'load_st0', 'load_xmm0', 'ones_xmm15',
                    'complement_xmm15', 'reject_xmm0', 'reject_xmm15', 'reject_st0'}
        completed = set()
        ready = False
        boundaries = 0
        for name, start, end in self.evidence['instructions']:
            with self.subTest(boundary=f'before {name}', rx=start):
                self.assertEqual(ready, 'publish_ready' in completed)
                if not required.issubset(completed):
                    self.assertFalse(ready)
            if name == 'publish_ready':
                self.assertTrue(required.issubset(completed))
                ready = True
            completed.add(name)
            with self.subTest(boundary=f'after {name}', rx=end):
                if ready:
                    self.assertTrue(required.issubset(completed))
            boundaries += 2
        self.assertTrue(ready)
        self.assertGreaterEqual(boundaries, 50)

    def moved_ready(self, target):
        at = self.evidence['offsets']['publish_ready']
        changed = self.code[:at] + self.code[at + len(READY):]
        return changed[:target] + READY + changed[target:]

    def test_rejects_readiness_before_initialization(self):
        # Recreates the historical early-ready store in actual compiled bytes.
        with self.assertRaises(ValueError):
            inspect_canary(self.moved_ready(self.evidence['offsets']['fninit']))

    def test_rejects_readiness_before_x87_comparison(self):
        with self.assertRaises(ValueError):
            inspect_canary(self.moved_ready(self.evidence['offsets']['compare_st0']))

    def test_rejects_missing_readiness_publication(self):
        at = self.evidence['offsets']['publish_ready']
        changed = self.code[:at] + b'\x90' * len(READY) + self.code[at + len(READY):]
        with self.assertRaises(ValueError):
            inspect_canary(changed)

    def test_rejects_failure_branch_redirected_to_ready(self):
        at = self.evidence['offsets']['reject_st0']
        target = self.evidence['offsets']['publish_ready']
        changed = bytearray(self.code)
        if changed[at] == 0x75:
            struct.pack_into('<b', changed, at + 1, target - (at + 2))
        else:
            struct.pack_into('<i', changed, at + 2, target - (at + 6))
        with self.assertRaises(ValueError):
            inspect_canary(bytes(changed))

    def test_rejects_initialized_branch_bypassing_checks(self):
        at = self.evidence['offsets']['skip_initialized']
        target = self.evidence['offsets']['publish_ready']
        changed = bytearray(self.code)
        if changed[at] == 0x75:
            struct.pack_into('<b', changed, at + 1, target - (at + 2))
        else:
            struct.pack_into('<i', changed, at + 2, target - (at + 6))
        with self.assertRaises(ValueError):
            inspect_canary(bytes(changed))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--elf', type=Path, default=ROOT / 'build/users/probe.elf')
    arguments, rest = parser.parse_known_args()
    ELF = arguments.elf.resolve(strict=True)
    unittest.main(argv=[sys.argv[0], *rest], verbosity=2)
