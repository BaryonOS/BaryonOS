"""Host-only source-integrity checks, no decoder or native success evidence."""
import importlib.util
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
        self.assertEqual(result["changed_files"], ["src/upsampler/scalar.rs"])
        self.assertEqual(result["unchanged_files"], 40)

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


if __name__ == "__main__":
    unittest.main()
