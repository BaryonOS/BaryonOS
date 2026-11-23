"""Host-only tests for the REL-05 characterization runner. Synthetic
transcripts and stores are never emulator evidence; they pin that the verdict
accepts only the floor-before-health counterexample and rejects the transcript
a fixed loader would produce."""
from __future__ import annotations
import hashlib
import struct
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import characterize_update_floor as cf  # noqa: E402
import update_disk as disk  # noqa: E402


def transcript(name: str, lines=None) -> str:
    noise = ["BdsDxe: loading Boot0001", "unrelated firmware text"]
    return "\r\n".join(noise + list(cf.expected_lines(name) if lines is None else lines)) + "\r\n"


def manifest(image: bytes, version: int) -> bytes:
    """Structural manifest fields only; the host oracle does not check signatures."""
    data = bytearray(disk.MANIFEST_LEN)
    data[:8] = disk.MANIFEST_MAGIC
    struct.pack_into("<IIIQQ", data, 8, disk.FORMAT_VERSION, 1, len(image), version, 0)
    data[36:68] = hashlib.sha256(image).digest()
    return bytes(data)


def store(active: int, files: dict[str, bytes]) -> bytes:
    data = bytearray(disk.STORE_SIZE)
    data[:8] = disk.STORE_MAGIC
    struct.pack_into("<IIQ9I", data, 8, disk.FORMAT_VERSION, active, 1, *disk.TABLE)
    for slot, version in (("A", 1), ("B", 2)):
        manifest_sector, image_sector = disk.REGIONS[slot]
        for sector, blob in ((manifest_sector, files[f"m{version}"]), (image_sector, files[f"k{version}"])):
            data[sector * disk.SECTOR:sector * disk.SECTOR + len(blob)] = blob
    return bytes(data)


IMAGES = {"k1": b"\x7fELF-v1" * 300, "k2": b"\x7fELF-v2" * 400}
FILES = {**IMAGES, "m1": manifest(IMAGES["k1"], 1), "m2": manifest(IMAGES["k2"], 2)}


class Plan(unittest.TestCase):
    def test_one_persistent_hazard_sequence_after_an_independent_control(self):
        self.assertEqual([b["name"] for b in cf.BOOTS],
                         ["control-v1", "boot-1-v2-panics", "boot-2-floor-kept", "boot-3-rollback-refused"])
        self.assertEqual([(b["state"], b["fresh"]) for b in cf.BOOTS],
                         [("control", True), ("hazard", True), ("hazard", False), ("hazard", False)])
        self.assertEqual([b["active"] for b in cf.BOOTS], [0, 1, 1, 0])
        self.assertIn("not a fix", cf.LABEL)
        self.assertIn("no progress points", cf.LABEL)

    def test_exact_expected_markers(self):
        self.assertEqual(cf.expected_lines("boot-1-v2-panics"),
                         ["BOOT:FLOOR:0", "BOOT:SLOT:B:VERSION:2", "BOOT:SIGNED:OK", "BOOT:EBS_OK",
                          "KERNEL:ENTRY", "BOOTINFO:OK", "M1:FAIL:KERNEL_PANIC"])
        for name in ("boot-2-floor-kept", "boot-3-rollback-refused"):
            self.assertEqual(cf.expected_lines(name)[:3],
                             ["BOOT:FLOOR:2", "BOOT:SLOT:A:REJECTED:DOWNGRADE", "BOOT:SLOT:B:VERSION:2"])
        self.assertEqual(cf.expected_lines("control-v1")[-1], "M1:PASS")

    def test_floor_variable_needle_is_utf16(self):
        self.assertTrue(cf.floor_variable_present(b"\0" + "NieuwOsMinVersion".encode("utf-16-le") + b"\0"))
        self.assertFalse(cf.floor_variable_present(b"NieuwOsMinVersion"))


class Transcript(unittest.TestCase):
    def test_counterexample_is_accepted(self):
        for boot in cf.BOOTS:
            rc = 33 if boot["healthy"] else 35
            self.assertEqual(cf.evaluate(boot["name"], transcript(boot["name"]), rc, False),
                             cf.expected_lines(boot["name"]))

    def test_fixed_loader_transcripts_are_rejected(self):
        # What a loader that defers the floor until health would print.
        fixed = {"boot-2-floor-kept": ["BOOT:FLOOR:0", "BOOT:SLOT:B:VERSION:2"],
                 "boot-3-rollback-refused": ["BOOT:FLOOR:1", "BOOT:SLOT:A:VERSION:1"]}
        for name, loader in fixed.items():
            tail = ["BOOT:SIGNED:OK", "BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK"]
            ending = "M1:PASS" if loader[-1].endswith(":1") else "M1:FAIL:KERNEL_PANIC"
            with self.assertRaises(ValueError):
                cf.evaluate(name, transcript(name, loader + tail + [ending]), 33 if ending == "M1:PASS" else 35,
                            False)

    def test_wrong_exit_timeout_or_markers_are_rejected(self):
        name = "boot-2-floor-kept"
        good = cf.expected_lines(name)
        cases = [
            (transcript(name), 33, False),  # a healthy exit code with a panic transcript
            (transcript(name), 35, True),  # timeout
            (transcript(name, good[:-1]), 35, False),  # no panic line: a loader failure also exits 35
            (transcript(name, good + ["M1:PASS"]), 35, False),
            (transcript(name, ["BOOT:FAIL:NO_VALID_SLOT"]), 35, False),
            (transcript(name, [good[1], good[0], *good[2:]]), 35, False),
            (transcript(name, [*good[:3], "BOOT:FAIL:FLOOR_WRITE", *good[3:]]), 35, False),
            (transcript(name, [line + "-x" for line in good]), 35, False),
        ]
        for text, rc, timed_out in cases:
            with self.assertRaises(ValueError):
                cf.evaluate(name, text, rc, timed_out)
        with self.assertRaises(ValueError):
            cf.evaluate("control-v1", transcript("control-v1"), 35, False)


class Store(unittest.TestCase):
    def test_intact_unchanged_store_is_accepted(self):
        for boot in cf.BOOTS:
            data = store(boot["active"], FILES)
            oracle = cf.check_store(boot["name"], data, data, FILES)
            self.assertEqual(oracle["regions"]["A"]["manifest"]["version"], 1)
            self.assertEqual(oracle["regions"]["B"]["manifest"]["version"], 2)

    def test_store_must_be_unchanged_and_header_as_set(self):
        name = "boot-3-rollback-refused"
        before = store(0, FILES)
        for region, byte in (("B", 3), ("inbox", 0)):  # the inbox is caught only by before == after
            changed = bytearray(before)
            disk.flip(changed, disk.image_offset(region, byte))
            with self.assertRaises(ValueError):
                cf.check_store(name, before, bytes(changed), FILES)
        with self.assertRaises(ValueError):
            cf.check_store(name, store(1, FILES), store(1, FILES), FILES)

    def test_slot_bundles_must_match_exactly(self):
        swapped = store(0, dict(FILES, k1=FILES["k2"], m1=FILES["m2"]))
        with self.assertRaises(ValueError):
            cf.check_store("control-v1", swapped, swapped, FILES)
        stale = store(0, dict(FILES, m1=manifest(FILES["k2"], 1)))  # digest no longer matches k1
        with self.assertRaises(ValueError):
            cf.check_store("control-v1", stale, stale, dict(FILES, m1=manifest(FILES["k2"], 1)))


if __name__ == "__main__":
    unittest.main()
