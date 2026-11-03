"""Counterexamples for the historical native-evidence reviewers; no VM is run."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import verify_receipts as receipts
import verify_regression as regression

BASE = Path(__file__).resolve().parent.parent


def load(path):
    return json.loads((BASE / path).read_text(encoding='utf-8-sig'))


class ReviewContracts(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.runs = load('native/matrix.json')['runs']
        cls.host = load('native/regression/build/volume-gate.json')['host']

    def test_expected_image_and_mode_for_each_label(self):
        self.assertEqual({run['label'] for run in self.runs}, set(receipts.EXPECTED_RUNS))
        for run in self.runs:
            receipts.check_run_contract(run)
            for field, wrong in (('source_image_sha256_before', '0' * 64),
                                 ('data_disk_read_only', not run['data_disk_read_only'])):
                with self.subTest(label=run['label'], field=field):
                    changed = dict(run, **{field: wrong})
                    with self.assertRaises(AssertionError):
                        receipts.check_run_contract(changed)
                    del changed[field]
                    with self.assertRaises(AssertionError):
                        receipts.check_run_contract(changed)

    def test_two_domain_digests_required(self):
        for run in self.runs[:2]:
            digests = run['oracle_digests']
            serial = [f'VOLUME:A:DIGEST:{digests[0]:020}', f'VOLUME:B:DIGEST:{digests[1]:020}']
            receipts.check_serial_digests(run, serial, run['label'])
            for invalid in ([], digests[:1], digests + [0], None):
                with self.subTest(label=run['label'], digests=invalid):
                    with self.assertRaises(AssertionError):
                        receipts.check_serial_digests(dict(run, oracle_digests=invalid), serial, run['label'])
            changed = dict(run)
            del changed['oracle_digests']
            with self.assertRaises(AssertionError):
                receipts.check_serial_digests(changed, serial, run['label'])
            for index in range(2):
                changed_serial = serial.copy()
                changed_serial[index] = changed_serial[index].rsplit(':', 1)[0] + ':0'
                with self.assertRaises(AssertionError):
                    receipts.check_serial_digests(run, changed_serial, run['label'])

    def test_synthetic_labels_require_their_kernel_profile(self):
        labels = {'own-normal': False, 'own-cold-recover': True}
        labels.update({f'{candidate}-{mode}': mode == 'readonly'
                       for candidate in ('btrfs', 'zero', 'zero-prefix-payload-tail', 'openzfs-leaf')
                       for mode in ('readonly', 'writable')})
        for label, readonly in labels.items():
            image = receipts.IMAGE_HASHES[label if label.startswith('own-')
                                          else label.rsplit('-', 1)[0]]
            profile = 'test-volume-recover' if readonly else 'native-volume'
            item = dict(label=label, source_image_sha256_before=image,
                        data_disk_read_only=readonly, profile=profile)
            receipts.check_run_contract(item)
            for wrong in ('native-volume' if readonly else 'test-volume-recover', '', None):
                with self.subTest(label=label, profile=wrong):
                    with self.assertRaisesRegex(AssertionError, 'profile'):
                        receipts.check_run_contract(dict(item, profile=wrong))
            del item['profile']
            with self.assertRaisesRegex(AssertionError, 'profile'):
                receipts.check_run_contract(item)

    def test_synthetic_v3_graphics_scope_order_and_exclusions(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            files = {'ui/graphics/a.rs': b'lower', 'ui/graphics/A.rs': b'upper',
                     'ui/graphics/Z/z.rs': b'nested', 'Cargo.toml': b'workspace'}
            excluded = {'ui/graphics/target/ignored.rs': b'cache',
                        'ui/graphics/verification/ignored.rs': b'proof'}
            for name, data in {**files, **excluded}.items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
            digest = hashlib.sha256(b'nieuw-os source inputs v3\n')
            for name in ('ui/graphics/A.rs', 'ui/graphics/Z/z.rs', 'ui/graphics/a.rs',
                         'Cargo.toml'):
                digest.update(name.encode())
                digest.update(files[name])
            with patch.object(receipts, 'SOURCE', root):
                self.assertEqual(receipts.source_fingerprint(), digest.hexdigest())
                (root / 'ui/graphics/a.rs').write_bytes(b'changed')
                self.assertNotEqual(receipts.source_fingerprint(), digest.hexdigest())

    def test_host_suites_require_expected_commands_and_nonzero_counts(self):
        regression.verify_host_checks(self.host)
        for index in range(3):
            with self.subTest(suite=index):
                changed = copy.deepcopy(self.host)
                changed[index]['command'] = changed[(index + 1) % 3]['command']
                with self.assertRaises(AssertionError):
                    regression.verify_host_checks(changed)
                changed = copy.deepcopy(self.host)
                output = ('test result: ok. 0 passed; 0 failed;' if index < 2
                          else 'Ran 0 tests in 0.001s\nOK')
                changed[index].update(stdout=output, stderr='', totals=output.splitlines())
                with self.assertRaises(AssertionError):
                    regression.verify_host_checks(changed)
                changed = copy.deepcopy(self.host)
                changed[index]['exit'] = 1
                with self.assertRaises(AssertionError):
                    regression.verify_host_checks(changed)
                changed = copy.deepcopy(self.host)
                changed[index]['stderr'] += '\nFAILED\n'
                changed[index]['totals'].append('FAILED')
                with self.assertRaises(AssertionError):
                    regression.verify_host_checks(changed)

    def test_cut_requires_kill_running_and_raw_exit_one(self):
        for case in regression.CUTS:
            good = {'raw_qemu_exit': 1, 'host_kill': True, 'running_before_kill': True}
            regression.check_cut_exit(good, case)
            for field, wrong in (('raw_qemu_exit', 33), ('raw_qemu_exit', True),
                                 ('raw_qemu_exit', 1.0), ('host_kill', False),
                                 ('running_before_kill', False)):
                with self.subTest(case=case, field=field, value=wrong):
                    changed = dict(good, **{field: wrong})
                    with self.assertRaises(AssertionError):
                        regression.check_cut_exit(changed, case)
                    del changed[field]
                    with self.assertRaises(AssertionError):
                        regression.check_cut_exit(changed, case)


if __name__ == '__main__':
    unittest.main()
