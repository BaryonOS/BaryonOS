"""Capture-gate tests use synthetic evidence and never claim a guest run."""
import hashlib
import json
import sys
import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import capture_gate
from gate_contract import guest_expectation


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
        required, expected_exit, forbidden = guest_expectation("m2", "test-m2-nx")
        stdout = "\n".join(required) + "\n"
        # Match the runner's byte-preserving capture on Windows as well as Unix.
        (guest / "serial.txt").write_bytes(stdout.encode("utf-8"))
        (guest / "stderr.txt").write_text("", encoding="utf-8")
        artifact = guest / "artifact.bin"
        artifact.write_bytes(b"synthetic artifact")
        digest = hashlib.sha256(artifact.read_bytes()).hexdigest() if valid_hash else "0" * 64
        result = {
            "stdout": stdout,
            "stderr": "",
            "raw_qemu_exit": expected_exit,
            "timed_out": False,
            "required_markers": list(required),
            "expected_exit": expected_exit,
            "forbidden_markers": list(forbidden),
            "artifact_sha256": {"artifact.bin": digest},
        }
        (guest / "result.json").write_text(json.dumps(result), encoding="utf-8")
        suite = {
            "status": "completed",
            "passed": True,
            "results": [
                {"name": "host-tests", "passed": True, "evidence": str(host)},
                {"name": "test-m2-nx", "passed": True, "evidence": str(guest)},
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
        self.assertFalse((self.destination / "preserved.txt").exists())
        self.assertEqual(json.loads((self.destination / "suite.json").read_text())["status"],
                         "completed")
        self.assertEqual(len(json.loads((self.destination / "guest-runs.json").read_text())), 1)

    def test_guest_output_cannot_be_captured_as_a_host_log(self):
        guest_log = self.build / "run/guest.stdout.txt"
        guest_log.parent.mkdir(parents=True, exist_ok=True)
        guest_log.write_text("unverified guest output\n", encoding="utf-8")
        guest_log.with_name("guest.json").write_text('{"returncode": 0}\n', encoding="utf-8")
        guest_log.with_name("guest.stderr.txt").write_text("", encoding="utf-8")
        suite_path = self.build / "m2-test-results.json"
        suite = json.loads(suite_path.read_text(encoding="utf-8"))
        suite["results"][0]["evidence"] = str(guest_log)
        suite_path.write_text(json.dumps(suite), encoding="utf-8")
        before = self._snapshot()

        with self.assertRaises(ValueError):
            capture_gate.capture("m2")

        self.assertEqual(self._snapshot(), before)

    def test_guest_cases_cannot_be_substituted_with_a_valid_host_log(self):
        source_suite = json.loads(
            (self.build / "m2-test-results.json").read_text(encoding="utf-8"))
        for phase, group_name in (("m2", "test-m2-nx"), ("m4", "test-m4-copy")):
            suite_path = self.build / f"{phase}-test-results.json"
            suite = json.loads(json.dumps(source_suite))
            suite["results"][0]["name"] = group_name
            suite["results"][0]["evidence"] = str(self.build / "logs/host.stdout.txt")
            suite["results"].pop()
            suite_path.write_text(json.dumps(suite), encoding="utf-8")
            before = self._snapshot()

            with self.assertRaisesRegex(ValueError, "Guest case requires run-directory evidence"):
                capture_gate.capture(phase)

            self.assertEqual(self._snapshot(), before)

    def test_guest_evidence_is_bound_to_phase_and_case_not_record_expectations(self):
        record = {
            "stdout": "BOOT:EBS_OK\nKERNEL:ENTRY\nBOOTINFO:OK\nM1:PASS\n",
            "raw_qemu_exit": 33,
            "timed_out": False,
            "required_markers": ["M1:PASS"],
            "expected_exit": 33,
            "forbidden_markers": [],
        }

        self.assertFalse(capture_gate.guest_matches("m2", "test-m2-nx", record))
        self.assertFalse(capture_gate.guest_matches("m4", "test-m4-copy", record))
        self.assertFalse(capture_gate.guest_matches("m4", "unknown-case", record))

    def test_guest_record_cannot_omit_raw_timeout_status(self):
        record = {
            "stdout": "BOOT:EBS_OK\nKERNEL:ENTRY\nBOOTINFO:OK\nM1:PASS\n",
            "raw_qemu_exit": 33,
        }

        self.assertFalse(capture_gate.guest_matches("m2", "test-m2-nx", record))

    def test_guest_exit_status_is_bound_to_the_phase_and_case(self):
        required, _, _ = guest_expectation("m4", "test-m4-copy")
        record = {
            "stdout": "\n".join(required),
            "raw_qemu_exit": 35,
            "timed_out": False,
            "required_markers": required,
            "expected_exit": 35,
            "forbidden_markers": [],
        }

        self.assertFalse(capture_gate.guest_matches("m4", "test-m4-copy", record))

    def test_m4_case_requires_its_specific_markers(self):
        result = {
            "raw_qemu_exit": 33,
            "stdout": "\n".join(("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK",
                                 "M4:CASE:COPY:PASS", "M4:SURVIVOR_PROGRESS:PASS",
                                 "M4:CLEANUP:PASS", "KS-09:PASS", "M4:PASS", "M1:PASS")),
            "timed_out": False,
            "required_markers": ["M1:PASS"],
            "expected_exit": 33,
            "forbidden_markers": [],
        }

        self.assertTrue(capture_gate.guest_matches("m4", "test-m4-copy", result))

    def test_archive_install_error_restores_the_previous_archive(self):
        staged = self.root / "staged"
        staged.mkdir()
        (staged / "suite.json").write_text('{"passed": false}\n', encoding="utf-8")
        before = self._snapshot()
        real_replace = capture_gate.os.replace

        def fail_install(source, target):
            if Path(source) == staged and Path(target) == self.destination:
                raise OSError("synthetic archive install failure")
            return real_replace(source, target)

        with patch.object(capture_gate.os, "replace", side_effect=fail_install):
            with self.assertRaisesRegex(OSError, "synthetic archive install failure"):
                capture_gate._replace_archive(staged, self.destination)

        self.assertEqual(self._snapshot(), before)
        self.assertTrue(staged.is_dir())


if __name__ == "__main__":
    unittest.main()
