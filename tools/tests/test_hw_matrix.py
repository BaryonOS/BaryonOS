"""Synthetic matrix fixtures; no hardware execution or acceptance."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import hw_matrix


class HardwareMatrixTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.matrix = json.loads((hw_matrix.ROOT / hw_matrix.MATRIX).read_text())
        (self.root / hw_matrix.MATRIX).parent.mkdir(parents=True)
        for profile in self.matrix["profiles"]:
            for device in profile["devices"]:
                if device["driver_path"] is not None:
                    path = self.root / device["driver_path"]
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text("synthetic source")

    def check(self, matrix=None):
        (self.root / hw_matrix.MATRIX).write_text(json.dumps(self.matrix if matrix is None else matrix))
        return hw_matrix.check(self.root)

    def record(self, **changes):
        record = dict(passed=True, level="H", target="explicit-test-target", device="gop", source_sha256="a" * 64)
        record.update(changes)
        path = self.root / "docs/verification/hw/result.json"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(record))
        return "docs/verification/hw/result.json"

    def test_current_inventory_and_missing_fields(self):
        self.assertEqual(self.check()["profiles"], 3)
        for mutate in (lambda d: d["profiles"][0]["devices"][0].pop("dma_posture"),
                       lambda d: d["profiles"].pop(),
                       lambda d: d["profiles"][0]["devices"].pop(2),
                       lambda d: d["profiles"][0].update(kind="hardware"),
                       lambda d: d["profiles"][0]["devices"][2].update(status="supported")):
            matrix = copy.deepcopy(self.matrix)
            mutate(matrix)
            with self.assertRaises(ValueError):
                self.check(matrix)

    def test_dangling_and_escaping_references(self):
        device = self.matrix["profiles"][0]["devices"][0]
        for name in ("docs/missing", "../outside", "/etc/passwd", "docs//bad", "C:\\bad"):
            device["evidence"] = [name]
            with self.assertRaises((ValueError, OSError)):
                self.check()
        device["evidence"] = []
        device["driver_path"] = "missing/driver.rs"
        with self.assertRaises(OSError):
            self.check()

    def test_physical_support_requires_target_bound_h_record(self):
        profile = self.matrix["profiles"][2]
        device = profile["devices"][0]
        device["status"] = "supported"
        with self.assertRaises(ValueError):
            self.check()
        profile["target"] = "explicit-test-target"
        device["evidence"] = [self.record()]
        self.assertTrue(self.check()["structurally_valid"])
        for change in (dict(level="native"), dict(passed=False), dict(target="another-machine"),
                       dict(device="another-device"), dict(source_sha256="unknown")):
            self.record(**change)
            with self.assertRaises(ValueError):
                self.check()

    def test_vtd_posture_needs_its_own_device_bound_record(self):
        profile = self.matrix["profiles"][2]
        profile["target"] = "explicit-test-target"
        device = profile["devices"][0]
        device["dma_posture"] = "VT-d contained"
        device["evidence"] = [self.record()]
        with self.assertRaises(ValueError):
            self.check()
        self.record(dma_posture="VT-d contained")
        self.assertTrue(self.check()["structurally_valid"])


if __name__ == "__main__":
    unittest.main()
