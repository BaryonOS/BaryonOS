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
    struct.pack_into("<Q", raw, 16, 11)
    raw[24:56] = b"NOSPROC1" + (11).to_bytes(8, "little") + bytes(16)
    raw[64:96] = bytes([0xA7]) * 32
    raw[96] = 4
    struct.pack_into("<Q", raw, 104, 2)
    struct.pack_into("<I", raw, 112, 256)
    raw[120:152] = b"NOSPROC1" + (12).to_bytes(8, "little") + bytes(16)
    at = 64 + oracle.NS_BYTES
    raw[at:at + 32] = bytes([0xB8]) * 32
    raw[at + 32] = 4
    struct.pack_into("<Q", raw, at + 40, 2)
    struct.pack_into("<I", raw, at + 48, 128)
    raw[at + 56:at + 88] = b"NOSPROC1" + (12).to_bytes(8, "little") + bytes(16)
    return bytes(raw)


def image_bytes(raw=None, *, history=False, other_object=False, generation=21):
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
        "revision": 20, "length": len(raw), "name": b"nas-authority-v1",
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


NAMESPACE = bytes([0xC9]) * 32


def interrupted_image(stage, *, current=None, retained=None, generation=None):
    """Synthetic volume with the exact interrupted-purge versions written."""
    current = oracle.interrupt_snapshot(NAMESPACE, 11, 12, generation=6, revoked_copy=False) \
        if current is None else current
    if retained is None and stage == "interrupt":
        retained = oracle.interrupt_snapshot(NAMESPACE, 11, 12, generation=5, revoked_copy=True)
    generation = oracle.INTERRUPT_VOLUME_GENERATION[stage] if generation is None else generation
    with tempfile.TemporaryDirectory(prefix="nieuw-os-nas-oracle-") as directory:
        path = Path(directory) / "synthetic.img"
        disk.create_image(path)
        image = bytearray(path.read_bytes())
    next_chunk = 0

    def place(raw, revision):
        nonlocal next_chunk
        chunks = [raw[i:i + disk.CHUNK] for i in range(0, len(raw), disk.CHUNK)]
        indices = list(range(next_chunk, next_chunk + len(chunks)))
        next_chunk += len(chunks)
        for index, chunk in zip(indices, chunks):
            start = (disk.FIRST + disk.DATA_START + index * disk.CHUNK_SECTORS) * disk.SECTOR
            image[start:start + len(chunk)] = chunk
        return {"revision": revision, "length": len(raw), "name": b"nas-authority-v1",
                "content_sum": disk.fnv(raw), "chunks": indices,
                "sums": [disk.fnv(chunk) for chunk in chunks]}

    head = place(current, generation if stage == "interrupt" else generation - 1)
    history = [place(retained, generation - 2)] if retained is not None else []
    for g in [generation - 1, generation]:
        entries = [disk.empty_entry() for _ in range(disk.OBJECTS)]
        entries[0] = dict(disk.empty_entry(), revision=g, request_id=g, created=1, modified=g,
                          present=True, current=dict(head, revision=min(head["revision"], g)),
                          history=copy.deepcopy(history))
        for index, sector in enumerate(disk.encode_table(g, g, entries, clocks=[g, 0])):
            start = (disk.FIRST + (g % 2) * disk.TABLE_SECTORS + index) * disk.SECTOR
            image[start:start + disk.SECTOR] = sector
    return bytes(image)


def resume_snapshot(revision, content, core, *, status=2, epoch=1, quota=128, guest=12):
    """Independent copy of the P02-T3 single-namespace layout (not the oracle's)."""
    raw = bytearray(17216)
    raw[:8] = b"NOSNAS01"
    struct.pack_into("<H", raw, 8, 1)
    struct.pack_into("<Q", raw, 16, core)
    raw[24:56] = b"NOSPROC1" + (11).to_bytes(8, "little") + bytes(16)
    raw[64:96] = NAMESPACE
    raw[96] = status
    struct.pack_into("<Q", raw, 104, epoch)
    struct.pack_into("<I", raw, 112, quota)
    raw[120:152] = b"NOSPROC1" + guest.to_bytes(8, "little") + bytes(16)
    struct.pack_into("<QH", raw, 192, revision, len(content))
    raw[208:208 + len(content)] = content
    return bytes(raw)


FIRST = bytes((i * 17 + 9) & 0xFF for i in range(96))
EDIT = bytes((i * 29 + 3) & 0xFF for i in range(32))


def resume_image(stage, **changes):
    revision, content, core = {"offline": (1, FIRST, 4), "resume": (2, EDIT, 5)}[stage]
    generation = changes.pop("generation", 1 + 2 * (core - 1))
    retained = changes.pop("retained", None)
    current = resume_snapshot(*changes.pop("current", (revision, content, core)), **changes)
    # The "repair" layout is a current version without history (head = generation - 1).
    return interrupted_image("repair", current=current, retained=retained, generation=generation)


class OfflineResumeOracleTests(unittest.TestCase):
    def test_exact_offline_and_resume_states_pass(self):
        offline = oracle.check_resume_image(resume_image("offline"), 11, 12, "offline")
        resume = oracle.check_resume_image(resume_image("resume"), 11, 12, "resume")
        self.assertEqual((offline["core_generation"], offline["volume_generation"],
                          offline["object_revision"], offline["stored_ciphertext_bytes"]),
                         (4, 7, 1, 96))
        self.assertEqual((resume["core_generation"], resume["volume_generation"],
                          resume["object_revision"], resume["stored_ciphertext_bytes"]),
                         (5, 9, 2, 32))
        self.assertNotIn(EDIT.hex(), json.dumps(resume))

    def test_unreconciled_replayed_or_altered_states_are_rejected(self):
        for stage, image in [
            ("resume", resume_image("offline")),            # resume never committed
            ("offline", resume_image("resume")),
            ("resume", resume_image("resume", current=(2, FIRST, 5))),  # wrong bytes
            ("resume", resume_image("resume", current=(3, EDIT, 6), generation=11)),  # replay
            ("resume", resume_image("resume", generation=11)),
            ("resume", resume_image("resume", retained=resume_snapshot(1, FIRST, 4))),
            ("resume", resume_image("resume", status=4)),
            ("resume", resume_image("resume", epoch=2)),
            ("resume", resume_image("resume", quota=256)),
            ("resume", resume_image("resume", guest=13)),
        ]:
            with self.subTest(stage=stage), self.assertRaises(ValueError):
                oracle.check_resume_image(image, 11, 12, stage)
        with self.assertRaises(ValueError):
            oracle.check_resume_image(resume_image("resume"), 11, 12, "repair")
        with self.assertRaises(ValueError):
            oracle.check_resume_image(resume_image("resume"), 11, 11, "resume")


class InterruptedPurgeOracleTests(unittest.TestCase):
    def test_exact_interrupt_and_repair_states_pass(self):
        interrupt = oracle.check_interrupted_image(interrupted_image("interrupt"), 11, 12, "interrupt")
        self.assertEqual((interrupt["authority_history_versions"],
                          interrupt["retained_ciphertext_bytes"]), (1, 96))
        repair = oracle.check_interrupted_image(interrupted_image("repair"), 11, 12, "repair")
        self.assertEqual((repair["authority_history_versions"],
                          repair["retained_ciphertext_bytes"]), (0, 0))
        self.assertEqual(repair["physical_erasure"], "not-assessed")
        self.assertNotIn(oracle.INTERRUPT_CONTENT.hex(), json.dumps(interrupt))

    def test_unrepaired_history_and_missing_history_are_stage_bound(self):
        retained = oracle.interrupt_snapshot(NAMESPACE, 11, 12, generation=5, revoked_copy=True)
        for stage, image in [
                ("repair", interrupted_image("repair", retained=retained)),
                ("interrupt", interrupted_image("repair", generation=10)),
                ("interrupt", interrupted_image("repair")),
                ("repair", interrupted_image("interrupt"))]:
            with self.subTest(stage=stage), self.assertRaises(ValueError):
                oracle.check_interrupted_image(image, 11, 12, stage)

    def test_revived_authority_or_altered_copy_is_rejected(self):
        tombstone = oracle.interrupt_snapshot(NAMESPACE, 11, 12, generation=6, revoked_copy=False)
        revoked = oracle.interrupt_snapshot(NAMESPACE, 11, 12, generation=5, revoked_copy=True)
        mutations = []
        for at in (16, 96, 104, 112, 120, 64 + 128):
            raw = bytearray(tombstone)
            raw[at] ^= 1
            mutations.append(("current", bytes(raw)))
        # Repair must not settle on the older revoked copy as current state.
        mutations.append(("current", revoked))
        for at in (96, 64 + 128 + 16, 64 + 128 + 16 + 95):
            raw = bytearray(revoked)
            raw[at] ^= 1
            mutations.append(("retained", bytes(raw)))
        for which, raw in mutations:
            stage = "interrupt" if which == "retained" else "repair"
            kwargs = {which: raw}
            with self.subTest(which=which), self.assertRaises(ValueError):
                oracle.check_interrupted_image(interrupted_image(stage, **kwargs), 11, 12, stage)

    def test_unknown_stage_identity_and_cli_flags_fail_closed(self):
        with self.assertRaises(ValueError):
            oracle.check_interrupted_image(interrupted_image("repair"), 11, 12, "cold")
        with self.assertRaises(ValueError):
            oracle.check_interrupted_image(interrupted_image("repair"), 12, 11, "repair")
        with tempfile.TemporaryDirectory(prefix="nieuw-os-nas-oracle-cli-") as directory:
            path = Path(directory) / "synthetic.img"
            path.write_bytes(interrupted_image("repair"))
            base = [sys.executable, "-B", str(oracle.HERE / "disk_oracle.py"), str(path),
                    "--admin-pid", "11", "--guest-pid", "12", "--stage", "repair"]
            result = subprocess.run(base, capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json.loads(result.stdout)["authority_history_versions"], 0)
            result = subprocess.run(base + ["--normal-sha256", "0" * 64],
                                    capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 2)


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
        for at in [start + relative for start in (64, 64 + oracle.NS_BYTES)
                   for relative in range(128, oracle.NS_BYTES)]:
            with self.subTest(offset=at):
                raw = bytearray(snapshot())
                raw[at] = 1
                with self.assertRaises(ValueError):
                    oracle.check_snapshot(raw, 11, 12)

    def test_schema_metadata_identity_and_extra_namespace_are_not_ignored(self):
        for at in [0, 8, 10, 16, 24, 56, 96, 97, 104, 112, 116, 120, 152, 184, 8640, 17215]:
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

    def test_both_tombstones_are_required_distinct_and_exact(self):
        for offset in (0, 32, 40, 48, 56, 88, 127):
            raw = bytearray(snapshot())
            at = 64 + oracle.NS_BYTES
            if offset == 0:
                raw[at:at + 32] = bytes(32)
            else:
                raw[at + offset] ^= 1
            with self.subTest(offset=offset), self.assertRaises(ValueError):
                oracle.check_snapshot(raw, 11, 12)
        raw = bytearray(snapshot())
        raw[64 + oracle.NS_BYTES:96 + oracle.NS_BYTES] = raw[64:96]
        with self.assertRaises(ValueError):
            oracle.check_snapshot(raw, 11, 12)

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
