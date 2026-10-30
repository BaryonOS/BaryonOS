"""Host-only source-integrity checks, no decoder or native success evidence."""
import importlib.util
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest

BASE = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("verify_vendor", BASE / "verify_vendor.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class VendorIntegrity(unittest.TestCase):
    def test_actual_reviewed_source_passes(self):
        result = module.verify()
        self.assertEqual(result["changed_files"], ["src/decoder.rs", "src/errors.rs", "src/mcu.rs", "src/upsampler/scalar.rs"])
        self.assertEqual(result["unchanged_files"], 37)

    def test_changed_source_and_archive_are_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            shutil.copytree(BASE / "vendor", base / "vendor")
            source = base / "vendor/zune-jpeg-0.5.16-rc3/src/decoder.rs"
            original = source.read_bytes()
            source.write_bytes(original + b"\n")
            with self.assertRaisesRegex(ValueError, "source changed"):
                module.verify(base)
            source.write_bytes(original)
            archive = base / "vendor/upstream/zune-jpeg-0.5.16-rc3.crate"
            archive.write_bytes(archive.read_bytes() + b"\x00")
            with self.assertRaisesRegex(ValueError, "archive checksum mismatch"):
                module.verify(base)

    def test_unreviewed_file_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            shutil.copytree(BASE / "vendor", base / "vendor")
            (base / "vendor/zune-jpeg-0.5.16-rc3/build.rs").write_text("fn main() {}\n")
            with self.assertRaisesRegex(ValueError, "file set changed"):
                module.verify(base)

    def test_snapshot_patch_must_be_the_exact_reviewed_source_diff(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            shutil.copytree(BASE / "vendor", base / "vendor")
            patch = base / "vendor/upstream/scan-header-heap.patch"
            patch.write_bytes(patch.read_bytes() + b"\n")
            with self.assertRaisesRegex(ValueError, "snapshot patch checksum"):
                module.verify(base)
            ledger = base / "vendor/upstream/source-verification.json"
            proof = json.loads(ledger.read_text())
            proof["snapshot_patch_sha256"] = hashlib.sha256(patch.read_bytes()).hexdigest()
            ledger.write_text(json.dumps(proof))
            with self.assertRaisesRegex(ValueError, "exact source delta"):
                module.verify(base)


if __name__ == "__main__":
    unittest.main()
