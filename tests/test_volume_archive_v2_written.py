"""Differential check of the Rust archive v2 writer (services/volume/src/archive.rs export_v2).

services/volume/tests/archive_v2_writer.rs exports two domains of a fixed scenario built through
the real volume service and commits the archives under
services/volume/tests/fixtures/archive-v2-written/, with manifest.txt derived from the source
volume table (not from the writer's bytes). That Rust test requires the committed bytes to equal a
fresh export; this test requires the independent reader (tools/volume_archive.py parse_v2), which
shares no code with volume-core, to accept every archive with exactly the manifest's values.
"""
from __future__ import annotations
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
sys.path.insert(0, str(ROOT / "tests"))
import volume_archive as va  # noqa: E402
from test_volume_archive_v2_fixtures import manifest  # noqa: E402

WRITTEN = ROOT / "services/volume/tests/fixtures/archive-v2-written"


class WrittenArchives(unittest.TestCase):
    def test_oracle_accepts_the_written_archives_with_the_source_values(self) -> None:
        names = sorted(p.name for p in WRITTEN.glob("*.bin"))
        self.assertEqual(names, ["domain0.bin", "domain1.bin"])
        data = {name: ((WRITTEN / name).read_bytes(), None) for name in names}
        self.assertEqual(manifest(data), (WRITTEN / "manifest.txt").read_text(encoding="utf-8"))

    def test_written_policy(self) -> None:
        for path in WRITTEN.glob("*.bin"):
            with self.subTest(file=path.name):
                parsed = va.parse_v2(path.read_bytes())
                # R2: never tombstones; S9 always checked (accounted).
                self.assertEqual((parsed["tombstones"], parsed["accounted"]), (False, True))
                self.assertTrue(all(o["versions"] or not o["present"] for o in parsed["objects"]))
                for o in parsed["objects"]:
                    revisions = [v["revision"] for v in o["versions"]]
                    # Only the consecutive-revision prefix is written.
                    self.assertEqual(revisions, list(range(revisions[0], revisions[0] - len(revisions), -1))
                                     if revisions else [])


if __name__ == "__main__":
    unittest.main()
