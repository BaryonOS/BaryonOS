"""Host tests for the S09 phase 3 L2 oracle in tools/run_volume.py.

Requirement: docs/implementation/S09-L2-NATIVE-EXPORT.md sections 2-4 and 6
(discrimination invariant, expected source and destination state, explicit
preparation and checks, source loss, v2 extraction, flip offset). Synthetic
images and archives are fed through each check; the listed mutations must
fail. These are host tests only: they prove the oracle, not the native stages.

PARENT is the domain-0 state of the `archive-export` image as measured on a
passing native run (slice-B worktree, build/run/volume-archive-export-2fc3d55b6a,
volume.after.img through outside_archive and volume_disk.analyse). The
stamps and version revisions are not in run_volume.FINAL, so they are
recorded here as measured values, not derived ones. Domain 1 is reduced to
one object: the oracle only requires it to stay unchanged.
"""
from __future__ import annotations
import ast
import json
import re
import sys
import tempfile
import unittest
import unittest.mock
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
sys.path.insert(0, str(ROOT / "tests"))
import run_volume as rv  # noqa: E402
import volume_archive as va  # noqa: E402
import volume_disk as vd  # noqa: E402
from test_volume_archive_v2 import ACCOUNTED, encode  # noqa: E402

APP = ROOT / "sdk/examples/volume-docs/src/app.rs"

# object -> (revision, created, modified, collection, tags, kind, present,
#            [(version revision, name, length, content revision), ...]) (measured, see above)
PARENT = {0: (4, 1, 57, 0, 0, 0, False, []),
          1: (3, 2, 50, 2, 0, 2, True, [(1, "Notitie ✓", 129, 1)]),
          2: (2, 3, 51, 1, 0, 0, True, [(1, "Leeg", 0, 1)]),
          3: (42, 5, 54, 0, 0, 0, True, [(41, "Concept", 5 * 4096, 38)]),
          4: (4, 46, 53, 0, 0, 0, True, [(4, "Herstart", 5000, 1)]),
          5: (1, 55, 55, 0, 0, 0, True, [(1, "Archief", 65536, 1)]),
          9: (1, 2, 2, 0, 0, 0, True, [(1, "Lijst", 300, 1)])}
PARENT_COLLECTIONS = {(0, 1): (1, "Werk"), (0, 2): (1, "Privé")}
PARENT_CLOCKS = [57, 27]
# Revision 1's S1-S4 without S5: objects 1 and 5 keep version revision 1,
# which a destination also assigns, so it must trip the discrimination invariant.
S1_S4 = rv.L2_SOURCE_STEP[:7]


def oracle_object(obj, revision=0, created=0, modified=0, collection=0, tags=0, kind=0, present=False,
                  versions=(), request_id=None):
    return {"object": obj, "revision": revision, "present": present, "collection": collection, "tags": tags,
            "kind": kind, "created": created, "modified": modified,
            "request_id": request_id if request_id is not None else (1000 + obj if revision else 0),
            "versions": [{"revision": r, "length": length, "name": name, "content_revision": c}
                         for r, name, length, c in versions]}


def parent_state():
    objects = [oracle_object(o, *PARENT[o]) if o in PARENT else oracle_object(o) for o in range(vd.OBJECTS)]
    collections = [{"index": i % vd.COLLECTIONS + 1, "domain": i // vd.COLLECTIONS, "present": False,
                    "revision": 0, "name": ""} for i in range(vd.DOMAINS * vd.COLLECTIONS)]
    for c in collections:
        if (c["domain"], c["index"]) in PARENT_COLLECTIONS:
            revision, name = PARENT_COLLECTIONS[(c["domain"], c["index"])]
            c.update(present=True, revision=revision, name=name, request_id=2000 + c["index"])
    return {"generation": sum(PARENT_CLOCKS), "clocks": list(PARENT_CLOCKS), "objects": objects,
            "collections": collections}


def volume_image(state, fmt=2, destination=None) -> bytes:
    """A two-copy image of an oracle-shaped state (generation g in its slot,
    a g - 1 seal in the other). Versions keep given chunks or get fresh ones;
    content is `content` or the fixture. `destination` (expected_l2 state)
    adds the v3 provenance."""
    image = bytearray(vd.empty_image_bytes())
    used = {c for item in state["objects"] for v in item["versions"] for c in v.get("chunks", [])}
    free = (c for c in range(vd.CHUNKS) if c not in used)
    entries, fresh_id = [], 5000
    for item in state["objects"]:
        versions = []
        for v in item["versions"]:
            data = v["content"] if "content" in v else vd.fixture(item["object"], v["content_revision"], v["length"])
            count = -(-len(data) // vd.CHUNK)
            chunks = v.get("chunks") or [next(free) for _ in range(count)]
            parts = [data[i * vd.CHUNK:(i + 1) * vd.CHUNK] for i in range(count)]
            for chunk, part in zip(chunks, parts):
                start = (vd.FIRST + vd.DATA_START + chunk * vd.CHUNK_SECTORS) * vd.SECTOR
                image[start:start + len(part)] = part
            versions.append(dict(vd.empty_version(), revision=v["revision"], length=len(data),
                                 name=v["name"].encode(), content_sum=vd.fnv(data), chunks=chunks,
                                 sums=[vd.fnv(p) for p in parts]))
        request_id = item["request_id"]
        if request_id is None:
            fresh_id += 1
            request_id = fresh_id
        entry = dict(vd.empty_entry(), revision=item["revision"], request_id=request_id,
                     created=item["created"], modified=item["modified"], tags=item["tags"], kind=item["kind"],
                     present=item["present"], collection=item["collection"],
                     current=versions[0] if item["present"] else vd.empty_version(),
                     history=versions[1:] if item["present"] else versions)
        for key in ("installed", "origin", "origins"):
            if key in item:
                entry[key] = item[key]
        entries.append(entry)
    collections = []
    for c in state["collections"]:
        request_id = c.get("request_id", 3000 + c["index"] if c["revision"] else 0)
        if request_id is None:
            fresh_id += 1
            request_id = fresh_id
        collections.append({"revision": c["revision"], "request_id": request_id, "present": c["present"],
                            "name": c["name"].encode(), "origin": c.get("origin", 0)})
    g, clocks = state["generation"], state["clocks"]
    reflected = next(e["request_id"] for e in entries + collections if e["revision"])
    newer = vd.encode_table(g, reflected, entries, collections, clocks, fmt=fmt)
    older = vd.encode_table(g - 1, reflected, entries, collections, [clocks[0] - 1] + clocks[1:], fmt=fmt)
    # A torn g - 1 copy (one unsealed sector): explained by the newer copy, never chosen.
    older[-1] = bytes([older[-1][0] ^ 1]) + older[-1][1:]
    for generation, sectors in ((g, newer), (g - 1, older)):
        start = (vd.FIRST + (generation % 2) * vd.TABLE_SECTORS) * vd.SECTOR
        image[start:start + len(sectors) * vd.SECTOR] = b"".join(sectors)
    return bytes(image)


def archive_of(source) -> bytes:
    """What export_v2 writes for domain 0 of `source` (spec-derived encoder)."""
    collections = [dict(index=c["index"], present=c["present"], revision=c["revision"], name=c["name"])
                   for c in source["collections"] if c["domain"] == 0 and c["revision"]]
    objects = []
    for item in source["objects"][:vd.PER_DOMAIN]:
        if not item["revision"]:
            continue
        keep = rv.replayable([v["revision"] for v in item["versions"]]) if item["present"] else 0
        objects.append(dict(slot=item["object"], present=item["present"], collection=item["collection"],
                            kind=item["kind"], tags=item["tags"], revision=item["revision"],
                            created=item["created"], modified=item["modified"],
                            versions=[(v["revision"], v["name"],
                                       vd.fixture(item["object"], v["content_revision"], v["length"]))
                                      for v in item["versions"][:keep]]))
    return encode(collections, objects, flags=ACCOUNTED, clock=source["clocks"][0])


def destination_state(state, base=None):
    """Oracle-shaped state of a fresh volume after expected_l2 `state`."""
    objects = [oracle_object(o) for o in range(vd.OBJECTS)]
    for slot, want in state["objects"].items():
        objects[slot] = dict(oracle_object(slot), revision=want["revision"], present=True,
                             collection=want["collection"], tags=want["tags"], kind=want["kind"],
                             created=want["created"], modified=want["modified"], request_id=None,
                             installed=want["installed"], origin=tuple(want["origin"]), origins=want["origins"],
                             versions=[{"revision": v["revision"], "length": v["length"], "name": v["name"],
                                        "content": v["content"]} for v in want["versions"]])
    collections = [{"index": i % vd.COLLECTIONS + 1, "domain": i // vd.COLLECTIONS, "present": False,
                    "revision": 0, "name": ""} for i in range(vd.DOMAINS * vd.COLLECTIONS)]
    for c in collections:
        want = state["collections"].get(c["index"]) if c["domain"] == 0 else None
        if want:
            c.update(present=True, revision=1, name=want["name"], origin=want["origin"], request_id=None)
    return {"generation": state["generation"], "clocks": state["clocks"], "objects": objects,
            "collections": collections}


def device_evidence(case, counts):
    return {"VOLUME:ARCHIVE:DEVICE:": 1, "VOLUME:ARCHIVE:DISK_OFFLINE:": 0, "VOLUME:ARCHIVE:DISK_FOUND:": 1,
            "VOLUME:ARCHIVE:SCOPE_REFUSALS:": 1, "VOLUME:ARCHIVE:ESCAPE_DENIALS:": 2,
            "VOLUME:ARCHIVE:RIGHTS_REFUSALS:": int(case != "l2-export"),
            **{f"VOLUME:ARCHIVE:{k}:": v for k, v in zip(("READS", "WRITES", "FLUSHES"), counts)},
            **{f"VOLUME:BROKER:ARCHIVE_DISK_{k}:": v for k, v in zip(("READS", "WRITES", "FLUSHES"), counts)}}


def words(**values):
    out = {f"VOLUME:ARCHIVE:{name}:": 0 for name in rv.L2_WORDS}
    out.update({f"VOLUME:ARCHIVE:{name.upper()}:": value for name, value in values.items()})
    if out["VOLUME:ARCHIVE:RESULT:"] < 0:
        out["VOLUME:ARCHIVE:RESULT:"] += 1 << 64
    return out


class Fixture:
    """The whole synthetic L2 chain from PARENT with the given source step."""

    def __init__(self, steps=None):
        steps = rv.L2_SOURCE_STEP if steps is None else steps
        self.steps = steps
        self.before = volume_image(parent_state())
        source = rv.expected_source(vd.analyse(self.before), steps)
        self.after = volume_image(source)
        self.source = vd.analyse(self.after)
        self.archive_bytes = archive_of(self.source)
        self.archive = va.parse_v2(self.archive_bytes)
        self.state, self.pairs = rv.expected_l2(self.archive)
        end = -(-len(self.archive_bytes) // rv.SECTOR) * rv.SECTOR
        self.disk1_before = rv.disk1_image(rv.DISK1_SECTORS)
        self.disk1 = (self.archive_bytes + bytes(end - len(self.archive_bytes))
                      + self.disk1_before[end:])
        self.restored = volume_image(destination_state(self.state), fmt=3)

    def export_evidence(self):
        end = -(-len(self.archive_bytes) // rv.SECTOR)
        return {**device_evidence("l2-export", [5, end, 1]),
                **words(skipped=rv.matches_volume_v2(self.archive, self.source), length=len(self.archive_bytes)),
                "VOLUME:SERVICE:COMMITS:": len(self.steps), "VOLUME:SERVICE:GENERATION:": self.source["generation"],
                "VOLUME:BROKER:VOLUME_DISK_WRITES:": 99, "VOLUME:BROKER:VOLUME_DISK_FLUSHES:": 9,
                "VOLUME:A:DIGEST:": self.source["digests"][0], "VOLUME:B:DIGEST:": self.source["digests"][1]}

    def restore_evidence(self, oracle):
        commits = self.state["report"]["committed"]
        return {**device_evidence("l2-restore", [7, 0, 0]), **words(**self.state["report"]),
                "VOLUME:SERVICE:COMMITS:": commits, "VOLUME:SERVICE:TABLE_WRITES:": commits,
                "VOLUME:BROKER:TABLE_SECTOR_WRITES:": 1 + commits * vd.TABLE_SECTORS,
                "VOLUME:SERVICE:GENERATION:": commits,
                "VOLUME:BROKER:VOLUME_DISK_WRITES:": 1 + commits * vd.TABLE_SECTORS + 30,
                "VOLUME:BROKER:VOLUME_DISK_FLUSHES:": 2 * commits + 1,
                "VOLUME:A:DIGEST:": oracle["digests"][0], "VOLUME:B:DIGEST:": oracle["digests"][1]}


class Discrimination(unittest.TestCase):
    """Spec section 2: the oracle computes every (origin, live) pair and
    fails the stage when one is equal."""

    def test_guest_source_step_is_discriminating(self):
        # app.rs l2_source_step (S1-S5) as mirrored: no origin equals its live value.
        fixture = Fixture()
        self.assertEqual([pair for pair in fixture.pairs if pair[1] == pair[2]], [])
        rv.discriminating(fixture.pairs)
        # Chunk-neutral: the source keeps its usage; the replay stays at 24 chunks.
        self.assertEqual(fixture.source["chunks_used"], vd.analyse(fixture.before)["chunks_used"])
        self.assertEqual(fixture.state["chunks_used"], 24)

    def test_s1_to_s4_alone_trips_the_invariant(self):
        # Without S5, objects 1 and 5 keep version revision 1, which the destination assigns too.
        pairs = Fixture(S1_S4).pairs
        self.assertEqual([pair for pair in pairs if pair[1] == pair[2]],
                         [("object 1 version 0", 1, 1), ("object 5 version 0", 1, 1)])
        with self.assertRaisesRegex(ValueError, "non-discriminating"):
            rv.discriminating(pairs)

    def test_one_equal_pair_trips_the_invariant(self):
        pairs = Fixture().pairs
        for index, (label, origin, live) in enumerate(pairs):
            with self.assertRaisesRegex(ValueError, "non-discriminating", msg=label):
                rv.discriminating(pairs[:index] + [(label, live, live)] + pairs[index + 1:])


class ExpectedState(unittest.TestCase):
    def test_source_and_destination_values(self):
        fixture = Fixture()
        source = fixture.source
        self.assertEqual((source["generation"], source["clocks"], rv.domain_generation(source)), (95, [68, 27], 69))
        self.assertEqual({o: [(v["revision"], v["length"]) for v in source["objects"][o]["versions"]]
                          for o in (1, 2, 5)},
                         {1: [(5, 129), (4, 0)], 2: [(4, 0), (3, 0), (1, 0)], 5: [(4, 65536), (3, 0)]})
        self.assertEqual(rv.matches_volume_v2(fixture.archive, source), 1)
        self.assertEqual((len(fixture.archive_bytes), rv.flip_v2(fixture.archive_bytes)), (92697, 59849))
        state = fixture.state
        self.assertEqual(state["report"], {"restored": 0b0011_1110, "collections": 0b111, "committed": 8,
                                           "skipped": 0})
        self.assertEqual((state["generation"], state["clocks"], state["domain_generation"], state["format"],
                          state["chunks_used"]), (8, [8, 0], 11, 3, 24))
        self.assertEqual({s: (o["revision"], o["created"], o["origin"], o["origins"])
                          for s, o in state["objects"].items()},
                         {1: (2, 4, [5, 2, 66], [5, 4, 0]), 2: (2, 5, [4, 3, 59], [4, 3, 0]),
                          3: (1, 6, [42, 5, 54], [41, 0, 0]), 4: (1, 7, [4, 46, 53], [4, 0, 0]),
                          5: (2, 8, [4, 55, 68], [4, 3, 0])})
        self.assertEqual(state["collections"], {1: {"name": "Werkmap", "origin": 2},
                                                2: {"name": "Thuis", "origin": 2},
                                                3: {"name": "Nieuwer", "origin": 2}})

    def test_source_step_mirrors_the_app(self):
        body = APP.read_text(encoding="utf-8")
        body = body[body.index("fn l2_source_step"):]
        body = body[:body.index("\n}\n")]
        steps = []
        for line in body.splitlines():
            write = re.search(r"app\.write\((\d+), [^,]+, [^,]+, (\d+), \d+\)", line)
            if write:
                steps.append(("write", int(write.group(1)), int(write.group(2))))
        meta = re.search(r"app\.meta\(\s*(\d+),\s*stat\.revision,\s*stat\.tags \^ (\d+),\s*stat\.kind,"
                         r"\s*stat\.collection,", body)
        self.assertIsNotNone(meta)
        steps.append(("meta", int(meta.group(1)), int(meta.group(2))))
        for index, expected, name in re.findall(r"\((\d+), (\d+), &b\"([^\"]+)\"\[\.\.\], \d+\)", body):
            steps.append(("collection", int(index), int(expected), name))
        # S5: per object of the loop list, a zero-length WRITE, then RESTORE of history slot 1.
        s5 = re.search(r"for \(index, step\) in \[([^\]]+)\] \{(.*?)\n    \}", body, re.S)
        self.assertIsNotNone(s5)
        self.assertIn("app.write(index, stat.revision, &name[..length], 0, step + 1)", s5.group(2))
        restore = re.search(r"app\.client\.restore\(app\.object\(index\), revision, (\d+)\)\s*== Ok\(revision \+ 1\)",
                            s5.group(2))
        self.assertIsNotNone(restore)
        for index, _ in re.findall(r"\((\d+), (\d+)\)", s5.group(1)):
            steps += [("write", int(index), 0), ("restore", int(index), int(restore.group(1)))]
        self.assertEqual(tuple(steps), rv.L2_SOURCE_STEP)
        # The first WRITE reads the stat revision, the second chains its reply.
        self.assertIn("app.write(2, stat.revision,", body)
        self.assertIn("app.write(2, revision,", body)


class ExportCheck(unittest.TestCase):
    def run_export(self, fixture, after=None, disk1=None, evidence=None):
        return rv.check_l2("l2-export", fixture.before, after or fixture.after, fixture.disk1_before,
                           disk1 or fixture.disk1, evidence or fixture.export_evidence())

    def test_guest_export_is_accepted(self):
        oracle = self.run_export(Fixture())
        self.assertEqual(oracle["l2"]["omitted"], 1)
        self.assertEqual(len(rv.L2_SOURCE_STEP), 11)

    def test_s1_to_s4_export_fails_as_non_discriminating(self):
        fixture = Fixture(S1_S4)
        with unittest.mock.patch.object(rv, "L2_SOURCE_STEP", fixture.steps):
            with self.assertRaisesRegex(ValueError, "non-discriminating"):
                self.run_export(fixture)

    def test_export_mutations_are_refused(self):
        fixture = Fixture()
        missing_step = volume_image(rv.expected_source(vd.analyse(fixture.before), fixture.steps[:-1]))
        flipped = bytearray(fixture.disk1)
        flipped[rv.flip_v2(fixture.archive_bytes)] ^= 1
        past = bytearray(fixture.disk1)
        past[-1] ^= 1
        cases = {"source step not applied": dict(after=missing_step),
                 "archive byte flipped": dict(disk1=bytes(flipped)),
                 "write past the archive": dict(disk1=bytes(past))}
        for key, value in (("VOLUME:ARCHIVE:SKIPPED:", 0), ("VOLUME:ARCHIVE:LENGTH:", 1),
                           ("VOLUME:SERVICE:COMMITS:", 7), ("VOLUME:ARCHIVE:WRITES:", 1),
                           ("VOLUME:ARCHIVE:RIGHTS_REFUSALS:", 1)):
            evidence = fixture.export_evidence()
            evidence[key] = value
            cases[key] = dict(evidence=evidence)
        for label, change in cases.items():
            with self.assertRaises(ValueError, msg=label):
                self.run_export(fixture, **change)

    def test_archive_must_equal_the_source(self):
        fixture = Fixture()
        source = json.loads(json.dumps(fixture.source))
        for mutate in (lambda s: s["objects"][3].update(created=6),
                       lambda s: s["objects"][2]["versions"].pop(1),
                       lambda s: s["collections"][2].update(revision=1)):
            changed = json.loads(json.dumps(source))
            mutate(changed)
            with self.assertRaises(ValueError):
                rv.matches_volume_v2(va.parse_v2(archive_of(changed)), source)


class RestoreCheck(unittest.TestCase):
    def setUp(self):
        self.fixture = Fixture()

    def check(self, case, after, before=None, evidence=None):
        fixture = self.fixture
        oracle = rv.analyse_v3(after, opaque=frozenset(fixture.state["objects"]))
        evidence = evidence or fixture.restore_evidence(oracle)
        return rv.check_l2(case, before or vd.empty_image_bytes(), after, fixture.disk1, fixture.disk1, evidence)

    def test_restored_state_is_accepted(self):
        oracle = self.check("l2-restore", self.fixture.restored)
        self.assertEqual(oracle["format"], 3)

    def test_restore_of_the_s1_to_s4_fixture_is_refused(self):
        fixture = Fixture(S1_S4)
        restored = volume_image(destination_state(fixture.state), fmt=3)
        oracle = rv.analyse_v3(restored, opaque=frozenset(fixture.state["objects"]))
        with self.assertRaisesRegex(ValueError, "non-discriminating"):
            rv.check_l2("l2-restore", vd.empty_image_bytes(), restored, fixture.disk1, fixture.disk1,
                        Fixture.restore_evidence(fixture, oracle))

    def mutated(self, change):
        state = destination_state(self.fixture.state)
        change(state)
        return volume_image(state, fmt=3)

    def test_provenance_mutations_are_refused(self):
        def origin(slot, index, value):
            def change(state):
                item = state["objects"][slot]
                item["origin"] = tuple(value if i == index else v for i, v in enumerate(item["origin"]))
            return change

        def version_origin(slot, value):
            def change(state):
                state["objects"][slot]["origins"] = [value] + state["objects"][slot]["origins"][1:]
            return change

        def collection(index, **fields):
            def change(state):
                state["collections"][index - 1].update(fields)
            return change

        mutations = {
            "flipped object origin": origin(3, 0, 43),
            # 3.1: live values written into each formerly colliding field.
            "collection 1 origin = live": collection(1, origin=1),
            "collection 2 origin = live": collection(2, origin=1),
            "object 5 origin revision = live": origin(5, 0, 2),
            "object 3 origin created = live": origin(3, 1, 6),
            "object 1 version origin = live": version_origin(1, 2),
            "object 5 version origin = live": version_origin(5, 2),
            "missing collection origin": collection(3, origin=0),
            "slot 0 installed": lambda s: s["objects"][0].update(
                revision=1, present=True, created=8, modified=8, request_id=None, installed=1,
                origin=(4, 1, 57), origins=[4, 0, 0],
                versions=[{"revision": 1, "length": 0, "name": "x", "content": b""}]),
        }
        for label, change in mutations.items():
            with self.assertRaises(ValueError, msg=label):
                self.check("l2-restore", self.mutated(change))

    def test_content_and_report_mutations_are_refused(self):
        restored = bytearray(self.fixture.restored)
        oracle = rv.analyse_v3(self.fixture.restored, opaque=frozenset(self.fixture.state["objects"]))
        for key, value in (("VOLUME:ARCHIVE:RESTORED:", 0b0011_1111), ("VOLUME:ARCHIVE:COMMITTED:", 7),
                           ("VOLUME:ARCHIVE:PROVENANCE_MISMATCHES:", 1), ("VOLUME:SERVICE:TABLE_WRITES:", 9),
                           ("VOLUME:BROKER:TABLE_SECTOR_WRITES:", 274), ("VOLUME:ARCHIVE:RESULT:", 1)):
            evidence = self.fixture.restore_evidence(oracle)
            evidence[key] = value
            with self.assertRaises(ValueError, msg=key):
                self.check("l2-restore", bytes(restored), evidence=evidence)

    def test_outside_slots_stay_never_used(self):
        # Domain 1 or an unused domain-0 slot touched after the restore.
        def touch(slot):
            def change(state):
                state["objects"][slot].update(revision=1, present=True, created=1, modified=1, request_id=None,
                                              versions=[{"revision": 1, "length": 0, "name": "x",
                                                         "content_revision": 1}])
                if slot >= vd.PER_DOMAIN:
                    state.update(generation=state["generation"] + 1, clocks=[state["clocks"][0], 1])
            return change
        with self.assertRaisesRegex(ValueError, "object 6 is not never-used"):
            self.check("l2-restore", self.mutated(touch(6)))
        with self.assertRaisesRegex(ValueError, "destination table"):
            self.check("l2-restore", self.mutated(touch(9)))
        with self.assertRaises(ValueError):
            self.check("l2-restore", self.mutated(lambda s: s["collections"][3].update(
                revision=1, present=True, name="Extra", request_id=None)))

    def test_used_domain(self):
        oracle = rv.analyse_v3(self.fixture.restored, opaque=frozenset(self.fixture.state["objects"]))
        evidence = {**self.fixture.restore_evidence(oracle), **words(result=rv.STATUS_STALE),
                    "VOLUME:BROKER:VOLUME_DISK_WRITES:": 0, "VOLUME:BROKER:VOLUME_DISK_FLUSHES:": 0}
        rv.check_l2("l2-used", self.fixture.restored, self.fixture.restored, self.fixture.disk1,
                    self.fixture.disk1, evidence)
        for key, value in (("VOLUME:ARCHIVE:RESULT:", 0), ("VOLUME:ARCHIVE:COMMITTED:", 1),
                           ("VOLUME:BROKER:VOLUME_DISK_FLUSHES:", 1)):
            wrong = dict(evidence, **{key: value})
            with self.assertRaises(ValueError, msg=key):
                rv.check_l2("l2-used", self.fixture.restored, self.fixture.restored, self.fixture.disk1,
                            self.fixture.disk1, wrong)


class CorruptCheck(unittest.TestCase):
    def setUp(self):
        self.fixture = Fixture()
        self.offset = rv.flip_v2(self.fixture.archive_bytes)
        disk1 = bytearray(self.fixture.disk1)
        disk1[self.offset] ^= 1
        self.disk1 = bytes(disk1)
        self.evidence = {**device_evidence("l2-corrupt", [3, 0, 0]), **words(result=rv.STATUS_CORRUPT),
                         "VOLUME:SERVICE:COMMITS:": 0, "VOLUME:SERVICE:TABLE_WRITES:": 0,
                         "VOLUME:BROKER:VOLUME_DISK_WRITES:": 0, "VOLUME:BROKER:VOLUME_DISK_FLUSHES:": 1}

    def test_flip_hits_content_of_the_largest_object(self):
        archive = self.fixture.archive
        largest = max(archive["objects"], key=lambda o: sum(v["length"] for v in o["versions"]))
        self.assertEqual(largest["slot"], 5)
        # The flipped archive is refused by a content checksum, not by a record seal.
        with self.assertRaisesRegex(ValueError, "content checksum of a slot 5"):
            va.parse_v2(self.disk1[:len(self.fixture.archive_bytes)])

    def test_corrupt_stage(self):
        empty = vd.empty_image_bytes()
        rv.check_l2("l2-corrupt", empty, empty, self.disk1, self.disk1, self.evidence)
        written = bytearray(empty)
        written[vd.FIRST * vd.SECTOR] = 1
        for label, args in (("disk 0 written", (empty, bytes(written), self.disk1, self.disk1, self.evidence)),
                            ("intact archive", (empty, empty, self.fixture.disk1, self.fixture.disk1, self.evidence)),
                            ("status Ok", (empty, empty, self.disk1, self.disk1, {**self.evidence, **words()})),
                            ("a commit", (empty, empty, self.disk1, self.disk1,
                                          dict(self.evidence, **{"VOLUME:SERVICE:COMMITS:": 1})))):
            with self.assertRaises(ValueError, msg=label):
                rv.check_l2("l2-corrupt", *args)


class Extraction(unittest.TestCase):
    def test_v2_archive_is_never_read_with_the_v1_length(self):
        fixture = Fixture()
        data = rv.disk_archive(fixture.disk1)
        self.assertEqual(data, fixture.archive_bytes)
        v1_length = va.stored_length(fixture.archive_bytes[:va.HEADER])
        self.assertNotEqual(v1_length, len(fixture.archive_bytes))
        with self.assertRaises(ValueError):
            va.parse_v2(fixture.disk1[:v1_length])

    def test_v2_length_beyond_the_bound_is_refused(self):
        header = bytearray(Fixture().archive_bytes[:va.HEADER_V2])
        header[24:32] = (va.MAX_ARCHIVE_V2).to_bytes(8, "little")
        with self.assertRaisesRegex(ValueError, "v2 bound"):
            rv.stored_archive_in(bytes(header) + bytes(10))


class Preparation(unittest.TestCase):
    """Spec section 4: one explicit branch per case; 6.8 source loss."""

    def test_every_l2_case_has_its_preparation(self):
        fixture = Fixture()
        with tempfile.TemporaryDirectory() as tmp:
            parent = Path(tmp) / "parent"
            parent.mkdir()
            (parent / "volume.img").write_bytes(fixture.before)
            (parent / "archive.img").write_bytes(fixture.disk1)
            info = {"identity": "p" * 32}
            (parent / "volume.json").write_text(json.dumps(info))
            results = {}
            for case in rv.L2_CASES:
                target = Path(tmp) / case
                target.mkdir()
                results[case] = (target, *rv.prepare_disks(case, parent, target, info))
            target, _, disk1, reference, flip = results["l2-export"]
            self.assertEqual((target / "volume.img").read_bytes(), rv.outside_archive(fixture.before))
            self.assertEqual((disk1, reference, flip), (rv.disk1_image(rv.DISK1_SECTORS), None, None))
            for case in rv.L2_SOURCE_LOSS:
                target, new, disk1, _, flip = results[case]
                self.assertEqual((target / "volume.img").read_bytes(), vd.empty_image_bytes())
                self.assertEqual(flip, rv.flip_v2(fixture.archive_bytes) if case == "l2-corrupt" else None)
                self.assertEqual(sum(a != b for a, b in zip(disk1, fixture.disk1)), case == "l2-corrupt")
                args = ["-drive", "if=pflash,file=a", "-drive", "if=pflash,file=b", "-drive", "if=virtio,file=fat:e",
                        "-drive", f"if=none,id=volume,file={target / 'volume.img'}",
                        "-drive", f"if=none,id=archive,file={target / 'archive.img'}"]
                rv.check_source_loss(vd.empty_image_bytes(), args, parent, new, Path("/"))
                with self.assertRaises(ValueError):
                    rv.check_source_loss(fixture.before, args, parent, new, Path("/"))
                with self.assertRaises(ValueError):
                    rv.check_source_loss(vd.empty_image_bytes(), args, parent, info, Path("/"))
                swapped = args[:-3] + [f"if=none,id=volume,file={parent / 'volume.img'}"] + args[-2:]
                with self.assertRaises(ValueError):
                    rv.check_source_loss(vd.empty_image_bytes(), swapped, parent, new, Path("/"))
                with self.assertRaises(ValueError):
                    rv.check_source_loss(vd.empty_image_bytes(), args + ["-drive", "if=none,id=x,file=y"],
                                         parent, new, Path("/"))
            target, _, disk1, _, _ = results["l2-used"]
            self.assertEqual(((target / "volume.img").read_bytes(), disk1), (fixture.before, fixture.disk1))
            with self.assertRaises(ValueError):
                rv.prepare_disks("l2-unknown", parent, Path(tmp) / "l2-export", info)

    def test_l2_case_never_reaches_the_v1_fall_through(self):
        for case in rv.L2_CASES:
            with self.assertRaisesRegex(ValueError, "v1 restore check"):
                rv.check_restored(case, b"", b"", {})
        with self.assertRaisesRegex(ValueError, "no second-disk check"):
            rv.check_disk("archive-disk-unknown", b"", b"", b"", b"", device_evidence("x", [1, 0, 0]), None)


class Plan(unittest.TestCase):
    def plan(self):
        tree = ast.parse((ROOT / "tools/test_volume.py").read_text(encoding="utf-8"))
        return next(ast.literal_eval(node.value) for node in tree.body
                    if isinstance(node, ast.Assign) and getattr(node.targets[0], "id", None) == "PLAN")

    def test_every_parent_is_the_last_writer_of_its_directory(self):
        plan = self.plan()
        self.assertEqual(len(plan), 30)
        self.assertEqual(plan[-4:], [("l2-export", "archive-export"), ("l2-restore", "l2-export"),
                                     ("l2-corrupt", "l2-export"), ("l2-used", "l2-restore")])
        directory, writer = {}, {}
        for number, (case, parent) in enumerate(plan):
            if parent is not None:
                self.assertEqual(writer[directory[parent]], parent, f"stage {number} {case}")
            where = number if rv.new_directory(case) else directory[parent]
            directory[case] = writer_key = where
            writer[writer_key] = case

    def test_tables(self):
        import build
        self.assertEqual(build.VOLUME_L2_CASES, rv.L2_CASES)
        groups = (build.VOLUME_L2_CASES, build.VOLUME_ARCHIVE_CASES, rv.INSTALL_CASES, build.VOLUME_SETTINGS_CASES)
        for i, first in enumerate(groups):
            for second in groups[i + 1:]:
                self.assertFalse(set(first) & set(second), (first, second))
        for case in rv.L2_CASES:
            self.assertEqual(rv.FEATURE[case], f"test-volume-{case}")
            self.assertIn(rv.FEATURE[case], build.VOLUME_FEATURES)
            self.assertIn(case, rv.DISK_CASES)
        self.assertEqual({c: rv.CASE_MARKER[c] for c in rv.L2_CASES},
                         {"l2-export": "L2_EXPORT", "l2-restore": "L2_RESTORE", "l2-corrupt": "L2_CORRUPT",
                          "l2-used": "L2_USED"})
        self.assertEqual([c for c in rv.L2_CASES if c in rv.DISK1_READ_ONLY], ["l2-restore", "l2-corrupt", "l2-used"])
        self.assertEqual([c for c in rv.L2_CASES if c in rv.READ_ONLY], ["l2-used"])
        self.assertEqual(rv.signed(18446744073709551598), rv.STATUS_CORRUPT)

if __name__ == "__main__":
    unittest.main()
