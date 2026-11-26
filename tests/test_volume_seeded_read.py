"""`seeded-read` volume image (tools/volume_disk.py) for read-only mounts.

The Python oracle must accept the seed, and the table area must equal the
committed sparse fixture that the Rust cross-check
(services/volume/tests/seeded_read.rs) mounts read-only with volume-core.
Regenerate the fixture after a deliberate seed change with:
python3 -B -c "import tests.test_volume_seeded_read as t; t.write_fixture()"
"""
from __future__ import annotations
import hashlib
from pathlib import Path
import re
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import volume_disk as vd  # noqa: E402

FIXTURE = ROOT / "services/volume/tests/fixtures/seeded-read/table.txt"
SHA256 = "204ae9cd6e9dea28dceedc8e6a97923d5595b6de06dbe3211ec243d11ed54420"


def table_runs() -> str:
    """Non-zero byte runs of the volume table area: `sector offset hex`."""
    image = bytearray(vd.empty_image_bytes())
    vd.seed_read(image)
    area = image[vd.FIRST * vd.SECTOR:(vd.FIRST + vd.DATA_START) * vd.SECTOR]
    lines = []
    for sector in range(vd.DATA_START):
        raw = bytes(area[sector * vd.SECTOR:(sector + 1) * vd.SECTOR])
        lines += [f"{sector} {m.start()} {m.group().hex()}" for m in re.finditer(rb"[^\x00]+", raw)]
    return "\n".join(lines) + "\n"


def write_fixture() -> None:
    FIXTURE.write_text(table_runs(), encoding="ascii")


class SeededRead(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.path = Path(self.directory.name) / "volume.img"
        self.info = vd.create_image(self.path, "seeded-read")
        self.image = self.path.read_bytes()

    def tearDown(self):
        self.directory.cleanup()

    def test_oracle_accepts_the_seed(self):
        report = vd.analyse(self.image)
        self.assertEqual(report["generation"], 1)
        self.assertEqual(report["chunks_used"], 3)
        objects = report["objects"]
        for obj, name, length in vd.SEED_READ:
            self.assertTrue(objects[obj]["present"])
            [version] = objects[obj]["versions"]
            self.assertEqual((version["name"], version["length"], version["content_revision"]),
                             (name.decode(), length, 1))
        self.assertFalse(any(o["present"] or o["versions"] for o in objects[2:]))

    def test_image_is_deterministic(self):
        self.assertEqual(self.info["sha256"], SHA256)
        self.assertEqual(hashlib.sha256(self.image).hexdigest(), SHA256)

    def test_rust_fixture_matches_the_seed(self):
        self.assertEqual(FIXTURE.read_text(encoding="ascii"), table_runs())

    def test_data_outside_the_seed_is_zero(self):
        volume = self.image[vd.FIRST * vd.SECTOR:(vd.FIRST + vd.VOLUME_SECTORS) * vd.SECTOR]
        used = (vd.DATA_START + 3 * vd.CHUNK_SECTORS) * vd.SECTOR
        self.assertFalse(any(volume[used:]))


if __name__ == "__main__":
    unittest.main()
