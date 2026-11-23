"""Host tests for the M6e/S06 runner's cut evidence rule (tools/run_volume.py).

Invariant: a native cut case only counts as "killed inside the table write"
when the other table copy holds a real torn attempt at the interrupted
generation; a content cut must leave that copy untouched.
"""
from __future__ import annotations
import ast
import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run_volume as rv  # noqa: E402
import volume_disk as vd  # noqa: E402

N = vd.TABLE_SECTORS


def oracle(generation: int, other: list[int], other_marker: bool = False) -> dict:
    slots = [None, None]
    slots[generation % 2] = [generation] * N
    slots[(generation + 1) % 2] = other
    markers = [False, False]
    markers[(generation + 1) % 2] = other_marker
    return {"generation": generation, "slots": slots, "slot_markers": markers}


class TornTableWriteTests(unittest.TestCase):
    def test_format_marker_remains_valid_first_commit_cut_evidence(self):
        image = bytearray(vd.empty_image_bytes())
        marker_start = vd.FIRST * vd.SECTOR
        image[marker_start:marker_start + vd.SECTOR] = vd.format_marker()
        entries = [vd.empty_entry() for _ in range(vd.OBJECTS)]
        entries[0] = dict(vd.empty_entry(), revision=1, request_id=5, created=1, modified=1, present=True,
                          current=dict(vd.empty_version(), revision=1, name=b"Doc", content_sum=vd.FNV_OFFSET))
        start = (vd.FIRST + vd.TABLE_SECTORS) * vd.SECTOR
        image[start:start + vd.TABLE_SECTORS * vd.SECTOR] = b"".join(vd.encode_table(1, 5, entries))
        # A durable marker authorizes recovery of the logical empty generation.
        self.assertEqual(rv.torn_sectors(vd.analyse(image), False), 0)
        entries[0] = dict(entries[0], revision=2, request_id=6, modified=2,
                          current=dict(entries[0]["current"], revision=2))
        start = vd.FIRST * vd.SECTOR
        image[start:start + 4 * vd.SECTOR] = b"".join(vd.encode_table(2, 6, entries)[:4])
        self.assertEqual(rv.torn_sectors(vd.analyse(image), True), 4)

    def test_torn_prefix_after_the_previous_copy_is_accepted(self):
        self.assertEqual(rv.torn_sectors(oracle(3, [4] * 5 + [2] * (N - 5)), True), 5)
        # The first commit's companion has format generation 0; a legacy
        # never-written zero companion has the same numeric sector summary.
        self.assertEqual(rv.torn_sectors(oracle(1, [2] * 4 + [0] * (N - 4)), True), 4)

    def test_untouched_or_unexplained_copies_are_refused_for_a_table_cut(self):
        for other in ([2] * N,                          # cut before the table write
                      [4] * N,                          # not torn at all
                      [2] + [4] * 4 + [2] * (N - 5),    # not a prefix
                      [4] * 5 + [-1] + [2] * (N - 6),   # garbage after the prefix
                      [4] * 5 + [0] * (N - 5)):         # previous copy missing
            with self.assertRaises(ValueError, msg=str(other[:8])):
                rv.torn_sectors(oracle(3, other), True)

    def test_content_cut_leaves_the_other_copy_untouched(self):
        self.assertEqual(rv.torn_sectors(oracle(1, [0] * N), False), 0)
        self.assertEqual(rv.torn_sectors(oracle(1, [-1] + [0] * (N - 1), other_marker=True), False), 0)
        with self.assertRaises(ValueError):
            rv.torn_sectors(oracle(1, [2] + [0] * (N - 1)), False)
        with self.assertRaises(ValueError):
            rv.torn_sectors(oracle(1, [-1] + [0] * (N - 1), other_marker=False), False)


APP = Path(__file__).resolve().parents[1] / "sdk/examples/volume-docs/src/app.rs"
SLOT = {1: 1, 2: 0}  # table copy of generation 1 (INSTALL_COLLECTION) and 2 (INSTALL)


def installed_entry(request_id: int, clock: int, **fields):
    """The fixture INSTALL as the service commits it (fresh chunks, lowest first)."""
    n, chunk, payload, versions = len(rv.INSTALL_VERSIONS), 0, {}, []
    for index, (name, length) in enumerate(rv.INSTALL_VERSIONS):
        data = vd.fixture(rv.INSTALL_OBJECT, n - index, length)
        chunks = list(range(chunk, chunk + -(-length // vd.CHUNK)))
        chunk += len(chunks)
        parts = [data[i * vd.CHUNK:(i + 1) * vd.CHUNK] for i in range(len(chunks))]
        payload.update(zip(chunks, parts))
        versions.append(dict(revision=n - index, length=length, name=name.encode(), content_sum=vd.fnv(data),
                             chunks=chunks, sums=[vd.fnv(part) for part in parts]))
    tags, kind, collection = rv.INSTALL_META
    entry = dict(vd.empty_entry(), revision=n, request_id=request_id, created=clock, modified=clock,
                 tags=tags, kind=kind, present=True, collection=collection, current=versions[0],
                 history=versions[1:], installed=n, origin=rv.INSTALL_ORIGIN,
                 origins=[rv.INSTALL_ORIGIN_0 - i for i in range(n)])
    entry.update(fields)
    return entry, payload


def install_tables(entry=None, generation: int = 2):
    index, name, origin = rv.INSTALL_COLLECTION
    collections = [{"revision": 0, "request_id": 0, "present": False, "name": b""}
                   for _ in range(vd.DOMAINS * vd.COLLECTIONS)]
    collections[index - 1] = {"revision": 1, "request_id": 11, "present": True, "name": name.encode(),
                              "origin": origin}
    first = vd.encode_table(1, 11, [vd.empty_entry() for _ in range(vd.OBJECTS)], collections, [1, 0], fmt=3)
    entries = [vd.empty_entry() for _ in range(vd.OBJECTS)]
    if entry is not None:
        entries[rv.INSTALL_OBJECT] = entry
    second = vd.encode_table(generation, 13, entries, collections, [generation, 0], fmt=3)
    return first, second


def install_image(entry, payload, second_sectors: int = vd.TABLE_SECTORS, generation: int = 2) -> bytes:
    image = bytearray(vd.empty_image_bytes())
    first, second = install_tables(entry, generation)
    marker = vd.FIRST * vd.SECTOR
    image[marker:marker + vd.SECTOR] = vd.format_marker()
    for slot, sectors in ((SLOT[1], first), (SLOT[2], second[:second_sectors])):
        start = (vd.FIRST + slot * vd.TABLE_SECTORS) * vd.SECTOR
        image[start:start + len(sectors) * vd.SECTOR] = b"".join(sectors)
    for chunk, data in payload.items():
        start = (vd.FIRST + vd.DATA_START + chunk * vd.CHUNK_SECTORS) * vd.SECTOR
        image[start:start + len(data)] = data
    return bytes(image)


def evidence(oracle, commits: int) -> dict:
    item = oracle["objects"][rv.INSTALL_OBJECT]
    return {"VOLUME:A:RECOVERED_REVISION:": item["revision"], "VOLUME:A:RECOVERED_COLLECTION:": item["collection"],
            "VOLUME:A:RECOVERED_SECOND:": item["installed"], "VOLUME:SERVICE:GENERATION:": oracle["generation"],
            "VOLUME:SERVICE:COMMITS:": commits, "VOLUME:SERVICE:TABLE_WRITES:": commits,
            "VOLUME:BROKER:TABLE_SECTOR_WRITES:": 1 + commits * vd.TABLE_SECTORS if commits else 0,
            "VOLUME:A:DIGEST:": oracle["digests"][0],
            "VOLUME:B:DIGEST:": oracle["digests"][1]}


class InstallOracleTests(unittest.TestCase):
    """S09 L2: the install oracle accepts exactly never-used or fully installed."""

    def installed(self):
        return rv.analyse_v3(install_image(*installed_entry(13, 2)))

    def test_fixture_matches_the_app(self):
        source = APP.read_text(encoding="utf-8")
        for constant, value in (("INSTALL_ORIGIN", rv.INSTALL_ORIGIN),
                                ("INSTALL_ORIGIN_0", rv.INSTALL_ORIGIN_0),
                                ("INSTALL_META", rv.INSTALL_META),
                                ("INSTALL_COLLECTION", rv.INSTALL_COLLECTION),
                                ("INSTALL_VERSIONS", list(rv.INSTALL_VERSIONS))):
            match = re.search(rf"const {constant}: [^=]+= ([^;]+);", source)
            self.assertIsNotNone(match, constant)
            literal = re.sub(r"0x[0-9a-f]+", lambda m: str(int(m.group(0), 16)), match.group(1))
            found = ast.literal_eval(literal.replace("[", "(").replace("]", ")"))
            self.assertEqual(found, tuple(value) if isinstance(value, (tuple, list)) else value, constant)
        self.assertEqual(rv.INSTALL_OBJECT, 0)  # install_flow installs app.object(0)

    def test_installed_state_is_accepted(self):
        oracle = self.installed()
        rv.check_install("install", oracle, evidence(oracle, rv.INSTALL_COMMITS))
        self.assertEqual(oracle["install_outcome"], "installed")
        oracle = self.installed()
        rv.check_install("install-recover", oracle, evidence(oracle, 0))
        oracle = self.installed()
        rv.check_install("recover-cut", oracle, evidence(oracle, 0))
        self.assertEqual(oracle["install_outcome"], "installed")

    def test_torn_install_commit_is_never_used(self):
        entry, payload = installed_entry(13, 2)
        oracle = rv.analyse_v3(install_image(entry, payload, second_sectors=4))
        rv.check_install("install-cut", oracle, {})
        self.assertEqual((oracle["install_outcome"], oracle["torn_sectors"]), ("never-used", 4))
        oracle = rv.analyse_v3(install_image(entry, payload, second_sectors=4))
        rv.check_install("recover-cut", oracle, evidence(oracle, 0))
        self.assertEqual(oracle["install_outcome"], "never-used")
        # The install and install-cut stages accept only their own outcome.
        with self.assertRaises(ValueError):
            rv.check_install("install", oracle, evidence(oracle, rv.INSTALL_COMMITS))
        with self.assertRaises(ValueError):
            rv.check_install("install-cut", self.installed(), {})

    def test_mutated_installs_are_refused(self):
        n = len(rv.INSTALL_VERSIONS)
        entry, payload = installed_entry(13, 2)
        wrong_origin = dict(entry, origins=[rv.INSTALL_ORIGIN_0, rv.INSTALL_ORIGIN_0 - 1, rv.INSTALL_ORIGIN_0 - 3])
        object_origin = dict(entry, origin=(rv.INSTALL_ORIGIN[0], rv.INSTALL_ORIGIN[1], rv.INSTALL_ORIGIN[2] + 1))
        stamps = dict(entry, created=1)
        # Partial: only the current version and one history version installed.
        partial = dict(entry, history=entry["history"][:1], installed=n - 1, origins=entry["origins"][:2] + [0])
        for mutated in (wrong_origin, object_origin, stamps, partial):
            with self.assertRaises(ValueError):
                oracle = rv.analyse_v3(install_image(mutated, payload))
                rv.check_install("recover-cut", oracle, evidence(oracle, 0))
        # A wrong content byte (checksums matching the wrong bytes too) and
        # swapped version contents are refused.
        current = entry["current"]
        data = bytearray(vd.fixture(rv.INSTALL_OBJECT, n, current["length"]))
        data[100] ^= 0x01
        chunk = current["chunks"][0]
        flipped = dict(current, content_sum=vd.fnv(bytes(data)),
                       sums=[vd.fnv(bytes(data[:vd.CHUNK]))] + current["sums"][1:])
        with self.assertRaises(ValueError):
            rv.analyse_v3(install_image(dict(entry, current=flipped),
                                        payload | {chunk: bytes(data[:vd.CHUNK])}))
        second = entry["history"][0]
        older = vd.fixture(rv.INSTALL_OBJECT, n - 2, second["length"])
        swapped = dict(second, content_sum=vd.fnv(older), sums=[vd.fnv(older)])
        with self.assertRaises(ValueError):
            oracle = rv.analyse_v3(install_image(dict(entry, history=[swapped] + entry["history"][1:]),
                                                 payload | {second["chunks"][0]: older}))
            rv.check_install("install", oracle, evidence(oracle, rv.INSTALL_COMMITS))

    def test_guest_evidence_must_match_the_oracle(self):
        oracle = self.installed()
        # A refusal that rewrote a table copy without a commit is caught too.
        for key, delta in (("VOLUME:A:RECOVERED_SECOND:", 1), ("VOLUME:SERVICE:COMMITS:", 1),
                           ("VOLUME:SERVICE:TABLE_WRITES:", 1), ("VOLUME:BROKER:TABLE_SECTOR_WRITES:", 1),
                           ("VOLUME:BROKER:TABLE_SECTOR_WRITES:", vd.TABLE_SECTORS), ("VOLUME:A:DIGEST:", 1)):
            wrong = evidence(oracle, rv.INSTALL_COMMITS)
            wrong[key] += delta
            with self.assertRaises(ValueError, msg=key):
                rv.check_install("install", oracle, wrong)

    def test_outside_scope_change_is_refused(self):
        image = bytearray(install_image(*installed_entry(13, 2)))
        image[-1] ^= 0x01
        with self.assertRaises(ValueError):
            rv.analyse_v3(bytes(image))


if __name__ == "__main__":
    unittest.main()
