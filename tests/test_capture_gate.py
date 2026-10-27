"""Capture-gate tests use synthetic evidence and never claim a guest run."""
import hashlib
import json
import sys
import tempfile
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import capture_gate


class CaptureGateArchive(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.build = self.root / "build"
        self.destination = self.root / "docs/verification/m2-m4/m2"
        self.old_root = capture_gate.ROOT
        self.old_build = capture_gate.BUILD
        capture_gate.ROOT = self.root
        capture_gate.BUILD = self.build
        self.addCleanup(setattr, capture_gate, "ROOT", self.old_root)
        self.addCleanup(setattr, capture_gate, "BUILD", self.old_build)
        self.destination.mkdir(parents=True)
        (self.destination / "host.stdout.txt").write_text("old host output\n", encoding="utf-8")
        (self.destination / "suite.json").write_text('{"passed": true}\n', encoding="utf-8")
        (self.destination / "guest-runs.json").write_text('[{"passed": true}]\n', encoding="utf-8")
        (self.destination / "preserved.txt").write_text("previous archive\n", encoding="utf-8")
        self._write_suite()

    def _write_suite(self, *, valid_hash=False):
        host = self.build / "logs/host.stdout.txt"
        host.parent.mkdir(parents=True, exist_ok=True)
        host.write_text("new host output\n", encoding="utf-8")
        host.with_name("host.json").write_text('{"returncode": 0}\n', encoding="utf-8")
        host.with_name("host.stderr.txt").write_text("", encoding="utf-8")

        guest = self.build / "run/guest-1"
        guest.mkdir(parents=True, exist_ok=True)
        stdout = "M1:PASS\n"
        (guest / "serial.txt").write_text(stdout, encoding="utf-8")
        (guest / "stderr.txt").write_text("", encoding="utf-8")
        artifact = guest / "artifact.bin"
        artifact.write_bytes(b"synthetic artifact")
        digest = hashlib.sha256(artifact.read_bytes()).hexdigest() if valid_hash else "0" * 64
        result = {
            "stdout": stdout,
            "stderr": "",
            "raw_qemu_exit": 33,
            "timed_out": False,
            "required_markers": ["M1:PASS"],
            "expected_exit": 33,
            "forbidden_markers": [],
            "artifact_sha256": {"artifact.bin": digest},
        }
        (guest / "result.json").write_text(json.dumps(result), encoding="utf-8")
        suite = {
            "status": "completed",
            "passed": True,
            "results": [
                {"name": "host-tests", "passed": True, "evidence": str(host)},
                {"name": "guest-smoke", "passed": True, "evidence": str(guest)},
            ],
        }
        self.build.mkdir(parents=True, exist_ok=True)
        (self.build / "m2-test-results.json").write_text(json.dumps(suite), encoding="utf-8")

    def _snapshot(self):
        return {path.relative_to(self.destination).as_posix(): path.read_bytes()
                for path in self.destination.rglob("*") if path.is_file()}

    def test_validation_failure_leaves_existing_archive_unchanged(self):
        before = self._snapshot()

        with self.assertRaisesRegex(ValueError, "Artifact changed"):
            capture_gate.capture("m2")

        self.assertEqual(self._snapshot(), before)

    def test_valid_capture_replaces_archive_only_after_validation(self):
        self._write_suite(valid_hash=True)

        capture_gate.capture("m2")

        self.assertEqual((self.destination / "host.stdout.txt").read_text(encoding="utf-8"),
                         "new host output\n")
        self.assertTrue((self.destination / "preserved.txt").is_file())
        self.assertEqual(json.loads((self.destination / "suite.json").read_text())["status"],
                         "completed")
        self.assertEqual(len(json.loads((self.destination / "guest-runs.json").read_text())), 1)


if __name__ == "__main__":
    unittest.main()
