"""Host tests for the independent S09 destination oracle (tools/destination_disk.py, R3).

Requirement: docs/implementation/S09-DESTINATION-SERVICE.md 2.6 (copy validity,
mount rule) and 8.2 R3. Every refusal test changes one field, recomputes the
FNV-64 and asserts the rule the oracle names, so a pass cannot come from the
checksum alone. The cross-check class reads images written by the Rust host
core (services/destination/tests/oracle_images.rs) when DEST_ORACLE_OUT is set.
"""
from __future__ import annotations
import json
import os
import struct
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parents[1] / "tools"
sys.path.insert(0, str(TOOLS))
import destination_disk as dd  # noqa: E402

SECTORS = 96
X = b"dest-test-id-001"
Y = b"other-dest-id-02"


def archive(length: int, seed: int) -> bytes:
    body = bytes((seed + i * 7) & 0xFF for i in range(length - 8))
    return body + struct.pack("<Q", dd.fnv(body))


def records(*lengths: int, sealed_from: int = 2) -> list[dict]:
    out, first = [], dd.DATA_START
    for index, length in enumerate(lengths):
        data = archive(length, index)
        out.append({"id": index + 1, "first": first, "length": length,
                    "trailer": struct.unpack_from("<Q", data, length - 8)[0],
                    "sealed_sequence": sealed_from + index, "label": b"backup".hex()})
        first += -(-length // dd.SECTOR)
    return out


def copy(entries: list[dict], sequence: int = None, ident: bytes = X, sectors: int = SECTORS,
         next_entry: int = None, next_free: int = None) -> bytearray:
    end = entries[-1]["first"] + -(-entries[-1]["length"] // dd.SECTOR) if entries else dd.DATA_START
    return dd.encode_copy(ident, sequence or len(entries) + 1, next_entry or len(entries) + 1,
                          end if next_free is None else next_free, sectors, entries)


def image(copy_a: bytes, copy_b: bytes, entries: list[dict] = ()) -> bytearray:
    return dd.build_image(SECTORS, copy_a, copy_b,
                          {r["first"]: archive(r["length"], i) for i, r in enumerate(entries)})


def patched(raw: bytes, offset: int, fmt: str, value) -> bytearray:
    out = bytearray(raw)
    struct.pack_into(fmt, out, offset, value)
    return dd.seal_copy(out)


def rec(index: int, field: int) -> int:
    return dd.RECORDS_AT + index * dd.RECORD + field


class CopyValidity(unittest.TestCase):
    """2.6: a copy is valid only if every rule holds."""

    def refuses(self, raw: bytes, rule: str, sectors: int = SECTORS):
        self.assertEqual(dd.check_copy(bytes(raw), sectors), (None, rule))

    def test_encoder_output_is_valid(self):
        for entries in ([], records(80), records(700, 1100, 513)):
            catalog, why = dd.check_copy(bytes(copy(entries)), SECTORS)
            self.assertIsNone(why)
            self.assertEqual(catalog["entries"], dd.decode_copy(bytes(copy(entries)), SECTORS)["entries"])
            self.assertEqual(len(catalog["entries"]), len(entries))

    def test_formatted_copy_fields(self):
        catalog = dd.decode_copy(bytes(copy([])), SECTORS)
        self.assertEqual((catalog["sequence"], catalog["next_entry_id"], catalog["next_free"],
                          catalog["formatted_sectors"], catalog["destination_id"]),
                         (1, 1, 16, SECTORS, X.hex()))

    def test_magic_version_checksum(self):
        base = copy(records(700))
        self.refuses(patched(base, 7, "<B", 2), "magic")
        self.refuses(patched(base, 8, "<H", 2), "version")
        damaged = bytearray(base)
        damaged[100] ^= 1
        self.refuses(damaged, "checksum")

    def test_count_and_header_padding(self):
        base = copy(records(700))
        self.refuses(patched(base, 10, "<B", 17), "count")
        self.refuses(patched(base, 12, "<B", 1), "header padding")

    def test_sequence_zero(self):
        self.refuses(patched(copy([]), 32, "<Q", 0), "sequence")

    def test_unused_records_and_tail_zero(self):
        base = copy(records(700))
        self.refuses(patched(base, rec(1, 0), "<Q", 9), "unused records")
        self.refuses(patched(base, 1500, "<B", 1), "unused records")
        # The count must cover every non-zero record: count 0 with a record present.
        self.refuses(patched(base, 10, "<B", 0), "unused records")

    def test_record_and_label_padding(self):
        base = copy(records(700))
        self.refuses(patched(base, rec(0, 44), "<B", 1), "record padding")
        self.refuses(patched(base, rec(0, 48 + 6), "<B", ord("x")), "label padding")
        self.refuses(patched(base, rec(0, 40), "<B", 33), "label padding")

    def test_entry_length_bounds(self):
        self.refuses(patched(copy(records(700)), rec(0, 16), "<Q", 79), "entry length")
        big = copy(records(dd.MAX_ARCHIVE), sectors=2048)
        self.assertIsNone(dd.check_copy(bytes(big), 2048)[1])
        self.refuses(patched(big, rec(0, 16), "<Q", dd.MAX_ARCHIVE + 1), "entry length", 2048)

    def test_strictly_ascending_ids_and_sectors(self):
        entries = records(700, 700)
        entries[1]["id"] = 1
        self.refuses(copy(entries, next_entry=3), "ascending")
        entries = records(700, 700)
        entries[0]["id"], entries[1]["id"] = 2, 1
        self.refuses(copy(entries, next_entry=3), "ascending")
        entries = records(700, 700)
        entries[1]["first"] = entries[0]["first"]
        self.refuses(copy(entries, next_free=entries[1]["first"] + 2), "ascending")

    def test_extents_contiguous(self):
        entries = records(700)
        entries[0]["first"] = 17
        self.refuses(copy(entries, next_free=19), "contiguous")
        entries = records(700, 700)
        entries[1]["first"] += 1
        self.refuses(copy(entries, next_free=entries[1]["first"] + 2), "contiguous")

    def test_next_free_equals_end_of_last_extent(self):
        self.refuses(copy(records(700), next_free=19), "next_free")
        self.refuses(copy([], next_free=17), "next_free")

    def test_n4_zero_entries_next_free_below_16(self):
        for value in (0, 8, 15):
            self.refuses(copy([], next_free=value), "next_free")

    def test_next_free_within_formatted_count(self):
        entries = records(dd.SECTOR * 20)
        self.refuses(copy(entries, sectors=30), "next_free bounds", 30)
        self.refuses(copy([], sectors=15), "next_free bounds", 15)

    def test_formatted_count_equals_scope(self):
        self.refuses(copy(records(700), sectors=SECTORS + 1), "formatted count")
        self.refuses(copy(records(700)), "formatted count", SECTORS - 1)

    def test_n4_zero_entries_next_entry_zero(self):
        self.refuses(patched(copy([]), 40, "<Q", 0), "next entry")

    def test_accepted_at_upper_limits(self):
        # R3-REV-2: 16 records, a 32-byte label and next_free == formatted count.
        entries = records(*[dd.SECTOR] * dd.MAX_ENTRIES)
        entries[-1]["label"] = (b"L" * dd.LABEL).hex()
        full = dd.DATA_START + dd.MAX_ENTRIES
        catalog, why = dd.check_copy(bytes(copy(entries, sectors=full)), full)
        self.assertIsNone(why)
        self.assertEqual((len(catalog["entries"]), catalog["next_free"], catalog["formatted_sectors"]),
                         (dd.MAX_ENTRIES, full, full))
        self.assertEqual(catalog["entries"][-1]["label"], (b"L" * dd.LABEL).hex())
        self.assertIsNone(dd.check_copy(bytes(copy(records(*[80] * dd.MAX_ENTRIES))), SECTORS)[1])
        self.assertIsNone(dd.check_copy(bytes(copy([], sectors=dd.DATA_START)), dd.DATA_START)[1])

    def test_encoder_refuses_to_change_copy_layout(self):
        # R3-REV-6 (b): a crafted fixture must not silently grow past 2048 bytes.
        entries = records(700)
        entries[0]["label"] = (b"x" * (dd.LABEL + 1)).hex()
        with self.assertRaises(ValueError):
            copy(entries)
        with self.assertRaises(ValueError):
            dd.encode_copy(X, 1, 26, dd.DATA_START, SECTORS, [{"id": 1, "first": 16, "length": 80}] * 25)

    def test_entry_id_below_next_entry(self):
        self.refuses(copy(records(700, 700), next_entry=2), "entry id")
        self.assertIsNone(dd.check_copy(bytes(copy(records(700), next_entry=9)), SECTORS)[1])


class SpecGapPins(unittest.TestCase):
    """Pinned current behaviour where the spec is undecided; flip when the owner decides.

    Label content at mount is owner item O14 (2.6, revision 3): the oracle
    checks only the label length and padding, not valid_name.
    """

    def test_label_content_unchecked_until_o14(self):
        for label in (b"..", b"\xff\xfe", b"a\x07b", "x\u202e".encode()):
            entries = records(700)
            entries[0]["label"] = label.hex()
            self.assertIsNone(dd.check_copy(bytes(copy(entries)), SECTORS)[1])


class RecordAndCopyValidity(unittest.TestCase):
    """N34 (2.6, revision 3): crafted copies, each valid except for one rule.

    Each is invalid; beside a valid copy the valid one is used, twice it is
    corrupt; a valid copy mounts beside non-zero sectors 8-15. The Rust core
    runs the same cases (services/destination/tests/catalog.rs,
    n34_record_and_copy_validity).
    """

    def crafted(self) -> dict[str, tuple[bytearray, str]]:
        base = copy(records(700, 1100))
        cases = {
            "zero records, copy sequence 0": (patched(copy([]), 32, "<Q", 0), "sequence"),
            "entry ID 0": (patched(base, rec(0, 0), "<Q", 0), "entry id 0"),
            "label length 0": (patched(base, rec(1, 40), "<B", 0), "label length"),
            "length 79": (patched(base, rec(1, 16), "<Q", 79), "entry length"),
            "length 525 329": (patched(base, rec(0, 16), "<Q", dd.MAX_ARCHIVE + 1), "entry length"),
            "sealing sequence 1": (patched(base, rec(0, 32), "<Q", 1), "sealing sequence"),
            "sealing sequence above the copy's": (patched(base, rec(1, 32), "<Q", 4),
                                                  "sealing sequence above copy"),
            "equal sealing sequences": (patched(base, rec(1, 32), "<Q", 2), "sealing sequence ascending"),
        }
        long_label = bytearray(base)
        long_label[rec(1, 40)] = 33
        long_label[rec(1, 48):rec(1, 80)] = b"x" * dd.LABEL
        cases["label length 33"] = (dd.seal_copy(long_label), "label padding")
        return cases

    def test_each_crafted_copy_invalid_with_its_rule(self):
        for name, (raw, rule) in self.crafted().items():
            with self.subTest(name):
                self.assertEqual(dd.check_copy(bytes(raw), SECTORS), (None, rule))

    def test_mount_state_beside_valid_copy_and_twice(self):
        valid = copy(records(700, 1100))
        for name, (raw, _) in self.crafted().items():
            with self.subTest(name):
                for a, b, slot in ((raw, valid, "B"), (valid, raw, "A")):
                    result = dd.mount(image(a, b), SECTORS)
                    self.assertEqual((result["state"], result["slot"]), ("mounted", slot))
                self.assertEqual(dd.mount(image(raw, raw), SECTORS)["state"], "corrupt")

    def test_valid_copy_beside_nonzero_reserved_sectors(self):
        a = copy(records(700))
        area = image(a, a)
        area[8 * dd.SECTOR:16 * dd.SECTOR] = b"\xa5" * (8 * dd.SECTOR)
        self.assertEqual(dd.mount(area, SECTORS)["state"], "mounted")

    def test_encoder_default_sealing_sequences_are_valid(self):
        # OP-4: without explicit sealing sequences the encoder seals record i at i + 2.
        plain = [{k: v for k, v in r.items() if k != "sealed_sequence"} for r in records(700, 1100, 300)]
        catalog, why = dd.check_copy(bytes(copy(plain)), SECTORS)
        self.assertIsNone(why)
        self.assertEqual([r["sealed_sequence"] for r in catalog["entries"]], [2, 3, 4])


class MountRule(unittest.TestCase):
    """2.6 mount rule and N8, N14, N25, N30 outcomes at the oracle layer."""

    def test_identical_copies(self):
        a = copy(records(700, 1100))
        result = dd.mount(image(a, a), SECTORS)
        self.assertEqual((result["state"], result["slot"]), ("mounted", "A"))
        self.assertEqual([r["id"] for r in result["catalog"]["entries"]], [1, 2])

    def test_one_valid_copy_beside_damage(self):
        good = copy(records(700))
        bad = bytearray(good)
        bad[200] ^= 0xFF
        for a, b, slot in ((bad, good, "B"), (good, bad, "A")):
            result = dd.mount(image(a, b), SECTORS)
            self.assertEqual((result["state"], result["slot"]), ("mounted", slot))
            self.assertEqual([c["reason"] for c in result["copies"] if not c["valid"]], ["checksum"])

    def test_n8_both_damaged_is_corrupt(self):
        good = copy(records(700))
        bad = bytearray(good)
        bad[200] ^= 0xFF
        self.assertEqual(dd.mount(image(bad, bad), SECTORS)["state"], "corrupt")

    def test_blank_and_reserved_sector_nonzero(self):
        zero = bytes(dd.COPY)
        self.assertEqual(dd.mount(image(zero, zero), SECTORS)["state"], "blank")
        area = image(zero, zero)
        area[12 * dd.SECTOR + 5] = 1
        self.assertEqual(dd.mount(area, SECTORS)["state"], "corrupt")

    def test_n8_zeroed_copies_with_entries_is_blank_candidate(self):
        entries = records(700)
        zero = bytes(dd.COPY)
        result = dd.mount(image(zero, zero, entries), SECTORS)
        self.assertEqual(result["state"], "blank")

    def test_n25_partial_format_copy_a(self):
        a = copy([])
        zero = bytes(dd.COPY)
        only_sector_0 = bytearray(dd.COPY)
        only_sector_0[:dd.SECTOR] = a[:dd.SECTOR]
        self.assertEqual(dd.mount(image(only_sector_0, zero), SECTORS)["state"], "corrupt")
        middle = bytearray(dd.COPY)
        middle[dd.SECTOR:3 * dd.SECTOR] = a[dd.SECTOR:3 * dd.SECTOR]
        self.assertEqual(dd.mount(image(middle, zero), SECTORS)["state"], "blank")
        result = dd.mount(image(a, zero), SECTORS)
        self.assertEqual((result["state"], result["slot"], result["catalog"]["sequence"]), ("mounted", "A", 1))

    def test_crash_between_barriers_higher_copy_used(self):
        old = copy(records(700), sequence=2)
        new = copy(records(700, 1100), sequence=3)
        for a, b, slot in ((new, old, "A"), (old, new, "B")):
            result = dd.mount(image(a, b), SECTORS)
            self.assertEqual((result["state"], result["slot"]), ("mounted", slot))
            self.assertEqual(result["catalog"]["sequence"], 3)

    def test_extension_allows_more_than_one_record(self):
        old = copy(records(700), sequence=2)
        new = copy(records(700, 1100, 300), sequence=5)
        self.assertEqual(dd.mount(image(old, new), SECTORS)["slot"], "B")

    def test_n30_non_extending_pairs_are_corrupt(self):
        # With valid copies, a record prefix plus contiguity already implies a
        # next_free that is not lower, so "next_free lower" has no isolated case.
        low = copy(records(700, 1100), sequence=3, next_entry=5)
        cases = {
            "record missing": copy(records(700), sequence=4, next_entry=5),
            "record changed": patched(copy(records(700, 1100, 300), sequence=4, next_entry=5),
                                      rec(0, 24), "<Q", 1),
            "other destination ID": copy(records(700, 1100, 300), sequence=4, ident=Y, next_entry=5),
            "next entry lower": copy(records(700, 1100), sequence=4, next_entry=4),
        }
        for name, high in cases.items():
            for a, b in ((high, low), (low, high)):
                with self.subTest(name):
                    self.assertIsNone(dd.check_copy(bytes(high), SECTORS)[1])
                    result = dd.mount(image(a, b), SECTORS)
                    self.assertEqual((result["state"], result["reason"]),
                                     ("corrupt", "higher copy does not extend the lower"))

    def test_equal_sequence_different_bytes_is_corrupt(self):
        a = copy(records(700), sequence=3)
        b = copy(records(700, 1100), sequence=3)
        result = dd.mount(image(a, b), SECTORS)
        self.assertEqual((result["state"], result["reason"]), ("corrupt", "equal sequences, different bytes"))

    def test_n8_read_error_is_failed_mount_never_older_copy(self):
        old = copy(records(700), sequence=2)
        new = copy(records(700, 1100), sequence=3)
        for sector in (0, 3):
            result = dd.mount(image(new, old), SECTORS, unreadable={sector})
            self.assertEqual((result["state"], result["slot"], result["catalog"]), ("failed", None, None))

    def test_n8_read_error_in_reserved_sectors_never_blank(self):
        zero = bytes(dd.COPY)
        self.assertEqual(dd.mount(image(zero, zero), SECTORS, unreadable={9})["state"], "failed")
        self.assertEqual(dd.mount(bytes(dd.AREA - 1), SECTORS)["state"], "failed")

    def test_n14_second_destination_id_is_reported(self):
        y = copy(records(700), ident=Y)
        result = dd.mount(image(y, y), SECTORS)
        self.assertEqual(result["state"], "mounted")
        self.assertEqual(result["catalog"]["destination_id"], Y.hex())
        self.assertNotEqual(result["catalog"]["destination_id"], X.hex())


class ImageFiles(unittest.TestCase):
    """analyse() and the CLI on files: extents, trailers, bounded reads."""

    def write(self, directory: str, data: bytes) -> Path:
        path = Path(directory) / "dest.img"
        path.write_bytes(bytes(data))
        return path

    def test_extents_and_trailers(self):
        entries = records(700, 1100)
        a = copy(entries)
        with tempfile.TemporaryDirectory() as directory:
            path = self.write(directory, image(a, a, entries))
            result = dd.analyse(path)
            got = result["catalog"]["entries"]
            self.assertEqual([(r["first"], r["end"], r["trailer_ok"], r["padding_zero"]) for r in got],
                             [(16, 18, True, True), (18, 21, True, True)])
            self.assertEqual(dd.extent_bytes(path, got[1]), archive(1100, 1))

    def test_damaged_trailer_and_truncated_extent(self):
        entries = records(700, 1100)
        a = copy(entries)
        data = image(a, a, entries)
        data[16 * dd.SECTOR + 699] ^= 1
        with tempfile.TemporaryDirectory() as directory:
            got = dd.analyse(self.write(directory, data))["catalog"]["entries"]
            self.assertEqual([r["trailer_ok"] for r in got], [False, True])
            got = dd.analyse(self.write(directory, data[:19 * dd.SECTOR]), SECTORS)["catalog"]["entries"]
            self.assertEqual([r["readable"] for r in got], [True, False])

    def test_scope_offset_and_size(self):
        entries = records(700)
        a = copy(entries)
        with tempfile.TemporaryDirectory() as directory:
            path = self.write(directory, bytes(5 * dd.SECTOR) + image(a, a, entries) + bytes(dd.SECTOR))
            result = dd.analyse(path, SECTORS, first=5)
            self.assertEqual(result["state"], "mounted")
            self.assertTrue(result["catalog"]["entries"][0]["trailer_ok"])
            self.assertEqual(dd.extent_bytes(path, result["catalog"]["entries"][0], 5), archive(700, 0))
            # Without the scope size the image tail counts: formatted count differs.
            self.assertEqual(dd.analyse(path, first=5)["copies"][0]["reason"], "formatted count")

    def test_padding_after_partial_last_sector(self):
        # R3-REV-1, spec 2.7 step 2: the partial last sector is zero-padded.
        entries = records(700, 1100)
        a = copy(entries)
        with tempfile.TemporaryDirectory() as directory:
            for offset in (700, 1023):
                with self.subTest(offset=offset):
                    data = image(a, a, entries)
                    data[16 * dd.SECTOR + offset] = 0x5A
                    got = dd.analyse(self.write(directory, data))["catalog"]["entries"]
                    self.assertEqual([(r["trailer_ok"], r["padding_zero"]) for r in got],
                                     [(True, False), (True, True)])

    def test_cli_refuses_negative_first(self):
        # R3-REV-6 (c): an operator argument error, not a traceback.
        a = copy(records(700))
        with tempfile.TemporaryDirectory() as directory:
            path = self.write(directory, image(a, a, records(700)))
            run = subprocess.run([sys.executable, "-B", str(TOOLS / "destination_disk.py"), str(path),
                                  "--first", "-1"], capture_output=True, text=True, check=False)
            self.assertEqual(run.returncode, 2, run.stderr)
            self.assertIn("--first must not be negative", run.stderr)
            self.assertNotIn("Traceback", run.stderr)

    def test_cli_json(self):
        a = copy(records(700))
        with tempfile.TemporaryDirectory() as directory:
            path = self.write(directory, image(a, a, records(700)))
            run = subprocess.run([sys.executable, "-B", str(TOOLS / "destination_disk.py"), str(path)],
                                 capture_output=True, text=True, check=False)
            self.assertEqual(run.returncode, 0, run.stderr)
            self.assertEqual(json.loads(run.stdout)["state"], "mounted")


def expectations(path: Path) -> dict:
    expect = {"records": []}
    for line in path.read_text(encoding="utf-8").splitlines():
        key, _, value = line.partition("=")
        if key == "record":
            ident, first, length, trailer, sealed, label = value.split(",")
            expect["records"].append({"id": int(ident), "first": int(first), "length": int(length),
                                      "trailer": int(trailer), "sealed_sequence": int(sealed),
                                      "label": label})
        else:
            expect[key] = value
    return expect


@unittest.skipUnless(os.environ.get("DEST_ORACLE_OUT"), "DEST_ORACLE_OUT not set (Rust cross-check images)")
class RustCrossCheck(unittest.TestCase):
    """The oracle and the Rust host core agree on images the Rust core wrote."""

    def test_rust_images(self):
        directory = Path(os.environ["DEST_ORACLE_OUT"])
        expects = sorted(directory.glob("*.expect"))
        self.assertGreaterEqual(len(expects), 6, f"no Rust images in {directory}")
        for expect_path in expects:
            with self.subTest(expect_path.stem):
                expect = expectations(expect_path)
                result = dd.analyse(expect_path.with_suffix(".img"))
                self.assertEqual(result["state"], expect["state"])
                if expect["state"] != "mounted":
                    continue
                catalog = result["catalog"]
                self.assertEqual((catalog["destination_id"], catalog["sequence"], catalog["next_entry_id"],
                                  catalog["next_free"], catalog["formatted_sectors"]),
                                 (expect["id"], int(expect["sequence"]), int(expect["next_entry"]),
                                  int(expect["next_free"]), int(expect["formatted"])))
                self.assertEqual([{k: r[k] for k in ("id", "first", "length", "trailer", "sealed_sequence",
                                                     "label")} for r in catalog["entries"]], expect["records"])
                self.assertTrue(all(r["trailer_ok"] and r["padding_zero"] for r in catalog["entries"]))


if __name__ == "__main__":
    unittest.main()
