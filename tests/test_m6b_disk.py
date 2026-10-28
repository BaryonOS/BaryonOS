"""Adversarial checks of the external disk oracle, not native-OS evidence."""
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import m6b_disk as d


def record(slot, object_id, revision, *, operation=2, payload=None):
    if payload is None:
        payload = d.fixture_value(object_id, revision) if operation == 2 else b''
    out = bytearray(512)
    out[:8] = d.MAGIC
    struct.pack_into('<HHIQQQQHH', out, 8, 1, 64, 0, slot+1, slot+1,
                     object_id, revision, operation, len(payload))
    out[64:64+len(payload)] = payload
    seal(out)
    return bytes(out)


def seal(data):
    data[504:] = bytes(8)
    struct.pack_into('<Q', data, 504, int(d.digest(data), 16))


def complete(initial):
    out = bytearray(initial)
    mutations = [(0,1,2), (1,1,2), (0,2,2), (1,2,3),
                 (2,1,2), (3,1,2), (2,2,2), (3,2,3)]
    for slot, (obj, rev, op) in enumerate(mutations):
        at = (d.FIRST + slot)*512
        out[at:at+512] = record(slot,obj,rev,operation=op)
    return bytes(out)


def markers(state, acks=8, rejected=6):
    lines = [f'M6B:SEQUENCE:{state["sequence"]}']
    for i, obj in enumerate(state['objects']):
        for field in ('revision','present','length','digest'):
            value = int(obj[field]) if field == 'present' else obj[field]
            lines.append(f'M6B:OBJECT{i}:{field.upper()}:{value}')
    lines += [f'M6B:ACKS:{acks}', f'M6B:REJECTED_NO_IO:{rejected}']
    return '\n'.join(lines) + '\n'


class DiskOracle(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='m6b-oracle-')
        self.patch = patch.object(d, 'BUILD', Path(self.tmp.name))
        self.patch.start()
        self.addCleanup(self.tmp.cleanup)
        self.addCleanup(self.patch.stop)
        self.volume = d.create_volume('unit', seed=23)

    def check_complete(self, data=None, output=None):
        data = complete(self.volume.initial) if data is None else data
        self.volume.path.write_bytes(data)
        if output is None:
            output = markers(d.scan(data))
        return d.verify_volume(self.volume,output,case='normal',before=self.volume.initial)

    def test_real_prefix_and_tombstones_match(self):
        result = self.check_complete()
        self.assertTrue(result['passed'])
        self.assertEqual(result['state']['sequence'], 8)
        self.assertEqual(result['state']['objects'][1]['revision'], 2)
        self.assertFalse(result['state']['objects'][1]['present'])

    def test_one_bit_in_each_record_byte_is_rejected(self):
        valid = record(0,0,1)
        for at in range(512):
            bad = bytearray(valid); bad[at] ^= 1
            with self.assertRaises(ValueError, msg=str(at)):
                d.parse_record(bytes(bad),0)

    def test_resealed_policy_errors_still_fail(self):
        for field, value in ((16,2),(32,4),(40,2)):
            bad = bytearray(record(0,0,1))
            struct.pack_into('<Q',bad,field,value); seal(bad)
            data = bytearray(self.volume.initial)
            data[d.FIRST*512:(d.FIRST+1)*512] = bad
            with self.assertRaises(ValueError): d.scan(data)
        data = bytearray(self.volume.initial)
        data[d.FIRST*512:(d.FIRST+1)*512] = record(0,0,1,operation=3)
        with self.assertRaises(ValueError): d.scan(data)

    def test_gap_and_stale_revisions_cannot_hide_in_valid_checksums(self):
        data = bytearray(complete(self.volume.initial))
        data[d.FIRST*512:(d.FIRST+1)*512] = bytes(512)
        with self.assertRaises(ValueError): d.scan(data)
        data = bytearray(self.volume.initial)
        data[d.FIRST*512:(d.FIRST+1)*512] = record(0,0,1)
        data[(d.FIRST+1)*512:(d.FIRST+2)*512] = record(1,0,1)
        with self.assertRaises(ValueError): d.scan(data)

    def test_canary_change_cannot_pass_with_perfect_guest_markers(self):
        data = bytearray(complete(self.volume.initial)); data[-1] ^= 1
        result = self.check_complete(bytes(data))
        self.assertFalse(result['passed'])
        self.assertFalse(result['checks']['outside_journal_unchanged'])

    def test_plausible_wrong_payload_and_matching_guest_digest_fail_scenario(self):
        data = bytearray(complete(self.volume.initial))
        at = (d.FIRST+2)*512
        data[at:at+512] = record(2,0,2,payload=b'x'*128)
        result = self.check_complete(bytes(data))
        self.assertFalse(result['passed'])
        self.assertFalse(result['checks']['record_payloads_match_native_scenario'])

    def test_missing_duplicate_unknown_and_wrong_state_markers_fail(self):
        data = complete(self.volume.initial); good = markers(d.scan(data))
        for output in ('',good+'M6B:SEQUENCE:8\n',good+'M6B:OBJECT4:REVISION:0\n',
                       good.replace('M6B:OBJECT1:REVISION:2','M6B:OBJECT1:REVISION:1')):
            self.assertFalse(self.check_complete(data,output)['passed'])

    def test_corrupt_case_must_be_unchanged_and_have_no_value_claim(self):
        v = d.create_volume('bad',seed=24,variant='corrupt')
        self.assertTrue(d.verify_volume(v,'',case='corrupt',before=v.initial)['passed'])
        self.assertFalse(d.verify_volume(v,markers(d.scan(self.volume.initial)),
                                        case='corrupt',before=v.initial)['passed'])

    def test_recovery_requires_no_disk_writes(self):
        before = complete(self.volume.initial)
        self.volume.path.write_bytes(before)
        out = markers(d.scan(before),acks=0)
        self.assertTrue(d.verify_volume(self.volume,out,case='recover',before=before)['passed'])
        self.assertFalse(d.verify_volume(self.volume,out,case='recover',before=self.volume.initial)['passed'])

    def test_manifest_identity_and_regular_file_size_are_enforced(self):
        manifest = self.volume.directory/'volume.json'
        saved = json.loads(manifest.read_text()); saved['identity']='substitute'
        manifest.write_text(json.dumps(saved))
        with self.assertRaises(ValueError): self.volume.guard()
        self.volume.path.write_bytes(bytes(32))
        with self.assertRaises(ValueError): self.volume.guard()

    def test_uncertain_mutation_requires_exact_one_real_record_without_live_state_claim(self):
        one = bytearray(record(0, 0, 1))
        struct.pack_into('<Q', one, 24, 10)
        seal(one)
        data = bytearray(self.volume.initial)
        at = d.FIRST*512
        data[at:at+512] = one
        self.volume.path.write_bytes(data)
        for case in ('write-error', 'flush-error'):
            result = d.verify_volume(self.volume, '', case=case, before=self.volume.initial)
            self.assertTrue(result['passed'])
            self.assertIsNone(result['observed'])
            self.assertEqual(result['state']['sequence'], 1)
            for output in (markers(d.scan(data), acks=0), 'M6B:ACKS:1\n', 'M6B:SEQUENCE:0\n'):
                self.assertFalse(d.verify_volume(self.volume, output, case=case,
                                                before=self.volume.initial)['passed'])
            # A cold process may report the recovered state, but must not write.
            self.assertTrue(d.verify_volume(self.volume, markers(d.scan(data), acks=0),
                case='recover', before=bytes(data))['passed'])

    def test_uncertain_mutation_rejects_replay_missing_record_and_wrong_bytes(self):
        one = bytearray(record(0, 0, 1))
        struct.pack_into('<Q', one, 24, 10)
        seal(one)
        good = bytearray(self.volume.initial)
        at = d.FIRST*512
        good[at:at+512] = one
        extra = bytearray(good)
        extra[at+512:at+1024] = record(1, 0, 2)
        canary = bytearray(good); canary[-1] ^= 1
        bad_value = bytearray(good)
        altered = bytearray(one); altered[64] ^= 1; seal(altered)
        bad_value[at:at+512] = altered
        for data in (self.volume.initial, extra, canary, bad_value):
            self.volume.path.write_bytes(data)
            for case in ('write-error', 'flush-error'):
                self.assertFalse(d.verify_volume(self.volume, '', case=case,
                                                before=self.volume.initial)['passed'])
    def test_all_labels_and_seeds_are_bounded(self):
        for label in ('../disk','/dev/null','a/b',''):
            with self.assertRaises(ValueError): d.create_volume(label)
        for seed in (-1,1<<64,True):
            with self.assertRaises(ValueError): d.synthetic_initial(seed)


if __name__ == '__main__': unittest.main()
