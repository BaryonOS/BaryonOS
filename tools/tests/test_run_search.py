"""Host counterexamples for tools/run_search.py (U05-T3 S2). The transcripts
and image views are hand-built literals; these tests are not native evidence
and do not exercise the image -> volume_disk.analyse path (step S3)."""
import copy
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_search as search
import test_search_oracle as oracle

MARKERS = ["BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "VOLUME:MANIFEST:PASS", "VOLUME:READY",
           "VOLUME:CASE:SEARCH:PASS", "VOLUME:RESOURCES:PASS", "VOLUME:PASS", "M1:PASS"]
# Hits of the fixed query set (own, notes, final, draft, budget, archive,
# projects, other), written out by hand from the scripted end state.
HITS = {
    0: [[(0, 2), (1, 2)], [(0, 2)], [(1, 2)], [], [], [(0, 2)], [], []],
    1: [[(8, 2), (9, 2)], [(8, 2)], [(9, 2)], [], [], [(8, 2)], [], []],
}


def chained(domain):
    value = oracle.FNV_OFFSET
    for hits in HITS[domain]:
        value = oracle.digest(value, hits)
    return value


def image_view():
    objects = []
    for obj in range(16):
        objects.append({"object": obj, "revision": 0, "present": False, "collection": 0, "versions": []})
    for base, own in ((0, "alpha"), (8, "beta")):
        objects[base].update(revision=2, present=True, collection=1, versions=[{"name": f"{own} notes"}])
        objects[base + 1].update(revision=2, present=True, versions=[{"name": f"{own} final"}])
        objects[base + 2].update(revision=2, versions=[{"name": f"{own} budget"}])
    collections = [{"index": i % 4 + 1, "domain": i // 4, "present": False, "revision": 0, "name": ""}
                   for i in range(8)]
    for i in (0, 4):
        collections[i].update(present=True, revision=2, name="Archive")
    return {"objects": objects, "collections": collections}


def words(**override):
    values = {}
    for domain, slot in enumerate("AB"):
        values.update({f"VOLUME:{slot}:DOCS:": 2, f"VOLUME:{slot}:DIGEST:": chained(domain),
                       f"VOLUME:{slot}:REQUESTS:": 60, f"VOLUME:{slot}:RECOVERED_REVISION:": 2,
                       f"VOLUME:{slot}:RECOVERED_COLLECTION:": 0, f"VOLUME:{slot}:RECOVERED_SECOND:": 0xFFF})
    values["VOLUME:SERVICE:REJECTED:"] = 4
    values.update(override)
    return values


def transcript(markers=MARKERS, values=None, extra=()):
    values = words() if values is None else values
    lines = list(markers[:5]) + [f"{k}{v:020d}" for k, v in values.items()] + list(markers[5:]) + list(extra)
    return "\n".join(lines) + "\n"


class VerdictTests(unittest.TestCase):
    def test_complete_run_passes(self):
        result = search.verdict(transcript(), 33, False, image_view())
        self.assertEqual(result["reported"]["A"]["digest"], chained(0))
        self.assertEqual(result["reported"]["B"]["own_hits"], 2)

    def test_oracle_matches_hand_written_hits(self):
        self.assertEqual(search.oracle_words(image_view(), 0), (2, chained(0), 2, 0))
        self.assertEqual(search.oracle_words(image_view(), 1), (2, chained(1), 2, 0))

    def test_missing_marker_fails(self):
        for marker in ("VOLUME:CASE:SEARCH:PASS", "M1:PASS", "VOLUME:READY"):
            with self.subTest(marker=marker), self.assertRaisesRegex(ValueError, "marker"):
                search.verdict(transcript([m for m in MARKERS if m != marker]), 33, False, image_view())

    def test_wrong_case_marker_fails(self):
        markers = [m.replace("SEARCH", "SETTINGS") for m in MARKERS]
        with self.assertRaisesRegex(ValueError, "marker"):
            search.verdict(transcript(markers), 33, False, image_view())

    def test_failure_marker_fails(self):
        with self.assertRaisesRegex(ValueError, "failure"):
            search.verdict(transcript(extra=["VOLUME:FAIL:SERVICE_STATE"]), 33, False, image_view())

    def test_wrong_exit_or_timeout_fails(self):
        for rc, timed_out in ((0, False), (1, False), (None, False), (33, True)):
            with self.subTest(rc=rc, timed_out=timed_out), self.assertRaisesRegex(ValueError, "raw exit"):
                search.verdict(transcript(), rc, timed_out, image_view())

    def test_reported_word_disagreeing_with_oracle_fails(self):
        for key, value in (("VOLUME:A:DIGEST:", chained(0) ^ 1), ("VOLUME:B:DOCS:", 3),
                           ("VOLUME:B:RECOVERED_COLLECTION:", 1), ("VOLUME:A:RECOVERED_REVISION:", 1)):
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, "oracle"):
                search.verdict(transcript(values=words(**{key: value})), 33, False, image_view())

    def test_image_not_in_scripted_state_fails(self):
        view = image_view()
        view["objects"][9]["versions"][0]["name"] = "beta draft"
        with self.assertRaisesRegex(ValueError, "object 9"):
            search.verdict(transcript(), 33, False, view)
        view = image_view()
        view["collections"][0]["name"] = "Projects"
        with self.assertRaisesRegex(ValueError, r"collection \(0, 1\)"):
            search.verdict(transcript(), 33, False, view)
        view = image_view()
        view["objects"][2]["present"] = True
        with self.assertRaisesRegex(ValueError, "object 2"):
            search.verdict(transcript(), 33, False, view)

    def test_missing_slot_b_word_fails(self):
        values = words()
        del values["VOLUME:B:RECOVERED_SECOND:"]
        with self.assertRaisesRegex(ValueError, "missing evidence"):
            search.verdict(transcript(values=values), 33, False, image_view())

    def test_incomplete_negatives_requests_or_rejections_fail(self):
        for key, value, pattern in (("VOLUME:B:RECOVERED_SECOND:", 0x7FF, "negative"),
                                    ("VOLUME:A:REQUESTS:", 47, "requests"),
                                    ("VOLUME:SERVICE:REJECTED:", 5, "rejected"),
                                    ("VOLUME:SERVICE:REJECTED:", 3, "rejected")):
            with self.subTest(key=key, value=value), self.assertRaisesRegex(ValueError, pattern):
                search.verdict(transcript(values=words(**{key: value})), 33, False, image_view())

    def test_duplicate_evidence_fails(self):
        text = transcript(extra=[f"VOLUME:A:DOCS:{2:020d}"])
        with self.assertRaisesRegex(ValueError, "duplicate"):
            search.verdict(text, 33, False, image_view())


class WiringTests(unittest.TestCase):
    def test_missing_wiring_refuses_before_boot(self):
        with tempfile.TemporaryDirectory() as root:
            with self.assertRaises(SystemExit) as raised:
                search.require_wiring(Path(root))
        message = str(raised.exception)
        for part in ("test-volume-search", "VOLUME:CASE:SEARCH:PASS", "VOLUME_SEARCH_CASES", "run_settings"):
            self.assertIn(part, message)


if __name__ == "__main__":
    unittest.main()
