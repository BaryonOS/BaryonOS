"""Host-only adversarial tests of an independent named-document disk decoder."""
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import m6c_disk as d
import m6b_disk as legacy


def record(slot, object_id, revision, *, operation=2, payload=None, name=None):
    if payload is None:
        payload = b"" if operation == 3 else d.fixture_value(object_id, revision - (operation == 4))
    if name is None:
        name = b"" if operation == 3 else d.fixture_name(object_id, revision)
    out = bytearray(512)
    out[:8] = d.MAGIC
    struct.pack_into("<HHIQQQQHHH", out, 8, 2, 64, 0, slot+1, slot+1,
                     object_id, revision, operation, len(payload), len(name))
    out[64:64+len(payload)] = payload
    out[192:192+len(name)] = name
    seal(out)
    return bytes(out)


def seal(data):
    data[504:] = bytes(8)
    struct.pack_into("<Q", data, 504, int(d.digest(data), 16))


def complete(initial):
    out = bytearray(initial)
    mutations = [(0, 1, 2), (1, 1, 2), (0, 2, 4), (1, 2, 3), (0, 3, 2),
                 (2, 1, 2), (3, 1, 2), (2, 2, 4), (3, 2, 3), (2, 3, 2)]
    for slot, (obj, rev, op) in enumerate(mutations):
        at = (d.FIRST+slot)*512
        out[at:at+512] = record(slot, obj, rev, operation=op)
    return bytes(out)


def markers(state, acks=10, rejected=6, catalogs=10):
    lines = [f'M6C:SEQUENCE:{state["sequence"]}']
    for i, obj in enumerate(state["objects"]):
        for field in ("revision", "present", "length", "digest", "name_length", "name_digest"):
            value = int(obj[field]) if field == "present" else obj[field]
            lines.append(f"M6C:OBJECT{i}:{field.upper()}:{value}")
    lines += [f"M6C:ACKS:{acks}", f"M6C:REJECTED_NO_IO:{rejected}", f"M6C:CATALOG_REPLIES:{catalogs}"]
    for domain in range(2):
        revision = sum(obj["revision"] for obj in state["objects"][2*domain:2*domain+2])
        lines.append(f"M6C:DOMAIN{domain}:REVISION:{revision}")
    return "\n".join(lines)+"\n"


class NamedDiskOracle(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="m6c-oracle-")
        self.addCleanup(temporary.cleanup)
        patcher = patch.object(d, "BUILD", Path(temporary.name))
        patcher.start()
        self.addCleanup(patcher.stop)
        self.volume = d.create_volume("unit", seed=23)

    def check_complete(self, data=None, output=None):
        data = complete(self.volume.initial) if data is None else data
        self.volume.path.write_bytes(data)
        if output is None:
            output = markers(d.scan(data))
        return d.verify_volume(self.volume, output, case="normal", before=self.volume.initial)

    def test_named_crud_rename_and_tombstones_match(self):
        result = self.check_complete()
        self.assertTrue(result["passed"], result)
        self.assertEqual(result["state"]["sequence"], 10)
        self.assertEqual(result["state"]["objects"][0]["name_digest"], d.digest(d.fixture_name(0, 3)))
        self.assertEqual(result["state"]["objects"][1], d.object_summary(2))

    def test_each_record_bitflip_is_rejected(self):
        valid = record(0, 0, 1)
        for at in range(512):
            bad = bytearray(valid)
            bad[at] ^= 1
            with self.assertRaises(ValueError, msg=str(at)):
                d.parse_record(bytes(bad), 0)

    def test_resealed_noncanonical_lengths_names_and_padding_are_rejected(self):
        for at, value in ((8, 1), (52, 33), (54, 1), (224, 1), (511, 1)):
            bad = bytearray(record(0, 0, 1))
            bad[at] = value
            if at != 511:
                seal(bad)
            with self.subTest(offset=at), self.assertRaises(ValueError):
                d.parse_record(bytes(bad), 0)
        for name in (b"", b"a"*33, b"\xff", b"a/b", b"a\\b", b"\0", b"..",
                     "a\u202eb".encode(), "a\u2066b".encode(), "a\u0085b".encode()):
            with self.subTest(name=name), self.assertRaises(ValueError):
                d.parse_record(record(0, 0, 1, name=name), 0)
        for name in ("é"*16, "編輯", "e\u0301", "a b"):
            self.assertTrue(d.valid_name(name.encode()))

    def test_checksums_cannot_hide_rename_content_change(self):
        data = bytearray(complete(self.volume.initial))
        at = (d.FIRST+2)*512
        data[at:at+512] = record(2, 0, 2, operation=4, payload=d.fixture_value(0, 2))
        with self.assertRaisesRegex(ValueError, "rename"):
            d.scan(data)

    def test_rename_and_delete_require_live_predecessor(self):
        for op in (3, 4):
            data = bytearray(self.volume.initial)
            data[d.FIRST*512:(d.FIRST+1)*512] = record(0, 0, 1, operation=op)
            with self.assertRaises(ValueError):
                d.scan(data)

    def test_tombstone_cannot_retain_name_or_content(self):
        for kwargs in ({"name": b"old"}, {"payload": b"old"}):
            with self.assertRaises(ValueError):
                d.parse_record(record(1, 0, 2, operation=3, **kwargs), 1)

    def test_gap_or_stale_revision_is_not_a_valid_prefix(self):
        data = bytearray(complete(self.volume.initial))
        data[d.FIRST*512:(d.FIRST+1)*512] = bytes(512)
        with self.assertRaises(ValueError):
            d.scan(data)
        data = bytearray(complete(self.volume.initial))
        data[(d.FIRST+2)*512:(d.FIRST+3)*512] = record(2, 0, 1)
        with self.assertRaises(ValueError):
            d.scan(data)

    def test_legacy_fixture_is_valid_v1_and_explicitly_rejected_by_v2(self):
        record_v1 = d.legacy_record()
        self.assertEqual(legacy.parse_record(record_v1, 0)["revision"], 1)
        with self.assertRaises(ValueError):
            d.parse_record(record_v1, 0)
        volume = d.create_volume("legacy", seed=24, variant="legacy")
        self.assertTrue(d.verify_volume(volume, "", case="legacy", before=volume.initial)["passed"])
        self.assertFalse(d.verify_volume(volume, "", case="normal", before=volume.initial)["passed"])

    def test_perfect_markers_do_not_hide_canary_damage(self):
        data = bytearray(complete(self.volume.initial))
        data[-1] ^= 1
        result = self.check_complete(bytes(data))
        self.assertFalse(result["passed"])
        self.assertFalse(result["checks"]["outside_journal_unchanged"])

    def test_self_consistent_wrong_label_fails_fixture_oracle(self):
        data = bytearray(complete(self.volume.initial))
        at = (d.FIRST+4)*512
        data[at:at+512] = record(4, 0, 3, name=b"wrong label")
        result = self.check_complete(bytes(data))
        self.assertFalse(result["passed"])
        self.assertFalse(result["checks"]["record_payloads_and_names_match_native_scenario"])

    def test_missing_duplicate_unknown_wrong_metadata_and_catalog_count_fail(self):
        data = complete(self.volume.initial)
        good = markers(d.scan(data))
        for output in ("", good+"M6C:SEQUENCE:10\n", good+"M6C:OBJECT4:REVISION:0\n",
                       good.replace("M6C:OBJECT1:REVISION:2", "M6C:OBJECT1:REVISION:1"),
                       good.replace("M6C:DOMAIN0:REVISION:5", "M6C:DOMAIN0:REVISION:10"),
                       good.replace("M6C:CATALOG_REPLIES:10", "M6C:CATALOG_REPLIES:9")):
            self.assertFalse(self.check_complete(data, output)["passed"])

    def test_cold_recovery_has_no_new_mutation_or_ack(self):
        before = complete(self.volume.initial)
        self.volume.path.write_bytes(before)
        out = markers(d.scan(before), acks=0, catalogs=2)
        self.assertTrue(d.verify_volume(self.volume, out, case="recover", before=before)["passed"])
        self.assertFalse(d.verify_volume(self.volume, out, case="recover", before=self.volume.initial)["passed"])

    def test_crash_before_and_after_rename_preserve_whole_names_and_content(self):
        for point, revision in (("before", 1), ("after", 2)):
            volume = d.create_volume("crash-"+point, seed=25)
            data = bytearray(volume.initial)
            data[d.FIRST*512:(d.FIRST+1)*512] = record(0, 0, 1)
            if revision == 2:
                data[(d.FIRST+1)*512:(d.FIRST+2)*512] = record(1, 0, 2, operation=4)
            volume.path.write_bytes(data)
            result = d.verify_volume(volume, "", case="vm-cut-"+point+"-rename", before=volume.initial)
            self.assertTrue(result["passed"], result)

    def test_corruption_rejection_must_not_mutate_image(self):
        volume = d.create_volume("corrupt", seed=26, variant="corrupt")
        self.assertTrue(d.verify_volume(volume, "", case="corrupt", before=volume.initial)["passed"])
        data = bytearray(volume.initial)
        data[d.FIRST*512+1] ^= 1
        volume.path.write_bytes(data)
        self.assertFalse(d.verify_volume(volume, "", case="corrupt", before=volume.initial)["passed"])

    def test_volume_manifest_and_size_cannot_be_substituted(self):
        path = self.volume.directory/"volume.json"
        saved = json.loads(path.read_text())
        saved["identity"] = "other"
        path.write_text(json.dumps(saved))
        with self.assertRaises(ValueError):
            self.volume.guard()

    def test_runner_oracle_binds_case_variant_and_fresh_before_bytes(self):
        data = complete(self.volume.initial)
        self.volume.path.write_bytes(data)
        report = d.verify_volume(self.volume, markers(d.scan(data)), case="normal", before=data)
        self.assertFalse(report["passed"])
        self.assertFalse(report["checks"]["fresh_mutator_before_image"])
        volume = d.create_volume("legacy-variant", seed=27, variant="legacy")
        report = d.verify_volume(volume, "", case="corrupt", before=volume.initial)
        self.assertFalse(report["passed"])
        self.assertFalse(report["checks"]["case_matches_volume_variant"])
        self.volume.path.write_bytes(bytes(32))
        with self.assertRaises(ValueError):
            self.volume.guard()

    def test_labels_seeds_and_case_are_bounded(self):
        for label in ("../disk", "/dev/null", "a/b", ""):
            with self.assertRaises(ValueError):
                d.create_volume(label)
        for seed in (-1, 1 << 64, True):
            with self.assertRaises(ValueError):
                d.synthetic_initial(seed)
        with self.assertRaises(ValueError):
            d.verify_volume(self.volume, "", case="unknown", before=self.volume.initial)


if __name__ == "__main__":
    unittest.main()
