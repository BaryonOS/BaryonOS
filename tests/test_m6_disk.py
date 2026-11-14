"""Synthetic oracle tests only. These strings do not execute guest/device code."""
import json
from pathlib import Path
import shutil
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from common import BUILD
from m6_disk import (FIRST, SECTOR, SIZE, Volume, create_volume, next_value,
                     parse_record, record, scan, verify_volume)


class M6DiskOracle(unittest.TestCase):
    def volume(self, **kwargs):
        volume = create_volume("oracle-unit", seed=41, **kwargs)
        self.addCleanup(self.clean, volume.directory)
        return volume

    @staticmethod
    def clean(path):
        path.resolve(strict=True).relative_to((BUILD / "run").resolve())
        shutil.rmtree(path)

    @staticmethod
    def markers(selected):
        return (f"M6A:GENERATION:{selected['generation']}\n"
                f"M6A:REQUEST_ID:{selected['request_id']}\n"
                f"M6A:VALUE_DIGEST:{selected['value_digest']}\n")

    @staticmethod
    def append(volume, generation=2):
        data = bytearray(volume.path.read_bytes())
        at = (FIRST + generation - 1) * SECTOR
        data[at:at+SECTOR] = volume.expected_records()[generation-1]
        volume.path.write_bytes(data)
        return scan(bytes(data))

    def test_real_bytes_and_full_record_match_are_required(self):
        volume = self.volume()
        selected = self.append(volume)
        output = self.markers(selected)
        self.assertTrue(verify_volume(volume, output, allowed_generations={2})["passed"])
        self.assertFalse(verify_volume(volume, output, allowed_generations={1})["passed"])
        data = bytearray(volume.path.read_bytes()); data[10000] ^= 1; volume.path.write_bytes(data)
        self.assertFalse(verify_volume(volume, output, allowed_generations={2})["passed"])

    def test_no_marker_forgery_duplicate_or_hybrid_payload(self):
        volume = self.volume(); selected = self.append(volume); output = self.markers(selected)
        for invalid in ("M6A:PASS\n", output+output, output.replace("GENERATION:2", "GENERATION:2-bogus"),
                        output.replace("VALUE_DIGEST:", "VALUE_DIGEST_BAD:"),
                        output.replace(selected["value_digest"], "0"*16)):
            self.assertFalse(verify_volume(volume, invalid, allowed_generations={2})["passed"])
        data = bytearray(volume.path.read_bytes()); data[(FIRST+1)*SECTOR+80] ^= 1
        volume.path.write_bytes(data)
        self.assertFalse(verify_volume(volume, output, allowed_generations={1, 2})["passed"])

    def test_torn_prefix_is_not_a_valid_new_record(self):
        volume = self.volume(variant="torn-tail")
        self.assertIsNone(parse_record(volume.initial[(FIRST+1)*SECTOR:(FIRST+2)*SECTOR], 1))
        old = scan(volume.initial); self.assertEqual(old["generation"], 1)
        self.assertTrue(verify_volume(volume, self.markers(old), allowed_generations={1}, unchanged=True)["passed"])

    def test_unknown_allows_only_old_new_bytes_and_never_a_success_value(self):
        volume = self.volume(); data = bytearray(volume.initial)
        new = volume.expected_records()[1]; at = (FIRST+1)*SECTOR
        data[at:at+200] = new[:200]; volume.path.write_bytes(data)
        self.assertTrue(verify_volume(volume, "M6A:OUTCOME:UNKNOWN\n", allowed_generations={1, 2},
                                      data_required=False, allow_torn=True)["passed"])
        data[at+300] = 0x91; volume.path.write_bytes(data)
        self.assertFalse(verify_volume(volume, "", allowed_generations={1, 2}, data_required=False,
                                       allow_torn=True)["passed"])

    def test_corrupt_all_is_not_silently_initialized(self):
        volume = self.volume(variant="corrupt-all")
        self.assertIsNone(scan(volume.initial))
        self.assertTrue(verify_volume(volume, "", allowed_generations={None}, data_required=False,
                                      unchanged=True)["passed"])
        changed = bytearray(volume.initial); changed[FIRST*SECTOR:(FIRST+1)*SECTOR] = record(0, bytes(128))
        volume.path.write_bytes(changed)
        self.assertFalse(verify_volume(volume, "", allowed_generations={None}, data_required=False)["passed"])

    def test_scratch_is_exact_and_recovery_must_not_mutate(self):
        volume = self.volume(scratch=True); selected = self.append(volume)
        data = bytearray(volume.path.read_bytes()); data[12*SECTOR:13*SECTOR] = bytes([0x3c])*255+bytes([0xa7])*257
        volume.path.write_bytes(data); before = bytes(data); output = self.markers(selected)
        self.assertTrue(verify_volume(volume, output, allowed_generations={2}, unchanged=True, before=before)["passed"])
        data[12*SECTOR] = 0xe1; volume.path.write_bytes(data)
        self.assertFalse(verify_volume(volume, output, allowed_generations={2})["passed"])

    def test_path_and_identity_fail_closed(self):
        for label in ("../external", "C:/disk", "a/b", "", "a," ):
            with self.assertRaises(ValueError): create_volume(label, seed=0)
        volume = self.volume()
        fake = Volume(BUILD.resolve(), volume.seed, volume.initial, "normal", False, volume.identity)
        with self.assertRaises(ValueError): fake.guard()
        meta = volume.directory / "volume.json"; content = json.loads(meta.read_text())
        content["identity"] = "other"; meta.write_text(json.dumps(content))
        with self.assertRaises(ValueError): volume.guard()

    def test_size_seed_and_record_bounds(self):
        volume = self.volume(); self.assertEqual(len(volume.initial), SIZE)
        for seed in (-1, 1 << 64, True):
            with self.assertRaises(ValueError): create_volume("bad-seed", seed=seed)
        self.assertIsNone(parse_record(volume.initial[FIRST*SECTOR:(FIRST+1)*SECTOR], 4))
        with self.assertRaises(ValueError): next_value(bytes(128), 5)
        with self.assertRaises(ValueError): scan(bytes(512))

    def test_full_requires_every_committed_prefix_record(self):
        volume = self.volume(); selected = self.append(volume, 4)
        output = self.markers(selected)
        self.assertFalse(verify_volume(volume, output, allowed_generations={4}, require_full_prefix=True)["passed"])
        data = bytearray(volume.path.read_bytes())
        for slot, raw in enumerate(volume.expected_records()):
            data[(FIRST+slot)*SECTOR:(FIRST+slot+1)*SECTOR] = raw
        volume.path.write_bytes(data)
        self.assertTrue(verify_volume(volume, output, allowed_generations={4}, require_full_prefix=True)["passed"])

    def test_recovery_retains_a_previously_permitted_torn_tail_exactly(self):
        volume = self.volume(); data = bytearray(volume.initial); at = (FIRST+1)*SECTOR
        data[at:at+200] = volume.expected_records()[1][:200]; volume.path.write_bytes(data)
        before = bytes(data); output = self.markers(scan(before))
        self.assertTrue(verify_volume(volume, output, allowed_generations={1}, allow_torn=True,
                                     unchanged=True, before=before)["passed"])
        data[at+10] ^= 1; volume.path.write_bytes(data)
        self.assertFalse(verify_volume(volume, output, allowed_generations={1}, allow_torn=True,
                                      unchanged=True, before=before)["passed"])

    def test_new_volume_is_not_evidence_of_a_previous_run(self):
        from run_m6 import previous_stage
        with self.assertRaises(FileNotFoundError): previous_stage(self.volume())


if __name__ == "__main__":
    unittest.main()
