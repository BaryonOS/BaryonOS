"""Host-only tests for the B03 runner and the slot-store oracle. Synthetic
transcripts and stores here are never emulator evidence; they pin that the
verdict rejects what it must and that the three independent descriptions of
the store layout (update-core, kernel scope, Python oracle) agree."""
from __future__ import annotations
import re
import struct
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import run_update as ru  # noqa: E402
import update_disk as disk  # noqa: E402


def transcript(stage: str, values: dict[str, int], extra: str = "") -> str:
    plan = ru.STAGES[stage]
    lines = list(plan["loader"])
    if plan["outcome"] == "NO_VALID_SLOT":
        lines.append("BOOT:FAIL:NO_VALID_SLOT")
        return "\n".join(lines) + "\n"
    lines += ["BOOT:SIGNED:OK", *ru.BOOT]
    lines.append(f"KERNEL:IMAGE_VERSION:{values.pop('IMAGE_VERSION', 1):020d}")
    lines.append("UPDATE:MANIFEST:PASS")
    if plan["outcome"] == "CUT":
        lines.append(ru.CUT_MARKER)
        return "\n".join(lines) + "\n"
    lines.append("UPDATE:READY")
    lines += [f"UPDATE:{key}:{value:020d}" for key, value in values.items()]
    if plan["outcome"] != "INSTALLED":
        lines.append(f"UPDATE:{plan['outcome']}")
    lines += ["UPDATE:RESOURCES:PASS", "UPDATE:PASS", "M1:PASS"]
    return "\n".join(lines) + ("\n" + extra if extra else "") + "\n"


def good(stage: str) -> dict[str, int]:
    values = dict(ru.STAGES[stage]["expect"])
    values.update(ru.STAGES[stage]["at_least"])
    values.setdefault("TICKS", 1234)
    return values


def verdict(stage: str, text: str, rc=33, timed_out=False, killed=False, running=False):
    return ru.evaluate(stage, text, rc, timed_out, killed, running)


class Layout(unittest.TestCase):
    def test_rotation_oracle_requires_the_successor_manifest(self):
        image = b"synthetic image" * 400
        manifest = bytearray(disk.MANIFEST_LEN)
        manifest[:8] = disk.MANIFEST_MAGIC
        struct.pack_into("<IIIQQ", manifest, 8, 1, 1, len(image), 2, 0)
        manifest[36:68] = bytes.fromhex(disk.sha256(image))
        successor = bytearray(manifest)
        successor[68:84] = b"\x02" * 16
        before = bytearray(disk.STORE_SIZE)
        before[:8] = disk.STORE_MAGIC
        struct.pack_into("<IIQ", before, 8, 1, 0, 1)
        struct.pack_into("<9I", before, 24, *disk.TABLE)
        after = bytearray(before)
        struct.pack_into("<IIQ", after, 8, 1, 1, 2)
        at = disk.SLOT_B_MANIFEST * disk.SECTOR
        after[at:at + len(successor)] = successor
        start = disk.SLOT_B_IMAGE * disk.SECTOR
        after[start:start + len(image)] = image
        with tempfile.TemporaryDirectory() as directory:
            build = Path(directory)
            (build / "update").mkdir()
            (build / "update/kernel-v2.elf").write_bytes(image)
            (build / "update/legacy.bin").write_bytes(manifest)
            (build / "update/next.bin").write_bytes(successor)
            images = {"files": {"m2": "legacy.bin", "mn": "next.bin"},
                      "kernels": {"k2": disk.sha256(image)},
                      "manifests": {"m2": disk.sha256(manifest), "mn": disk.sha256(successor)}}
            with mock.patch.object(ru, "BUILD", build):
                ru.check_store("rotate-install", before, after, images)
                with self.assertRaisesRegex(ValueError, "exactly the v2 bundle"):
                    ru.check_store("install", before, after, images)
                after[at:at + len(manifest)] = manifest
                with self.assertRaisesRegex(ValueError, "exactly the v2 bundle"):
                    ru.check_store("rotate-install", before, after, images)
                ru.check_store("install", before, after, images)
                with self.assertRaisesRegex(ValueError, "store changed"):
                    ru.check_store("key-revoked", before, after, images)

    def test_python_table_matches_update_core_and_kernel_scope(self):
        core = (ROOT / "crates/update-core/src/lib.rs").read_text()
        capacity = int(re.search(r"pub const IMAGE_CAPACITY_SECTORS: u32 = (\d+);", core).group(1))
        self.assertEqual(capacity, disk.IMAGE_CAPACITY_SECTORS)
        self.assertEqual(int(re.search(r"pub const MANIFEST_LEN: usize = (\d+);", core).group(1)), disk.MANIFEST_LEN)
        self.assertEqual(int(re.search(r"pub const MLDSA65_SIGNATURE: usize = (\d+);", core).group(1)),
                         disk.MLDSA_SIGNATURE_LEN)
        kernel = (ROOT / "kernel/src/m2/m3/update.rs").read_text()
        self.assertEqual(int(re.search(r"const SECTORS: u64 = (\d+);", kernel).group(1)), disk.STORE_SECTORS)
        self.assertEqual(int(re.search(r"const CUT_AFTER_WRITES: u64 = (\d+);", kernel).group(1)), 64)
        self.assertEqual(disk.STORE_SECTORS, 1 + 3 * (disk.MANIFEST_SECTORS + disk.IMAGE_CAPACITY_SECTORS))

    def test_header_parse_pins_the_table_and_hint(self):
        header = bytearray(disk.SECTOR)
        header[:8] = disk.STORE_MAGIC
        struct.pack_into("<IIQ", header, 8, 1, 1, 7)
        struct.pack_into("<9I", header, 24, *disk.TABLE)
        self.assertEqual(disk.parse_header(bytes(header)), {"active": 1, "generation": 7})
        wrong = bytearray(header)
        struct.pack_into("<I", wrong, 28, disk.SLOT_A_IMAGE + 1)
        with self.assertRaisesRegex(ValueError, "table"):
            disk.parse_header(bytes(wrong))
        disk.set_active(header, 0)
        self.assertEqual(disk.parse_header(bytes(header))["active"], 0)
        with self.assertRaises(ValueError):
            disk.set_active(header, 2)

    def test_manifest_parse_is_structural_only(self):
        manifest = bytearray(disk.MANIFEST_LEN)
        manifest[:8] = disk.MANIFEST_MAGIC
        struct.pack_into("<IIIQQ", manifest, 8, 1, 1, 5000, 3, 1)
        manifest[36:68] = b"\x11" * 32
        manifest[84:148] = b"\x22" * 64
        manifest[148:148 + disk.MLDSA_SIGNATURE_LEN] = b"\x33" * disk.MLDSA_SIGNATURE_LEN
        parsed = disk.parse_manifest(bytes(manifest))
        self.assertEqual((parsed["image_length"], parsed["version"], parsed["min_version"]), (5000, 3, 1))
        self.assertIsNone(disk.parse_manifest(bytes(disk.MANIFEST_LEN)))
        manifest[-1] = 1
        with self.assertRaisesRegex(ValueError, "tail"):
            disk.parse_manifest(bytes(manifest))

    def test_analyse_and_region_helpers_on_a_synthetic_store(self):
        store = bytearray(disk.STORE_SIZE)
        store[:8] = disk.STORE_MAGIC
        struct.pack_into("<IIQ", store, 8, 1, 0, 1)
        struct.pack_into("<9I", store, 24, *disk.TABLE)
        image = bytes(range(256)) * 20  # 5120 bytes
        import hashlib
        manifest = bytearray(disk.MANIFEST_LEN)
        manifest[:8] = disk.MANIFEST_MAGIC
        struct.pack_into("<IIIQQ", manifest, 8, 1, 1, len(image), 1, 0)
        manifest[36:68] = hashlib.sha256(image).digest()
        at = disk.SLOT_A_MANIFEST * disk.SECTOR
        store[at:at + disk.MANIFEST_LEN] = manifest
        at = disk.SLOT_A_IMAGE * disk.SECTOR
        store[at:at + len(image)] = image
        facts = disk.analyse(bytes(store))
        self.assertEqual(facts["header"], {"active": 0, "generation": 1})
        self.assertTrue(facts["regions"]["A"]["image_matches_manifest"])
        self.assertEqual(facts["regions"]["A"]["image_nonzero_sectors"], 10)
        self.assertIsNone(facts["regions"]["B"]["manifest"])
        self.assertTrue(disk.image_region_is(bytes(store), "A", image))
        self.assertTrue(disk.manifest_region_is(bytes(store), "A", bytes(manifest)))
        tampered = bytearray(store)
        disk.flip(tampered, disk.image_offset("A", 100))
        self.assertFalse(disk.analyse(bytes(tampered))["regions"]["A"]["image_matches_manifest"])
        self.assertFalse(disk.slot_identical(bytes(store), bytes(tampered), "A"))
        self.assertTrue(disk.slot_identical(bytes(store), bytes(tampered), "B"))
        with self.assertRaises(ValueError):
            disk.analyse(bytes(store[:-1]))


class Verdict(unittest.TestCase):
    def test_lifecycle_rejections_require_both_consumers_and_zero_writes(self):
        for stage, reason, code, fixture in [
            ("key-revoked", "KEY_REVOKED", 7, "mr"),
            ("key-expired", "KEY_EXPIRED", 8, "me"),
            ("key-not-yet-valid", "KEY_NOT_YET_VALID", 9, "mp"),
        ]:
            plan = ru.STAGES[stage]
            self.assertEqual(plan["pack"][1]["slot-b"][0], fixture)
            self.assertEqual(plan["pack"][1]["inbox"][0], fixture)
            self.assertIn(f"BOOT:SLOT:B:REJECTED:{reason}", plan["loader"])
            self.assertEqual(plan["outcome"], f"REJECTED:{reason}")
            self.assertEqual(plan["expect"]["ACTIVE_REJECT"], code)
            self.assertEqual(plan["expect"]["INBOX_REJECT"], code)
            self.assertEqual(plan["expect"]["BROKER:WRITES"], 0)
            self.assertEqual(plan["expect"]["BROKER:FLUSHES"], 0)

    def test_every_stage_has_a_plan(self):
        self.assertEqual(set(ru.ORDER), set(ru.STAGES))
        for stage, plan in ru.STAGES.items():
            self.assertIn(plan["outcome"].split(":")[0], ("INSTALLED", "UP_TO_DATE", "REJECTED", "CUT", "NO_VALID_SLOT"))
            if not plan["fresh"]:
                self.assertEqual(ru.STAGES[plan["parent"]]["sequence"], plan["sequence"])
            expected = ru.expected_markers(stage)
            self.assertEqual(expected[-1], "BOOT:FAIL:NO_VALID_SLOT" if plan["outcome"] == "NO_VALID_SLOT"
                             else ru.CUT_MARKER if plan["outcome"] == "CUT" else "M1:PASS")

    def test_complete_transcripts_pass_for_every_stage(self):
        for stage in ru.ORDER:
            plan = ru.STAGES[stage]
            text = transcript(stage, good(stage))
            if plan["outcome"] == "CUT":
                found = verdict(stage, text, rc=-9, killed=True, running=True)
                self.assertEqual(found["IMAGE_VERSION"], 1)
            elif plan["outcome"] == "NO_VALID_SLOT":
                self.assertEqual(verdict(stage, text, rc=35), {})
            else:
                self.assertEqual(verdict(stage, text)["TICKS"], 1234)

    def test_numbers_require_twenty_digits_and_map_the_kernel_version(self):
        self.assertEqual(ru.numbers("UPDATE:RESULT:00000000000000000001\nKERNEL:IMAGE_VERSION:00000000000000000002\n"),
                         {"RESULT": 1, "IMAGE_VERSION": 2})
        self.assertEqual(ru.numbers("UPDATE:RESULT:1\n"), {})
        with self.assertRaises(ValueError):
            ru.numbers("UPDATE:RESULT:00000000000000000001\nUPDATE:RESULT:00000000000000000001\n")

    def test_failure_exit_and_order_reject(self):
        text = transcript("install", good("install"), "UPDATE:FAIL:LATE")
        with self.assertRaisesRegex(ValueError, "failure marker"):
            verdict("install", text)
        with self.assertRaisesRegex(ValueError, "raw exit"):
            verdict("install", transcript("install", good("install")), rc=35)
        with self.assertRaisesRegex(ValueError, "raw exit"):
            verdict("install", transcript("install", good("install")), timed_out=True)
        text = transcript("boot-v2", good("boot-v2")).replace("BOOT:SLOT:B:VERSION:2", "BOOT:SLOT:A:VERSION:1")
        with self.assertRaisesRegex(ValueError, "marker sequence"):
            verdict("boot-v2", text)
        with self.assertRaisesRegex(ValueError, "marker sequence"):
            verdict("downgrade", transcript("boot-v2", good("boot-v2")))
        # A cut without a real kill of a live guest is not a cut.
        with self.assertRaisesRegex(ValueError, "host kill"):
            verdict("cut", transcript("cut", good("cut")), rc=33)
        # NO_VALID_SLOT must stop before the kernel and exit with 0x11.
        with self.assertRaisesRegex(ValueError, "raw exit"):
            verdict("no-valid-slot", transcript("no-valid-slot", good("no-valid-slot")), rc=33)
        with self.assertRaisesRegex(ValueError, "marker sequence"):
            verdict("no-valid-slot", transcript("no-valid-slot", good("no-valid-slot")) + "KERNEL:ENTRY\n", rc=35)

    def test_each_expected_number_is_load_bearing(self):
        for stage in ru.ORDER:
            plan = ru.STAGES[stage]
            if plan["outcome"] in ("CUT", "NO_VALID_SLOT"):
                continue
            for key in plan["expect"]:
                values = good(stage)
                values[key] += 1
                with self.assertRaisesRegex(ValueError, re.escape(key)):
                    verdict(stage, transcript(stage, values))
            for key, minimum in plan["at_least"].items():
                values = good(stage)
                values[key] = minimum - 1
                with self.assertRaisesRegex(ValueError, re.escape(key)):
                    verdict(stage, transcript(stage, values))
            values = good(stage)
            del values["TICKS"]
            with self.assertRaisesRegex(ValueError, "tick budget"):
                verdict(stage, transcript(stage, values))

    def test_mutations_only_touch_the_named_bytes(self):
        store = bytearray(disk.STORE_SIZE)
        ru.mutate(store, [("flip_manifest", "B", disk.MLDSA_SIGNATURE_OFFSET + 100), ("flip_image", "inbox", 1000)])
        changed = [index for index, byte in enumerate(store) if byte]
        self.assertEqual(changed, [disk.manifest_offset("B", disk.MLDSA_SIGNATURE_OFFSET + 100),
                                   disk.image_offset("inbox", 1000)])
        with self.assertRaises(ValueError):
            ru.mutate(store, [("zero", 1)])


if __name__ == "__main__":
    unittest.main()
