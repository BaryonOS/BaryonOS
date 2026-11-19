"""Shared S09 archive v2 fixtures (services/volume/tests/fixtures/archive-v2/).

The archives are built with the spec-derived encoder of tests/test_volume_archive_v2.py, and
manifest.txt is written from the independent reader's (tools/volume_archive.py parse_v2) output,
so the Rust reader in volume-core is checked against the same bytes and values without an
encoder of its own. This test regenerates every file and requires byte equality.

Regenerate after a deliberate change: python3 -B tests/test_volume_archive_v2_fixtures.py --write
"""
from __future__ import annotations
import struct
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
sys.path.insert(0, str(ROOT / "tests"))
import volume_archive as va  # noqa: E402
from test_volume_archive_v2 import (  # noqa: E402
    ACCOUNTED, TOMBSTONES, encode, first_object, obj, present_only, put, sample, setbyte, v1_sample)

FIXTURES = ROOT / "services/volume/tests/fixtures/archive-v2"


def restore_parts():
    """High source revisions and stamps, multi-chunk content, collections at indices 2 and 4,
    an uncollected object without metadata, a content-free record and a deleted collection."""
    big = bytes((i * 7 + i // 251) % 256 for i in range(2 * va.CHUNK + 17))
    return dict(
        collections=[dict(index=2, present=True, revision=901, name="Archief"),
                     dict(index=3, present=False, revision=902, name=""),
                     dict(index=4, present=True, revision=903, name="Foto's")],
        objects=[obj(1, 950, [(950, "Plan v3", big), (940, "Plan v2", big[:va.CHUNK + 1]), (930, "Plan", b"p")],
                     collection=4, kind=7, tags=0x8001, created=910, modified=960),
                 obj(3, 945, [(944, "Kaal", b"zonder metadata")], created=920, modified=945),
                 obj(4, 948, [(947, "Lijst", bytes(range(256)) * 3), (946, "Lijst", b"")],
                     collection=2, created=921, modified=948),
                 obj(7, 949, [], present=False, created=930, modified=949)])


def over_quota_parts():
    """Valid under S10 (each object's largest version has 2 chunks), but replay needs 41 chunks."""
    objects, budget = [], 17
    for slot in range(8):
        versions = []
        for j in range(3):
            two = budget > 0
            budget -= two
            versions.append((30 - j, f"q{slot}{j}", bytes([slot + 1]) * (va.CHUNK + 1 if two else 1)))
        objects.append(obj(slot, 30, versions))
    return dict(collections=[], objects=objects)


def full_parts():
    """25 chunks under replay: one more than the volume keeps free once the other domain holds
    its full quota (64 - 40); the two-chunk version is slot 7's current one, written last."""
    return dict(collections=[], objects=[
        obj(slot, 30, [(30 - j, f"f{slot}{j}", bytes([slot + 1]) * (va.CHUNK + 1 if slot == 7 and j == 0 else 1))
                       for j in range(3)]) for slot in range(8)])


def s10_parts():
    return dict(collections=[], objects=[obj(0, 1, [(1, "a", b"\x01" * va.MAX_LENGTH)]),
                                         obj(1, 1, [(1, "b", b"\x02" * va.MAX_LENGTH)]),
                                         obj(2, 3, [(3, "c", b"x"), (2, "c", b"\x03" * (8 * va.CHUNK + 1))])])


def without(parts, *indices):
    parts["objects"] = [o for i, o in enumerate(parts["objects"]) if i not in indices]
    return parts


def s8_parts():
    parts = sample()
    parts["collections"][0] = dict(parts["collections"][0], revision=21)
    return parts


def fixtures() -> dict[str, tuple[bytes, str | None]]:
    """name -> (bytes, expected refusal message or None for a valid archive)."""
    full = TOMBSTONES | ACCOUNTED
    data = {
        "present-only.bin": (encode(**present_only(), flags=0), None),
        "accounted.bin": (encode(**without(sample(), 2), flags=ACCOUNTED), None),
        "tombstones.bin": (encode(**sample(), flags=full), None),
        "restore.bin": (encode(**restore_parts(), flags=ACCOUNTED, clock=1000), None),
        "over-quota.bin": (encode(**over_quota_parts(), flags=0, clock=30), None),
        "full.bin": (encode(**full_parts(), flags=0, clock=30), None),
        "s1-version-revision-zero.bin": (encode(**first_object(
            versions=[(8, "a", b"1"), (6, "b", b"2"), (0, "c", b"3")])), "version revision is zero"),
        "s2-newest-version-above-object.bin": (encode(**first_object(versions=[(10, "a", b"1")])),
                                               "newest version revision exceeds"),
        "s3-not-decreasing.bin": (encode(**first_object(
            versions=[(8, "a", b"1"), (5, "b", b"2"), (6, "c", b"3")])), "not strictly decreasing"),
        "s4-created-zero.bin": (encode(**first_object(created=0)), "created stamp is zero"),
        "s5-created-after-modified.bin": (encode(**first_object(created=13)), "created stamp after modified"),
        "s6-modified-above-clock.bin": (encode(**first_object(modified=21)), "modified stamp exceeds"),
        "s7-object-revision-above-clock.bin": (encode(**first_object(revision=21)),
                                               "object revision exceeds"),
        "s8-collection-revision-above-clock.bin": (encode(**s8_parts()), "collection revision exceeds"),
        "s9-generation.bin": (encode(**sample(), generation=31), "differs from the domain generation"),
        "s10-chunk-budget.bin": (encode(**s10_parts(), flags=0), "chunk usage exceeds the domain quota"),
        "header-unknown-flag.bin": (encode(**sample(), tweak={"header": setbyte(10, 0x07)}),
                                    "invalid archive header"),
        "header-byte-11.bin": (encode(**sample(), tweak={"header": setbyte(11, 1)}), "invalid archive header"),
        "header-byte-40.bin": (encode(**sample(), tweak={"header": setbyte(40, 1)}), "invalid archive header"),
        "header-byte-55.bin": (encode(**sample(), tweak={"header": setbyte(55, 1)}), "invalid archive header"),
        "header-versions-above-records.bin": (encode(**sample(), tweak={"header": put(14, "<H", 7)}),
                                              "invalid archive trailer"),
        "header-content-above-records.bin": (encode(**sample(), tweak={"header": put(24, "<Q", 70)}),
                                             "invalid archive trailer"),
        "deleted-collection-unaccounted.bin": (encode(**without(sample(), 3), flags=TOMBSTONES),
                                               "deleted collection without the accounted flag"),
        "tombstone-unflagged.bin": (encode(**sample(), flags=ACCOUNTED), "tombstone without the tombstone flag"),
        "content-free-unaccounted.bin": (encode(**dict(sample(), collections=sample()["collections"][:1]),
                                                flags=TOMBSTONES),
                                         "content-free record without the accounted flag"),
        "object-in-deleted-collection.bin": (encode(**sample(), tweak={("o", 1): setbyte(2, 2)}),
                                             "invalid object record"),
        "absent-object-with-kind.bin": (encode(**sample(), tweak={("o", 2): put(4, "<H", 1)}),
                                        "invalid object record"),
        "object-reserved-byte.bin": (encode(**sample(), tweak={("o", 0): setbyte(119, 1)}),
                                     "invalid object record"),
        "version-reserved-byte.bin": (encode(**sample(), tweak={("v", 0, 0): setbyte(2, 1)}),
                                      "invalid version record"),
        "trailing-byte.bin": (encode(**sample()) + b"\x00", "bytes after the archive trailer"),
        "v1.bin": (v1_sample(), "invalid archive header"),
    }
    for name, (tweak, message) in STRUCTURAL.items():
        data[f"struct-{name}.bin"] = (encode(**sample(), tweak=tweak), message)
    data["struct-history-count-3.bin"] = (encode(**first_object(
        versions=sample()["objects"][1]["versions"] + [(1, "x", b"")])), "invalid object record")
    return data


# Resealed structural cases (as in test_volume_archive_v2.Rules), several of which guard indexing.
STRUCTURAL = {
    "collection-index-0": ({("c", 0): setbyte(0, 0)}, "invalid collection record"),
    "collection-index-not-ascending": ({("c", 1): setbyte(0, 1)}, "invalid collection record"),
    "collection-index-5": ({("c", 1): setbyte(0, 5)}, "invalid collection record"),
    "collection-state-2": ({("c", 0): setbyte(1, 2)}, "invalid collection record"),
    "collection-name-length-33": ({("c", 0): setbyte(2, 33)}, "invalid collection record"),
    "collection-invalid-name": ({("c", 0): setbyte(16, 0xff)}, "invalid collection record"),
    "collection-revision-0": ({("c", 0): put(8, "<Q", 0)}, "invalid collection record"),
    "collection-deleted-with-name": ({("c", 1): setbyte(2, 1)}, "invalid collection record"),
    "collection-reserved-byte": ({("c", 0): setbyte(3, 1)}, "invalid collection record"),
    "collection-name-padding": ({("c", 0): setbyte(16 + 4, 1)}, "invalid collection record"),
    "collection-tail-byte": ({("c", 1): setbyte(55, 1)}, "invalid collection record"),
    "object-slot-not-ascending": ({("o", 1): setbyte(0, 0)}, "invalid object record"),
    "object-slot-8": ({("o", 3): setbyte(0, 8)}, "invalid object record"),
    "object-state-2": ({("o", 0): setbyte(1, 2)}, "invalid object record"),
    "object-revision-0": ({("o", 0): put(16, "<Q", 0)}, "invalid object record"),
    "object-collection-not-archived": ({("o", 1): setbyte(2, 3)}, "invalid object record"),
    "object-collection-5": ({("o", 1): setbyte(2, 5)}, "invalid object record"),
    "absent-object-with-collection": ({("o", 2): setbyte(2, 1)}, "invalid object record"),
    "absent-object-with-tags": ({("o", 2): put(8, "<I", 1)}, "invalid object record"),
    "object-byte-6": ({("o", 0): setbyte(6, 1)}, "invalid object record"),
    "object-byte-12": ({("o", 0): setbyte(12, 1)}, "invalid object record"),
    "object-byte-40": ({("o", 0): setbyte(40, 1)}, "invalid object record"),
    "version-name-length-0": ({("v", 0, 0): setbyte(0, 0)}, "invalid version record"),
    "version-name-length-33": ({("v", 0, 0): setbyte(0, 33)}, "invalid version record"),
    "version-invalid-name": ({("v", 0, 0): setbyte(24, 0xff)}, "invalid version record"),
    "version-name-padding": ({("v", 0, 0): setbyte(24 + 4, 1)}, "invalid version record"),
    "version-length-65537": ({("v", 0, 0): put(4, "<I", 65537)}, "invalid version record"),
    "version-content-checksum": ({("v", 1, 1): put(16, "<Q", 1)}, "content checksum"),
    "header-object-count-9": ({"header": setbyte(12, 9)}, "invalid archive header"),
    "header-collection-count-5": ({"header": setbyte(13, 5)}, "invalid archive header"),
    "header-version-count-25": ({"header": put(14, "<H", 25)}, "invalid archive header"),
    "header-content-above-versions": ({"header": put(14, "<H", 0)}, "invalid archive header"),
    "header-versions-below-records": ({"header": put(14, "<H", 5)}, "exceed the header counts"),
    "header-content-below-records": ({"header": put(24, "<Q", 10)}, "exceed the header counts"),
}


def manifest(data: dict[str, tuple[bytes, str | None]]) -> str:
    """One line per recorded value, taken from the independent reader, never from the encoder inputs."""
    lines = []
    for name, (archive, message) in sorted(data.items()):
        if message is not None:
            lines.append(f"{name} refused {message}")
            continue
        parsed = va.parse_v2(archive)
        lines.append(f"{name} archive {int(parsed['tombstones'])} {int(parsed['accounted'])} "
                     f"{parsed['generation']} {parsed['clock']} {parsed['content_bytes']} {parsed['length']}")
        for index, c in parsed["collections"].items():
            lines.append(f"{name} collection {index} {int(c['present'])} {c['revision']} "
                         f"{c['name'].encode().hex() or '-'}")
        for o in parsed["objects"]:
            lines.append(f"{name} object {o['slot']} {int(o['present'])} {o['collection']} {o['kind']} {o['tags']} "
                         f"{o['revision']} {o['created']} {o['modified']} {len(o['versions'])}")
            for i, v in enumerate(o["versions"]):
                lines.append(f"{name} version {o['slot']} {i} {v['revision']} {v['length']} {v['content_sum']} "
                             f"{v['name'].encode().hex()}")
    return "\n".join(lines) + "\n"


def write() -> None:
    FIXTURES.mkdir(parents=True, exist_ok=True)
    data = fixtures()
    for name, (archive, _) in data.items():
        (FIXTURES / name).write_bytes(archive)
    (FIXTURES / "manifest.txt").write_text(manifest(data), encoding="utf-8")


class Fixtures(unittest.TestCase):
    def test_committed_files_equal_regenerated(self) -> None:
        data = fixtures()
        committed = {p.name for p in FIXTURES.iterdir()}
        self.assertEqual(committed, set(data) | {"manifest.txt", ".gitattributes"})
        for name, (archive, _) in data.items():
            with self.subTest(file=name):
                self.assertEqual((FIXTURES / name).read_bytes(), archive)
        self.assertEqual((FIXTURES / "manifest.txt").read_text(encoding="utf-8"), manifest(data))

    def test_oracle_verdicts(self) -> None:
        for name, (archive, message) in fixtures().items():
            with self.subTest(file=name):
                if message is None:
                    self.assertEqual(va.parse_v2(archive)["length"], len(archive))
                else:
                    with self.assertRaisesRegex(ValueError, message):
                        va.parse_v2(archive)

    def test_replay_needs_more_than_the_quota(self) -> None:
        parsed = va.parse_v2(fixtures()["over-quota.bin"][0])
        need = sum(-(-v["length"] // va.CHUNK) for o in parsed["objects"] for v in o["versions"])
        self.assertEqual(need, va.QUOTA + 1)

    def test_v1_fixture_is_a_valid_v1_archive(self) -> None:
        self.assertEqual(len(va.parse(fixtures()["v1.bin"][0])["objects"]), 2)


if __name__ == "__main__":
    if sys.argv[1:] == ["--write"]:
        write()
    else:
        unittest.main()
