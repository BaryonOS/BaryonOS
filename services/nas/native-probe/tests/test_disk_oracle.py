import copy
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import disk_oracle as oracle

disk = oracle.volume_disk


def snapshot():
    raw = bytearray(17216)
    raw[:8] = b"NOSNAS01"
    struct.pack_into("<H", raw, 8, 1)
    struct.pack_into("<Q", raw, 16, 6)
    raw[24:56] = b"NOSPROC1" + (11).to_bytes(8, "little") + bytes(16)
    raw[64:96] = bytes([0xA7]) * 32
    raw[96] = 4
    struct.pack_into("<Q", raw, 104, 2)
    struct.pack_into("<I", raw, 112, 128)
    raw[120:152] = b"NOSPROC1" + (12).to_bytes(8, "little") + bytes(16)
    return bytes(raw)


def image_bytes(raw=None, *, history=False, other_object=False, generation=11):
    raw = snapshot() if raw is None else raw
    with tempfile.TemporaryDirectory(prefix="nieuw-os-nas-oracle-") as directory:
        path = Path(directory) / "synthetic.img"
        disk.create_image(path)
        image = bytearray(path.read_bytes())
    chunks = [raw[i:i + disk.CHUNK] for i in range(0, len(raw), disk.CHUNK)]
    for index, chunk in enumerate(chunks):
        start = (disk.FIRST + disk.DATA_START + index * disk.CHUNK_SECTORS) * disk.SECTOR
        image[start:start + len(chunk)] = chunk
    current = {
        "revision": 10, "length": len(raw), "name": b"nas-authority-v1",
        "content_sum": disk.fnv(raw), "chunks": list(range(len(chunks))),
        "sums": [disk.fnv(chunk) for chunk in chunks],
    }
    for g in [generation - 1, generation]:
        entries = [disk.empty_entry() for _ in range(disk.OBJECTS)]
        entries[0] = dict(
            disk.empty_entry(), revision=g, request_id=g, created=1, modified=g,
            present=True, current=copy.deepcopy(current),
            history=[dict(copy.deepcopy(current), revision=8)] if history else [],
        )
        if other_object:
            entries[1] = dict(
                disk.empty_entry(), revision=1, request_id=1, created=1, modified=1,
            )
        for index, sector in enumerate(disk.encode_table(g, g, entries, clocks=[g, 0])):
            start = (disk.FIRST + (g % 2) * disk.TABLE_SECTORS + index) * disk.SECTOR
            image[start:start + disk.SECTOR] = sector
    return bytes(image)


class DiskOracleTests(unittest.TestCase):
    def test_exact_final_state_and_unchanged_cold_image_pass_without_payload_report(self):
        image = image_bytes()
        normal = oracle.check_image(image, 11, 12)
        self.assertIsNone(normal["cold_disk_unchanged"])
        self.assertEqual(normal["status"], "Evicted")
        self.assertEqual(normal["authority_history_versions"], 0)
        self.assertEqual(normal["guest_execution"], "not-assessed")
        cold = oracle.check_image(image, 11, 12, normal["image_sha256"])
        self.assertTrue(cold["cold_disk_unchanged"])
        self.assertNotIn("content_hex", json.dumps(cold))
        self.assertNotIn(snapshot().hex(), json.dumps(cold))

    def test_every_cleared_object_byte_is_required_zero(self):
        for at in range(64 + 128, 64 + oracle.NS_BYTES):
            with self.subTest(offset=at):
                raw = bytearray(snapshot())
                raw[at] = 1
                with self.assertRaises(ValueError):
                    oracle.check_snapshot(raw, 11, 12)

    def test_schema_metadata_identity_and_extra_namespace_are_not_ignored(self):
        for at in [0, 8, 10, 16, 24, 56, 96, 97, 104, 112, 116, 120, 152, 184, 4352, 17215]:
            with self.subTest(offset=at):
                raw = bytearray(snapshot())
                raw[at] ^= 1
                with self.assertRaises(ValueError):
                    oracle.check_snapshot(raw, 11, 12)
        for raw in [snapshot()[:-1], snapshot() + b"\x00"]:
            with self.assertRaises(ValueError):
                oracle.check_snapshot(raw, 11, 12)
        raw = bytearray(snapshot())
        raw[64:96] = bytes(32)
        with self.assertRaises(ValueError):
            oracle.check_snapshot(raw, 11, 12)
        for admin, guest in [(0, 12), (11, 11), (True, 12), (11, -1), (11, 1 << 64)]:
            with self.assertRaises(ValueError):
                oracle.check_snapshot(snapshot(), admin, guest)
        with self.assertRaises(ValueError):
            oracle.check_snapshot(snapshot(), 12, 11)

    def test_retained_history_and_unrelated_mutations_are_rejected(self):
        for kwargs in [{"history": True}, {"other_object": True}]:
            with self.subTest(kwargs=kwargs), self.assertRaises(ValueError):
                oracle.check_image(image_bytes(**kwargs), 11, 12)

    def test_valid_volume_checksums_do_not_hide_wrong_nas_state(self):
        raw = bytearray(snapshot())
        raw[96] = 2
        with self.assertRaisesRegex(ValueError, "NAS final"):
            oracle.check_image(image_bytes(raw), 11, 12)

    def test_canaries_chunk_integrity_and_changed_cold_disk_fail(self):
        image = image_bytes()
        for at in [0, (disk.FIRST + disk.DATA_START) * disk.SECTOR]:
            broken = bytearray(image)
            broken[at] ^= 1
            with self.subTest(offset=at), self.assertRaises(ValueError):
                oracle.check_image(broken, 11, 12)
        digest = hashlib.sha256(image).hexdigest()
        changed = bytearray(image)
        changed[-1] ^= 1
        with self.assertRaisesRegex(ValueError, "cold recovery changed"):
            oracle.check_image(changed, 11, 12, digest)
        for invalid in ["", "x" * 64, "0" * 63, digest.upper()]:
            with self.assertRaises(ValueError):
                oracle.check_image(image, 11, 12, invalid)
        with self.assertRaises(ValueError):
            oracle.check_image(image[:-1], 11, 12)

    def test_huge_canonical_revisions_cannot_enter_fixture_search(self):
        image = image_bytes(generation=(1 << 64) - 1)
        with patch.object(disk, "analyse") as analyse:
            with self.assertRaisesRegex(ValueError, "publication count"):
                oracle.check_image(image, 11, 12)
            analyse.assert_not_called()

    def test_cli_is_read_only_and_refuses_bad_image(self):
        with tempfile.TemporaryDirectory(prefix="nieuw-os-nas-oracle-cli-") as directory:
            path = Path(directory) / "synthetic.img"
            original = image_bytes()
            path.write_bytes(original)
            command = [
                sys.executable, "-B", str(oracle.HERE / "disk_oracle.py"), str(path),
                "--admin-pid", "11", "--guest-pid", "12",
            ]
            result = subprocess.run(command, capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(path.read_bytes(), original)
            self.assertEqual(json.loads(result.stdout)["status"], "Evicted")
            path.write_bytes(b"not a volume")
            result = subprocess.run(command, capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 2)
            self.assertEqual(result.stdout, "")
            self.assertIn("only a regular 1-MiB synthetic image", result.stderr)
            self.assertEqual(path.read_bytes(), b"not a volume")


if __name__ == "__main__":
    unittest.main()
