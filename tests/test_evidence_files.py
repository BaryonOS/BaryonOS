import hashlib
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from update_evidence_files import image_metadata, update_index

ROOT = Path(__file__).resolve().parents[1]


def png(width, height):
    return (b"\x89PNG\r\n\x1a\n" + struct.pack(">I", 13) + b"IHDR"
            + struct.pack(">II", width, height) + bytes(5))


def jpeg(width, height):
    return (b"\xff\xd8\xff\xc0" + struct.pack(">H", 11)
            + bytes((8,)) + struct.pack(">HH", height, width) + bytes((1, 1, 0x11, 0))
            + b"\xff\xd9")


class EvidenceImageMetadataTests(unittest.TestCase):
    def test_detects_png_and_jpeg_from_signatures(self):
        self.assertEqual(image_metadata(png(320, 240)), {"format": "png", "dimensions": [320, 240]})
        self.assertEqual(image_metadata(jpeg(305, 804)), {"format": "jpeg", "dimensions": [305, 804]})

    def test_rejects_unknown_and_truncated_image_data(self):
        for data in (b"not an image", b"\x89PNG\r\n\x1a\n", b"\xff\xd8"):
            with self.subTest(data=data), self.assertRaises(ValueError):
                image_metadata(data)

    def test_updates_registry_metadata_and_hash_from_image_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            verification = Path(temporary)
            screenshots = verification / "screenshots"
            screenshots.mkdir()
            image = jpeg(305, 804)
            (screenshots / "access-320.png").write_bytes(image)
            index_path = verification / "evidence-files.json"
            index_path.write_text(json.dumps({
                "screenshots/access-320.png": {"sha256": "", "bytes": 0, "dimensions": [0, 0]},
                "browser-report.json": {"sha256": "unchanged"},
            }), encoding="utf-8")

            self.assertEqual(update_index(index_path, verification), 1)

            index = json.loads(index_path.read_text(encoding="utf-8"))
            self.assertEqual(index["screenshots/access-320.png"], {
                "sha256": hashlib.sha256(image).hexdigest(),
                "bytes": len(image),
                "format": "jpeg",
                "dimensions": [305, 804],
            })
            self.assertEqual(index["browser-report.json"], {"sha256": "unchanged"})

    def test_checked_in_screenshot_registry_matches_file_signatures(self):
        verification = ROOT / "docs/verification"
        index = json.loads((verification / "evidence-files.json").read_text(encoding="utf-8"))
        screenshots = {path: record for path, record in index.items() if path.startswith("screenshots/")}
        self.assertEqual(len(screenshots), 5)
        for relative_path, record in screenshots.items():
            data = (verification / relative_path).read_bytes()
            with self.subTest(path=relative_path):
                self.assertEqual(record["format"], "jpeg")
                self.assertEqual(record["dimensions"], image_metadata(data)["dimensions"])
                self.assertEqual(record["sha256"], hashlib.sha256(data).hexdigest())
                self.assertEqual(record["bytes"], len(data))


if __name__ == "__main__":
    unittest.main()
